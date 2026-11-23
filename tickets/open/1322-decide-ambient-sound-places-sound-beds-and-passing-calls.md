---
id: "1322"
title: "Decide: ambient sound, a place's sound bed and its passing calls"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 1322 — Decide: ambient sound, a place's sound bed and its passing calls

## Context

Nick (2026-11-23):

> "earlier I said I like how stardew handles the little things like the
> petal on screen effect but also I see they have particle effect for
> hits and like ambient sounds like bird chirps and such we need to take
> some inspiration from here for giving the game more life even if its
> not a direct 1:1 copy since this isnt a life sim we need to consider
> like ambience, particles, etc. figure out how we can weave it in to
> give a breath of fresh air lets take it chapter by chapter"

This is one of three "palette" decisions the chapter-by-chapter passes
(1324–1331) pick from: **0096** (moving tiles and drifting effects:
petals, leaves, snow), **1323** (hit particles and small touches on the
map) and this one, **ambient sound**.

Today the game has music cues and sound effects (`docs/design/audio.md`)
and exactly one ambience: the title's campfire crackle (0922,
`title_campfire` in `assets/audio/audio.ron`, looped about 15 dB under
the title song). A battle map, a scene's place (0029's backdrops) and
the world map have music and nothing under it.

The 00xx block is full (0099), so this decision takes a 13xx number
(`tickets/README.md`, *Numbering*).

## Nick input

**Decision** (`ask-nick`, several rounds expected; a listening page, as
for 0922's crackle and the sound-effect auditions in `audio.md`). Every
comparison below is checked against the game before it is shown (memory:
honest game comparisons); drop one that doesn't hold.

1. **What ambience is.** Options to hear over a Chapter 1 battle frame
   with its music:
   - A. **None** (today; Fire Emblem on the Game Boy Advance plays only
     music on the map).
   - B. **A sound bed only**: one quiet loop per place (wind and grass, a
     river, a hall's hush), under the music, like the title's crackle.
   - C. **A bed plus passing calls**: B, plus one-shot sounds at random
     gaps (a bird, a dog, a bell, a gull), like Stardew Valley's
     birdsong in the morning and crickets at night, or Octopath
     Traveler's towns (check both).
   - D. Describe your own.
2. **How loud** under the music: three levels (e.g. −12, −18, −24 dB
   under the track, the range Nick used for the crackle).
3. **When it's heard:** on the battle map only; also in the fight
   (the combat stage); in scenes over their place (0029); on the world
   map and in towns; in camp. Does the bed keep playing when the music
   fades between rounds of the world map's track (`audio.md`)?
4. **Time and weather:** does a map's light (0438: day, dusk, night)
   change its bed (birds by day, crickets by night, as Stardew does)?
5. **Passing calls' pace:** how often (two paces to compare, e.g. one
   every 8–20 s vs 20–45 s), and whether they ever play during a fight.
6. **The Playdate** (1314, `Reduced` tier): no ambience, a bed only, or
   the same. The default is a bed only, or none if the bench says so.

## Scope

**In:** the questions with the listening page (throwaway, never
committed); the answers in `docs/design/audio.md` (a new *Ambience*
section) and a row in `docs/design/README.md`; 1332 adjusted to match
(its options, its default volumes).

**Out (do not do):** building any of it (1332); choosing each chapter's
sounds (1324–1331); drifting effects and moving tiles (0096); hit
particles (1323); changing a music cue.

## Implementation steps

1. Read `docs/design/audio.md` (rules 1–9, *Starting values*), 0922's
   completion notes and `crates/ui/src/screens/title.rs` (`crackle`).
2. Find candidate beds and calls on Freesound and OpenGameArt under
   CC0 or CC-BY 4.0 only (ADR-0013): a meadow, a river, a hall, birds, a
   dog, a bell, crickets. Record each one's author, page and license.
3. Build the listening page: a Chapter 1 battle frame (`frame-png`,
   cropped to the panel and a few tiles, ADR-0118) with the music and
   each option layered, levels as in question 2.
4. Ask Nick; record his words and the rules; mark anything he didn't
   pick as *Claude's starting rule*; adjust 1332.

## Acceptance criteria

- [ ] `docs/design/audio.md` has an *Ambience* section answering
      questions 1–6, with Nick's words quoted.
- [ ] Every sound Nick picked has its author, page and license recorded
      there (to be credited in `THIRD_PARTY_ASSETS.md` by 1332).
- [ ] 1332's scope matches the answer.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

None (a decision).

## Lesser targets

Question 6 decides the Playdate's ambience; GBA: no target (1314).

## Completion notes
