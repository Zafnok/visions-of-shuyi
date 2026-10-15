---
id: "0248"
title: "The battle's forecast, unit info, fight box, level-up and rewind screen: say what they show as a view, painted by a skin"
type: feature
milestone: M1 Engine
model: opus-5.5
effort: high
status: done
blocked_by: ["0242"]
nick_input: none
completed: 2026-10-15
---

# 0248 — Forecast, unit info, fight box, EXP/level-up and rewind as view + skin

## Context

Split (b) of ticket 0242, which gives every screen the split the Options
screen has
([ADR-0054](../../docs/adr/0054-screens-say-what-they-show-as-a-view-a-skin-paints-it.md):
a screen's logic builds a view, plain data; a skin paints it). Nick asked
for this in 0805 (2026-10-04): a bought UI pack must be able to replace the
look "without affecting logic".

0242 made `BattleView` (`crates/ui/src/screens/battle/view.rs`) and its
glyph skin (`battle/glyph.rs`) for the side panel, help bar, menus,
questions, objective, banners, notes, tips and heal numbers. Five parts
were left as they were: the view hands the skin the battle's own state for
them in `OldParts` (`forecast`, `info`, `fight_box`, `progress`, `rewind`
plus `state`), and the skin calls their old draw functions:

- `forecast::draw_forecast` (`battle/forecast.rs`): the attack forecast in
  the side panel.
- `info::draw_info` (`battle/info.rs`): the unit info screen.
- `playback::draw_box` (`battle/playback.rs`): the fight box shown when
  combat animations are off.
- `progress::draw` (`battle/progress.rs`): the EXP bar and level-up page.
  The class-change screen also paints it: `ClassChangeBody::Result` holds
  a `Progress` (`screens/class_change/view.rs`).
- `RewindScreen::draw` (`battle/rewind.rs`): the rewind list in the side
  panel.

## Nick input

None. Nothing the player sees changes.

## Scope

**In:**
- A view for each of the five, built from the battle state by the logic
  (`forecast.rs`, `info.rs`, `playback.rs`, `progress.rs`, `rewind.rs` keep
  their rules and text; their drawing moves to the skin). Values are
  meanings: an HP bar is `{ hp, max }` (and the HP after the attack for the
  forecast), a forecast line is numbers and flags (doubles, kill, effective,
  affinity), a stat gain is `{ name, before, gain, at_max }`.
- `BattleView` holds these views instead of `OldParts`; remove `OldParts`.
- The class-change screen's `ClassChangeBody::Result` holds the progress
  view, not `Progress`.
- Tests: behaviour tests read the views; look tests move to the skin.

**Out (do not do):**
- The dialogue screen: ticket 0249.
- Any change to what is shown, its text, keys, timing or layout.
- The combat stage (`battle/stage/`): already a view and a skin (0413).
- A second skin or a way to choose skins. The Playdate skins of these
  parts are ticket 1039.

## Implementation steps

1. For each part, add its view types to `battle/view.rs` (or a
   `battle/<part>/view.rs` if it grows past ~150 lines) and a builder in
   its logic module: `forecast::view(state, words, &Targeting) ->
   ForecastView`, `info::view(state, words, &Unit) -> InfoView`,
   `playback::box_view(words, &Playback) -> FightBoxView`,
   `progress::view(...) -> ProgressView` (portrait ids and expressions as
   data; the skin places them with `portrait::draw_portrait`),
   `RewindScreen::view(words) -> RewindView`.
2. Move the drawing into the battle skin (`battle/glyph.rs`, or
   `battle/glyph/<part>.rs`): the layout constants (`forecast.rs`'s rows
   and columns, `info.rs`'s, `playback.rs`'s `BOX`, `progress.rs`'s, the
   panel's `TEXT_X` / `TEXT_W` already in `glyph.rs`) go with it.
3. `BattleView` gets `forecast`, `info`, `fight_box`, `progress`, `rewind`
   fields of those views; `BattleScreen::view` fills them; `OldParts`
   goes. The class-change view and skin use the progress view.
4. Extend the grep test in `screens/title/tests.rs`
   (`the_logic_modules_name_no_cells_or_colours`) to the five logic
   modules (with `battle/mode.rs`, in the list of modules without a
   `draw`).
5. Tests: the many tests reading forecast rows through cells
   (`attack_tests.rs` `panel(&h, 2)`, `art_tests.rs`, `magic_tests.rs`,
   `rewind_tests.rs` `panel`, `progress_tests.rs` `shows`, `turn_tests.rs`'s
   info screen `shows`) read the views; keep a few in the skin's tests to
   pin where rows are painted. Add view tests of each part appearing and
   going (`battle/view_tests.rs`).

## Acceptance criteria

- [x] `forecast.rs`, `info.rs`, `playback.rs`, `progress.rs` and
      `rewind.rs` don't name `UiColor`, `Rect`, `BoxStyle`, `Cell` or
      `draw_portrait` (the grep test covers them).
- [x] `OldParts` is gone; `battle/glyph.rs` paints the five from views.
- [x] No snapshot file changes.
- [x] `battle/view_tests.rs` has a test of each part appearing and going.
- [x] `cargo xtask check-keys` and `check-text` pass; the `check-text`
      limit isn't raised.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each view's content per state; the skin's placement.
- Property: none.
- Snapshot / integration: the existing snapshots, unchanged; Harness tests
  read the views for behaviour.

## Completion notes

What was done:

- Each of the five parts has a view (plain data) in `battle/view/`:
  `ForecastView` (`forecast.rs`), `InfoView` (`info.rs`), `FightBoxView`
  (`fight_box.rs`), `ProgressView` (`progress.rs`) and `RewindView`
  (`rewind.rs`), re-exported from `battle/view.rs`. The logic modules
  build them: `forecast::view`, `info::view`, `playback::box_view`,
  `progress::view` and `RewindScreen::view`. Values are meanings: HP as
  `{ now, after, max }`, strikes as text plus `effective` / `kills` /
  `after_a_fall` flags, totals as `{ damage, strikes }`, a weapon's
  durability as `left` / `max` or a broken flag, stat gains as `{ name,
  before, gain }`, the EXP bar as `{ exp, max }`, the level-up face as a
  portrait id and expression (`tips::view::FaceView`, reused).
- Their drawing moved to the battle skin, one module each:
  `battle/glyph/{forecast,info,fight_box,progress,rewind}.rs`, with their
  layout constants, the skull, the rewind list's wrapping and scrolling,
  and the EXP bar's cell count (`bar_cells`, now shared with the results
  screen's skin).
- `BattleView` holds `forecast`, `info`, `fight_box`, `progress` and
  `rewind`; `OldParts` is gone, and with it `BattleView`'s lifetime.
- The class change's `ClassChangeBody::Result` and the results screen's
  `ResultsBody::LevelUp` hold the progress view (`None` once the last page
  has closed); both paint it with the battle's progress skin.
- The grep test (`the_logic_modules_name_no_cells_or_colours`) lists the
  five logic modules with `battle/mode.rs` and also checks that none of
  them names `GlyphBuffer`.
- Tests: the Harness tests that read forecast rows, the info screen, the
  fight box, the EXP and level-up pages and the rewind panel through cells
  read the views now (`attack_tests`, `art_tests`, `magic_tests`,
  `progress_tests`, `rewind_tests`, `trigger_tests`, `turn_tests`, the
  class change and results tests). The skin's tests pin where rows go and
  their colours. `view_tests.rs` has a test of each part appearing and
  going (a new one for the fight box and the EXP bar). No snapshot changed.

Deviations:

- The results screen also paints the level-up pages, so it holds the
  progress view too (the ticket only named the class change).
- The views live in `battle/view/<part>.rs` rather than
  `battle/<part>/view.rs`, beside the rest of `BattleView`.
- A stat gain has no `at_max`: the level-up pages show no `MAX` mark
  today, and adding one would change what is shown.
- The inline tests of `progress.rs` and `rewind.rs` moved to
  `progress/tests.rs` and `rewind/tests.rs`: the grep test reads the whole
  file, and their drawing tests moved to the skin.

No gameplay rules were decided. Nothing the player sees changes.
