---
id: "0140"
title: "Controls and one-bit patterns in check --target"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0138"]
nick_input: none
completed: 2026-11-01
---

# 0140 — Controls and one-bit patterns in check --target

## Context

0138 lists the game's targets, so `cargo shuyi check --target all` checks the
frame and the text on each. It still reports two limits: `the controls aren't
checked` (`GameTools::controls`) and `playdate's one-bit patterns aren't
checked` (`GameTools::one_bit`). The game's `Trpg` tools offer neither, and no
screen declares its actions (`Screen::actions`) yet.

## Nick input

None, unless the check finds a screen whose required actions don't fit a
target. A redesign of that screen's controls is then Nick's: file a `00xx`.

## Scope

**In:**
- `Screen::actions` on the game's screens, split into more tickets if it won't
  fit one PR.
- `Trpg::controls` and `Trpg::one_bit` in `crates/xtask/src/game_tools.rs`.
- A test that `check --target all` no longer reports those two limits.

**Out (do not do):**
- Changing any control or default key (`keyboard-input`).
- The identity's limit, which is 1199's.

## Implementation steps

1. Read `shuyi-input`, `shuyi-screen` and the engine's `shuyi_ui::input::fit`.
2. Offer `one_bit` from the Playdate's palette and pattern files.
3. Declare each screen's actions; offer `controls`.
4. Read the problems; fix wiring errors, and file design questions.

Estimate: about 300 production lines, 800 total.

## Acceptance criteria

- [x] `cargo shuyi check --target all` reports neither limit.
- [x] Every problem it finds is fixed or has a ticket.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the project wiring, in `crates/xtask/src/game_tools/tests.rs`.

## Completion notes

- Every screen declares its actions from one table,
  `crates/ui/src/screens/actions.rs`, whose `every_screen` is what
  `Trpg::controls` checks. A new screen must be added there by hand: nothing
  fails if it is left off.
- Title and Game Over declare every action, because any press skips them.
- The check found one real problem: the engine's Playdate profile has A at
  `East`, B at `West` and one chord, while the game's front end reports A at
  `South` and B at `East` and uses three chords. No control changed. The game
  now has its own profile, `targets/playdate.ron`, naming what its front end
  (ADR-0070) really does; a copy of the engine's budgets comes with it.
- Deviation: the game has no pattern file, because it draws with inks
  (ADR-0074). `Trpg::one_bit` derives the map from the palette's inks
  (`crates/xtask/src/game_tools/one_bit.rs`). The map's `text` stays empty,
  since an ink has one pattern for ground and glyph; a test keeps the text
  roles solid black instead.
- The declared lists come from reading the code. The Harness now filters by
  them and the UI tests pass, but only paths the tests play are proven.
- No design questions, no gameplay rules decided, nothing to judge by playing.

