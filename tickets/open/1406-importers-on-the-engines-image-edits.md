---
id: "1406"
title: "The art importers on the engine's image edits: their own picture types and pixel loops go"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#2287"]
nick_input: none
completed:
---

# 1406 — The art importers on the engine's image edits

## Context

The third engine-friction sweep (1397, `docs/engine/friction.md`) found
that the engine's `Image` has no edits, so five of the game's importers
in `crates/tools/src` each roll their own buffer and pixel loops, about
200 production lines:
- `portrait_import.rs`: its `Picture`, `crop` and `split_sheet`;
- `title_art.rs`: `type Picture = (u32, u32, Vec<u8>)`;
- `tileset_import.rs`: `Sheet::tile`, `mirrored`, `paired`, `tint`,
  `brightness`, `recolour`, `halved` and three hand blits;
- `battler_import.rs`: `key_out`, `to_hsv`/`to_rgb`, `recolour`;
- `map_sprite_import.rs`: its grid cells, mirror and re-layout.

Engine ticket 2287 adds crop, grid cell, blit, alpha over, flip, key
out, recolour, tint by brightness, HSV and halve on `Image`. Each item
it replaces is marked `// engine-friction: engine#2287, moves (game
1406 deletes it)`.

## Nick input

None. The imported pictures must come out byte for byte the same.

## Scope

**In:**
- Move the pin past engine 2287 if it isn't (`engine-change` §7).
- The five importers on `Image` and its edits; the marked items go.

**Out (do not do):**
- The importers' tables naming the game's art (`BATTLERS`, `SPRITES`,
  expressions, re-layout constants): they stay.
- 1405's corner kits (its own ticket; either order: the two touch
  different functions of `tileset_import.rs`).

## Implementation steps

1. Estimate: about 200 lines removed, 40 added.
2. Move the pin if needed.
3. One importer at a time onto `Image`: run its check against the
   committed output after each.

## Acceptance criteria

- [ ] `grep -n "engine#2287" crates/` finds nothing.
- [ ] Each importer's check reports its committed output unchanged.
- [ ] The importers' tests pass.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing importer tests, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket.)*
