---
id: "1241"
title: Play the game's sound on the engine's in-house backend
type: feature
milestone: "Engine: carve"
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["engine#3084"]
nick_input: none
completed:
---

# 1241 — Play the game's sound on the engine's in-house backend

## Context

Engine PR #240 (3031) came in with the pin move of 2026-11-03 (ticket
1086). It adds `shuyi-app`'s `in-house-sound` feature, off by default:
sound output of the engine's own, through CPAL on Windows and a Web
Audio worklet in the browser. On Windows it measured 19 ms from play to
the speaker, against quad-snd's 85. It plays music loops gaplessly,
which quad-snd's native loops don't (the pin move of 2026-11-01, #206).
Past its voice limit it skips a sound with a log line, where quad-snd steals a voice.
Nick signed off on its sounds and music on 2026-11-02.

A game that turns the feature on needs the engine's `alsa-sys` patch in
its root `Cargo.toml`, because CPAL and quad-snd each bring a crate with
`links = "alsa"`. Engine ticket 3084 makes the backend the engine's
default with quad-snd still selectable. Once it lands, the game gets the
backend at a pin move rather than carrying the patch itself.

Engine PR #380 (3089), in the pin since the pin move of 2026-11-13,
fixes a gap in the backend's Windows output (its audio thread now runs
as a "Pro Audio" task) and starts the sounds one frame asks for on the
same sample. It also counts the callbacks that found the device buffer
empty (`Output::dry_callbacks`, `Output::raised_priority`): the check of
the loops below reads that count, so a gap heard in the game's build is
known to be the output's or not.

## Nick input

None. His sign-off on the backend's sound is the engine's, given on
2026-11-02.

## Scope

**In:**
- Move the pin past engine 3084 (or turn `in-house-sound` on with the
  engine's patch, if 3084 changes plan). The game's Windows and web
  builds then play through the backend.
- Check the game's music loops play gaplessly: the title's and a
  battle's tracks, recorded with the engine's sound-parity tools or
  listened to in a build, with the output's dry callbacks counted
  (engine #380) and zero on Windows. Check its sounds, fades and volumes play as
  before.
- Note in the completion notes any game sound that the voice limit now
  skips, which quad-snd would have stolen a voice for.

**Out:**
- The Playdate's sound (its own players, ADR-0084).
- Changing any sound or track.

## Implementation steps

1. Read engine ADR-0075 and engine 3084's completion notes.
2. Move the pin (the `engine-pin` skill), or add the feature and the
   patch.
3. Run the game on Windows and the web build, and check the loops.

## Acceptance criteria

- [ ] The Windows and web builds play through the in-house backend.
- [ ] The title's and a battle's music loop with no gap.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None new: the engine's sound-parity gates hold the backend. A game
  test that builds a backend by name changes with it.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
