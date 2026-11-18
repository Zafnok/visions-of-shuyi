# ADR-0129: xtask is the process tool, and `trpg-tools` the game's tool program

- **Status:** Accepted
- **Date:** 2026-11-18
- **Related tickets:** 0188
- **Amends:** ADR-0089 §4 ("This game's tool program is xtask").

## Context

ADR-0089 §4 made `xtask` both the repository's process tool (the ticket
lint, `pr-size`, `engine-pin`) and the game's tool program that `cargo
shuyi` runs. So every cheap check built the whole game first: xtask
depended on `trpg-ui`, `trpg-bots`, `trpg-content`, `trpg-core`,
`trpg-writer` and the engine's `shuyi-cli`, `shuyi-core` and
`shuyi-platform` (80 crates, 2 min 21 s for a cold `cargo build -p xtask`
on Nick's machine). The commit, push and pull-request hooks run
`cargo xtask pr-size --hook`, which in a fresh worktree waited for that
build and collided with a running gate; CI's `tickets` job, which keeps no
`target/` (ticket 0171), built the game from scratch on every push to run
six checks that read only text files. The friction sweep of 2026-11-17
(`docs/research/friction-2026-11-17.md`) also counted 111 of 377 commits
touching `crates/xtask`, 12 of 23 engine-pin pull requests among them,
because the engine's API reaches it through `game_tools`.

The process files need almost none of the game: about 5,000 lines over
`serde_json`, `serde_norway` and `trpg-engine-watch` (whose only
dependency is `serde_json`). Two reached further: `content_layer.rs` used
two path constants of `trpg_content::bundle` and `shuyi_cli::Project`, and
`skill_pack.rs` the skill pack `shuyi_cli` embeds. `shuyi-cli` depends on
`shuyi-ui` and `shuyi-platform`, so it is not cheap.

## Decision

1. **Two crates.**
   - **`xtask`** (`crates/xtask`) is the process tool, and stays the name
     `cargo xtask` runs. Its commands: `ticket-lint`, `check-mentions`,
     `check-leaks`, `engine`, `engine-pin`, `clean-merged-targets`,
     `code-lines` and `pr-size`, with the repository's own tests (the
     workflow guard, every workflow parses, one integration test program
     per crate, the Markdown the code reads, the crates' lints and the
     bots' fence). Its dependencies: `trpg-engine-watch`, `serde_json`,
     `serde_norway` and `mutants`; no other `trpg-*` crate and no `shuyi-*`
     crate. A test in xtask keeps it so.
   - **`trpg-tools`** (`crates/tools`) is the game's tool program:
     `shuyi` (`game_tools`, what `cargo shuyi` runs), `playtest`,
     `skirmish`, `voice`, `lines`, `check-script`, `story-import`, the
     importers, the generators, `sfx`, `icon`, `playdate*`,
     `readme-pictures` and `skill-pack`. `shuyi.ron`'s `game` names it
     (`(package: "trpg-tools", args: ["shuyi"])`) and `.cargo/config.toml`'s
     `shuyi` alias is `run -q -p trpg-tools -- shuyi`.
2. **Old names keep working, for good.** `cargo xtask <command>` for a
   command of `trpg-tools` runs `cargo run -q -p trpg-tools -- <command>
   <args>` in the repository and answers its exit code, with
   `--features private-assets` when xtask was built with it. The names are
   `xtask::FORWARDED`; a test in `trpg-tools` keeps that list its
   commands, both ways. The forwarding stays: skills, docs, tickets and
   habits say `cargo xtask playtest`, and renaming them buys nothing. A
   forwarded command builds the game, as it always did.
3. **What the two share.** xtask has a small library (`src/lib.rs`):
   `content_layer` (the layered listing of the game's files on disk that
   `check-leaks`, `check-script` and `voice` read) and `FORWARDED`.
   `trpg-tools` depends on it. The two folders `content_layer` needs are
   written out in xtask (`assets-private/content`, `assets`), and a test in
   `trpg-tools` keeps them equal to `trpg_content::bundle`'s
   `CONTENT_DISPLAY_ROOT` and `DISPLAY_ROOT`. The project view `check-keys`
   and `lint` run on (`content_layer::project`, which needs `shuyi_cli`)
   stays in `trpg-tools`.
4. **`skill-pack` is the game's tool program's.** The pack is embedded in
   `shuyi-cli` (ADR-0097), and reading it without that crate would mean
   reading the engine's sources out of Cargo's git checkout. `cargo xtask
   skill-pack [--check]` still works, through the forwarding.
5. **CI.** The `tickets` job runs only xtask's checks (`pr-size`, the
   skill loaders, `ticket-lint`, `check-mentions`, `check-leaks`,
   `engine-pin --check`). A new job, `shuyi-checks`, runs the steps that
   need the game's tool program (`check-keys`, `skill-pack --check`,
   `check-skills`, `check-text`, `check --target all`, `app-info --check`,
   `lint`), with the same private content and no `target/` in the cache
   (ticket 0171). `ci-result` needs both. `playtest.yml` builds and runs
   `trpg-tools`.

## Consequences

- A cold `cargo build -p xtask` compiles 15 crates in about 15 seconds
  instead of 80 in 2 min 21 s, so the hooks and the `tickets` job answer
  at once in a fresh worktree; an engine-pin pull request that changes
  `game_tools` no longer touches xtask.
- `cargo xtask <game command>` runs two cargo commands, the second one
  inside the first's run; the game's build is the same one as before.
- Two lists name the forwarded commands (xtask's `FORWARDED` and
  `trpg-tools`' dispatch); the test in `trpg-tools` catches a command
  added to one and not the other.
- A future ticket that edits a game command edits `crates/tools/src/…`;
  tickets written before this one may still name `crates/xtask/src/…`
  for those files.
- **Not done here, for later:** an engine command for the ticket lint and
  `pr-size` would let the game and the engine share that code (the engine
  has its own). That is an engine change, so it needs
  Nick's OK and an engine ticket first; nothing is filed.

## Alternatives considered

- **Keep one crate and cache CI's build** — loses on ADR-0117 §4: a job
  that checks out the private content keeps no `target/`, and the local
  hooks in a fresh worktree would still wait for the game.
- **A `game` feature on xtask** that turns the game's commands on — every
  `cargo xtask` call names features or builds the default; a feature that
  pulls the game in is the build we are avoiding, one flag away, and the
  engine-pin churn stays in the same crate.
- **Copy `content_layer`'s listing into both crates** — two copies of the
  same 50 lines that must agree; the small library costs less.
- **Read the skill pack from Cargo's git checkout of the engine** so
  `skill-pack` could stay in xtask — it depends on where Cargo keeps its
  checkouts, and the local checkout (`cargo xtask engine`) is a patch;
  `shuyi-cli` already knows which pack is pinned.
- **Rename every `cargo xtask <game command>` to `cargo run -p
  trpg-tools`** in skills and docs — churn across dozens of files for no
  gain while the forwarding works.
