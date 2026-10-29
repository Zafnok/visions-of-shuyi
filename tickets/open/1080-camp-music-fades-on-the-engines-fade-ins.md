---
id: "1080"
title: Camp's music fades as audio.md says, on the engine's fade-ins
type: feature
milestone: Post–Chapter 1
model: sonnet-5
effort: low
status: todo
blocked_by: ["1003", "engine#2341"]
nick_input: none
completed:
---

# 1080 — Camp's music fades as audio.md says, on the engine's fade-ins

## Context

Ticket 1003 made the Camp screen rotate the `camp` pool's tracks
(`docs/design/audio.md`, *Starting values*: a track fades out over 3 s
at its end, camp is quiet 5 s, the next fades in over 3 s). The engine
at 1003's pin fades music out only over the manifest's one fade
(`music_fade_ms`, 0.5 s) and never fades a track in, so 1003 ships a
marked workaround (`// engine-friction: engine#2341` in
`crates/ui/src/screens/camp/music.rs`): the fade-out is the 0.5 s one
and the next track starts at full volume.

Engine ticket 2341 (music crossfades and fade-ins, a switch per request)
adds what camp needs.

## Nick input

None. The timings are already decided (audio.md, *Starting values*).

## Scope

**In:**
- ~~Move the engine pin past engine#2341 (`engine-change` skill, step 7).~~
  (done by the pin move of 2026-10-29, `c9fb84c9` → `50ae33db`)
- Camp's switches use 2341's per-request fades: 3 s out, 3 s in, the
  5 s quiet between them.
- Delete the workaround and its `engine-friction` marker.

**Out (do not do):**
- Any other screen's music switch.

## Implementation steps

1. ~~Move the pin;~~ (done by the pin move of 2026-10-29, `c9fb84c9` →
   `50ae33db`) read 2341's API (`AudioRequest::play_music(cue, Some(switch))`
   and `MusicSwitch`; the backends already start a track at
   `MusicCommand::Start`'s `gain`).
2. Change `CampMusic` to ask for the fades; update its tests.

## Acceptance criteria

- [ ] Camp fades a track out over 3 s, waits 5 s, fades the next in over
      3 s (tests of the music commands).
- [ ] No `engine-friction: engine#2341` marker is left.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `CampMusic`'s schedule and the requests it makes.
- Integration: the harness's music commands while camp is open.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
