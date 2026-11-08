---
id: "0476"
title: Vix speaks her lessons and tips
type: feature
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0043", "0238", "0475", "0721", "0720", "0826"]
nick_input: none
completed: 2026-11-08
---

# 0476 — Vix speaks her lessons and tips

## Context

Nick, 2026-11-01: "I think we can voice Vix too." (0043,
`docs/design/voices-languages-and-script.md`, *Voices*.) Her voice is `guide`
in the cast table, read by Chatterbox. 0475 leaves her lines free of keys. Clips
belong to dialogue line ids (ADR-0046, ADR-0045 §3), and lessons and tips have
none yet.

**From 0475 (done):** what she says is a lesson's or a tip's `text`, and
the content check keeps every key out of it. A lesson's items and a tip's
`how` (its instructions, with the keys) are silent text: voice `text` only.

## Nick input

None. Verified by tests. What is left to judge by playing goes on the Act 1
playtest's checklist (1064).

## Scope

**In:** line ids for lessons' and tips' `text`; the lesson and tip boxes asking
for her clip and stopping it; `cargo xtask voice` covering them; her clips for
every lesson and tip in the private repository.

**Out (do not do):** voicing `items`, `how` or the *Guides* list's headings
(silent, Nick); replaying her in the *Guides* list unless a line is opened
there as a box already.

## Implementation steps

1. Give lessons and tips line ids the way ADR-0045 §3 does for script lines, so
   rewording gives a new id. Scene id in the voice folder: `vix`.
2. When a box opens, call `play_voice`; closing it, or another box opening,
   stops it. With voices off nothing is asked for (0238's `Ctx::play_voice`).
3. Extend 0721's `status` and `generate` to read lessons and tips, with speaker
   `guide`.
4. Generate her clips (`cargo xtask voice generate --scene vix`), check them by
   transcribing as 0722 does, commit to the private repository, and pin.

## Acceptance criteria

- [x] Harness test: a lesson opening with a clip in the test manifest queues one `PlayVoice`; closing it queues `StopVoice`.
- [x] With voices off, no voice request is queued (test).
- [x] `items` and `how` never ask for a voice (test).
- [x] `cargo xtask voice status` reports nothing missing for `vix`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: Harness tests on `audio_requests()`.

## Completion notes

**What was done.**

- **Line ids (step 1):** `trpg_content::voice::guide_scene` builds scene
  `vix` from the lessons' and tips' `text`, one `guide` line each, its id
  `vix_<hash>` made as a script line's is (`line_hash` of speaker and
  words), so rewording gives a new id. The box's line breaks are read as
  spaces (they are only where the box breaks the text). The same words
  twice would be one line. Items and `how` get no line.
- **Playing (step 2):** the battle screen notes the guide's line of the box
  on screen as each frame ends (`battle/guide_voice.rs`): a box coming on
  screen calls `ctx.play_voice`, it going or another box taking its place
  calls `ctx.stop_voice`. The voice manifest is checked against the
  dialogue plus scene `vix` (`Shuyi::voices`). With voices off
  `Ctx::play_voice` asks for nothing (0238).
- **The tool (step 3):** `cargo xtask voice` reads scene `vix` with the
  story's scenes (`--scene vix` for hers alone), speaker `guide`, who is
  now in `characters.ron`'s `speakers` so the cast may name her.
- **Clips (step 4):** 22 clips, one per shipped lesson (10) and tip (12),
  made by `cargo xtask voice generate --scene vix` with Chatterbox and her
  reference clip `guide.flac` (cast entry `"guide"` added to the private
  repository's `cast.ron`). Dry run: 22 undirected, 2,478 characters. No
  one listened to them: as in 0722, the tool transcribes every take and
  keeps only one that says the line's words; all 22 passed, none failed.
  All are Vorbis, 44.1 kHz, mono (ffprobe), 1.3 MB together. Loudness
  (`ebur128`): 18 at −17.9 to −19.4 LUFS, 4 at −20.5 to −22.5, held under
  the target by the −1 dBTP peak ceiling, the same rule as the music.
  `cargo xtask voice status --scene vix`: nothing to make. Committed to the
  private repository (`2f843fe`), between two of 0468's commits. 0468
  merged first and pinned its later commit (`db3e808`), which has these
  clips, so this PR keeps that pin.
- **A stop only when she spoke:** `ctx.stop_voice` is asked for only when
  her clip was asked for; a stop with nothing playing changed the kept
  Quick Battle session's record (`sessions::`), which has no clips.

**Deviations.**

- The tool voices the **shipped** lessons only, not the test lesson
  (`assets-test/data/lessons.ron`, embedded in debug builds, which the
  tool is): a clip of a line a build that ships doesn't have would make
  that build refuse the whole manifest. The game's own check still reads
  the test lesson in a debug build, where it simply has no clip.
- A box back on screen after a pause (a combat playing over an open
  lesson, then the lesson again) is not said a second time: she stops when
  the box goes and stays quiet when the same box returns. Without this
  every lesson that stays open across an attack would start over.

**The agent's starting rules** (for the Act 1 playtest, 1064):

- She speaks a box once, as it first comes on screen; a box hidden by a
  combat or a scene and shown again stays silent. (Judge by playing:
  whether she should say it again.)
- The lesson box the rewind screen shows (the rewind lesson) is said when
  it opened on the battle map, not again on the rewind screen.
- The Guides list says nothing (the ticket's *Out*).

**Left to judge by playing (1064):** how her generated voice sounds in
the lessons and tips; whether a tip's line overlapping the start of a
lesson's (closing a tip stops her, then the lesson speaks) feels right.

**Measured.** Engine friction: estimate 90 production lines, measured 111
(1.2×), under 2×. PR size: estimate 400, measured 479.

**Follow-ups.** None.
