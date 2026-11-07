---
id: "0511"
title: "Boss and green AI: weigh debuffs, stances, moves after attacking and drain"
type: feature
milestone: M4 Enemy AI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0503"]
nick_input: none
completed: 2026-11-07
---

# 0511 — Boss AI weighs effects the forecast doesn't show

## Context

Ticket 0503 lets bosses and combat green units choose Combat Arts and
actives. It scores each one from the attack's forecast (plus Line Pierce's
strike), minus a penalty for the durability it costs
(`crates/core/src/ai.rs`, module docs → *Arts and actives*). An effect the
forecast's numbers don't show scores nothing, so with the penalty the AI
never picks an art or active **for** it:

- a **debuff** on the target (Pinning Shot's Mov −3, Pressure Point's
  Spd −3; `ArtEffect::on_first_hit`),
- a **stance** on later turns (Sidestep's avoid +20; `ArtEffect::stance`, an
  active's `Stance` rider). Its bonus in the attack's own combat is already
  in the forecast,
- a **move after the attack** (Vault, Swoop; `ActiveEffect::Strike::post_move`),
- **drain** (`ActiveEffect::Strike::drain`).

So a boss archer never uses Pinning Shot, and a gauntlet boss uses Sidestep
only when its avoid already pays off in the combat itself. It was to be
done only if the Chapter 1 playtest (0804) found bosses flat. Act 1 has
a boss in every chapter, several with arts (`docs/story/outline.md`),
and Nick no longer spot-checks gameplay (2026-10-21), so it is done
before the Act 1 playtest (1064) instead (re-gated 2026-10-21, 0129).

Design: `docs/design/combat-arts.md` (the arts and what they do; *Enemies:
bosses only; combat green units too*).

**Either order with 0521:** that bug ticket repairs art reach and Cascade
damage in the existing score walk; this ticket adds utility beyond those
numbers. If 0521 is done, extend its final walk and preserve its fixtures.
If this lands first, leave 0521's numerical/range corrections to that
ticket and keep the utility branches compatible. The zero-new-weight
comparison uses the base revision this PR actually builds on, including
0330/0521 when present, rather than restoring their old bugs.

## Nick input

None. The numbers are *tunable* AI weights (`assets/data/ai.ron`).

## Scope

**In:**
- A score for each of the four effects above, added to the technique's
  score in `Planner::best_attack` (`crates/core/src/ai.rs`), with new
  weights in `AiWeights` / `assets/data/ai.ron`.

**Out (do not do):**
- Ordinary enemies or non-combat green units using arts or actives (never).
- Changing how the plain attack is scored, or the weights 0501 and 0503 set.
- Non-combat actives (0503's rule for a unit holding its tile stays).
- UI.
- Art-range/Cascade scoring fixes (0521) and first-hit combat mechanics
  (0330). They are separate fixes, not new utility weights.

## Implementation steps

1. `BattleState::attack_forecast` (`crates/core/src/battle.rs`) returns the
   forecast and the pierce. Extend what it returns (a small `pub(crate)`
   struct) with what the AI needs from the validated `AttackStep`: the
   art's `on_first_hit` debuff and `stance`, the active's `stance`,
   `post_move` and `drain`.
2. In `core::ai`, add to a technique's score, each with its own weight in
   `AiWeights` (all *tunable*):
   - **debuff:** `debuff × amount × P(at least one of the attacker's strikes
     hits)`, only if the target survives (use the `Odds` walk; add the
     chance to `Odds`);
   - **stance:** `stance × (Def + Res + avoid / 10 of its bonuses)`, only if
     the unit's tile after the attack is in the danger zone
     (`Planner::danger`) and the unit doesn't already have the effect;
   - **move after:** `post_move` if the attacking tile is in the danger zone
     (it can step away);
   - **drain:** `damage × min(E[HP dealt] / 2, HP the attacker is missing)`.
3. Document the rules in the module docs and the weights in `ai.ron`; update
   the `ai.ron` loader tests in `crates/content/src/ai.rs`.

## Acceptance criteria

- [x] A boss archer in range of a unit it can't kill uses Pinning Shot when the debuff weight makes it worth its durability, and a plain shot when the weight is 0 (tests).
- [x] A boss that will be attacked next turn prefers a stance art over a plain attack of the same numbers; one out of every hostile unit's reach doesn't (tests).
- [x] With every new weight at 0 the AI chooses exactly as on this PR's
      base revision (test), preserving 0330/0521's fixes if already merged.
- [x] Same state → same command (determinism test); every emitted command is valid (0501's property test still passes).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the cases above, as ASCII scenes in `crates/core/src/ai/tests.rs`.
- Property: 0501's random-state AI test keeps passing with the new weights.

## Completion notes

- `BattleState::attack_forecast` now returns an `AttackOutlook`: the
  forecast, Line Pierce's strike, the art's first-hit debuff and the art's
  or active's stance (each with its source), the active's `post_move` and
  `drain`.
- `core::ai` adds the four scores to a technique's score (`Planner::utility`),
  documented in the module docs (*What the forecast doesn't show*) and in
  `assets/data/ai.ron`. `Odds` gained `debuffed`: the chance the attacker
  hits at least once and the target stands (the property test against
  `resolve` checks it too).
- New weights (*tunable*): `debuff: 8`, `stance: 10`, `post_move: 10`,
  `drain: 5`. With them a boss archer pins a unit it can't kill (Mov −3 on
  a sure hit: 24 against 15 for 3 durability), a gauntlet boss in reach of
  the player's units Sidesteps (20 against 15), an archer in danger Vaults
  (10 against 5), and a wounded caster Siphons only when the spell use is
  cheap enough (12.5 for 10 HP missing against 15: not at the starting
  costs).
- **Chapter 4's recorded playthrough was re-recorded**: Dace now uses his
  arts, so the old commands no longer finished him on turn 5. The new
  recording still wins on turn 5 with nobody lost; only turn 5 changed
  (one more attack needed).
- The zero-weight criterion is tested on this PR's scenes
  (`without_the_new_weights_every_scene_plays_as_before`) and by every
  0501/0503/0521 AI test still passing unchanged with the starting weights
  (their fixtures have no art or active with these effects).
- Friction: estimate 80 production lines, measured 92 (1.15×).

**The agent's starting rules** (for the Act 1 playtest, 1064):

- A debuff counts its stats' amounts added up, plus the avoid it takes off
  / 10 (Off Balance). It counts nothing on a target that already has that
  art's debuff (it would only refresh).
- A stance counts its Def + Res + Avoid / 10 only; a stance's reaction
  (Riposte, Spear Wall, Deflect…) and other stats score nothing.
- "In danger" is the AI's danger zone as it stands before the attack (the
  same one healers use): a hostile unit could attack the tile this turn.
- Drain counts the HP missing before the combat, not what the combat's
  counter takes.
