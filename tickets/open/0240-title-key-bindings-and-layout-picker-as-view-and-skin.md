---
id: "0240"
title: "Title, Key bindings and layout picker: say what they show as a view, painted by a skin"
type: feature
milestone: M1 Engine
model: sonnet-5
effort: high
status: todo
blocked_by: ["0805"]
nick_input: none
completed:
---

# 0240 — Title, Key bindings and layout picker as view + skin

## Context

Nick, on ticket 0805 (2026-10-04): the Options screen's "rendering is
decoupled from glyphs so that if we want to swap out the lo-fi retro look
with a purchased pack later (as we did with sprite art) then it is simple
to swap it out without affecting logic", and then: "make a follow up
ticket for title screen and all other heavily coupled glyph rendering <->
logic screens like key rebinding screen to decouple them as well".

[ADR-0054](../../docs/adr/0054-screens-say-what-they-show-as-a-view-a-skin-paints-it.md)
is the pattern: a screen's logic builds a **view** (plain data: what is
shown, no cells or colours) and a **skin** paints it. The Options screen
is the worked example: read `crates/ui/src/screens/options.rs`,
`options/view.rs` and `options/glyph.rs` first.

This ticket does the three screens the player meets first and the shared
`Menu` widget they use. 0241 does the other full-screen menus, 0242 the
battle screen's panels and the dialogue screen.

## Nick input

None. Nothing the player sees changes: every snapshot must stay
byte-for-byte the same.

## Scope

**In:**
- `TitleScreen` (`crates/ui/src/screens/title.rs`).
- `KeyBindingsScreen` (`crates/ui/src/screens/key_bindings.rs`), both its
  keyboard and controller sides.
- `LayoutPickerScreen` (`crates/ui/src/screens/layout_picker.rs`),
  including its keyboard picture.
- `widgets::Menu` (`crates/ui/src/widgets/menu.rs`): its drawing moves to
  a skin function; its state (items, focus) stays.

**Out (do not do):**
- Any change to what a screen shows, its text, its keys or its layout.
- A second skin, a skin trait or a way to choose skins (ADR-0054: not
  until a second skin exists).
- Other screens (0241, 0242).
- Moving text literals into the language file: that is 0234. *Either
  order works:* if 0234 is done, the view's strings come from
  `ctx.text(…)` as it left them; if not, the literals move into the logic
  module unchanged (the view still holds finished strings) and 0234 then
  converts them there. Don't raise `check-text`'s count.
- The title's art (0811) and intro cinematic (0819): *either order
  works.* If they are done, what they show becomes fields of the title's
  view (a backdrop stays a backdrop: the skin places it); if not, they
  add those fields when they come.

## Implementation steps

1. `widgets::Menu`: add `pub fn view(&self) -> MenuView` (`items:
   Vec<MenuItemView { label, suffix, enabled, focused }>`), and move
   `Menu::draw` and `Menu::size` into `crates/ui/src/widgets/menu/glyph.rs`
   as `paint(palette, &MenuView, buf, x, y)` and `size(&MenuView)`. Keep
   `Menu::draw` as a thin call to it until 0241 and 0242 have converted
   every caller; say so in its doc comment.
2. Title: `title/view.rs` with `TitleView { title, subtitle, prompt:
   Option<String> (the "press any key" state), menu: Option<MenuView>,
   notice: Option<String>, help, debug_hint: Option<String> }`;
   `TitleScreen::view(&self, ctx)`; `title/glyph.rs` with `paint`. `draw`
   becomes `glyph::paint(ctx, &self.view(ctx), buf)`. The row constants
   (`TITLE_ROW`, `MENU_ROW`, …) move to the skin.
3. Key bindings: `key_bindings/view.rs` with the side shown (keyboard or
   controller), the title, the column headings, the groups (heading +
   rows), each row's label, its fixed extra key if any, its slots (`Empty`
   / `Bound(name)` / `Capturing(prompt)`), whether it is focused and which
   slot, whether it is unmapped and whether that blocks leaving, whether it
   just lost a key (the flash), the Restore defaults row, the message, the
   open question, the choices list of the controller side, and the help
   line. `key_bindings/glyph.rs` paints it. `PANEL`, `LABEL_X`, `SLOT_W`
   and the other layout constants move to the skin.
4. Layout picker: `layout_picker/view.rs` with the title and, per layout,
   its name, whether it is focused, its legend (keys + what they do, and
   which entry is movement) and its keyboard: `keys: Vec<KeyCapView { key,
   role: Movement | Bound | Unbound }>`. The picture's geometry (which key
   sits where) is the skin's; which keys exist and their role is the
   view's. The `// check-keys: keyboard picture` items move with the code
   that names keys; `cargo xtask check-keys` must still pass, so update
   its allowed path for the marker if the picture's key list moves to
   `layout_picker/glyph.rs` (`crates/xtask/src/check_keys.rs`), with its
   test.
5. Each screen opts into `Screen::as_any` (if it doesn't already) so a
   Harness test can read its view.
6. Tests: move every test that reads cells to learn *what happened* onto
   the view (`h.game().screen::<TitleScreen>()…view(ctx)`, or the unit
   test's own screen). Tests of the look (row positions, colours, the
   highlight bar) move into the skin's test module. Don't delete an
   assertion: restate it against the view or keep it in the skin's tests.
7. `crates/ui/README.md`: in *How to add a screen*, describe the three
   parts and point at Options as the example.

## Acceptance criteria

- [ ] `title.rs`, `key_bindings.rs` and `layout_picker.rs` (the logic
      modules) don't import `UiColor`, `Rect`, `BoxStyle` or `Cell`, and
      their `draw` is one call to their skin's `paint` (a test greps the
      three files for those names, like `check-keys` does for keys).
- [ ] No snapshot file changes (`git diff --stat -- '*.snap'` is empty).
- [ ] Each of the three views has a unit test that checks its content for
      at least: the first frame, a moved focus, and each overlay the
      screen has (the title's notice and prompt; Key bindings' capture,
      question, blocked-leave message and both sides; the picker opened
      from Options).
- [ ] `cargo xtask check-keys` and `check-text` pass; the `check-text`
      limit isn't raised.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each view's content (above); each skin's placement of what its
  old tests checked.
- Property: none.
- Snapshot / integration: the existing snapshots, unchanged; Harness tests
  read views for behaviour.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
