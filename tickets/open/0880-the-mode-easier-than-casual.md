---
id: "0880"
title: "The mode easier than Casual"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["0098"]
nick_input: answer-first
completed:
---

# 0880 — The mode easier than Casual

## Context

Nick (2026-11-22) wants a mode below Casual; 0098 decides what it
changes, its name and where it's picked. Modes live in `GameMode`
(`crates/core/src/campaign.rs`); rewind charges by map tier
(`Difficulty`); the Classic → Casual switch is at Preparations. Saves
carry the mode (check the save format's version rules before adding a
variant). The new bot that plays it is 0536.

## Nick input

**Answer first:** 0098. Nothing to check after merge: verified by tests
and 0536's bot bands; what's left to judge in play goes on 1064's
checklist.

## Scope

**In (per 0098):** the new mode in core (a `GameMode` variant or a
separate setting, whichever 0098's answer fits), its rules through
`Command` → `Event` (ADR-0004), its picker and text by key, saves and
old saves loading, the play record saying which mode was played.

**Out (do not do):** the bot (0536); retuning Casual or Classic.

## Implementation steps

1. Core: the mode and its rules, unit and property tests.
2. Content: any data it needs (enemy stat scale, rewind counts).
3. UI: where 0098 says it's picked; the mode shown on the status/
   objective screen.
4. Saves: the variant, a load test of an older save.

## Acceptance criteria

- [ ] Core tests for each rule 0098 names.
- [ ] A save made before this ticket still loads (a test).
- [ ] The picker's view test.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit/property (core); save round-trip; screen view tests.

## Lesser targets

Playdate: the same rules; the picker in its skin (0091's Options rules).
GBA: no target.

## Completion notes

