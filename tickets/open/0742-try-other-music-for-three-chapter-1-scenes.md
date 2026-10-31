---
id: "0742"
title: "Try other music for three Chapter 1 scenes (the letter, the Red Company, after the fight)"
type: content
milestone: M6 Story & dialogue
model: reasoning
effort: medium
status: todo
blocked_by: ["0737"]
nick_input: decision
completed:
---

# 0742 — Try other music for three Chapter 1 scenes

## Context

`docs/design/audio.md`, *Chapter 1's cues*: four scene tracks were
The agent's picks (0803) that Nick wanted to hear with the scenes'
backgrounds. Chapter 1 playtest round 3 (0804, 2026-10-16), with the
places in (0732–0734):

| Scene | Track now | Nick |
| ----- | --------- | ---- |
| The letter (`ch01_intro`, the yard at dawn) | `village_home`, "Squirrel Village" (SoManyWhales) | "we can experiment some more. the music is ok but not perfect." |
| The Red Company (`ch01_prebattle`) | `dungeon_tense`, "Dark Forest Theme" (cynicmusic) | "we can experiment more, it's ok." |
| After the fight (`ch01_victory`) | `scene_sad`, "Peractorum" (César da Rocha) | "we can experiment" |
| Veyne Hall (`ch01_tbc`) | `talk_antagonist`, "Classical Murder" (Alexandr Zhelanov) | "Fits" (approved, recorded in `audio.md` by 0804's round 3 PR) |

Since the script is being rewritten (0737), the scenes' moods may shift,
so this ticket waits for the rewrite (0737) and is heard against the
new scenes.

The rules from 0020 apply (`audio.md`): free music under CC0, CC-BY 4.0
or CC-BY 3.0 (ADR-0013, ADR-0027), credited in `THIRD_PARTY_ASSETS.md`,
nothing that costs money.

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

**Decision.** A listening page (Artifact) per scene: 3–4 candidates each,
the current track among them, each with author, licence and a line on why
it fits. Best is a short recording of the scene playing with each track.
Nick picks, or asks for another round.

## Scope

**In:** finding candidates (the game's existing cues first:
`talk_calm`, `village`, `side_quest`, `scene_tragic`, `graveyard_desert`;
then new free tracks), the listening page, adding picked tracks
(`music/`, the cue table, credits), changing the `@music` lines in
`assets/dialogue/ch01.dlg`.

**Out (do not do):** the battle's music (Nick's pick, `battle_bright`);
Veyne Hall's track.

## Implementation steps

1. List candidates per scene with licences.
2. Build the listening page; ask Nick.
3. Add the picked tracks and credits; change the `@music` lines; record
   the picks in `audio.md`.

## Acceptance criteria

- [ ] Nick picked a track for each of the three scenes (or kept one).
- [ ] New tracks are credited in `THIRD_PARTY_ASSETS.md` with allowed
      licences; `cargo deny` / the asset checks pass.
- [ ] `audio.md` records the picks as Nick's.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Content validation: every `@music` cue resolves to a track.

## Completion notes

*(Filled in by the session that completes the ticket.)*
