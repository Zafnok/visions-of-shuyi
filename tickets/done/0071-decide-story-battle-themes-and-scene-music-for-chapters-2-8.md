---
id: "0071"
title: "Decide: story-battle themes and scene music for Chapters 2–8"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-28
---

# 0071 — Decide: story-battle themes and scene music for Chapters 2–8

## Context

Every battle file names its music (`music: Cue("battle_bright")` in
`assets/battles/ch01.ron`, 0807) and every scene can set its mood with
`@music` (0710). Chapter 1's picks are in
[`docs/design/audio.md`](../../docs/design/audio.md) (*Chapter 1's
cues*): Nick chose the battle theme ("A"), approved one scene pick
("Fits") and wants to "experiment" with three others (0742).

Act 1's other seven story battles (`docs/story/outline.md`) need theirs:
Kell's Ford (a burning village), Greywater (an island abbey), Veyne Hall
(the lead's childhood home, Dace as boss), the Envoy's House (holding a
harbour house at night), the Ashfields (crossing a burnt plain between
two armies), the Undercroft (catacombs), and Saltmere Docks (the finale,
Hollis's death). Skirmishes already use the `skirmish` pool (1008);
world map, capital, camp and shop music are 0025–0028.

## Nick input

**Decision**, with the `ask-nick` skill, by ear: a listening page (an
Artifact, like 0020's) with 3–5 candidates for each battle and for each
group of scenes, plus "reuse a cue we already have" and "describe your
own". Expect several rounds; group the questions by chapter.

## Questions to ask

1. **Each story battle's theme** (Chapters 2–8): one per battle, or one
   theme shared by several (FE GBA reuses its map themes across
   chapters; Three Houses gives each big battle its own)? Which track?
2. **The finale (Chapter 8)**: its own theme, and a different track for
   the moment Hollis dies and the ship sails?
3. **Scene moods** that `audio.md` has no cue for yet: the confession
   (Chapter 5's night scene), the king's closing scene (Varenhall), the
   Vigil (Ossary, Crane). Reuse or new?

## Scope

**In:**
- Candidates from the sources and licence rules in `audio.md` (*The
  rules*): CC0 / CC-BY only, no Content ID composers, looping and
  no-vocal versions, every track credited.
- The listening page, the questions, and recording the answers in
  `audio.md`: a *Chapters 2–8's cues* table like Chapter 1's (moment,
  cue, track, decided), new rows in *Music cues*, Nick's words in the
  appendix.

**Out (do not do):**
- Importing the tracks or writing `music:` and `@music` into data: each
  chapter ticket does its own (0848–0854; import with
  `assets-src/audio/import.py`, then `assets/audio/audio.ron`,
  `THIRD_PARTY_ASSETS.md` and the credits).
- Chapter 1's music (0742), places' music (0025–0028).

## Implementation steps

1. Read `audio.md` and the outline; list the moments per chapter.
2. Shortlist candidates per moment; check each licence on its source
   page.
3. Build the listening page; ask (`ask-nick`); record the answers.

## Acceptance criteria

- [x] Every Act 1 story battle (Chapters 2–8) has a theme recorded in
      `audio.md`, with Nick's words.
- [x] The scene moods the scripts will need have a cue (new or reused).
- [x] `cargo xtask ticket-lint` passes.

## Tests required

- None (decision).

## Completion notes

Three rounds on a listening page
(https://claude.ai/artifact/PYL5bgcLWQx3C2CXnM5V3V): 34 new candidates in
round 1, 11 more for Veyne Hall and the Envoy's House in round 2, 8 for
Ossary in round 3, with the cues we already have beside them. Every
licence was checked on its OpenGameArt page. Recorded in `audio.md`
(*Music cues*, the new *Chapters 2–8's cues*, and Nick's words in the
appendix).

- Battles: Kell's Ford `battle_desperate` (Determined Pursuit), Greywater
  `battle_church_2`, Veyne Hall `battle_rival` (A Slave To No One), the
  Envoy's House `battle_night_siege` (Battle Music, Peter Eastman), the
  Ashfields `battle_war` (Laments of the War), the Undercroft
  `battle_church_3`, Saltmere Docks `battle_finale` (Heroic Demise). One
  theme per battle; the epic tier stays unused in Act 1.
- Scenes: the confession `scene_confide` (Soliloquy), Hollis dies
  `scene_loss` then the ship sails `scene_departure` (A Long Way From
  Home), the king's closing scene `scene_lamplight` (Snowfall), Ossary's
  streets `holy_city` (Wendale Abbey), the empty dormitory `scene_dread`
  (Solemn (Opening)), Crane `talk_antagonist`.
- Kept for later: Ringing of Steel, Dark Descent.
- Nine new cues to import; each chapter ticket (0848–0854) imports its
  own, as the ticket's *Out* says. Two tracks list voices (Wendale Abbey,
  Solemn (Opening)): `audio.md` asks for an ear check for words on import.

*Claude's starting rules* (tunable, for Nick to veto):
- If Hollis already fell in Classic mode, the rewritten farewell plays the
  same two cues (`scene_loss`, then `scene_departure`).
- Every other Chapter 2–8 scene picks from the existing mood cues in its
  script ticket, as Chapter 1's did, and is heard in the Act 1 playtest.

Gates: docs only; `ticket-lint` and `typos` run (no code changed).
