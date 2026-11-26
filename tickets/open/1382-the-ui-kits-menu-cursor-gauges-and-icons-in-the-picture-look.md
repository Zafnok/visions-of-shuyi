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
  box, and the EXP bar (results and level-up), from the pack's bar
  pictures (frame + fill cut to the value).
- **Icons** (only if 1379 said yes): weapon type, item or spell icon
  before the name in the item, equip, arts, skill and shop lists, and
  the stat names if picked. Each icon named in a data file
  (`assets/ui/icons.ron`: weapon type or item kind → picture), a
  missing one means no icon.
- Their importer entries in `assets-src/ui/`, re-run, pin moved.

**Out (do not do):**
- The map's cursor and range tiles (look-and-feel *Cursor*; MattWalkden's
  pieces are not this ticket).
- The map's unit HP bars (0441's rules stay).
- Frames (1380, 1381); the font; the world map's icons (1082); class
  icons (1006); button pictures in hints (0255).

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
- [ ] HP and EXP bars are drawn from the pack, filled to the value
      (unit tests of the fill width at 0, 1, half and full).
- [ ] Icons, if chosen, show before each listed weapon, item or spell; a
      content test fails on a weapon type without one.
- [ ] Glyph-look and no-art snapshots unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: gauge fill widths; icon lookup.
- Snapshot / integration: Harness tests for the cursor and bars'
  sprite items; content test for `icons.ron`.

## Completion notes

*(Filled in by the session that completes the ticket.)*
