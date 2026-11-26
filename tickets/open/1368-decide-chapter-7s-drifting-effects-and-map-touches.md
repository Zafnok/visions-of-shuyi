---
id: "1368"
title: "Decide: Chapter 7's drifting effects and map touches"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: blocked
blocked_by: ["0096", "1323", "0853"]
blocks: ["1064"]
nick_input: decision
completed:
---

# 1368 — Decide: Chapter 7's drifting effects and map touches

## Context

Nick (2026-11-23) wants the game to have more life, taking from Stardew
Valley's small touches (drifting petals, hit particles, birdsong) without
copying a life sim, "chapter by chapter". His full words are in 1322.

Two palette decisions say *what kinds* of life the battle map has:
**0096** (moving tiles; drifting petals, leaves, ash, snow, fireflies;
battle map only, `look-and-feel.md`) and **1323** (hit particles; dust,
splashes and creatures on the map). This ticket picks, for Chapter 7's
battle maps only, which of them each map gets and how dense, so the
chapters differ from each other and each fits its story.

Split from the chapter's single ambience ticket (2026-11-26) so the
sounds don't wait for the hit particles' decision (1323): 1330 picks
the sound beds, 1368 the drifting effects, moving tiles and map
touches. One round with Nick may still ask both if both are ready.

Chapter 7's maps: The Undercroft (0853): catacombs and narrow stairs under Ossary; the skirmishes *Pilgrims' Road* and *The Ossuary Gate* (1090). Each map's mood, time of day and
beats are in the chapter's beat sheet and battle files (the private
repository's `story/` and `content/`, ADR-0117; `cargo shuyi
private-assets`).

The battle is 0853's (open), so this decision waits for
it: the pictures are of the real map.

The skirmishes are 1090's (open). Either order works: if 1090 isn't
done, decide them from 1090's descriptions and say so in the answer;
the build (1376) waits for 1090.

The 00xx block is full (0099), so this decision takes a 13xx number.
Several chapters may be asked in one round with Nick; each ticket's PR
records only its own chapter.

## Nick input

**Decision** (`ask-nick`; expect two rounds). For each map, show a
moving loop of the real frame (`frame-png` frames in the picture look,
cropped per ADR-0118; the map's bed from 1330 under it if picked)
of two or three options, plus "none". The agent's starting list to show,
**not decided**:

- dust motes in torchlight, torches flickering (0096's moving tiles), no creatures, or rats that scurry off (1323)

Questions, per map: the drifting effect and its density, moving tiles
(0096's palette), the touches and creatures (1323's), and any story
moment that changes them mid-battle (a turn, a trigger, a death). Plus
"describe your own". Every comparison to another game is checked before
it is shown.

## Scope

**In:** the loops (throwaway, never committed); Nick's picks in the
private repository's `story/design/` (its look part, through the
`content-change` skill), with a public line in `look-and-feel.md` that
names this ticket and says only what isn't story; 1376 adjusted to
the picks.

**Out (do not do):** sound beds (1330); talk scenes and the world
map (no drifting effects there, `look-and-feel.md`); building anything
(1376, and the systems 1315, 1316, 1334); the fight's hit bursts
(1333, the same for every chapter); other chapters; changing the script
(AGENTS.md rule 3).

## Implementation steps

1. Read the palette answers (0096, 1323) and Chapter 7's beat sheet and
   battle files.
2. Pick the palette's effects for each map; build the loops.
3. Ask; record; adjust 1376.

## Acceptance criteria

- [ ] Every map listed above has a recorded pick (or "none"), Nick's
      words quoted.
- [ ] 1376 lists exactly those picks.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

None (a decision).

## Lesser targets

The Playdate shows the map still (1314); this ticket only notes a map
that should differ there.

## Completion notes
