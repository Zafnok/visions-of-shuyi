---
id: "0051"
title: "Decide: the randomizer — what it changes, where it's offered, sharing a seed"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0804"]
nick_input: decision
completed:
---

# 0051 — Decide: the randomizer

## Context

Nick, 2026-10-05
([`docs/design/replayability.md`](../../docs/design/replayability.md),
*The randomizer*): yes to a built-in randomizer ("3. sure"). Fire Emblem
fans build their own for most FE games (randomized classes, growths,
weapons, recruit order); ours can be official, because all units, classes
and items are data files (`assets/data/`), and the bots can check that a
randomized game can be won (0506).

Things a randomizer has to respect: the lead is the lord with their own
class line (`progression.md`, *The lord's line*); fliers are tier 3+
(0017); magic classes drop weapons at tier 3 (`magic.md`); recruitment is by
defeating marked enemies or by quests (`battle-scenes-and-recruitment.md`);
portraits and battle art belong to characters (`look-and-feel.md`).

## Nick input

**Decision.** Run with the `ask-nick` skill, one question at a time, each
with options from real games and "describe your own".

## Scope

**In:** the questions below; the answers in `replayability.md` (*The
randomizer*); the `docs/design/README.md` row; the implementation tickets.

**Out (do not do):** any game code; randomizing the story or the script;
the run mode (0050).

## Implementation steps

1. Read `replayability.md`, `progression.md`, `weapons-and-items.md` and
   `battle-scenes-and-recruitment.md`.
2. **Ask Nick**, in this order:
   1. **What can be randomized.** A checklist, each on or off by the player:
      player classes; growths; base stats; starting weapons and items;
      enemy classes; shop stock; chest and village items; the order units
      join. Mark which ones the community randomizers do (all of them).
   2. **The lord.** Options: the lead always keeps the lord line; the lead
      is randomized too.
   3. **A character's look with a new class.** Options: keep the portrait
      and use the new class's map sprite; keep both (the sprite no longer
      matches the class); only randomize among classes the character has
      art for.
   4. **Where it's offered.** Options: a New Game option from the start;
      after finishing the campaign once; a separate title menu entry.
   5. **Sharing.** A short seed code the player can type or paste so a
      friend gets the same game (with the same checklist).
   6. **Winnable or not.** Options: every seed is checked by the bots before
      the player starts (slow: minutes); the player is warned that a seed
      may be very hard; seeds that fail are rerolled silently.
   7. **Stars, Visions and Ironman** in a randomized game: all allowed, or
      marked so they don't count with the normal ones.
3. Record each answer verbatim and as a rule.
4. Write the implementation tickets (`write-ticket`): the randomizer in
   `content` (a seed and the checklist turn the data into a randomized
   copy, deterministic), the New Game options, and the bot check if Nick
   wants it (after 0506).

## Acceptance criteria

- [ ] Every question above has Nick's answer, verbatim, in `replayability.md`.
- [ ] The implementation tickets exist; `cargo xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes
