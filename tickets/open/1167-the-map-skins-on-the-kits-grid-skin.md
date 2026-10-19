---
id: "1167"
title: "The map skins on the kit's grid skin: the game's MapSkin trait and the one-bit skin's copied paint order go"
type: feature
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: todo
blocked_by: ["engine#2313"]
nick_input: none
completed:
---

# 1167 — The map skins on the kit's grid skin

## Context

The first engine-friction sweep (1153, `docs/engine/friction.md`)
measured the map view (`crates/ui/src/map_view*`) at 1266 production
lines against an estimate of 750 (1.7×). About 190 lines are glue:

- `map_view/skin.rs`'s `MapSkin` repeats the kit's `GridSkin` method
  for method, adding zoom, a third walk speed and a `paint` that takes
  the game's `Ctx`; `map_view/glyph.rs` and `map_view/sprite.rs` only
  forward to the kit;
- `map_view/playdate.rs` copies the kit glyph skin's paint order, its
  `draw_tiles` and `draw_terrain`, differing in how ranges and flashes
  look and a side margin;
- `map_view/scene.rs` converts the game's range kinds to the kit's
  names and back.

Engine ticket 2313 (`Zafnok/shuyi-engine`) gives `GridSkin` zoom, a
held walk speed and a documented per-frame painter, and `GlyphSkin` a
`GlyphStyle` (range fill or tint, flash blend or invert, a margin).

## Nick input

None.

## Scope

**In:**
- Move the pin past 2313 (`engine-change` skill, step 7).
- The game's skins are kit `GridSkin`s with the game's unit painters;
  `MapSkin` and its forwarding impls go. Where the battle screen needs
  the game's context to paint units, build the painter in `draw`.
- The one-bit map skin is the kit's `GlyphSkin` with a `GlyphStyle`
  and the game's one-bit unit painter; its copied paint order goes.
- The range-kind conversion goes if the kit's names can be used
  directly; otherwise say why in the Completion notes.

**Out (do not do):**
- How the units look (the painters stay the game's: ADR-0038).
- Any change the player sees, on the PC or the Playdate.

## Implementation steps

1. Write the estimate: about 190 lines removed, 40 added.
2. Glyph skin first, then sprite, then one-bit; map snapshots unchanged
   after each.

## Acceptance criteria

- [ ] `grep -rn "trait MapSkin" crates/ui/src` finds nothing.
- [ ] Every map snapshot (glyph, sprite, one-bit) is unchanged.
- [ ] `cargo xtask code-lines` shows `trpg-ui` fell by at least 120.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing map skin tests and snapshots, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
