---
id: "0328"
title: "Arts C–S, part 4: Disarm, Marked Shot and Unbroken Stride"
type: feature
milestone: M2 Core rules
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0018", "0325", "0330"]
nick_input: none
completed: 2026-11-09
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

- [x] A disarmed enemy doesn't counter and its AI doesn't attack with a
      weapon on its phase; it can still cast.
- [x] A miss before the first hit does not activate Disarm or Marked Shot;
      after the first hit their effects change the remaining applicable
      strikes/counters in that combat, in preview and resolution.
- [x] A marked unit's attackers get +20 hit / +10 crit in their forecast,
      until the end of its next phase.
- [x] Unbroken Stride gives one more turn after a kill, never twice in a
      phase, and nothing without a kill.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit / battle: the three rules and their expiry.
- Property: a disarmed unit never makes a weapon strike.
- Snapshot: a forecast with each line.

## Completion notes

**Done.** Disarm (Sword B, 5), Marked Shot (Bow A, 6) and Unbroken Stride
(Sword S, 8) are in `assets/data/arts.ron` with the design doc's numbers.

- **Data.** `ArtEffect` gained `on_first_hit_status: [Disarmed |
  Marked(hit, crit)]` (states the first hit puts on the target, beside
  0325's stat and avoid debuffs) and `on_kill: Some(ActAgain)`. Both use
  0330's first-hit path: the debuff is a timed effect on the target
  until the end of its next phase, and applying it again refreshes it.
- **Disarm.** `TimedMods.disarmed`. A disarmed unit has no weapon in any
  combat (`fighters`), so it doesn't counter, and its weapon attacks are
  refused (`CommandError::Disarmed`). `legal_commands` and the AI follow
  from that, and its weapons leave its danger zone (`threat_ranges`).
  Spells still work. From the first hit, the counters left in that combat
  are cancelled. This needed one fix in `resolve`: a side with no
  numbers used to end the whole combat (`break`); now it skips that
  strike (`continue`), as `if_all_hit` already did. When the first strike
  is sure to hit (hit 100, no Riposte, no Deflect), the forecast shows no
  counter.
- **Mark.** `CombatMods.hit_taken` and `crit_taken` on the marked unit
  are added to every strike at it, in `combat::side`. The forecast,
  `resolve`, the AI and the threat forecast all read them from there.
- **Unbroken Stride.** If the target fell and the user still stands, the
  user is ready again (`acted` stays false, `Event::ActsAgain` instead of
  `UnitActed`, no move after its attack). The art's effect stays on the
  user until the start of the next phase. While it is there, the art is
  refused and dimmed (`Unit::art_spent`).
- **Forecast lines:** `disarms on hit`, `marks: hit +20, crit +10`,
  `if it falls: act again`. The panel's counter column shows `no counter`
  once the first strike is sure to hit.
- **Tests:** `battle/tests/rule_arts.rs` has each rule with its expiry,
  the first-hit timing (miss, then hit) and the AI casting instead of
  attacking. It also has the property test that a disarmed unit never
  makes a weapon strike, under AI and random play. That test failed with
  Disarm's weapon removal taken out. The forecast view test reads the
  three lines.

**Starting rules decided by the agent** (the design doc is silent;
Nick may veto):

1. The mark helps **counters** at the marked unit too: "every attack on
   it" is read as every strike at it. A marked enemy that attacks on its
   phase is countered at +20 hit / +10 crit.
2. A disarmed unit's **Overwatch** doesn't shoot, and a disarmed ally
   doesn't join **Chain Strike**: both are weapon strikes.
3. A disarmed enemy's weapons leave its **danger zone**. Only its spells'
   ranges show.
4. A unit that acts again after Unbroken Stride gets **no Vault/Swoop
   move** after that attack. It gets the full move instead.
5. A target that **goes down** (can't be killed) counts as falling for
   Unbroken Stride.
6. The mark's crit +10 is added **before** Deadly Blow doubles crit.

**For the Act 1 playtest (1064):** how Disarm's `disarms on hit` line and
the `no counter` column read; whether +20/+10 from a mark feels right; and
whether a second action from Unbroken Stride is clear on the map, with
the unit simply not greyed out.

**Follow-ups:** none. The AI valuing these effects is already 0511.

**Size:** estimate about 180 production lines, judged by 0325's nine arts
(149 lines; these three also touch validation and the turn flow); measured
139 added (0.8×). PR about 850 changed lines (estimate 1,000).
