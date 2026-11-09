---
id: "0488"
title: "The seize point stays visible when a unit stands on it"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0488 — The seize point stays visible when a unit stands on it

## Context

Nick's playtest of the Act 1 build (2026-11-08):

> "ch3's throne is extremely hard to see and requires using cursor over
> the tile to identify while it's occupied by the boss"

Chapter 3's objective is `Seize(pos: (16, 3), by_lord: true)`
(`assets/battles/ch03.ron` ~:60). The tile is the `throne` (`'K'` in
`assets/maps/ch03.map`, legend ~:24), the prior's seat: the boss
(`greywater_prior`, Stationary, `ch03.ron` ~:44) sits on it from turn 1,
with a Hound swordsman at (17, 3) and a warden guard at (18, 4) beside him.

How the tile looks:

- **Glyph look:** the throne is `♔♕` (`assets/data/terrain.ron` ~:88, fg
  `fort`, bg `floor_bg`). A unit's two letters replace a tile's glyphs
  (`crates/ui/src/map_view/glyph/units.rs`, `draw_unit`, which keeps the
  cell's background), so with the boss on it the tile is a unit on a floor
  background like any other.
- **Picture look:** Tiny Tales has no throne; the stand-in is a chair
  (`assets-src/tilesets/tiny_tales.ron` ~:35); the boss's sprite covers it.
- Nothing in the map scene says which tile is the objective. The only
  place that names it is the map menu's Objective line, "Seize the
  {place}" (`crates/ui/src/screens/battle/map_menu.rs` ~:237–248,
  `ui.ron` ~:574), and the terrain panel when the cursor rests there.

There is a pattern for this: an escape map's exit area is always tinted in
the map scene (`BattleScreen::add_ranges`,
`crates/ui/src/screens/battle/mod.rs` ~:2303–2309, range `range::EXIT`,
colour `exit_area`, `crates/ui/src/map_view/scene.rs` ~:41), under every
other range, and each map skin paints it: the glyph skin tints the tile's
background (which a unit's letters keep), the picture skin lays its
overlay, and the Playdate's one-bit skin fills it with its marked glyph
(`map_view/playdate.rs`, `ONE_BIT`, `RangeLook::Fill`).

## Nick input

None. Verified by the scene tests below and `cargo shuyi frame-png`
pictures of Chapter 3's turn 1 in the glyph look, the picture look and
the Playdate's in the PR. The mark's colour is *the agent's starting
rule* (ADR-0038: a skin's look, from the palette), and goes on the Act 1
playtest's checklist (1064).

## Scope

**In:**
- A new scene range `range::SEIZE` (colour name e.g. `seize_point`,
  `UiColor::SeizePoint`, `crates/ui/src/color.rs`), added to
  `range::ALL`. Its letter in the scene's text must not clash with
  `d m a h e`.
- `add_ranges` tints a `Seize` objective's tile with it, always, under
  every other range (as the exit area), whoever stands on it.
- Its colour in every palette: `assets/data/palette.ron` and
  `assets/data/palettes/{moonlit,painterly,parchment,playdate}.ron`. Pick
  a colour that stands apart from the move, attack, heal, danger and
  exit colours and from the throne's and gate's backgrounds (look at
  the frame-png pictures; `ascii-art` skill for the glyph look).
- Every map skin paints it through the range it already paints (glyph,
  picture, one-bit). If one of them skips ranges under a unit, fix that
  skin for this range.
- One sentence for the seize point's mark beside the exit area's
  (`docs/design/turn-structure.md` ~:397, "The map marks an exit area")
  and in `docs/design/look-and-feel.md`'s battle map section.

**Out (do not do):**
- Better throne art (0471, Nick picks bought art). Either order: the mark
  is independent of the tile's picture; if 0471 is done first, take the
  pictures with its throne.
- The objective line naming the place (0331) and the objective shown when
  a battle starts (0481). Either order; neither touches the map scene.
- Marking other objectives (a boss, a defend point).

## Implementation steps

Expected size: about 40–60 production lines (a range, a colour, one tint
in `add_ranges`, palette entries), plus tests.

1. Write the failing test first: on Chapter 3's battle, the scene's tints
   at (16, 3) contain the seize range on turn 1, with the boss on it.
2. Add the range, the colour and the tint.
3. Check each skin paints it under a unit; take the three pictures.
4. Update `scene/tests.rs` (`ranges_are_named_by_their_colours`), the
   sprite skin's per-range test (`map_view/sprite.rs` ~:937 loops over
   `range::ALL`), the palette tests and snapshots.

## Acceptance criteria

- [ ] On Chapter 3's battle at turn 1, `h.tints_at((16, 3))` contains
      `range::SEIZE` while the prior stands there (Harness test, like
      `the_exit_area_is_on_the_map` in `screens/battle/act1_tests.rs`).
- [ ] A battle with another objective has no seize tint (test).
- [ ] With a unit selected, its move range still shows over the seize
      tint on other tiles, and the seize tile keeps its tint under the
      ranges' order (test of the scene's layer order).
- [ ] Every palette has `seize_point` (the palette check passes).
- [ ] `frame-png` pictures of Chapter 3 turn 1 around the throne, in the
      glyph look, the picture look (`--features private-assets`) and the
      Playdate skin, are in the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the scene's ranges (`map_view/scene/tests.rs`).
- Integration: the Harness test on Chapter 3.
- Snapshot: the map snapshots that show a Seize battle, updated.

## Completion notes

*(Filled in by the session that completes the ticket.)*
