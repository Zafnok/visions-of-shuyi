---
id: "1106"
title: "Input actions become an open set the game defines (ActionSet), with engine UI actions and named layouts"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1105"]
nick_input: none
completed:
---

# 1106 — Input actions: an open set the game defines

## Context

The biggest blocker to a second game (`docs/engine/boundary.md`,
coupling 1): `Action` is this game's closed enum of 18 variants
(`crates/content/src/keymap.rs:338-479`, with `ALL`, `name`,
`is_repeatable`, `is_required`, `is_rebindable`), and the keymap,
`InputState`, `FrameInput`, saved key bindings (`PlayerKeys`,
`ui/src/input/bindings.rs`), help text (`widgets/help.rs`), tips and the
rebinding screen's `ROWS` (`ui/src/screens/key_bindings.rs:56-77`) are
built on it. `Layout` is a closed two-variant enum (right- and
left-handed, `keymap.rs:518-548`).

This ticket makes the mechanism generic **in place** (no crate moves;
1107 moves it). Read the `keyboard-input` skill, ADR-0015, ADR-0031,
ADR-0034, ADR-0053 first: rebinding, help text by keymap and the saved
format must keep working exactly.

## Nick input

None. Nothing a player sees changes.

## Scope

**In:**
- `trait ActionSet: Copy + Ord + Eq + Hash + Debug + 'static` with
  `ALL: &'static [Self]`, `name`, `from_name`, `is_repeatable`,
  `is_required`, `is_rebindable`, and `fn ui(self) -> Option<UiAction>`.
- `enum UiAction { Up, Down, Left, Right, Confirm, Cancel, Debug }`,
  owned by the engine-to-be: what widgets (`Menu::handle`,
  `widgets/menu.rs:151-162`), the rebinding screen and `Game`'s debug
  hook react to. The game's `Action` maps `CursorUp → Up`, etc.
- Generic types: `KeymapDef<A>`, `Keymap<A>`, `InputState<A>`,
  `FrameInput<A>`, `PlayerKeys<A>`, `HelpKeys<A>`, `key_name<A>`.
- The fixed Esc → Cancel and Delete rules (`ui/src/input.rs:33`,
  ADR-0031) become data on the `ActionSet` (`fn fixed_keys()`), not
  code that names this game's actions; `select_action` and
  `end_turn_accept_actions` (`input.rs:319-339`) and the helpers in
  `widgets/help.rs:82-90` move to the game's battle code.
- `Layout` becomes named data: `keymap.ron` lists layouts by id with a
  text key; the picker shows what the file lists. The saved bindings
  (version 2, ADR-0053) keep reading and writing the same names, so
  saved files load unchanged.
- The rebinding screen takes its rows and groups from the game
  (`&[(A, text_key, group)]`).
- The game: `pub type Keymap = shuyi_keymap::Keymap<Action>` style
  aliases in `trpg-ui` so screens keep compiling with field-path edits
  only.

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6). 1034's button chords and 1035's crank input move with input; the `playdate` key table stays the game's.

**Out (do not do):** moving crates (1107); mouse, touch and analog axes
(1121); new actions.

## Implementation steps

1. Add `ActionSet` and `UiAction` in `trpg-content` (beside `Action`),
   implement it for `Action`.
2. Make the keymap types generic; add aliases; fix call sites.
3. Move this game's helper functions to the battle screen.
4. Make `Layout` data; check that a saved bindings file written before
   the change loads unchanged (fixture from `main`).
5. Make the rebinding screen take injected rows.
6. `check-keys` and `check-text` pass; snapshots unchanged.

## Acceptance criteria

- [ ] A unit test defines a three-action `ActionSet` (in a test module)
      and uses `Keymap`, `InputState`, `PlayerKeys` and `HelpKeys` with
      it: rebinding, repeat and help text work.
- [ ] A bindings file saved by the current `main` loads and round-trips
      byte-identically (fixture test).
- [ ] No snapshot changes; every input, controller and rebinding test
      passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the generic types with a toy action set.
- Integration: existing key-binding, layout-picker and controller tests.

## Completion notes
