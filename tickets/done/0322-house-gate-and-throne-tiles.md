---
id: "0322"
title: "House, gate and throne tiles: terrain data, our own glyphs, stand-in pictures"
type: feature
milestone: M2 Core rules
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0037"]
nick_input: none
completed: 2026-10-23
---

# 0322 — House, gate and throne tiles

## Context

Ticket 0037 (2026-10-22) added three building tiles beside `fort`:
`house`, `gate` and `throne` (`docs/design/terrain.md`, *Building tiles*,
and its movement and combat tables). Chapter 2's save-the-village (0313,
0848) needs the house; Chapter 3's abbey (0849) needs a gate and a
Seize point. No tile heals (`heal_percent: 0`); capturing is not in
Act 1.

The glyph look is **one picture across the tile's two cells** (Nick:
two of the same letter per tile is "visually confusing"), drawn as our
own glyphs in `assets-src/fonts/pad-shapes.bdf` like the gamepad's
button shapes (ticket 0220, ADR-0036). Nick picked these drawings
(16×16 px; `#` is ink in the tile's `fg`, `.` is its `bg`; the left 8
columns are the left cell, the right 8 the right cell):

```
house: lean-to shack       gate: portcullis          throne: crowned throne
fg wood, bg wood_bg        fg fort, bg stone_bg      fg fort, bg floor_bg
................           ................          ...#...##...#...
................           ....########....          ...##.####.##...
................           ..##.#..#..##...          ...##########...
..........###...           .#.#.#..#..#.#..          ...#........#...
......#######...           .###########.#..          ...#..####..#...
...##########...           #..#..#..#..#..#          ...#..#..#..#...
.#############..           #..#..#..#..#..#          ...#..####..#...
..#.#.#.#.#.#...           ################          ...#........#...
..##........#...           #..#..#..#..#..#          .####......####.
..#.....###.#...           #..#..#..#..#..#          .#..########..#.
..#.##..#.#.#...           ################          .#..#......#..#.
..#.##..#.#.#...           #..#..#..#..#..#          .##############.
..#.....#.#.#...           #..#..#..#..#..#          ..##........##..
.##############.           #..#..#..#..#..#          ..##........##..
................           ################          ..##........##..
................           ................          ................
```

The picture look has **no good bought picture** for the house ("too
modern/nice", Nick) or the throne (the bundle has none). Use stand-ins
until 0471 finds better ones: the house is the World Map set's small
single-tile house (`world-map/Set_C_Icons`, row 0), the gate the
Fortress set's open archway (`dungeons-1/Set_A_Fortress1`, tile 26, which
Nick called "serviceable"), the throne a chair from
`overworld/Set_C_Interiors` (row 12) on the floor. Record each as a
stand-in in `look-and-feel.md`'s terrain table, like *Burning*.

## Nick input

None. The rules and the looks are decided (0037). Verified by tests and
`cargo shuyi frame-png` pictures of a test map in both looks in the PR
(`--features private-assets` for the pictures; never commit a mockup
pasted from the bought files). What is left to judge by playing goes on
the Act 1 playtest's checklist (1064).

## Scope

**In:**
- `assets/data/terrain.ron`: `house`, `gate`, `throne` with
  `terrain.md`'s costs (1 for every movement type) and bonuses (house
  0 Def / +10 Avoid; gate +3 / +20; throne +3 / +30), `heal_percent: 0`;
  update the file's header comment ("`fort` is the only building tile").
  Palette names already exist (`wood`, `wood_bg`, `fort`, `stone_bg`,
  `floor_bg`).
- The three drawings as six glyphs (a left and a right cell each) in
  `assets-src/fonts/pad-shapes.bdf`, at Private Use Area code points
  (U+E000 up) or wherever `ATLAS_RANGES` allows; regenerate the atlas
  (`cargo shuyi font-atlas …`, `assets/fonts/README.md`) and add them to
  the README's table and to `REQUIRED_GLYPHS` / `shuyi.ron`'s
  `required_glyphs` if the terrain check needs them there. If the
  engine's atlas tool refuses those code points, follow the
  `engine-change` skill.
- Every map skin (ADR-0038): the glyph skin draws each tile's two
  glyphs as for any terrain; the picture skin
  (`assets-src/tilesets/tiny_tales.ron`, then `cargo xtask
  tileset-import`) draws the stand-ins on grass (house, gate) or floor
  (throne), as `fort` is drawn; the Playdate map skin
  (`crates/ui/src/map_view/playdate*`) draws them as it draws `fort`.
- Terrain-info text for the three tiles where the game names terrain
  (the info panel reads `name` from `terrain.ron`; add language entries if
  the names are translated).
- Tests in `crates/content/src/terrain.rs` like the `fort` rows (costs and
  bonuses), and a test map in `assets-test/` with the three tiles.
- `look-and-feel.md`: the three rows in the picture-look terrain table,
  marked as stand-ins (0471).

**Out (do not do):**
- `Evacuate`, raiders, burning (0313); the action menu and objective line
  (0409); Chapter 2 and 3's maps (0848, 0849).
- Healing (none) and capturing (not in Act 1).
- Finding or buying better pictures (0471).
- Changing Chapter 1's map.

## Implementation steps

1. Read `terrain.md` (*Building tiles*), `look-and-feel.md` (*Colour*,
   the picture-look terrain table), `assets/fonts/README.md`,
   `assets/maps/README.md`.
2. Add the terrains to `terrain.ron`; extend the terrain tests.
3. Draw the glyphs in `pad-shapes.bdf` exactly as above (BDF rows are
   one hex byte per 8-px row, top row first; `BBX 8 16 0 -4` as the
   existing glyphs); regenerate the atlas; render one with `cargo shuyi
   frame-png` and look at it.
4. Picture skin: stand-in layers in `tiny_tales.ron`, import, check the
   tileset tests.
5. Playdate skin.
6. Test map with all three next to a fort; `frame-png` it in both looks
   for the PR.

Expected size: about 60 production lines plus data (three terrain rows,
six glyphs, three tileset layers, the Playdate skin's cases); compare
0422 (the fort's glyph) and 0437's `fort` layer.

## Acceptance criteria

- [x] `terrain.ron` has `house`, `gate`, `throne` with `terrain.md`'s
      numbers; terrain tests cover them.
- [x] The atlas has the six new glyphs; a map with the three tiles draws
      each as one picture in the glyph look (frame-png in the PR).
- [x] The picture skin and the Playdate skin draw all three (frame-png
      in the PR, built with `--features private-assets`).
- [x] `look-and-feel.md` lists the three stand-ins.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: terrain costs and bonuses; the tileset maps each new terrain.
- Snapshot: the glyph skin's look of the three tiles (in
  `crates/ui/src/map_view/glyph*`, the only place a test reads cells).

## Completion notes

Done as specified; data, glyphs and tests, one production line of Rust
(a required-glyphs row). Run on Opus 5.5 (the ticket says Sonnet 5).

- **Terrain:** `house` (0 Def / +10 Avoid), `gate` (+3 / +20), `throne`
  (+3 / +30), all cost 1 for every movement type, `heal_percent: 0`;
  terrain tests cover them.
- **Glyphs:** Nick's three drawings as six glyphs, `☖☗` house, `⛨⛩`
  gate, `♔♕` throne. Placed at code points Terminus lacks inside the
  atlas's ranges: the engine's `ATLAS_RANGES` has no Private Use Area,
  and the ticket allowed "wherever `ATLAS_RANGES` allows", so no engine
  change. The atlas is 557 glyphs; `REQUIRED_GLYPHS` and `shuyi.ron`
  list them.
- **Picture look:** stand-ins in `tiny_tales.ron`: house = World Map
  Icons tile 0 (the small pointed-roof house), gate = Fortress tile 26
  (the archway), throne = Overworld Interiors tile 205 (a cushioned
  chair), on grass, grass and the floor. Re-imported. The private
  repository has the new tileset (398701e), and `assets-private.rev`
  pins it. `look-and-feel.md` lists the three as stand-ins (0471).
- **Playdate skin:** nothing to change; it draws each terrain's glyphs in
  the Playdate palette's inks, which already have `wood`, `fort`,
  `stone_bg`, `floor_bg`.
- **Test map:** `assets-test/maps/test_buildings.map` (a fort, houses,
  an outdoor gate, a gate in a wall, a throne in a hall); a map test
  checks it has all four building tiles. The glyph skin has a test of
  the four tiles' cells and colours.

**Deviations.** The glyphs went in a new `assets-src/fonts/tile-shapes.bdf`
instead of `pad-shapes.bdf`: an xtask test requires every pad-shapes glyph
to be a 7×7 button shape on the baseline, and these are 16×16 pictures.
The README's regeneration command and the atlas test take the third file.
The PR's pictures were made on the Quick Battle's map with the three tiles
put on it for the picture only (`frame-png --map` paints a battle's map,
and no battle uses `test_buildings`), not committed.

**Snapshots changed:** the glyph sampler (557 glyphs) and four map
snapshots whose test-tileset tile positions moved (the regenerated test
tilesets have three more terrains).

**the agent's starting rules:** none. **For the Act 1 playtest (1064):** the
three stand-in pictures, the gate's archway is a small dark opening on
the grass.

Engine friction: estimate 10, measured 1 (under 2×).
