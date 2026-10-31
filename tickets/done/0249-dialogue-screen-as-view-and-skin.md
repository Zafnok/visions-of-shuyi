---
id: "0249"
title: "The dialogue screen: say what it shows as a view, painted by a skin"
type: feature
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0248"]
nick_input: none
completed: 2026-10-15
---

# 0249 — The dialogue screen as view + skin

## Context

Split (c) of ticket 0242, the last screen to get the split of
[ADR-0054](../../docs/adr/0054-screens-say-what-they-show-as-a-view-a-skin-paints-it.md):
the logic builds a view (plain data), a skin paints it. Nick asked for this
in 0805 (2026-10-04): a bought UI pack must be able to replace the look
"without affecting logic". 0242 did the battle screen's panels, 0248 its
forecast, info, fight box, level-up and rewind.

`DialogueScreen` (`crates/ui/src/screens/dialogue.rs`) still draws in
`draw`: `draw_scene`, `draw_stage`, `draw_box_frame`, `draw_lines` and the
public `draw_told`, which the cinematic's talk shots also call
(`crates/ui/src/cinematic/talk.rs`). It imports `UiColor`, `Rect`,
`BoxStyle`, `Cell` and `draw_portrait`. `DialoguePlayer::current()`
(`crates/ui/src/dialogue.rs`) already gives a `View` of what is said; the
dialogue log (`screens/dialogue/log*`) is already a view and a skin.

## Nick input

None. Nothing the player sees changes.

## Scope

**In:**
- `screens/dialogue/view.rs`: `DialogueView` with what `DialoguePlayer`
  gives (speaker, lines, portraits and their expressions, caption) plus
  what the screen adds: how much of the page is revealed, the open choice
  menu (`MenuView`), the skip question, and the log's `LogView` when open.
- `screens/dialogue/glyph.rs`: `paint(ctx, &view, buf)`; the text box rect
  (`TEXT_BOX`) and the other layout constants move there; portraits
  through `portrait::draw_portrait`.
- `draw_told` becomes a paint of a view the cinematic builds (the talk
  shot keeps calling one function).
- Extend the grep test in `screens/title/tests.rs` to `dialogue.rs`.

**Out (do not do):**
- The dialogue log's look (already a skin).
- Any change to what is shown, its text, keys, timing or layout.
- A second skin or a way to choose skins. The Playdate skin is 1043.

## Implementation steps

1. Write `DialogueView` (and its parts) in `screens/dialogue/view.rs`;
   values are meanings (revealed characters as a count, the speaker's
   side, a portrait's id and expression).
2. `DialogueScreen::view(&self, ctx)`; `draw` becomes
   `glyph::paint(ctx, &self.view(ctx), buf);`.
3. Move `draw_scene`, `draw_stage`, `draw_box_frame`, `draw_lines` and
   `draw_told`'s drawing into `screens/dialogue/glyph.rs`; give
   `cinematic/talk.rs` a view to paint.
4. Add `("dialogue.rs", include_str!("../dialogue.rs"))` to the grep test
   `the_logic_modules_name_no_cells_or_colours`.
5. Tests: behaviour tests in `screens/dialogue/tests.rs` that read cells
   read the view; look tests move to `screens/dialogue/glyph/tests.rs`.
   Add tests of the choice menu, the skip question and the reveal
   appearing and going in the view.

## Acceptance criteria

- [x] `dialogue.rs` doesn't name `UiColor`, `Rect`, `BoxStyle`, `Cell` or
      `draw_portrait`, and its `draw` is one call to its skin (the grep
      test covers it).
- [x] No snapshot file changes.
- [x] The view has unit tests for the reveal, the choices and the skip
      question appearing and going.
- [x] `cargo xtask check-keys` and `check-text` pass; the `check-text`
      limit isn't raised.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the view's content per state; the skin's placement.
- Property: none.
- Snapshot / integration: the existing snapshots (dialogue and cinematic),
  unchanged; Harness tests read the view for behaviour.

## Completion notes

- `screens/dialogue/view.rs`: `DialogueView` (overlay or not, caption,
  the two sides' `FaceView`s with portrait id, expression and name, the
  speaker's side, the text box's `TextView`, the open log's `LogView`).
  `TextView` has the page's lines in full, the revealed characters as a
  count, the replies as a `MenuView`, the skip question (`SkipView`) and,
  while the page waits for a key, a `WaitView` (the key's name, the log's
  hint if its action has a key, and whether the blinking marker is lit).
- `screens/dialogue/glyph.rs`: `paint`, with the layout constants
  (`TEXT_BOX`, the frames, `LISTENER_DIM`) and how the text box grows
  over the replies (`text_box`, which was a public method of the screen).
  `FaceFrame`/`draw_face`/`FRAME`, which the cinematic's characters shot
  uses, moved here too: they are the look.
- `draw_told` and `Told` are gone: the talk shot builds a view with
  `view::told` and calls `glyph::paint`, the same function the screen
  uses (with no choices, no skip question and nothing waiting, `paint`
  draws exactly what `draw_told` did; the existing test that the talk
  shot matches the dialogue screen cell for cell still passes).
- The grep test covers `dialogue.rs`. The behaviour tests in
  `dialogue/tests.rs` and `tests/it/dialogue.rs` read the view; the look
  tests moved to `dialogue/glyph/tests.rs` and build views by hand. New
  view tests: the reveal (`revealed`, `waiting` appearing once the page
  is shown, blinking, going with the page and under the skip question),
  the replies appearing, following the cursor and going, the skip
  question appearing and going, the log appearing and going, no log hint
  without a log key.
- Deviation: the blink's timing (`BLINK_S`) stays in the logic, which
  says whether the marker is lit, as the map scene does for a unit's
  blink; a skin only draws it or not.
- No snapshot changed; nothing the player sees changes. No new
  gameplay rules.
