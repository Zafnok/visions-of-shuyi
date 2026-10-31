---
id: "0326"
title: "Arts C–S, part 2: stances that react (Riposte, Spear Wall, Deflect, Overwatch)"
type: feature
milestone: M2 Core rules
model: reasoning
effort: medium
status: todo
blocked_by: ["0018", "0325"]
nick_input: none
completed:
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

- [ ] Riposte: a scripted enemy-phase attack on a user in the stance shows
      the user's strike first (battle test reading the events).
- [ ] Spear Wall: an archer attacking at 2 is countered.
- [ ] Deflect: the first hitting strike misses and the strike back
      happens; only once per stance.
- [ ] Overwatch: the first enemy ending a move in range is shot before
      acting; one that doesn't move isn't; it triggers once.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit / battle: the four rules above, and each stance ending at the start
  of the user's next phase.
- Snapshot / integration: the Arts menu with Overwatch, a stance forecast.

## Completion notes

*(Filled in by the session that completes the ticket.)*
