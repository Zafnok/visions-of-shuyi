---
id: "1219"
title: "Tileset terrain looks on the engine's format: look.rs and the terrain half of the tileset checks go"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#2243"]
nick_input: none
completed:
---

# 1219 — Tileset terrain looks on the engine's format

## Context

The game/engine boundary audit of 2026-11-03 found that
`crates/content/src/tileset/look.rs` (191 production lines: `LookFile`,
`LayerFile`, `CornersAt`, `SidesAt`, `Looking`, `missing`,
`name_problem`) and the terrain half of `tileset.rs` (`Checker::inside`,
`grid_rect`, `inside`, the `tile_px` and `looks` rules of
`own_problems`, the look loop of `parse_tileset`) name nothing of this
game but `TILE_LOOKS`, the terrain lookup and the words "not in
terrain.ron". `docs/engine/boundary.md` contradicts itself: line 170
marks `tileset/look.rs` E, line 177 keeps the format here "because the
terrain names are this game's", which 1116's terrain resolver for maps
made stale.

[engine#2243](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/2243-terrain-tiles-for-a-tileset-as-a-format.md)
adds `shuyi_assets::tileset::{TerrainTilesDef, LookDef, LayerDef,
TileRules}`, their `problems` and `resolve`, and `ImageRect::cell` and
`ImageInfo::holds`. This ticket moves the tileset loader onto them. The
tileset files don't change: their fields are the engine's spelling.

## Nick input

None. Nothing a player sees changes; an unknown terrain's error text
does (the engine's words).

## Scope

**In:**
- The pin past engine#2243.
- `TilesetFile`'s `layers` and `looks` typed with the engine's `LayerDef`
  and `LookDef`; when `terrain`, `image` and `tile_px` are all written, a
  `TerrainTilesDef` built from them, checked with `problems(&TILE_RULES)`
  and resolved with `resolve`.
- `look.rs` deleted; `grid_rect` and `inside` replaced by
  `ImageRect::cell` and `ImageInfo::holds` in the unit-picture checks.
- The tests of moved checks deleted; the game keeps its own.
- `docs/engine/boundary.md` lines 170 and 177.
- **Either order with 1222:** if 1222 is done, drop `tileset/look.rs`
  from the line-170 row it left; if not, remove `tileset/look.rs` from
  that row here (1222 then removes `portrait.rs`).
- **Either order with 1233:** if 1233 is done, edit the tileset rows as
  it left them; if not, edit lines 170 and 177 as they stand (1233
  keeps this ticket's wording).

**Out (do not do):**
- Changing a tileset file, `assets/tilesets/README.md`'s format, units,
  walking sheets or `LEAD_PICTURES`.
- Moving the tileset into the registry (`formats.rs`): its units need the
  class and character tables.

## Implementation steps

1. **Pin** past engine#2243 (`engine-change` §7, `engine-pin` skill),
   unless an open `engine-pin` PR has; read its API.
2. In `crates/content/src/tileset.rs`: a `TILE_RULES:
   TileRules<'static>` of `map::TILE_LOOKS` and `SIDE_PX`; the terrain
   map `BTreeMap<String, TerrainId>` from `refs.terrain` (each
   `TerrainDisplay`'s id with `id_of`); in `parse_tileset`, the def's
   `problems` and `resolve` errors joined to the unit problems; keep
   `own_problems`' id, `unit_px` and "terrain needs `image`/`tile_px`,
   layers and looks need `terrain`" rules.
3. Re-export `CornerLayer, Layer, Look, MIXES, TileLayer, mix, mix_name,
   parse_mix` from `shuyi_assets::tileset` (xtask's `tileset_import`
   uses `mix` and `parse_mix`). Delete `tileset/look.rs`.
4. Tests: delete `a_tileset_reads_its_layers_and_its_other_looks`,
   `a_layer_names_real_terrain_and_real_mixes_inside_the_image`,
   `every_look_has_a_tile_for_every_terrain_and_a_name_a_map_can_give`,
   the rectangle cases of `every_rectangle_lies_inside_the_image`, and
   look.rs' test; keep `the_embedded_test_tileset_loads`, the unit and
   coupling tests, and add one that an unknown terrain in a layer comes
   back with the tileset's file.
5. Boundary rows (above): `tileset.rs` keeps units and the file's
   coupling; the ground's format is the engine's.
6. **Measure** with `cargo xtask code-lines`: estimate about 230
   production lines removed and 35 added; about 250 test lines removed.

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed engine#2243.
- [ ] `crates/content/src/tileset/look.rs` doesn't exist;
      `grep -n "fn grid_rect\|fn inside" crates/content/src/tileset.rs`
      finds nothing.
- [ ] Every tileset in `assets/` and `assets-test/` loads unchanged
      (`the_embedded_test_tileset_loads`, the all-assets test).
- [ ] An unknown terrain in a layer is reported with the tileset's file
      (a test).
- [ ] All gates in the `run-gates` skill pass, including
      `cargo xtask ticket-lint --check-engine`.

## Tests required

- Unit: the kept tileset tests and the new terrain-error test.
- Integration: the all-assets test.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
