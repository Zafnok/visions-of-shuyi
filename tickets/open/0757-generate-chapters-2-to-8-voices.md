---
id: "0757"
title: Generate Chapters 2 to 8's voices
type: content
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0077", "0722", "0754", "0750", "0762"]
nick_input: none
completed:
---

# 0757 — Generate Chapters 2 to 8's voices

## Context

0722 voices Chapter 1 and 0077 casts the characters after it. This does the rest
of the scripts, with 0722's method.

**Waits for 0762** (2026-11-07): a friend's playtest heard long, unnatural
pauses at ellipses. 0762 changes the text the tool sends and adds a pause and
pace audit after `generate`; these chapters (about 217 of the 227 lines with
`...`) are made with both, and none of their clips is left flagged.

**After the writer's rewrite** (Nick, 2026-11-08; Lane J of *Act 1
critical path*): a human writer is being hired (0766) and may reword
every line. A line's id is a hash of its words, so a clip or a
translation made before the rewrite is redone after it. Don't start
this ticket while 0766 is open without asking Nick.

## Nick input

None. What is left to judge by playing goes on the Act 1 playtest's checklist
(1064), if this lands before it, or the next playtest.

## Scope

**In:** clips and manifest for `ch02`–`ch08` in the private repository; `say.ron`
entries for their names; the pin.

**Out (do not do):** rewording lines; Japanese and Chinese (0755, 0756 cover
Chapter 1; write a ticket for more); chapters after 8.

## Implementation steps

0. Write voice directions into each chapter's script first, as 0758 did for
   Chapter 1 (ADR-0109): adding one later makes that line again.
1. Follow 0722's steps 1–5, one chapter per commit in the private repository. A
   rewrite of a script after this makes its lines stale, and `cargo xtask voice
   generate` redoes only those.
2. Check the private repository's size after each chapter, and split this ticket
   if the audio makes a chapter's commit unreasonable.

## Acceptance criteria

- [ ] `cargo xtask voice status` reports no missing or stale clip for `ch02`–`ch08`, besides lines listed as left silent.
- [ ] Every clip is OGG Vorbis, 44.1 kHz, mono, loudness-matched.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None new.

## Completion notes

