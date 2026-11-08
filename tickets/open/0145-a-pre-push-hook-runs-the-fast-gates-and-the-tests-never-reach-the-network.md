---
id: "0145"
title: A pre-push hook runs the fast gates, and the tests never reach the network
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
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

## Nick input

None.

## Scope

**In:**
- A tracked hooks folder, `scripts/git-hooks/pre-push`, set up with `git
  config core.hooksPath scripts/git-hooks` (in `AGENTS.md`'s environment
  section and the setup scripts). It runs, in seconds:
  - `cargo fmt --all --check`;
  - `typos`;
  - `cargo xtask ticket-lint --pr-branch <current>`;
  - `cargo xtask check-mentions` on the last commit's subject.
  
  It works in every harness, because git runs it.
- `run-gates` prints a summary with `HEAD`'s commit at the end, which is
  pasted into the PR body, so "all gates pass" names the tree it was run
  on.
- The xtask and crate tests run hermetic. CI's `test` job, and a
  `run-gates` line for local runs, set `GH_CONFIG_DIR` to an empty
  folder and unset `GH_TOKEN`. A test that needs `gh` uses a fake on
  `PATH` or is marked as network-only and run apart.

**Out (do not do):**
- Running mutants locally (never on Nick's machine).
- The CI noise fixes (0150).

## Implementation steps

1. Write `scripts/git-hooks/pre-push` (POSIX sh, run by Git Bash on
   Windows).
2. Add the env lines to the `test` job in `.github/workflows/ci.yml`, and
   find any other test that shells out to `gh` or `git ls-remote`
   (`grep -rn '"gh"' crates`).
3. Add the summary to `run-gates`, then run `pwsh -File
   scripts/sync-agent-skills.ps1`.

## Acceptance criteria

- [ ] With the hook set up, a push with unformatted Rust or a typo is
      refused.
- [ ] `cargo test --workspace` with `GH_CONFIG_DIR` empty and no
      `GH_TOKEN` passes locally.
- [ ] `run-gates` ends with a summary naming `HEAD`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Manual: the hook refusing a push, recorded in the notes.
- The existing tests, run hermetic.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
