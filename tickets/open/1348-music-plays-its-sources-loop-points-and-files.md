---
id: "1348"
title: "The music plays its sources' loop points, an intro once and then the loop, from the files as their authors made them"
type: feature
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["1241"]
nick_input: none
completed:
---

# 1348 — The music plays its sources' loop points

## Context

The engine pin move of 2026-11-23 passed:

- engine PR #528 (engine ticket 2397, engine ADR-0137): a music cue can
  name a loop region (`loop_start_ms`, `loop_end_ms`), or its Ogg file's
  `LOOPSTART`/`LOOPLENGTH` tags can; the track plays its intro once, then
  goes round the region, and the music clock follows it. Only the
  `in-house-sound` backend plays the region: quad-snd, the game's today,
  plays the whole file with one warning. The Playdate's file player is
  engine ticket 3334.
- engine PRs #529 and #537 (engine tickets 2203 and 2234): `.flac` and
  `.mp3` sounds and tracks load and play (and JPEG, WebP, QOI and JPEG XL
  pictures draw), each behind a feature of `shuyi-assets`.

The game's music was cut and re-encoded by hand to get around both
(`docs/design/audio.md`'s table and the content's audio credits say which):
the night siege's Battle Music had its 4.6 s intro cut, so only its loop
(0:04.6 to 1:28.6) plays; one battle track's author's loop point isn't
used, so it goes round its whole file; and several tracks came as MP3 or
FLAC and ship as Ogg re-encodes.

1241 moves the game's sound onto the in-house backend; this ticket
follows it.

## Nick input

None: the loops are the ones `docs/design/audio.md` records from each
source. Whether a track sounds right with its intro is left for the Act 1
playtest's checklist (1064).

## Scope

**In:**
- Each track whose source gives a loop point (search the content's
  `audio/audio.ron` credits' notes and `docs/design/audio.md` for
  "loop"): its cue names the region, from the source's whole file. The
  bought or downloaded original goes in the private repository through
  the `content-change` skill, its credit unchanged.
- The tracks that came as MP3 or FLAC: measure each source against its
  Ogg (bytes in the package and the web download, and whether the
  browser and the Playdate's bake take it) and keep whichever is
  smaller; a re-encode stays only when it is.
- A test that each cue with a region loads and passes the manifest's
  check; the music clock's position round the region (the Harness).

**Out (do not do):**
- Choosing new music, or a track's volume.
- The Playdate's loop (engine 3334): until it lands, the Playdate keeps
  the cut file (its export bakes from the cue; say how in the notes).
- Pictures in other formats: the bought art is PNG.

## Implementation steps

1. List the tracks with a source loop point and the MP3 or FLAC sources.
2. Bring the originals into the private repository (`content-change`).
3. Name each region in its cue; turn on the `shuyi-assets` features the
   kept sources need.
4. Play each on the in-house backend; check the intro and the loop.

## Acceptance criteria

- [ ] Every track with a source loop point names it, and plays its intro
      once (a Harness test of the music clock).
- [ ] The source-or-Ogg measure per track in the completion notes.
- [ ] `check --target all` and the Playdate export pass.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the manifest's cues with regions load.
- Integration: the music clock round a region.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
