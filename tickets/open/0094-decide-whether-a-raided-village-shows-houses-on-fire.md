---
id: "0094"
title: "Decide whether a raided village shows houses on fire"
type: design-decision
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0094 — Decide whether a raided village shows houses on fire

## Context

Ticket 0471 (2026-11-22) chose the art for the picture look's houses
(edermunizz's thatched cottages) and burning trees (Minifantasy: Raided
Village's moving flames), recorded in `docs/design/look-and-feel.md`,
*Art bought beside Tiny Tales for the stand-ins above*. While choosing,
Nick asked to see the fire on the houses too ("can I see it on the
houses too?", "try to make the fire cover the whole house if possible"),
and the mockups showed cottages burning.

Today a house is only a house (`docs/design/terrain.md`, *Building
tiles*, ticket 0037): nothing in the rules or the map makes it burn.
Kell's Ford (Chapter 2) is "a race to save a burning village"
(`docs/design/audio.md`, 0071). Whether its houses are drawn on fire,
and whether that is only a look or changes the rules, is Nick's call.

## Nick input

**Decision** (asked with the `ask-nick` skill, with pictures: the
0471 mockups of burning cottages can be re-rendered in the scratchpad
from the bought art once 0460 is done, or from the store previews):

1. Do burning houses appear at all? Options from games: Fire Emblem
   GBA (villages are destroyed by brigands: the village tile turns to
   a ruin, no fire drawn), Fire Emblem: Three Houses (burning
   buildings as scenery in some maps), Triangle Strategy / Tactics Ogre
   (fire spreads on tiles and hurts units), or only as scenery set by
   the map.
2. If yes: a look only (the map file marks a house as burning), or a
   rule (a burning house is a terrain with its own numbers, like
   Burning woods, or houses catch fire from fire magic)?
3. Does a burned-out house look different afterwards (a ruin)?

## Scope

**In:**
- Ask; record Nick's words and the rules in `docs/design/terrain.md`
  (or `look-and-feel.md` if it is a look only); update `docs/design/README.md`.
- Write the implementation ticket(s) his answer needs, `blocked_by` 0460
  if they use its flames, and add them to 1064's `blocked_by` if
  Chapter 2 needs them.

**Out (do not do):**
- Any code or art.

## Implementation steps

1. Read `terrain.md` (*Building tiles*, Burning), `look-and-feel.md`
   (the 0471 decision), the Kell's Ford outline in
   `assets-private/story/` (`cargo shuyi private-assets`).
2. Ask the questions above with the `ask-nick` skill and pictures.
3. Record the answer; write the follow-up tickets.

## Acceptance criteria

- [ ] Nick's answer and words are in `docs/design/`.
- [ ] Follow-up tickets exist for what he decided, with their chains.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None (a decision).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
