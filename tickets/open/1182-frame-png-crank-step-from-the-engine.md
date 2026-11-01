---
id: "1182"
title: "The game's frame-png winds the crank with the engine's `--crank` step: delete the game's own `--crank` and `--then-pad`"
type: infra
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: ["engine#2540"]
nick_input: none
completed:
---

# 1182 — frame-png's crank step from the engine

## Context

Ticket 1039 needed a picture of the Playdate's rewind screen, which only
the crank opens. The engine's `frame-png` script (`shuyi_cli::frame_png`)
had `--keys`, `--pad` and `--wait` but no crank step, so the game added two
options of its own to its `GameTools` in `crates/xtask/src/game_tools.rs`:
`--crank DEGREES` and `--then-pad BUTTONS`, played after the script's steps
(the `After` steps of `Start`), marked `// engine-friction: engine#2540`.
Engine ticket 2540 adds `--crank` as a script step, in order with the
others. The friction row is 1039's in `docs/engine/friction.md`.

The pin is already past engine 2540 (game PR #482). This is ready; the
remaining work is deleting the game-specific after-script path. Preserve
the engine dependency as the minimum revision requirement.

## Nick input

None. Tooling only; nothing the player sees changes.

## Scope

**In:**
- ~~Move the engine pin past engine#2540 (`cargo xtask engine-pin`,
  `engine-change` skill step 7), with any small fixes the newer engine
  needs.~~ (done by the pin move of 2026-10-29, `c9fb84c9` → `50ae33db`,
  which also deleted the game's `--crank`: the engine's parser takes the
  flag first, so the game's could no longer be reached)
- Delete the game's `--then-pad` option, `After`, and their handling in
  `crates/xtask/src/game_tools.rs`; the `frame-png` test that used it
  uses the engine's steps (`--crank -30 --pad East`).
- Update the commands for 1039's pictures if any doc names
  `--then-pad`.

**Out (do not do):**
- Any change to the pictures or the game.

**Either order with 1170:** the pin move of 2026-11-01 moved the Playdate's
A and B to East and West (engine 2410), and the examples below use them.
Whichever lands second proves the crank/pad sequence still reaches rewind
and its help names agree.

## Implementation steps

1. ~~Move the pin (`cargo xtask engine-pin <rev>`), build, fix what the
   newer engine needs.~~ (done by the pin move of 2026-10-29, `c9fb84c9`
   → `50ae33db`)
2. Remove `OPTIONS`' `--then-pad` entry (the array back to 6),
   `Start::after`, `enum After` and the loop after `script.play`.
3. Rewrite `the_crank_and_buttons_after_the_steps_open_the_rewind_screen`
   in `crates/xtask/src/game_tools/tests.rs` on the engine's step (a
   script with `--crank -30` shows `rewind`).

## Acceptance criteria

- [ ] `cargo shuyi frame-png r.png --playdate --pad "DpadDown East" --wait 0.1 --pad "DpadLeft East" --wait 0.1 --pad East --crank -30`
      shows the rewind screen.
- [ ] No `engine-friction: engine#2540`, `--then-pad`, `Start::after` or
      `enum After` remains in the implementation or current commands.
      Search the behavior and every spelling across `crates/` and current
      docs after merging main, not only the marker. Historical done
      tickets may retain their original descriptions.
- [ ] Ordered engine steps `--crank -30 --pad East` reach the same
      rewind state and picture as the old after-script fixture; the
      removed `--then-pad` option is rejected instead of silently ignored.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit/integration: the rewritten frame-png test, ordered crank/pad
  steps and rejection of the removed option.

Expected size: about 30 production lines removed and a small test rewrite;
under 500 changed lines including the ticket move.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
