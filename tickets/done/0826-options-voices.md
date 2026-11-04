---
id: "0826"
title: "Options: voices on or off, voice volume, and what a new game asks"
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0043", "0238", "0805"]
nick_input: answer-first
completed: 2026-11-03
---

# 0826 — Options: voices on or off, voice volume, and what a new game asks

## Context

Nick: AI voices "with options menu to turn it off, maybe even ask upon
starting a new game" (`docs/design/voices-languages-and-script.md`).
0043 settles whether voices start on or off, whether a new game asks, and
the wording. 0238 put `voices_on` and `voice_volume` in `Ctx`; 0805 builds
Options and `Settings`.

**Where they are kept** (0058, `docs/design/options.md`, *Where options are
kept*): both settings are fields of `Settings`, so they belong to the
current player profile (0836) like every option; nothing here needs the
profiles to exist first.

## Nick input

**Answer first:** ticket 0043 (questions 5 and 6). Build exactly the
wording and behaviour in the design doc.

## Scope

**In:** the two settings saved; the Options rows; the new-game question.

**Out (do not do):** the credits line and store text (0907); a voice
language setting (a voice speaks the text's language, 0043); the
auto-advance row (0859); the New Game options menu (0860).

## Implementation steps

1. `Settings` (0805) gains `voices: bool` and `voice_volume: u8` (0–10,
   default 8, *tunable*); `voices` is on until the New Game question
   answers it, so a player who has no answer yet (an old save) hears
   voices. Bump the settings version and migrate. Move 0238's `Ctx` fields
   to read from `Settings`.
2. Options: a **"Voices (AI-generated)"** row (On / Off) and a "Voice
   volume" row beside music and sound volume. Turning voices off stops a
   clip that is playing.
2b. A **Voice on hover in the log: On / Off** row under the voice volume,
   starting On (0729; `options.md`, *Voices in the dialogue log*). Off, the
   dialogue log's cursor plays no voice (0720 step 7).
3. Ask **"Play with AI-generated voices?"** once when a new game starts,
   after the familiarity question (0835), with Yes highlighted; the answer
   sets `Settings::voices` and can be changed in Options afterwards. If no
   `voice/` folder was found, don't ask and hide nothing: the rows stay,
   they just have nothing to play. 0860 later moves the question into a New
   Game options menu.
4. All text through `ctx.text` if 0233 has landed; otherwise literals,
   and add a line to 0243's list.

## Acceptance criteria

- [x] Voices off: the dialogue screen queues no voice request; on: it does (Harness test, with 0720; if 0720 isn't done, test `play_voice` directly).
- [x] Voice volume changes what `app` plays; 0 is silent; both settings survive a restart (tests).
- [x] Old saved settings load with 0043's defaults (test).
- [x] The new-game question behaves as the design doc says (Harness test).
- [ ] Voice on hover in the log: Off stops the log's cursor playing voices (Harness test, with 0720). *Not met here: 0720 isn't done, so the log plays no voice yet; the setting and its row exist, and 0720 step 7 now names it.*
- [x] Snapshot of the Options rows.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: settings migration.
- Snapshot / integration: Options rows; the new-game question.

## Completion notes

- **Engine first** (`engine-change`): the voice switch and volume moved from
  `Ctx` fields into the engine's `Settings` (engine ticket 3208,
  `Zafnok/shuyi-engine#253`), which also stops the clip playing when voices
  go off, adds `Ctx::has_voices` and the `OptionRow::voices` /
  `voice_volume` rows. This PR moves the pin to it.
- **Settings** version 2: `voices`, `voice_volume` and the game's
  `log_voice`; a version 1 save loads with voices on, at 80, and the log's
  hover voice on.
- **Deviation:** the voice volume is 0–100 (default 80) like the music and
  sound, not the step's 0–10 (default 8): the same slider and number box as
  the rows beside it, and the same loudness by default.
- **Options:** *Voices (AI-generated)* On/Off, *Voice volume*, *Voice on
  hover in the log* On/Off, under Sound volume. Shown with or without a
  `voice/` folder. Left out on the Playdate, which plays no voice yet
  (ticket 1048 now says to bring them back).
- **New Game:** "Play with AI-generated voices?" after the familiarity
  question, `Yes` highlighted, only when voice clips were found; the answer
  is the Voices setting, saved in the profile. Cancel goes back to the
  familiarity question, and the lead screen's Cancel comes back to it.
- `docs/design/options.md` still says the log's row "isn't shown" until the
  game has voices; this ticket's step 3 says the rows stay, so they do.

- Engine friction: estimate 150 production lines (judged by 0844's
  Intro setting and 0835's familiarity screen; written once the engine
  side was clear, not before the first line), measured 136 (0.9×), under
  2×. The engine side (3208) is what kept it small: without it the game
  would have copied its settings into `Ctx` every frame.

**The agent's starting rules** (for the Act 1 playtest, 1064):

- Cancel on the voices question goes back to the familiarity question
  (as Cancel does on every New Game question).
- The voices question looks like "Save your progress?" (a question over
  `Yes` / `No`).
- On the Playdate the three voice rows are hidden until it plays voices.

