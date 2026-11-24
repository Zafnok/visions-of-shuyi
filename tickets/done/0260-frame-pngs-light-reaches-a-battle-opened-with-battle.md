---
id: "0260"
title: "frame-png's --light reaches a battle opened with --battle"
type: bug
milestone: M1 Engine
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-24
---

# 0260 — frame-png's --light reaches a battle opened with --battle

## Context

Found by 1322 (2026-11-23) while rendering the Chapter 1 battle for a
listening page:

```
cargo run -p trpg-tools --features private-assets -- shuyi frame-png a.png --battle ch01 --light noon
cargo run -p trpg-tools --features private-assets -- shuyi frame-png b.png --battle ch01 --light night
```

**Expected:** `b.png`'s ground is darker and bluer (`assets/data/lighting.ron`,
`night`), as the option's help says: "fight every battle in this light".
**Actual:** the two files are byte for byte the same.

Cause (`crates/tools/src/game_tools.rs`, the function that builds the
harness from `Start`): `Opens::Battle` starts the battle inside
`Harness::with_layout_starting(…, |ctx| start_on_battle(ctx, id))`, and
only **after** that does `if let Some(light) = &start.light` add the light
to `content.battle_lights`. The battle's look (`Content::battle_look`,
`crates/content/src/lib.rs`) was already taken, so the light never
reaches it. A light given with the title start (a battle reached by
`--keys`) works, because the battle starts later.

## Nick input

None. Verified by the test below.

## Scope

**In:** apply `--light` before any start opens a battle or a scene; a
test.

**Out (do not do):** changing what a light looks like (0438); the MCP
session's own light, unless it has the same order bug (then fix it the
same way and say so in the PR).

## Implementation steps

1. In `crates/tools/src/game_tools.rs`, set `battle_lights` in the
   `setup` closure every start runs (or before the `match &start.opens`),
   so the content has the light when `start_on_battle` or
   `start_on_scene` runs. Keep the unknown-light error as it is.
2. A test in `crates/tools/src/game_tools/tests.rs`: `--battle <the test
   chapter's battle> --light night` and the same without `--light` give
   different pictures (use the battle the other `--battle` tests use). A
   light shades only the pictures of the ground, never glyphs: if the
   test build draws that battle's ground as glyphs, assert instead that
   the open battle's look carries the `night` light (the harness's
   content, `Content::battle_look`).

## Acceptance criteria

- [x] The test above passes, and fails on today's code.
- [x] `a_light_must_be_one_of_the_lights` still passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the tool's own test (above).

## Completion notes
`--light` is now applied in `harness()`'s `setup` closure (a new
`light_every_battle`), which every start runs before it opens the title,
a battle, a scene or the script preview, so `start_on_battle` takes the
battle's look with the light already in `content.battle_lights`. An
unknown light still fails with the same message, now returned after the
harness is built.

The MCP session (`start`) builds its game with the same `harness()`, so
it had the same order bug and is fixed by the same change.

Test: `the_light_reaches_a_battle_opened_at_once`
(`crates/tools/src/game_tools/tests.rs`). The test build draws the test
battle's ground as glyphs, which no light shades, so it reads the open
battle's map scene look (`night` with `--light night`, not `night`
without); it fails on the old code (`noon`).

No gameplay rules decided. No follow-up tickets.
