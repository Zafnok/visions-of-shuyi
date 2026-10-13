---
id: "0245"
title: "Lead select, class change and Preparations: say what they show as a view, painted by a skin"
type: feature
milestone: M1 Engine
model: sonnet-5
effort: high
status: done
blocked_by: ["0241"]
nick_input: none
completed: 2026-10-12
---

# 0245 — Lead select, class change and Preparations as view + skin

## Context

The rest of 0241, split off as its scope said to (the eight screens were
more than one PR). 0241 did `mode_select`, `game_over`, `save`, `results`
and `credits`; this ticket does the three biggest screens, with the same
pattern.
[ADR-0054](../../docs/adr/0054-screens-say-what-they-show-as-a-view-a-skin-paints-it.md)
is the pattern; `crates/ui/src/screens/options.rs` with `options/view.rs`
and `options/glyph.rs` is the worked example, and 0240 and 0241 did the
others (`results/` and `save/` show a screen with a table, a question
and a level-up page; `widgets/menu/{view,glyph}.rs` is the `Menu`).

## Nick input

None. Nothing the player sees changes.

## Scope

**In** (all under `crates/ui/src/screens/`): `lead_select.rs` (with its
name box and letter grid), `class_change.rs` (the promotion and reclass
screens) and `preparations.rs`.

**Out (do not do):**
- Any change to what a screen shows, its text, keys or layout.
- A second skin or a way to choose skins.
- The battle screen and the dialogue screen (0242).
- Text literals to the language file (0234; class change's: 0243). *Either
  order works*, as in 0240 and 0241: the view always holds finished
  strings.
- The level-up and promotion pages class change shows with
  `battle::progress::draw`: they are the battle screen's `Progress`,
  converted by 0242. Class change's view holds the `Progress` as
  `ResultsBody::LevelUp` does (0241), and its skin calls `progress::draw`
  until 0242 makes it a view.
- Pictures stay pictures: where a screen places a portrait or another
  sprite (lead select, class change), the view names *what* (the
  portrait's id, its expression, dimmed or not) and the skin places it
  with `portrait::draw_portrait` (ADR-0043).

If this is over ~600 changed lines besides tests, split it again the same
way: do the largest of the three here and write a ticket for the others,
blocked by this one, and make 0242 blocked by that too.

## Implementation steps

1. Per screen: add `<name>/view.rs` (what is shown: headings, rows, values
   as meanings, focus, messages, questions, help), `fn view(&self, ctx)
   -> <Name>View` on the screen, and `<name>/glyph.rs` with `paint(ctx,
   &view, buf)`. Move every layout constant, box, bar and colour into the
   skin. `draw` becomes one call. The three modules are already folders
   (`lead_select/`, `class_change/`, `preparations/` hold `tests.rs` and
   `snapshots/`): the two files go beside them.
2. Values are meanings (ADR-0054): an HP or EXP bar is `{ value, max }`, a
   disabled row is `enabled: false`, not a dim colour; a stat that went up
   is `change: Some(+2)`.
3. Screens using `widgets::Menu` put its `MenuView` in their view and call
   `widgets::menu::glyph::paint` from their skin. When no caller of
   `Menu::draw` is left outside the battle screen, leave it for 0242 to
   remove.
4. Each screen opts into `Screen::as_any`. The game flow hosts them
   (`FlowScreen`, `Stage`): add `FlowScreen` accessors where a test needs
   the hosted screen (as `battle()` and `preparations()` already are).
5. Extend the grep test in `screens/title/tests.rs`
   (`the_logic_modules_name_no_cells_or_colours`) with `lead_select.rs`,
   `class_change.rs` and `preparations.rs`.
6. Tests, as 0241: behaviour tests read the view; look tests move to the
   skin's test module (`glyph/tests.rs` beside `glyph.rs`); no assertion is
   dropped. Helpers a skin test needs from the screen's `tests.rs` are
   `pub(super)`. Tests that move into a folder module keep their snapshots
   (`git mv` the `.snap` files; their content must not change).

## Acceptance criteria

- [x] None of the three logic modules imports `UiColor`, `Rect`,
      `BoxStyle` or `Cell`; each `draw` is one call to its skin (the grep
      test is extended).
- [x] No snapshot file's content changes.
- [x] Each view has a unit test of its content on the first frame and in
      each state the screen has (the name box and the letter grid, a
      promotion question, a reclass choice, a full pack, a benched unit,
      the leave question, …).
- [x] `cargo xtask check-keys` and `check-text` pass; the `check-text`
      limit isn't raised.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each view's content; each skin's placement of what its old tests
  checked.
- Property: none.
- Snapshot / integration: the existing snapshots, unchanged.

## Completion notes

**Split, as the scope said to.** The three screens were far over ~600
changed lines, so this ticket did the largest, **Preparations**; class
change is 0246 and lead select is 0247 (two tickets, not one: the two
together would have been over the limit again). 0242 is now blocked by
both.

What was done:

- `preparations/view.rs`: `PreparationsView` (heading, tabs, whether the
  cursor is on the tabs, the shown tab's body, the message, the leave
  question, the help line). Each list is a `ListView<Row>` with its
  title, every row, the cursor and a `ListState` (`Idle`, `Chosen`,
  `Focused`). Rows are meanings: a unit is `{ name, deployed }`, a slot is
  `{ label, item: Option, wear }`, a stock row is `{ name, detail,
  usable }`, a consumable is `{ name, count, effect }`.
- `preparations/glyph.rs`: `paint`, with every box, row constant, colour,
  column width, the `-` of an empty slot and the scrolling window
  (`window_start`: which rows fit is the box's business).
- `preparations.rs`: `view()`, `draw` is one call, `as_any`; it names no
  cell, colour, rectangle or box style (the grep test covers it).
- `StockRow` holds `name` and `detail` instead of one padded `text` (the
  padding is the skin's).
- Tests: seven view tests (first frame, the cursor through the loadout
  lists, put back, a benched unit, the pack up to full, Options / Fight /
  the leave question, an empty army) and one through the Harness and
  `FlowScreen::preparations()`. The two look tests
  (`a_long_list_scrolls_inside_its_box` and the window half of
  `lists_wrap_and_scroll`) moved to `preparations/glyph/tests.rs`, with
  new tests of the placement and the colours. The four snapshots are
  unchanged.

Acceptance criteria, for Preparations: all met. For lead select and class
change they carry over to 0247 and 0246.

Deviations: none besides the split. No gameplay rule was decided; nothing
the player sees changed.
