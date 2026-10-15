# ADR-0068: Key groups: a key or button on one action per group, one press giving each group's

- **Status:** Accepted
- **Date:** 2026-10-14
- **Related tickets:** 0729, 0217, 0816, 1107
- **Amends:** ADR-0031 and ADR-0053 (a key or button is in at most one slot
  *of each group*, not of the whole table; their saved format is unchanged)

## Context

Until now a chord (or a controller button) could be on only one action:
`keymap.ron` refused it on two, `Keymap` mapped a chord to one `Action`,
and binding a key on the Key bindings screen moved it off whichever action
had it (ADR-0031, ADR-0053).

Ticket 0729 adds the dialogue log's key. Nick's answer
(`docs/design/controls.md`, *Dialogue log*): the log's action lives in "a
different context", so its key may be one another action has, and "we
should have systems to support this behavior across other keys as well".
The default pad button for it, the left face button, is already Danger
zone's: the map and a scene never want both at once.

## Decision

- **`KeyGroup`** (`trpg-content`'s keymap): every `Action` has a group,
  `Action::group()`. Today `General` (every action before) and
  `DialogueLog` (`Action::DialogueLog`). A new group is a variant and a
  line in `Action::group`; nothing else changes.
- **One action per group.** `keymap.ron`'s validation, `LayoutBindings` /
  `PadBindings` (`find`, `bind`, the load-time repair) and the property
  tests all key uniqueness on `(group, chord)` / `(group, button)`. Binding
  a key moves it only from an action of the same group.
- **One press, every group's action.** `Keymap::actions(chord)` and
  `pad_actions(button)` give the chord's actions in group order (`action`
  / `pad_action` stay as the first of them). `InputState` emits each of
  them for the press and keeps each held, so `FrameInput::actions` can
  hold, say, `[DangerZone, DialogueLog]` for one button.
- **The screen decides.** Screens already ignore actions they don't use,
  so the battle ignores `DialogueLog` and a scene ignores `DangerZone`.
  Where one screen uses two actions that share a key (the dialogue screen:
  the log's key bound to Up, which moves the replies), the screen settles
  it by looking at the chords and buttons pressed this frame
  (`FrameInput::pressed_chords` / `pressed_buttons`) against the keymap:
  `DialogueScreen::log_key_free`. Its rule is Nick's ("replies win").
- **The Key bindings screen** shows each non-`General` group as its own
  heading after *Optional*, so the player can see which keys may overlap.

## Consequences

- Other actions can share keys later by getting a group; each needs a
  design answer for which actions share and who wins on a screen that
  uses both.
- A key test of "this key does X" must say which group it looks in
  (`KeymapDef::bindings(layout, group)`, `LayoutBindings::find(group, …)`).
- The saved config needs no new version: slots are saved per action, as
  before. A config from before 0729 loads with the log's default key even
  if the player had put that key on a `General` action.
- Moving input to the engine (ticket 1107) moves `KeyGroup` with it: the
  engine knows that actions come in groups, the game names the groups.

## Alternatives considered

- **Contexts on the keymap** (a keymap per screen kind, the active one
  chosen by the screen on top) — the cleaner model for many contexts, but
  it changes how every screen gets its input and how `Game` builds frames,
  for one shared key today. Groups give the same freedom with the lookup
  unchanged for every existing action.
- **Letting the log's key reuse a cursor action** (Up opens the log while a
  line shows) — Nick's first answer, but Up is also how the replies move;
  he asked for a deliberate entry of its own.
- **Allowing any duplicate chord** and letting screens sort it out — loses
  the "binding a key moves it" rule that keeps the map's keys unambiguous.
