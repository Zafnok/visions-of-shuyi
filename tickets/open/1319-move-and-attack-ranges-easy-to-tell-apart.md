---
id: "1319"
title: "Move and attack ranges are stronger and easy to tell apart"
type: tuning
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1319 — Move and attack ranges are stronger and easy to tell apart

## Context

Nick, after watching a new player play Chapter 1 (2026-11-22):

> "Probably need to make the blue/red move and attack tiles more opaque
> since they're a bit hard to tell the difference for a noob"

Colours: `assets/data/palette.ron` (around line 38): `move_range
#2a4468`, `attack_range #5c2a20`, `heal_range #2e5a34`, `danger_zone
#3e1812`; each theme overrides them in `assets/data/palettes/` (moonlit
`#1c3a6a`/`#5a1a1a`, painterly `#1a5a78`/`#7a2c14`, parchment
`#a8c0e0`/`#e8a898`, playdate grays and the one-bit patterns). The
blend is in the engine at the pin (`crates/ui/src/grid/`):
`glyph.rs` `OVERLAY_BLEND = 0.75`; `ranges.rs` `SHOWN_BLEND = 0.6`,
`GLINT_BASE = 0.55`. The sprite skin paints tints in the engine's
`grid/sprite/ground.rs` (`paint_tints`). The player's `RangeStyle`
(Slab default, Shimmer, Glint, RaisedTiles, Still) is in
`crates/ui/src/settings.rs`.

Both colours are dark and low in saturation, so over the bought grass
tiles blue and red read as two murky shades.

## Nick input

None: Nick asked for it directly. Verified by `frame-png` pictures and a
contrast test; what's left to judge in play goes on 1064's checklist.

## Scope

**In:**
- Brighter, more saturated range colours in every palette. Starting
  values to try (default palette): move `#3a6fd0`, attack `#d04030`,
  heal `#3a9a4a`; then each theme in the same direction (one notch, per
  the memory on nudges).
- If the colours alone don't make it opaque enough, the blend: an engine
  change (`engine-change`, a small engine ticket) making the overlay
  strength a skin setting the game sets (e.g. 0.85 for the tint), rather
  than a constant.
- A test that move and attack tints differ by a minimum contrast (e.g.
  a hue gap and a ΔE over a threshold) over the grass tile, in every
  palette.

**Out (do not do):** the Playdate's one-bit patterns (already distinct
by pattern); new range styles.

## Implementation steps

1. `frame-png` of a Chapter 1 unit selected with move and attack ranges
   over grass, forest and road, in the picture and glyph looks, every
   theme: before.
2. Change the palettes; render again; adjust until both read clearly
   at a glance.
3. If needed, the engine's blend setting.
4. The contrast test in `crates/ui`.

## Acceptance criteria

- [ ] Before/after pictures per theme in the PR.
- [ ] The contrast test passes for every palette.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the palette contrast test.

## Lesser targets

Playdate: unchanged (patterns). GBA: no target.

## Completion notes

