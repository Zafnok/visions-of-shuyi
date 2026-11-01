---
id: "0755"
title: Japanese voices for Chapter 1
type: content
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0719", "0722"]
nick_input: decision
completed:
---

# 0755 — Japanese voices for Chapter 1

## Context

Nick chose "Japanese voices too" in 0043
(`docs/design/voices-languages-and-script.md`, *Voices*). The tools speak
Japanese. Nobody here can check spoken Japanese, which Nick accepted when
choosing. 0719 makes Chapter 1's Japanese text, and 0722 the English voices.

## Nick input

**Decision:** one listening round before generating: each Chapter 1 speaker's
existing voice reading three Japanese lines, on both readers. He says who sounds
wrong, and those get newly designed voices (no pitch shifting). He judges the
voice only.

## Scope

**In:** the listening round; `voice/ja/` clips, manifest and cast in the private
repository; a pin.

**Out (do not do):** Chinese (0756); later chapters; a voice language setting (a
voice speaks the text's language).

## Implementation steps

1. Make the round's page from three lines per speaker. Keep its source in
   `assets-src/audio/`.
2. Record any changed voices as new clips in `assets-src/audio/voice-refs/`, as
   `<id>.ja.flac`, and add them to the design doc's cast table.
3. `cargo xtask voice generate --lang ja`. Transcribe each clip with a Japanese
   speech-to-text model and compare with `spoken`; list mismatches. Regenerate
   bad ones; a line that fails three times stays silent and is listed in the
   notes.
4. Commit to the private repository, pin, and add the game's language pick to
   the voice manifest's read (ADR-0046: only `en` is read until the player can
   pick a language).

## Acceptance criteria

- [ ] Nick's answers are in the design doc, verbatim.
- [ ] `cargo xtask voice status --lang ja` reports nothing missing for `ch01`, besides lines listed as left silent.
- [ ] In Japanese the game plays `voice/ja/` and in English `voice/en/` (test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the manifest read follows the language.

## Completion notes

