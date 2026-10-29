---
id: "0026"
title: "Decide: the capital city music"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-28
---

# 0026 — Decide: the capital city music

## Context

Ticket 0020 picked the game's music ([`docs/design/audio.md`](../../docs/design/audio.md)),
but left some places and moments without a cue: "capital city, world map,
camp, shops, victory and defeat stings, game over, level up. Nick hasn't been
asked about these." (open sub-questions). Nick asked (2026-09-29) for each of
these moments to get its own ticket. Level up is not one of them: Nick is
handling the level-up and EXP sounds himself.

This ticket picks **the capital city music**, cue id `capital`. It plays when
the army is in a capital (a large city town node on the world map, built by 1007). `city_first_visit` already covers the *first* visit to a big city; this is the everyday capital track.

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

1. **Which track** plays for `capital`? Options: the candidates on the
   page (each with composer, licence and a one-line description), **reuse a
   cue we already have** (name the closest ones from audio.md's table),
   **no music here**, or **describe your own** (then do another round).
- One track for every capital, or one per capital (per realm)?

## Scope

**In:**
- Find candidates (OpenGameArt, freesound, itch.io free assets, and the
  libraries vetted in 0020). Check each licence on its source page.
  Include **Adventure Map** by controllerhead (CC-BY 3.0,
  https://opengameart.org/node/53699): Nick heard it on 0025's page and
  said it "would be decent as capital city music, or a royal march theme"
  (`audio.md`, *The world map*).
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

- [x] `audio.md` has a `capital` row with a source URL and a licence that
      ADR-0013/0027 allow, or a recorded "no music here" / "reuse `<cue>`".
- [x] The moment is gone from audio.md's *Open sub-questions*.
- [x] Nick's words are quoted in audio.md's appendix.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None (design decision; docs only).

## Completion notes

Nick picked over four rounds of a listening page
(https://claude.ai/artifact/EBQJyYJ43taXowUeU8cyYh), 21 candidates in all:

- `capital` is **Adventures in Adventureland** by Kevin MacLeod (CC-BY 4.0,
  4:21), Nick's "general theme".
- **One track for every capital**, not one per realm.
- Rounds 1–3 (OpenGameArt) found nothing that sounded like a capital. After
  round 2 Nick said a capital should be "bustling and grand" (FF12's
  Rabanastre) or "busy medieval streets" (the Witcher 3's Novigrad); round 4,
  from Kevin MacLeod's catalogue, landed.
- Kept for other tickets, at Nick's request: Minstrel Guild (a tavern or
  bustling streets), Court of the Queen (an upper court), Market on the Sea
  (a seaside village), Enchanted Festival (a feast day), The Bard's Tale (a
  road), Market Day, Town Theme and Town Theme 3. Listed in `audio.md`,
  *The capital city*; 0027 and 0028 now point at them.
- Adventure Map, kept by 0025 as a capital candidate, is now kept as a royal
  march only.
- 1007's music step now says one track for every capital, with
  `city_first_visit` still on the first visit.

- `capital` is added to `RESERVED` in `crates/content/src/audio.rs` (the
  test that every `audio.md` cue is in the manifest), as 0071 did for its
  cues: 1007 imports it and drops it from that list.

Recorded as stated, for Nick to veto: "T as a general theme" is read as
T being `capital` itself.
