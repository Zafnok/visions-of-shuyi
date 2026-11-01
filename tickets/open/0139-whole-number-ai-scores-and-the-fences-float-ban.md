---
id: "0139"
title: "Whole-number AI scores, then the rules fence's float ban on trpg-core"
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0138"]
nick_input: none
completed:
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

- [ ] `crates/core/clippy.toml` bans `f32` and `f64`, and `lib.rs` denies
      float arithmetic outside tests.
- [ ] `cargo shuyi lint --json` has no findings.
- [ ] Any change in the AI's choices is measured and named in the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the existing AI tests, and exact scores at the rounding edges.
- Bots: a playtest before and after.

## Completion notes

*(Filled in by the session that completes the ticket.)*
