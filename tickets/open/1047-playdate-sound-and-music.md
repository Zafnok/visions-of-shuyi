---
id: "1047"
title: "Playdate: sound effects and music in formats the device plays"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1033", "1036"]
nick_input: sign-off
completed:
---

# 1047 — Sound and music on the Playdate

## Context

Nick kept **music and sound effects** on the Playdate, "converted to
formats the Playdate plays" (`docs/design/playdate.md`, Q5).

Audio is data (ADR-0026): screens ask for cues through `Ctx::audio`
(`crates/ui/src/audio.rs`); `Game::frame` hands `AudioRequest`s and the
`MusicState`'s commands to the front end, which plays them. Cues and their
files are in `assets/audio/audio.ron`; sound effects are OGG files in
`assets/audio/sfx/` (embedded), music tracks are OGG files in `music/`
beside the game (loaded when first played), each with its `length_ms`.

The Playdate SDK plays sound with its `sound` API: a **sample player**
for short sounds loaded in memory and a **file player** that streams
long files from the game's folder. Both take WAV (PCM or IMA ADPCM) and
the file player also MP3 (check the SDK 3.1.2 docs for the exact list
before choosing). ADPCM is 4 bits a sample and cheap to decode.

## Nick input

**Sign-off:** Nick plays through the title and a turn with sound on his
Playdate and says whether the music and sounds are good enough (0061:
"is audio quality and ROM size enough").

## Scope

**In:**
- **Conversion at build time** in `cargo xtask playdate` (or a
  `playdate-audio` step it calls): decode each OGG used by Chapter 1
  (sounds and music; the list from `audio.ron` and 1036's bundle list)
  and write the format chosen. Decode with a permissively licensed crate
  (e.g. `lewton`, MIT OR Apache-2.0; check ADR-0013 and `cargo deny`) in
  `xtask`; write WAV/ADPCM ourselves if no suitable crate. Choose the
  sample rate, channels and format per kind (sounds, music) from what
  the SDK supports and the `.pdx` size, and record the choice and the
  sizes in an ADR (`write-adr`: "Playdate audio formats"), with the total
  size of Chapter 1's audio.
- **Where the files go:** sounds inside the `.pdx` loaded into sample
  players at start or first use; music as files in the `.pdx` streamed
  by the file player. Nothing is committed: generated into
  `target/playdate/`.
- **Playing:** the front end (`crates/playdate/src/audio.rs`) turns each
  frame's `AudioRequest`s and music commands into SDK calls: play a sound
  at its volume, switch music (fading out as the PC does), stop, and
  keeps the `MusicClock` the title cinematic follows (`Game::set_music_playing`)
  in time with the file player's position.
- Volume options (sound, music) work as on the PC.
- The shim gains the sound calls.
- **Measure on the device:** the CPU cost of playing music (frame time
  with and without music in a battle) and the `.pdx` size.

**Out (do not do):**
- Voices (1048).
- Changing any sound or track, or the PC's audio.

**Carve (1104, 1110):** either order works. If audio cues moved to the
engine (1104), the converter follows the engine's cue format; the front
end stays beside the engine's app (1110).

## Implementation steps

1. Read ADR-0026, `crates/ui/src/audio.rs`, `crates/app/src/audio*`,
   the SDK's *Sound* docs.
2. Decide formats; write the ADR.
3. The converter; tests on a short test tone (decode → convert →
   known header and length).
4. The front end's audio; Simulator check (title song with the
   cinematic, menu sounds, a fight's sounds).
5. Ask Nick to sideload.

## Measured on Nick's Playdate

Frame time in a battle with and without music; the `.pdx` size; Nick's
word on how it sounds.

## Acceptance criteria

- [ ] Every sound and music cue Chapter 1 uses plays in the Simulator and
      on the device.
- [ ] The ADR records the formats and sizes.
- [ ] Device numbers and Nick's comment in the PR / completion notes.
- [ ] `cargo deny check licenses` passes.
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: the converter on a test tone; the request-to-SDK mapping (pure
  part, with a fake SDK).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
