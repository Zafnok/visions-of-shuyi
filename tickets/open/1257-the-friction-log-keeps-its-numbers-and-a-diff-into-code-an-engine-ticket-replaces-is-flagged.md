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

2026-11-17 friction sweep ([report](../../docs/research/friction-2026-11-17.md), *Processes that slow us
down*): this ticket's checks fold into 0181. The row checker over
`docs/engine/friction.md` and the warning on a diff into code an
`engine#`-blocked ticket replaces run inside 0181's `cargo xtask
check-pr`, in one CI job with the checks of 0151, 1174 and 1239, not as
a `check-friction` command of their own. What stays here: `engine-pin`
step 4 (an adoption records the lines removed), 1233's text, and filling
the empty rows. If this is worked before 0181, do only those parts.

## Nick input

None.

## Scope

**In:**
- The row checker and the diff warning are 0181's `cargo xtask
  check-pr` (see the 2026-11-17 line): every row of
  `docs/engine/friction.md` has an estimate and a measured number (an
  adopted row, the lines removed), and a PR that changes a file an open
  `engine#`-blocked ticket names as the code it replaces gets a warning
  naming that ticket. Not this ticket's code.
- `engine-pin` step 4: an adoption records the lines removed in the
  friction row.
- 1233's text says when it re-runs: after every boundary audit, and at
  least once per Act.

**Out (do not do):**
- The audit itself (1233).
- The engine-or-game verdict per part (0148's skill text).

## Implementation steps

1. Update `engine-pin`, `engine-friction` and 1233, then run `pwsh -File
   scripts/sync-agent-skills.ps1`.
2. Fill the empty rows of `friction.md` from the PRs they name (with
   0181's `check-pr` if it has landed, else by reading the table).

## Acceptance criteria

- [ ] Every row of `docs/engine/friction.md` has an estimate and a
      measured number, and `engine-pin` step 4 records an adoption's
      lines removed.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None beyond the gates: docs and skills (the checker's tests are
  0181's).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
