---
id: "1326"
title: "Decide: Chapter 3's ambience, particles and sounds"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: blocked
blocked_by: ["0096", "1322", "1323"]
blocks: ["1064"]
nick_input: decision
completed:
---

# 1326 — Decide: Chapter 3's ambience, particles and sounds

## Context

Nick (2026-11-23) wants the game to have more life, taking from Stardew
Valley's small touches (drifting petals, hit particles, birdsong) without
copying a life sim, "chapter by chapter". His full words are in 1322.

Three palette decisions say *what kinds* of life the game has: **0096**
(moving tiles; drifting petals, leaves, ash, snow), **1322** (ambient
sound: a bed per place and light) and **1323** (hit particles; dust,
splashes and creatures on the map). This ticket picks, for Chapter 3
only, which of them each place gets, so the chapters differ from each
other and each fits its story.

Chapter 3's places: Greywater Abbey (0849): a causeway to an island abbey, walls, the scriptorium, the reliquary; and the chapter's scenes. Each place's mood, time of day and
beats are in the chapter's beat sheet and script (the private
repository's `story/` and `content/`, ADR-0117; `cargo shuyi
private-assets`).

The 00xx block is full (0099), so this decision takes a 13xx number.
Several chapters may be asked in one round with Nick; each ticket's PR
records only its own chapter.

## Nick input

**Decision** (`ask-nick`; expect two rounds). For each place, show a
moving loop of the real frame (`frame-png` frames in both looks, cropped
per ADR-0118, with the sounds layered: a listening page as for 1322)
of two or three options, plus "none". The agent's starting list to show,
**not decided**:

- the battle: sea and wind around the causeway, gulls as passing calls, the abbey's bell
- inside (the scriptorium, the reliquary): stillness, dust motes in the light, a candle's flicker
- spray or mist drifting over the causeway (0096's palette), gulls that lift off as units come near (1323)
- the scenes: the abbey's bell once at a beat the script marks, or the bed only

1322's answer (2026-11-23): a bed per place, which may follow the
map's light; **no passing calls for now, and never in a battle**
(1344 asks again later). A call in the list above is heard only as
part of a bed's recording, or waits for 1344.

Questions, per place: the bed (from 1322's palette), the
drifting effect and moving tiles (0096's), the touches and creatures
(1323's), and any story moment that changes them mid-battle (a turn, a
trigger, a death). Plus "describe your own". Every comparison to another
game is checked before it is shown.

## Scope

**In:** the loops (throwaway, never committed); Nick's picks in the
private repository's `story/design/` (its audio and look parts, through
the `content-change` skill), with a public line in
`docs/design/audio.md` and `look-and-feel.md` that names this ticket
and says only what isn't story; 1337 adjusted to the picks.

**Out (do not do):** building anything (1337, and the systems 1315,
1316, 1332, 1334); other chapters; music cues (0071, 0742); changing
the script (dialogue writing is paused, AGENTS.md rule 3).

## Implementation steps

1. Read the palette answers (0096, 1322, 1323) and Chapter 3's beat
   sheet and script.
2. Pick candidate sounds (CC0 / CC-BY 4.0 only, ADR-0013) and the
   palette's effects for each place; build the loops.
3. Ask; record; adjust 1337.

## Acceptance criteria

- [ ] Every place listed above has a recorded pick (or "none"), Nick's
      words quoted, and every sound named with its license.
- [ ] 1337 lists exactly those picks.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

None (a decision).

## Lesser targets

What the Playdate gets follows the palettes' answers (0096, 1322, 1323);
this ticket only notes a place that should differ there.

## Completion notes
