---
id: "0145"
title: A pre-push hook runs the fast gates, and the tests never reach the network
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: high
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0145 — A pre-push hook runs the fast gates, and the tests never reach the network

## Context

In the retro of game PRs #513–#580 (2026-11-08), 35 of 60 PRs were
pushed more than once after opening, and only 23 of 55 merged PRs were
green on their first push. Apart from mutants (CI only, by design), the
red jobs were mostly gates `run-gates` covers, run on a different tree
or not at all:

- **The gates weren't run before pushing:**
  - #514 failed `typos` and the `every_third_party_asset_has_a_credit`
    test;
  - #554 needed a `fmt` fix, and #572 a clippy fix.
- **`main` moved under the branch:** #533 (a duplicate id), #543 (a
  stale recorded playthrough) and #563 (snapshots).
- **A test reached the network.** In #513,
  `engine_pin_check_on_the_real_repo_exits_0` called `gh`. It passed
  locally only because `gh` was logged in, and failed `test` on all
  three systems in CI. The fix added `engine-pin --check --offline`, but
  nothing stops the next such test.

The engine's twin is engine#2054 (`cargo xtask gates` and a pre-push
hook).

2026-11-17 friction sweep
([report](../../docs/research/friction-2026-11-17.md), *Processes that
slow us down*, "Local gates are 27-30 hand-typed commands"): `run-gates`
lists about 30 commands with no subset for a docs- or ticket-only
change, and 133 of 675 commits were docs or tickets only. engine#2054 is
done at the pin (e738ffc): the engine's `crates/xtask/src/gates.rs`
(1,228 lines with its tests) reads `changes.yml`'s classifier at run
time to pick the gates a diff needs, has `--fast` (the gates that take
seconds) and `--list`, and a test holds its gate list to `ci.yml`'s
jobs. So this ticket ports it rather than designing one. The sweep
called the modes `--quick` and `--full`; use the engine's names
(`--fast`; no flag runs the diff's gates; a `--full` that runs every
gate, as CI on a code change) so later fixes port both ways. Bucket
siblings (process and tooling): 0188, 0189 (whose `run-gates` budget of
60 lines needs this command), 0190, 0191, 0147, 0181.

## Nick input

None.

## Scope

**In:**
- A tracked hooks folder, `scripts/git-hooks/pre-push`, set up with `git
  config core.hooksPath scripts/git-hooks` (in `AGENTS.md`'s environment
  section and the setup scripts; ticket 1271 made the folder, with its
  `post-checkout`, and the `AGENTS.md` line already). It runs, in seconds:
  - `cargo fmt --all --check`;
  - `typos`;
  - `cargo xtask ticket-lint --pr-branch <current>`;
  - `cargo xtask check-mentions` on the last commit's subject.
  
  It works in every harness, because git runs it.
- **`cargo xtask gates`**, ported from the engine's `gates.rs`
  (engine#2054): the game's gates in `GATES`, each naming the `ci.yml`
  job it mirrors and the `changes.yml` output its `if:` reads; `--fast`,
  `--list`, `--base <ref>` and `--full`; and the test that fails when
  `ci.yml` gains a job that is neither a gate nor a CI-only job with its
  reason (mutants, bench, the Playdate device build). The pre-push hook
  runs `cargo xtask gates --fast`. Its summary block (the engine's has
  one) is the `HEAD` summary of the next bullet.
- `run-gates` names that one command (`--full` to reproduce a CI
  failure) and keeps only what each gate means and this machine's
  environment facts, so 0189 can hold it to 60 lines.
- `run-gates` prints a summary with `HEAD`'s commit at the end, which is
  pasted into the PR body, so "all gates pass" names the tree it was run
  on.
- The xtask and crate tests run hermetic. CI's `test` job, and a
  `run-gates` line for local runs, set `GH_CONFIG_DIR` to an empty
  folder and unset `GH_TOKEN`. A test that needs `gh` uses a fake on
  `PATH` or is marked as network-only and run apart.

**Out (do not do):**
- Running mutants locally (never on Nick's machine); `gates` lists them
  as CI-only.
- Moving `gates` into the light process crate of 0188. Either order: if
  0188 lands first, put `gates.rs` in its process crate.
- Making `gates` an engine command for every game (an engine step, only
  with Nick's OK).
- The CI noise fixes (0150).

**Either order with 1423** (the engine's redundancy sweep of 2026-11-28): if 1423 is done, the engine's
`shuyi-process` already does this ticket's Rust part with the game's
config. Do only its skill and doc edits, or move it to `done/` as 1423
says. If not, port as planned, and 1423 deletes the port when the engine
ships its crate (engine#3448 and #3449, if Nick says yes).

## Implementation steps

1. Write `scripts/git-hooks/pre-push` (POSIX sh, run by Git Bash on
   Windows).
2. Port `gates.rs` from the engine at the pin (`gh api
   repos/Zafnok/shuyi-engine/contents/crates/xtask/src/gates.rs?ref=<rev>`,
   `<rev>` from `shuyi-engine.rev`):
   replace the engine's gates with the game's (the `run-gates` list and
   `ci.yml`'s jobs), keep the classifier reader and the `ci.yml` test,
   and wire `gates` into `main.rs`'s dispatch and `USAGE`. Expected
   size: about 900 lines with tests; with the rest of this ticket about
   1,300. If it passes about 2,000, split `gates` into its own ticket
   blocked by this one.
3. Add the env lines to the `test` job in `.github/workflows/ci.yml`, and
   find any other test that shells out to `gh` or `git ls-remote`
   (`grep -rn '"gh"' crates`).
4. Add the summary to `run-gates`, shorten its command list to the
   `gates` command, then run `pwsh -File
   scripts/sync-agent-skills.ps1`.

## Acceptance criteria

- [ ] With the hook set up, a push with unformatted Rust or a typo is
      refused.
- [ ] `cargo test --workspace` with `GH_CONFIG_DIR` empty and no
      `GH_TOKEN` passes locally.
- [ ] `run-gates` ends with a summary naming `HEAD`.
- [ ] `cargo xtask gates --list` on a tickets-only diff lists only the
      gates `changes.yml` runs for it; `--full --list` lists every gate.
- [ ] Adding a job to a copy of `ci.yml` fails the drift test until it
      is a gate or a named CI-only job.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Manual: the hook refusing a push, recorded in the notes.
- Unit: `gates`' classifier reading and the `ci.yml` drift test, ported
  with the engine's tests.
- The existing tests, run hermetic.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
