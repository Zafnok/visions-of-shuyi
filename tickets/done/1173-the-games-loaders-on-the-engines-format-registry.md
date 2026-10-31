---
id: "1173"
title: "The game's loaders on the engine's format registry: its kinds and formats registered at start-up, the pin moved"
type: feature
milestone: "Engine: carve"
model: implementation
effort: high
status: done
blocked_by: ["engine#2201"]
nick_input: none
completed: 2026-10-20
---

# 1173 — The game's loaders on the engine's format registry

## Context

Engine ticket 2201 made formats plugins (engine ADR-0007, engine
ADR-0019, the engine's `docs/formats.md`):

- an asset kind is a plain type (`AssetKind`);
- a file format reads one kind from one file's bytes (`Format`);
- the registry `shuyi_assets::Formats` holds them;
- code asks it for an asset by id: `formats.load::<PaletteDef>(bundle,
  "data/palette")`.

Each of the engine's parsers is now a format of its kind, and keeps its
old function. `Formats::standard()` holds those that need nothing of a
game:

- PNG, a one-bit picture in a PNG's place;
- WAV and OGG Vorbis;
- the audio manifest, credits, font atlas, palette, lights and names
  files;
- `.dlg` without directives.

The formats that read a game's own types are the game's to register:

- `Dlg<D>`, `KeymapRon<A>`, `MapText<T, F>`, `CinematicRon<S, P>`;
- `LangPackRon`, with its limits and tokens;
- `Ron<T>`, for a data table.

So are the standard ones configured with the game's rules, put in place
of the standard ones with `Formats::replace`:

- `PaletteRon::requiring`, `FontAtlasRon::requiring`;
- `LightingRon::new`, `CreditsRon::new`.

Engine ADR-0007 asks that "a game registers its own kinds and formats at
start-up", and that its loaders change only their `load()` calls. Checks
across assets stay where they are and run after loading:

- a scene against the characters (`dialogue::scripts_from_parsed` with
  the game's `SceneRules`);
- a cinematic against the music and shots (`cinematic::check`, with the
  file's `AssetPath::stem`);
- voice, tips, the one-bit palette and the whole languages directory.

Then a new format (a Yarn script, a JPEG, the shipping pack of engine
ticket 2204) reaches the game by one `register` call.

## Nick input

None. Nothing a player sees changes.

## Scope

**In:**
- Move the pin past engine 2201 (`engine-change` skill, step 6).
- **A process-wide registry in `trpg-content`**, beside
  `bundle::install()`: `formats::install() -> &'static Formats`. It is
  built once, as the bundle is, from `Formats::standard()`, with:
  - `replace(PaletteRon::requiring(<the game's colours>))`;
  - `replace(FontAtlasRon::requiring(font::REQUIRED_GLYPHS))`;
  - `replace(LightingRon::new(<the game's noon light>))`;
  - `replace(CreditsRon::new(<the manifest as errors name it>,
    <the private folder>))`;
  - `register` for `Dlg::<the game's Directive>`,
    `KeymapRon::<the game's Action>`, `MapText::<TerrainId, Feature>`
    (with `map::FORMAT` and a terrain resolver that owns its table) and
    `CinematicRon::<the game's shot, its actions>`.
- **The game's loaders read through it.** Each `load()` that reads one
  engine kind from the bundle calls `formats::install().load::<Kind>(
  bundle::install(), PATH)` instead of `bundle::file` and the parser:
  - `font::load`;
  - `palette::load` and its themes (`load_theme`);
  - `lighting::load`;
  - `names::load`;
  - `credits::load_file`;
  - `audio::load`;
  - `keymap::load_keymap`;
  - `image::load` (`ImageTable::load_with`, still skipping the atlas);
  - `dialogue::load_scripts` (`Formats::load_under` on the dialogue
    folder, then `scripts_from_parsed` with the game's rules);
  - `map::load_all` and `cinematic::load_all`, keeping their checks
    after the decode.
- **The game's own kinds.** Each data table whose parser reads its file
  alone (`ai`, `art`, `item`, `skill`) gets an `AssetKind` impl and a
  format: `Ron<T>`, or a small format that calls its `from_source`.
  These are registered in the same place, and their `load()` reads
  through the registry.

**Out (do not do):**
- Changing what any loader accepts or reports. The content checks'
  output stays the same, error for error.
- Tables whose parsers check against other tables
  (`character::from_source` takes the classes and items, `battle`,
  `chapter`, `support`, `spell`, `lesson`, `title`, `tip`, `voice`).
  They keep their loaders. A format for them is a later ticket, if
  wanted.
- `lang`: the languages directory stays on `lang::from_files`, which
  checks every pack against the game's data text at once. `LangPackRon`
  can replace it later, once the data text is built before the registry.
- New formats; the shipping pack (engine 2204).
- The engine-friction folder loader (game#1169, engine 2208): if 1169
  lands first, the loaders it moved to `Bundle::load_each` that read an
  engine kind move again here, to `Formats::load_under`. If this one
  lands first, 1169 leaves them on the registry.

## Implementation steps

1. Write the estimate: about 150 lines added (the registry, the kinds),
   about 120 removed (the read-and-parse lines of each loader).
2. `cargo xtask engine`, then move the pin to the engine's `main` past
   2201. Build: nothing should break, since 2201 is additive.
3. Write `crates/content/src/formats.rs` with `install()`. A test checks
   it with the engine's `Formats::ids`, kind by kind: which format reads
   each kind.
4. Move the loaders one at a time. Each keeps its tests: they test the
   same files and must report the same.
5. The game's single-file tables as kinds, with a test per table that
   `install().load::<Table>` equals its old loader on the real file.

## Acceptance criteria

- [x] `shuyi-engine.rev` is at or past the merge of engine 2201.
- [x] `grep -rn "from_source(&display" crates/content/src` finds only
      the tables listed under **Out**.
- [x] A test lists the registry's kinds and their formats.
- [x] The content checks (`cargo xtask` content checks, and `cargo test
      -p trpg-content`) report the same on the real and test files.
- [x] `cargo xtask ticket-lint --check-engine` passes (after engine PR #32, the ticket of engine 2219, merged).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the registry's kinds and formats; each moved loader's existing
  tests, unchanged in what they check.
- Integration: the game loads (`trpg-content`'s embedded-bundle tests)
  through the registry.

## Completion notes

**Done.** `crates/content/src/formats.rs`: `install()` builds the
game's registry once (a `OnceBox`, as the bundle), from
`Formats::standard()` with `PaletteRon::requiring`,
`FontAtlasRon::requiring`, `LightingRon::new(NOON)` and `CreditsRon::new`
put in place of the standard ones, then `Dlg::<Stage>`,
`KeymapRon::<Action>`, the map format (`map::register`, a `MapText` whose
terrain resolver owns the terrain file's ids), the cinematic format
(`cinematic::register`, its raw shots and action lists) and the four data
tables. Every loader the ticket lists reads through it: font, palette and
its themes, lighting, names (its test part still merged after), credits,
audio, keymap, the image table (`ImageTable::load_with`), dialogue, maps,
cinematics, `ai`, `art`, `item` and `skill`. The checks across assets
stay where they were and run after the decode.

**Deviations.**
- **The pin did not move.** It was already past engine 2201 (moved by an
  earlier ticket), so the first criterion was met on arrival.
- **The data tables are `formats::Table<T>`**, not their own types:
  `AiWeights`, `ArtTable`, `ItemTable` and `SkillTable` are `trpg_core`'s
  and `AssetKind` is the engine's, so Rust's orphan rule forbids the impl.
  Each table implements the game's `DataTable` trait (its name and its
  `from_source`), read by one format, `TableRon<T>`.
- **Folders are read with `formats::load_in`, not `Formats::load_under`**:
  `load_under` returns nothing when one file fails, so the checks after
  (cinematics, dialogue) would stop reporting on the files that read, and
  it reads subfolders where the loaders read one folder. `load_in` is
  each file of the kind directly in the folder, read or with its errors.
  It is a workaround marked `// engine-friction: engine#2219`; engine
  ticket 2219 (`Formats::load_each`, its PR
  [Zafnok/shuyi-engine#32](https://github.com/Zafnok/shuyi-engine/pull/32))
  and game ticket 1176 replace it.
- **`map::load_all()` takes no terrain table**: the map format resolves
  terrain ids itself, from the terrain file read without its colour check
  when the registry is built. `load_embedded` still loads maps only when
  the terrain loaded, so the ids are the same.
- **`class` and `terrain` keep their loaders** (the criterion's grep finds
  them beside the listed tables): their parsers check against other
  tables too (movement types, the palette), which puts them under **Out**.
- **What the loaders report is the same on every real and test file.**
  Two small differences on broken files only: a missing file is still
  "file not found in asset bundle" (the registry would add "nor
  ….ron.ron"; `formats::load` keeps the old message), but a file that
  isn't UTF-8 now says so instead of "not found"; and a `.dlg` file with a
  syntax error is no longer rule-checked itself (the `.dlg` format gives
  its errors, not its half-read scenes), though every other file still
  is (a test checks it).

**Tests.** `formats::tests`: the registry's kinds and their formats, the
game's formats registered on the standard ones without a conflict, each
table equal to its parser on the real file, a table's errors, the old
missing-file message, and `load_in` on a fixture bundle.
`dialogue::tests::load_scripts_reports_a_broken_file_and_the_checks_of_the_others`.
Every loader's existing tests pass unchanged.

**Engine friction:** estimate 150, measured 208 (1.4×), under 2×; one
workaround signal, outcome 2 (engine ergonomics), shipped with the
marker. Row in `docs/engine/friction.md`.

**Follow-ups:** engine ticket 2219; game ticket 1176 (`blocked_by`
`engine#2219`).

**Nick:** nothing changes in the game; no rules decided.
