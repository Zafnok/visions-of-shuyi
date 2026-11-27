---
id: "1334"
title: "Small touches on the battle map: dust, splashes and creatures"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["1323", "1314", "1333"]
blocks: ["1064"]
nick_input: answer-first
completed:
---

# 1334 — Small touches on the battle map: dust, splashes and creatures

## Context

1323 decided (2026-11-27) what lives on the battle map:
`docs/design/look-and-feel.md`, *Particles and small touches* (each
effect's file in the pack is named there):

- **A heal and a level up**, on the unit's tile: green crosses rise out
  of a ring as a heal's `+6` shows; rays of light shoot up out of a ring
  as the EXP bar fills past 100, before the level-up page.
- **Under units' feet**, one touch per tile stepped onto, at the game's
  walking speed: a small dust puff on plain and road; the puff and a few
  falling leaves in forest (thicket too, Claude's starting rule); a small
  splash and a ripple when a foot unit wades into water; nothing on other
  terrain.
- **Hits with combat animations off:** one drawn frame of a small white
  star on the struck unit's tile for a tenth of a second, at each blow
  that lands, one after the other; nothing on a miss.
- **Creatures:** ByteWright's *Ambient Critters* on open plain (0096),
  wandering; one on a tile next to where a unit steps startles, runs a
  couple of tiles away over open plain and settles again. The pack's
  birds hop; only butterflies, bees and dragonflies fly.
- **The Playdate:** none of it.

The art is ByteWright's *Pixel VFX & SFX Mega Pack* and *Ambient
Critters* (bought 2026-11-24, `assets-private/library/bytewright/`):
sheets drawn by the artist with their frame times and anchors in a
`.json`, played as flipbooks by time; not particle emitters, so this
ticket does not need 1316's engine particles (2026-11-27, 1323). It
reuses 1333's effect table and loader (hence blocked by 1333).

Anything shown on the battle map goes in the map scene and is painted
by the map skins, never drawn straight into the buffer (AGENTS.md rule
8, ADR-0038): `crates/ui/src/map_view/scene.rs` and the skins in
`crates/ui/src/map_view/` (the sprite skin's `sprite/units.rs` already
reads time for idle motion). A unit's walk is shown at the player's speed
from the time alone (`shuyi-animate`); the touches key off that walk.
Creatures are decoration: `core`, the bots and play records never see
them, and they never stand on a tile the cursor or a unit needs to show.
1314 (done, ADR-0131) gives the tier (`ctx.platform.graphics`); each
touch carries a `Motion` with `lowest: Full`.

## Nick input

None: 1323 answered it. Nothing to check after merge: verified by tests
and `frame-png` strips; what's left to judge in play (1323's starting
rules) goes on 1064's checklist.

## Scope

**In:**
- The sheets 1323 picked, copied from the library into
  `assets-private/game/` with ByteWright's credit (as in 1333), and their
  rows in 1333's effect table: a step touch per terrain (a terrain table,
  so a new terrain is a row), the heal, the level up, the map hit star.
- Scene items in `map_view/scene.rs` for each, at the scene's time; both
  map skins paint them (the glyph look with 1323's starting rule: the
  font's marks, like the title's embers).
- The map hit with combat animations off: one frame, 0.1 s, per landed
  blow, on the struck unit's tile.
- Creatures: a spawn rule per map (from 1370–1377's data; the test
  chapter names one), only on open plain; startle and run computed from
  units' places at the scene's time (seed and time, no state kept by the
  rules).
- All of it holds still or vanishes with `ctx.idle_motion` off (and
  0093's switch, if built); none below `Full`.

**Out (do not do):** the fight's bursts (1333); drifting effects
(1316); moving tiles (1315); choosing each map's creatures (1335–1342).

## Implementation steps

1. Write the expected size (about 350 production lines: scene items, a
   terrain table, the heal, level-up and hit items, a creature rule, two
   skins) and measure (ADR-0092).
2. Scene items in `map_view/scene.rs`; tests that read the scene.
3. Skins; tier; idle motion.
4. `frame-png` strips of a walk through plain and forest, a wade into
   water, a heal, a level up, a fight with animations off, and a rabbit
   running from a unit (crop, ADR-0118).

## Acceptance criteria

- [ ] A test: a unit stepping onto a water tile puts a splash in the
      scene at that tile and time.
- [ ] A test: with combat animations off, each landed blow puts one star
      on the struck unit's tile for 0.1 s, and a miss puts none.
- [ ] A test: a critter next to a unit's new tile is startled, then
      stands on open plain away from it.
- [ ] A test: no creature is ever on the cursor's or a unit's tile.
- [ ] A test: the `Reduced` tier's scene has none of it.
- [ ] A test: play records replay identically with the touches on.
- [ ] `frame-png` strips in the PR; `cargo shuyi bench` within budget.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the terrain table; the creature rule at fixed times.
- Scene: the items at fixed times (behaviour tests read the scene).
- Look: one snapshot per skin.

## Lesser targets

The Playdate (`Reduced`): none (1323); GBA: no target.

## Completion notes
