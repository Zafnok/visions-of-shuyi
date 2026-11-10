---
id: "1066"
title: "Playdate: the world map, towns and the shop in one bit"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1007", "1008", "0409", "1038"]
nick_input: none
completed: 2026-11-09
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
      its shop used, and the game saved and loaded on the map. *Not done
      here: a Playdate build has no debug menu to reach the test map; the
      same walk runs in the harness on the Playdate's skins and buttons
      (see Deviations). The device check is 1064's.*
- [x] Every node kind and marker is told apart in the pictures without
      colour.
- [x] No PC snapshot changes.
- [x] Starting rules and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [x] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit (look, Playdate skin): the map with each node kind, the menu, the
  shop.
- Integration (harness, Playdate skin set): travel → town → shop → buy
  → leave → save → load.

## Completion notes

**Done.** The world map (`crates/ui/src/screens/world_map/playdate.rs`)
and the shop (`crates/ui/src/screens/battle/playdate/shop.rs`) have
one-bit skins, picked by `SkinSet`; a town's list of shops is the world
map's list window. On the Playdate the map's menu lists Find and Your
army first (their keys have no button there, `MenuEntry::entries`), and
the system menu's Options opens Options on the map. The world map's
track and the tracks Act 1's towns play are in `playdate-music.txt`
(the map data was already in the bundle: everything under `assets/` is
but what `playdate-bundle.txt` leaves out). Pictures:
`docs/design/playdate/1066-*.png`; the starting rules are in
`docs/design/playdate.md`, *The world map, towns and the shop (1066)*.

**Estimate (ADR-0092):** about 350 production lines, by 1043's dialogue
skin (217) and 1045's Options skin (100) plus 1035's map-menu shortcuts.
Measured (`cargo xtask code-lines`): 434 added, 1.2×: under 2×.

**Deviations:**
- **The Simulator walk-through wasn't done here.** The test world map is
  reached through the debug menu, which a Playdate build doesn't have
  (ADR-0057), and Act 1's map only after Chapter 3. The same walk
  (travel, a town, its shop, buy, leave, save, load) runs in the harness
  on the Playdate's console, inks, skins and buttons
  (`on_the_playdate_shop_in_a_town_then_save_on_the_map_and_load_it_again`),
  and the CI `playdate` job builds the `.pdx`. The device check is
  1064's.
- **A village's shop in battle** wears the same Playdate skin: it is the
  same screen view as a town's (0409), and one line picks it.
- **The every-kind picture** is drawn from a view edited to hold every
  kind at once (the test map at its start has no random skirmish and no
  battle won); the look test checks each kind's inks on the real paint.
- **A place not open yet** that is a town and one that is a capital look
  the same (both `⌂` in grey dots), as on the PC (both grey).

**Follow-up tickets:** 1260 (the music Chapters 2 to 8 ask for still
plays as silence on the Playdate; no ticket had it).

**The agent's starting rules, for 1064's checklist (Nick can veto):**
- The map fills the screen and scrolls with the cursor, kept in the
  middle; no side panel, no help line (as the battle screen).
- A place's kind is its symbol and its plate's fill: black for a story
  battle, a side quest and a capital; grey for a fixed or random
  skirmish; outlined for a town and a battle won; grey dots for a place
  not open yet; a bare dot for a place to pass through.
- The army: the lead's letters white on black under a black flag; the
  cursor's corner marks flip what they cover and don't pulse.
- The place under the cursor is a window on the side away from the
  cursor, top corner, or bottom when that hides fewer places; the act in
  the bottom of its frame, cut past 22 cells.
- A list (menu, Find, a town's shops) at the top of that side, the
  place's window under it when both fit.
- The menu lists Find and Your army first (Find, Your army, Save, Camp,
  Options).
- The shop shows 8 lines at once and what the focused item is under them
  (the PC's line has no room for its numbers); the help on the bottom row.

**Left to judge by playing (1064):** whether a level marker on the map's
top row (cut off when the place is on the screen's first row) matters;
whether the grey `!`/`?` read at 1× on the device; whether the place's
window repeating a town's shops under its list of shops is noise.
