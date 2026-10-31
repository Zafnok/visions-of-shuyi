---
id: "0241"
title: "The other full-screen menus: say what they show as a view, painted by a skin"
type: feature
milestone: M1 Engine
model: implementation
effort: high
status: done
blocked_by: ["0240"]
nick_input: none
completed: 2026-10-11
---

# 0241 — The other full-screen menus as view + skin

## Context

Second of three tickets that give every screen the split the Options
screen has (ticket 0805, Nick 2026-10-04: a bought UI pack must be able to
replace the look "without affecting logic").
[ADR-0054](../../docs/adr/0054-screens-say-what-they-show-as-a-view-a-skin-paints-it.md)
is the pattern; `crates/ui/src/screens/options.rs` with `options/view.rs`
and `options/glyph.rs` is the worked example, and 0240 did the title, Key
bindings, the layout picker and the `Menu` widget (`MenuView`,
`widgets/menu/glyph.rs`).

## Nick input

None. Nothing the player sees changes.

## Scope

**In** (all under `crates/ui/src/screens/`):
- `mode_select.rs`, `lead_select.rs` (with its name box and letter grid),
  `game_over.rs` (Game Over and "To be continued"), `save.rs` (the save
  prompt and the slot picker), `results.rs`, `credits.rs`,
  `class_change.rs`, `preparations.rs`.

**Out (do not do):**
- Any change to what a screen shows, its text, keys or layout.
- A second skin or a way to choose skins.
- The battle screen and the dialogue screen (0242).
- Text literals to the language file (0234; class change's: 0243). *Either order works*, as in
  0240: the view always holds finished strings.
- Pictures stay pictures: where a screen places a portrait or another
  sprite (lead select, results' level-up page, class change), the view
  names *what* (the portrait's id, its expression, dimmed or not) and the
  skin places it with `portrait::draw_portrait` (ADR-0043).

If this is over ~600 changed lines besides tests (it likely is: these
files are about 4,500 lines), split it: do `mode_select`, `game_over`,
`save`, `results` and `credits` here, and write a ticket of this same
pattern for `lead_select`, `class_change` and `preparations`, blocked by
this one, and make 0242 blocked by it too.

## Implementation steps

1. Per screen, in the order above: add `<name>/view.rs` (what is shown:
   headings, rows, values as meanings, focus, messages, questions, help),
   `fn view(&self, ctx) -> <Name>View` on the screen, and
   `<name>/glyph.rs` with `paint(ctx, &view, buf)`. Move every layout
   constant, box, bar and colour into the skin. `draw` becomes one call.
   A screen whose module is already a folder (`class_change`,
   `preparations`, `lead_select`) gets the two files beside its tests.
2. Values are meanings (ADR-0054): an EXP bar is `{ filled, total }` or a
   fraction, not a string of blocks; a disabled row is `enabled: false`,
   not a dim colour; a stat that went up is `change: Some(+2)`.
3. Screens using `widgets::Menu` put its `MenuView` in their view and
   call `widgets::menu::glyph::paint` from their skin. When no caller of
   `Menu::draw` is left outside the battle screen, leave it for 0242 to
   remove.
4. Each screen opts into `Screen::as_any`. The game flow hosts most of
   these (`FlowScreen`, `Stage`): add `FlowScreen` accessors where a test
   needs the hosted screen (as `battle()` and `preparations()` already
   are).
5. Tests: as 0240 step 6. Behaviour tests read the view; look tests move
   to the skin's test module; no assertion is dropped.

## Acceptance criteria

- [x] None of the logic modules listed imports `UiColor`, `Rect`,
      `BoxStyle` or `Cell`; each `draw` is one call to its skin (extend
      0240's grep test to these files). *For the five screens done here;
      `lead_select`, `class_change` and `preparations` are 0245 (the split
      the scope asked for).*
- [x] No snapshot file changes. *Contents unchanged; the five snapshots of
      `mode_select` and `game_over` moved with their tests into the new
      folders (`git mv`).*
- [x] Each view has a unit test of its content on the first frame and in
      each state the screen has (a question open, a message, a second
      page, the letter grid, a full pack, …). *For the five screens.*
- [x] `cargo xtask check-keys` and `check-text` pass; the `check-text`
      limit isn't raised.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each view's content; each skin's placement of what its old tests
  checked.
- Property: none.
- Snapshot / integration: the existing snapshots, unchanged.

## Completion notes

Done for `mode_select`, `game_over` (Game Over and "To be continued"),
`save` (the prompt and the slot picker), `results` and `credits`. Each is
now `<name>.rs` (logic, `view()`, one-call `draw`, `as_any`),
`<name>/view.rs` and `<name>/glyph.rs` (with `glyph/tests.rs` for the
bigger skins); the grep test in `screens/title/tests.rs` covers them.
Nothing the player sees changed: no snapshot's content moved.

- **Split, as the scope said:** `lead_select`, `class_change` and
  `preparations` are ticket 0245 (blocked by this one); 0242 is now blocked
  by 0245 too, and its and 0243's text names it.
- Values are meanings: the results EXP bar is `{ exp, exp_max }`, a slot is
  `Empty` / `Saved` / `Unreadable`, the save range is the finished text
  (`6-25 of 30`) and the skin adds the spaces around it.
- Scrolling stays in the logic: the slot picker's window of 20 slots and
  the credits' wrapped rows (with the width of each kind, pinned to the
  panel by a skin test) are screen capacities, not looks.
- The results' level-up page is the battle screen's `Progress` (drawn by
  `progress::draw`, which 0242 converts), so `ResultsBody::LevelUp` holds a
  clone of it for now.
- The `Menu::draw` callers left outside the battle screen are the three
  screens of 0245; 0242 removes it.
- Snapshots of `mode_select` and `game_over` moved into the screens'
  folders (insta looks beside the test file); content identical.
- No game-design rules decided. Nothing for Nick to check.
