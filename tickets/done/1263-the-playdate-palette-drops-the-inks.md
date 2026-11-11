---
id: "1263"
title: The Playdate's palette drops the inks; its pattern map alone says how each colour is drawn
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-11
---

# 1263 — The Playdate's palette drops the inks; its pattern map alone says how each colour is drawn

## Context

The Playdate's palette (`assets/data/palettes/playdate.ron`, ADR-0074)
names an *ink* for each colour (`"text": "black"`, `"cursor": "invert"`),
loaded as the ink's reserved RGB code. Since 1253 the Playdate's build
draws by the pattern map beside it (`playdate@onebit.ron`, engine
ADR-0026), which gives each role the ink's patterns; the codes are read
back nowhere but in tests. Engine PR #362 (engine ticket 2389, ADR-0107)
then removed the inks from the engine (`shuyi_assets::ink`,
`PaletteDef::from_ink_source`), and the pin move that passed it (the
pin PR of 2026-11-11) kept the game unchanged by moving `Ink` and the
loader into `crates/content/src/ink.rs` (about 115 lines with tests).

That copy is a second name for what the pattern map already says. The
palette can be an ordinary one (any RGB per role, loaded with
`PaletteDef::from_source` and checked against the default theme's names)
and the map alone can say how each role is drawn, as the engine's
one-bit games do.

## Nick input

None. Nothing on the Playdate's screen changes: the frames before and
after are compared pixel for pixel.

## Scope

**In:**
- `playdate.ron` gives each role a colour (`"#rrggbb"`, as
  `palette.ron`), the roles that drew alike sharing one, and its header
  comment says the map draws them.
- `trpg_content::ink` becomes the Playdate's palette and map loaders
  only: `Ink`, its codes, `from_ink_source` and `Ink::patterns`
  deleted (about 110 lines removed), the module perhaps renamed
  (`playdate_palette`).
- The game's Playdate code and tests that name an `Ink`
  (`crates/playdate/src/card.rs`, `start.rs`, `crates/ui/src/onebit.rs`,
  `map_view/playdate.rs` and its tests, `screens/*/playdate/tests.rs`)
  name the palette's role, or read the pixel from the map's patterns
  (`PatternMap::glyph`/`shape`/`ground`).
- ADR-0074's §1 ("inks are reserved colours") marked superseded by the
  pattern map (engine ADR-0026, ADR-0107); `crates/ui/README.md`'s
  `onebit` row says the same.

**Out (do not do):**
- New looks for the map's marks (1076).
- Any change to what a Playdate frame shows.

## Implementation steps

1. Record every Playdate frame the tests draw today (`cargo test -p
   trpg-ui` with the one-bit checks, and `cargo shuyi frame-png
   --target playdate` for the title, a battle and a menu) to compare
   after.
2. Rewrite `playdate.ron` with RGB values; load it with
   `PaletteDef::from_source(&display, source, &required)` where
   `required` is the default theme's names.
3. Delete `Ink` and the ink loader; keep
   `content::ink::the_pattern_map_gives_each_role_its_inks_patterns`'s
   job as a test that every role of the palette has a row in the map and
   `shuyi_assets::onebit::check` finds nothing.
4. Move each call site in the list above off `Ink`: a colour by its role
   name from the loaded palette, a pixel check through the map.
5. Compare the frames of step 1: identical.

## Acceptance criteria

- [x] `grep -rn "\bInk\b" crates` finds nothing.
- [x] The Playdate frames of step 1 are byte for byte the same.
- [x] `cargo xtask code-lines` shows the game's production code smaller
      by about 100 lines.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the palette loads with every default-theme name; the map covers
  every role; a missing role is an error.
- Snapshot / integration: the existing one-bit checks
  (`crates/ui/tests/it/playdate.rs`, the map view's Playdate tests)
  unchanged.

## Completion notes

- `assets/data/palettes/playdate.ron` is an ordinary palette: each role
  an RGB colour, loaded with `PaletteDef::from_source` and the default
  theme's names as required. The colours are the old inks' codes, so the
  roles drawn alike share one (`#000000` black, `#ffffff` white,
  `#010101` edged black, `#808080` grey, `#e0e0e0` dots, `#c0c0c0`
  diagonals, `#404040` dashes, `#ff00ff` flip); both files' headers say
  the map draws them.
- `trpg_content::ink` is now `trpg_content::playdate_palette`: the two
  loaders only. `Ink`, its codes, `from_ink_source` and `Ink::patterns`
  are gone. Its tests: the palette loads with every default-theme name,
  the map has a row for every role (and `onebit::check` finds nothing),
  a missing colour is an error.
- Production call sites: the map skin's `Inks::of`, the combat picture's
  `named_ink` and the dialogue's `place_ground` fall back to a default
  theme colour (`UiColor`) when a palette lacks the Playdate's name
  (only the PC's palette in a test), instead of an ink's code. The
  loading card draws in plain black and white, and a test checks they
  are the palette's `text` and `black`.
- Tests name a role (`c("text")`, `c("move_range")`, …) instead of an
  ink, and read a pattern's pixel through the map (`white_at`).
- ADR-0074 §1 and §3 marked superseded by the pattern map (engine
  ADR-0026, ADR-0107); `crates/ui/README.md`'s `onebit` row and the ADR
  index say the same.
- Frames: title, title prompt, title menu, options, a battle, its map
  alone, its menu, a move and a dialogue, recorded with `cargo shuyi
  frame-png --playdate` before and after: all nine byte for byte the
  same.
- `cargo xtask code-lines`: production 59 added, 172 removed (113
  smaller). Friction: 2.4× (estimate 25 added), accepted because a
  changed line counts as added (the skins' fallbacks rewritten from
  `Ink::X` to `UiColor::X`); row in `docs/engine/friction.md`.
- PR size: estimate 900, measured about 1,130.
- No gameplay rules decided. Nothing on the Playdate's screen changes.
