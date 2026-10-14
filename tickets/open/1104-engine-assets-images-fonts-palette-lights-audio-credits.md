---
id: "1104"
title: "engine-assets: images, fonts, named colors, lights, audio cues and credits"
type: infra
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: todo
blocked_by: ["1103"]
nick_input: none
completed:
---

# 1104 — engine-assets: images, fonts, colors, lights, audio, credits

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
- Move to `engine-assets`: `ImageId`, `ImageTable` and the PNG header
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

**Out (do not do):** `UiColor` in `ui` (1108); languages and dialogue
(1105); keymaps (1106/1107).

## Implementation steps

1. Move one module at a time with its inline tests; tests that read the
   real `assets/` files (`embedded_*`, `design_cues`, the theme tests)
   stay in `trpg-content`.
2. Cut `palette.rs` as above.
3. Run `check-engine-boundary`, all tests, the WASM build.

## Acceptance criteria

- [ ] The listed types live in `engine-assets` and are tested there with
      fixtures only.
- [ ] No snapshot changes; every existing test passes.
- [ ] `docs/engine/boundary.md` rows updated.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the moved inline tests.

## Completion notes
