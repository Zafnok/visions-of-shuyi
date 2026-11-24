---
id: "1324"
title: "Decide: Chapter 1's ambience, particles and sounds"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: blocked
blocked_by: ["0096", "1322", "1323"]
nick_input: decision
completed:
---

# 1324 — Decide: Chapter 1's ambience, particles and sounds

## Context

Nick (2026-11-23) wants the game to have more life, taking from Stardew
Valley's small touches (drifting petals, hit particles, birdsong) without
copying a life sim, "chapter by chapter". His full words are in 1322.

Three palette decisions say *what kinds* of life the game has: **0096**
(moving tiles; drifting petals, leaves, ash, snow), **1322** (ambient
sound: a bed per place and light) and **1323** (hit particles; dust,
splashes and creatures on the map). This ticket picks, for Chapter 1
only, which of them each place gets, so the chapters differ from each
other and each fits its story.

Chapter 1's places: the battle (0803; the green before the chapel, 0741) and the scenes' places (0029, 0732-0734): the letter, the Red Company, after the fight, Veyne Hall. Each place's mood, time of day and
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

- the battle: wind in grass and birdsong by day (Stardew Valley's spring mornings, check), a chapel bell as a passing call, petals or leaves drifting (0096's palette), butterflies that scatter (1323's palette)
- after the fight: the birds gone quiet, a crow or two; or only the wind, so the loss is heard
- Veyne Hall: a hall's hush and a hearth's crackle (the title's crackle is the reference, 0922)
- the letter and the Red Company: the same village bed as the battle, or none so the scene is set apart

1322's answer (2026-11-23): a bed per place, which may follow the
map's light; **no passing calls for now, and never in a battle**
(1344 asks again later). A call in the list above is heard only as
part of a bed's recording, or waits for 1344.
Of the day beds heard for 1322, Nick liked the Summer Meadow (baryy,
freesound 409143) best: the battle's options start from it.

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
and says only what isn't story; 1335 adjusted to the picks.

**Out (do not do):** building anything (1335, and the systems 1315,
1316, 1332, 1334); other chapters; music cues (0071, 0742); changing
the script (dialogue writing is paused, AGENTS.md rule 3).

## Implementation steps

1. Read the palette answers (0096, 1322, 1323) and Chapter 1's beat
   sheet and script.
2. Pick candidate sounds (CC0 / CC-BY 4.0 only, ADR-0013) and the
   palette's effects for each place; build the loops.
3. Ask; record; adjust 1335.

## Acceptance criteria

- [ ] Every place listed above has a recorded pick (or "none"), Nick's
      words quoted, and every sound named with its license.
- [ ] 1335 lists exactly those picks.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

None (a decision).

## Lesser targets

What the Playdate gets follows the palettes' answers (0096, 1322, 1323);
this ticket only notes a place that should differ there.

## Completion notes
