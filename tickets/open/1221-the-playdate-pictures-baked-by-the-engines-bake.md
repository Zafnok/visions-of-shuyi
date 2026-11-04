---
id: "1221"
title: "The Playdate's one-bit pictures made by the engine's convert settings, the same files byte for byte"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#2245"]
nick_input: none
completed:
---

# 1221 — The Playdate's pictures made by the engine's convert settings

## Context

From the game/engine boundary audit of 2026-11-03. ADR-0085 made the
Playdate's one-bit pictures at build time with the engine's
`bitmap::convert`, by a game table: `crates/content/src/playdate_images.rs`
(265 production lines: `KINDS`, the style methods, `Lift`, `with_rim`,
`make`, `jobs`), called by `crates/content/build.rs`
(`convert_pictures`) and by `cargo xtask playdate-images`. The engine's
3007 bake couldn't express it (pin ledger row #190: no scale, styles,
lift or rim; other ditherers), so the machinery stayed here.
[engine#2245](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/2245-the-one-bit-bake-takes-styles-scale-a-lift-and-a-rim.md)
adds a `convert` import-settings section (scale, named styles, lift,
rim, integers), `ConvertSettings::make`, `bitmap::convert::{lift,
with_rim}` and `import_settings::nearest`. Which pictures, at what size
and in which styles stays the game's (ADR-0085 §2–3,
`docs/design/playdate.md`), now as `.meta` data.

**Either order with 1170** (the Playdate front end on the engine's
backend, which wires a one-bit pictures adapter to `shuyi export
playdate`): if 1170 is done, its adapter calls this module; move it onto
`ConvertSettings` too, or drop it if the engine's bake now makes the
same files from the `.meta` data. If not, nothing more to do here:
1170's adapter starts from the `.meta` data this ticket writes.

## Nick input

None. The pictures must not change: every converted file comes out
byte for byte the same (below), so nothing on screen can.

## Scope

**In:**
- Move the pin past engine#2245 (`engine-change` skill §7, with its
  `engine-pin` ledger row), unless an open `engine-pin` PR has.
- `KINDS` and `NOT_BACKDROPS` as `.meta` data: `_folder.meta` in
  `assets/portraits`, `battlers`, `battlebacks` and `title`, and
  `(convert: ())` in `title/logo.png.meta` and `title/game_over.png.meta`.
- `playdate_images.rs` keeps only what reads game data: the battler
  sidecars' scale (doubled), `jobs`, `touched_up`/`touch_ups`.
- `build.rs`, the xtask tool, `bundle.rs`'s and the battle stage's tests
  on the engine's types.

**Out (do not do):**
- Changing a picture, a style, a size or the 4 × 4 Bayer look.
- Baking a pack or using target layers (1170 decides the export path).
- 1042's faces and touch-ups, and the private-art layout (ADR-0040):
  they stay as they are, and `cargo xtask playdate-images`, the tool
  1042 converts with, keeps its command and its output.

**Either order with 1233:** if 1233 is done, its `bundle.rs`/`build.rs`
row (`docs/engine/boundary.md:165`) says the one-bit pictures move with
1221: write them as moved; if not, change that row's words here, and
1233 leaves them.

## Implementation steps

1. **Pin** past engine#2245.
2. **Baseline:** build `trpg-content` (and with `private-assets` where
   `assets-private/` is checked out), run `cargo xtask playdate-images`,
   and save a SHA-256 list of every file under `OUT_DIR/onebit*/` and
   `target/playdate/images/` to the scratchpad.
3. **Data:** portraits `scale: (2, 1)`, styles `a`, `a-lines` (`curve:
   800`, `lines: Some((contrast: 300, dark: 150))`), `f-lines` (as
   `a-lines`, `levels: 5`), `g` (`levels: 4`); battlers style `a`, `rim:
   2`; battlebacks `scale: (25, 21)`, `lift: Some((scale: 600, floor:
   120))`, style `a` with `stretch: 0`; title `scale: (25, 21)`, style
   `a` with `curve: 800`.
4. **Code:** delete `Kind`, `Style`, `style`, `Scale`, the `STYLE_*`,
   `LINES`, `LIGHTER`, `BACKDROP`, `BATTLE_*`, `SCREEN_WIDE`,
   `FIGHTER_RIM`, `NOT_BACKDROPS`, `kind_of`, `Lift`, `make`, `with_rim`.
   `Job` holds a `ConvertSettings` read by `import_settings::nearest`
   (the private folder first, then `assets/`), its `scale` set from the
   sidecar for `battlers/`. `build.rs` calls `job.settings.make`; the
   xtask names style files by `ConvertStyle::name`; `bundle.rs`'s test
   asks whether a path's settings are on; the stage tests use the
   engine's `with_rim` and the battlers' `rim`.
5. **Compare** the SHA-256 lists: identical. Add a dated note to
   ADR-0085 that the machinery is the engine's (engine#2245).
6. **Measure** with `cargo xtask code-lines`.

Expected size: about 170 production lines removed, about 30 added, and
about 45 lines of `.meta` data; about 100 test lines removed.

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed engine#2245.
- [ ] The before and after SHA-256 lists of step 2 are identical (public
      pictures always; the bought ones when checked out); the
      Completion notes give the file count and the list's own hash.
- [ ] `grep -n "fn with_rim\|struct Lift\|KINDS" crates/content/src`
      finds nothing.
- [ ] `crates/content/src/formats/tests.rs` lists the `convert` section.
- [ ] All gates in the `run-gates` skill pass, including the CI
      `playdate` job.

## Tests required

- Unit: `jobs` (a battler's scale from its sidecar, a portrait's four
  styles in order, the logo off), touch-ups as now.
- Integration: the Playdate table test in `bundle.rs`; the stage tests.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
