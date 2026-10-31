---
id: "1107"
title: "Move input (keymaps, repeat, controllers, saved bindings, help keys) to the engine"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["1106"]
nick_input: none
completed: 2026-10-16
---

# 1107 — Move input to the engine

## Context

After 1106 the input mechanism is generic over `ActionSet`. ADR-0063 §2:
keymap *files* (keys, chords, controller buttons, sticks, layouts,
validation; `content/src/keymap.rs` minus `Action`) go to
`shuyi-assets`; the runtime (`InputState`, repeat, pads, saved
bindings, help keys; `ui/src/input*`, `widgets/help.rs`) goes to a new
`shuyi-ui` crate. `shuyi-ui` starts here; 1108 and later add to it.

## Nick input

None.

## Scope

**In:**
- New crate `engine/crates/ui` (package `shuyi-ui`), depending on
  `shuyi-assets` only.
- Move the keymap format (about 740 lines plus its 1,005 lines of tests)
  to `shuyi-assets::keymap`; `Action`, its `ActionSet` impl and the
  game's `keymap.ron` stay in `trpg-content`.
- Move `input.rs`, `input/bindings`, `input/pad.rs` and the generic part
  of `widgets/help.rs` to `shuyi-ui::input`.
- Re-export or alias under the old paths.

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6). 1034's button chords and 1035's crank input move with input; the `playdate` key table stays the game's.

**Out (do not do):** the rebinding and layout-picker *screens* (1112);
`app`'s key and pad code (1110).

## Implementation steps

1. Create `shuyi-ui`; add it to `engine/README.md`.
2. Move modules with their tests; the tests that use the game's
   `Action` stay in the game or are rewritten with a test action set.
3. Run `check-engine-boundary`, all tests, the WASM build.

## Acceptance criteria

- [x] `cargo test -p shuyi-assets -p shuyi-ui` passes with no game
      crate built.
- [x] No snapshot changes; all tests pass.
- [x] `docs/engine/boundary.md` rows updated.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: moved tests, rewritten against a test action set where needed.

## Completion notes

Stacked on 1106's PR.

- **`shuyi-assets::keymap`**: the keymap file format (`git mv` of
  `content/src/keymap/format.rs`) with `action_set` (`ActionSet`,
  `UiAction`). `trpg-content::keymap` re-exports it, and `keymap::format`
  and `keymap::action_set` are the engine's modules under the old paths.
  `Action`, `KeyGroup`, the two layouts, `load_keymap` and `keymap.ron`
  stay the game's.
- **`shuyi-ui`** (new, `engine/crates/ui`, depends on `shuyi-assets`
  only): `input` with `bindings`, `fixed`, `frame`, `help_keys`, `keymap`,
  `pad` and `state` (`git mv` from `ui/src/input/`). `trpg-ui::input`
  re-exports the submodules and keeps the aliases with `Action`,
  `KEY_ROWS` and the text keys; `widgets/help.rs` keeps the `HelpKeys`
  alias and re-exports the helpers.
- **Tests:** the engine's tests run on a **sample game's actions**
  (`shuyi_assets::keymap::sample`, behind the `sample-actions` feature for
  `shuyi-ui`'s tests): the same nineteen actions and a copy of the
  keymap file, so the format's, input's, bindings', pads' and help text's
  tests moved almost unchanged, without the game. Tests of this game's own
  keys (the design's defaults, required actions, the UI mapping) stay in
  `trpg-content`. Two tests stay in the game because they need it: every
  button name is in the game's font (`trpg-ui::input`), and a menu driven
  by another game's actions (`widgets/menu.rs`, now the sample's, since
  the menu stays in the game until 1108).
- **Deviations:** `LayoutBindings::from_def`, `InputState::starting_on` and
  `InputState::keyboard_used` are now `pub` (the game calls them across the
  crate line), and `PadKind::ALL` and `pad::DIRECTION_SETS` are `pub` for
  the font test. `shuyi-ui` has its own `DEBUG_TOOLS` and `debug-tools`
  feature, which `trpg-ui`'s `debug-tools` turns on. `hashbrown` joins the
  engine's allowed crates (MIT OR Apache-2.0). `keymap/sample.rs` is
  excluded from mutation testing (test data). `check-keys` no longer lists
  the moved folders; it doesn't scan `engine/`. CI and `run-gates` build
  `shuyi-ui` without `std`.

No gameplay rules were decided. Nick input: none.
