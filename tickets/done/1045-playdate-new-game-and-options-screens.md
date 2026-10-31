---
id: "1045"
title: "Playdate: New Game and Options screens in one bit (mode, familiarity, lead select, options, button bindings)"
type: feature
milestone: Post–Chapter 1
model: implementation
effort: high
status: done
blocked_by: ["1037", "1035", "0247"]
nick_input: sign-off
completed: 2026-10-18
---

# 1045 — The Playdate's New Game and Options screens

## Context

Each screen builds a view a skin paints (ADR-0054). The Playdate gets a
one-bit paint of each Chapter 1 screen, chosen by 1037's `SkinSet`, at
50 × 15 cells. This ticket does the screens from New Game to the first
scene, and Options:

| Screen | View today |
| ------ | ---------- |
| Mode select (`screens/mode_select*`) | `ModeSelectView` |
| Familiarity (`screens/familiarity.rs`) | the mode screen's view |
| Lead select (`screens/lead_select.rs`), with the letter grid `NameEntry` | after 0247 |
| Options (`screens/options*`) | `OptionsView` |
| Key bindings (`screens/key_bindings*`), its controller side | `KeyBindingsView` |

The Playdate has no keyboard (1033), so the lead's name is typed on the
letter grid, and Key bindings shows only the Playdate's buttons, chords
(1034) and crank (1035): players rebind everything there
(`keyboard-input` skill). 1035 left the crank's two actions (Rewind
back / Rewind forward, `input::CRANK_ACTIONS`) off `KEY_ROWS` so the
PC's screen keeps its rows; on the Playdate (`Keyboard::Absent`) the
controller side lists them too (their labels in the language file), and
the test `rows_list_every_rebindable_action_once_required_first` says
where.

## Nick input

**Sign-off:** Nick goes through New Game to the first scene and opens
Options in the Simulator or on his Playdate, and comments.

## Scope

**In:**
- A Playdate paint for each screen above, one-bit, 50 × 15, in the
  style 1038 set for windows (black frame, white inside).
- Options: only rows that mean something on the Playdate (e.g. no
  window/fullscreen, no keyboard layout, no colour theme; language only
  if the build has more than one: one build per language, 0061). Decide
  which rows from what each does; the hiding is logic in the options
  screen (view) under the "no keyboard"/Playdate setting, so the PC
  keeps every row. List the hidden rows in the PR.
- Key bindings on the Playdate: the controller side only, naming the
  Playdate's buttons, chords and crank, and capturing a chord when
  rebinding (1034's API).
- Profiles: if 0837 (the Profile screen) has landed, its Playdate paint
  too; if not, 0837 adds a line (see its note).
- Look tests with each skin; behaviour tests read the views.

**Out (do not do):**
- Title screen (1044); preparations, results, saves, game over, credits
  (1046).
- Any PC change.

**Carve (1112):** either order works. If generic screens and widgets
moved to the engine, their Playdate paints are the game's skins over
them.

## Implementation steps

1. Read ADR-0054 and each screen's view; `docs/design/playdate.md`.
2. One paint at a time with its look test.
3. Options rows: decide and test (Playdate hides, PC keeps).
4. `frame-png --playdate` pictures of each screen for the PR.

## Acceptance criteria

- [ ] In the Simulator: New Game → mode → familiarity → lead (name typed
      on the grid) → the first scene; Options opens from the title and
      the system menu; a binding can be changed and restored. *(All but
      two parts: see* In the Simulator *in the notes.)*
- [x] Pictures of each screen in the PR.
- [x] No PC snapshot changes; the PC's Options keeps every row.
- [x] Nick's sign-off comment is in the completion notes (or "no comment
      yet").
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with each skin).
- Integration (harness, Playdate skin set): New Game to the first scene;
  rebinding on Key bindings.

## Completion notes

**Done.**

- **Choosing a screen's skin** (ADR-0091): `screens::Skins<V>` holds a
  screen's glyph and Playdate painters and paints with the one of
  `ctx.game.skin_set`; each screen's `draw` is `SKINS.paint(ctx,
  &self.view(ctx), buf);` (the ADR-0054 grep test accepts it). The
  engine's Options and Key bindings take the game's painter
  (`painted_by`, ADR-0087's screens), so their Playdate paints are the
  game's skins over them, as the carve note asked.
- **The shared one-bit look** (`crates/ui/src/screens/playdate.rs`): the
  white ground; windows white inside a double black frame, the title in
  the top of the frame, a message in the bottom; the focus as a bar
  (white on black); grey dots (`window_text_off`, a new name in
  `playdate.ron`) for what can't be chosen; lists that scroll with the
  focus with `▲`/`▼` in the frame; question and list windows.
- **Playdate paints**, each a `playdate.rs` beside the screen's `glyph.rs`
  with look tests on cells: mode select and familiarity (one paint), the
  lead (faces at 2×, chosen double-framed, the other faded), the letter
  grid and typing box (`widgets/name_entry/playdate.rs`), Options, Key
  bindings, Profile.
- **Options on the Playdate** (`options::shown_rows`): hidden are
  **Layout** (no keyboard: `ctx.keyboard`), and under the Playdate skin
  set **Fullscreen, Colors, Map look, Map zoom** (`options::PC_ONLY`).
  The PC keeps every row (test
  `the_playdate_leaves_out_the_rows_that_mean_nothing_there`). There is no
  Language row on either machine yet (0825); the Playdate's portrait style
  setting is 1042's.
- **Key bindings with no keyboard** (engine): the controller side alone
  (no switch; leaving needs only a button on every required action). The
  game lists `input::PLAYDATE_KEY_ROWS` then: the crank's **Rewind back**
  and **Rewind forward** after Rewind (labels in the language files, en
  and the test pack), no **Zoom map**. The test
  `rows_list_every_rebindable_action_once_required_first` keeps the PC's
  rows; the new `the_playdates_rows_add_the_cranks_and_leave_out_zoom`
  says where the crank's go.
- **Capturing a chord** (engine, 1034's API): a second button down while
  the first is held (or both in one frame) makes the slot that chord; it
  goes in when either is let go; held alone a second, a button still
  backs out. A button of the capture still held when it ends does nothing
  when let go: the Playdate's B is Cancel *on release* (it waits for its
  chord partners), so letting go of B after `B+↓` (or after holding B to
  back out) would otherwise have left the screen. The Simulator found this;
  the harness test now presses the chord as a player does.
- **`cargo shuyi frame-png --playdate`** now starts as the device does (no
  keyboard, the Playdate's pad and buttons), so the pictures show its
  rows and button names; `--pad "East+DpadDown"` presses buttons
  together. Pictures: `docs/design/playdate/1045-*.png`.
- **Integration tests** (`crates/ui/tests/it/playdate.rs`, Playdate
  skins and inks): New Game → mode → familiarity → lead with the name
  spelled on the grid → Chapter 1's opening scene, each screen
  snapshotted and checked against the one-bit reference renderer; Options
  from the title and the system menu, then on Key bindings `B+↓` put in
  Danger zone's slot, saved, and Restore defaults putting it back.

**In the Simulator** (release build, driven by key messages to its
window): New Game → mode → familiarity → lead → the name spelled on the
grid (`EllerA`) → Start (the screen went to the scene's fade; its first
lines weren't shot); Options (from the title) → Key bindings →
`B+↓` into Danger zone's slot → B saved it → Restore defaults put it
back; the Profile screen and its grid with digits. Each screen drew as in
the pictures. The title (1044) is still the PC's skin, cut off, so its
menu can't be seen; the system menu's Options and the opening scene's
lines are checked by the harness tests, not in the Simulator. Nick's
sign-off covers them.

**Deviations.** The ticket's window style was to come from 1038, still
open: these screens use the guide's box's (0066: double black frame,
white inside), which 1038 can take up. Key bindings on the Playdate has
no slot headings (`Button 2` would not fit its column). The engine's
`only_the_first_button_down_is_watched` test is now
`two_buttons_down_together_make_a_chord` (a second button used to be
ignored; it now makes a chord, on every pad).

***the agent's starting rules*** (in `docs/design/playdate.md`, *New Game,
Options, Key bindings and Profile*; Nick can veto): the focus is a bar and
what can't be chosen is grey dots; on the mode and familiarity screens
only the focused choice's explaining line shows; the lead's faces are at
2×, the other one faded; the letter grid has no blank rows between its
letters; Options leaves out Layout, Fullscreen, Colors, Map look and Map
zoom; long lists scroll with the focus in the middle; Key bindings lists
the crank's two as Rewind back / Rewind forward and leaves out Zoom map;
an action that just lost its button is marked `*`; a chord is captured by
holding one button and pressing another.

**Follow-ups:** none.

**Nick's sign-off:** no comment yet.
