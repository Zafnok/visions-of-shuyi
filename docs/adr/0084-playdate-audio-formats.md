# ADR-0084: Playdate audio formats: mono IMA ADPCM at 44.1 kHz, converted at build time

- **Status:** Accepted
- **Date:** 2026-10-17
- **Related tickets:** 1047 (this), 1036, 1048, 1049
- **Builds on:** ADR-0026 (audio cues as data, played by the platform),
  ADR-0037 (the music clock), ADR-0070 (the Playdate front end),
  ADR-0082 (the Playdate's bundle)

## Context

Nick kept the music and the sound effects on the Playdate, "converted to
formats the Playdate plays" (`docs/design/playdate.md`, Q5). His sign-off
question was "is audio quality and ROM size enough".

What the PC has (ADR-0026): 21 sound files (OGG Vorbis, and WAV for the
sounds we make ourselves) embedded in the game, and the music as OGG files
in `music/` beside it. Every file is 44.1 kHz, mono or stereo. The game
names cues; the platform plays them.

What the Playdate SDK 3.1.2 plays (*Inside Playdate*, "Sound"; its
`pd_api_sound.h`):

- A **sample player** plays a sound held in memory. It takes PCM (8 or
  16 bit) or IMA ADPCM, mono or stereo, not MP3.
- A **file player** streams a file from the game's folder with a small
  buffer (1 s by default). It takes the same formats and MP3.
- The SDK's `pdc` compiles a `.wav` in the game's `Source/` folder into
  its own `.pda` file. The SDK opens one by its path without the
  extension.
- The audio system runs at 44.1 kHz. The docs: "ADPCM is the ideal audio
  format to use for Playdate games. It is less CPU-intensive to decode
  than MP3, while still providing a much smaller file size than
  uncompressed audio". They name ffmpeg's `adpcm_ima_wav` and Audacity's
  IMA ADPCM export to make it.
- The speaker is a single small one; the headphone jack is stereo.
- There is no CPU-use call in the C API (Lua's `getStats` has no C
  counterpart), so the cost of playing music shows only as slower frames.

The game is already short of CPU on the device (tickets 1055, 1057,
1059), so the decoder's cost matters more than a few MB of storage.

Chapter 1's music (what the Playdate build ships, 1036's "only what it
uses"): the title, four scene tracks, the battle, the 13 victory jingles
and the defeat track: 20 files, 12 min 39 s.

## Decision

### 1. Formats

| | Sounds | Music |
| --- | --- | --- |
| Encoding | IMA ADPCM, 4 bits a sample | IMA ADPCM, 4 bits a sample |
| Channels | Mono (stereo sources averaged) | Mono (stereo sources averaged) |
| Rate | 44.1 kHz (the sources' own) | 44.1 kHz (the sources' own) |
| Blocks | 1,024 bytes (2,041 samples) | 1,024 bytes (2,041 samples) |
| Played by | A sample player, from memory | A file player, streamed |
| Size | 22 KB a second | 22 KB a second |

- **ADPCM**, as the SDK recommends: a quarter of 16-bit PCM, and far
  cheaper to decode than MP3.
- **Mono**: the speaker is mono; stereo would double the size for
  headphones only.
- **44.1 kHz**: the system's own rate, so nothing is resampled at play
  time, the track lengths in `audio.ron` (`length_ms`, ADR-0037) stay
  exact, and orchestral music keeps its top end. Halving the rate halves
  the size; it is the first thing to change if Nick finds the size too
  big or the CPU too short.
- **1,024-byte blocks**: what both tools the SDK's docs name write for
  mono at 44.1 kHz. The last block is padded with silence and a `fact`
  chunk gives the real number of samples. `pdc` trims the `.pda` to it
  (checked: `menu_move` is 2,048 bytes of blocks in the WAV and 1,465 in
  the `.pda`), so a looped track loops with no gap.

### 2. Sizes (Chapter 1)

| | Files | Bytes | Where |
| --- | --- | --- | --- |
| Sounds | 21 | 340,256 | The `.pdx` (`audio/`); all in RAM while the game runs |
| Music | 20 (12 min 39 s) | 16,802,340 | The `.pdx` (`music/`); two 1 s buffers at most in RAM |
| **All audio** | 41 | **17,142,596** (about 17 MB) | |
| `pdex.bin` (the game) | | 2,505,424 (was 2,487,387) | |

The device's `.pdx` is about 19.6 MB with its audio; the Simulator's
`pdex.dll` adds 6 MB to the one the build makes for both.

### 3. Converted by the build, never committed

`cargo xtask playdate` converts the audio before `pdc` packs the game
(`crates/xtask/src/playdate_audio.rs`, a `Step::Audio` in its plan):

- **What:** every file of every sound cue in `assets/audio/audio.ron`,
  and the music cues listed in `crates/content/playdate-music.txt` (a
  music cue or a pool per line; a pool ships all its tracks; an unknown
  id stops the build). Sounds are not listed: they are shared by every
  chapter and small. Music is a list of what ships, not of what is left
  out (ADR-0082's way round), because the Playdate's build is Chapter 1
  and a later chapter's track must not reach it by default.
- **How:** `lewton` decodes OGG Vorbis and `hound` reads WAV (both MIT
  and/or Apache-2.0, already in the lock file through macroquad's audio);
  the IMA ADPCM encoder and the WAV writer are ours (about 100 lines).
- **Where:** sounds to `Source/audio/<file without extension>.wav`, music
  to `Source/music/<file without extension>.wav`, the paths
  `trpg_content::audio::playdate_sound_path` and `playdate_music_path`
  give the front end. Two files on one path stop the build.
- **Cache:** decoding the music takes seconds, so each converted file is
  kept in `target/playdate/audio-v<FORMAT>/` and made again only when its
  source is newer. `FORMAT` changes with the conversion.
- The build's summary line ends with the totals
  (`audio: 21 sound files 0.4 MB, 20 tracks 16.8 MB`, the WAVs before
  `pdc` trims them).

The PC's audio and its files do not change. The Playdate's bundle still
leaves `audio/sfx/` out (ADR-0082): the front end plays its own copies.

### 4. Playing

`crates/playdate/src/audio.rs` does on the Playdate what the PC's player
does (ADR-0026 §2), plain code behind a `Speaker` trait; `src/pd/sound.rs`
is the SDK's side and `shim/shim.c` gains one wrapper per sound call.

- **Sounds:** every sound file is loaded into memory while the loading
  card shows (the load report's "Sounds: 21 files loaded", and its
  "(the sounds)" step's time). A sound plays on one of 8 sample players:
  a free one, else they are taken in turn. Its volume is the cue's times
  the request's times the player's Sound option. A sound at no volume is
  not played. Variants are picked by a counter through `splitmix64`, not
  the game's RNG (ADR-0019).
- **Music:** `MusicState`'s commands as on the PC: `Load` opens the file
  in a file player, `Start` plays it (looped for ever, or once), `Gain`
  sets its volume each frame of a fade, `Stop` stops and frees it. A
  track the `.pdx` lacks plays as silence, with a warning in the log.
- **The music clock** (ADR-0037) is the file player's own offset in its
  file, read before each frame; `None` once it stops. So the title
  cinematic follows what is heard, not the wall clock, and no clock runs
  on while a track is still opening.
- **Options:** the Music and Sound volumes multiply in as on the PC, and
  a change reaches the track playing.
- A game that quits stops its music.

### 5. Measuring

- Once a minute the log names the track heard and whether its file
  player ran out of data ("Music: battle_bright, no gaps").
- **The bench build** (`--bench`, ticket 1055) pauses the music every
  other minute and says so in its minute report ("Bench: the music was
  paused this minute"). Its whole-frame renders are a fixed workload, so
  their time with and without the file player decoding is the music's
  CPU cost.

## Consequences

- The Playdate plays every sound and Chapter 1's music with no change to
  the game, the PC or the files in the repository.
- Each new chapter on the Playdate adds its tracks to
  `playdate-music.txt`. The Chapter 1 test
  (`crates/playdate/src/chapter_1.rs`) fails if the game starts a track,
  or a scene names one, that the list does not ship.
- The `.pdx` grows by about 17 MB, nearly all music. At 22 KB a second,
  each further minute of music is about 1.3 MB.
- Sounds cost 340 KB of RAM for the whole game.
- The first build converts for some seconds; later ones copy the cache.
- Voices (1048) can use the same converter and the file player.
- 1049 (one build per language) changes nothing here: audio is the same
  in every language.

## Alternatives considered

- **Stereo music** — 33.6 MB for Chapter 1, for headphones only, and
  twice the decoding.
- **22.05 kHz** — 8.4 MB and half the decoding, but duller orchestral
  music and resampling at play time. Kept as the first fallback if the
  device numbers or Nick say so.
- **MP3 music** — about 12 MB at 128 kbit/s, but the SDK warns that MP3
  decoding is CPU-intensive, the game is short of CPU, and an MP3
  encoder is a heavy dependency (LAME is LGPL, ADR-0013).
- **16-bit PCM sounds** — no decoding at all, but four times the RAM
  (1.4 MB) for a cost ADPCM barely has.
- **Sounds embedded in `pdex.bin`, played with `newSampleFromData`** —
  no file reads, but the decoded table would need its own build step
  into the Rust crate; `pdc` and `sample->load` already do it.
- **ffmpeg at build time** — the SDK's own suggestion, but another tool
  to install on Nick's machine and on CI; the encoder is small.
- **Load each sound at first use** — a faster start by about 4 ms in the
  Simulator, but a file read in the middle of a frame the first time a
  sound plays.
- **The wall clock for the music clock, as on the PC** — the file
  player knows exactly where it is; the wall clock would run ahead
  while a file opens and drift over loops.
