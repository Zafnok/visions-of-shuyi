---
id: "1332"
title: "A place's ambience plays under the music: its sound bed, with its own volume"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1322", "1314"]
blocks: ["1064"]
nick_input: answer-first
completed: 2026-11-25
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

- [x] A test: the bed starts with the battle and fades out with it.
- [x] A test: the bed stops when the fight opens and is back after it.
- [x] A test: a map at night plays the place's night bed; a light with no
      bed of its own plays the noon bed.
- [x] A test: the Ambience volume starts 10 under the music's, and with it
      at 0 no bed is started.
- [x] A test: the bed dips under a voiced line by the music's numbers.
- [x] A test: an ambience naming an unknown cue fails content validation.
- [x] `THIRD_PARTY_ASSETS.md` lists every new sound.
- [x] All gates in the `run-gates` skill pass.

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

Built as ADR-0132. No engine change: the beds play in the game's own
sound group `ambience` (volume 90, 10 under the music's 100) with a
`ducks` rule copying the music's, both the engine's (engine ADR-0091,
ADR-0099); the Options row is the engine's volume row on that group.

- **Data:** `assets/data/ambience.ron` (new) names each ambience's beds by
  light and the ambience of the scenes' places (by map id) and camp; a
  battle file names its own (`ambience: "meadow"`, read on the side like
  its `light`); a world map names the map's and each town's
  (`ambience: Some(...)`). Content validation refuses an unknown cue, a
  cue that isn't looped or isn't in the `ambience` group, an unknown
  light, an ambience without a `noon` bed, an unknown map and an unknown
  ambience name.
- **Player:** `trpg_ui::ambience`, one bed at a time on the context,
  hooked into the battle screen (hushed while a fight's stage shows), the
  dialogue screen (its place; `@place battle` plays the battle's), the
  world map (an open town's, else the map's) and camp.
- **The bed:** Summer Meadow (baryy, freesound 409143, CC0), the HQ
  preview, 40 s from 0:20, mono, looped with its next 2 s crossfaded in,
  imported by `import.py` to the music's loudness (-20.4 LUFS; it needed
  +31 dB, so a limiter at -1 dBTP keeps its chirps under the ceiling,
  where the music's import would have stopped 5 dB short). The test
  files' Quick Battle, test battle and test world map play it (meadow).
- **Private content:** its `audio.ron` declares the same group, dip, bed
  cue and credit (content-change PR in the private repository); no
  chapter names an ambience yet (1335-1342).
- **Playdate:** no bench was run, so it plays no bed: its skins turn the
  player off, its build bakes no `ambience`-group cue, its bundle leaves
  out `audio/ambience/`, and the Ambience row is PC-only. A later ticket
  runs the bench with a bed to turn them on.
- **Tests touched:** the kept Quick Battle session was recorded again
  (a frame's hash counts the sounds asked for, and the battle now asks for
  its bed); the skin test compares the sounds without the fades' per-frame
  volume steps, which showed footsteps under the sprite skin falling some
  frames later than under glyphs: an older difference, ticket **1358**
  (new).
- **Size:** estimate 320 production lines, measured 409 (1.3×); PR about
  1,450 changed lines.
- **Deviation:** the ticket's step 1 named an engine ticket if the group
  or the dip needed the engine; neither did.

**Claude's starting rules** (recorded in `audio.md`, *Starting values*;
for the Act 1 playtest, 1064):

- A town with no bed of its own plays the world map's while it is open.
- A scene over the battle with no place of its own leaves the battle's bed
  playing; a scene in black plays none.
- The Playdate plays no bed and has no Ambience row until the bench is run
  with them.
- The stretch of Summer Meadow (40 s from 0:20) is Claude's; Nick heard
  the recording on 1322's listening page.

To judge by ear in the Act 1 playtest (1064): the bed's level at 90 under
the music, its fades (0.5 s) around a fight, and the loop's seam.
