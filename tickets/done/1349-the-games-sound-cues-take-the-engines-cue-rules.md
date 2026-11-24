---
id: "1349"
title: "The game's sound cues take the engine's cue rules where a repeat is a fault: no stacked cursor ticks, no same footstep twice"
type: feature
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-23
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

- [x] A test: a held cursor key plays one tick per step it moves, with
      no stack, and `dropped_sounds` lists the rest.
- [x] A test: two footsteps in a row never play the same file.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the Harness tests above, in `crates/ui/tests/it/`.

## Completion notes

- **Which cues stack** (searched in `crates/ui`): the map cursor's tick
  (`cursor_tick`, one per tile moved, so two arrows in one frame or a late
  frame owed several held-key repeats ticks several times at once), the
  menus' `menu_move` (the same through `AudioQueue::menu`), and the step
  sounds (`effects.rs` plays one per tile entered in a frame, and a
  hurried walk enters a tile about every 21 ms). The hit, cast and heal
  cues can't: a cue's next strike is at least a flash, a result and a gap
  later (1.05 s, about 260 ms at ×4 hurry), so they got no rule.
- **The gap: `min_gap_ms: 30`** on `cursor_move`, `menu_move`,
  `step_foot`, `step_armored` and `step_mounted`, in both manifests
  (`assets/audio/audio.ron` and the private repository's
  `content/audio/audio.ron`). Measured in the Harness: a held arrow
  (repeat every 55 ms, `keymap.ron`) ticks every three or four frames
  (50 or 67 ms), and two taps in the Harness are two frames (33 ms) apart.
  30 drops a second play in the same frame or the next one at 60 frames a
  second, and stays under every held repeat and under two frames at 30
  frames a second, so a tile the cursor moves on its own frame always
  ticks.
- **`pick: NoRepeat`** on `step_foot`, the one cue with several files.
- **Tests** (`crates/ui/tests/it/sound_cues.rs`): before the rules, two
  arrows in one frame played two ticks and nothing was dropped, and the
  enemy phase's footsteps played files `0, 0, 1, 0, 2, 2, …`. After: a
  held key ticks once per tile with no drop, a two-arrow frame and a late
  frame tick once with the rest in `dropped_sounds`, and fourteen
  footsteps never repeat a file.
- **Session record**: not blessed; it did not move (its hashes hold the
  sounds asked for, which the rules don't change).
- `assets/audio/README.md` names the new fields.
- *For the Act 1 playtest (1064), not a rule decided:* a walk hurried
  with Confirm held plays fewer footsteps (one per ~33 ms instead of
  nearly every frame). `audio.md` says each step picks one of three "at
  random"; NoRepeat still does, among the two the last step didn't play.
- No follow-up tickets.
