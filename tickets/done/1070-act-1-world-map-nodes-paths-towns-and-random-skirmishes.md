---
id: "1070"
title: "Act 1's world map: nodes, paths, the four towns' stock and the random-skirmish pool"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1007", "1008", "0044", "0013", "0506", "0320", "0323", "0608"]
nick_input: none
completed: 2026-11-01
---

# 1070 — Act 1's world map

## Context

After Chapter 3 the Act 1 world map opens (`docs/story/outline.md`;
`docs/design/world-structure.md`). 1007 builds the world map (data
format `assets/worldmaps/*.ron`, node kinds, travel, towns, save and
camp) and 1008 the skirmishes (fixed ones, and random groups from a pool
of authored maps at a capped level); both ship only test maps. This
ticket writes Act 1's real map from the outline:

| Chapter | Story battle | Fixed skirmishes (1071) | Town | Side quests (1072, 1083, 1084) |
| --- | --- | --- | --- | --- |
| 4 | Veyne Hall | *Aldwater Toll Bridge* (guards), *Millhaven Road* | Millhaven | *The Company Ledger* (Tamsin) |
| 5 | The Envoy's House | *The Salt Road* (guards), *Smugglers' Cove* | Saltmere | *Letters Home* (Aske), *The Captain's Oath* (Hollis) |
| 6 | The Ashfields | *The Burnt Mill* (guards), *Brennish Outriders* | Cairnford | none new |
| 7 | The Undercroft | *Pilgrims' Road* (guards), *The Ossuary Gate* | Ossary | *The Other Cart* (Rue), *Unwritten* (Maud) |
| 8 | Saltmere Docks | none | Saltmere (last shop) | none |

The towns' stock follows `weapons-and-items.md` (*Money and shops*) and
0044's tiers ("the shop inventory grows with the story",
`world-structure.md`); Saltmere is "the biggest shop in Act 1". The
random skirmishes' numbers are `world-structure.md`'s *starting values*
(1008 put them in a data file); their enemy levels come from the act's
level range, which waits on 0013's number scale.

## Nick input

None. No check after merge (Nick, 2026-10-21): `cargo shuyi frame-png`
pictures of the map in the PR, and how the map plays goes on 1064's
checklist. The layout (where places sit) follows the outline; stock
lists and the act's level range are *the agent's starting values*, listed
in the Completion notes.

## Scope

**In:**
- `assets/worldmaps/act1.ron` (or 1007's name for it): every node in the
  table, with the paths between them, unlock conditions by story
  progress (each chapter's story battle opens the next chapter's
  region; side quests by `world-structure.md`'s conditions: that
  character recruited and alive, the outline's unlock points), place
  music per node (1007's cues).
- **Story-battle nodes for Chapters 4–8**, placed, named and locked
  until their chapter exists: 0850–0854 each fill in their own node. If
  1007's format can't hold a node without its battle, add the smallest
  field that can (an optional battle) here.
- **Fixed-skirmish and side-quest nodes**, placed and locked the same
  way: 1071, 1072, 1083 and 1084 fill them in.
- **The four towns' stock** (Millhaven, Saltmere, Cairnford, Ossary), by
  chapter, within 0044's tiers; Saltmere's last stock for Chapter 8.
- **The act's enemy-level range** and the random-skirmish pool: a few
  small authored skirmish maps (1008's plain battle files), Act 1
  terrain, played by the bots (`cargo xtask playtest <map> --bot all`)
  so none softlocks.
- Chapter 3's `next` reaching the world map, if 1007's format wants it
  set here (else 0850 does). (1007: Chapter 3's `next` is Chapter 4, and
  Chapter 4's file says `world_map: Some("act1")`.)
- **The map's picture for the picture look** (Nick picked it in 1007,
  `look-and-feel.md`, *The world map*): one image made from the bought
  World Map tiles (`assets-private/library/tiny-tales/tilesets/world-map/`)
  laid to match the glyph ground, in `assets-private/game/` (ADR-0040,
  its credit's `private` list), named by the map file's `picture`; and the
  glyph `ground` rows for the glyph look. A story battle's node can't
  name a chapter that doesn't exist yet (1007's checks): put the nodes of
  later chapters' battles in as each chapter ticket adds its chapter, or
  as `Empty` places until then.

**Out (do not do):**
- The story battles (0850–0854), the fixed skirmishes (1071), the side
  quests (1072), supports and camp events (1073).
- New node kinds or world-map rules (1007, 1008).

## Implementation steps

1. Read `world-structure.md`, the outline's Chapters 4–8, 1007's and
   1008's formats and test maps, `weapons-and-items.md`, 0044's answer.
2. Lay out the map (`ascii-art` skill), then write the data.
3. Stock lists by chapter; the level range; the random pool.
4. Bots on the pool maps; `frame-png` of the map for the PR.

## Acceptance criteria

- [x] The Act 1 map loads and validates; every node is reachable once
      its conditions hold (1007's check).
- [x] Each town's shop opens with its stock (harness test from a save
      on the map).
- [x] The random pool's maps each pass a bot run without a softlock.
- [x] Starting values are listed in the Completion notes (1064's
      checklist).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Content checks: the map, towns, pool maps.
- Integration: travel between the Chapter 4 nodes; a town's shop.

## Completion notes

**What was done.** `assets/worldmaps/act1.ron` holds all 24 places and 23
roads: the outline's table, plus Greywater (where the army starts) and The
Valley Road (`ch04.md`'s first scene). The army goes from the Thornmarch in the
south-west, through the heartland, down the Salt Road to Saltmere, east to the
Ashfields, north to Ossary and back to Saltmere's docks. The four towns are
real, with their shops and music. `assets/skirmishes/act1.ron` and three pool
battles are in (`act1_random_road`, `_ford`, `_ash`, each on its own 16 x 10
map). The glyph `ground` rows are in the file. The picture is
`assets-private/game/worldmaps/act1.png` (private commit `71da8d0`, pinned here,
credited), made by that repository's `tools/worldmaps/act1_picture.py`.

**Two small additions to 1007's format** (the ticket allowed the smallest field
that could hold a node without its battle; `assets/worldmaps/README.md`):

- **`NotBuilt`**, a condition that never holds. Every battle not made yet is an
  `Empty` place with `NotBuilt` beside the conditions it will open by, so the
  real unlocks are already written and only `NotBuilt` and the kind change. The
  file's comments say which ticket fills which place and whether it guards.
  `final_node` may name a `NotBuilt` place, because Chapter 8 doesn't exist yet.
- **A shop's `later`**, more stock once conditions hold, because the format had
  no way to grow a stock "by chapter". `WorldMap::town_shops` is the one lookup.

**Not reachable in play yet.** No chapter says `world_map: Some("act1")`: Chapter
3 (0849) and Chapter 4 (0850) don't exist, so 0850 connects the map, as the
ticket allowed. Tests reach it from a save.

**Those filling it in:** `act1_the_army_walks_chapter_4s_open_places` and its
snapshot name what is open today. 1071 and 0850 update them as they open places.

**The agent's starting values** (for 1064's checklist):

- **The act's enemy levels: 1 to 30.** About 21 authored battles at about 2
  levels each. It only bounds the random groups' cap.
- **Random skirmishes:** `world-structure.md`'s values and 1008's rewards: 0-2
  groups after a story win, at most 3, 4-6 enemies, Easy, the latest story
  battle's level minus 1; half the time no clear loot, a quarter 100 gold, a
  quarter a Potion.
- **Stock** (Hi-Potion from Chapter 8 and Steel from Chapter 5 are Nick's, 0044):

| Town | Opens | Armory | Vendor | Alchemist | Tailor | Blacksmith |
| --- | --- | --- | --- | --- | --- | --- |
| Millhaven | Ch 4 | five Iron weapons, Leather Vest, Chain Mail; from Ch 5 Steel Sword, Steel Spear | Potion | | | yes |
| Saltmere | Ch 5 | Iron and Steel of all five, Chain Mail, Iron Plate, Speed Ring, Power Ring | Potion; from Ch 8 Hi-Potion | Potion; from Ch 8 Hi-Potion | Linen Robe, Leather Vest | yes |
| Cairnford | Ch 6 | Iron and Steel spears, axes, bows; Chain Mail, Iron Plate | Potion | | | yes |
| Ossary | Ch 7 | Iron and Steel swords, gauntlets; Focus Charm | | Potion | Linen Robe, Leather Vest | |

- **No town sells the Elixir in Act 1** (the design's reading: only a handful).
  **Steel armour isn't sold yet:** 0074 hasn't decided it.
- **When things open:** Saltmere, The Envoy's House and Smugglers' Cove after The
  Salt Road; Cairnford and the Chapter 6 fights after The Envoy's House; Pilgrims'
  Road and *The Other Cart* after The Ashfields; Ossary, The Undercroft and The
  Ossuary Gate after Pilgrims' Road; the docks after The Undercroft. *Unwritten*
  opens "once Ossary is reached", read as Pilgrims' Road won. Every side quest also
  needs its character in the army.
- **Place music:** `village` in Millhaven and Cairnford, `city_first_visit` in
  Saltmere, `graveyard_desert` in Ossary and the Ashfields, `dungeon_tense` in the
  Undercroft, `side_quest` on side quests.

**To judge by playing (1064):** where places sit, whether the picture's river and
coast read well, and how hard the random groups feel at their real levels.

**Bots** (`cargo xtask playtest <battle> --bot all`). No softlock: every try on
all three maps ended in a win or a defeat, 40 tries a bot and then 20 counted one
by one (at most 14 turns against the cap of 60). Casual, 40 tries each: road
37/40, ford 38/40, ash 37/40.
The persona bands are **not met**: road and ford read too hard for Normal (18/40,
24/40) and road for Hardcore (21/40), and ash too easy. These runs use the
files' starting levels (the party at Lv 1-3 against Lv 2 Hounds), not a real group
level, so this isn't balance; the bands wait for the real story battles' levels.

**Pictures in the PR:** `docs/img/1070-act1-glyph-start.png`, `-glyph-saltmere.png`,
`-picture-saltmere.png` (game screenshots). They were made with a throwaway debug
start, since reverted, so the panel's second line says "Test World I".

**Sizes.** Estimated before most of the code: about 70 production lines, judged
against 1008's content and core parts, and about 900 changed in all. Measured:
123 added (1.76x), under 2x; 889 changed before this ticket's move.

**Follow-ups:** none new. 0074 adds Steel armour to the stock lists when decided.
