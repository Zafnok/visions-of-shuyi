---
id: "0176"
title: A red main opens one issue and gets one fix, and repo state turns only the tickets job red
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0176 — A red main opens one issue and gets one fix, and repo state turns only the tickets job red

## Context

From the [retro of PRs #616–#735](../../docs/research/retro-2026-11-17-prs-616-735.md)
(T12). #626 and the stacked #627 were both green, but together they left
two copies of ticket 1262. CI on `main` failed at 04:39 UTC on
2026-11-11 and was green at 05:04. In those 25 minutes, #630, #631,
#632 and #634 each deleted the same stale copy: nothing told the other
sessions that #630 was already fixing it. The one bad file also turned
`tickets`, `test` ×3 and `coverage` red, because xtask tests re-run
`tickets` job commands on the real repository (for example
`ticket_lint_succeeds_on_the_real_repo`, `crates/xtask/src/main.rs:1336`).
The mutants `baseline` job (`mutants.yml:94`) runs them too.

ADR-0042 says a red `main` is fixed first. This ticket makes that fix
happen once.

## Nick input

None. Verified by a dry run of the workflow and by the xtask tests.

## Scope

**In:**
- A `main-health` workflow that keeps one `main-red` issue open while
  `main` is red and closes it when `main` is green.
- `work-ticket`, `run-gates` and ADR-0042 say: a PR red from a red
  `main` waits for or merges the open fix PR, and doesn't fix it again.
- xtask tests that only repeat a `tickets` job command on the real
  repository go. The `tickets` job and `run-gates` still run each one.

**Out (do not do):**
- **A merge queue or "require up to date".** ADR-0042 rejected both
  ("Revisit if `main` goes red often"), and one 25-minute red is not
  often. The closed `main-red` issues become the count a later ticket
  and ADR would need.
- Two open PRs taking one ticket or ADR number: 0131 and 0144.
- Cancelled runs shown as neutral, fixed proptest seeds: 0150.
- Push runs on `main` cancelling each other: 0174 keys them by sha.
  The issue closes on the next green run.
- `the_real_repository_reads` (`crates/engine-watch/src/fetch.rs`) and
  `check_script_exits_zero_on_the_repository_scripts`: no `tickets`
  job step repeats them.

## Implementation steps

1. Add `.github/workflows/main-health.yml`. It runs on a schedule every
   10 minutes (`"7-59/10 * * * *"`) and on `workflow_dispatch` with a
   `dry_run` input that defaults to `true`, as `engine-watch.yml` does.
   Don't use `workflow_run`, which zizmor flags. Set `permissions: {}`
   at the top and give the one job `actions: read` and `issues: write`.
   It needs no checkout. Use `concurrency: main-health` with
   `cancel-in-progress: false`.
2. Its bash step uses `set -euo pipefail` and no `$?` capture. For each
   of `CI`, `CodeQL` and `Security`, take the newest completed push run
   on `main` that wasn't cancelled (`gh run list --workflow <name>
   --branch main --event push --status completed --json
   databaseId,conclusion,headSha,url`). Read its jobs with `gh run view
   <id> --json jobs`. The run is red when a required check in it
   failed: `ci-result`, `tickets`, `codeql-result`, `deny`, `machete`,
   `typos` or `zizmor` (ADR-0041). Read the jobs, not the run's
   conclusion: `bench` fails without blocking until 1250 (`ci.yml:496`).
3. If something is red and no `main-red` issue is open, open one titled
   "main is red". List each red workflow's failed job, run link, head
   commit and that commit's PR (the `(#NNN)` in its subject). End the
   body with "Fixing it? Comment here with your PR before you start."
   If the issue is already open, edit its body. If nothing is red, close
   the issue with the green runs' links. Create the label first with
   `gh label create main-red --force`. A dry run only prints.
4. In `work-ticket` §6, add to the bullet "Wait for CI. If a check
   fails…": if it fails on something the PR didn't touch, run `gh issue
   list --label main-red --state open`. If the issue names an open fix
   PR, wait for it to merge, then merge `origin/main`. If you can't
   wait, merge its branch (`git fetch origin pull/<n>/head && git merge
   FETCH_HEAD`). If it names none, comment your PR on the issue first,
   then fix it. Put one line after `run-gates`' *Order* list that points
   there. Then run `pwsh -File scripts/sync-agent-skills.ps1`.
5. In ADR-0042's Decision bullet "If `main` goes red after a merge…",
   add that the `main-red` issue (ticket 0176) names the one fix PR.
6. Delete the real-repo checks, and keep the argument and fixture
   checks that share a test with them:
   - ticket-lint: `main.rs:1336`, `tickets.rs:864`
     `real_repo_tickets_pass`, `tests/cli.rs:20`.
   - check-keys and check-text: `main.rs:1578`, `main.rs:1594` (keep
     `--bogus` → 2), `game_tools/tests.rs:492`
     `the_checks_pass_on_this_repository`.
   - engine-pin: `main.rs:1584` (keep the `check_report` and argument
     lines), `engine.rs:614` `the_real_repo_agrees_with_itself`,
     `tests/cli.rs:77`.
   - skill-pack: `skill_pack.rs:236`. **Either order with 1224:** if
     1224 is done, delete the test it moved instead.
7. Keep a success-path test for each wrapper, so mutants still fail.
   Split `ticket_lint` into `fn ticket_lint_in(root: &Path, args:
   &[String]) -> u8` and test it on a temp dir (`tickets.rs` has
   `make_temp_tickets_dir`). Do the same for any wrapper whose 0 path
   loses its only test.

## Acceptance criteria

- [ ] A dispatch dry run of `main-health` prints its decision and
      changes no issue (run link in the PR body).
- [ ] With a second copy of an open ticket (same id) in
      `tickets/open/`, `cargo test -p xtask` passes and `cargo xtask
      ticket-lint` fails with `duplicate ticket id`. The copy is then
      deleted.
- [ ] `rg -n "real_repo|on_this_repository" crates/xtask` finds no test.
- [ ] `work-ticket` and `run-gates` name the `main-red` issue, and
      `pwsh -File scripts/sync-agent-skills.ps1 -Check` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `ticket_lint_in` gives 0 on a clean fixture and 1 on a fixture
  with a duplicate id; the same for each wrapper from step 7.
- Workflow: one dispatch dry run on the branch (`gh workflow run
  main-health.yml --ref <branch>`); zizmor passes on the new file.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
