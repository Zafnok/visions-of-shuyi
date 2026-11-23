---
id: "1334"
title: "Small touches on the battle map: dust, splashes and creatures"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["1323", "1316", "1314"]
nick_input: answer-first
completed:
---

# 1334 — Small touches on the battle map: dust, splashes and creatures

## Context

1323 decides which small touches live on the battle map: dust as a unit
steps, a splash in shallow water, leaves shaken in a forest, a burst on
the target's tile when combat animations are off, and creatures (birds,
butterflies, crows, gulls) that pass by or scatter. 1316 brings the
engine's particles and the tier cap; 1314 the tiers.

Anything shown on the battle map goes in the map scene and is painted
by the map skins, never drawn straight into the buffer (AGENTS.md rule
8, ADR-0038): `crates/ui/src/map_view/scene.rs` and the skins in
`crates/ui/src/map_view/` (the sprite skin's `sprite/units.rs` already
reads time for idle motion). A unit's walk is shown at the player's speed
from the time alone (`shuyi-animate`); the touches key off that walk.
Creatures are decoration: `core`, the bots and play records never see
them, and they never stand on a tile the cursor or a unit needs to show.

## Nick input

**Answer first:** 1323. Nothing to check after merge: verified by tests
and `frame-png` strips; what's left to judge in play goes on 1064's
checklist.

## Scope

**In:**
- The touches 1323 picked, as effect data and scene items: a step puff
  keyed by the terrain under the step (a terrain table, so a new terrain
  is a row), the map hit burst, and creatures with a spawn rule (per map,
  from 1335–1342's data; the test chapter names one).
- Creatures that scatter: computed from units' places at the scene's time
  (seed and time, no state kept by the rules).
- Both map skins paint them; the glyph look with glyphs.
- They hold still or vanish with `ctx.idle_motion` off (and 0093's switch,
  if built).

**Out (do not do):** drifting effects (1316); moving tiles (1315);
choosing each map's creatures (1335–1342); the fight's bursts (1333).

## Implementation steps

1. Write the expected size (about 300 production lines: scene items, a
   terrain table, a creature rule, two skins) and measure (ADR-0092).
2. Scene items in `map_view/scene.rs`; tests that read the scene.
3. Skins; tier cap; idle motion.
4. `frame-png` strips of a walk through grass and a ford, and a bird
   flying off.

## Acceptance criteria

- [ ] A test: a unit stepping onto a ford tile puts a splash in the
      scene at that tile and time.
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

Off below `Full` (1314) unless 1323 says otherwise; GBA: no target.

## Completion notes
