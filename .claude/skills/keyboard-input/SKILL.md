---
name: keyboard-input
description: This game's own input rules on top of the engine pack's shuyi-input — its Actions, its keymap file and Nick's default keys, the files allowed to name keys, the Playdate's buttons, chords and crank, and help-bar, tip and dialogue text that names a key. Use when adding or changing a screen's controls, a help bar, a tip or dialogue text that names a key, a new input Action, a default key, or the key-binding config/screen. Not for the input rules every Shuyi game keeps (shuyi-input) or for running the gates (run-gates).
metadata:
  version: "1.0"
---

# Keyboard input in this game

Nick (ticket 0030): "never hardcode keyboard input, draw from the config
which can be set by user".

**Read [`shuyi-input`](../shuyi-input/SKILL.md) first.** It holds every
rule a game on Shuyi Engine keeps: react to actions, never keys; name keys
in text only through the keymap; expect any key or none; the defaults are
the keymap file's; reserved keys are the action set's `fixed_keys`; held
and repeated input comes from the engine; controllers are keys too; how a
game writes its own `ActionSet`; testing with the Harness. This skill
holds only what is this game's: its files, its actions, Nick's defaults,
the Playdate.

Background (in `docs/`): `design/controls.md` (Nick's keys and buttons,
*Rebinding keys*), ADR-0015 (actions, layouts), ADR-0031 (player
bindings, slots, fixed keys, saved config), ADR-0034 (controllers),
ADR-0036 (text follows the device), ADR-0053 (the player's controller
buttons), ADR-0078 (generic input), ADR-0079 (button chords, the
Playdate) and ADR-0086 (the crank).

## The pipeline (only these places know about keys)

```
app/src/keys.rs        macroquad KeyCode → Key / Chord         (engine's shuyi-app: only macroquad key code)
assets/data/keymap.ron default chords per action, per layout   (Nick's keys, controls.md)
player config          Storage key `keybindings`               (keys per layout + shared buttons, 3 slots; input::bindings)
input::{keymap,state}  Keymap + InputState: Chord → Action      (engine; fixed Esc = Cancel: `Action`'s `fixed_keys`)
screens                see only `FrameInput.actions` / `is_held(Action)`
screens::key_bindings  the player edits their slots (engine; reads `pressed_chords` / `pressed_buttons` to capture)
widgets/help.rs, tips  Action → key or button name for text     (`ctx.help_keys()`)
```

Controllers:

```
app/src/pads.rs, pads/   gilrs / browser Gamepad API → PadState   (engine; only pad reads)
input::pad               Pads: PadState → Button down/up          (engine; sticks, Switch swap, merging)
assets/data/keymap.ron   `pad`: default buttons per action; `stick` thresholds; `playdate`
input::bindings          PadBindings: the player's buttons        (engine; one setup for both layouts; saved under `pad`)
input::pad               PadKind::button_name: the only table of button names
```

## Files allowed to name keys and buttons

`Key::…`, `Chord::…` or key names in strings may appear only in:

- `crates/content/src/keymap.rs` and `crates/ui/src/input.rs`;
- test code (`#[cfg(test)]` items, `tests.rs`, `*_tests.rs`, `tests/`,
  `crates/ui/src/harness.rs`);
- the keyboard *picture* of the layout picker (which keys exist and what
  each does), the engine's since ticket 1112: each picture item sits under
  a `// check-keys: keyboard picture` comment, a marker the game allows
  only in `crates/ui/src/screens/layout_picker.rs` and its `glyph.rs`;
- the engine (its `crates/assets/src/keymap.rs`, `crates/ui/src/input/`,
  the platform shell's keyboard and controllers; not scanned). Only the
  engine's `crates/app/src/keys.rs` uses `KeyCode` or macroquad's key reads.

`Button::` may be named where `Key::` may, plus the Playdate's
`crates/playdate/src/pads.rs`.

`cargo shuyi check-keys` enforces this (`shuyi.ron` says what it scans:
`crates/{ui,app,content}/src` and `assets/`). In `assets/` (`.ron`
strings except `keymap.ron`: tips, item and skill descriptions; `.dlg`
dialogue) text is prose, so it fails only on an instruction to press a
key (`press f`, `hold Shift`, `hit Space`), a `Shift+` chord, `WASD` or
`[F]`; "Escape!" or "fires arrows" are fine there. Even the layout
picker's own keys are data: the `layout_picker` section of `keymap.ron`.

## This game's actions

`Action` is in `crates/content/src/keymap.rs`. Rules that name it live
with the game: the split keys in `screens/battle/split_keys.rs`, the Key
bindings rows in `input::KEY_ROWS`. `trpg-content::keymap` and
`trpg-ui::input` re-export the engine's generic types under their old
paths; game code uses the aliases with `Action` (`input::Keymap`,
`screen::FrameInput`, `widgets::help::HelpKeys`). Engine tests use the
engine's sample actions, never this game's. The layouts are
`RIGHT_HANDED` and `LEFT_HANDED`.

Beyond the engine's rules:

- **Default keys and buttons are Nick's.** `keymap.ron` must match the
  tables in `docs/design/controls.md` (*Rebinding keys*, *Controller →
  Default buttons*); a test pins each. Changing a default is a design
  change: ask with the `ask-nick` skill, then edit both files. Never pick
  a default yourself.
- **Reserved keys**: `Shift+Esc` and `Shift+Delete` are ordinary keys.
  The Debug key is not rebindable (and reserved in builds with debug
  tools).
- **No button is fixed or reserved**, so a screen that must work with
  only a controller can't rely on a "safe" button (the Key bindings
  screen binds on release and backs out on a hold).
- **Two places read more than actions.** The Key bindings screen
  (engine's `screens/key_bindings.rs`) captures with `pressed_chords()`,
  `pressed_buttons()` and `released_buttons()`, and asks `input.rs`'s
  helpers (`is_capture_abort`, `is_clear_slot`, `capture_abort_key_name`,
  `clear_slot_key_name`, `CAPTURE_PROMPT`, `CAPTURE_BUTTON_PROMPT`)
  instead of naming keys. **Text boxes** (the lead's name, 0801) ignore
  actions while open, take `FrameInput::text()`, and ask `input::text_key`
  for their fixed Enter / Backspace / Escape, with `input::text_keys_help`
  as help line.

## Text that names a key

- Help bars: `widgets::help::{key_name, all_key_names, cursor_keys_name,
  help_line}`, given `ctx.help_keys()`. Never pass a bare keymap (it
  doesn't compile) and never branch on `ctx.device` in a screen.
- For other bindings, or the keyboard's keys, start from it:
  `ctx.help_keys().of(&keymap)`, `ctx.help_keys().on_keyboard()`.
  `HelpKeys::new` and `HelpKeys::keyboard` give the English words: tests
  and the debug hint only.
- Tips and other data text: `{ActionName}` / `{Cursor}` placeholders
  (`assets/data/tips.ron`, `assets/data/lessons.ron`,
  `crates/ui/src/tips.rs`). Doc comments name the action ("the End turn
  key"), not a key.
- A button's name comes only from `PadKind::button_name` (D-pad
  directions are `↑ ↓ ← →`; the cursor is `D-pad/L-stick`). A new name
  needs its glyphs in the font (`assets/fonts/README.md`, *Our own
  glyphs*); a test checks every name can be drawn.
- Button names are longer than keys (`Options`, `D-pad/L-stick`): a help
  line must fit its row on every pad
  (`the_longest_help_bar_fits_on_every_pad` in `tests/it/controller.rs`).

## The Playdate

- **Button chords** (ticket 1034): a pad binding may be two buttons
  together, `"South+East"`. The Playdate uses `keymap.ron`'s `playdate`
  table instead of `pad` (`Ctx::use_playdate_buttons`, called by its front
  end): A + B ends the turn, hold B + left / right picks a unit. A chord
  button holds back its own action until its partner comes (B until it is
  let go, A for the `chord` window), so Confirm and Cancel can come in a
  frame with no button going down: never assume an action arrives in the
  frame its button went down (`pad_pressed()` covers it). Help text names
  a chord like a button (`A+B`, joined by the language's
  `keys.chord_join`).
- **The crank** (ticket 1035) is two more buttons, `CrankBack` and
  `CrankForward`: each notch wound is a press and a release, bound only in
  the `playdate` table (to `RewindBack` / `RewindForward`, which the Key
  bindings screen leaves out: `input::CRANK_ACTIONS`). Only the front end
  reads the crank, through `RawInputEvent::from_crank`.
- **The system menu** sends actions with `Game::act` (`game::system_menu`);
  `Action::Options` exists for it and has no key anywhere.

## Adding an action

1. `crates/content/src/keymap.rs`: the `Action` variant with a doc comment
   saying what it does, in `Action::ALL`, its RON name, and `ActionSet::ui`
   (usually `None`).
2. **Required or optional** is Nick's call: the list is in `controls.md`
   *Rebinding keys*. If the action isn't covered, ask with `ask-nick`. Set
   `Action::is_required` to match.
3. `assets/data/keymap.ron`: the action in **every** layout (`[]` if it has
   no default key), at most 3 chords (`SLOTS`), keys from `controls.md`;
   its buttons in `pad` (and `playdate` if the Playdate has it).
4. The Key bindings screen: its player-facing label and `RowGroup` in
   `KEY_ROWS` (`crates/ui/src/input.rs`), among the required or the
   optional actions (a test checks each rebindable action is listed once,
   required ones first).
5. Tests: the keymap-matches-design test, a screen test driven by the
   `Action`, and a Harness test using the default key.

## Tests in this game

- Harness scripts (`h.keys("Down f")`) pin the default right-handed
  layout. Rebind with `h.ctx_mut().set_layout_bindings(…)` (an edited
  `ctx.layout_bindings(layout)`), buttons with `set_pad_bindings(…)`.
- Pads: `h.pad("DpadDown South")`, `h.hold_pad("DpadRight", 1.0)`,
  `h.pad("South+East")` for a chord, `h.use_pad(PadKind::PlayStation)`
  (a generic pad by default), `h.crank(-30.0)` for one notch back,
  `h.system_menu(entry)`.
- A controller run compared with a keyboard run compares
  `h.snapshot_as(Device::Keyboard)`.

## Checklist before pushing input work

`shuyi-input`'s checklist, and:

- [ ] Works in both layouts.
- [ ] Defaults match `docs/design/controls.md`.
- [ ] Help text still fits its row on every pad, and reads right on the
      Playdate.
- [ ] `cargo shuyi check-keys` passes.
