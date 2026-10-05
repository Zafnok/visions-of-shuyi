---
id: "0050"
title: "Decide: the roguelike run mode (title menu, own save slots)"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: high
status: todo
blocked_by: ["0804", "0047"]
nick_input: decision
completed:
---

# 0050 — Decide: the run mode

## Context

Nick, 2026-10-04 and 2026-10-05
([`docs/design/replayability.md`](../../docs/design/replayability.md),
*The run mode*). Claude pitched the skirmish generator (0510, a dev tool) as
a post-game mode: a chain of generated, bot-balanced fights, drafting
recruits and Visions from three, with permadeath, ending when the lord
falls, and things to unlock between runs (Into the Breach, Slay the Spire).
Already decided, not asked again:

- **Yes**, "as an extra option on main menu, with its own save slots".

What the mode can build on: generated skirmishes at any level, autobalanced
until the bots land in their bands (0509, 0510; `playtest-bots.md`);
Visions (0047, which comes first so this ticket reuses its answers); the
whole class tree, arts and spells (`progression.md`, `combat-arts.md`,
`magic.md`). The bible's Ashfields are full of echoes (elementals,
`docs/story/bible.md`), which is one possible setting.

## Nick input

**Decision.** Run with the `ask-nick` skill, one question at a time, each
with options from real games and "describe your own". Show a mockup of a
run's map and a draft screen (no bought art, ADR-0040).

## Scope

**In:** the questions below; the answers in `replayability.md` (*The run
mode*); the `docs/design/README.md` row; the implementation tickets.

**Out (do not do):** any game code; changing the campaign; a daily run
(Nick: "maybe, but later"; not ticketed).

## Implementation steps

1. Read `replayability.md`, 0047's answers, `playtest-bots.md`,
   `world-structure.md` and the bible.
2. Render the mockups (`docs/screenshots/0050-*.png`).
3. **Ask Nick**, in this order:
   1. **Name and setting.** Options: a place from the bible (the Ashfields);
      a dream, a vision or a trial with no place in the story; no setting,
      just a mode name.
   2. **When it unlocks.** Options: from the start; after Chapter 3 (when
      the world map opens); after finishing the campaign.
   3. **Who fights.** Options: the story cast (spoilers if open from the
      start); generic recruits drafted during the run (Into the Breach's
      pilots); a mix: a small fixed squad plus drafted recruits.
   4. **A run's shape.** Options: a branching path of fights, shops and
      rests (Slay the Spire); a few islands of three fights each and a
      finale (Into the Breach); one fight after another until defeat. And
      how long a run takes (about 1 hour, 2 hours, open-ended).
   5. **What is drafted between fights.** Recruits, Visions, items, level-ups,
      class changes: which of them, always from three.
   6. **Death and undo.** Permadeath always (and is the run lost with the
      lord, or with the whole squad?); rewind charges or none.
   7. **Between runs.** Options: nothing carries over (pure roguelike); new
      content unlocks (classes, starting units, cards: Into the Breach's
      squads); permanent upgrades (Hades' mirror). And whether anything
      carries into the campaign (0048 may answer that).
   8. **Difficulty.** Options: rising with each fight; chosen at the start
      (Into the Breach's Easy/Normal/Hard); heat from 0047.
   9. **Story.** None; short scenes the first time something happens; a
      small arc that ends after enough runs (Hades).
4. Record each answer verbatim and as a rule.
5. Write the implementation tickets (`write-ticket`): runs in core (state,
   drafts, saved runs in their own slots), the title menu entry and screens,
   and generating each fight with 0510's tool at play time. Say in each
   which open tickets they need (0509, 0510, 0047's tickets).

## Acceptance criteria

- [ ] Every question above has Nick's answer, verbatim, in `replayability.md`.
- [ ] The mockups are in `docs/screenshots/` and contain no bought art.
- [ ] The implementation tickets exist; `cargo xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes
