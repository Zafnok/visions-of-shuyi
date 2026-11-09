---
id: "1089"
title: "World map town pictures cut from the right rows"
type: bug
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-09
---

# 1089 — World map town pictures cut from the right rows

## Context

Nick's playtest of the Act 1 build (2026-11-08):

> "The world map tiling is messed up for some towns with the top and
> bottom tiles swapped making it appear amateur"

**How the picture look draws places.** `world_map/picture.rs`
(`crates/ui/src/screens/world_map/picture.rs` ~:36) draws one picture,
`worldmaps/act1.png` (bought art, in the private repository's `game/`),
stretched over the map, then each place's 16-px symbol plate over it. The
picture is made by the private repository's
`tools/worldmaps/act1_picture.py` (ticket 1070), from the bought World Map
pack. Its `PLACES` table crops each place's picture from the sheet
`Set_C_Icons` as `(column, row, columns, rows)` of 16-px tiles and pastes
it centred on the place (`y = py·16 + 8 − rows·8`):

```
'greywater': (6, 11, 1, 3), 'millhaven': (0, 2, 2, 2), 'cairnford': (4, 3, 2, 2),
'saltmere': (2, 8, 2, 3), 'ossary': (8, 0, 2, 3), 'veyne_hall': (6, 8, 2, 3),
```

**What is wrong.** On `Set_C_Icons` (256 × 256,
`assets-private/library/tiny-tales/tilesets/world-map/Images/`) the
two-by-two towns start on rows 2, 4 and 6 (columns 0, 2, 4, 6), the
castles are two by three from row 8, the fortresses two by three from
row 0 (columns 8 and 10), the towers one by three from row 11 (columns 6
and 7). Cairnford's crop starts on **row 3**: its top half is the bottom
of the town at (4, 2) and its bottom half the top of the town at (4, 4),
the "top and bottom swapped". The other five crops start on an icon's
top row (checked against the sheet on 2026-11-08). The whole picture is
drawn whatever is open, so Cairnford shows already in Chapter 4.

A second thing that can read as broken: the 16-px plate is drawn over the
middle of each 32–48 px picture, hiding its middle (1082 would replace the
picture look's plates with icons; Nick picks them there).

AGENTS.md, *Bought art*: never commit the sheet, a crop of it or a mockup
pasted from it; a picture of the running game may be shown.

## Nick input

None. Verified by the tool's check below and a picture of the running
game's world map in the picture look attached to the PR. Which town picture
Cairnford gets is *the agent's starting rule*: the town at (4, 4) (the one
whose roof shows in Cairnford's spot today; Millhaven's is (0, 2)); on the
Act 1 playtest's checklist (1064).

## Scope

**In (the private repository, then the pin):**
- `tools/worldmaps/act1_picture.py`: Cairnford's crop on an icon's top row.
- **A check in the tool** that runs before it draws: a table of the
  sheet's icons (each icon's column, row, columns and rows, as above), and
  an assertion that every `PLACES` entry is exactly one of them, naming
  the place and the nearest icon if not. Also a pixel check that a crop's
  bottom row of pixels isn't fully opaque (a cut through an icon's middle
  is, today's Cairnford crop is).
- Rerun the tool (`python act1_picture.py <act1.ron> <game/worldmaps/act1.png>`),
  commit the picture and the tool in the private repository, push, then
  in the game `cargo shuyi private-assets --pin` and commit
  `assets-private.rev` (AGENTS.md, *Bought art*).

**Out (do not do):**
- The plates (1082).
- The glyph look (one `⌂` on a plate, `world_map/glyph.rs`).
- New pictures for places that have none.

## Implementation steps

1. `cargo shuyi private-assets --library` for the sheet; read the tool.
2. Write the check; run it: it fails on Cairnford.
3. Fix the entry; rerun the tool; look at the result locally.
4. Push the private repository; pin; a picture of the running game's world
   map (`cargo run -p trpg-app --features private-assets`, or `cargo shuyi
   frame-png` built with that feature) attached to the PR, not committed.

## Acceptance criteria

- [x] The tool's check fails on today's table (shown in the PR text) and
      passes on the fixed one.
- [x] `assets-private.rev` names the private commit with the new picture.
- [x] The game's private-assets test passes (`cargo test -p trpg-content
      --features private-assets --test it private_assets::`, the
      `run-gates` skill's step for a change to `assets-private/game/`).
- [x] A picture of the world map with Cairnford whole is attached to the
      PR.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The tool's own check (private repository).
- The private-assets tests in the game (`run-gates`).

## Completion notes

Done 2026-11-09. Private repository commit `ef5116e` (pinned in
`assets-private.rev`):

- `tools/worldmaps/act1_picture.py` gained `ICON_TABLE` (the 38 icons of
  `Set_C_Icons`, read off the sheet: the small houses of rows 0, 1 and 11,
  the fortresses, the two-by-two towns of rows 2, 4, 6 and the snow towns
  of row 12, the castles of row 8, the snow castle and the two towers of
  row 11) and `check_places()`, run first by `main`. On today's table it
  stops with `cairnford: (4, 3, 2, 2) is not an icon of Set_C_Icons;
  nearest is (4, 2, 2, 2)` and exit 1; on the fixed one it passes.
- Cairnford's crop is `(4, 4, 2, 2)` (*the agent's starting rule* from the
  ticket: the green-and-blue town whose roof showed in its spot). The
  regenerated `game/worldmaps/act1.png` differs from the old one only in
  Cairnford's 32 × 32 px spot.

**Deviation:** the pixel check (a crop whose bottom row of pixels is fully
opaque) is in, but it does **not** fire on the old Cairnford crop: the
middle of the town at (4, 4) has transparent pixels at its edges. The table
check is what catches it. No icon in the table ends on an opaque row, so
the pixel check stays as a second guard for a crop of the wrong size.

**Picture of the game:** `cargo shuyi frame-png` built with
`private-assets`, through the debug menu's "Test world map" pointed at
`ch04` by a throwaway edit of `TEST_WORLD_CHAPTER` (reverted), as 1070
did. Not committed (the ticket: attached, not committed); `gh` can't
upload a picture, so it went to Nick with the PR link.

For the Act 1 playtest (1064): which town picture Cairnford shows, and the
16-px plates over the middle of every place's picture (1082).
