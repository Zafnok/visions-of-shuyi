---
id: "1047"
title: "Playdate: sound effects and music in formats the device plays"
type: feature
milestone: Post–Chapter 1
model: reasoning
effort: high
status: done
blocked_by: ["1033", "1036"]
nick_input: sign-off
completed: 2026-10-17
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

- [x] Every sound and music cue Chapter 1 uses plays in the Simulator
      (the device: Nick's sign-off, below).
- [x] The ADR records the formats and sizes (ADR-0084).
- [ ] Device numbers and Nick's comment in the PR / completion notes
      (waits for Nick's sideload; sign-off never blocks the PR).
- [x] `cargo deny check licenses` passes.
- [x] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: the converter on a test tone; the request-to-SDK mapping (pure
  part, with a fake SDK).

## Completion notes

**Done (2026-10-17).** The Playdate plays every sound and Chapter 1's
music ([ADR-0084](../../docs/adr/0084-playdate-audio-formats.md)).

- **Formats:** mono IMA ADPCM at 44.1 kHz for both sounds and music, 22
  KB a second. Sounds: 21 files, 340 KB, all loaded into memory behind
  the loading card. Music: Chapter 1's 20 tracks (12 min 39 s), 16.8 MB,
  each streamed by a file player. The device's `.pdx` is about 19.6 MB;
  `pdex.bin` grew 18 KB (2,487,387 → 2,505,424).
- **Converter:** `cargo xtask playdate` decodes (`lewton`, `hound`),
  mixes to mono and writes IMA ADPCM WAVs into `Source/audio/` and
  `Source/music/` (our own encoder), cached in
  `target/playdate/audio-v1/`; `pdc` makes them `.pda`s and trims the
  padded last block, so loops have no gap. Which music ships:
  `crates/content/playdate-music.txt` (cues and pools). The Chapter 1
  test now fails if Chapter 1 starts a track, or a scene names one, that
  the list leaves out.
- **Front end:** `crates/playdate/src/audio.rs` (the PC's rules, behind a
  `Speaker` trait, tested with a fake) and `src/pd/sound.rs` +
  `shim.c` (the SDK's sample and file players). 8 sample players. Fades,
  volumes and the Music/Sound options as on the PC. The music clock is
  the file player's position, so the title cinematic follows the song
  as heard.
- **Simulator check (debug build, a throwaway log, removed):** the title
  song starts on the first press and the clock follows it (1 s per 30
  frames); the cinematic's fight sound (`hit_sword`), the menu sounds,
  the scene tracks (`village_home`, `dungeon_tense`) with their fades,
  `battle_bright` and the map cursor's tick all reach the SDK; 21 sound
  files load in about 4 ms. No audio warnings. I can't listen to the
  Simulator, so how it *sounds* is Nick's sign-off.
- **Measuring on the device:** each minute's log says which track plays
  and whether it ran out of data (`Music: battle_bright, no gaps`). The
  bench build (`--bench`) pauses the music every other minute and says
  so: its fast-render times in a minute with music and one without give
  what music costs. The SDK's C API has no CPU-use call.
- **Deviation:** the ticket asked for "frame time with and without music
  in a battle"; the bench build's alternating minutes are how (above),
  since the normal build should never stop its music.
- **CI:** the `playdate` job also checks that `pdc` packed a converted
  sound and track.

No gameplay rules decided. *the agent's starting choices* (sound only, Nick
can veto): music is mono (the speaker is mono; stereo doubles the size
for headphones), and a sound at volume 0 is not played at all.

**For Nick (sign-off):** sideload `VisionsOfShuyi.pdx` (`cargo xtask
playdate --release`), listen to the title with its cinematic, a menu and
a fight, and say whether the music and sounds are good enough. For the
CPU cost, the bench build (`--release --bench`) in a battle for two
minutes or more: paste its minute lines.
