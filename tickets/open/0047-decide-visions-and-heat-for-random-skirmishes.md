---
id: "0047"
title: "Decide: Visions (pick one of three, or skip) and heat for random skirmishes"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0804"]
nick_input: decision
completed:
---

# 0047 — Decide: Visions and heat for random skirmishes

## Context

Nick, 2026-10-04 and 2026-10-05
([`docs/design/replayability.md`](../../docs/design/replayability.md),
*Visions*). Already decided, not asked again:

- **Random skirmishes have Visions instead of stars**: an **opt-in** choice
  of three cards, or skip. A card brings a drawback or condition and an
  extra reward (Nick: "take a card from a selection of 3 or skip it ... it
  gives a drawback or condition but an extra reward").
- **Story battles and fixed skirmishes never have Visions**; they have
  stars (0046).
- The rewards are more meaningful than the unused-rewind EXP bonus, never
  needed to finish the game, and paid slightly differently from stars. What
  they are is 0048.
- The name and the lore are open: "don't necessarily need to be in lore but
  it could be doable".
- **Heat** (Hades' Pact of Punishment: turn the difficulty up for better
  loot) was also a yes ("1. sure").

Random skirmishes themselves (`world-structure.md`, ticket 1008): they
appear on cleared world-map nodes, are small (4–6 enemies, 3–6 turns), are
Easy tier, and are autobalanced for the bots with **no** card taken
(`playtest-bots.md`; 0509). The story bible has vows, echoes and the Door;
Shuyi is the Jade Reach's unseen guardian (`docs/story/bible.md`, `title.md`).

## Nick input

**Decision.** Run with the `ask-nick` skill, one question at a time, each
with options from real games and "describe your own". Show a rendered
mockup of the three-card choice on a random-skirmish node (no bought art,
ADR-0040).

## Scope

**In:** the questions below; the answers written into
`docs/design/replayability.md` (*Visions*); a starter list of cards
(drawbacks only; rewards come from 0048); the `docs/design/README.md` row;
the implementation tickets the answers call for.

**Out (do not do):** any game code; what the cards pay (0048); stars
(0046); the run mode's own use of Visions (0050 decides that, reusing these
answers).

## Implementation steps

1. Read `replayability.md`, `world-structure.md` (*Random skirmishes*),
   `playtest-bots.md` and `docs/story/bible.md` (*Magic*, glossary).
2. Draft a list of about 20 drawback cards that use only rules the game has
   (stats, turn limits, reinforcements, rewind charges, the battle pack,
   healing, terrain magic, deployment size). Mark which would need new rules.
3. Render the mockup (`docs/screenshots/0047-*.png`).
4. **Ask Nick**, in this order:
   1. **The name, and the lore.** Options: "Visions", tied to the title and
      to Shuyi (the story pipeline would then give them a source); a
      plain name with no lore ("Challenges", "Pacts", "Omens"); a lore word
      from the bible (a vow sworn before the battle).
   2. **When the choice appears.** Options: on entering the node, before
      Preparations; on the Preparations screen; before the node is chosen,
      so the cards help pick which skirmish to fight.
   3. **How many cards may be taken.** Options: one or none (what Nick
      described); several, stacking (Hades' Pact, Slay the Spire's
      Ascension); one, plus heat as a separate dial.
   4. **Heat.** Options: none beyond the cards; a heat dial on every random
      skirmish (each step adds a fixed drawback and a bigger reward);
      heat that rises the more skirmishes are won in a row.
   5. **What a card looks like.** Options: a fixed drawback with a fixed
      reward (Slay the Spire's Neow bonuses); a drawback whose reward grows
      with how hard it is; a condition to meet ("win in 4 turns") rather
      than a handicap.
   6. **The starter drawbacks.** Show the list from step 2; Nick keeps,
      drops or changes each.
   7. **Losing, retrying and rewinding with a card.** Options: retry keeps
      the same card; retry draws again (lets players fish for an easy
      card); a lost battle loses the card's reward only. And whether a card
      takes rewind charges away.
   8. **How hard a card is, shown to the player.** Options: nothing; a
      difficulty mark measured by the bots (each card is played many times
      and its win rate shown as one to three skulls); the reward size only.
   9. **Ironman** (0049): are Visions allowed?
5. Record each answer verbatim and as a rule.
6. Write the implementation tickets (`write-ticket`): cards in core (data,
   effects, the draw from the battle's seed so a rewind or a suspend can't
   redraw), the choice screen, and the bots playing each card to measure
   it (after 0506). Add a line to 1008 naming them.

## Acceptance criteria

- [ ] Every question above has Nick's answer, verbatim, in `replayability.md`.
- [ ] The starter card list is in `replayability.md`, each card marked
      "uses existing rules" or "needs a new rule".
- [ ] The mockup is in `docs/screenshots/` and contains no bought art.
- [ ] The implementation tickets exist; `cargo xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes
