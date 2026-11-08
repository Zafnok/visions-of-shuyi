---
id: "1257"
title: The friction log keeps its numbers, and a diff into code an engine ticket replaces is flagged
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1257 — The friction log keeps its numbers, and a diff into code an engine ticket replaces is flagged

## Context

Game ADR-0092's friction review is how the game finds engine-shaped
code. The retro of 2026-11-08 found it worked once a follow-up ticket
existed: the game adopted engine pieces within 1–2 days (#518 deleted
673 lines, and #521, #529, #549, #565 and #574 adopted theirs). Before
that, it missed things:

- **#520** extended the private RON scanner to two more file kinds,
  despite the 0509 friction row's own trigger rule. Only the boundary
  audit (#536) caught it, and engine 2244 is still open.
- **Copies with a pending engine replacement kept growing.**
  - #534 and #556 grew `playdate_images.rs` after 1221 and engine 2245
    were filed.
  - #537 grew `report.rs` while engine 3090 was in flight.
  - #520, #535, #538 and #544 kept editing what 1180 replaces.
- **Rows and counts slipped.** The 0760 row has no estimate. Adoptions
  under 2× leave no trace, so the log can't show what adopting saved.
- **The boundary map is stale.** `docs/engine/boundary.md` was last
  changed on 2026-10-22, and its refresh (1233) is still open.

## Nick input

None.

## Scope

**In:**
- `cargo xtask check-friction`: every row of `docs/engine/friction.md`
  has an estimate and a measured number. A row marked adopted has the
  lines removed. It runs in the tickets job.
- A CI warning on a PR that changes a file an open `engine#`-blocked
  ticket names in its Scope or steps as the code it replaces. The
  warning names that ticket, so the session either adds to the engine
  ticket or explains in the PR why the copy must grow.
- `engine-pin` step 4: an adoption records the lines removed in the
  friction row.
- 1233's text says when it re-runs: after every boundary audit, and at
  least once per Act.

**Out (do not do):**
- The audit itself (1233).
- The engine-or-game verdict per part (0148's skill text).

## Implementation steps

1. Write the row checker over `friction.md`'s table.
2. Build an index of open tickets with `engine#` blockers and the file
   paths their text names. Warn when the PR's diff touches one.
3. Update `engine-pin`, `engine-friction` and 1233, then run `pwsh -File
   scripts/sync-agent-skills.ps1`.
4. Fill the rows the checker finds empty, from the PRs they name.

## Acceptance criteria

- [ ] `cargo xtask check-friction` fails a row with no estimate, and
      passes on the filled log.
- [ ] A diff touching `crates/playdate/src/…` named by an open
      engine-blocked ticket warns with its id (a fixture test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the row checker and the file-path index.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
