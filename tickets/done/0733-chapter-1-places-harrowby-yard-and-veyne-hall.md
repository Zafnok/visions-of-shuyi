---
id: "0733"
title: "Chapter 1's places: the yard at Harrowby and the study at Veyne Hall"
type: content
milestone: M6 Story & dialogue
model: reasoning
effort: medium
status: done
blocked_by: ["0732"]
nick_input: none
completed: 2026-10-16
---

# 0733 — Chapter 1's places: the yard at Harrowby and the study at Veyne Hall

## Context

Ticket 0029 decided that conversations are drawn over their place,
darkened (`docs/design/look-and-feel.md`, *Places behind conversations*).
0732 adds the `@place` directive and draws a place from a map in
`assets/maps/`. Two Chapter 1 scenes happen away from the battle and need
places of their own (`docs/story/chapters/ch01.md`):

- `ch01_intro`: Harrowby, the yard behind the lead's rented house, at
  dawn. The lead and Hollis spar there (0734 makes them move).
- `ch01_tbc`: Veyne Hall, the lead's father's old study, at night, under
  the Veyne banner. Dace reads a report and paces (0734).

Nick saw mockups made from the bought tilesets' sample maps (a cabin in a
yard; a stone hall) and wants "the tileset that is appropriate". The
places are laid out from the bought tilesets the battle map already uses
(ADR-0040, `assets-private/game/tilesets/tiny_tales.ron`), and from the
Overworld and Tower tilesets in `assets-private/library/tiny-tales/` if
the house, the yard's fence and well, and the study's furniture need tiles
the game's tileset doesn't have yet.

## Nick input

None here; he sees the places with 0734's sign-off.

## Scope

**In:**
- `assets/maps/harrowby_yard.map` and `assets/maps/veyne_study.map`:
  sized to fill the dialogue screen at the battle map's zoom, with the
  glyph look readable (a house wall, a door, a yard, a fence; a room's
  walls, floor, a desk, a banner). Terrain kinds the battle rules don't
  need may be added for decoration only; they must not change any battle.
- The sprite skin's tiles for them: new tiles cut into the private
  tileset (`assets-private/game/tilesets/`), with their pack added to the
  credit's `private` list (`assets/data/credits.ron`, ADR-0051), pinned
  with `cargo xtask private-assets --pin`.
- The glyph skin's glyphs for any new terrain kind (the `ascii-art`
  skill).
- Time of day: dawn for the yard, night for the study, using the sprite
  skin's lighting (ADR-0058) if a map can name its light; otherwise leave
  it and say so in the Completion notes.
- `ch01.dlg`: `@place harrowby_yard x,y` in `ch01_intro` after the
  opening captions (they stay on black), `@place veyne_study x,y` in
  `ch01_tbc`.

**Out (do not do):**
- The moving opening (0734).
- Any change to the battle map `ch01.map`.
- Buying art.

## Implementation steps

1. Look at the sample maps (`assets-private/library/tiny-tales/tilesets/
   overworld/Sample Maps/Cabin_in_the_Woods.png`, `tower/Sample Maps/
   Tower_Interior1.png`) for what the artist's tiles make; pick the tiles.
2. Add the tiles to the private tileset and `tiny_tales.ron`; any new
   terrain kind to `assets/data` with the glyph skin's glyph.
3. Write the two `.map` files; check them in both looks with
   `cargo xtask frame-png` (with `--features private-assets` for the
   pictures).
4. Add the two `@place` lines to `ch01.dlg`.
5. Credits, pin, `THIRD_PARTY_ASSETS.md` if a new pack is used.

## Acceptance criteria

- [x] `ch01_intro` (after the captions) and `ch01_tbc` show their places,
      darkened, in both map looks.
- [x] The all-assets test passes and no battle's terrain changed.
- [x] `cargo xtask private-assets --pin` accepts the new files (credited).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The all-assets test covers the maps and the script.
- Snapshot: `ch01_intro`'s first talking line over the yard, glyph look.

## Completion notes

**Done.** `ch01_intro` (after its three captions, which stay on black) is
drawn over the yard behind the rented house at Harrowby, at dawn, and
`ch01_tbc` over the old lord's study at Veyne Hall, at night, darkened,
in both map looks.

- **Maps**: `assets/maps/harrowby_yard.map` (the back of the house with
  its door, the yard's beaten earth, the practice swords' rack, the well,
  a hedge round it with the way out at the bottom, woods at the corners)
  and `assets/maps/veyne_study.map` (stone walls, the Veyne hanging on
  the far wall, bookshelves, the desk with the report, a rug, the door
  the messenger leaves by). 25 × 16 tiles: the whole screen at the battle
  map's zoom (×2).
- **Decoration terrain** (`terrain.ron`, 11 kinds: `roof`, `house_wall`,
  `house_door`, `hedge`, `well`, `rack`, `desk`, `papers`, `shelf`,
  `banner`, `rug`), marked `decoration: true`. A battle whose map has any
  fails to load (`battle.rs`, tested), so no battle changed; ch01.map is
  untouched. Their glyphs (`╱╲ ││ ▐▌ ∩∩ () †† ╤╤ ≡≡ ╫╫ ▼▼ ◊◊`) are in the
  font already and now in `REQUIRED_GLYPHS`; colours reuse palette names.
- **Pictures**: from the Overworld set (Small Village house, Exteriors
  bush and well, Interiors bookshelf, valance and shield, Tables, Carpet),
  added to `assets-src/tilesets/tiny_tales.ron` and imported into the
  private tileset (pushed, pinned to `c62e3bf`). The packed image was
  already credited, so the credit's `private` list is unchanged; its
  comment and `THIRD_PARTY_ASSETS.md` now name the Overworld set
  (*Overworld* 2.0.1).
- **Time of day**: maps can name their light (ADR-0058), so the yard is
  in a new `dawn` light and the study in `night`. The glyph look ignores
  light, as on the battle map.
- **Tests**: `tests/it/places.rs` (the captions on black, then the yard;
  snapshot of the first talking line over it, glyph look; the study),
  the decoration check, the dawn light, the terrain table. The test
  tilesets were regenerated (more terrain: a second row of tiles, so the
  test sheets' pictures moved; five sprite snapshots changed only in
  where a picture is in the test image).

**Deviations**

- The bundle has **no fence and no banner**. The yard's fence is a
  hedge (a row of the Exteriors set's bushes); the Veyne banner is the
  Interiors set's pale blue valance with a blue-and-gold shield on its
  middle tile. The Tower set wasn't needed.
- In the glyph look a 25 × 16 place shows whole in the middle of the
  screen with black around it, as the battle map does behind
  `ch01_prebattle`.

**For Nick** (with 0734's sign-off): both places are dark: the 0.4
darkening of every place, on top of dawn and night light. The study at
night especially; if it is too dark to read, the light or the darkening
can be raised.

*the agent's starting rules (Nick can veto):*

- Dawn light: a little lighter than dusk and rosy rather than golden
  (`lighting.ron`, tunable).
- The yard has a weapon rack for the practice swords and a well; the
  study has bookshelves and a rug. None of these is in the beat sheet.
- The rug can be walked on like the floor; the other decoration can't
  (it matters only if 0734 walks someone across a place).
