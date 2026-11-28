---
id: "1414"
title: "Sound beds, camp and town music, and the battle's queued cues on the engine's SoundBed, Rotation and CueQueue"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3253"]
nick_input: none
completed:
---

# 1414 — Sound beds, music rotation and queued cues on the engine's

## Context

The third engine-friction sweep (1397, `docs/engine/friction.md`) found
three generic sound schedules in the game. Together they are about 228
production lines, for about 40 of the game's own (5.7×):
- **the shared sound bed**: `crates/ui/src/ambience.rs`, ticket 1332;
  called from the battle, camp, dialogue and world map screens;
- **the rotation of the camp pool's tracks**, with a quiet gap:
  `screens/camp/music.rs` (0027), plus the pool setups in
  `screens/camp.rs` and `screens/world_map.rs` (`town_music`);
- **sound cues due after a delay**: `CueQueue` in
  `screens/battle/event_sounds.rs`.

Engine ticket 3253 (`Zafnok/shuyi-engine`) builds them as
`audio::SoundBed`, `audio::Rotation` and `audio::CueQueue`. The marked
items carry `// engine-friction: engine#3253, moves (game 1414 deletes it)`.

## Nick input

None. Every fade, gap and delay keeps its value
(`docs/design/audio.md`, *Starting values*).

## Scope

**In:**
- Move the pin past engine 3253, if it isn't already (`engine-change`
  skill §7).
- `AmbiencePlayer`, `play` and `leave` go onto `SoundBed`, kept in
  `Shuyi::ambience`. `bed()` (which bed goes with which ambience and
  light) and the `AMBIENCE` volume check stay.
- `CampMusic`, `MusicCue` and `ask` go onto `Rotation`. The two pool
  setups become one `Rotation::from_pool` call each, with `QUIET_S`,
  `FADE_IN_MS` and `FADE_OUT_MS` as its `RotationTimes`.
- `CueQueue` becomes the engine's. `event_sounds`' cue table (which
  event makes which cue) stays.

**Out (do not do):**
- any change in what is heard, or when;
- the title's song clock, which follows the music
  (`screens/title/song.rs`; another sweep row).

## Implementation steps

1. Write the estimate: about 190 lines removed, 35 added.
2. Move the pin; replace each marked piece with its tests green:
   - `ambience/tests.rs`;
   - `camp/music.rs`'s tests;
   - `event_sounds.rs`' tests.

   Tests the engine now covers may go: say which.

## Acceptance criteria

- [ ] `grep -rn "engine#3253" crates/ui/src` finds nothing.
- [ ] The camp, world map, battle and dialogue tests that read
      `AudioQueue` requests pass unchanged.
- [ ] The engine-friction line is in the PR body.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing audio-request tests of those screens, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
