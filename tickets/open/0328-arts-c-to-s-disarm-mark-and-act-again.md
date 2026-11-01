---
id: "0328"
title: "Arts C–S, part 4: Disarm, Marked Shot and Unbroken Stride"
type: feature
milestone: M2 Core rules
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0018", "0325", "0330"]
nick_input: none
completed:
---

# 0328 — Arts C–S, part 4: Disarm, Marked Shot and Unbroken Stride

## Context

Ticket 0018 (`docs/design/combat-arts.md`, *Ranks C–S*, *Exact rules*)
decided three arts that change what units may do, not just numbers:

- **Disarm** (Sword B, 5): from the first hit, the target can't counter
  (this combat too) and can't attack with a weapon until the end of its
  next phase. It can still move, cast and use non-attack actives.
- **Marked Shot** (Bow A, 6): from the first hit, until the end of its
  next phase, every attack on the target by its enemies gets hit +20 and
  crit +10 (their forecasts show it).
- **Unbroken Stride** (Sword S, 8): if the target falls from the user's
  strikes this combat, the user may move its full Mov again and take
  another action; not Unbroken Stride again that phase.

0330 is required first: it fixes the shipped debuffs' first-hit timing
and shared preview/resolution path. Reuse that activation point for
Disarm and Marked Shot; applying a state after the entire combat would
miss Disarm's remaining counters and Marked Shot's subsequent strikes.

## Nick input

None. Verified by tests; the feel goes on 1064's checklist.

## Scope

**In:**
- A timed "disarmed" state and a timed "marked" state on a unit (beside
  0312's debuffs), read by legal-action checks (`crates/core/src/legal.rs`),
  counters and the forecast; the AI never plans a weapon attack with a
  disarmed unit.
- Refreshing a unit's turn after Unbroken Stride (its `moved`/`acted`
  state), with a once-per-phase flag.
- The three arts in `arts.ron`; forecast lines (`disarms on hit` / `no
  counter`, `marks: hit +20, crit +10`, `if it falls: act again`).

**Out (do not do):**
- 0325–0327's arts. The AI valuing these effects beyond the forecast
  (0511).

## Implementation steps

1. Read the design doc's rules, 0312's debuff code and 0330's completed
   first-hit strike path and regressions.
2. Add the two states and their expiry at the end of the target's next
   phase; refresh-on-reapply.
3. Disarm in counters, legal actions and the AI's attack options.
4. Mark in the hit/crit formulas of attacks on the marked unit.
5. Unbroken Stride: after the combat, if the target fell, refresh the
   user's turn; the flag clears at the next phase.

## Acceptance criteria

- [ ] A disarmed enemy doesn't counter and its AI doesn't attack with a
      weapon on its phase; it can still cast.
- [ ] A miss before the first hit does not activate Disarm or Marked Shot;
      after the first hit their effects change the remaining applicable
      strikes/counters in that combat, in preview and resolution.
- [ ] A marked unit's attackers get +20 hit / +10 crit in their forecast,
      until the end of its next phase.
- [ ] Unbroken Stride gives one more turn after a kill, never twice in a
      phase, and nothing without a kill.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit / battle: the three rules and their expiry.
- Property: a disarmed unit never makes a weapon strike.
- Snapshot: a forecast with each line.

## Completion notes

*(Filled in by the session that completes the ticket.)*
