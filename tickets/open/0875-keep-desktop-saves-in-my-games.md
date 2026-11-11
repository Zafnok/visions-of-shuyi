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
- If the folder choice is the engine's (`AppConfig`, `data_dir`), the
  change goes there first (`engine-change`).

**Out (do not do):**
- The web's storage (stays `localStorage`), the Playdate's.
- Steam Cloud (0903): it reads whatever folder this picks.

## Implementation steps

1. Read the engine's storage folder code; decide where the choice lives
   (engine `AppConfig` or the game), with an ADR or a section in
   ADR-0039.
2. The folder, the move, their tests (a temp folder for both places).

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
