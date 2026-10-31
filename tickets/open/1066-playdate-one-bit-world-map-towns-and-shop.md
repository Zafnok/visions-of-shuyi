---
id: "1066"
title: "Playdate: the world map, towns and the shop in one bit"
type: feature
milestone: Post–Chapter 1
model: reasoning
effort: high
status: todo
blocked_by: ["1007", "1008", "0409", "1038"]
nick_input: none
completed:
---

# 1066 — Playdate: the world map, towns and the shop in one bit

## Context

Act 1 plays Chapters 4–8 on a world map (`docs/story/outline.md`,
`docs/design/world-structure.md`), and Nick plays Act 1 on his Playdate
in 1064. The Playdate front end (`crates/playdate`, ADR-0070) paints
each screen's view with its own one-bit skin (ADR-0054: a screen's
logic builds a view, a skin paints it), at today's 8 × 16 cells on a
50 × 15 screen, in the window style 1038 set (`docs/design/playdate.md`,
*Size*, *Layout*, *Telling things apart without colour*).

This ticket paints the world-map screens the PC gets from:

- **1007**: the world map (nodes, paths, the army marker, hover info,
  the symbol and level marker above each battle node), the world map
  menu (Save, Camp, Options), towns.
- **1008**: fixed and random skirmish markers, which must stay "clearly
  delineated" (Nick, `world-structure.md`) without colour.
- **0409**: the shop (buy, sell, repair), opened from a town.

The PC's look is 1007's mockup answer in `look-and-feel.md`; the
Playdate's one-bit version follows `playdate.md`'s rules for telling
things apart (black / grey / outlined, the range marks).

## Nick input

None. No check after merge (Nick, 2026-10-21): `cargo shuyi frame-png
--playdate` pictures go in the PR, and how the map reads on the device
goes on 1064's checklist. If the world map can't follow `playdate.md`'s
rules (for example the node symbols can't be told apart in one bit),
that is a design question: ask it with `ask-nick` before building,
with rendered one-bit mockups.

## Scope

**In:**
- One-bit paints for the world map, its menu, a town and the shop
  screens, chosen by `SkinSet`, at 50 × 15 cells; the map scrolls if the
  act's map is wider than the screen (as the battle map does, 1038).
- The node kinds, the skirmish markers and the level markers told apart
  without colour, following `playdate.md`.
- The Playdate's buttons and crank for these screens as `playdate.md`
  (*Buttons and the crank*) lays them out; the system menu entries the
  world map needs, if any.
- Look tests with the Playdate skin; behaviour tests read the views.
- The Playdate bundle (1036's list) includes the world map data.

**Out (do not do):**
- Camp, supports, promotion and the Preparations shop (1067).
- Any PC change; the world map's rules (1007, 1008).
- Loading only one chapter's data at a time (1068).

## Implementation steps

1. Read `playdate.md`, 1007's and 1008's views and the PC skins, 1038's
   window style, `crates/playdate/README.md`.
2. Estimate the size first (ADR-0092): about 300 production lines, by
   comparison with 1046's batch of screen paints.
3. One paint at a time, each with its look test.
4. `cargo shuyi frame-png --playdate` pictures of each screen for the
   PR: the test world map with every node kind, a town, the shop.
5. Simulator: travel, enter a town, buy, save and load on the map.

## Acceptance criteria

- [ ] In the Simulator, the test world map can be travelled, a town and
      its shop used, and the game saved and loaded on the map.
- [ ] Every node kind and marker is told apart in the pictures without
      colour.
- [ ] No PC snapshot changes.
- [ ] Starting rules and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit (look, Playdate skin): the map with each node kind, the menu, the
  shop.
- Integration (harness, Playdate skin set): travel → town → shop → buy
  → leave → save → load.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
