---
id: "1405"
title: "The tileset importer's corner sheets from the engine's Tiled, patch and quarter recipes"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#2286"]
nick_input: none
completed:
---

# 1405 — The tileset importer's corner sheets from the engine's recipes

## Context

The third engine-friction sweep (1397, `docs/engine/friction.md`) found
about 355 production lines in `crates/tools/src/tileset_import.rs` that
turn bought kits into the 15 mixes of a corner layer: the Tiled reader
(`tags`, `unescape`, `corner_terrains`, `parse_tsx`, `Sheet::corners`),
the 3×3 patch (`PATCH`, `patch_layer`) and the RPG Maker quarter kits
(`Kit::quarters`, `quartered`, `joined`, `quarters_layer`). None of it
names a game concept. Engine ticket 2286 (`Zafnok/shuyi-engine`) adds
`Make::Tiled`, `Make::Patch` and `Make::Quarters` corner recipes beside
`Generated` and `Rotated`. Each item it replaces is marked
`// engine-friction: engine#2286, moves (game 1405 deletes it)`.

## Nick input

None. The imported sheets must come out the same: the tileset tests and
`cargo shuyi frame-png` pictures of the maps show it.

## Scope

**In:**
- Move the pin past engine 2286 if it isn't (`engine-change` §7).
- The importer builds each corner layer from the engine's recipe for its
  kit; the marked code goes.
- The importer's mapping file keeps naming the kit per layer (its own
  `LayerMap`), translated into the engine's `Make`.

**Out (do not do):**
- The game's own tileset format (`tileset_ron.rs`), trees, walls,
  shadows, props and faces: they stay.
- Re-importing or changing any bought art.

## Implementation steps

1. Estimate: about 360 lines removed, 40 added (the translation from
   `LayerMap` to `Make`).
2. Move the pin if needed; read 2286's completion notes.
3. Replace the Tiled, patch and quarter paths of `layer` with the
   recipes; delete the marked items.
4. Run the importer's `--check` and compare its output with the
   committed tileset files.

## Acceptance criteria

- [ ] `grep -n "engine#2286" crates/` finds nothing.
- [ ] The importer's check reports the committed tileset files
      unchanged (or the completion notes explain a byte difference that
      paints the same).
- [ ] `tileset_import`'s tests pass (their expected sheets unchanged).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing importer tests, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket.)*
