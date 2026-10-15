---
id: "1051"
title: "Enemy AI plans once per phase instead of once per command (same choices, faster on the Playdate)"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: blocked
blocked_by: ["1050"]
nick_input: none
completed:
---

# 1051 — The enemy AI plans once per phase

## Context

**Optional:** only if Nick finds the enemy's pauses too long when he
plays Chapter 1 on his Playdate (1050 decides; if he doesn't, 1050
closes this ticket as not needed).

1026 measured on Nick's Playdate (ADR-0070 §5): a 14-enemy phase thinks
for 1.2 s in all, 80 ms per enemy on average and 0.23 s at worst, as a
short pause before each enemy acts. `next_command` (in
`crates/core/src/ai*`) replans every enemy for each command, so a phase
costs about n² decisions. Planning the phase once and replaying the plan
(replanning only what a command's outcome invalidates) is the obvious
speed-up. ADR-0004: `core` is deterministic, and **the AI's choices must
stay identical** (the bots, play records and replays depend on it).

## Nick input

None to decide (1050 records whether it is needed).

## Scope

**In:**
- Plan once per phase in `core`'s AI, with whatever cache or incremental
  replanning keeps every choice the same as today's per-command
  planning.
- A property test: over many generated or recorded battle states (the
  bots' play records, 1026's 31 Chapter 1 phase states), the new
  planner gives exactly the same command sequence as the old one. Keep
  the old planner as the test's oracle (test-only code).
- Measure on the PC (criterion or a timing test) and on Nick's Playdate
  (the same 31 phases as 1026, e.g. with the spike's bench approach or a
  debug entry in the Playdate front end).

**Out (do not do):**
- Changing what the AI decides, its values or difficulty.
- Threads or anything non-deterministic.

**Carve (1114):** either order works. If the AI moved with the tactics
kit to the engine, change it there.

## Implementation steps

1. Read ADR-0004, `docs/design/` AI docs, `crates/core/src/ai*`, the
   spike's bench (`spike/playdate`, `playdate/bench-gen`).
2. The planner and the oracle property test.
3. Measure PC and device; numbers in the PR.

## Measured on Nick's Playdate

The 31 Chapter 1 phases (1026's set): total, average, slowest phase and
slowest single decision, before and after.

## Acceptance criteria

- [ ] The property test shows identical commands to the old planner.
- [ ] Device numbers before/after in the PR.
- [ ] All gates in the `run-gates` skill pass (mutation testing in CI).

## Tests required

- Property: identical choices to the old planner.
- Unit: the cache's invalidation cases.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
