---
id: "1319"
title: "Move and attack ranges are stronger and easy to tell apart"
type: tuning
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
blocks: ["1064"]
nick_input: none
completed: 2026-11-23
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

- [x] Before/after pictures per theme in the PR.
- [x] The contrast test passes for every palette.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the palette contrast test.

## Lesser targets

Playdate: unchanged (patterns). GBA: no target.

## Completion notes

- **Colours**: move `#3a6fd0`, attack `#d83020`, heal `#3a9a4a`, the
  same in every theme but the Playdate's (its patterns are out of scope);
  the danger zone is unchanged. The attack red is a notch redder than the
  ticket's starting `#d04030`: over the picture look's mint grass the
  tint blends toward tan, and the purer red keeps it rust instead.
- **Nick's decision** (2026-11-23, shown animated before / halfway /
  after renders of Chapter 1 in every theme and look): the range colours
  don't change with the theme, since a washed-out theme such as Parchment
  washed them out too. Recorded in `docs/design/look-and-feel.md` (its
  theme table follows) and in each palette file.
- **Stronger tints in the picture look**: Nick also picked tints at 150%
  of today's strength over the bought tiles (attack tiles read red, not
  rust, over grass). That needs the engine change this ticket named (a
  tint strength the game sets per skin), so it is its own game ticket
  after the engine's, as his rule for engine steps says: engine#3243
  (`Zafnok/shuyi-engine`, a tint strength a game sets for each skin),
  merged as Zafnok/shuyi-engine#545 and pinned by #808; then 1343 sets
  the picture look's tints to 150%. The glyph look keeps today's strength.
- **The tests** (`crates/ui/src/map_view/range_contrast_tests.rs`):
  every theme has the same range colours; and in every theme, the move
  and attack tints, over `grass_bg`, `forest_bg`, `road_bg` and the
  bought grass tile's colour (`#59cf93`, measured from a picture: the
  gates never read the art), at the weakest strength any range style
  tints (the shimmer's trough, 0.45), are at least ΔE 30 (CIE76) and 75°
  of hue apart. The old colours fail it in every theme (the default's
  were ΔE 16 and 32° apart over the grass picture); the new ones pass
  with ΔE 42.6 and 111° at the least.
- **Pictures**: `docs/screenshots/1319-ranges-glyphs.png` and
  `1319-ranges-pictures.png`, before and after in each theme, on the test
  Quick Battle (ADR-0118: no tracked picture of a real map). Chapter 1
  was rendered too, locally, and reads the same way.
- **Left for the Act 1 playtest (1064)**: whether enemy initials on an
  attack tile, and player initials on a move tile in Parchment, are
  still easy to read in the glyph look.
