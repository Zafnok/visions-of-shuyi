---
id: "1349"
title: "The game's sound cues take the engine's cue rules where a repeat is a fault: no stacked cursor ticks, no same footstep twice"
type: feature
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1349 — The game's sound cues take the engine's cue rules

## Context

The engine pin move of 2026-11-23 passed:

- engine PR #536 (engine ticket 2345, engine ADR-0139): cue rules as
  data in the audio manifest, decided once in `App` for every backend.
  A sound's entry reads `pick` (`Random`, the default, `NoRepeat`,
  `Shuffle`, `InOrder`), `volume_min` and `min_gap_ms`. A play of a cue
  sooner than its gap after the last one that sounded is dropped, and
  `App::dropped_sounds` and `Harness::dropped_sounds` list it. Which file
  of a cue plays is now picked from `ctx.music_seed`, the same on every
  backend.
- engine PR #542 (engine ticket 3419): `cargo shuyi run`'s answer (and
  the MCP server's `step`) lists the plays the rules dropped.

Hello, Grid's cursor cues got `min_gap_ms: 40`, "so a cursor that moves
several cells in one frame ticks once". The game's `assets/audio/audio.ron`
has no rule on any cue: `cursor_move` and `menu_move` play once per step
(a held key's repeats included), and `step_foot`'s three files are picked
at random, so the same footstep can play twice in a row.

## Nick input

None for the rules that remove a fault the player hears (stacked ticks,
the same file twice in a row). A rule that changes a sound's feel
(`volume_min`, a `Shuffle` of a voice) is out: that is a decision for
Nick, with the sounds to hear (`ask-nick`), in a ticket of its own.

## Scope

**In:**
- Find the cues that can stack in one frame or a few (search where
  `cursor_move`, `menu_move`, `step_foot` and the battle's hit cues are
  requested in `crates/ui`), with a Harness test that shows the stack
  today (`Harness::sounds`).
- `min_gap_ms` on those cues, the smallest gap that plays one tick per
  frame of a held key's repeat (measure with the Harness at the game's
  repeat rate).
- `pick: NoRepeat` on each cue with several files.
- Tests read `Harness::dropped_sounds`; the kept Quick Battle session is
  blessed again if its sounds move.

**Out (do not do):**
- A new sound, a volume, or `volume_min`.
- The Playdate's own audio (`crates/playdate/src/audio.rs`): it plays
  what the requests say.

Estimate: data in `assets/audio/audio.ron` and the content's manifest
(`content-change`), and about 60 lines of tests.

## Implementation steps

1. The Harness test of a held cursor key's ticks and of three footsteps.
2. Add the rules; run the test again: one tick per step, no file twice.
3. Bless the session record if it moved, naming engine PR #536 as why.

## Acceptance criteria

- [ ] A test: a held cursor key plays one tick per step it moves, with
      no stack, and `dropped_sounds` lists the rest.
- [ ] A test: two footsteps in a row never play the same file.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the Harness tests above, in `crates/ui/tests/it/`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
