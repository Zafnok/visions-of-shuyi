---
id: "1131"
title: "Move the carved engine code to the Shuyi Engine repository with its history; the game pins it by revision"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: done
blocked_by: ["1118"]
nick_input: none
completed: 2026-10-19
---

# 1131 — Move the engine code to Shuyi Engine

## Context

ADR-0063 §3 and §8. The engine's repository,
[`Zafnok/shuyi-engine`](https://github.com/Zafnok/shuyi-engine), has
existed since 2026-10-14. It is public, dual-licensed MIT OR
Apache-2.0, and holds the engine's ADRs, tickets, decisions and skills.
The code is carved in this repository's `engine/` folder (1102–1118).
When the first example runs on engine crates alone (1118), the code moves
there with its history, and the game uses it like any other dependency.

Moving the code is what makes it MIT OR Apache-2.0: Nick owns it and
chose that license (the engine repository's `docs/decisions.md`). This
repository's `LICENSE` is not changed.

The engine repository's MVP tickets wait for this one (`game#1131` in
their `blocked_by`): 2002, 2201, 1129, 1133, 1134 and 1132.

## Nick input

None. The repository exists, and Nick's `gh` login can push to it.

## Scope

**In:**
- **Measure first.** Over the last 30 days, how many merged PRs here
  changed a file under `engine/`? Use `gh pr list --state merged
  --search "merged:>=<date>"` and `gh pr view --json files`. Write the
  count in the Completion notes. If it is 1 in 5 or more, also do the
  local-checkout step below (ADR-0063 §8).
- **Move with history.**
  1. Run `git filter-repo --subdirectory-filter engine` on a fresh
     clone.
  2. Push the result to a branch of the engine repository.
  3. Merge that branch into the engine repository's `main` with
     `--allow-unrelated-histories`, through a PR there.
  4. Resolve any clash with the files already there (`README.md`,
     `docs/`): the engine repository's versions win, and the moved
     files' content is merged into them.
- **CI for the engine repository.** Add the code gates to the CI that
  engine ticket 2002 started:
  - fmt;
  - clippy pedantic;
  - tests and docs;
  - the web build of the examples;
  - `cargo deny` and machete;
  - the mutation gate in shards (ADR-0073);
  - Pages for the examples.

  Update its `run-gates` skill.
- **This repository:**
  - Delete `engine/`.
  - Make the game's crates depend on the engine by git URL and revision.
    `shuyi-engine.rev` holds the pinned revision (like
    `assets-private.rev`, ADR-0040), and `cargo xtask engine-pin`
    updates it.
  - Retire `check-engine-boundary` (1102): the boundary is now a
    repository wall.
  - Update `CLAUDE.md` and the `work-ticket` skill: a ticket that needs
    an engine change is an engine PR first, then a game PR that moves
    the pin. One line each: 1152 turns it into the `engine-change`
    skill and teaches `ticket-lint` the `engine#NNNN` ids.
- **The local checkout** (only if the measure says so):
  - `cargo xtask engine` checks the engine repository out at the pinned
    revision into a git-ignored `shuyi-engine/` folder.
  - A Cargo `[patch]` in `.cargo/config.toml` points the game at that
    folder, so one working copy edits both.
  - CI never uses the patch.

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6).

**Out (do not do):**
- the engine's MVP tickets (they are the engine repository's);
- publishing to crates.io. Write an engine ticket for the first release
  if none exists.

## Implementation steps

1. Measure.
2. Filter, push, and open the merge PR in the engine repository. Its CI
   must be green.
3. Switch the game to the pinned dependency and delete `engine/`. The
   game's gates must pass.
4. Add the local checkout if needed.
5. Update `CLAUDE.md`, the skills, `docs/engine/README.md`, and
   ADR-0063's status line ("code moved on …").

## Acceptance criteria

- [x] The engine repository's CI is green with the code, and history is
      kept (`git log` on a moved file shows commits from before the
      move).
- [x] The game builds from the pinned revision, native and web, and its
      gates pass.
- [x] `shuyi-engine.rev` and `cargo xtask engine-pin` work.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None beyond both repositories' gates.

## Completion notes

**Done.** The engine's code is in
[`Zafnok/shuyi-engine`](https://github.com/Zafnok/shuyi-engine) with its
history, through its PR
[Zafnok/shuyi-engine#6](https://github.com/Zafnok/shuyi-engine/pull/6),
and the game builds on it pinned by revision.

**The measure.** In the 30 days before the move (from 2026-09-19), 24 of
the 332 PRs merged here changed a file under `engine/`: 1 in 14, under
the 1 in 5 of ADR-0063 §8. But `engine/` only existed from 2026-10-14
(1102, PR 261): of the 83 PRs merged since then, 24 changed it (29%, more
than 1 in 5). The 30 days counted mostly time when no PR could touch it,
so I went by what the rule asks ("is the API still moving?") and set up
the local checkout too. (PR 325's 167 files were counted in full: `gh pr
list` lists only 100 per PR.)

**The move** (engine PR #6):
- `git filter-repo --subdirectory-filter engine` on a fresh clone of
  `main` (single branch, no tags, so the game's release tags stayed here):
  24 commits, 269 files. The commit messages' `(#NNN)` and "PRs #323 and
  #324" became `Zafnok/visions-of-shuyi#NNN`, so they don't link to the
  engine repository's own PRs, and the dialogue directives in two
  messages (`@if`, `@place`, `@opening`) got backticks so GitHub doesn't
  read them as mentions. History is kept: `git log
  crates/core/src/rng.rs` there reaches the 1114 commit that carved it.
- Merged beside the plans with `--allow-unrelated-histories`; only
  `README.md` clashed. The engine repository's version won, with the
  crate table and the Hello, Grid section merged in; the moved README's
  record of what each carve ticket moved is its `docs/carve.md`.
- A workspace of its own: `Cargo.toml` (version 0.1.0; every crate
  `license = "MIT OR Apache-2.0"` instead of the game's `license-file`),
  this repository's `Cargo.lock` pruned to the engine's 148 packages (the
  versions the game tested), the toolchain, fmt and clippy configs,
  `deny.toml` (which checks the engine's own crates' licenses too), the
  mutants and nextest configs. The test that kept `shuyi-app`'s copied
  lints equal to the workspace's moved from our xtask into `shuyi-app`.
- The moved comments named this repository's ADRs as `ADR-NNNN`, which
  there means the engine's: they say `game ADR-NNNN` now, in comments and
  Markdown only (error texts that name an ADR, such as `credits.rs`'s
  "ADR-0013", are unchanged, because tests here and there read them).
  Paths lost `engine/`.
- **CI** (`ci.yml`): fmt, clippy pedantic, tests on Windows, Linux and
  macOS, docs, Hello, Grid's web build, the no-std build of `shuyi-core`,
  `shuyi-assets` and `shuyi-ui`, `cargo deny`, machete, one `ci-result`.
  `mutants.yml` is our sharded gate (ADR-0073) and the weekly full run;
  `pages.yml` publishes Hello, Grid at `/hello-grid/`. Its `run-gates`
  skill lists them. `THIRD_PARTY_ASSETS.md` there lists the font, the web
  shell's JavaScript and the controller database, each with its license
  text (the database's text was copied from `crates/app/`).
- Engine tickets written: **2005** (the first release on crates.io, and
  the game on versions; none existed) and **2006** (`check-keys` and
  `check-text` on the engine's own code: our `shuyi.ron` never scanned
  `engine/`, and nothing scans it there yet).

**This repository:**
- `engine/` is deleted. The five engine crates are in
  `[workspace.dependencies]` by git URL and `rev`, with default features
  off for the three the Playdate builds without `std`; each crate takes
  them with `workspace = true` and the features it used (`std`, `png`,
  `sample-actions`). `Cargo.lock` changed only for the engine packages.
- `shuyi-engine.rev` names the pin. `cargo xtask engine-pin <commit>`
  moves it, `Cargo.toml` and `Cargo.lock` together (with no commit, the
  one the local checkout is at, once it is pushed), and `engine-pin
  --check` fails when they disagree, or when the lock names a local
  checkout. It replaces `check-engine-boundary` in CI's `tickets` job;
  `check-engine-boundary` (1102) is retired with its module.
- **The local checkout.** `cargo xtask engine` clones the engine at the pin
  into `shuyi-engine/` (an existing checkout is fetched and never moved:
  it may hold engine work) and writes `.cargo/engine-patch.toml`, the
  `[patch]`. `.cargo/config.toml` includes it only when it exists
  (`include = [{ path = …, optional = true }]`), and both files are
  git-ignored, so CI never uses the patch. `cargo xtask engine --off` takes
  it away and writes the lock for the pin again. `Cargo.toml` excludes
  `shuyi-engine` from the workspace, or Cargo makes its crates members of
  ours.
- `cargo shuyi` is now `cargo xtask shuyi` (a Cargo alias of an alias):
  Cargo can't `cargo run -p` a dependency, and xtask already had the
  game's tools for `frame-png` and `lang-status`.
- CI: Hello, Grid's web build and the engine crates' own no-std builds
  left (the engine's CI has them; ours still builds the engine crates
  without `std` under `trpg-core`, `trpg-content` and `trpg-ui`);
  `--exclude shuyi-app` left coverage and the mutants baseline; Sonar and
  `.cargo/mutants.toml` stopped naming `engine/`. Pages no longer
  publishes `/engine/hello-grid/`: the engine's Pages does.
- The release's web package copied macroquad's license text from
  `engine/`. It now downloads it from the engine at the pinned commit,
  with the two others the web shell ships, `quad-storage.js`'s and
  `sapp_jsutils.js`'s, which the package was missing.
- `THIRD_PARTY_ASSETS.md`: the web shell's three files point at the
  engine's repository; Hello, Grid's font row went there.
- Docs: `CLAUDE.md` (rule 1 and the map), the `work-ticket`,
  `run-gates`, `keyboard-input` and `engine-friction` skills,
  `docs/engine/README.md` (how the two repositories work together),
  ADR-0063's status line and the ADR index, the roadmap and the README.

**Deviations from the plan:**
- Engine ticket 2002 hadn't started the engine's CI, so this ticket wrote
  `ci.yml` with the code gates; 2002 now only adds its three checks (its
  text says so).
- The `[patch]` is in `.cargo/engine-patch.toml`, included by
  `.cargo/config.toml`, not in `config.toml` itself: a patch there would
  be in every checkout and in CI. I tried Cargo's `paths` overrides
  first, which leave `Cargo.lock` alone, but Cargo warns on each engine
  crate that they're "known to produce buggy behavior" and will become an
  error.
- For the move PR itself, the engine's mutation gate diffs from the
  carved history's tip (the code as it left here, where each line passed
  this gate PR by PR), not from an engine `main` with no code: 4 mutants
  instead of all 7,607, which no shard could finish (our own weekly full
  run hits GitHub's 6-hour limit). The condition is inert once `main`
  holds the carved history (`.github/mutants-diff.sh` there).

No gameplay rule was decided: nothing in the game changes.

**Engine friction:** estimate 220 production lines (`engine.rs` like
`shuyi-cli`'s `private_assets.rs` fetch and pin, about 250 lines, plus
the dispatch), measured 328 added in `crates/xtask` (1.5×): under 2×.

**For Nick:**
- Merge the engine's PR #6 **first**, with **"Create a merge commit"**
  (not squash or rebase): a squash would flatten the history this ticket
  keeps. The pin is that PR's last commit, which stays in `main`'s history
  after a merge commit.
- Turn Pages on there: the engine repository's Settings → Pages →
  Source: GitHub Actions. Until then its Pages deploy fails; nothing else
  does.
- Nothing to play: the game is the same game.
