---
id: "1178"
title: "Arrows at the map's edges on the Playdate"
type: feature
milestone: Post–Chapter 1
model: reasoning
effort: medium
status: todo
blocked_by: ["1038"]
nick_input: decision
completed:
---

# 1178 — Arrows at the map's edges on the Playdate

## Context

Ticket 0449 put arrows at the edges of the battle map's view where the
map goes on past it (`docs/design/look-and-feel.md`, *Arrows at the map's
edges*): the battle screen sets the scene's `more`
(`BattleScreen::scene_of_size`, only while the player browses), and the
glyph skin, the sprite skin and sprite units on glyph terrain paint them
in their unit painters' `paint_over` (`crates/ui/src/map_view/glyph.rs`,
`sprite.rs`, the placement and the fade in `map_view/edges.rs`).

The Playdate's map skin (`crates/ui/src/map_view/playdate.rs`,
`OneBitUnits`) paints none: 0449's scope named the three PC skins only.
The Playdate shows 24 × 15 tiles (`docs/design/playdate.md`), and Chapter
1's map is 24 × 16, so a row is off screen and nothing says so. Act 1's
later maps may be bigger.

1038 lays out the Playdate's battle screen (the full map and corner
pop-ups), which decides how many tiles it shows: needed first.

## Nick input

**Decision**, only if the one-bit screen can't follow 0449's look: Nick
picked a big arrow fading smoothly from full to 35% and back once a
second (B2), in the cursor's colour. One bit has no 35%. Render the
Playdate's map (`cargo shuyi frame-png --playdate --map`, Chapter 1) with
the arrow glyph (`↑ ↓ ← →`, as the PC glyph look) in the skin's inks, and
ask (`ask-nick` skill) between, at least: **A** a steady arrow (no
fade); **B** an arrow blinking on and off once a second, like the
Playdate cursor's marks; **C** describe your own. Record the answer in
`docs/design/playdate.md`.

## Scope

**In:**
- `OneBitUnits::paint_over`: the arrow glyphs Nick picks, on the cells
  the PC glyph skin uses (`edges::arrows`, `glyph::edge_arrow`), in the
  Playdate's inks, driven by the scene's `clock_ms` if they blink.

**Out (do not do):**
- When the scene has arrows, or where they sit: 0449's.
- The camera and when it scrolls (0072).
- The Playdate battle screen's layout (1038).

## Implementation steps

1. Ask Nick as above; record the answer.
2. Add `paint_over` to `OneBitUnits` (`crates/ui/src/map_view/playdate.rs`)
   reusing `edges::arrows` and `glyph::edge_arrow`.
3. Add the `more` case to the Playdate skin's tests
   (`crates/ui/src/map_view/playdate/tests.rs`) and check its
   paints-only-the-area property still holds (`any_scene` sets `more`).
4. Render it with `cargo shuyi frame-png --playdate` and look at it.

## Acceptance criteria

- [ ] Nick's answer is in `docs/design/playdate.md` with his words.
- [ ] On Chapter 1 on the Playdate, the scene's `more` is drawn as arrows
      (a test of the skin reading cells), and a map that fits has none.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the arrows' cells and inks under the Playdate skin, and their
  blink if Nick picks one.
- Property: the skin paints only its area with `more` set.
- Snapshot: Chapter 1's Playdate map frame.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
