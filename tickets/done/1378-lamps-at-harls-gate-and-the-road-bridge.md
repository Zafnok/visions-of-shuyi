---
id: "1378"
title: "The lamps at Harl's gate and the road bridge in Rogue Adventure"
type: content
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
blocks: ["1064"]
nick_input: none
completed: 2026-11-27
---

# 1378 — The lamps at Harl's gate and the road bridge in Rogue Adventure

## Context

Split from 1360 (2026-11-26, Nick: "Split"), the tileset overhaul 0096
decided (`docs/design/look-and-feel.md`, *Chapter 1's map*: "Lamps burn
at the fort's gate and the bridges"). 1360 drew Harl's fort as the stone
keep and Harrowby as a round tower with today's tileset format; the
lamps couldn't be: they stand at places no terrain names (two on the
grass under the keep's south gate, two on the river beside the road
bridge), and a map file names only terrain.

**engine#3247** adds what they need: a map's `look` may place named
pictures on tiles (`look: (tiles: "rogue_adventure", props: { (19, 5):
"lamp" })`, `shuyi_assets::grid_map::MapLook::props`), a tileset's look
says what each name is drawn as (`shuyi_assets::tileset::Look::props`, a
picture at its own size), and the sprite skin's ground paints each one
standing on its tile, its bottom on the tile's bottom, after the look's
layers. It doesn't animate them: the game swaps a picture's frames by
its clock as it does for its tiles (`crates/ui/src/map_view/sprite/moving.rs`,
`trpg_content::tileset::TileFrames`, ADR-0131).

**How 0096's mockups drew the lamps:** `RA_Jungle_Animation.png`
(`assets-private/library/elv-games/tilesets/rogue-adventure/`), 16 × 32,
four frames 32 pixels apart from pixel (0, 256), 150 ms each, standing
on (19, 5) and (21, 5) beside the keep's south gate and (10, 6) and
(12, 6) beside the road bridge (Chapter 1's map), reaching a tile above.

## Nick input

None: 0096 decided it. Nothing to check after merge: `frame-png`
pictures of the gate and the bridge before and after in the PR
(cropped, ADR-0118); what's left to judge in play goes on 1064's
checklist.

## Scope

**In:**
- ~~The pin moved past engine#3247~~: done by the pin move of 2026-11-27 (`docs/engine/pins.md`).
- **The format's props, either order with 1219.** Engine PR #620
  (2281), in the pin since the same move, reads a look's `props` in the
  engine's terrain-tiles format (`shuyi_assets::tileset::LookDef`). If
  1219 is done, the game's tileset is on that format: its props are the
  engine's, and the step below is only the game's checks of them (the
  names the maps place, the frames). If not, write `props` in
  `LookFile` with the engine's field names and shape, so 1219 swaps it
  for the engine's without a change to the files, and add a line to
  1219 that it deletes the game's props too.
- The tileset format (`trpg_content::tileset`, `look.rs`'s `LookFile`):
  a look's `props`, by name, each a picture of the image (its size in
  pixels, inside the image), checked; `assets/tilesets/README.md`; an
  ADR if the format's shape changes (`write-adr`).
- The maps: a `props` name a map's look places must be one its tileset's
  look has (the content checks, with the map's line); `assets/maps/README.md`.
- Moving props: a placed picture whose cell has `frames` moves as a tile
  does, its still the first frame on a lesser tier (ADR-0131).
- The importer (`crates/tools/src/tileset_import.rs`): a look's props in
  the mapping (`assets-src/tilesets/tiny_tales.ron`), packed two cells
  tall, their frames from a strip (the lamp's four frames, 2 tiles
  apart, 150 ms).
- The four lamps placed in Chapter 1's map, in the private repository
  (`content-change`); the art re-packed, pushed, `assets-private.rev`
  moved.

**Out (do not do):** the keep and the tower (1360, done); hills (1359,
1361); trees (1356); other chapters' lamps or props; a lamp's light on
the ground around it.

## Implementation steps

1. ~~Move the pin past engine#3247~~: done by the pin move of
   2026-11-27 (`docs/engine/pins.md`).
2. `props` in the tileset format (either order with 1219, *Scope*) and
   its checks; the map check of the
   names placed; READMEs.
3. Frames for a prop's picture in `moving.rs` (the look's `props`
   swapped as its tiles are).
4. The importer's props and the mapping's lamp; re-pack.
5. Chapter 1's map places the four lamps (private pull request), then
   `frame-png` before and after.

## Acceptance criteria

- [x] Four lamps stand at Harl's south gate and beside the road bridge,
      reaching a tile up, burning (moving) on the PC and still on the
      Playdate's tier.
- [x] A map placing a name its tileset's look lacks fails the content
      check with its line.
- [x] Every other map looks as before.
- [x] Before/after pictures in the PR. (Made, cropped and described
      there; kept local as 1384's were, since they show bought art and a
      real map.)
- [x] The tileset import's tests pass; `assets-private.rev` moved. (By
      its pin file, 0195's flow: the fold moves `assets-private.rev`.)
- [x] All gates in the `run-gates` skill pass.

## Tests required

- `trpg_content::tileset`: a look's props read and checked; a map's
  placed names checked.
- The sprite skin: a placed lamp moves through its frames by the clock
  and stands still on a lesser tier (with the skin, ADR-0038).
- The importer: a prop packed two cells tall with its frames.

## Lesser targets

The Playdate map uses the glyph look (0061) unless 0067 changes that;
the lamps' stills are their first frame. GBA: no target.

## Completion notes

- **The format (either order with 1219: 1219 isn't done).** A look's
  `props`, at a tileset's top for its own look and in each look of
  `looks`, written with the engine's own `PropAt`
  (`shuyi_assets::tileset::PropAt`, engine PR #620): `"lamp": (at: (7,
  24), size: (16, 32))`, the size a tile's if left out. Checked: a name,
  the size's sides 8 to 64 px, the picture inside the image; `props`
  need `terrain`. 1219's Context has a dated line: it deletes the
  game's props reading with the rest of `look.rs`.
- **The maps.** `map::check_props`, run with the tilesets: every name a
  map's look places must be a prop of that look in each tileset that has
  the look, reported at the map's line and column (`ch01.map:14:13: look:
  prop "lantern" at (21, 5) is not a prop of tileset "tiny_tales"'s look
  "rogue_adventure" (its props: "lamp")`). A tileset without the look
  (the test tilesets have no `rogue_adventure`) paints the map with its
  own and isn't asked. The engine already reports a prop off the map.
- **Moving props.** `TileFrames::look_at` swaps `Look::props` with the
  rest, so the existing `frame_skin` (`moving.rs`) paints a lamp's frame
  at the clock and its first frame on a lesser tier or with idle motion
  off. A moving cell drawn only as a prop moves at the prop's size (each
  frame 16 × 32 from its own cell); a cell drawn at a tile's size keeps
  moving as before. `TileFrames` collects from `MovingTile`s, for tests.
  1357's Context has a dated line (engine#2359 covers props).
- **The importer.** A look's `props` in the mapping name each prop's top
  cell; it is packed over the cell below (`Cut::Stacked`), and its
  sheet's `moving` strip moves both. The lamp: `RA_Jungle_Animation.png`
  tile 256, `(frames: 4, step: 32, ms: 150)`. **Deviation:** the ticket
  says the frames are "32 pixels apart"/"2 tiles apart"; they run down
  the column (2 rows apart, as the fire pit's), not across (the cells to
  the right are other lamps). Checked on the sheet.
- **Content:** Zafnok/visions-of-shuyi-assets#44 (the four lamps in
  `ch01.map`, the re-packed `tiny_tales.png`/`.ron`). The lamp is packed
  at (7, 24); the moving cells after it moved one place down the image
  and the rows naming them moved with them. #44 was merged by hand
  before 0195 landed, on top of 1380's art (#43), whose four `game/ui/`
  pictures the game doesn't credit yet, so `--pin` refused the private
  `main`. Converted to 0195's flow (ADR-0135): `assets-private.rev` is
  `main`'s, and the pin file `assets-private.d/t1378-lamps-harls-gate-road-bridge.rev`
  names #44's head (`463632ee`; its deleted branch pushed again there).
  On this merge `private-land.yml` sees #44 merged by hand and folds the
  private `main`.
- **Credits:** the ELV row's comment and `THIRD_PARTY_ASSETS.md` name the
  lamp (`RA_Jungle_Animation.png`); the packed image was already credited.
- **README shots** (1389): `battle-forecast`, `battle-ranges`,
  `spell-menu` and `dialogue` remade. The three battle shots differ only
  in two 32-px columns, the bridge lamps. `dialogue` was already stale
  on `main`: 1384 (merged after 1389) gave the place behind a talk the
  screen clock.
- **Checked:** `frame-png` of `ch01` (picture look, at 1× and 2×): the
  two lamps either side of the keep's south gate on the faces below its
  wall, the two on the river either side of the road bridge, each a tile
  tall above its tile. The pictures stay local (bought art, ADR-0040; a
  real map, ADR-0118).
- **No ADR:** the format's new field is the engine's shape, already
  decided there; the frames' size rule extends ADR-0131 and is written
  in `assets/tilesets/README.md`.

**No gameplay rule decided.** Left to judge in play (1064): the lamps'
look on the river (standing in the water beside the bridge's planks, as
0096's mockups placed them) and on the wall faces at the gate; no light
on the ground around them (out of scope).
