---
id: "0139"
title: "Whole-number AI scores, then the rules fence's float ban on trpg-core"
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0138"]
nick_input: none
completed: 2026-11-01
---

# 0139 — Whole-number AI scores and the fence's float ban

## Context

0138 gave `trpg-core` the engine template's fence in `crates/core/clippy.toml`,
except its `f32` and `f64` bans and the `clippy::float_arithmetic` deny:
`crates/core/src/ai.rs` scores its options in `f64` (odds, terrain, weights).
ADR-0004 asks the rules to be deterministic, and floats can come out
differently on another machine or on the web, so a record could replay
differently there. The Playdate and web builds run this same code.

## Nick input

None, as long as the AI picks the same moves. If whole-number scores can't
keep them, that is a tuning change for the bots to measure, not a redesign.

## Scope

**In:**
- The AI's scores and odds in whole numbers (`crates/core/src/ai.rs`).
- `f32` and `f64` in `crates/core/clippy.toml`, and
  `#![cfg_attr(not(test), deny(clippy::float_arithmetic))]` in `lib.rs`.
- Tests that use a float mark it with `#[expect]`, as `shuyi-gates` says.

**Out (do not do):**
- New AI behavior or weights.

## Implementation steps

1. Read `shuyi-gates` (*The rules crate's fence*) and the AI's scoring.
2. Choose a fixed-point scale; move the scores to it.
3. Compare the AI's choices before and after with the bots and the AI's tests.
4. Add the bans and the deny; `cargo shuyi lint` stays clean.

Estimate: about 150 production lines changed, 400 total.

## Acceptance criteria

- [x] `crates/core/clippy.toml` bans `f32` and `f64`, and `lib.rs` denies
      float arithmetic outside tests.
- [x] `cargo shuyi lint --json` has no findings.
- [x] Any change in the AI's choices is measured and named in the PR.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the existing AI tests, and exact scores at the rounding edges.
- Bots: a playtest before and after.

## Completion notes

The AI's chances and scores in `crates/core/src/ai.rs` are whole numbers, and
`trpg-core` bans `f32` and `f64` and denies float arithmetic outside tests. A
temporary float in the crate fails `cargo shuyi lint` with `SH2103`. The
progression tests' averages keep their floats, marked with `#[expect]`.

**The scale:** a chance of 1 is 100^8, so every chance of a combat of up to 8
strikes (4 a side, the rules' limit) is exact, and scores are `i128`, which
holds any weights and HP. Nothing rounds in a real combat. A longer one rounds
a hit chance down and gives the rest to the miss.

**The AI's choices did change, in one case only: an exact tie.** Unhorse on a
10-damage strike gains exactly what its durability costs (+10 hit is 1 HP, at
10 a point, against 2 durability at 5). The documented tie rule gives that to
the plain attack, but float rounding put the art ahead by a hair. Now the plain
attack wins, as the module docs always said.

Measured, same seeds before and after:
- Every command of 500 seeded AI-against-AI battles (62,468 commands): 490
  battles identical. The other 10 first part at this tie: all 10 partings are
  Unhorse against the plain attack, and logging showed equal scores for both.
- Playtest bots, 100 tries each on `ch01`, `ch02` and `quick`: all baseline
  tries identical, and 3 of 900 persona tries differ (2 on `ch01`, 1 on `ch02`).
  The one change in a win count: `ch01` hardcore 69 to 68, still in its band.
  Those 3 tries weren't traced back to the tie one by one.
- `quick`'s persona results are outside their bands as they were before
  (casual 91, normal 46, hardcore 45): unchanged by this ticket.

Not measured: speed on the Playdate. Desktop timings weren't a clean comparison
(other programs were running on the machine).

**Beyond the ticket's steps:** `the_rules_crate_has_the_fences_bans` (0138) also
checks the float bans and the deny, so they can't quietly go missing.

Gameplay rules decided: none. Follow-up tickets: none.
