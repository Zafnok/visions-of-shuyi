---
id: "0169"
title: A blocked playtest battle keeps its report
type: bug
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-13
---

# 0169 — A blocked playtest battle keeps its report

## Context

Found by ticket 0168: the writer's mirror's check (`tools/story-mirror/check.yml`)
stopped before printing its report, because a GitHub Actions `run:` step
runs under `bash -e` and `set -uo pipefail` leaves `-e` on. 0168 fixed
the mirror's check.

`.github/workflows/playtest.yml`'s *Play the battle* step has the same
shape:

```bash
set -uo pipefail
target/release/xtask playtest "$BATTLE" --bot all | tee "reports/$BATTLE.txt"
echo "${PIPESTATUS[0]}" > "reports/$BATTLE.status"
```

When the battle is blocked (xtask exits non-zero), `pipefail` makes the
pipeline fail, `-e` stops the step, and `reports/$BATTLE.status` is
never written. The upload step and *Fail if the battle is blocked* are
then skipped: the job is still red, but the report artifact, the one
thing worth reading for a blocked battle, is not uploaded and the
`::error::` line naming the battle never prints.

## Nick input

None.

## Scope

**In:**
- The *Play the battle* step of `.github/workflows/playtest.yml`.
- A look through the other workflows for a step that relies on an exit
  code after a failing command under the default `bash -e` shell.

**Out (do not do):**
- The mirror's `check.yml` (0168 fixed it).
- Any change to what the playtest checks.

## Implementation steps

1. In *Play the battle*, catch the status without stopping the step, e.g.
   `set -euo pipefail`, `status=0`, the pipeline `|| status=$?` (with
   `pipefail` this is xtask's code, `tee` doesn't fail), then
   `echo "$status" > "reports/$BATTLE.status"`.
2. `grep -n "PIPESTATUS\|code=\$?\|status=\$?" .github/workflows/*.yml`
   and fix any other step of the same shape the same way.
3. zizmor and actionlint stay clean (the `run-gates` skill).

## Acceptance criteria

- [x] A blocked battle's run uploads `playtest-report-<battle>` and prints
      the `::error::<battle>: blocked …` line (shown with a run link, e.g.
      a throwaway branch whose battle is made unwinnable, or reasoned from
      the step's script if no such battle is at hand: said so).
- [x] A winnable battle's run is unchanged (green).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- By hand, in the completion notes: the runs above, with links.

## Completion notes

- *Play the battle* now runs under `set -euo pipefail` and catches the
  playtest's code with `status=0` and `... | tee ... || status=$?`, then
  writes it to `reports/$BATTLE.status`. With `pipefail` the pipeline's
  code is xtask's (`tee` doesn't fail), and the `||` keeps `-e` (ours and
  the runner's `bash -e`) from ending the step, so the report is uploaded
  and *Fail if the battle is blocked* prints the `::error::` line.
- Other workflows: the only other exit-code capture
  (`grep -rn 'PIPESTATUS\|$?' .github/ scripts/`) is ci.yml's *Bench*
  step, which already wraps it in `set +e` ... `set -e`. Nothing else to
  fix.
- Blocked battle: reasoned from the script, no throwaway run (every
  battle plays in a throwaway branch's run, about 14 jobs up to 30
  minutes each, to show one shell line). The same lines were run locally
  under `bash -e` with a command that prints a line and exits 3: the step
  exited 0, the status file held 3 and the report held the line.
- Winnable battles: this PR changes the playtest, a lever, so its own
  Playtest run plays every battle; it stays green (the PR's checks).
- Gates: zizmor (`--min-severity medium`) and actionlint clean on
  `playtest.yml`; no Rust changed.
- Engine friction: none: no code. No follow-up tickets.
