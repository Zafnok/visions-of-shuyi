---
id: "1357"
title: "Moving tiles through the engine's tile motion"
type: infra
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["engine#2359"]
nick_input: none
completed:
---

# 1357 — Moving tiles through the engine's tile motion

## Context

1315 (2026-11-24) gave the battle map's tiles frames. The kit's sprite
skin (`shuyi_ui::grid::sprite`) paints one picture per cell and knows no
frames, so the game works round it:
`crates/ui/src/map_view/sprite/moving.rs` builds, for each frame in
which a cell moves, a copy of the kit's skin whose look has each moving
cell's picture swapped for its frame then (`frame_skin`), and
`MapSkin::paint` paints with it. It is marked
`// engine-friction: engine#2359, moves (game 1357 deletes it)`.

Engine ticket 2359 gives the grid kit exactly this: `TileMotion` (a
still picture's rectangle to a clip and a phase), `SpriteSkin::
with_motion`, and the ground painting each rectangle through it at
`MapScene::clock_ms`, with per-tile phases. It is parked behind 2231
(animation clips); this ticket is the game asking for it, which
unparks both (Nick's call: the `engine-change` skill, his OK before an
engine PR).

**2026-11-27, from 1378:** a look's props (Chapter 1's lamps) move too:
`TileFrames::look_at` (`trpg_content::tileset::frames`) swaps
`Look::props` with the rest, so `frame_skin` paints a lamp's frame, and
a moving cell drawn only as a prop moves at the prop's size (16 × 32).
`moving.rs` itself only gained tests. engine#2359 covers it: its
Context says its frames apply to `Look::props` as to the other
rectangles; its `TileMotion` keys clips by still rectangle, which a
prop's 16 × 32 rectangle is.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine#2359 (`engine-pin` skill).
- Build the kit's `TileMotion` from the tileset's `frames`
  (`trpg_content::tileset::TileFrames`) and give it to the sprite skin
  with `with_motion`; delete `moving.rs`, `MapSkin`'s `sprite_glyph`
  and the swap in `MapSkin::paint`.
- Keep the game's rules: a cell's first frame on a tier below its
  `motion.lowest` (`ctx.platform.graphics`) and while idle motion is off.
  If 2359's skin can't see the tier, the game hands it a motion with only
  the cells that move on this tier.

**Out (do not do):** new moving tiles; trees (1356).

## Implementation steps

1. Check engine#2359 is done and the pin passes it.
2. Replace the workaround; keep `moving.rs`'s tests (moved where the new
   code is) passing unchanged.
3. Run the gates and `cargo shuyi bench`.

## Acceptance criteria

- [ ] No `engine#2359` marker is left in the game.
- [ ] 1315's skin tests pass unchanged (water frames at fixed times,
      idle motion off, the lesser tiers, every zoom).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Look: 1315's, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket.)*
