---
id: "1332"
title: "A place's ambience plays under the music: its sound bed, with its own volume"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["1322", "1314"]
blocks: ["1064"]
nick_input: answer-first
completed:
---

# 1332 — A place's ambience plays under the music: its sound bed, with its own volume

## Context

1322 decided what ambience is (`docs/design/audio.md`, *Ambience*, and
`options.md`, *Volumes*): **a sound bed per place, no passing calls** (Nick
didn't want them "for now"; 1344 asks again later). A bed is levelled on
import to the music's loudness and has **its own volume on Options**,
starting 10 under the music's (90 against 100). It plays on the battle map,
in a scene over its place, on the world map, in an open town and in camp;
**not in the fight** (it stops when the combat stage opens and comes back
after); it **dips under a voiced line** as the music does (rule 11); the
world map's bed keeps playing through the 30 s quiet between rounds of the
map's track; a place may have **a bed per light** (noon, dawn, dusk,
night); the Playdate plays the beds if the bench says they run well. This
ticket builds the system and the one bed 1322 picked (Summer Meadow, the
day bed); each chapter's own choice is its chapter ticket (1335–1342).

What exists: looped sounds already work (the title's crackle, 0922:
`ctx.audio.start_sound(FIRE_SOUND)` and the engine's `stop_sound` fade,
`crates/ui/src/screens/title.rs`); cues live in `assets/audio/audio.ron`
with tags; volumes follow `audio.md`, *Starting values*. Battle sounds
are played from events in `crates/ui/src/screens/battle/event_sounds.rs`
and `sounds.rs`. The engine's parked ticket 2370 (weather ambiences:
particles, an overlay and a loop sound) is the engine's version of part
of this.

The rules never hear any of it: `core` has no clock and no sound
(ADR-0004), so play records and bots are unchanged.

The engine already lets a game give each sound group its own starting
volume in its audio manifest (`groups: [(id: "music", volume: 90)]`, 0879's
context), and the music's dip under a voice is `audio.ron`'s `ducks`
(0865): check both at the pin before building a new group or a new dip.
0879 may move the music's starting volume; the ambience's start stays 10
under it.

## Nick input

**Answer first:** 1322 (answered 2026-11-23). Nothing to check after merge: verified by tests;
what is left to judge by ear goes on 1064's checklist.

## Scope

**In:**
- An **ambience** as data (a new file format or rows in `audio.ron`, the
  agent decides; ADR if it's a new format): a bed cue, and a bed per light
  (noon, dawn, dusk, night) where a place has one; a light without one
  plays the noon bed (*Claude's starting rule*, `audio.md`).
- A battle file, a scene's place and a world-map town name an ambience by
  id (public stand-in data in `assets/`; the private chapters through the
  `content-change` skill only for what 1335–1342 set, not here).
- A new sound group, **ambience**, with its own **Ambience volume** row
  on Options under the Sound volume, starting 10 under the music's, kept
  in the player profile (a settings version bump if the saved form
  changes), back to its start with `Restore defaults` (`options.md`).
- The player: starts the bed with the screen and fades it with the
  music's 0.5 s fade; fades it out when a fight's combat stage opens and
  back in when it closes; keeps the world map's bed through the quiet
  between rounds of the map's track; dips it under a voiced line by the
  music's numbers; switches it with the map's light.
- The bed 1322 picked, Summer Meadow (baryy, freesound 409143, CC0),
  imported by `assets-src/audio/import.py` to the music's loudness
  (`MUSIC_LUFS`), a 40 s stretch looped with its next 2 s crossfaded in as
  for the crackle, credited in `THIRD_PARTY_ASSETS.md` and the credits.
- The test chapter (`assets-test/`) names one ambience, so the gates
  exercise it without the private content.

**Out (do not do):** passing calls (not wanted for now; 1344); choosing a
chapter's ambience (1335–1342); particles (1316, 1333, 1334); new music;
positional or stereo sound.

## Implementation steps

1. Run `engine-friction` before coding: write the expected size (about
   250 production lines: a data type, a loader, the sound group and its
   Options row, and the bed's hook in five screens). If a group's own
   volume or the dip needs the engine (2370), follow `engine-change` and
   ask Nick before the engine ticket (memory: engine steps need Nick's
   OK); otherwise build it in `crates/ui`.
2. The data type and loader in `crates/content`, with its validation
   (unknown cue, unknown light).
3. The ambience sound group and its Options row (`options.md`, *Volumes*).
4. `crates/ui/src/ambience.rs`: which bed a screen plays (its place and
   light), started, faded and dipped as above.
5. Hook it into the battle screen (and the fight's open and close), the
   dialogue screen's place, the world map and its towns, and camp.
6. Import the bed; credits.

## Acceptance criteria

- [ ] A test: the bed starts with the battle and fades out with it.
- [ ] A test: the bed stops when the fight opens and is back after it.
- [ ] A test: a map at night plays the place's night bed; a light with no
      bed of its own plays the noon bed.
- [ ] A test: the Ambience volume starts 10 under the music's, and with it
      at 0 no bed is started.
- [ ] A test: the bed dips under a voiced line by the music's numbers.
- [ ] A test: an ambience naming an unknown cue fails content validation.
- [ ] `THIRD_PARTY_ASSETS.md` lists every new sound.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: which bed a place and light give; the loader's validation.
- Integration: the battle screen through the headless harness starts and
  stops the bed, also around a fight (the audio fake records the calls);
  the Options screen's new row (snapshot).

## Lesser targets

The Playdate plays the same beds if `cargo shuyi bench` says they run
well within 1314's `Reduced` tier; otherwise it bundles none and has no
Ambience row. Unbundled sounds leave the ambience silent, never a crash (as
the title's crackle does, 0924). GBA: no target.

## Completion notes
