---
id: "1382"
title: "The UI kit's menu cursor, gauges and icons in the picture look"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["1380"]
nick_input: none
completed:
---

# 1382 — The UI kit's menu cursor, gauges and icons in the picture look

## Context

1379 decides, for the picture look, the menu cursor and focused row
(question 3), the gauges (question 4: HP on the side panel, forecast and
fight, EXP) and whether lists show icons (question 5).
`docs/design/look-and-feel.md`, *Windows (the picture look)*, holds
the answers. 1380 builds the importer (`cargo xtask ui-import`,
`assets-src/ui/`) and the data file (`assets/ui/frames.ron`) this
ticket adds to.

**What 1379 picked** (*Windows (the picture look)*, questions 3 to 5):

- **Cursor** (Nick: *"yes hand + highlighted row"*): the focused row on
  Pixel Quest's `Panels/Titles/F-UI_Title C.png` (dark red; slices 6, 4,
  6, 4), its text white; and a white glove pointing right at it,
  andreyin's CC0 *Hand cursor*
  (`library/opengameart/cursors/andreyin_hand_cursor0000.png`, 27x20
  drawn pixels), at **half the font's pixel** at window scale 2 (the
  font's pixel at scale 1), hanging over the window's left edge, and
  **nudging** toward the row and back (*"the nudge is good"*; starting
  rule: 0, 1, 2, 3, 2, 1 screen pixels, about 0.6 s a cycle, still while
  a key is held to scroll). Nick may swap the picture later for a more
  "professional" hand: keep it one file.
- **HP gauges** everywhere (side panel, forecast, fight box): Pixel
  Quest's `Dynamic Bars/F-UI_DynamicBar_A1.png` made symmetric (its right
  half, from column 70, mirrored to the left), cut at its plain middle so
  the caps show once; the slot (rows 11 to 15) filled by the HP left,
  **green above half, yellow down to a quarter, red at a quarter or
  less** (Nick: *"HP green for high hp, yellow mid hp, red low hp"*; the
  thresholds are a starting rule), fill strips `Dynamic Bars/F-UI_GreenBar.png`,
  `F-UI_GoldBar.png` (or a yellow recolour) and `F-UI_RedBar.png`.
- **Icons** (*"yes, weapons and items"*): ELV Games' Rogue Adventure item
  icons (`library/elv-games/icons/rogue-adventure-items/`), 16x16 at the
  font's pixel, before each weapon or item name.

Today the focused row is a gold bar behind the text (`Wait` in the unit
menu). HP is a flat green bar on the side panel (the map's unit bars
have their own rules, look-and-feel *Units on the map*). There are no
icons in lists.

## Nick input

None (1379 decided). If 1379 answered "no" to a part (for example no
icons), that part is dropped here and the completion notes say so.

## Scope

**In:**
- **Cursor:** the pack's pointer and/or row highlight as sprite items
  on every list and menu that draws a focused row (the engine's list
  widget if it draws it: then the engine change goes through
  `engine-change`), animated only if the pack's cursor is (its frames,
  drawn from the time alone, `shuyi-animate`).
- **Gauges:** the HP bars of the side panel, the forecast and the fight
  box, from the bar picked above (frame + fill cut to the value, its
  colour by the HP left).
- **Icons:** a weapon's or item's icon before its name in the side
  panel and the item, equip and shop lists (spells and arts only if their
  item has one). Each icon named in a data file
  (`assets/ui/icons.ron`: weapon type or item kind → picture), a
  missing one means no icon.
- Their importer entries in `assets-src/ui/`, re-run, pin moved.

**Out (do not do):**
- The map's cursor and range tiles (look-and-feel *Cursor*; MattWalkden's
  pieces are not this ticket).
- The map's unit HP bars (0441's rules stay).
- Frames (1380, 1381); the font (1386); the world map's icons (1082);
  class icons (1006); button pictures in hints (0255).
- The **EXP bar and the volume sliders** (1388 decides them; it adds them
  here if this ticket is still open) and the **stat icons** (1387, the
  same way). Without them, the EXP bar and the sliders keep today's look.

## Implementation steps

1. Estimate the lines (ADR-0092): about 700 (cursor 150, gauges 250,
   icons and their data 300), measured with `cargo xtask code-lines`.
2. Add the pieces to `assets-src/ui/` and `frames.ron` (or `icons.ron`),
   run `ui-import`, push and pin.
3. Cursor, then gauges, then icons, each falling back to today's look
   in the glyph look or without the art.
4. `frame-png` the unit menu, the forecast, the fight, the shop and
   results in both looks; check the bars at 0, 1, half and full HP.

## Lesser targets

Playdate: unchanged. `reduced`/`minimal`: a still cursor (its first
frame) in `reduced`; glyph cursor and bars in `minimal` (ADR-0131).

## Acceptance criteria

- [ ] The focused row of every menu shows the chosen cursor in the
      picture look (a Harness test reads its sprite item).
- [ ] HP bars are drawn from the pack, filled to the value in the
      colour of its band (unit tests of the fill width and colour at 0,
      1, a quarter, half and full).
- [ ] Icons show before each listed weapon and item; a content test
      fails on a weapon or item kind without one.
- [ ] Glyph-look and no-art snapshots unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: gauge fill widths; icon lookup.
- Snapshot / integration: Harness tests for the cursor and bars'
  sprite items; content test for `icons.ron`.

## Completion notes

*(Filled in by the session that completes the ticket.)*
