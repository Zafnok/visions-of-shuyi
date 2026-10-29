# ADR-0099: World maps are a file per act, their rules in core, their look in the file, painted by a glyph and a picture skin

- **Status:** Accepted
- **Date:** 2026-10-29
- **Related tickets:** 1007, 1008, 1070, 1082, 1198
- **Extends:** ADR-0005 (content as data), ADR-0035 (chapter files and
  the flow screen), ADR-0038 and ADR-0054 (graphics are a skin; a screen
  says what it shows as a view), ADR-0039 (the save format)

## Context

`docs/design/world-structure.md` (ticket 0008) wants a world map per act
from Chapter 4 on: places joined by roads, story battles that open new
places, side quests, towns with shops, saving anywhere on the map, camp
from its menu, and a new map when an act ends. Nick picked its look over
five rounds (`look-and-feel.md`, *The world map*), for the glyph look
and the picture look, and its controls (`controls.md`, *World map*).
Chapters 1–3 stay linear and must play exactly as before.

## Decision

1. **A file per act**, `assets/worldmaps/<id>.ron`
   (`assets/worldmaps/README.md`): the places (`id`, `name`, `pos`,
   `kind`, `music`, `opens_when`), the roads (two places and the cells
   where they turn), `start`, `final_node`, `next`, the act's level
   range, the map's music, its size, its glyph ground and an optional
   picture. `trpg_content::worldmap` loads and checks it into a
   `WorldMapDef`: the rules (`trpg_core::WorldMap`) and the look (where
   places are drawn, the roads' turns, the ground, the picture, names
   and music), which core never sees.
2. **The rules are core's** (`trpg_core::worldmap`, ADR-0004): which
   places are open (conditions; a story battle also waits for its
   chapter), travel (fewest roads, around an unwon battle when it can,
   else stopping at it), entering a town, clearing a battle and the act
   change, and a town's trades. The state is `Campaign::world`
   (`WorldState`: the map, the army's place, the places cleared); it
   changes only through `Campaign::world_command`
   (`WorldCommand::{TravelTo, Enter}`), `clear_node` and `trade`, which
   return what happened (`WorldEvent`s, or the gold). What is open is
   worked out from the campaign, never stored, so it can't drift.
3. **Chapters say where they live:** `ChapterDef::world_map`. A story
   battle's place names its chapter; content checks both sides agree. A
   chapter without it is linear and unchanged.
4. **The flow** (`ui::flow`) shows `WorldMapScreen` for a world-map
   chapter; the screen pops with a `WorldBattle` when the army arrives
   at an unwon battle, the flow plays its scenes and battle, clears the
   place and shows the map again (or the next act's, or "To be
   continued"). A side quest plays through a `ChapterDef` made from its
   place, so the battle code needs nothing new.
5. **Saving:** `SavePoint::WorldMap`, a new variant, and
   `Campaign::world`, left out of a save while `None`. No older save
   holds either, so every `SAVE_VERSION` 2 save reads and writes back the
   same; the version stays 2 (as `Campaign::tutorial` did, ADR-0039).
6. **The look is two skins of one view** (`WorldMapView`, ADR-0054): the
   glyph skin paints the ground in cells and the roads, plates and cursor
   as pixel overlays (the road band one width across and down, which
   cells can't give); the picture skin draws the map's picture, dotted
   roads, the same plates and the lead's map sprite, and is used when the
   player's map look is pictures and the map has a picture. The world
   map isn't a battle map, so it has no `MapScene`: nothing of the
   battle's grid kit fits it (no tiles, units or ranges).
7. **Its two shortcuts are a key group of their own**
   (`KeyGroup::WorldMap`), so they may share the battle's keys and
   buttons (Nick: "different contexts can reuse buttons").

## Consequences

- Act 1's real map (1070) is a data file and a picture; skirmishes
  (1008) add node kinds and rules to the same modules.
- The map shows whole, at most 70 × 30 cells: a larger act needs
  scrolling (a ticket when one does).
- The PC Key bindings screen had no room for the shortcuts' rows: the
  engine's skin now scrolls (engine#2394), and 1198 lists them once the
  pin is past it.
- The Playdate has no world map skin or buttons yet; its Options and Key
  bindings screens leave the world map's rows out.
