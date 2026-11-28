---
id: "1415"
title: "The Profile screen's export, import and save folder line from the engine's profiles screen"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3254"]
nick_input: none
completed:
---

# 1415 — Profile export and import from the engine

## Context

The third engine-friction sweep (1397, `docs/engine/friction.md`)
found that the game wraps the engine's `ProfilesScreen` to add export,
import, an open-folder key, the "where the saves are" line and the
"saved to" notice. It also keeps generic export plumbing in
`save_files.rs` and `save_files/import.rs`. About 330 production lines
do a job the game should cover in about 60, and none of them names a
game concept. Engine ticket 3254 (`Zafnok/shuyi-engine`) puts these
pieces on the engine's profiles screen and storage.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine 3254 (`engine-change` skill, §7) if it isn't
  there yet.
- `screens/profile.rs` becomes a short opener: the engine's screen with
  `.exporting(save_files::NAME_START)`, `.importing(...)` that opens the
  game's `ImportScreen` for a profile, and `.with_place()`. The tip on
  the first visit (`TipTrigger::FirstProfileScreen`) stays the game's.
- `screens/files.rs` and `files/glyph.rs`: the place line goes. Whatever
  the Load Game list still uses stays, on the engine's place line if it
  fits.
- `save_files.rs`: `export_name`, `safe_name`, `file_name`,
  `exported_of`, `ExportNotice` and `ProfileKeys` go. `export` keeps the
  game's `Export` (profile or slot) and passes the slot suffix.
- `save_files/import.rs`: `name_in`, `with_number`, `capitalized`,
  `new_name` and `OtherProfile` go. `GameCheck` keeps only the game's
  keys (slots, suspend, `SET_UP_KEY`, `LAST_SAVE_KEY`), chained with the
  engine's `engine_checks`.

**Out (do not do):**
- Any change the player sees: the keys, the help line, the notice text
  and the file names stay as they are.
- The Load Game list's export of one slot (`save.rs`). If the engine's
  pieces fit it, it may use them; otherwise leave it.

## Implementation steps

1. Write the engine-friction estimate first: about 270 lines removed and
   40 added.
2. Move the pin. Rebuild `ProfileScreen` on the engine's builders, and
   keep `save_files::hear` handing the platform's answer to the
   screen, unless the engine screen hears it itself.
3. Delete the copies listed in Scope (each is marked `// engine-friction:
   engine#3254`).
4. Run the existing tests: `save_files/tests.rs`, `save_files/import`
   tests and the profile screen's snapshots.

## Acceptance criteria

- [ ] `grep -rn "engine#3254" crates` finds nothing.
- [ ] The Profile screen's snapshots, PC and Playdate, are unchanged.
- [ ] An exported profile's file name and contents are byte for byte
      the same as before (`save_files` tests).
- [ ] `cargo xtask code-lines` shows `trpg-ui` down by at least 200.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit/snapshot: the existing ones. Move a test to the engine only where
  it now tests the engine's code alone.

## Completion notes

*(Filled in by the session that completes the ticket.)*
