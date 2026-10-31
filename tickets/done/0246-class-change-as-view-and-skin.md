---
id: "0246"
title: "Class change: say what it shows as a view, painted by a skin"
type: feature
milestone: M1 Engine
model: implementation
effort: high
status: done
blocked_by: ["0245"]
nick_input: none
completed: 2026-10-15
---

# 0246 — Class change as view + skin

## Context

The rest of 0245, split off as its scope said to: the three screens were
more than one PR, so 0245 did Preparations (the largest) and this ticket
and 0247 do the other two.
[ADR-0054](../../docs/adr/0054-screens-say-what-they-show-as-a-view-a-skin-paints-it.md)
is the pattern. The closest worked examples:
`crates/ui/src/screens/preparations.rs` with `preparations/view.rs` and
`preparations/glyph.rs` (0245: lists with a cursor, rows that can't be
taken, a question) and `screens/results.rs` with `results/` (0241: a view
that holds the battle screen's `Progress` for its level-up pages).

## Nick input

None. Nothing the player sees changes.

## Scope

**In:** `crates/ui/src/screens/class_change.rs` (the promotion and
reclass screens).

**Out (do not do):**
- Any change to what the screen shows, its text, keys or layout.
- A second skin or a way to choose skins.
- Lead select (0247; *either order works*, they share no code).
- The battle screen and the dialogue screen (0242).
- Text literals to the language file (0234, 0243). *Either order works*:
  the view always holds finished strings.
- The level-up and promotion pages the screen shows with
  `battle::progress::draw`: they are the battle screen's `Progress`,
  converted by 0242. The view holds the `Progress` as
  `ResultsBody::LevelUp` does (`results/view.rs`), and the skin calls
  `progress::draw` until 0242 makes it a view.

## Implementation steps

1. Add `class_change/view.rs`: a `ClassChangeView` with the help line and
   a body that is the columns (`Choosing` and `Confirming`), a result
   page (`Box<Progress>`) or nothing (`Done`). The columns' body says: the
   title; the unit's name and its class-and-level line; the focused
   class's seal as a name and `held: u32` (the skin shows a count of 0 in
   the warning colour: that is a look, so the view holds the number, not a
   colour); the "no class to change to" text when there are no options;
   the columns on show (the logic keeps the window of `COLUMNS`), each
   with `focused`, `offered: bool` (a refused class is `offered: false`,
   not a dim colour), its name, tier text, record text, Mov line, movement
   type, tags, weapon lines, active skill (`Option<String>`), its stat
   rows and the reason it is refused (`Option<String>`); `more_left` and
   `more_right` as `Option<String>`; and the question while it is open
   (the question, its cost, the keys that answer it).
2. A stat row is a meaning (ADR-0054): `{ name, before, after, at_max:
   bool }` (or `change: Option<StatValue>`), not a line with the gain cut
   out of it by searching for `+`. The skin builds the same text with
   `progress::stat_row` and the `MAX` mark and paints the gain and the
   mark in the highlight colour. The mark's text (`class_change.max`) goes
   in the view as a string. Keep `stat_line`, `mov_line` and
   `weapon_lines` (they are public and tested); `stat_line` may move to
   the skin if only the skin uses it, with its tests.
3. Add `fn view(&self, ctx: &Ctx) -> ClassChangeView` and
   `class_change/glyph.rs` with `pub fn paint(ctx, &view, buf)`. Move
   every layout constant (`COLUMN_W`, `COLUMNS_X`, `COLUMNS_Y`,
   `COLUMN_H`, `TEXT_W`, the row constants), box, colour and the `--`
   placeholder into the skin. `COLUMNS` (how many columns are on show) is
   used by the logic's scrolling: it stays in `class_change.rs`. `draw`
   becomes `glyph::paint(ctx, &self.view(ctx), buf);`.
4. `ClassChangeScreen` opts into `Screen::as_any`.
5. Add `("class_change.rs", include_str!("../class_change.rs"))` to the
   grep test in `screens/title/tests.rs`
   (`the_logic_modules_name_no_cells_or_colours`).
6. Tests, as 0245: behaviour tests in `class_change/tests.rs` read the
   view; tests of a look (cells, colours, positions) move to
   `class_change/glyph/tests.rs`, declared with `#[cfg(test)] mod tests;`
   at the end of `glyph.rs`; no assertion is dropped. Helpers a skin test
   needs from `class_change/tests.rs` become `pub(super)`. Snapshot tests
   stay in `class_change/tests.rs` so their `.snap` files keep their names
   and content.
7. Update `crates/ui/README.md` (the *The look is a skin* paragraph: add
   class change to the converted screens).

## Acceptance criteria

- [x] `class_change.rs` doesn't import `UiColor`, `Rect`, `BoxStyle` or
      `Cell`; its `draw` is one call to its skin (the grep test is
      extended).
- [x] No snapshot file's content changes.
- [x] The view has a unit test of its content on the first frame of a
      promotion and of a reclass (saved class level and `new`), with a
      refused class and its reason, with the columns scrolled (`more`
      marks), with a stat at its ceiling, with the question open, and
      while a result page shows.
- [x] `cargo xtask check-keys` and `check-text` pass; the `check-text`
      limit isn't raised.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the view's content; the skin's placement of what the old tests
  checked.
- Property: none.
- Snapshot / integration: the existing snapshots, unchanged.

## Completion notes

Done. `class_change.rs` builds a `ClassChangeView` (`class_change/view.rs`)
and `class_change/glyph.rs` paints it; `draw` is one call to the skin and
the screen opts into `Screen::as_any`. The grep test now covers
`class_change.rs`, and the crate README lists class change with the
converted screens. No snapshot changed.

Deviations, all small:

- The body's columns are `Columns(Box<ColumnsView>)`: clippy's
  `large_enum_variant` refused the unboxed variant.
- The stat row is `StatRowView { name, before, after, at_max }`. The rule
  for `at_max` (the value ends at or past the hard ceiling) stays in the
  logic as `pub fn stat_row(words, kind, before, after, ceiling)`.
  `stat_line` moved to the skin as `stat_line(&StatRowView, mark)`, since
  only the skin writes the text. Its old assertions are split between the
  two (`text_rows` in `tests.rs`, `stat_rows` in `glyph/tests.rs`).
- To write the row from the view's name without searching the line for
  `+`, `battle/progress.rs` gained `stat_row_parts(name, before, gain)`:
  the row up to the gain, and the gain. `progress::stat_row` is now built
  from it and writes exactly what it did before.
- The "Weapons" and "Active" headings are fields of `ColumnsView`
  (`weapons_heading`, `active_heading`). The ticket's list didn't name
  them, but they are text, so the view holds them.
- An option whose class is missing from the content now has no column
  (before, it got an empty box). That can't happen, because the targets
  come from the content's own classes.
- Tests: the cell and colour assertions of the old behaviour tests moved
  to `glyph/tests.rs` unchanged. The behaviour tests read the view
  instead. Added: the seal count's warning colour, every cell painted,
  and the `Done` view.

No gameplay rules decided. Nothing the player sees changes. No follow-up
tickets.
