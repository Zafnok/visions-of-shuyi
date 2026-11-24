---
id: "1295"
title: "The unit info screen shows the unit's portrait"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
blocks: ["1064"]
nick_input: none
completed: 2026-11-17
---

# 1295 — The unit info screen shows the unit's portrait

## Context

Nick, 2026-11-17, after playing: "why do none of the units' portraits
show up in unit info (pressing E on them in battle)".

The unit info screen (ticket 0405) was built before portraits existed.
It got a placeholder box with the words from `battle.portrait`
(`crates/ui/src/screens/battle/info.rs`, module doc: "a placeholder
until portraits exist, 0703"). Ticket 0703 added the portrait format
and the level-up page shows faces, but the info screen was never
updated. Its glyph skin still draws a single box and that label in
`PORTRAIT` (`crates/ui/src/screens/battle/glyph/info.rs`, a 24 × 12
cell area at `(2, 1)`), and `InfoView::portrait` is still the
placeholder text (`crates/ui/src/screens/battle/view/info.rs`).

The level-up page already does the lookup this needs:
`progress::portrait_of` and `progress::face_of`
(`crates/ui/src/screens/battle/progress.rs`) find a character's
portrait, with the lead's id standing for the portrait of the gender the
player picked (`LeadProfile::portrait_for`), and its skin draws it with
`crate::portrait::draw_portrait` / `draw_portrait_in`
(`glyph/progress.rs`, `playdate/progress.rs`).

## Nick input

None. The look follows the level-up page, which shows the same portraits
in the same two looks. Whether it looks right goes on the Act 1
playtest's checklist (1064).

## Scope

**In:**
- The info screen's view carries the unit's face (portrait id and
  expression) when its unit is a named character with a portrait, and
  the placeholder text otherwise (generic soldiers and enemies with no
  portrait keep the box and label as today).
- The expression: the portrait's first expression (its neutral one,
  the same fallback `face_of` uses), never a level-up expression.
- The glyph skin draws the face inside `PORTRAIT` with
  `draw_portrait_in`, sized to the area, in both the glyph look and the
  picture look, as the level-up page does.

**Out (do not do):**
- The Playdate's info screen (`playdate/info.rs`): its layout has no
  portrait slot (1039, 1075). Leave it unless a face fits without moving
  anything; if it doesn't, note it in the completion notes.
- The side panel's portrait and any new expression rules.
- Portraits as the engine's picture set (1222) and engine panels (1238).
  Either order works: if 1222 or 1238 lands first, use its types for the
  same thing; if this lands first, they move this code with the rest.
- Drawing or editing any character art (ADR-0040).

## Implementation steps

1. Make the face lookup shared: move `portrait_of` out of
   `progress.rs` (or make it `pub(super)`), and add a small function
   that returns a `FaceView` with the portrait's first expression.
   Reuse `FaceView` (`view/progress.rs`) rather than adding a new type.
2. In `view/info.rs`, add `face: Option<FaceView>` to `InfoView` and keep
   `portrait` as the placeholder text (fix its doc comment). Fill it
   where the view is built (`BattleScreen`'s view, `mod.rs`, the
   `Mode::Info { unit }` arm), from the unit's character id,
   `ctx.game.content.portraits` and `ctx.game.lead` (the `faces` pair
   already built a few lines below).
3. In `glyph/info.rs`, when `view.face` is `Some`, draw the portrait in
   `PORTRAIT` with `draw_portrait_in` (no dim, not mirrored) instead of
   the placeholder label; keep the box or drop it to match the level-up
   page's frame. Otherwise paint as today.
4. Update the module docs of `info.rs` and `glyph/info.rs`.

Expected size: about 40 production lines (one field, one lookup, one
draw call), judged by the level-up page's face (`face_of` and its
skin's draw).

## Acceptance criteria

- [x] Opening unit info on a named character with a portrait shows its
      portrait in both looks; on the lead, the portrait of the picked
      gender.
- [x] A unit with no portrait still shows the placeholder box and label.
- [x] A test reads `InfoView::face` for a character with a portrait, the
      lead (both genders) and a generic unit.
- [x] A look test shows the info screen draws the portrait's picture
      (like `glyph/progress.rs`'s check that `buf` draws a picture of the
      portrait).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the view's face for a character, the lead by gender, and a unit
  with none.
- Snapshot / integration: the info screen's look with a portrait (a
  `cargo shuyi frame-png` picture of it in the PR, cropped to the
  screen, test characters only: ADR-0118).

## Completion notes

- `progress::portrait_of` is now `pub(super)`; the new `info::face`
  uses it and returns a `FaceView` with the portrait's first expression
  (`neutral` for every portrait today). `InfoView` gained `face`; the
  battle screen's view fills it (`mod.rs`, the `Mode::Info` arm), like
  `supports`. `info::view` leaves it `None`.
- The glyph skin keeps the single box and draws the face inside it with
  `draw_portrait_in` (the inner 22 × 10 cells, at the largest whole
  scale that fits, no dim, not mirrored); the placeholder label only
  when there is no face or its portrait is missing. The PC's two looks
  share this screen skin (only the map differs), so both show it.
- Four info text snapshots changed: the `portrait` label left the box
  and a sprite overlay line was added (`portraits/test_lord/neutral.png`,
  `test_knight` in two).
- Playdate (`playdate/info.rs`): left as it is. Its four pages fill the
  window, with no free cell block for a face without moving anything.
- No starting rules: the expression rule (first, neutral) is the
  ticket's.
- For the Act 1 playtest (1064): whether the face in the info box reads
  well at its size, in both looks.
