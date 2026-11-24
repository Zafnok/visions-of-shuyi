---
id: "1320"
title: "The game fills a 1280x720 window instead of sitting small in its middle"
type: bug
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1320 — The game fills a 1280x720 window instead of sitting small in its middle

## Context

Nick (2026-11-22):

> "When not in full screen, it seems the battle viewport is smaller than
> the allotted viewport like on 1280x720 the battle viewport is not
> expanded to its full potential"

Cause: the console is a fixed 100×32 cells of 8×16 px, 800×512 px
(`crates/ui/src/console.rs`, `ConsoleSpec::PC`); the window starts at 2×
plus 40 px (`crates/app/src/main.rs`). The engine's `layout()` (shuyi
engine `crates/ui/src/frame/console.rs`) takes the largest **whole**
scale that fits, at least 1, and letterboxes the rest. At 1280×720,
min(1280/800, 720/512) = min(1.6, 1.41) → scale 1: an 800×512 picture in
wide black bars. The map viewport is a fixed `MAP_VIEW` of 70×30 cells
(`crates/ui/src/screens/battle/layout.rs`).

Not covered by 1217 (the battle's camera on the engine's `Viewport`, a
refactor with no change a player sees, blocked by engine#3206) or 0072
(when the map scrolls). Related done tickets: 1283 (window sized by the
console), 1108, 1030, 1200.

## Decision (2026-11-23)

**Fractional scale, drawn sharp, through the engine's engine#2333**
(engine ADR-0140). Weighed:

- **Fractional scale with sharp bilinear** (chosen): the engine draws
  the 100 × 32 console at a whole scale into a texture, then stretches
  it with linear filtering, so a pixel's edge blends over one window
  pixel at most. No screen changes: they keep laying out on 100 × 32.
  Nick approved one tweak in the engine: `Fit` keeps the whole scale
  when that already fills 90 % or more (1080p at 2×, 4K at 4×: 95 %), so
  those stay pixel-perfect. At 1280 × 720 the game becomes 1125 × 720
  (100 % of the height); at 1366 × 768, 1200 × 768; at 1600 × 900,
  1406 × 900; at 800 × 600, 1× already fills the width.
- **More cells** lost: it needs the engine's parked 2323 and 2376 and
  every screen of the game laid out against any console size (title,
  dialogue, menus, cinematics and art are drawn for 800 px), several
  PRs past the 3,000-line limit. Text stays at 1× on 1280 × 720.
- **A mix** lost for the same reason.

Engine PR: Zafnok/shuyi-engine#541 (2333, merged). ~~The pin moves in a
session of its own, after he confirms.~~ Done by the pin move of
2026-11-23 (`docs/engine/pins.md`).

What is left here, now that the pin is past 2333:

1. `impl GameSettings for` the game's settings: `const DEFAULT_SCALE:
   ScaleMode = ScaleMode::Fit;`. The Playdate ignores it.
2. `shuyi.ron`'s `desktop` and `web` targets: `scale: fit`, so
   `check --target` checks the text at the scale the window draws. (The
   check comparing them with `DEFAULT_SCALE` is engine#3242, not built
   yet: keep them equal by hand.)
3. The game's test of the placement (step 1, `crates/ui/src/console.rs`):
   `place(ScaleMode::Fit, (800, 512), window)` at 1280 × 720,
   1366 × 768, 1920 × 1080, 1600 × 900 and 800 × 600, each at least 90 %
   of one side; today's `layout` values beside them.
4. A game ADR recording this choice (pointing at engine ADR-0140).
5. Step 3's "two console sizes" is moot: the console stays 100 × 32.
   Say so in the completion notes.
6. A windowed screenshot at 1280 × 720 before and after.

## Nick input

None (technical). Record the choice as an ADR.

## Scope

**In:** pick and build one way the frame uses the window; options for
the agent to weigh (record which in an ADR):
- **Fractional scale** of the whole frame (nearest-neighbour at
  1.41× smears 8×16 glyphs and 16 px tiles; a sharp-bilinear filter may
  be acceptable);
- **More cells**: the console grows by whole cells to fill the window at
  the largest whole scale (1280×720 at 1× → 160×45 cells), and the battle
  map's view (`MAP_VIEW`) grows with it, showing more tiles; screens lay
  out against the console's size, not constants;
- a mix (whole scale, then more cells for the remainder).
- Engine half via `engine-change` if `layout()` or the console size
  must change there.
- Check web (the canvas in the itch.io frame) and the window's resize.

**Out (do not do):** the Playdate (fixed 400×240); 1217's refactor (if it
lands first, build on the engine's `Viewport`); the zoom key (0439).

## Implementation steps

1. A test of `layout()` (or the game's wrapper) at 1280×720, 1366×768,
   1920×1080, 1600×900 and 800×600: today's scale and bars.
2. Decide the approach (ADR), engine PR if needed, ~~pin move~~ (done
   by the pin move of 2026-11-23).
3. Make the battle screen's layout (`layout.rs`) and the screens that
   assume 100×32 read the console's size; tests at two sizes.
4. `frame-png` (or a windowed screenshot) at 1280×720 before and after.

## Acceptance criteria

- [ ] At 1280×720 the game uses at least 90% of the window's height or
      width (a test of the layout).
- [ ] Glyph text stays sharp (no fractional nearest-neighbour), or the ADR
      says why the chosen filter is acceptable.
- [ ] Battle, menus and dialogue lay out correctly at two console sizes
      (tests).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the layout at several window sizes.
- Integration: screens' views at two console sizes.

## Lesser targets

Playdate's screen is fixed; its console must stay 400×240's. Web: the
same as desktop. GBA: no target.

## Completion notes

