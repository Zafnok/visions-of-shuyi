---
id: "0326"
title: "Arts C–S, part 2: stances that react (Riposte, Spear Wall, Deflect, Overwatch)"
type: feature
milestone: M2 Core rules
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0018", "0325"]
nick_input: none
completed: 2026-11-03
---

# 0326 — Arts C–S, part 2: stances that react

## Context

Ticket 0018 (`docs/design/combat-arts.md`, *Ranks C–S*, *Exact rules*)
decided four arts whose stance changes how later combats go, not just
their numbers:

- **Riposte** (Sword C, 4): while defending, the user's first counter
  strike comes before the attacker's first strike (Vantage).
- **Spear Wall** (Spear B, 4): the user counters at distance 1–2 with its
  spear, counters hit +15.
- **Deflect** (Gauntlet B, 5): the first enemy strike that would hit
  misses, and the user strikes back at once (once per stance).
- **Overwatch** (Bow B, 5): **an action of its own, no attack** (the one
  exception to "arts only when attacking"): pays 5, ends the action, and
  the first enemy that ends a move within the bow's range is shot once
  before it acts.

0312's stances (`ArtEffect::stance`, `TimedMods`) only carry stat and
combat bonuses; these need flags the battle reads when a combat is built
or a unit ends a move.

**Either order with 0330:** if the first-hit timing correction is done,
extend its shared strike path and preserve its same-combat debuff and
preview regressions. If not, implement these reactions on the current
path without taking on the numeric-debuff fix; 0330 then preserves the
reaction ordering and tests while repairing activation timing. Neither
ticket may restore the other's old behavior by replacing the strike loop.

## Nick input

None. Verified by tests; the feel goes on 1064's checklist.

## Scope

**In:**
- Stance flags for strike order (Riposte), counter range (Spear Wall),
  a one-time deflect-and-strike-back (Deflect), and an overwatch trigger
  on an enemy's move end (Overwatch), in `core` (`art.rs`, `battle.rs`,
  `combat.rs`), with `Event`s for the playback (the deflect, the
  overwatch shot).
- Overwatch as an `Arts` menu line with no target (`crates/ui/src/screens/battle/art_list.rs`):
  `Overwatch  −5 dur  B  ends action`, dimmed only when the bow is broken.
- The four arts in `arts.ron`; the forecast's `stance:` lines.
- The map shows a unit on Overwatch with its stance, as Brace does (in the
  map scene and every map skin, ADR-0038).

**Out (do not do):**
- 0325's, 0327's, 0328's arts. The AI choosing these (0511 values
  stances; until then bosses don't pick them for their stance alone).

## Implementation steps

1. Read the design doc's rules for the four and 0312's stance code.
2. Add the stance flags (`TimedMods` or a new field beside it) and read
   them where combats are resolved: strike order, counter reach, the
   deflect, the overwatch trigger in movement resolution (after a move
   ends, before the unit's action; an AI unit that falls doesn't act).
3. Overwatch as a `Command` of its own (an art used as an action), paid
   as an art, giving weapon EXP base 4/2 and unit EXP for the shot.
4. UI: the art line, the stance lines, the playback of the reactions.

## Acceptance criteria

- [x] Riposte: a scripted enemy-phase attack on a user in the stance shows
      the user's strike first (battle test reading the events).
- [x] Spear Wall: an archer attacking at 2 is countered.
- [x] Deflect: the first hitting strike misses and the strike back
      happens; only once per stance.
- [x] Overwatch: the first enemy ending a move in range is shot before
      acting; one that doesn't move isn't; it triggers once.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit / battle: the four rules above, and each stance ending at the start
  of the user's next phase.
- Snapshot / integration: the Arts menu with Overwatch, a stance forecast.

## Completion notes

The four arts are in `assets/data/arts.ron` with the design doc's ranks
and costs; their stances react in `core` and show in the menu, the
forecast, the playback and on the map.

- A stance's reaction is `TimedMods::reaction: Option<StanceReaction>`
  (`StrikesFirst`, `Counters { reach, hit }`, `Deflect`, `Overwatch`), one
  per stance, beside Hold the Ford's `stops_enemies`. Data:
  `stance: Some((reaction: Some(Deflect)))`.
- The combat maths gained `Forecast::reactions` (`Reactions {
  defender_first, deflects }`) and a `reaction` mark on each `Strike` and
  `PlannedStrike` (`Deflected`, `StrikeBack`). `strike_order` puts a
  Riposte counter first; `resolve` and `if_all_hit` turn the first strike
  at a deflecting side that would hit into a miss and add the strike back,
  so the preview's plan and kill mark agree with the fight. The battle sets
  the reactions when an attack is planned (the art's own stance counts) and
  again from the units in every `clash`, then ends a Deflect that reacted
  (`EffectExpired`).
- Spear Wall widens the defender's weapon to 1..reach and adds the hit to
  its counters in `fighters`.
- Overwatch is `UnitAction::UseArt { art, slot }` (`ArtEffect::is_action`):
  equips the bow, pays, takes the stance, ends the action. After an `Act`'s
  move (`BattleState::walk`), each hostile unit on Overwatch whose weapon
  reaches the mover shoots once: `Event::OverwatchShot { unit, art,
  target }`, a one-strike `CombatResolved`, `EffectExpired`, weapon EXP as
  an art, the fall, unit EXP. A mover that falls doesn't act. An attack
  with Overwatch, or another art as an action, is refused
  (`ArtNotUsable`); content refuses an Overwatch art with other effects.
  The bots' legal commands list `UseArt`.
- UI: the Arts menu line `Overwatch  −5 dur  B  ends action` is chosen at
  once (no target) and dimmed only when the bow can't pay; stance notes
  read `strikes first`, `counters at 1–2 counters hit +15`, `deflects a hit
  and strikes back`, `shoots an enemy moving in range` (shortened from the
  longer wording to fit the menu's 40-character notes); the playback shows
  `DEFLECTED` and names an Overwatch shot in its banner whoever's it is;
  the rewind list reads `used Overwatch`. The map shows every reacting
  stance as a bonus (the scene's `bonus` mark, every skin), as Brace's.
- 0330 was already done: these extend its shared strike path; its
  first-hit debuff tests still pass.
- Friction: estimate ~400 production lines, written after the code rather
  than before (the 0325 lapse again); measured 454: 1.1×, under 2×.
- PR size: 1,925 changed lines.
- Follow-up: 0759 (a dialogue print/parse round trip fails on text that
  starts with `(`; found by the property test in a full run, unrelated to
  this ticket).

The agent's starting rules (the design doc is silent; for the Act 1
playtest, 1064):

- Deflect's strike back uses the user's first-strike damage, doesn't count
  as one of its strikes, its Cascade hits or the attacker's first hit, and
  a user with no numbers in that combat (it can't counter it, e.g. out of
  reach or Guard Break) only deflects.
- Deflect works in the combat it is used in (a stance counts from that
  combat), and on any strike at the user: Line Pierce's or an Overwatch
  shot too.
- Overwatch reacts only to a move made in an `Act` (not a raider's move
  after its attack). The shot uses the equipped weapon as it is then: a bow
  that Overwatch's own payment brought to 0 shoots broken. Several units on
  Overwatch each shoot the same mover, in unit order. The shot gives no
  support points.
- Spear Wall's reach applies to the weapon the user is attacked with (never
  a spell); its +15 to every counter strike.
- The forecast's stance line keeps the existing `stance: …` form; the
  mockup's `until your next phase` line isn't shown, as for Sidestep.

Left to judge by playing (1064): the feel of each reaction, whether the
Overwatch note wording reads well, and the numbers.
