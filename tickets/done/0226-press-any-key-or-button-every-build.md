---
id: "0226"
title: "\"Press any key or button\" on every build, and it decides whether to show Pick your layout"
type: feature
milestone: M1 Engine
model: opus-5.5
effort: medium
status: done
blocked_by: ["0219", "0220"]
nick_input: sign-off
completed: 2026-10-04
---

# 0226 — "Press any key or button" on every build

## Context

Ticket 0224 added a `Press any key` prompt to the web title (it unlocks
browser sound). In ticket 0032 (controller) Nick revisited that: the prompt
now shows on **every build**, accepts a key **or a controller button**,
shows a keyboard picture and a controller picture, and the first press
decides whether "Pick your layout" is needed. Rules:
[`docs/design/title-screen.md`](../../docs/design/title-screen.md) (*Every
build, keys and buttons*) and
[`docs/design/controls.md`](../../docs/design/controls.md) (*Pick your
layout with a controller*).

Today (0224) `Game::start` pushes the layout picker **on top of** the
title at first launch, so on web the picker comes before the prompt. The
new order is prompt first, then the picker only if a key was pressed.

Builds on 0219 (pad presses reach `Game`) and 0220 (last-used device in
`InputState`). Follow the `keyboard-input` skill (the prompt names no key)
and the `ascii-art` skill (the two pictures).

## Nick input

**Sign-off, two steps:**
1. **Before building:** pick the keyboard and controller pictures from 2–3
   rendered mockups (step 1), in the title screen at real size.
2. **After merge**, on Pages and the Windows download: with a controller,
   press a button at the title (no layout screen), play, then press a key
   (layout screen opens over the game). Pick a layout; it never shows again.

## Scope

**In:**
- The prompt on every build, once per launch; `Press any key or button`
  with the chosen pictures.
- Any key or any pad button dismisses it (the press does nothing else, as
  in 0224).
- First launch order: prompt → (key) "Pick your layout" / (button) menu.
- A pad button while the picker is open closes it without choosing.
- While no layout has been chosen: a key press after pad use opens the
  picker straight away over whatever screen is showing (battle included);
  that key does nothing else.
- Once a layout is picked, the picker never opens by itself again.
- Web: if a pad press doesn't unlock the browser's sound, the music starts
  at the first key press (check Chrome and Firefox; note what happens).

**Out (do not do):**
- Title art (0811). Changing the picker screen itself.
- Button names in help bars (0220). Rebinding (0816).

## Implementation steps

1. **Mockups** (`ascii-art` skill): 2–3 options for a small keyboard
   picture and controller picture on the title line (glyph art in the
   game's font, a few cells tall, e.g. a keycap row `▐q w e▌` and a pad
   outline), rendered at real size in the full title screen. Send to Nick;
   record his pick in `title-screen.md`.
2. `crates/app/src/main.rs`: set `KeyPrompt::Waiting` on every build (not
   only wasm).
3. `crates/ui/src/game.rs`: the prompt is dismissed by a key **or** a pad
   press (0219 already made a pad press dismiss it, so controller players
   weren't stuck on the web title); remember which (`FirstPress::Key | Pad`). Stop pushing the layout
   picker in `Game::start`; instead, when the prompt is dismissed by a key
   and `ctx.layout().is_none()`, push `LayoutPickerScreen`.
4. Layout still unchosen: when 0220's `InputState::device()` changes from a
   pad to the keyboard, push `LayoutPickerScreen` on top of the current
   stack and swallow that key. *Note from 0220 (done):* `Game` copies the
   device into `ctx.device` every frame, but it only follows presses that
   **do** something (bound keys and buttons, ADR-0036), and before a layout
   is chosen only the layout picker's few keys are bound. For "a key was
   pressed" look at the raw `RawInputEvent::Down` events in `Game::step`
   (as the prompt already does), and use `ctx.device` only to know the pad
   was in use before. While the picker is open, a pad press pops
   it without choosing. (So no key acts before a layout is chosen: the
   first one always opens the picker.)
5. `crates/ui/src/screens/title.rs`: the prompt text `Press any key or
   button` (a constant, as `PRESS_ANY_KEY` today) with the pictures from
   step 1.
6. Update `title-screen.md` if any starting rule changes while building.

## Acceptance criteria

- [x] Nick picked the pictures (step 1: "B: three rows"). His sign-off of
      the flow is after merge (it never blocks the PR).
- [x] Snapshot: title waiting with `Press any key or button` and both
      pictures.
- [x] Harness (first launch, no saved layout): key at the prompt → picker
      opens; pad press at the prompt → menu, no picker.
- [x] Harness: picker open, pad press → picker closes, no layout saved.
- [x] Harness: no layout, pad used in battle, then a key → picker opens
      over the battle and the key didn't act; after picking, switching pad
      → key never opens it again.
- [x] Native builds show the prompt (test with the native `Ctx`).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: waiting title.
- Integration (Harness): every flow in the acceptance criteria, pad events
  only where needed (no hardware).

## Completion notes

**Done.** Every build now opens on `Press any key or button` with the
keyboard and controller pictures Nick picked (B, three rows; screenshot
`docs/screenshots/0226-press-any-key-or-button.png`). A key at the prompt
opens "Pick your layout" when none is chosen; a button goes straight to the
menu. A button closes the layout screen without choosing, and until a layout
is picked the next key opens it again over whatever is showing.

**How it is built.**
- `app` sets `KeyPrompt::Waiting` on every build. `Ctx`'s default stays
  `Off`, so tests still start at the menu unless they ask for the prompt
  (`Harness::at_prompt()`, `at_prompt_with_layout()`; they replace
  `on_web…`, and `cargo xtask frame-png --web` is now `--prompt`).
- `Game::start` no longer pushes the picker. `Game::step` does: while no
  layout is chosen and the picker isn't open, a frame with a key press
  pushes it and drops that frame's key presses (they never reach the
  input, so nothing repeats or acts).
- `LayoutPickerScreen` pops on any pad press unless it was opened from
  Options.

**Deviations.**
- No `FirstPress::Key | Pad` is stored (step 3), and `ctx.device` isn't
  consulted (step 4): the ticket's own closing remark, "no key acts before
  a layout is chosen: the first one always opens the picker", gives every
  flow without either, with less state.
- The key that opens the picker marks the keyboard as the device in use
  (`InputState::keyboard_used`), so the picker's help line names keys.

**Tests.** Snapshot `the_title_waits_for_a_key_or_button`; Harness flows in
`crates/ui/tests/it/title.rs` (`first_launch_a_key_at_the_prompt_…`,
`first_launch_a_button_at_the_prompt_…`,
`a_button_closes_the_layout_picker_without_choosing`,
`a_key_mid_battle_opens_the_layout_picker_until_one_is_picked`,
`a_later_launch_shows_the_prompt_then_the_menu`); unit tests in `game.rs`
and `layout_picker.rs`. The picker's own tests open it with a first key.

**Web sound, not checked on real browsers.** Browsers unlock sound on a
key press, click or touch, not on a controller button, so after a button
the menu shows silently and the music starts at the first key press or
click (0224's page script resumes the sound on each of those until it
runs). I had no controller and no Chrome or Firefox with sound blocked to
confirm it: please listen for it at sign-off.

**Claude's starting rules (Nick can veto; also in `title-screen.md` and
`controls.md`):**
- The pictures are dim like the line, don't blink, and keep a 5-cell gap
  from the words in every language.
- First launch, a key at the prompt: the menu and the music start once
  "Pick your layout" closes.
- Until a layout is picked, no key does anything but open "Pick your
  layout", even after the player closed it with the pad.
- The button that closes "Pick your layout" does nothing else; nudging a
  stick counts as a button.
- Opened from Options, the layout screen is steered with the pad as before
  (a button doesn't close it).
- Typing the lead's name with no layout picked opens the layout screen
  first, like any other key.

**Follow-ups.** None. Ticket 0811 (title art) now says to leave room for
the three-row pictures.
