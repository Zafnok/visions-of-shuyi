---
id: "1413"
title: "The conversation's log, auto-advance and voices ahead on the engine's: History, Clip, auto_advance and coming_lines go"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3252"]
nick_input: none
completed:
---

# 1413 — The conversation's log, auto-advance and voices ahead on the engine's

## Context

The third engine-friction sweep (1397, `docs/engine/friction.md`)
found that the conversation screen keeps three generic pieces on top
of the engine's `DialoguePlayer`. They come to 135 production lines,
for about 25 of the game's own (5.4×):
- **the log's memory**: `screens/dialogue/history.rs`, a jump back that
  asks a later reply again (0729);
- **auto-advance** (0859, 0863): `Clip`, `auto_advance` and their
  constants in `screens/dialogue.rs`;
- **the lines that can come next**, whose clips are kept loaded:
  `coming_lines`, ADR-0046.

Engine ticket 3252 (`Zafnok/shuyi-engine`) builds them as
`dialogue::backlog`, `dialogue::auto` and
`DialoguePlayer::coming_lines`. The marked items carry
`// engine-friction: engine#3252, moves (game 1413 deletes it)`.

## Nick input

None. Auto-advance's times and the log's rules stay as they are.

## Scope

**In:**
- Move the pin past engine 3252, if it isn't already (`engine-change`
  skill §7).
- `History` becomes `backlog::Backlog<DialoguePlayer>`, keeping `Seen`'s
  fields: the game's `DialoguePlayer` newtype is the `P`.
- `Clip` and `auto_advance` go onto `auto::VoiceClip` and
  `auto::AutoAdvance`, built from `AutoTimes` holding today's four
  tunables.
- `coming_lines` calls the engine's, through the newtype, with
  `VOICES_AHEAD`.
- What stays the game's:
  - `is_voiced` (its `RowKind` rule);
  - the log's view and skins, with the lead's portrait.

**Out (do not do):**
- the log screen's look or keys;
- the auto-advance setting row (it is the game's option).

## Implementation steps

1. Write the estimate: about 120 lines removed, 30 added.
2. Move the pin; replace the marked items; keep
   `screens/dialogue/tests.rs` and `dialogue/history/tests.rs` green.
   History tests the engine now covers may go: say which.

## Acceptance criteria

- [ ] `grep -rn "engine#3252" crates/ui/src` finds nothing.
- [ ] The dialogue screen's auto-advance and log tests pass unchanged.
- [ ] The engine-friction line is in the PR body.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing dialogue tests, unchanged in what they check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
