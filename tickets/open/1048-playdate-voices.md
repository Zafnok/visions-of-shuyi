---
id: "1048"
title: "Playdate: Chapter 1's voices on the device"
type: feature
milestone: Post–Chapter 1
model: sonnet-5
effort: medium
status: todo
blocked_by: ["1047", "1043", "0722"]
nick_input: sign-off
completed:
---

# 1048 — Voices on the Playdate

## Context

Nick kept **voices** on the Playdate: "is audio quality and ROM size
enough to keep voices? if so, we can keep" (`docs/design/playdate.md`,
Q5). They fit: about 11 KB a second of compressed speech against the
Playdate's 4 GB of storage.

Voice clips are by dialogue line id (ADR-0046, `docs/voice.md`): OGG,
44.1 kHz, mono, in `voice/<lang>/<scene>/<line id>.ogg` beside the game,
from the private repository (`cargo xtask private-assets`). The dialogue
screen asks for a line's clip through `Ctx::play_voice` (0720); 0722
generates Chapter 1's clips. 1047 converts and plays audio on the
Playdate; 1043 is the Playdate dialogue screen.

## Nick input

**Sign-off:** Nick listens to Chapter 1's opening scene on his Playdate
and says whether the voices sound good enough.

## Scope

**In:**
- Convert the build's language's clips with 1047's converter, in the
  speech format 1047's ADR names (or add one: mono, a lower sample rate
  is fine for speech; record it in that ADR), into the `.pdx`'s own
  `voice/` folder at build time (never committed; the clips come from
  the private checkout, ADR-0040/0046).
- The front end plays `AudioRequest`s for voices with the file player
  (or a sample player per line), stopping the previous line as the PC
  does.
- The voice volume option works.
- Builds without the private clips are silent, as on the PC.
- The `.pdx` size with voices in the PR.

**Out (do not do):**
- Other languages' clips (1049 adds per-language builds).
- Generating or changing any clip.

**Carve:** none.

## Implementation steps

1. Read ADR-0046, `docs/voice.md`, 1047's ADR and front end audio.
2. Convert; play; Simulator check of a scene.
3. Ask Nick to sideload.

## Acceptance criteria

- [ ] Chapter 1's scenes play their voices in the Simulator and on the
      device.
- [ ] The `.pdx` size with voices is in the PR.
- [ ] Nick's comment in the completion notes (or "no comment yet").
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the voice request mapping (fake SDK); conversion of a test clip
  (`crates/app/tests/voice/` has test tones).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
