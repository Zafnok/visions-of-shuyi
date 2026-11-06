---
id: "1246"
title: Pause or mute while the game's window has no focus
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 1246 — Pause or mute while the game's window has no focus

## Context

The pin move of 2026-11-05 (issue #548) brought two engine PRs:

- #268 (3042): `shuyi-app` sends `FocusLost` and `FocusGained` on the
  desktop and the web, lets go of held keys and pad buttons when focus
  is lost, and ignores pads without focus. The game gets this through
  `shuyi_app::run` with nothing to change.
- #269 (3093): a background policy the game sets,
  `GameSettings::BACKGROUND` (`Background::KEEP_RUNNING`, `paused()`,
  `muted()`, or both). Paused: no update, no input, the last frame shown
  again, and the clock restarts on return. Muted: the three volumes at 0
  until focus comes back. The default keeps running with its sound, so
  nothing changed at the pin. Engine ADR-0077.

The game's `ShuyiSettings` (`crates/ui/src/settings.rs`) implements
`GameSettings` and keeps the default. Whether the game should pause, go
quiet, both or neither when the player tabs away is a look-and-feel
call, so it is Nick's.

## Nick input

**Decision**, asked before building: what happens when the player
switches to another window. Options from real games: keeps running with
sound (the default today); keeps running, muted (many PC games); pauses
and mutes (many console ports); a player setting in Options choosing
among them, with one of these as its default. A turn game has little
running while the player waits, so the main difference is the music
and any animation playing.

## Scope

**In:**
- `ShuyiSettings`'s `BACKGROUND` as Nick chose; if he chose a setting,
  its Options row (text by key, ADR-0045) and its saved field.

**Out (do not do):**
- The Playdate (its own suspend, 1170).
- Pausing the battle's rules: the engine's pause holds the whole frame.

## Implementation steps

1. Read engine ADR-0076 and ADR-0077 at the pin.
2. Set `BACKGROUND` on `ShuyiSettings`, or add the setting and read it.
3. If a setting: the Options row, the settings file's next version with
   its default for old files.

## Acceptance criteria

- [ ] With Nick's choice, a Harness test sends `FocusLost`: the frame
      holds and/or the volumes read 0; `FocusGained` brings them back.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: Harness tests of focus lost and gained.
- Unit: an old settings file loads with the default, if a setting is
  added.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
