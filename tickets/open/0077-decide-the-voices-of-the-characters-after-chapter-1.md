---
id: "0077"
title: "Decide: the voices of the characters after Chapter 1"
type: design-decision
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0043", "0750"]
nick_input: decision
completed:
---

# 0077 — Decide: the voices of the characters after Chapter 1

## Context

0043 cast Chapter 1's speakers and Vix, and left later chapters' characters
"their own round when they are written". Chapter 8's script is 0750, the last of
the eight.

## Nick input

**Decision.** `ask-nick` skill. By ear, on a listening page with one portrait and
a two-sentence bio per character. Nick asked for those in 0043's casting round.

## Scope

**In:** every speaking character in `ch02`–`ch08` who has lines worth a voice of
their own; which minor speakers share a voice; the listening page; the cast table
in `docs/design/voices-languages-and-script.md` and their clips in
`assets-src/audio/voice-refs/`.

**Out (do not do):** generating chapters (0757); game code; pitch-shifted
candidates; imitating any real person or character.

## Implementation steps

1. List the speakers in `assets/dialogue/ch02.dlg`–`ch08.dlg` with their line
   counts, and read each one's voice card.
2. For each, design three candidates (`assets-src/audio/voice-refs/README.md`)
   and generate them on both readers. Check each voice's pitch first, so a man's
   part never gets a woman's voice, which happened in 0043.
3. Publish the page. Portraits are bought art: publish them with the page and
   never commit them (ADR-0040). Keep the page's source in `assets-src/audio/`.
4. Iterate as Nick asks, with new voices near the one he liked.
5. Record his answers verbatim, and add the clips and table rows.

## Acceptance criteria

- [ ] Every speaker is cast, or marked as sharing a voice, with Nick's words in the design doc.
- [ ] Each chosen voice has a clip and a README row in `assets-src/audio/voice-refs/`.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes

