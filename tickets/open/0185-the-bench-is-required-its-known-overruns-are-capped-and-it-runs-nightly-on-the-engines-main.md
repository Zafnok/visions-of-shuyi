---
id: "0185"
title: The bench is required, its known overruns are capped, and it runs nightly on the engine's main
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: high
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0185 — The bench is required, its known overruns are capped, and it runs nightly on the engine's main

## Context

From the retro of PRs #616–#735 (item T7,
[report](../../docs/research/retro-2026-11-17-prs-616-735.md)). CI's
`bench` job (ticket 1244, `docs/targets.md`) is not in `ci-result`'s
`needs`, so it never blocks a merge. Its `KNOWN_OVER` list
(`scripts/bench-known-over.sh`) turns any value of a listed key into a
warning, with no cap, for 4 of the 6 Playdate scenarios. #732 (a pin
move) put `battle-cursor-sweep` back over budget a week after #630
fixed it, and nothing went red. No job benches the engine's `main`
before a pin move. The web budget is measured on the `export` job's
`--without private-assets` package, with none of a release's content
or art.

1250 waited for every overrun to be fixed before making the job
required. Of its blockers only 1248 is still open. This ticket makes
the job required now, with each known overrun capped. Bringing the
cursor sweep back under is 1285.

## Nick input

None. Verified by unit tests and by CI going red on a capped overrun
(below). Nothing here is judged by playing.

## Scope

**In:**
- `bench` required; `KNOWN_OVER` becomes a file of capped lines, each
  with an open ticket (steps 1–3).
- The web package budget measured on a content-and-art build that is
  never uploaded or cached (step 4).
- A nightly bench of the game on the engine's `main` (step 5).
- The rule in two skills and `docs/targets.md` (step 6).

**Out (do not do):**
- Bringing any number under its budget: the cursor sweep is 1285, the
  one-bit drawing 1248, where a battle frame's time goes 1057.
- Caching `BattleScreen`'s panel view or dropping the Glyph-only skip of
  #720 (commit 6be0f11e). That is Playdate frame work for the 1057/1248
  lane.
- Raising a budget. Fewer WASM builds per push in general (0174, which
  says what to do if this ticket lands first).
  What CI caches (0171; this ticket adds nothing to the caches).
- Deleting `bench/known-over.txt` and its handling once 1248 and 1285
  have emptied it (1250, narrowed to that).

## Implementation steps

Expected size: about 500 lines (about 250 of code and workflow, 180 of
tests, the rest docs).

1. **The gate in xtask.** Add `crates/xtask/src/bench_gate.rs` and a
   `bench-gate` command in `dispatch` (`crates/xtask/src/main.rs`) and
   `USAGE`. `cargo xtask bench-gate <exit code> <log> <report.json>`
   does what `scripts/bench-known-over.sh` does today. Keep its key
   format (`target/what/scenario`, or `target/package`) and its
   `error[SH` check. It reads the known list from `bench/known-over.txt`.
   Each line is `<key> <recorded value> <ticket id>`, and `#` starts a
   comment. Check that `cargo shuyi bench` ignores a file in `bench/`
   that isn't `*.bench.ron`; if it doesn't, put the list next to it in
   `bench-known-over.txt` at the root. Rules:
   - an over-budget key not in the list: error;
   - a listed key above `recorded × 1.02`: error naming both numbers;
   - a listed key whose ticket has no file in `tickets/open/`: error;
   - a listed key at or under its recorded value: warning naming the
     ticket;
   - a listed key no longer over budget: notice "take the line out".
   Delete `scripts/bench-known-over.sh`.
2. **The list.** Write `bench/known-over.txt` from the four keys in the
   `Bench` step's `KNOWN_OVER` (`ci.yml`, the `bench` job). Use the values
   of the latest `bench-report` artifact on `main`, with tickets 1248
   (three keys) and 1285 (`battle-cursor-sweep`). If 1285 is done first,
   leave its key out. Point the `Bench` step at `cargo xtask bench-gate`
   and drop the env var.
3. **Required.** Add `bench` to `ci-result`'s `needs` and rewrite the
   comments above the `bench` and `ci-result` jobs.
4. **Web with content.** Move the `export` job's steps into the `bench`
   job, before `Bench`: the licenses report, Binaryen and
   `cargo shuyi export web --release`. Set the job's `private-assets` step
   to `art: "true"` and pass its `export-flag` output, as
   `release.yml`'s `Export` step does (give the step `id: private`).
   Then delete the `export` job, its
   `export-web` artifact and the download step, and drop `export` from
   `ci-result`'s `needs`.
   Build the export with `CARGO_TARGET_DIR=$RUNNER_TEMP/export-target`, so
   `rust-cache` never saves it. If `cargo shuyi export` ignores
   `CARGO_TARGET_DIR`, delete its `target/wasm32-unknown-unknown` output
   in a final `if: always()` step instead. Check that the uploaded
   `dist/bench` holds only the report. Record the job's minutes before and
   after in the PR body.
5. **Nightly on the engine's `main`.** Add
   `.github/workflows/engine-bench.yml`: a `schedule` (once a day, off the
   hour) and `workflow_dispatch`, with `permissions: contents: read,
   issues: write`.
   - Stop early if the engine's `main` (`gh api
     repos/Zafnok/shuyi-engine/commits/main --jq .sha`) is the pin. Else
     check out the content, install Valgrind as `bench` does, and run
     `cargo xtask engine` (clones the pin, writes the patch).
   - Run `cargo shuyi bench --out dist/bench-pin`. Then
     `git -C shuyi-engine checkout --detach origin/main`, and run the bench
     again into `dist/bench-main`. Skip the package if the bench has a
     flag for it; otherwise ignore package rows.
   - No `rust-cache`, and no artifact other than the two reports.
   - Add `cargo xtask bench-gate compare <pin.json> <main.json>`. It
     prints the keys whose value rose more than 1%, or that went over a
     budget. Read the report's per-scenario shape from a recent artifact.
   - If any key moved, create or edit the one open issue labeled
     `engine-bench` (create the label if it is missing). Its body is the
     table, both SHAs, the compare link
     `https://github.com/Zafnok/shuyi-engine/compare/<pin>...<main>` and
     `git log --oneline <pin>..origin/main` (the subjects carry the engine
     PR numbers). Close the issue when nothing moved.
   - If the game doesn't build on `main`, write a notice and exit 0. API
     breaks are the `engine-pin` issue's job.
6. **Rules and docs.** In `.claude/skills/run-gates/SKILL.md` and
   `.claude/skills/engine-pin/SKILL.md`, add one rule. A new line in
   `bench/known-over.txt`, or a higher recorded value, needs an open
   ticket that brings it under, and the PR body says why. Never raise a
   budget instead. A pin move reads the open `engine-bench` issue first.
   Run `pwsh -File scripts/sync-agent-skills.ps1`. Update `docs/targets.md`
   (*Running it*, *Over budget*) for the file, the cap, the content
   package and the nightly run.

## Acceptance criteria

- [ ] `ci-result`'s `needs` has `bench` and no `export`; the `export` job
      and `scripts/bench-known-over.sh` are gone.
- [ ] On this PR, a throwaway commit that lowers one recorded value in
      `bench/known-over.txt` by 5% turns `ci-result` red. Reverting it
      turns it green; the PR body links both runs.
- [ ] The `bench` summary's web package row is measured from the
      content-and-art export, and no job uploads that package.
- [ ] A `workflow_dispatch` run of `engine-bench.yml` finishes and its
      log shows the compare output; its link is in the PR body.
- [ ] Both skills and `docs/targets.md` state the cap rule;
      `sync-agent-skills.ps1 -Check` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (`bench_gate.rs`): parsing the list (comments, a bad line), each
  of the five rules in step 1, and `compare` at 0.9%, 1.1% and a new
  overrun. Use small JSON fixtures in the test module.
- CI: the red/green proof above, and one manual run of the nightly
  workflow.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
