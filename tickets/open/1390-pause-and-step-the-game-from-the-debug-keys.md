---
id: "1390"
title: "Pause the game and step it one frame from two debug keys"
type: infra
milestone: M1 Engine
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 1390 — Pause the game and step it one frame from two debug keys

## Context

Engine PR #605 (engine ticket 2316, engine ADR-0153), in the pin since the
pin move of 2026-11-27 (`docs/engine/pins.md`), lets a debug build stop on
the frame that goes wrong: `UiAction::DebugPause` holds the game (its
clock and modules stand, the debug tools' screens still run) and
`UiAction::DebugStep` plays one frame while it is held. The "Debug view"
screen gains a slow-motion row (1 to 8 times slower), a status line naming
the bound pause and step keys, and a `readout` channel that draws the top
screen's lines in the corner. The two actions never reach a screen, with
debug tools on or off. A stepped run keeps the same state hash as a
straight run, so a record still replays.

The game gets the slow-motion row and the readout at the pin, but no key
pauses or steps: its `Action` (`crates/content/src/keymap.rs`) maps only
`Action::Debug` to a debug `UiAction` (`Self::Debug => Some(UiAction::Debug)`,
`F2`, keyboard only, not rebindable). The engine's sample keymap uses `F3`
and `F4`.

The game's motion that would use it: the walk shown at the player's speed,
the combat scene's hits, the title's cinematic, a dialogue line typing out,
the swaying trees (1356) and the moving tiles (ADR-0131).

## Nick input

**Decision:** default keys are Nick's (the `keyboard-input` skill).
Ask with the `ask-nick` skill, in one question, which two keys pause and
step in a build with debug tools, keyboard only and not rebindable like
the Debug key: offer `F3` and `F4` (the engine's sample), `F5` and `F6`,
and *describe your own*. These keys exist only in builds with debug tools,
never in a release.

## Scope

**In:**
- `Action::DebugPause` and `Action::DebugStep` in `crates/content/src/keymap.rs`,
  mapped to `UiAction::DebugPause` and `UiAction::DebugStep`, not
  rebindable and in the debug key group as `Debug` is, with no button.
- Their keys in `assets/data/keymap.ron` (both layouts) and in
  `docs/design/controls.md`'s tables, as Nick picks them; the tests that
  pin each default.
- The Key bindings screen treats the two keys as it treats the Debug key
  (`That key can't be used` in builds with the debug menu).
- `docs/design/controls.md`: a line for the two keys beside the Debug
  key's, saying what they do.

**Out (do not do):**
- Any change to the engine's pause, step or slow motion.
- The Playdate: it has no keyboard, and no debug key either.
- Release builds: the keys do nothing there, as the Debug key.

## Implementation steps

1. Ask Nick (above). Then follow the `keyboard-input` skill for a new
   Action.
2. Add the two Actions after `Action::Debug`, with `ui()` mapping them,
   `is_rebindable()` false and their `group()` the Debug key's.
3. Add their keys to `assets/data/keymap.ron` and the tables in
   `docs/design/controls.md`.
4. Make the Key bindings screen reserve them as it reserves the Debug key.
5. A Harness test: with debug tools on, the pause key holds the battle's
   clock (a move's walk stands still), the step key moves it one frame,
   and with debug tools off neither key reaches a screen.

Expected size: about 30 production lines (two actions, their mapping and
reservation), plus the keymap file, the design tables and tests.

## Acceptance criteria

- [ ] With debug tools on, the pause key holds the game and the step key
  plays one frame (a Harness test reading the clock or the scene).
- [ ] Neither key reaches a screen, with debug tools on or off.
- [ ] The Key bindings screen refuses both keys in builds with the debug
  menu, as it refuses the Debug key.
- [ ] `keymap.ron` and `docs/design/controls.md` agree, and a test pins
  each default.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the two Actions' `ui()`, `is_rebindable()` and group.
- Integration: the Harness test of step 5; the Key bindings screen's
  refusal.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
