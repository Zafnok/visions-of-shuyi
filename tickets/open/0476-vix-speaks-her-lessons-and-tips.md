---
id: "0476"
title: Vix speaks her lessons and tips
type: feature
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0043", "0238", "0475", "0721", "0720", "0826"]
nick_input: none
completed:
---

# 0476 — Vix speaks her lessons and tips

## Context

Nick, 2026-11-01: "I think we can voice Vix too." (0043,
`docs/design/voices-languages-and-script.md`, *Voices*.) Her voice is `guide`
in the cast table, read by Chatterbox. 0475 leaves her lines free of keys. Clips
belong to dialogue line ids (ADR-0046, ADR-0045 §3), and lessons and tips have
none yet.

## Nick input

None. Verified by tests. What is left to judge by playing goes on the Act 1
playtest's checklist (1064).

## Scope

**In:** line ids for lessons' and tips' `text`; the lesson and tip boxes asking
for her clip and stopping it; `cargo xtask voice` covering them; her clips for
every lesson and tip in the private repository.

**Out (do not do):** voicing `items`, `how` or the *Guides* list's headings
(silent, Nick); replaying her in the *Guides* list unless a line is opened
there as a box already.

## Implementation steps

1. Give lessons and tips line ids the way ADR-0045 §3 does for script lines, so
   rewording gives a new id. Scene id in the voice folder: `vix`.
2. When a box opens, call `play_voice`; closing it, or another box opening,
   stops it. With voices off nothing is asked for (0238's `Ctx::play_voice`).
3. Extend 0721's `status` and `generate` to read lessons and tips, with speaker
   `guide`.
4. Generate her clips (`cargo xtask voice generate --scene vix`), check them by
   transcribing as 0722 does, commit to the private repository, and pin.

## Acceptance criteria

- [ ] Harness test: a lesson opening with a clip in the test manifest queues one `PlayVoice`; closing it queues `StopVoice`.
- [ ] With voices off, no voice request is queued (test).
- [ ] `items` and `how` never ask for a voice (test).
- [ ] `cargo xtask voice status` reports nothing missing for `vix`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: Harness tests on `audio_requests()`.

## Completion notes

