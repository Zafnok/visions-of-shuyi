---
id: "0188"
title: The process checks build in seconds, because xtask no longer builds the game
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
nick_input: none
completed: 2026-11-18
---

# 0188 — The process checks build in seconds, because xtask no longer builds the game

## Context

The [friction sweep of 2026-11-17](../../docs/research/friction-2026-11-17.md)
(*Processes that slow us down*, "xtask is one crate, so cheap checks build
the whole game"; *Recommended order* item 9) found that every cheap
repository check compiles the whole game first:

- `crates/xtask/Cargo.toml` depends on `trpg-ui` (feature `harness`),
  `trpg-bots`, `trpg-content`, `trpg-core`, `trpg-writer`,
  `trpg-engine-watch`, `shuyi-cli`, `shuyi-core`, `shuyi-platform`
  (`harness`), `shuyi-assets` (`png`) and `gif`. The crate is 27,226
  lines in 52 files.
- `.claude/settings.json` runs `cargo xtask pr-size --hook` in 6 hooks
  (every commit, push and `gh pr create`). In a fresh worktree each one
  waits for a full game build, which collides with a running gate (one
  cargo at a time per worktree).
- `.github/workflows/ci.yml`'s `tickets` job (line 347) has
  `cache-targets: false` since #741 (ticket 0171, line 375), so every
  push compiles the game from scratch to run `pr-size`, `ticket-lint`,
  `check-mentions`, `check-leaks`, `engine-pin --check` and
  `skill-pack --check`.
- 111 of 377 commits since 2026-10-15 touched `crates/xtask`, and 12 of
  23 engine-pin PRs had to edit it, because the engine's API reaches it
  through `game_tools`.

The process files need almost none of that: `tickets.rs` (984 lines),
`pr_size.rs` (649), `code_lines.rs` (579), `mentions.rs` (94),
`leaks.rs` (490), `content_layer.rs` (192), `engine.rs` (683, on
`trpg-engine-watch`, whose only dependency is `serde_json`),
`skill_pack.rs` (244), `clean_targets.rs` (1,019) and
`workflow_guard.rs` (323): about 5,250 lines. Two of them reach heavier
crates: `skill_pack.rs` uses `shuyi_cli::skills::Pack`, and
`content_layer.rs` uses `shuyi_cli::Project` and two path constants from
`trpg_content::bundle`.

**Caveat:** the same `tickets` job also runs `cargo shuyi check-keys`,
`check-skills`, `check-text`, `check --target all`, `app-info --check`
and `lint`. `cargo shuyi` is `cargo xtask shuyi` (`.cargo/config.toml`
line 13), and `shuyi.ron` line 31 names `game: (package: "xtask")`, so
those still need the game's tool program. A light process crate alone
makes the commit hook and the process steps fast; the `shuyi` steps stay
a full build, in a job of their own.

This bucket of the sweep (process and tooling): this ticket, 0189 (a
rules-text budget), 0190 (retire the Sonar issue lane), 0191 (delete the
frozen one-shot generators), and the amended 0145 (`cargo xtask gates`),
0147 and 0181 (port the engine's ticket lint), with 0151, 1174, 1257 and
1239 folding into 0181.

Related, either order:

- 0174 (a push runs only the CI its change can break) trims which jobs
  run; this ticket makes the cheapest one cheap. If 0174 lands first,
  split the job it leaves.
- Tickets that edit files this one moves: 0145, 0147, 0181, 1172 and
  1235. If this lands first, its PR updates the paths those tickets
  name (`grep -rln "crates/xtask/src/<file>" tickets/open`). If they land
  first, move their changes with the files.
- 0191 deletes generators from `crates/xtask`. Either order: fewer files
  to move if it lands first.

## Nick input

None.

## Scope

**In:**
- Write an ADR with the `write-adr` skill (the next free number) that
  amends ADR-0089 §4 ("This game's tool program is xtask"): which crate
  is the process tool and which is the game's tool program, which
  commands go where, and how old command names keep working.
- Two crates:
  - **`xtask`** stays the name `cargo xtask` runs, and holds only
    repository process: `ticket-lint`, `check-mentions`, `check-leaks`,
    `engine`, `engine-pin`, `skill-pack`, `clean-merged-targets`,
    `code-lines`, `pr-size`, and the workflow guard tests. Its
    dependencies: no `trpg-*` crate except `trpg-engine-watch`, no
    `shuyi-*` crate unless measured cheap (below).
  - **`trpg-tools`** (new, `crates/tools`) is the game's tool program:
    `shuyi` (`game_tools`), `playtest`, `skirmish`, `voice`, `lines`,
    `check-script`, `story-import`, the importers, `sfx`, the generators,
    `playdate*` and `readme-pictures`. `shuyi.ron`'s `game:` names it,
    and `.cargo/config.toml`'s `shuyi` alias runs it.
- Old names keep working: `cargo xtask <game command>` forwards to
  `trpg-tools` (for example by running `cargo run -q -p trpg-tools --`
  with the same arguments), so no skill, workflow or doc has to be
  renamed in this PR. The ADR says whether that forwarding stays.
- `ci.yml`: the `tickets` job runs only the light `xtask` steps. A new
  job (`shuyi-checks`, or the name the ADR picks) runs the `cargo shuyi`
  steps on the tool program, with the same `cache-targets: false` as
  today (0171). `ci-result` needs both.

**Out (do not do):**
- Changing what any check checks, or its output.
- Making ticket lint or pr-size an engine command (an engine step that
  needs Nick's OK first: see *Implementation steps* 1, last bullet).
- The CI triggers (0174) and the pre-push hook (0145).
- Renaming `cargo xtask …` in skills and docs.

## Implementation steps

1. **Measure first.** With `cargo tree -p xtask -e normal`, list what
   `skill_pack.rs` and `content_layer.rs` pull in through `shuyi-cli`
   and `trpg-content`. If `shuyi-cli` brings in `shuyi-ui` and
   `shuyi-platform` (it depends on both in `Cargo.lock`), do not depend
   on it from the light crate:
   - `content_layer.rs`: copy the two `trpg_content::bundle` constants
     with a test in `trpg-tools` that they still match, and read
     `shuyi.ron`'s few fields with `ron` directly, or keep `check-leaks`
     in `trpg-tools` and say so in the ADR.
   - `skill_pack.rs`: the same choice for `shuyi_cli::skills::Pack`.
   - Note in the ADR (not a ticket) that an engine command for the
     ticket lint and pr-size would let both repositories share the code:
     an engine ticket only with Nick's OK.
2. `git mv` the process files into a fresh `crates/xtask/src/` and the
   rest into `crates/tools/src/`, keeping each file's history. Split
   `main.rs`'s dispatch and `USAGE` the same way, and
   `crates/xtask/tests/cli.rs` by command.
3. Add the forwarding arm to the light `xtask`'s dispatch for every
   command that moved, with a test that `cargo xtask playtest --help`
   still answers.
4. `shuyi.ron` line 31: `game: (package: "trpg-tools", args: ["shuyi"])`.
   `.cargo/config.toml`: `shuyi = "run -q -p trpg-tools -- shuyi"`.
   Check every workflow that builds or runs xtask for a game command
   (`playtest.yml` builds `-p xtask` at line 130 and runs
   `target/release/xtask` from line 178) and point it at `trpg-tools`.
5. Split the `tickets` job in `ci.yml` as in *Scope*, and keep
   `mutants.yml`'s workspace list right (it excludes some packages by
   name).
6. Update `AGENTS.md`'s map row for `crates/` and `xtask`, ADR-0089's
   status line, and `docs/adr/README.md`.
7. Time `cargo build -p xtask` cold in the PR and in CI's `tickets` job,
   before and after, and put both in the Completion notes.

Expected size: about 300 lines of new Rust (two dispatches, the
forwarding, the copied constants and their test), 100 of CI and config,
about 150 of ADR and docs. The moves are renames; if `pr-size` counts a
moved file as changed lines and the total passes about 2,000, split off
step 5 and the CI job as a second ticket blocked by this one.

## Acceptance criteria

- [x] `cargo tree -p xtask -e normal` names no `trpg-ui`, `trpg-bots`,
      `trpg-content`, `trpg-core` or `trpg-writer`.
- [x] `cargo xtask ticket-lint`, `pr-size`, `check-mentions`,
      `engine-pin --check --offline` and `skill-pack --check` pass from a
      clean `target/`, and the Completion notes give the cold build time
      before and after.
- [x] `cargo xtask playtest --help` and `cargo shuyi check-keys` still
      work.
- [x] CI's `tickets` job no longer builds the game (its log shows no
      `Compiling trpg-ui`), and the `cargo shuyi` steps run in their own
      job.
- [x] The ADR is written and listed in `docs/adr/README.md`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: every existing test moves with its file and still passes.
- Integration: `crates/xtask/tests/cli.rs` and its `trpg-tools` twin
  cover the forwarding (one moved command answers through `xtask`).
- Unit: if the `trpg_content::bundle` constants are copied, a test in
  `trpg-tools` that the copies match.

## Completion notes

- **Two crates** (ADR-0129, amending ADR-0089 §4). `xtask` keeps
  `ticket-lint`, `check-mentions`, `check-leaks`, `engine`, `engine-pin`,
  `clean-merged-targets`, `code-lines`, `pr-size` and the repository's
  own tests (workflow guard, workflow YAML, Markdown read by code, one
  test program per crate, lints, the bots' fence); its dependencies are
  `trpg-engine-watch`, `serde_json`, `serde_norway` and `mutants`, and a
  test keeps out every other `trpg-*` and `shuyi-*` crate. `trpg-tools`
  (`crates/tools`) has the rest, moved with `git mv` (history kept), its
  insta snapshot renamed for the new crate name.
- **Measured first:** `shuyi-cli` brings in `shuyi-ui` and
  `shuyi-platform`, so the light crate doesn't depend on it.
  `content_layer`'s file listing went to a small library in xtask
  (`xtask::content_layer`, with the two folders written out) that
  `check-leaks` and `trpg-tools` share, and a `trpg-tools` test keeps the
  two folders equal to `trpg_content::bundle`'s. **`skill-pack` stays in
  `trpg-tools`** (deviation the ticket allowed): the pack is embedded in
  `shuyi-cli`. `cargo xtask skill-pack --check` works through the
  forwarding, and CI runs it in the new job.
- **Forwarding, for good:** `cargo xtask <moved command>` runs
  `cargo run -q -p trpg-tools -- <command> <args>`, with `--release` and
  `--features private-assets` when xtask was built that way (so the
  docs' `cargo run --release -p xtask -- playtest …` still plays an
  optimized build). The names are `xtask::FORWARDED`; a `trpg-tools` test
  checks the list against its own usage both ways. The usage text now
  lists `icon`, `skirmish`, `check-script` and `voice`, which it lacked.
- **Config and CI:** `shuyi.ron`'s `game` and the `shuyi` alias name
  `trpg-tools`. The `tickets` job runs only xtask's checks; the new
  `shuyi-checks` job runs `check-keys`, `skill-pack --check`,
  `check-skills`, `check-text`, `check --target all`, `app-info --check`
  and `lint` with the same content checkout and no cached `target/`, and
  `ci-result` needs it. `playtest.yml` builds and uploads `trpg-tools`
  (the workflow guard's allowed upload follows), `playtest-levers.sh`
  and `changes.yml` name the moved files, `.cargo/mutants.toml`'s comment
  too. `mutants.yml` excludes packages by name only for `trpg-app` and
  `trpg-story-preview`, so it needed nothing.
- **Paths:** the open tickets and live docs that named a moved file
  (`crates/xtask/src/<file>`) now name `crates/tools/src/<file>`; a
  mention of `main.rs` was left, since both crates have one.
- **Cold build of `cargo build -p xtask`** (Nick's machine, empty
  `target/`, debug): before, 80 crates in 2 min 21 s; after, 15 crates in
  15 s. CI's `tickets` job: on `main` before, its first cargo step
  (`ticket-lint`, which built the game) took 1 min 46 s of the job's
  3 min 05 s; on this PR (09e61a63), its first cargo step (`pr-size`)
  took 7 s and the job 1 min 17 s. The game's build moved to
  `shuyi-checks` (`check-keys`, its first step: 1 min 45 s of 2 min 27 s).
- **Friction:** estimate 250 production lines, measured 700 added
  (2.8×), accepted: 573 of them are `main.rs`'s game commands moved to
  `trpg-tools`' new `main.rs` (the removed column), about 130 are new.
  Logged in `docs/engine/friction.md`.
- **PR size:** about 3,100 changed lines, over the ticket's 2,000. Almost
  2,100 of them are `main.rs`'s game half moving to a new file, which git
  counts as removed and added. Splitting off the CI job (step 5, about
  60 lines) would not bring it under, so it ships whole.
- **Not filed:** an engine command for the ticket lint and `pr-size`
  (ADR-0129, *Consequences*): an engine step that needs Nick's OK first.
