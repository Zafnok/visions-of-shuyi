---
id: "1324"
title: "Decide: Chapter 1's ambient sounds"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["1322"]
blocks: ["1064"]
nick_input: decision
completed: 2026-11-26
---

# 1324 — Decide: Chapter 1's ambient sounds

## Context

Nick (2026-11-23) wants the game to have more life, taking from Stardew
Valley's small touches (drifting petals, hit particles, birdsong) without
copying a life sim, "chapter by chapter". His full words are in 1322.

**1322** decided the ambient sound palette: a sound bed per place, which
may follow the map's light. This ticket picks, for Chapter 1 only, which
bed each place gets, so the chapters differ from each other and each fits
its story.

Split from the chapter's single ambience ticket (2026-11-26) so the
sounds don't wait for the hit particles' decision (1323): 1324 picks
the sound beds, 1362 the drifting effects, moving tiles and map
touches. One round with Nick may still ask both if both are ready.

Chapter 1's places: the battle (0803; the green before the chapel, 0741) and the scenes' places (0029, 0732-0734): the letter, the Red Company, after the fight, Veyne Hall. Each place's mood, time of day and
beats are in the chapter's beat sheet and script (the private
repository's `story/` and `content/`, ADR-0117; `cargo shuyi
private-assets`).

The 00xx block is full (0099), so this decision takes a 13xx number.
Several chapters may be asked in one round with Nick; each ticket's PR
records only its own chapter.

## Nick input

**Decision** (`ask-nick`; expect two rounds). For each place, a
listening page as for 1322: a `frame-png` frame of the place (picture
look, cropped per ADR-0118), its music, and two or three beds plus
"none", with a volume slider per layer. The agent's starting list to
show, **not decided**:

- the battle: wind in grass and birdsong by day (Stardew Valley's spring mornings, check), a chapel bell as a passing call
- after the fight: the birds gone quiet, a crow or two; or only the wind, so the loss is heard
- Veyne Hall: a hall's hush and a hearth's crackle (the title's crackle is the reference, 0922)
- the letter and the Red Company: the same village bed as the battle, or none so the scene is set apart

1322's answer (2026-11-23): a bed per place, which may follow the
map's light; **no passing calls for now, and never in a battle**
(1344 asks again later). A call in the list above is heard only as
part of a bed's recording, or waits for 1344.
Of the day beds heard for 1322, Nick liked the Summer Meadow (baryy,
freesound 409143) best: the battle's options start from it.

Questions, per place: the bed (from 1322's palette), whether it follows
the light, and any story moment that changes it (a turn, a trigger, a
death, a scene's beat). Plus "describe your own". Every comparison to
another game is checked before it is shown.

## Scope

**In:** the listening pages (throwaway, never committed); Nick's picks in
the private repository's `story/design/` (its audio part, through the
`content-change` skill), with a public line in `docs/design/audio.md`
that names this ticket and says only what isn't story; 1335 adjusted
to the picks.

**Out (do not do):** drifting effects, moving tiles, touches and
creatures (1362); building anything (1335, and the player 1332);
other chapters; music cues (0071, 0742); passing calls (1344); changing
the script (dialogue writing is paused, AGENTS.md rule 3).

## Implementation steps

1. Read 1322's answer (`audio.md`, *Ambience*) and Chapter 1's beat
   sheet and script.
2. Pick candidate beds (CC0 / CC-BY 4.0 only, ADR-0013) for each place;
   build the listening pages.
3. Ask; record; adjust 1335.

## Acceptance criteria

- [x] Every place listed above has a recorded bed (or "none"), Nick's
      words quoted, and every sound named with its license.
- [x] 1335 lists exactly those picks.
- [x] `cargo xtask ticket-lint` passes.

## Tests required

None (a decision).

## Lesser targets

What the Playdate plays follows 1322's answer; this ticket only notes a
place that should differ there.

## Completion notes

Asked in one round of a listening page
(https://claude.ai/artifact/UcFuQDHFE19pg1qLxWYycr; throwaway, never
committed: it shows bought art). It had the five places, each with its
`frame-png` frame in the picture look (the battle as a moving loop of its
river and trees) and its music, the beds split into layers with a slider
each. Nick set every mix himself and sent screenshots of the sliders.
Recorded in the private `story/design/audio.md`, *Chapter 1's ambience*
(picks per place, his words), and publicly in `docs/design/audio.md`,
*Chapter 1's beds* (the sounds, licenses and the rules that aren't story).

- **Deviation from the starting list:** Nick asked mid-round to match
  the sound to what moves on the map (river, swaying trees, critters),
  so the battle's options were built from layers. No chapel bell or
  passing call in the battle (1322).
- **The 1322 Summer Meadow is replaced.** Nick heard its insect buzz as
  "a low beep" and rejected even a filtered copy. A spectrum scan found
  steady tones at 145, 215 and 290 Hz. Six other recordings were scored
  frame by frame for buzz, and the Hungary meadow (Alcappuccino 416206)
  replaced it. The 1322 river (BurghRecords) was also dropped, because
  its birds moved with its slider. 1335 swaps both in the game.
- **A wood-stove recording** (Yulathh 767875, CC-BY 4.0) was dropped
  for a hum before Nick heard it.
- **Follow-up tickets:**
  - 1383, an `@ambience` directive, needed by 1335 for the two scenes
    that play over the battle map with their own beds.
  - 1384, a fire in the yard and a candle in the study, so the picture
    matches the crackle (Nick: don't block on it).
  - 1385, the chapel bell in the yard scene (Nick: don't block on it).
- **Claude's choices (technical):** 1335 imports one premixed loop per
  place, so 1332's format stays as it is.
- **Recorded on Nick's "sure", to confirm in 1385:** the dawn chorus stops
  when the bell rings.
- **Open:** a river that gets louder near the camera ("we might revisit
  if we get spatial aware audio mixing").
- **Lesser targets:** none differs. The Playdate plays no bed unless the
  bench allows (1322).
