---
id: "1343"
title: "The picture look's range tints are 150% as strong"
type: tuning
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: ["1319"]
nick_input: none
completed: 2026-11-23
---

# 1343 — The picture look's range tints are 150% as strong

## Context

Ticket 1319 made the move and attack ranges brighter and the same in
every theme, after a new player found them hard to tell apart (Nick,
2026-11-22). Shown animated renders of Chapter 1 on 2026-11-23, Nick
also picked the picture look's tints at **150%** of today's strength:
at today's strengths (0.45 to 0.75, by range style), the red attack
tint over the bought tiles' bright green grass blends to rust, and at
150% it reads red. The glyph look keeps today's strength (its red
already reads red).

It uses the engine's `RangeShow::with_strength(percent)` (`engine#3243`,
`Zafnok/shuyi-engine#545`): a tint strength a game sets for each skin,
scaling `RangeShow::strength` and capped at the range's color itself.
The game's pin is already past it (#808 moved it to `7d60a21a`, which
includes `223d751a`, the engine's merge of #545).

## Nick input

None: Nick picked 150% from the renders (2026-11-23). Verified by tests
and `cargo shuyi frame-png` pictures; what is left to judge by playing
goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- The sprite skin's range tints at 150%; the glyph and one-bit skins
  unchanged. The strength applies to every range the sprite skin tints:
  the danger zone and an escape map's exit area too, and the seize point
  of 0488 if that lands first (either order works: 0488's tint then
  takes the same strength with no change here). The pictures of step 3
  show the danger zone too, so the playtest (1064) can judge it.

**Out (do not do):** moving the engine pin (done by #808); the range
colours (1319); a player's option for the strength (none was asked
for); new range styles.

## Implementation steps

1. `crates/ui/src/map_view/skin.rs`, `MapSkin::with_marks`: for
   `Units::Sprite`, build the glyph skin it wraps with
   `show.with_strength(SPRITE_RANGE_STRENGTH)`, a named constant of 150
   with a comment naming this ticket and Nick's pick. The glyph and
   one-bit skins keep `show` as it is.
2. A test beside `with_marks`' tests: the sprite skin's `RangeShow` has
   `strength_percent` 150 for every `RangeStyle`, the glyph skin's 100.
3. `cargo shuyi frame-png` of a unit's ranges in the test Quick Battle,
   picture look, before and after, every theme (ADR-0118: no tracked
   picture of a real map).

## Acceptance criteria

- [ ] The sprite skin tints ranges at 150% for every range style; the
      glyph skin at 100% (the test of step 2).
- [ ] Before/after pictures in the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the skins' range strengths (step 2).
- Snapshots that show the sprite skin's range tints follow (only their
  colours change).

## Lesser targets

Playdate: unchanged (one-bit patterns). GBA: no target.

## Completion notes

- `MapSkin::range_show` (`crates/ui/src/map_view/skin.rs`) gives the
  sprite skin `RangeStyle::show().with_strength(SPRITE_RANGE_STRENGTH)`
  (150) and the glyph and one-bit skins the style's own show;
  `with_marks` builds every skin with it. A small method in place of the
  inline call in step 1, so the test of step 2 can read the strength
  (the kit skin behind `MapSkin` is a `dyn GridSkin`).
- Test: `the_picture_look_tints_ranges_at_150_percent_and_the_others_at_100`,
  every `RangeStyle`, and the rest of the look unchanged.
- Pictures: `docs/screenshots/1343-ranges.png` (a selected unit's
  ranges) and `1343-danger.png` (the danger zone), test Quick Battle,
  picture look, every theme, before and after.
- No starting rules decided.

### For the Act 1 playtest (1064)

- For the Act 1 playtest (1064): at 150% the danger zone over the bought
  grass is much darker (a deep brown-red rather than a light green-brown
  wash); judge whether it still lets the terrain read.
