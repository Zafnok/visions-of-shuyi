---
id: "0536"
title: "A newcomer bot that plays the mode easier than Casual"
type: feature
milestone: M4 Enemy AI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["0098", "0880"]
nick_input: answer-first
completed:
---

# 0536 — A newcomer bot that plays the mode easier than Casual

## Context

Nick (2026-11-22): a mode below Casual "would need a new bot profile to
pair". Personas are in `crates/bots/personas.ron` and
`crates/bots/src/persona.rs`; their bands per tier in
`crates/bots/targets.ron` (`Targets`, `crates/bots/src/playtest.rs`);
the design is `docs/design/playtest-bots.md` (personas around line 71,
bands around line 90). Bots never rewind (memory: automated playtesting).
0098 sets the new bot's band; 0880 builds the mode.

## Nick input

**Answer first:** 0098 (the band). Verified by the bot runs.

## Scope

**In:** a `newcomer` persona that plays like someone new to games
(e.g. a smaller thinking budget, more mistakes: walking into enemy
ranges, attacking without checking the forecast, sometimes ignoring the
objective), playing 0880's mode; its band in `targets.ron`; its row in
`playtest-bots.md`; Chapter 1's run in `ch01_difficulty.rs`; CI's bot
job includes it (0530: one battle per job).

**Out (do not do):** retuning Chapter 1 to meet the band (file a tuning
ticket if it misses); human-play calibration (0508).

## Implementation steps

1. Persona data and any new behaviour knobs in `crates/bots`.
2. Its band; the Chapter 1 test; the docs row.
3. Run `cargo shuyi playtest` on Chapter 1 and put the result in the PR.

## Acceptance criteria

- [ ] The newcomer persona loads and plays Chapter 1 in 0880's mode (a
      test).
- [ ] Its Chapter 1 win rate and interval are in the PR; a miss files a
      tuning ticket.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The bots' persona and band tests.

## Lesser targets

None (bots are dev tooling, never in the game).

## Completion notes

