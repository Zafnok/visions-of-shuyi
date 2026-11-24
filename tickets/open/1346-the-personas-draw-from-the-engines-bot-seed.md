---
id: "1346"
title: "The playtest personas draw from the engine's bot seed, not a salt of their own"
type: infra
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1346 — The personas draw from the engine's bot seed

## Context

The engine pin move of 2026-11-23 passed engine PR #523 (engine ticket
2133, the bots' data made alike, engine ADR-0135). It adds
`shuyi_core::bots::playtest::bot_seed(seed)`: the seed a try's bots draw
from, two `next_u32` of `SimRng::new(seed)`, so the bots' stream is never
the one the stage's luck is drawn from.

The game has its own copy of that job: `crates/bots/src/persona.rs`'s
`SEED_SALT` (`0x5EED_0506_B075_0000`), XORed into the try's seed in
`PersonaBot::new` (`seed ^ SEED_SALT`) for the persona's `TurnPlan`. A
XOR with a constant keeps the two streams apart only as long as nothing
else draws from a seed near the salt; the engine's way is the one its
template and Hello, Grid use.

## Nick input

None.

## Scope

**In:**
- `PersonaBot::new` seeds its plan with `bot_seed(seed)`; `SEED_SALT`
  and its comment go (about 5 lines removed).
- Any other bot of `crates/bots` that draws randomness from the try's
  seed (search `seed` in `crates/bots/src`) does the same.
- The bots' kept results move with the new draws: re-record
  `crates/bots/wins/*.ron` and the baseline (`crates/bots/tests/baseline.rs`)
  as their docs say, and check each persona is still inside its band in
  `crates/bots/targets.ron`. A persona that leaves its band is a finding
  for the completion notes, not a band to move.

**Out (do not do):**
- Changing a band, a persona's settings or the enemy AI.
- The engine's `schema` feature for the persona files (no editor reads
  them yet).

Estimate: about 10 production lines changed; the kept results are data.

## Implementation steps

1. Replace the salt with `shuyi_core::bots::playtest::bot_seed` in
   `crates/bots/src/persona.rs`.
2. Run the bots' tests; re-record what moved, with the counts before and
   after in the completion notes.

## Acceptance criteria

- [ ] `rg SEED_SALT crates` finds nothing.
- [ ] A test: two tries of one seed draw the same persona moves, and a
      persona's first draw differs from the battle's first roll of that
      seed.
- [ ] Every persona inside its band, or the miss written in the notes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the seed test above, in `crates/bots/src/persona/tests.rs`.
- Integration: the bots' baseline.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
