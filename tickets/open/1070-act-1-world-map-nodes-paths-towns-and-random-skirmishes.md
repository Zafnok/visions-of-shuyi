---
id: "1070"
title: "Act 1's world map: nodes, paths, the four towns' stock and the random-skirmish pool"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1007", "1008", "0044", "0013", "0506", "0320", "0323", "0608"]
nick_input: none
completed:
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

- [ ] The Act 1 map loads and validates; every node is reachable once
      its conditions hold (1007's check).
- [ ] Each town's shop opens with its stock (harness test from a save
      on the map).
- [ ] The random pool's maps each pass a bot run without a softlock.
- [ ] Starting values are listed in the Completion notes (1064's
      checklist).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Content checks: the map, towns, pool maps.
- Integration: travel between the Chapter 4 nodes; a town's shop.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
