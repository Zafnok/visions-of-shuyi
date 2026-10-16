---
id: "0733"
title: "Chapter 1's places: the yard at Harrowby and the study at Veyne Hall"
type: content
milestone: M6 Story & dialogue
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0732"]
nick_input: none
completed:
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

- [ ] `ch01_intro` (after the captions) and `ch01_tbc` show their places,
      darkened, in both map looks.
- [ ] The all-assets test passes and no battle's terrain changed.
- [ ] `cargo xtask private-assets --pin` accepts the new files (credited).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The all-assets test covers the maps and the script.
- Snapshot: `ch01_intro`'s first talking line over the yard, glyph look.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
