---
id: "1332"
title: "A place's ambience plays under the music: its sound bed and passing calls"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["1322", "1314"]
nick_input: answer-first
completed:
---

# 1332 — A place's ambience plays under the music: its sound bed and passing calls

## Context

1322 decides what ambience is (a bed, passing calls, how loud, on which
screens, whether light changes it, the Playdate's). This ticket builds
the system and the sounds 1322 picked; each chapter's own choice is its
chapter ticket (1335–1342).

What exists: looped sounds already work (the title's crackle, 0922:
`ctx.audio.start_sound(FIRE_SOUND)` and the engine's `stop_sound` fade,
`crates/ui/src/screens/title.rs`); cues live in `assets/audio/audio.ron`
with tags; volumes follow `audio.md`, *Starting values*. Battle sounds
are played from events in `crates/ui/src/screens/battle/event_sounds.rs`
and `sounds.rs`. The engine's parked ticket 2370 (weather ambiences:
particles, an overlay and a loop sound) is the engine's version of part
of this.

The rules never hear any of it: `core` has no clock and no sound
(ADR-0004), so play records and bots are unchanged. Passing calls are
picked from the screen's time and a seed (never `rand` with a clock
seed), so a test at a fixed time knows what plays.

## Nick input

**Answer first:** 1322. Nothing to check after merge: verified by tests;
what is left to judge by ear goes on 1064's checklist.

## Scope

**In:**
- An **ambience** as data (a new file format or rows in `audio.ron`, the
  agent decides; ADR if it's a new format): a bed cue, a list of call cues
  with a gap range, a volume, and per light (day/dusk/night) variants if
  1322 wants them.
- A battle file, a scene's place and a world-map town name an ambience by
  id (public stand-in data in `assets/`; the private chapters through the
  `content-change` skill only for what 1335–1342 set, not here).
- The player: starts the bed with the screen, fades it with the music's
  0.5 s fade, schedules calls from seed and time, ducks or stops as 1322
  says during a fight and in menus; the Sound volume (Options) scales it.
- The sounds 1322 picked, imported by `assets-src/audio/import.py`
  (volume-matched, loop crossfaded as for the crackle), credited in
  `THIRD_PARTY_ASSETS.md` and the credits.
- The test chapter (`assets-test/`) names one ambience, so the gates
  exercise it without the private content.

**Out (do not do):** choosing a chapter's ambience (1335–1342); particles
(1316, 1333, 1334); new music; positional or stereo sound unless 1322
asks for it.

## Implementation steps

1. Run `engine-friction` before coding: write the expected size (about
   250 production lines: a data type, a loader, a small scheduler and its
   hook in three screens). If the scheduler belongs in the engine (2370),
   follow `engine-change` and ask Nick before the engine ticket (memory:
   engine steps need Nick's OK); otherwise build it in `crates/ui`.
2. The data type and loader in `crates/content`, with its validation
   (unknown cue, empty gap range).
3. `crates/ui/src/ambience.rs`: `AmbiencePlayer` with `update(ctx, now)`
   that starts the bed and returns the calls due between two times (a
   pure function of seed and time, like `title.rs`'s `play_sounds`).
4. Hook it into the battle screen, the dialogue screen's place and the
   world map's towns, per 1322's list.
5. Import the sounds; credits.

## Acceptance criteria

- [ ] A test: the same seed and times give the same calls.
- [ ] A test: the bed starts with the battle and fades out with it.
- [ ] A test: with Sound at 0 nothing is started.
- [ ] A test: an ambience naming an unknown cue fails content validation.
- [ ] `THIRD_PARTY_ASSETS.md` lists every new sound.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the scheduler; the loader's validation.
- Integration: the battle screen through the headless harness starts and
  stops the bed (the audio fake records the calls).

## Lesser targets

As 1322 decides for the Playdate; unbundled sounds leave the ambience
silent, never a crash (as the title's crackle does, 0924). GBA: no target.

## Completion notes
