---
id: "1168"
title: "The options rows on the engine's enum setting rows: the Named trait and the test-only row copies go"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: ["engine#2314"]
nick_input: none
completed:
---

# 1168 — The options rows on the engine's enum rows

## Context

The first engine-friction sweep (1153, `docs/engine/friction.md`)
measured the game's options screen (`crates/ui/src/screens/options.rs`
and its one-bit skin) at 519 production lines against an estimate of
300 (1.7×):

- the `Named` trait, its five impls and `choice()` (84 lines) exist
  because the engine's `RowValue::Choice` takes plain function pointers
  by index. Engine ticket 2314 (`Zafnok/shuyi-engine`) adds a
  `ChoiceSetting` trait and `OptionRow::choice::<T>`;
- `Row::is_setting` repeats the engine's `OptionRow::is_setting`, and
  it and `Row::from_key` are used only by tests; the restore-defaults
  question repeats the engine's literal text key. That part needs no
  engine change.

## Nick input

None.

## Scope

**In:**
- Move the pin past 2314 (`engine-change` skill, step 7).
- The enum rows on `OptionRow::choice`; `Named` and `choice()` go.
- The test-only helpers go (tests use the engine's) and the repeated
  text key uses the engine's constant if it has one.

**Out (do not do):**
- Which rows show on which platform and campaign (`shown_rows`): real
  logic.
- Any change the player sees.

## Implementation steps

1. Write the estimate: about 90 lines removed, 15 added.
2. Move the pin; the rows; the helpers.

## Acceptance criteria

- [ ] `grep -n "trait Named" crates/ui/src/screens/options.rs` finds
      nothing.
- [ ] The options tests and snapshots pass unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing tests, unchanged in what they check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
