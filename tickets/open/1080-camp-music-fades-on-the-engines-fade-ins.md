---
id: "1080"
title: Camp and world-map music use the specified fades and quiet intervals
type: feature
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["1003", "1007", "engine#2341", "engine#2398"]
nick_input: none
completed:
---

# 1080 — Camp and world-map music use the specified fades and quiet intervals

## Context

The 2026-11-01 review of the latest 60 PRs in each repository found an
incomplete adoption handoff. Camp (1003,
[game PR 477](https://github.com/Zafnok/visions-of-shuyi/pull/477)) and
the world map (1007,
[game PR 483](https://github.com/Zafnok/visions-of-shuyi/pull/483)) both
use `crates/ui/src/screens/camp/music.rs`'s `CampMusic`. They pass the
audio manifest's 500 ms fade and start the next track at full gain.

[Audio design](../../docs/design/audio.md), *Starting values*, already
specifies 3 s out / 5 s quiet / 3 s in for camp, and 3 s out / 30 s
quiet / 3 s in for the world map. The previous scope covered camp only.

Engine 2341 shipped per-request play switches and is already pinned by
the 2026-10-29 move (`c9fb84c9` to `50ae33db`). It did not add a stop-fade
override: `MusicState::stop` still takes the manifest's default duration.
Engine 2398 supplies that missing operation; its implementation merges
first, then this ticket moves the pin and adopts it. This is a missing
feature found by audit, not a measured engine-friction review.

## Nick input

None. The timings are already decided. Tests inspect requests and music
commands; anything left to judge by playing belongs on the Act 1 playtest
checklist (1064), not a new sign-off.

## Scope

**In:**
- Move the pin past engine 2398 through `engine-change` step 7, or reuse
  the engine-pin PR if it has already moved it.
- Both consumers use the same schedule and shipped audio operations:
  explicit 3 s fade to silence, their own quiet interval, then a 3 s
  fade-in. Count the quiet after the fade finishes, not from its start.
- Preserve camp's initial random choice and pool rotation, and the world
  map's single-track repeat and town music/resume behavior.
- Remove the obsolete engine 2341 workaround and its marker; an ordinary
  music switch elsewhere still uses the existing 500 ms default.

**Out (do not do):**
- New music, cue ids, timing decisions or changes to any other screen.
- Audio device/backend implementation, owned by the engine.
- Replacing the scene tool or Playdate front end.

## Implementation steps

1. Read engine 2398's completion notes and migration information; move the
   pin and settle any matching engine-pin issue through the existing skill.
2. Read `screens/camp/music.rs`, its callers in `screens/camp.rs` and
   `screens/world_map.rs`, and the game's audio wrapper. Compare the
   engine's play/stop requests and `MusicCommand` gains.
3. Add regression cases for both complete cycles before changing the
   requests. Advance a controlled clock through fade, quiet and restart.
4. Update `CampMusic` and both request adapters to use explicit fades;
   adjust the stop schedule so it starts 3 s before the track's end.
5. Test covering/uncovering camp, entering/leaving a town and a request
   interrupting a quiet interval. Keep existing resume rules and cue order.
6. Remove the stale workaround text/marker and record both timing traces
   in completion notes. Expected production change: about 100–180 lines
   across the shared schedule and two existing adapters, plus focused tests;
   planned PR under 2,000 changed lines.

## Acceptance criteria

- [ ] With controlled time, camp commands fade to zero over 3 s, remain
      silent for 5 s, then start the next cue at zero and reach full gain
      over 3 s. No audible overlap replaces the quiet.
- [ ] The world map repeats with 3 s out / 30 s silence / 3 s in; its
      fade duration is not read from the 500 ms manifest default.
- [ ] Town music and return to the map, and camp's cover/resume behavior,
      follow their existing rules; no duplicate start or unwanted restart
      occurs during a quiet interval.
- [ ] An unrelated ordinary music switch retains its 500 ms behavior.
- [ ] No obsolete `engine-friction: engine#2341` marker remains in the
      shared schedule or either consumer.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: shared schedule boundaries, both quiet intervals, interruption and
  resume, including time steps crossing a boundary.
- Integration: harness request/command traces for camp and world map,
  checking gains and exact transition times with a fake clock.
- Regression: the old implementation fails the explicit 3 s fade-out and
  fade-in checks; no speaker or manual timing measurement is needed.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
