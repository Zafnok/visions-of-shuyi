---
id: "0027"
title: "Decide: the camp music"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-28
---

# 0027 — Decide: the camp music

## Context

Ticket 0020 picked the game's music ([`docs/design/audio.md`](../../docs/design/audio.md)),
but left some places and moments without a cue: "capital city, world map,
camp, shops, victory and defeat stings, game over, level up. Nick hasn't been
asked about these." (open sub-questions). Nick asked (2026-09-29) for each of
these moments to get its own ticket. Level up is not one of them: Nick is
handling the level-up and EXP sounds himself.

This ticket picks **the camp music**, cue id `camp`. It plays when
the Camp screen is open (supports and camp events, `supports.md`; built by 1003).

Loops.

Not needed for Chapter 1. Its place is built after Chapter 1, so there is no rush.

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

1. **Which track** plays for `camp`? Options: the candidates on the
   page (each with composer, licence and a one-line description), **reuse a
   cue we already have** (name the closest ones from audio.md's table),
   **no music here**, or **describe your own** (then do another round).
- Should camp reuse `village_home` (the home-base village) instead of a new track?

## Scope

**In:**
- Find candidates (OpenGameArt, freesound, itch.io free assets, and the
  libraries vetted in 0020). Check each licence on its source page.
- Offer the tracks 0026 kept "in our backpocket for other music selection
  tickets" (`audio.md`, *The capital city*). For more, Kevin MacLeod's
  catalogue (incompetech, CC-BY 4.0) landed in 0026 after three rounds
  of OpenGameArt tracks didn't.
- A listening page (an Artifact, like 0020's) with players for every
  candidate and the question above. Use the `ask-nick` skill.
- Record the answer in `docs/design/audio.md`: a row in *Music cues* (cue,
  when it plays, track, composer, source, licence, version to import), Nick's
  words in the appendix, and remove the moment from *Open sub-questions*.

**Out (do not do):**
- Importing or playing the file. Ticket 1003 does that (import with
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

- [x] `audio.md` has a `camp` row with a source URL and a licence that
      ADR-0013/0027 allow, or a recorded "no music here" / "reuse `<cue>`".
- [x] The moment is gone from audio.md's *Open sub-questions*.
- [x] Nick's words are quoted in audio.md's appendix.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None (design decision; docs only).

## Completion notes

One round of a listening page
(https://claude.ai/artifact/TkCSYxm6jv26PQNct8uRv7): six new tracks (A–F),
five kept from 0026 (G, L, N, O, Q) and `village_home` (V). Nick picked
three, B (The Old Tower Inn), C (Woodland Fantasy) and G (The Bard's Tale),
and asked for them to **rotate** with a fade out, a pause and a fade in,
instead of one track picked at random like a skirmish. Recorded in
`docs/design/audio.md` as the `camp` pool (a cue row, a *Camp* section, a
starting value and the appendix); camp is gone from *Open sub-questions*.
1003's step now says to import all three and rotate them.

*the agent's starting rules* (in audio.md's *Starting values*, tunable):

- Between camp tracks: 3 s fade out, 5 s of quiet, 3 s fade in.
- The first track is picked at random each time camp opens, then the order
  goes round the pool, so no track plays twice in a row.
- The full versions are imported, not the loop files, since each track
  plays to its end and fades.
