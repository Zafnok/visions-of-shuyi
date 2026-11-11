---
id: "0872"
title: "Export saves to a file, and show the player where saves are kept"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: blocked
blocked_by: ["engine#3309", "engine#2138"]
nick_input: decision
completed:
---

# 0872 — Export saves to a file, and show the player where saves are kept

## Context

Nick, 2026-11-10:

> "I want the game/engine to allow exportable save files. I would presume
> if I played on native build, the saves are already stashed somewhere.
> But, most of my testing is in web. We should make it very easy to see
> where the saves are, and allow export/import, even in web. Export I
> would think would become a download of i.e. a .ron file."

Where saves are today (ADR-0039, ADR-0069): each profile's keys
(`p<n>-settings`, `p<n>-keybindings`, `p<n>-layout`, `p<n>-tips_seen`,
`p<n>-slot_01` … `p<n>-slot_30`, `p<n>-suspend`, `p<n>-last_save`) go
through the engine's `Storage`: files in `%APPDATA%/tactical-rpg/` on the
desktop, `localStorage` keys under `tactical-rpg/` on the web
(`crates/app/src/main.rs`, `STORAGE_NAME`). The game shows neither, and
on the web a player can't reach them without the browser's dev tools.

The engine pieces:

- **engine#3309** (files the player keeps): `Ctx::offer_file` (a
  download on the web, `<data dir>/exports/<name>` on the desktop),
  `Ctx::open_storage_folder`, and `PlatformFacts::storage_place`.
- **engine#2138** (save bundles): `write_bundle` packs a storage's keys
  into one RON file stamped with the build that made it (`Made`, engine
  2111).

This ticket is the export half; 0873 is the import half and its version
check.

**0914 waits for this ticket** (Nick, 2026-11-11, `docs/design/demo.md`):
the Chapter 1 web demo has no "Save your progress?" and ends on "Export
save?" Yes / No, which offers a file of the game after Chapter 1 that
the full game imports (0873) and continues into Chapter 2 from. The demo
has no save in storage to export there: it bundles the campaign as the
chapter save would hold it. So build the export so that 0914 can hand it
entries that aren't in the profile's storage (a slot's key and the
encoded save), not only a filter over the stored keys, and keep the file
importable by 0873 whatever Nick answers here about what a file holds.

## Nick input

**Decision**, asked with `ask-nick` at the start of the ticket, before
building (one round, both looks, as `docs/design/options.md` records):

1. **Where do "where saves are", Export and Import live?**
   - *A — on the Profile screen*, for the profile under the cursor:
     "Export profile" and "Import profile" (Cookie Clicker and Kittens
     Game keep their export and import with the game's options; a profile
     is the unit here, as Steam's per-user saves are).
   - *B — a "Saves" page in Options*: the place line, Export, Import
     (Cookie Clicker's Options: "Export save", "Import save", "Save to
     file", "Load from file"; A Dark Room's "save" menu).
   - *C — on the Load Game slot list*: export the slot under the cursor
     as its own file (RetroArch's save-state files, one per slot).
   - Describe your own.
2. **What one exported file holds:** the whole profile (30 slots, the
   suspend save, settings, keys, tips seen), or the save slots alone, or
   one slot.
3. **Import over a profile that already has saves:** import as a new
   profile, never touching the one in use (the agent's starting rule:
   confirm before overwriting), or replace after a yes/no
   question.

Record the answers in `docs/design/options.md` (*Saves outside the
game*). 0873 builds the import from the same answers.

Verified by tests and `cargo shuyi frame-png` pictures in the PR; what
is left to judge by playing goes on the Act 1 playtest's checklist
(1064), never a check after merge.

## Scope

**In:**
- **Where saves are:** a line the player can read in the place Nick
  picks, from `PlatformFacts::storage_place`: on the desktop the folder's
  path (shortened from the left to fit, with an action to open it:
  `Ctx::open_storage_folder`); on the web "Saved in this browser's
  storage for this site; clearing site data deletes them" and the key
  prefix. New text keys in the `en` pack.
- **Export:** builds a bundle with `write_bundle` over the keys Nick's
  answer covers (through the profile's store, so the keys are without
  the `p<n>-` prefix and import into any profile), and offers it with
  `offer_file`. File name: `visions-of-shuyi-<profile name made safe>-<n>.ron`
  (no date: `ui` has no clock; the desktop's `(2)` suffix keeps names
  apart). A notice after it says where it went (`FileOffered`):
  "Downloaded", or the exports folder's path with the open-folder action.
- The Playdate: no export (no files a player reaches); its skin leaves
  the rows out (`PlatformFacts`).
- An ADR only if the bundle's key list or the file name becomes a
  convention other screens follow (`write-adr`); otherwise a section in
  ADR-0039 *Consequences* naming the bundle.

**Out (do not do):**
- Import and its version check: 0873.
- Migrating old saves (engine 2110, After MVP): not this ticket.
- Steam Cloud (0903).
- A native "save as" dialog: engine#3309 has none.

## Implementation steps

1. **First step: move the pin** past the merge of engine#3309 and
   engine#2138 (`engine-change` §7 and the `engine-pin` skill's review),
   unless an open `engine-pin` PR already does.
2. Ask Nick (above); write `docs/design/options.md`'s section.
3. `crates/ui/src/save_files.rs` (new): `export_keys(answer) -> impl Fn(&str) -> bool`,
   `export_name(profile) -> String`, `place_text(ctx) -> String`.
4. The screen Nick picked: rows built in its view (plain data, ADR-0054)
   and painted by its glyph skin; actions declared, no key named in code
   (`keyboard-input`). If it is the engine's Profile screen
   (`shuyi_ui::screens::profiles`), the rows are a game-side extension
   through its builder (`opening_after_add` is the pattern); if that
   needs an engine hook, follow `engine-change`.
5. Handle `SystemEvent::FileOffered` in the screen: a notice line.

Expected size: about 350–500 production lines (a view of three rows,
its skin, the export function), plus tests.

## Acceptance criteria

- [ ] `docs/design/options.md` records Nick's three answers.
- [ ] A Harness test: Export on a profile with two slots and settings
      offers one file named `visions-of-shuyi-…-1.ron` that holds exactly
      the keys of Nick's answer, without the `p<n>-` prefix.
- [ ] A Harness test: the place line reads the folder on a desktop
      `storage_place` and the browser text on a web one.
- [ ] Snapshot of the screen with the new rows (desktop and web facts);
      `cargo shuyi frame-png` pictures of both in the PR.
- [ ] No key named in code or text (`cargo shuyi check-keys`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `export_name` (a profile name with spaces and non-ASCII),
  `export_keys`.
- Integration: the Harness tests above.
- Snapshot: the screen's view.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
