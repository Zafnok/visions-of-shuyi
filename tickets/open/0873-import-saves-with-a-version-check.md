---
id: "0873"
title: "Import saves from a file, with a check of which saves this version can read"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["0872"]
nick_input: none
completed:
---

# 0873 — Import saves from a file, with a version check

## Context

Nick, 2026-11-10: "import would read it and determine if a version
compatibility issue exists or it can import cross-version." 0872 exports
a bundle file (engine#2138's format) and records where Import lives and
what it does over existing saves (`docs/design/options.md`, *Saves
outside the game*).

What "cross-version" means here (ADR-0039): every save names its
format's version (`trpg_core::save::SAVE_VERSION`, now 2; settings
`SETTINGS_VERSION`; the engine's key bindings and profile list their
own). A bundle made by **another build** imports fine when its saves'
versions are this build's: most patches don't change a saved type. A
save of **another version** is refused, as loading one is today
(`SaveError::Incompatible`), because nothing migrates saves yet. When
engine 2110 (migrations) lands and the game adopts it (no game ticket
yet: ADR-0039 says a migration ticket decides), older saves become
`Migratable` and import through it; nothing here changes for that
beyond the verdict's text.

The engine pieces:

- **engine#3309:** `Ctx::ask_for_file` (a picker on the web, answered by
  `SystemEvent::FileChosen`), and `SystemEvent::FilesDropped` (a file
  dropped on the window or the page, already there since engine 3092),
  which is the desktop's way in.
- **engine#2138:** `plan_import` gives a verdict per entry (`Ready`,
  `Incompatible { found, wants }`, `Unreadable`, `Unknown`), which keys
  it would replace, and the build stamp's `Mismatch`; `apply` writes the
  chosen entries all or nothing.

## Nick input

None: 0872 asks where Import lives and what it does over existing saves.

## Scope

**In:**
- **The checker** (`crates/ui/src/save_files.rs`, from 0872): an
  `EntryCheck` list for the game's keys: `slot_NN` and `suspend` as
  `SaveFile` (`FormatCheck::<SaveFile>`, plus the content check
  `save::decode` already does: a suspend save whose chapter the content
  lacks is `Unreadable`), `settings` with `SETTINGS_VERSION`,
  `keybindings` and `layout` with the engine's own readers, `tips_seen`
  and `last_save` (`last_save` is `Ready` only if the slot it names is
  imported or present).
- **Getting the file:** Import asks for a file (`ask_for_file([".ron"])`)
  where `PlatformFacts::file_picker`, and otherwise says "Drop the file
  on the window" and takes the next `FilesDropped` with one file. A
  dropped file anywhere else in the game is ignored, as today.
- **The report screen**, before anything is written: one line per save
  slot in the file (chapter, playtime, as the Load Game list shows them,
  `save::slots`) with its verdict in words: "Ready", "Ready — replaces
  slot 3", "From an incompatible version (save version 1, this game
  reads 2)", "Can't be read". A line when the file was made by another
  build ("Made by version 0.4.0; this is 0.5.0"), from `Mismatch`.
  Then Import / Cancel (a yes/no question, focus on the
  non-destructive choice).
- **Applying** with `apply`, taking every `Ready` entry, into the
  profile Nick's answer to 0872 says (a new profile through
  `Ctx::add_profile`, or the one chosen after its own yes/no). A
  notice with the count imported and refused.
- Text keys in the `en` pack; the Playdate leaves Import out.

**Out (do not do):**
- Export and the place line: 0872.
- Migrating saves of an older version: engine 2110 and a game ticket
  that ADR-0039's *Consequences* calls for (not written yet; it needs
  Nick's answer on what players keep after a patch, per engine 2110's
  Nick input).
- Merging two saves, or choosing entries one by one (all `Ready`
  entries are taken).
- Importing a single `.ron` save that isn't a bundle (a raw slot file
  from `%APPDATA%`): refused as "not an exported save file".

## Implementation steps

1. ~~Pin: move it past engine#2138~~: done by the pin move of
   2026-11-11 to `85e8d5c5` (past engine#3309: the pin move before it).
2. The checker list in `save_files.rs`, with a unit test per key kind.
3. `crates/ui/src/screens/save_import.rs` (new): the screen's logic
   (waiting for the file, the plan, the question), its view (plain data,
   ADR-0054) and a glyph skin in `save_import/glyph.rs`; declared
   actions only (`keyboard-input`). Opened from the row 0872 added.
4. Wire `SystemEvent::FileChosen` / `FilesDropped` to it while it waits.
5. Apply, then refresh what reads the profile: if the profile in use
   was replaced, reload its settings and keys as `switch_profile` does
   (ADR-0069).

Expected size: about 600–800 production lines (a screen with three
states, its skin, the checker list), plus tests. Judge it by the slot
picker (`screens/save.rs`).

## Acceptance criteria

- [ ] A Harness test: a bundle with a current slot, a slot of save
      version 1, a damaged slot and an unknown key shows "Ready",
      "incompatible version (1, reads 2)", "Can't be read" and leaves the
      unknown key out; Import writes only the current slot.
- [ ] A Harness test: a bundle made by another game version stamp but
      the same save versions imports every slot and shows the "made by"
      line.
- [ ] A Harness test: Cancel writes nothing; a storage that fails mid-
      import leaves the profile as it was (engine#2138's `apply`).
- [ ] A Harness test: on facts without a picker, Import says to drop the
      file, and a `FilesDropped` with the bundle opens the report.
- [ ] A round trip: 0872's export of a profile, imported into a fresh
      profile, gives the same `save::slots` list.
- [ ] Snapshots of the report screen; `cargo shuyi frame-png` pictures
      in the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each key kind's verdict; `last_save`'s rule.
- Integration: the Harness tests above, with `MemoryStorage`.
- Snapshot: the report screen's view in its three states.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
