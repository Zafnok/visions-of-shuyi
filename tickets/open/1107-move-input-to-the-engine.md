---
id: "1107"
title: "Move input (keymaps, repeat, controllers, saved bindings, help keys) to the engine"
type: infra
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: todo
blocked_by: ["1106"]
nick_input: none
completed:
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

- [ ] `cargo test -p shuyi-assets -p shuyi-ui` passes with no game
      crate built.
- [ ] No snapshot changes; all tests pass.
- [ ] `docs/engine/boundary.md` rows updated.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: moved tests, rewritten against a test action set where needed.

## Completion notes
