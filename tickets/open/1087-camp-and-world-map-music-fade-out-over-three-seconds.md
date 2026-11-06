---
id: "1087"
title: Camp and world-map music fade out over 3 s
type: feature
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["1080", "engine#2398"]
nick_input: none
completed:
---

# 1087 — Camp and world-map music fade out over 3 s

## Context

[Audio design](../../docs/design/audio.md), *Starting values*, gives
camp and the world map 3 s out / 5 s quiet / 3 s in (camp) and 3 s out /
30 s quiet / 3 s in (the world map). Ticket 1080 shipped the fade-ins and
both quiet intervals through `AudioQueue::play_music_with`, but the
engine at the pin can stop music only over the manifest's one fade
(`music_fade_ms`, 500 ms): `MusicState::stop` always uses the default
switch's `out_ms`.

Engine 2398
([its ticket](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/2398-stop-music-with-a-fade-chosen-per-request.md))
adds a stop request that carries its own fade. 1080 left the one
workaround marked in `crates/ui/src/screens/camp/music.rs`, on the stop
in `ask`:
`// engine-friction: engine#2398 (the fade-out is the manifest's one
fade, not audio.md's 3 s)`. This ticket adopts the stop and removes it.

## Nick input

None. The timings are already decided. Tests inspect requests and music
commands; anything left to judge by playing belongs on the Act 1 playtest
checklist (1064).

## Scope

**In:**
- ~~Move the pin past engine 2398 (`engine-change` step 7, with the
  `engine-pin` skill's review and ledger section), or reuse an
  `engine-pin` PR that has already moved it.~~(done by the pin move of 2026-11-05).
- `music::ask` stops with an explicit 3 s fade (`FADE_OUT_MS`) through
  the engine's new stop operation.
- `CampMusic` starts each fade-out 3 s before the track's end, not the
  manifest's fade: camp and the world map pass the 3 s instead of
  `music_fade_ms` to `CampMusic::start`. The quiet still counts from the
  fade's end.
- Remove the `engine#2398` marker and the module docs' workaround
  paragraph.

**Out (do not do):**
- Any other screen's stop or switch: they keep the manifest's 500 ms.
- New cues, timings or pool changes.
- The engine change itself (engine 2398).

## Implementation steps

1. ~~Check engine 2398 is in the engine's `tickets/done/`, then move the
   pin past it (`cargo xtask engine-pin`)~~ (done by the pin move of 2026-11-05); read its completion notes
   for the stop operation's name.
2. In `crates/ui/src/screens/camp/music.rs`: add
   `pub const FADE_OUT_MS: u32 = 3000;`; make `ask`'s `Stop` arm use the
   new stop with `FADE_OUT_MS`; drop the marker and the module docs'
   workaround paragraph.
3. In `crates/ui/src/screens/camp.rs` (`start_music`) and
   `crates/ui/src/screens/world_map.rs` (`tick_music`), pass
   `f64::from(FADE_OUT_MS) / 1000.0` as `fade_s` instead of the
   manifest's fade.
4. Update the tests 1080 added: `each_camp_track_fades_in_after_five_seconds_of_quiet`
   (camp/tests.rs) and `the_maps_track_fades_in_again_after_thirty_seconds_of_quiet`
   (world_map/tests.rs) trace gains through `music::Speaker`; check the
   gain is 1 at 3 s before the end, about 0.5 at 1.5 s before, and 0 at
   the end, then the quiet and fade-in as now. Update
   `the_maps_track_rests_between_rounds` and
   `camp_plays_the_camp_pool_in_turn`, which time the stop by the
   manifest's fade.
5. Expected production change: about 15–30 lines; PR well under 2,000.

## Acceptance criteria

- [ ] With controlled time, camp's and the world map's track fade from
      full gain to zero over 3 s ending with the track, not over 500 ms.
- [ ] The quiet (5 s camp, 30 s map) and the 3 s fade-in are unchanged.
- [ ] The town's music switch and every other screen's stop keep the
      manifest's 500 ms (the existing `a_towns_music_while_it_is_open_then_the_maps`).
- [ ] No `engine-friction: engine#2398` marker remains.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `CampMusic`'s schedule with a 3 s fade (the existing
  `music.rs` tests with the fade changed).
- Integration: the two gain traces above, through `music::Speaker`.
- Regression: today's 500 ms stop fails the 3 s fade-out check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
