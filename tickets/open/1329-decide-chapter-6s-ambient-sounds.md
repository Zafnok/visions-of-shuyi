---
id: "1329"
title: "Decide: Chapter 6's ambient sounds"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: blocked
blocked_by: ["1322", "0852"]
blocks: ["1064"]
nick_input: decision
completed:
---

# 1329 — Decide: Chapter 6's ambient sounds

## Context

Nick (2026-11-23) wants the game to have more life, taking from Stardew
Valley's small touches (drifting petals, hit particles, birdsong) without
copying a life sim, "chapter by chapter". His full words are in 1322.

**1322** decided the ambient sound palette: a sound bed per place, which
may follow the map's light. This ticket picks, for Chapter 6 only, which
bed each place gets, so the chapters differ from each other and each fits
its story.

Split from the chapter's single ambience ticket (2026-11-26) so the
sounds don't wait for the hit particles' decision (1323): 1329 picks
the sound beds, 1367 the drifting effects, moving tiles and map
touches. One round with Nick may still ask both if both are ready.

Chapter 6's places: The Ashfields (0852): a burnt plain between two armies, with echoes (fire elementals); the skirmishes *The Burnt Mill* and *Brennish Outriders* (1090); and the chapter's scenes. Each place's mood, time of day and
beats are in the chapter's beat sheet and script (the private
repository's `story/` and `content/`, ADR-0117; `cargo shuyi
private-assets`).

The battle is 0852's (open), so this decision waits for
it: the pictures are of the real map.

The skirmishes are 1090's (open). Either order works: if 1090 isn't
done, decide them from 1090's descriptions and say so in the answer;
the build (1340) waits for 1090.

The 00xx block is full (0099), so this decision takes a 13xx number.
Several chapters may be asked in one round with Nick; each ticket's PR
records only its own chapter.

## Nick input

**Decision** (`ask-nick`; expect two rounds). For each place, a
listening page as for 1322: a `frame-png` frame of the place (picture
look, cropped per ADR-0118), its music, and two or three beds plus
"none", with a volume slider per layer. The agent's starting list to
show, **not decided**:

- the battle: wind over a dead plain, the two armies heard far off (drums, horns) as passing calls
- the echoes' own sound (a low roar) while one is on screen, or none
- The Burnt Mill: a wheel creaking in a dry race; Brennish Outriders: hooves on a hill, wind

1322's answer (2026-11-23): a bed per place, which may follow the
map's light; **no passing calls for now, and never in a battle**
(1344 asks again later). A call in the list above is heard only as
part of a bed's recording, or waits for 1344.

Questions, per place: the bed (from 1322's palette), whether it follows
the light, and any story moment that changes it (a turn, a trigger, a
death, a scene's beat). Plus "describe your own". Every comparison to
another game is checked before it is shown.

## Scope

**In:** the listening pages (throwaway, never committed); Nick's picks in
the private repository's `story/design/` (its audio part, through the
`content-change` skill), with a public line in `docs/design/audio.md`
that names this ticket and says only what isn't story; 1340 adjusted
to the picks.

**Out (do not do):** drifting effects, moving tiles, touches and
creatures (1367); building anything (1340, and the player 1332);
other chapters; music cues (0071, 0742); passing calls (1344); changing
the script (dialogue writing is paused, AGENTS.md rule 3).

## Implementation steps

1. Read 1322's answer (`audio.md`, *Ambience*) and Chapter 6's beat
   sheet and script.
2. Pick candidate beds (CC0 / CC-BY 4.0 only, ADR-0013) for each place;
   build the listening pages.
3. Ask; record; adjust 1340.

## Acceptance criteria

- [ ] Every place listed above has a recorded bed (or "none"), Nick's
      words quoted, and every sound named with its license.
- [ ] 1340 lists exactly those picks.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

None (a decision).

## Lesser targets

What the Playdate plays follows 1322's answer; this ticket only notes a
place that should differ there.

## Completion notes
