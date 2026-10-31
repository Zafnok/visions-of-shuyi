---
id: "1104"
title: "shuyi-assets: images, fonts, named colors, lights, audio cues and credits"
type: infra
milestone: "Engine: carve"
model: implementation
effort: medium
status: done
blocked_by: ["1103"]
nick_input: none
completed: 2026-10-16
---

# 1104 — shuyi-assets: images, fonts, colors, lights, audio, credits

## Context

ADR-0063 §6. The frame, input and audio need these types, and none of
these modules imports `trpg_core` (`docs/engine/boundary.md`,
*content*): `image.rs`, `font.rs`, `lighting.rs`, `audio.rs`,
`credits.rs`, and the generic part of `palette.rs`. Each loader is
already `load()` (reads the bundle) plus a pure parser, so the parser
moves and `load()` stays with the game where it names the game's paths.

## Nick input

None.

## Scope

**In:**
- Move to `shuyi-assets`: `ImageId`, `ImageTable` and the PNG header
  reader; `FontAtlasDef`; `Light` and light recipes; `AudioManifest`
  (cues, pools, credits, licenses, WAV/OGG header checks); credits and
  ADR-0051's check that bought files are credited (the private folder
  names become a parameter).
- `palette.rs` is cut: named colors, hex parsing and theme *files* move;
  `Theme` (this game's four themes) and `REQUIRED_COLORS` (move range,
  attack range, danger zone, EXP bar…) stay in `trpg-content`, and the
  engine's palette loader takes the required names as an argument.
- `font::REQUIRED_GLYPHS` stays the game's and is passed in.
- Re-export everything under the old paths in `trpg-content`.

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6). 1032's one-bit font and Playdate palette, 1040's one-bit image format and 1047's audio conversion move with the assets they belong to.

**Out (do not do):** `UiColor` in `ui` (1108); languages and dialogue
(1105); keymaps (1106/1107).

## Implementation steps

1. Move one module at a time with its inline tests; tests that read the
   real `assets/` files (`embedded_*`, `design_cues`, the theme tests)
   stay in `trpg-content`.
2. Cut `palette.rs` as above.
3. Run `check-engine-boundary`, all tests, the WASM build.

## Acceptance criteria

- [x] The listed types live in `shuyi-assets` and are tested there with
      fixtures only.
- [x] No snapshot changes; every existing test passes.
- [x] `docs/engine/boundary.md` rows updated.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the moved inline tests.

## Completion notes

Done on Opus 5.5 (the ticket names Sonnet 5).

- `shuyi-assets` gained `image`, `font`, `lighting`, `audio`, `credits`,
  `palette` and `ink`, each with its fixture-only inline tests. They
  take what names the game's files as arguments: `ImageTable::load(bundle,
  skip)` and `from_files(bundle, files)`; `FontAtlasDef::from_source(...,
  png_file, png, required)` and `problems(required)`;
  `lighting::from_source(file, source, default)`; the audio `Files` now
  carries the sound files' shown names and the music folder's name; the
  credits' `Places` names the audio manifest and the private folder;
  `PaletteDef::from_source(file, source, required)`.
- Content keeps the game's paths, `REQUIRED_GLYPHS`, `NOON`, `Theme`,
  `REQUIRED_COLORS`, `load_themes` and every `load()`, plus wrappers with
  the old arguments (`font::from_source`, `lighting::from_source`,
  `credits::from_source`, `palette::from_source`), and re-exports the
  rest under the old paths. The tests that read the real files stayed.
- **Deviation:** a type's inherent `load()` can't stay on a type that now
  lives in another crate, so `ImageTable::load()`, `ImageTable::from_files`,
  `FontAtlasDef::load()`, `PaletteDef::load()`/`load_theme` and
  `PaletteDef::load_playdate` became free functions (`image::load()`,
  `image::from_files`, `font::load()`, `palette::load()`/`load_theme`,
  `ink::load_playdate`), like `audio::load()` already was. About 30 call
  sites changed with them.
- Playdate: 1032's ink palette moved (`Ink`, `PaletteDef::from_ink_source`);
  the one-bit atlas is only a path in content and stays. 1040 and 1047
  have no content code yet. `shuyi-assets` keeps building without `std`
  (it now needs `libm` for the lights).
- No gameplay rules decided.

