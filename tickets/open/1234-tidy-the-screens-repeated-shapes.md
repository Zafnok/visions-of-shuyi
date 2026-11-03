---
id: "1234"
title: "Tidy the screens' repeated shapes: one choice screen, one lookup per word, one blink and strike, shared stage geometry"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1234 — Tidy the screens' repeated shapes

## Context

From the game/engine boundary audit of 2026-11-03. None of this is the
engine's (no game-neutral piece is missing); it is game code written two
or three times (ADR-0092, *game-side structure*):

1. **Three pick-one screens.** `ModeSelectScreen` (`mode_select.rs`) and
   `FamiliarityScreen` (`familiarity.rs`) match line for line: a const
   table of `(value, name key, line key)` (`MODES`, `LEVELS`), `new`
   building a `Menu`, `help`, a `view` with a line per choice, `update`
   = `pick_one` (`screens/mod.rs:83`), the same `Screen` impl.
   `SavePromptScreen` (`save.rs:49`) is the same without the lines and
   without cancel. Both test files have
   `the_focused_line_follows_the_menu`.
2. **`Words` repeats the English fallback.** About 21 methods in
   `crates/ui/src/words.rs` are `match self.pack { Some((lang, code)) =>
   lang.x(code, ..), None => english }`.
3. **The fight's blink and shown strike, twice.** `playback.rs`
   `box_view` and `stage.rs` `view_for` both compute the flash phase
   (`(local / blink) as u32`, `is_multiple_of(2)`) and both match
   `Beat::Result | Beat::Gap` to the strike shown. `result_text`
   (`playback.rs:638`) repeats `stage::blow`'s four-way match.
4. **Two stage skins, one geometry.** `playdate/stage.rs`
   `fighter_sprite`, `reach_px`, `result_row` and `int` copy
   `stage/glyph.rs`'s; only the sizes, the middles, the feet row and the
   flash colour differ. Both skins build the plate's numbers from text
   keys themselves (`plate`, `numbers`), against ADR-0054. The HP bar
   that shades lost HP is written in `glyph/forecast.rs` (`paint_bar`)
   and `playdate.rs` (`hp_bar`).
5. **`overlap()`** in `playdate/forecast.rs:104` is the engine's
   `Rect::intersect(..).is_some()`.

## Nick input

None. No snapshot should change.

## Scope

**In:**
- A game `ChoiceScreen<T>` (`crates/ui/src/screens/choice.rs`) over a
  static spec: name, heading key, help key, actions, rows of `(T, name
  key, Option<line key>)`, whether Cancel backs out, and its `Skins`.
  Mode, familiarity and the save prompt become specs; one view type
  replaces `ModeSelectView`, `FamiliarityView` and `SavePromptView`.
  Keep `pub type` names (`ModeSelectScreen = ChoiceScreen<GameMode>`)
  where callers or tests downcast.
- `Words`: one private helper (`self.pack.map_or(english, |(l, c)| ..)`),
  each method one call.
- `Playback::flash_on(local) -> bool` and `Playback::shown_strike() ->
  Option<&Strike>`, used by `box_view` and `view_for`; `result_text`
  matches on `stage::blow`.
- A small stage geometry (sizes, middles, feet row) per look, with one
  `fighter_sprite`, `reach_px`, `result_row`; the plate's numbers parts
  and the HP name built into `FighterView` by the stage's view code; one
  `hp_loss_bar` in `battle/units.rs` for both skins.
- `overlap()` replaced by `Rect::intersect`.

**Out (do not do):**
- Placing boxes over the map (`menu_origin_in`, `box_top_in`, the corner,
  message and forecast edge choices): look decisions, they stay
  game-side as they are.
- `sound_for_strike` (it classifies by damage, weapon and element).
- The battle notes' wrapping and `unwrap_notes` (1077).
- An engine choice screen (only if a second game wants one).

**Either order with 0860** (the New Game options menu): if 0860 is done,
mode and familiarity are on its menu and the save prompt is the only
pick-one screen left: skip `ChoiceScreen` and do the rest. If not, build
it; 0860 then moves the mode and familiarity rows onto its menu.
**Either order with 1078** (the Playdate's fight at 30 frames a second):
if 1078 is done, the shared `fighter_sprite` keeps what it changed in
`playdate/stage.rs`; if not, 1078 works on the shared function.
**Either order with 1230** (the carve's dead leftovers): if 1230 is
done, `ModeSelectScreen::menu()` is gone: nothing else changes; if not,
`mode_select.rs` goes here with `menu()`, and 1230 skips it.

## Implementation steps

1. Read the files above and `shuyi-screen` (screens, views, skins).
2. `ChoiceScreen`; move the three screens, one at a time, tests green
   after each. Merge the duplicate focused-line test into a
   `choice/tests.rs`.
3. `Words`, then `Playback`, then the stage geometry and plate, the HP
   bar, `overlap`.
4. Run the PC and Playdate snapshot tests; none should change.
5. Measure with `cargo xtask code-lines`.

Expected size: about 215 production lines removed and 70 added (the
choice screen about 50, the geometry about 20); about 80 test lines
removed.

## Acceptance criteria

- [ ] `mode_select.rs` and `familiarity.rs` hold only their specs (or are
      gone), and `SavePromptScreen` is a `ChoiceScreen<bool>`.
- [ ] `grep -c "match self.pack" crates/ui/src/words.rs` is at most 1.
- [ ] `grep -rn "is_multiple_of(2)" crates/ui/src/screens/battle` finds
      only `Playback`.
- [ ] One `fighter_sprite` and one `hp_loss_bar`; `fn overlap` is gone.
- [ ] No snapshot changes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `ChoiceScreen` (Confirm pops with the value, Cancel only where
  the spec allows, the focused line); `flash_on` at each half-period.
- Snapshot: the three screens and both stage skins (unchanged).

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
