---
id: "0826"
title: "Options: voices on or off, voice volume, and what a new game asks"
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0043", "0238", "0805"]
nick_input: answer-first
completed:
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

- [ ] Voices off: the dialogue screen queues no voice request; on: it does (Harness test, with 0720; if 0720 isn't done, test `play_voice` directly).
- [ ] Voice volume changes what `app` plays; 0 is silent; both settings survive a restart (tests).
- [ ] Old saved settings load with 0043's defaults (test).
- [ ] The new-game question behaves as the design doc says (Harness test).
- [ ] Voice on hover in the log: Off stops the log's cursor playing voices (Harness test, with 0720).
- [ ] Snapshot of the Options rows.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: settings migration.
- Snapshot / integration: Options rows; the new-game question.

## Completion notes

