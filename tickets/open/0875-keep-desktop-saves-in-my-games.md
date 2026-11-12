---
id: "0875"
title: "Keep the desktop's saves in Documents/My Games, and move the old ones there"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0875 — Keep the desktop's saves in Documents/My Games

## Context

Nick, 2026-11-11, answering 0872 (`docs/design/options.md`, *Saves
outside the game*): "the save should never default to AppData probably
should be in Documents or My Games folder which is more accessible".

Today the desktop keeps every key in `%APPDATA%/tactical-rpg/`: the
engine's `shuyi_app` storage (`crates/app/src/storage/native.rs`,
`data_dir(name)`), named by `STORAGE_NAME` in `crates/app/src/main.rs`
(ADR-0039, ADR-0069). 0872 shows that folder's path on the Profile and
Load Game screens (`PlatformFacts::storage_place`), so they follow the
move with nothing more to do.

## Nick input

None: the folder is decided. `Documents\My Games\<game>` is the agent's
pick between the two Nick named (the common place for PC games' saves);
the folder's own name (the game's name, not `tactical-rpg`) is the
agent's too.

## Scope

**In:**
- The desktop's storage folder: `Documents\My Games\Visions of Shuyi`
  on Windows (the known-folder Documents, which may be redirected, e.g.
  to OneDrive); the other desktops keep their own conventions, or follow
  the same idea where they have one (the agent decides, `write-adr`).
- **Moving the old saves:** on the first launch after the change, every
  file of `%APPDATA%/tactical-rpg/` (and its `exports/`) moves to the new
  folder when the new one has none of the game's keys yet; a failure
  leaves the old folder in use for that launch and logs why. Nothing is
  ever lost or written over.
- The engine does the folder and the move: engine PR #376 (engine
  ticket 3310, engine ADR-0108), in the pin since the pin move of
  2026-11-11, adds `AppConfig::desktop_folder`, whose
  `DesktopFolder::MyGames(name)` is `<Documents>\My Games\<name>` on
  Windows (the known folder, OneDrive followed) and the data folder
  elsewhere, and moves the old folder's files there once, as above
  (exports too; a failed copy keeps the old folder and logs why). The
  game sets it in `crates/app/src/main.rs`, and
  `storage::desktop_dir(name, desktop)` names the folder without moving
  anything wherever the game shows or checks the path.

**Out (do not do):**
- The web's storage (stays `localStorage`), the Playdate's.
- Steam Cloud (0903): it reads whatever folder this picks.

## Implementation steps

1. ~~Decide where the choice lives~~: the engine's `AppConfig`
   (done by the pin move of 2026-11-11). Note the folder in ADR-0039.
2. Set `desktop_folder: DesktopFolder::MyGames(..)` with the game's
   name; the game's tests that the move reaches the old saves (the
   engine's own suite tests the move with temp folders).

## Acceptance criteria

- [ ] A test: a first launch with saves in the old folder moves them all
      to the new one; a second launch finds them there.
- [ ] A test: a new folder that already holds saves is used as it is;
      the old folder is left alone.
- [ ] The Profile and Load Game place line shows the new path.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit / integration: the move, with temporary folders.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
