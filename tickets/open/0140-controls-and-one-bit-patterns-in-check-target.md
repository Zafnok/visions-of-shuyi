---
id: "0140"
title: "Controls and one-bit patterns in check --target"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0138"]
nick_input: none
completed:
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

- [ ] `cargo shuyi check --target all` reports neither limit.
- [ ] Every problem it finds is fixed or has a ticket.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the project wiring, in `crates/xtask/src/game_tools/tests.rs`.

## Completion notes

*(Filled in by the session that completes the ticket.)*
