---
id: "1260"
title: "Playdate: ship the music Chapters 2 to 8 ask for"
type: feature
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1260 — Playdate: ship the music Chapters 2 to 8 ask for

## Context

The Playdate build ships only the music listed in
`crates/content/playdate-music.txt` (ticket 1047, ADR-0084); a track not
listed plays as silence there, with a warning. The list has Chapter 1's
tracks, and since 1066 the world map's and its towns'. Chapters 2 to 8
(0848 on) added battles and scenes whose tracks aren't listed, so they
are silent on the Playdate (0848's Completion notes say so), and so are
the world map's other places' cues (`side_quest`, `dungeon_tense` is
already in). Nick plays Act 1 on his Playdate in 1064.

## Nick input

None.

## Scope

**In:**
- Every music cue or pool Act 1's chapters, battles, scenes and world
  map places ask for, in `playdate-music.txt`.
- A test like `crates/playdate/src/chapter_1.rs`'s that fails while a
  track Act 1 asks for is not listed (the chapters on the world map
  included), so a new chapter can't go silent unnoticed.
- The `.pdx`'s size before and after in the Completion notes.

**Out (do not do):**
- Loading one chapter's assets at a time (1068).
- Any change to the PC's music.

## Implementation steps

1. Gather the cues Act 1 asks for (`@music` in `assets/dialogue/`, the
   battles' `music`, the world maps' `music`).
2. List them in `playdate-music.txt`.
3. Write the test; measure the `.pdx` (`cargo xtask playdate --release`).

## Acceptance criteria

- [ ] No track Act 1 asks for plays as silence on the Playdate.
- [ ] The test fails if one is left out.
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: every cue Act 1 asks for is in the Playdate's music.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
