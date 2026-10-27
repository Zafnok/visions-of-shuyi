---
id: "0025"
title: "Decide: the world map music"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-26
---

# 0025 — Decide: the world map music

## Context

Ticket 0020 picked the game's music ([`docs/design/audio.md`](../../docs/design/audio.md)),
but left some places and moments without a cue: "capital city, world map,
camp, shops, victory and defeat stings, game over, level up. Nick hasn't been
asked about these." (open sub-questions). Nick asked (2026-09-29) for each of
these moments to get its own ticket. Level up is not one of them: Nick is
handling the level-up and EXP sounds himself.

This ticket picks **the world map music**, cue id `world_map`. It plays when
the army is on the world map (0008, `world-structure.md`; built by 1007), outside towns and battles.

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

1. **Which track** plays for `world_map`? Options: the candidates on the
   page (each with composer, licence and a one-line description), **reuse a
   cue we already have** (name the closest ones from audio.md's table),
   **no music here**, or **describe your own** (then do another round).
- One world-map track for the whole game, or one per act?

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
- Importing or playing the file. Ticket 1007 does that (import with
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

- [x] `audio.md` has a `world_map` row with a source URL and a licence that
      ADR-0013/0027 allow, or a recorded "no music here" / "reuse `<cue>`".
- [x] The moment is gone from audio.md's *Open sub-questions*.
- [x] Nick's words are quoted in audio.md's appendix.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None (design decision; docs only).

## Completion notes

Two rounds on a listening page
(https://claude.ai/artifact/VLAU7eYPcqx7pWRmSsVLzw), 13 candidates, all
licences checked on their OpenGameArt pages. Recorded in `audio.md`
(*Music cues* and *The world map*), with Nick's words in the appendix.

- `world_map` (Act 1): The World At Your Door by Jonathan Shaw, CC-BY 3.0.
- Question 2 answered as "a new track per act, or per story arc", so the
  ticket picked more than one cue: `world_map_war` (Act 3, The Tread of
  War, Jonathan Shaw) and `world_map_somber` (reserved for a somber
  stretch, World Map by Scott Elliott). Act 2's eastern map waits for the
  wuxia round (already an open sub-question).
- Nick asked that the map's track fade out, stay quiet for a while and fade
  back in instead of looping back to back.
- Seven more tracks are kept for places and moments Nick named while
  listening (a crystal cave and its windy and rainy parts, an enemy
  mobilizing, very sad scenes, a beginner area, an airship or mount, a
  capital or royal march).
- Downstream: 0026 now lists Adventure Map as a candidate; 1007's music
  step says the map's cue is per-act data and describes the fade and
  pause.

- The quiet between two rounds of the map's track is 30 s (Nick, after
  the page: "probably more like 30s pause or maybe longer").

*Claude's starting rules* (Nick didn't give these; tunable):
- The map's track fades out over 3 s and back in over 3 s.

Gates: docs only; `ticket-lint` and `typos` run (no code changed).
