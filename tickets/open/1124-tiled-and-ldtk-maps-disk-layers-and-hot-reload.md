---
id: "1124"
title: "Tiled and LDtk maps, folders on disk as bundle layers, and hot reload in debug builds"
type: feature
milestone: "Engine: real time"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1119"]
nick_input: none
completed:
---

# 1124 — Tiled and LDtk maps; disk layers; hot reload

## Context

ADR-0065 §1–§2. Developers draw maps in Tiled or LDtk. The `tiled` crate
(MIT) reads TMX/TSX in pure Rust, but only with default features off,
because its `zstd` feature builds C code. LDtk's crates are stale, but
LDtk publishes a JSON schema and generated serde types (MIT). Both
importers produce one `TileMap`:

- tile layers: a `Grid` of tile ids per tileset;
- object layers: named boxes and points with properties;
- the map's own properties.

Disk folders as bundle layers serve hot reload now and mods later
(1011).

## Nick input

None.

## Scope

**In:**
- `engine_assets::tilemap::TileMap` and the two importers:
  - Tiled: `tiled` with `default-features = false`;
  - LDtk: the vendored generated types with their MIT notice in
    `THIRD_PARTY_ASSETS.md`.
  Both have fixtures made with each tool's free editor.
- `TileCollision` from a named layer (1122).
- Disk layers: `Layer::Disk(path)`, read by `engine-app` and handed to
  the bundle as bytes (only `engine-app` touches files, ADR-0004).
- Hot reload in debug builds: `engine-app` checks the modification times
  of the disk layers' files once a second. It then calls a reload hook
  that re-runs the loaders for the changed files. Errors show in a debug
  overlay instead of crashing.
- `engine-cli import <map> --print` prints the map as our text.

**Out (do not do):** mods' folder and load order (1014); the web build's
disk layers (none on the web).

## Implementation steps

1. Write `TileMap` and the Tiled importer, with fixtures.
2. Add the LDtk types and importer, with fixtures.
3. Add disk layers and hot reload, with a test that changes a fixture
   file through a fake clock and file source.

## Acceptance criteria

- [ ] Both fixtures import to the same expected `TileMap` (a snapshot of
      its text dump).
- [ ] A broken fixture names the file and the place in it.
- [ ] The hot reload test passes.
- [ ] `cargo deny check` passes, and the WASM build passes with both
      importers.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Fixture and snapshot tests as above.
- Unit: reload with a fake file source.

## Completion notes
