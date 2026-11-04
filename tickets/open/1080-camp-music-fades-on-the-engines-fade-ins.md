---
id: "1080"
title: Camp and world-map music use the specified fades and quiet intervals
type: feature
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["1003", "1007"]
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
the 2026-10-29 move (`c9fb84c9` to `50ae33db`):
`AudioQueue::play_music_with(cue, MusicSwitch)` and
`MusicSwitch::new(out_ms, in_ms, overlap)` (engine
`crates/assets/src/audio.rs:90`). Nothing in `crates/` calls them yet.
It did not add a stop-fade override: `MusicState::stop` still takes the
manifest's default duration. Engine 2398
([its ticket](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/2398-stop-music-with-a-fade-chosen-per-request.md))
supplies that missing operation. This is a missing feature found by
audit, not a measured engine-friction review.

The game/engine boundary audit of 2026-11-03 (finding screens-flow#6)
found the marker half stale: `music.rs` lines 11–14 say the engine
"doesn't fade one in", and line 97 still reads
`// engine-friction: engine#2341`, though 2341 is done. So **the
fade-in half starts now**, and engine 2398 is no longer needed first.
Either order with engine#2398:

- **If 2398 is in the engine's `tickets/done/` when this starts:** move
  the pin past it and do both halves here.
- **If not:** ship the fade-ins and both quiet intervals now. The stop
  keeps the manifest's fade, marked `// engine-friction: engine#2398`
  (the only marker left; 1239's pin check reads markers by their engine
  ticket). Then, as `engine-change` §2 says, write the game follow-up
  (`blocked_by: ["engine#2398"]`) that adopts the stop fade, and name it
  in engine 2398's Context in place of `game#1080` (a one-line engine
  ticket PR). Add the follow-up to 1064's `blocked_by` beside 1080, and
  to the lines of `docs/ROADMAP.md` and 1064 that name 1080's music.

## Nick input

None. The timings are already decided. Tests inspect requests and music
commands; anything left to judge by playing belongs on the Act 1 playtest
checklist (1064), not a new sign-off.

## Scope

**In:**
- The fade-ins: every track `CampMusic` starts, in camp and on the
  world map, fades in over 3 s through `play_music_with`.
- Both consumers use the same schedule: a fade to silence, their own
  quiet interval, then a 3 s fade-in. Count the quiet after the fade
  finishes, not from its start.
- With engine 2398 done: move the pin past it through `engine-change`
  step 7 (or reuse the engine-pin PR if it has already moved it), and
  fade out over an explicit 3 s.
- Preserve camp's initial random choice and pool rotation, and the world
  map's single-track repeat and town music/resume behavior.
- Remove the stale engine 2341 text and its marker; an ordinary music
  switch elsewhere still uses the existing 500 ms default.

**Out (do not do):**
- New music, cue ids, timing decisions or changes to any other screen.
- Audio device/backend implementation, owned by the engine.
- Replacing the scene tool or Playdate front end.

## Implementation steps

1. Check engine 2398 on the engine's `main`. If done, read its
   completion notes, move the pin and settle any matching engine-pin
   issue through the existing skill. If not, no pin move is needed.
2. Read `screens/camp/music.rs`, its callers in `screens/camp.rs` and
   `screens/world_map.rs` (`CampMusic::start`, line 642), and the game's
   audio wrapper. Compare the engine's play/stop requests and
   `MusicCommand` gains.
3. Add regression cases for both complete cycles before changing the
   requests. Advance a controlled clock through fade, quiet and restart.
4. Fade-ins: where camp and the world map turn `MusicCue::Play(cue)`
   into a request, ask `play_music_with(&cue, MusicSwitch::new(fade_ms, 3000, false))`
   (the quiet has already silenced the old track; `out_ms` only matters
   when a switch interrupts a track). Fade-out: with 2398,
   `stop_music_with(3000)` and a schedule that stops 3 s before the
   track's end; without it, the current stop, its marker rewritten to
   `// engine-friction: engine#2398 (the fade-out is the manifest's one
   fade, not audio.md's 3 s)`, and lines 11–14 saying only that.
5. Test covering/uncovering camp, entering/leaving a town and a request
   interrupting a quiet interval. Keep existing resume rules and cue order.
6. Record both timing traces in completion notes, and the follow-up's
   number if step 4 kept the marker. Expected production change: about
   100–180 lines across the shared schedule and two existing adapters,
   plus focused tests; planned PR under 2,000 changed lines.

## Acceptance criteria

- [ ] With controlled time, the next camp cue starts at zero gain and
      reaches full gain over 3 s after a 5 s silence; the world map's
      repeat does the same after 30 s. No audible overlap replaces the quiet.
- [ ] With engine 2398 adopted: camp and the world map fade to zero over
      3 s, not over the 500 ms manifest default. Without it: the stop is
      marked `// engine-friction: engine#2398` and a follow-up ticket
      blocked by `engine#2398` exists.
- [ ] Town music and return to the map, and camp's cover/resume behavior,
      follow their existing rules; no duplicate start or unwanted restart
      occurs during a quiet interval.
- [ ] An unrelated ordinary music switch retains its 500 ms behavior.
- [ ] No `engine-friction: engine#2341` marker remains, and `music.rs`
      no longer says the engine can't fade a track in.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: shared schedule boundaries, both quiet intervals, interruption and
  resume, including time steps crossing a boundary.
- Integration: harness request/command traces for camp and world map,
  checking gains and exact transition times with a fake clock.
- Regression: the old implementation fails the 3 s fade-in checks (and
  the 3 s fade-out check when 2398 is adopted); no speaker or manual
  timing measurement is needed.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
