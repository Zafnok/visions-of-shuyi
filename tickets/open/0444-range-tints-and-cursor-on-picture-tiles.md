---
id: "0444"
title: "Range tints and the cursor on picture tiles: the strength and look Nick picks"
type: tuning
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: todo
blocked_by: ["0437"]
nick_input: sign-off
completed:
---

# 0444 — Range tints and the cursor on picture tiles

## Context

Ticket 0437 drew the battle map's terrain with the bought tiles
(ADR-0052). Two things on top of the terrain were left exactly as they
were, because nothing had decided otherwise, and on bright picture tiles
both read worse than on the dark glyph ground:

- **Range tints.** A move, attack, heal or danger range tints its tiles
  75% of the way to the range's colour (`look-and-feel.md`, *Colour*:
  "about 75% *(tunable)*, so the ground stays visible underneath"). That
  was judged on glyphs, where the tint colours a dark background and the
  glyphs stay on top. On a picture tile 75% hides most of the picture:
  with a unit selected, the player can hardly tell forest from plain
  inside its range. Nick was sent the Quick Battle with a unit selected at
  **A 75%, B 50% and C 35%** (2026-10-03).
- **The cursor.** A sprite skin with terrain tiles draws the cursor as
  1-pixel corner marks just inside the tile, 3 pixels long (0433's
  placeholder; the glyph look's marks sit in the gaps the font leaves
  around letters, which pictures don't have). On grass they are hard to
  see at 1×.

Both are look decisions: Nick's.

Code: `crates/ui/src/map_view/glyph.rs` (`OVERLAY_BLEND`, shared by both
looks today), `crates/ui/src/map_view/sprite/ground.rs` (`tints`, the
range's strength on picture tiles), `crates/ui/src/map_view/sprite.rs`
(`paint_own_cursor`, `corner_arms`).

## Nick input

**Sign-off:** Nick says which tint strength he wants on picture tiles (A,
B, C, or another), and whether the cursor needs to be easier to see. If he
wants the cursor changed, render options first (`ask-nick`, with frames
from `cargo xtask frame-png` built with `--features private-assets`, sent
to him and never committed): for example thicker marks, marks with a dark
edge, or a frame round the tile.

If Nick has answered neither when this ticket is picked up, ask him before
writing code.

## Scope

**In:**
- A tint strength of its own for tiles painted as pictures, at the value
  Nick picks. The glyph look keeps 75%.
- The cursor on picture tiles as Nick picks, in all three `CursorStyle`s.
- `docs/design/look-and-feel.md` records both.

**Out (do not do):**
- The glyph look's tints or cursor.
- Lighting (0438) and zoom (0439).
- A player setting for either.

## Implementation steps

1. Record Nick's answers in `docs/design/look-and-feel.md` (*Battle map:
   bought tiles and unit sprites*), and remove the item from *Open
   sub-questions*.
2. `ground.rs`: give `tints` a constant of its own for ranges and flashes
   (e.g. `PICTURE_TINT`), in place of `OVERLAY_BLEND`. The mixed skin
   (sprite units on glyph terrain) keeps the glyph skin's.
3. `sprite.rs`: change `paint_own_cursor` / `corner_arms` to the look Nick
   picked. Keep the cursor inside its tile or cut it at the map area
   (`sprite_skin_paints_only_the_area`).
4. Update the tests that assert the old numbers (`ground.rs`:
   `a_tinted_tile_has_its_own_shape_over_it_in_the_tints_colour`,
   `flashes_tint_towards_the_terrain_colour_then_the_ranges_then_the_glow`;
   `sprite.rs`: `the_cursor_marks_the_corners_inside_its_tile_or_glows`)
   and review the changed snapshots.
5. Render the Quick Battle with a unit selected (`frame-png`, private
   assets) and look at it; send it to Nick.

## Acceptance criteria

- [ ] A range on picture tiles is tinted at the strength Nick picked; on
      the glyph skin and the mixed skin it is unchanged (tests name both
      numbers).
- [ ] The cursor on picture tiles is the look Nick picked, in every
      `CursorStyle`.
- [ ] Every glyph-skin snapshot is unchanged.
- [ ] `look-and-feel.md` records both decisions.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the tint's strength on a picture tile and on a glyph tile; the
  cursor's rectangles for each style.
- Property: the existing `…_paints_only_the_area` properties still pass.
- Snapshot / integration: the sprite-skin snapshots, reviewed.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
