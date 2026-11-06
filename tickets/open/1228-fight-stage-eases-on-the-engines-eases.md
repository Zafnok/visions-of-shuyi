---
id: "1228"
title: "The fight stage's eases and the AI camera's pan on the engine's ease curves and Lerp"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: blocked
blocked_by: ["engine#2109"]
nick_input: none
completed:
---

# 1228 — The fight stage's eases on the engine's eases

## Context

Found by the game/engine boundary audit of 2026-11-03. The game writes
its own curves where the engine will have named ones:

- `crates/ui/src/screens/battle/stage/motion.rs`: private `ease_out`
  (lines 66 to 73, `1 - (1 - t)^2`) and `ease_in` (76 to 83, `t^2`),
  each with its own finite-and-clamp guard. `run_up` and `go_back` use
  them.
- `crates/ui/src/screens/battle/ai_phase.rs`: `AiAction::camera` (250
  to 267) mixes the camera's origin between two tiles by a hand lerp
  with `f32::round`.

Engine 2109
([link](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/2109-ease-curves-and-tweens-drawn-from-the-time-alone.md))
adds `shuyi_core::ease` (`Ease`, `Family`, `Lerp`, `ONE`) and
`shuyi_ui::ease::ease_f32`, which clamps its input and counts NaN as 0,
as the game's guards do. 2109's Context names this ticket.

## Nick input

None. The tests below are the check; a pixel that moves in a fight
picture is listed in the PR, not judged by Nick.

## Scope

**In:**
- `ease_out(t)` becomes `ease_f32(&Ease::Out(Family::Quad), t)` and
  `ease_in(t)` becomes `ease_f32(&Ease::In(Family::Quad), t)` at their
  call sites in `run_up` and `go_back`; the two private functions go.
- `AiAction::camera` takes its part `k` into 1/65536 (`ONE`) and mixes
  the origin with `Lerp::mix` over `(i32, i32)`; the hand lerp and its
  `allow` go.

**Out (do not do):**
- The engine's own motions (cursor pulse, music fades, `Pan`): engine
  2136.
- A camera blend in place of the pan (engine 2362, parked).
- `struck`'s dodge curve (`4p(1 - p)`) and shake: no named ease matches
  them.
- The walk functions of `ai_phase.rs`: 1217.

**Either order with 1217:** both edit `ai_phase.rs`, in different
functions. If 1217 is done, keep its walk helpers; if not, nothing to
do (1217 leaves `camera` to this ticket).

## Implementation steps

1. ~~Move the pin past engine 2109's merge (`engine-change` §7, the
   `engine-pin` skill), unless an open `engine-pin` PR has done so.~~
   (done by the pin move of 2026-11-05).
2. Save `cargo shuyi frame-png` pictures of a fight's run-up, a
   critical's wind-up and the striker going back, before the change.
3. Replace the two eases in `motion.rs`.
4. Replace the lerp in `AiAction::camera`. `Lerp::mix` rounds halves
   up, `f32::round` away from zero: the two can differ by one tile only
   at an exact half on a pan that goes up or left.
5. Take the same pictures again and compare. Expect about 18 production
   lines removed and 4 added (`cargo xtask code-lines`).

## Acceptance criteria

- [ ] `cargo xtask ticket-lint --check-engine` passes with the pin past
      engine 2109.
- [ ] `motion.rs` has no `fn ease_in` or `fn ease_out`, and
      `AiAction::camera` no hand lerp (`grep`).
- [ ] `motion.rs`'s tests pass unchanged (their tolerance, `1e-4`, is
      above `ease_f32`'s 2/65536), NaN cases included.
- [ ] `ai_phase.rs`'s pan tests (`pans_then_highlights_then_walks` and
      the others reading `camera()`) pass with their expected tiles; a
      changed one is only an exact half on an up or left pan, named in
      the completion notes.
- [ ] Every fight-stage snapshot passes, or the completion notes list
      each one moved (at most a pixel) with the before and after
      pictures.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the existing `motion.rs` and `ai_phase.rs` tests.
- Snapshot: the fight stage's, plus the `frame-png` comparison above.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
