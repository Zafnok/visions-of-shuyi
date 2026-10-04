---
id: "0240"
title: "The other full-screen menus: say what they show as a view, painted by a skin"
type: feature
milestone: M1 Engine
model: sonnet-5
effort: high
status: todo
blocked_by: ["0239"]
nick_input: none
completed:
---

# 0240 — The other full-screen menus as view + skin

## Context

Second of three tickets that give every screen the split the Options
screen has (ticket 0805, Nick 2026-10-04: a bought UI pack must be able to
replace the look "without affecting logic").
[ADR-0054](../../docs/adr/0054-screens-say-what-they-show-as-a-view-a-skin-paints-it.md)
is the pattern; `crates/ui/src/screens/options.rs` with `options/view.rs`
and `options/glyph.rs` is the worked example, and 0239 did the title, Key
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
- The battle screen and the dialogue screen (0241).
- Text literals to the language file (0234). *Either order works*, as in
  0239: the view always holds finished strings.
- Pictures stay pictures: where a screen places a portrait or another
  sprite (lead select, results' level-up page, class change), the view
  names *what* (the portrait's id, its expression, dimmed or not) and the
  skin places it with `portrait::draw_portrait` (ADR-0043).

If this is over ~600 changed lines besides tests (it likely is: these
files are about 4,500 lines), split it: do `mode_select`, `game_over`,
`save`, `results` and `credits` here, and write a ticket of this same
pattern for `lead_select`, `class_change` and `preparations`, blocked by
this one, and make 0241 blocked by it too.

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
   `Menu::draw` is left outside the battle screen, leave it for 0241 to
   remove.
4. Each screen opts into `Screen::as_any`. The game flow hosts most of
   these (`FlowScreen`, `Stage`): add `FlowScreen` accessors where a test
   needs the hosted screen (as `battle()` and `preparations()` already
   are).
5. Tests: as 0239 step 6. Behaviour tests read the view; look tests move
   to the skin's test module; no assertion is dropped.

## Acceptance criteria

- [ ] None of the logic modules listed imports `UiColor`, `Rect`,
      `BoxStyle` or `Cell`; each `draw` is one call to its skin (extend
      0239's grep test to these files).
- [ ] No snapshot file changes.
- [ ] Each view has a unit test of its content on the first frame and in
      each state the screen has (a question open, a message, a second
      page, the letter grid, a full pack, …).
- [ ] `cargo xtask check-keys` and `check-text` pass; the `check-text`
      limit isn't raised.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each view's content; each skin's placement of what its old tests
  checked.
- Property: none.
- Snapshot / integration: the existing snapshots, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
