---
id: "1111"
title: "The Harness core (key and pad scripts, waits, audio capture, snapshots) moves to the engine"
type: infra
milestone: "Engine: carve"
model: sonnet-5
effort: high
status: todo
blocked_by: ["1110"]
nick_input: none
completed:
---

# 1111 — The Harness core moves to the engine

## Context

The Harness (`ui/src/harness.rs`, ADR-0017) is what lets tests and
agents drive the game headlessly. About 750 of its 1,124 lines are
generic (key, pad and wait scripts in 1/60 s frames, audio capture, the
music simulation, `top_screen`, `snapshot`); the rest is wired to this
game (`flow()`, `battle()`, `map_scene()`, `cursor_tile()`, the test
chapter, `harness.rs:20-28,542`). `docs/engine/boundary.md`, coupling 10.

## Nick input

None.

## Scope

**In:**
- `shuyi_ui::harness::Harness<G, A>` wrapping `shuyi_ui::Game<G, A>`
  with the generic API, behind a `harness` feature (as today, ADR-0017).
- `trpg-ui`'s `Harness` becomes a thin wrapper adding `flow()`,
  `battle()`, `map_scene()`, `cursor_tile()` and the test chapter start.
- The engine's E-class tests in `ui/tests/it` that only need the generic
  Harness (key bindings, controller, rebind buttons, layout picker,
  sprite test; `docs/engine/boundary.md`, *ui*) are rewritten against a
  toy game and move; the game keeps a copy only where it tests its own
  screens.
- `frame-png`'s use of the Harness goes through the generic part.

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6).

**Out (do not do):** the CLI contract (1129); MCP (1133).

## Implementation steps

1. Split `harness.rs`; wrap.
2. Write the toy game used by engine tests in
   `engine/crates/ui/tests/toy/` (two screens, a menu, a sprite).
3. Move the tests that fit.

## Acceptance criteria

- [ ] `cargo test -p shuyi-ui` runs Harness tests against the toy game.
- [ ] Every game Harness test passes unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the toy game's Harness tests.

## Completion notes
