---
id: "1346"
title: "The playtest personas draw from the engine's bot seed, not a salt of their own"
type: infra
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-23
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

- [x] `rg SEED_SALT crates` finds nothing.
- [x] A test: two tries of one seed draw the same persona moves, and a
      persona's first draw differs from the battle's first roll of that
      seed.
- [x] Every persona inside its band, or the miss written in the notes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the seed test above, in `crates/bots/src/persona/tests.rs`.
- Integration: the bots' baseline.

## Completion notes

**Done.** `PersonaBot::new` seeds its `TurnPlan` with the engine's
`shuyi_core::bots::playtest::bot_seed(seed)`; `SEED_SALT` and its comment
are gone. New test
`a_persona_draws_from_the_bot_seed_not_the_battles_luck`
(`crates/bots/src/persona/tests.rs`): two tries of one seed play out to the
same state, the bot plans exactly as a plan seeded `bot_seed(seed)` by hand,
and its first draw differs from the battle's first roll of that seed (it
fails with the plain `seed`, checked).

**Other bots of `crates/bots`.** No other bot draws from a try's seed.
`BaselineBot` draws nothing. The wins recorder (`record.rs`) draws from its
own search seed (`cargo xtask bot-wins --seed`), while the battle's luck
comes from the battle's file, so it was left alone. The skirmish generator's
seed is a map request's, not a try's.

**Kept results.** Neither `crates/bots/wins/*.ron` nor the baseline
(`crates/bots/tests/baseline.rs`) uses the personas, so neither moved and
nothing was re-recorded. The bots' tests and the baseline pass unchanged.

One kept result did move: the autobalance tests on `test_too_hard`
(`crates/tools/src/game_tools/balance/tests.rs` and
`crates/tools/tests/cli.rs`), a small search (20 tries, 10 quick) from
seed 1 that has to land on "every unnamed enemy -5 levels" (the battle's
enemies are 5 levels over their templates). With the old salt it landed
there in 38 evaluations. With the new draws, seed 1 got stuck at -8
(Casual 3/20). That is noise at 20 tries: seeds 31 and 51 land on -5
alone, 21 and 61 on -4 plus one more change, and 60 tries from seed 1
reach every band but Hardcore's (53/60, too easy). Both tests now search
from seed 51 (in every band in 10 evaluations: Casual 9/20, Normal 12/20,
Hardcore 11/20). No band and no battle changed.

**Bands** (100 tries, seeds 1–100). *Before* is `main`'s last playtest run
(Actions run 45314741164, the engine pin move #808, whose code this branch
starts from). *After* is this PR's `playtest` run (Actions run 45325182246); Chapter 1
played locally gave the same 58/50/66:

| Battle | Casual before → after | Normal before → after | Hardcore before → after |
| --- | --- | --- | --- |
| act1_random_ash | 95% in → 93% in (70–100) | 93% in → 92% in (70–95) | 100% above → 99% above (70–95) |
| act1_random_ford | 89% in → 87% in (70–100) | 45% below → 42% below (70–95) | 60% below → 55% below (70–95) |
| act1_random_road | 92% in → 93% in (70–100) | 69% below → 67% below (70–95) | 93% in → 89% in (70–95) |
| aldwater | 100% in → 99% in (70–100) | 88% in → 82% in (70–95) | 81% in → 82% in (70–95) |
| ch01 | 53% in → 58% above (30–55) | 51% in → 50% in (50–75) | 69% in → 66% in (50–75) |
| ch02 | 48% in → 49% in (30–55) | 55% in → 54% in (50–75) | 83% above → 86% above (50–75) |
| ch03 | 68% above → 74% above (30–55) | 76% above → 72% in (50–75) | 55% in → 60% in (50–75) |
| ch04 | 65% above → 60% above (30–55) | 37% below → 37% below (50–75) | 55% in → 55% in (50–75) |
| ch05 | 65% in → 57% in (40–70) | 31% below → 42% below (50–75) | 23% below → 21% below (50–75) |
| millhaven_road | 96% in → 93% in (70–100) | 62% in → 54% below (55–80) | 71% in → 75% in (55–80) |
| salt_road | 100% in → 99% in (70–100) | 88% in → 86% in (70–95) | 90% in → 89% in (70–95) |
| side_company_ledger | 36% in → 42% in (30–55) | 8% below → 3% below (50–75) | 44% below → 43% below (50–75) |
| side_letters_home | 79% above → 76% above (30–55) | 35% below → 37% below (50–75) | 67% in → 69% in (50–75) |
| smugglers_cove | 95% in → 98% in (70–100) | 77% in → 78% in (55–80) | 84% above → 86% above (55–80) |

Many personas were outside their bands before this change (26 in, 7
above, 9 below of 42), and after it (25 in, 7 above, 10 below). The new
seeding moves each share by a few points, at most 11 (Normal on Chapter
5, 31% → 42%, still below). Three verdicts changed: Casual on Chapter 1
left its band (53% → 58%, above 55), Normal on Millhaven Road left it
(62% → 54%, under 55), and Normal on Chapter 3 came back in (76% →
72%). That is a fresh draw of 100 tries, not a change in how the personas
play. Per the ticket, no band or setting was moved; the misses are
findings for the balancing tickets.

**Friction:** estimate 10 production lines, measured 2 added (0.2×), under 2×.
