---
id: "0024"
title: "Decide: the Game Over music"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-12
---

# 0024 — Decide: the Game Over music

## Context

Ticket 0020 picked the game's music ([`docs/design/audio.md`](../../docs/design/audio.md)),
but left some places and moments without a cue: "capital city, world map,
camp, shops, victory and defeat stings, game over, level up. Nick hasn't been
asked about these." (open sub-questions). Nick asked (2026-09-29) for each of
these moments to get its own ticket. Level up is not one of them: Nick is
handling the level-up and EXP sounds himself.

This ticket picks **the Game Over music**, cue id `game_over`. It plays when
the Game Over screen (built by 0801: `Retry chapter` / `Title`) is shown. Today 0807 makes that screen stop the music; this cue replaces that.

**Mostly answered by 0023 (2026-10-12).** Nick picked The Fallen (Jonathan
Shaw, 77 s) for `defeat` "with the music carrying over to game over, so take
the whole piece": it starts at the `DEFEAT` banner and keeps playing on the
Game Over screen, which starts no track of its own (`audio.md`). So there is
no `game_over` track to pick unless Nick changes his mind after hearing it.

What is left: when the 77 s run out and the screen is still open, loop the
piece or stay silent. No listening page is needed for that.

Needed for Chapter 1: its implementation ticket 0809 blocks the playtest (0804).

The rules from 0020 still apply:

- Recorded music by other composers, under CC0, CC-BY 4.0 or CC-BY 3.0
  (audio.md rule 1, ADR-0013, ADR-0027). Nothing that costs money, nothing
  copyleft or non-commercial.
- Avoid composers who register their tracks with YouTube Content ID (0020
  left out Scott Buckley, Alexander Nakarada and Vindsvept for this reason;
  see 0904).
- Prefer looping and no-vocal versions (rule 9); every track is credited
  (rule 3).
- Music comes in tiers (rule 4): nothing from the epic tier here unless Nick
  says so.

## Nick input

**Decision.** Nick listens to 4–6 candidate tracks on a listening page and
picks one, or asks for another round.

## Questions

1. When `defeat` (77 s) ends and the Game Over screen is still open: **loop
   it**, or **play once and stay silent**? (The source says the track loops
   seamlessly.)
2. Only if Nick now wants a separate Game Over track after all: run the
   listening round as first planned.

## Scope

**In:**
- Find candidates (OpenGameArt, freesound, itch.io free assets, and the
  libraries vetted in 0020). Check each licence on its source page.
- A listening page (an Artifact, like 0020's) with players for every
  candidate and the question above. Use the `ask-nick` skill.
- Record the answer in `docs/design/audio.md`: a row in *Music cues* (cue,
  when it plays, track, composer, source, licence, version to import), Nick's
  words in the appendix, and remove the moment from *Open sub-questions*.

**Out (do not do):**
- Importing or playing the file. Ticket 0809 does that (import with
  `assets-src/audio/import.py`, then `assets/audio/audio.ron`,
  `THIRD_PARTY_ASSETS.md` and the credits).
- Level-up or EXP sounds (Nick's own work).
- Other moments: each has its own ticket (0022–0028).

## Implementation steps

1. Read `docs/design/audio.md` and 0020's appendix for the tone Nick liked.
2. Shortlist 4–6 tracks that fit the moment and the licence rules. Note
   version (loop / no vocal) and length.
3. Build the listening page and run the question with `ask-nick`.
4. Record the answer in `audio.md` as above.

## Acceptance criteria

- [x] `audio.md` records loop or once for `defeat` on the Game Over screen
      (or a `game_over` row, if Nick asked for a separate track).
- [x] The moment is gone from audio.md's *Open sub-questions*.
- [x] Nick's words are quoted in audio.md's appendix.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None (design decision; docs only).

## Completion notes

**Answered while building 0809 (2026-10-12), and archived by its PR.**
0023 had left one question: when The Fallen ends and the Game Over screen
is still open, loop or silence. No listening page was needed. Nick picked
"Loop it", then added: "ensure the loop has a 10-15s pause between loops
for defeat/game over, and transition b/w defeat and game over should not
reset music playhead". Recorded in `audio.md` (the `defeat` row, the
paragraph under the table, *Starting values* and the appendix); the open
sub-question is removed. There is no `game_over` cue. 0809 built it.
