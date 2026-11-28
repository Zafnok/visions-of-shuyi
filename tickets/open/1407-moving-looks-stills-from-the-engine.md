---
id: "1407"
title: "Moving looks' stills from the engine: the game's Motion, Still and check go"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: blocked
blocked_by: ["engine#2288"]
nick_input: none
completed:
---

# 1407 — Moving looks' stills from the engine

## Context

The third engine-friction sweep (1397, `docs/engine/friction.md`) found
`crates/content/src/motion.rs` (about 105 production lines: `Motion`,
`Still`, `Show`, `Drawn`, `Motion::drawn`, `check`, from 1314 and
ADR-0131) written entirely over the engine's `GraphicsTier`. Nothing in
it names a game concept. Engine ticket 2288 moves that shape into
`shuyi_assets::target`. The file is marked on its first line
`// engine-friction: engine#2288, moves (game 1407 deletes it)`.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine 2288 if it isn't (`engine-change` §7).
- `motion.rs` becomes a re-export of the engine's types (or goes, with
  its users importing the engine's); the content test
  `every_moving_look_names_its_stills` calls the engine's `check`.
- The data files keep their `motion:` fields as written (the engine's
  RON form is the game's).

**Out (do not do):**
- The game's moving tiles and trees (`tileset/frames.rs`, `trees.rs`):
  parked engine#2359 holds those.

## Implementation steps

1. Estimate: about 100 lines removed, 5 added.
2. Move the pin if needed; swap the types; run the content tests.

## Acceptance criteria

- [ ] `grep -n "engine#2288" crates/` finds nothing.
- [ ] `every_moving_look_names_its_stills` passes, and its report lists
      the same stills to do as before.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing content and skin tests, unchanged; `motion.rs`'s own
  tests go with it (the engine's cover them).

## Completion notes

*(Filled in by the session that completes the ticket.)*
