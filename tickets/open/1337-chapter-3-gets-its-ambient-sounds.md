---
id: "1337"
title: "Chapter 3 gets its ambient sounds"
type: content
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["1326", "1332"]
blocks: ["1064"]
nick_input: answer-first
completed:
---

# 1337 — Chapter 3 gets its ambient sounds

## Context

1326 records Nick's picks for Chapter 3's places (Greywater Abbey (0849): a causeway to an island abbey, walls, the scriptorium, the reliquary; and the chapter's scenes): each
place's sound bed (no passing calls, 1322), whether it follows the light,
and any story moment that changes it. The player is built by 1332; this
ticket only sets Chapter 3's data.

Split from the chapter's single ambience ticket (2026-11-26): the
drifting effects, moving tiles and map touches are 1372's.

The chapter's battles, maps and scripts are in the private repository's
`content/` (ADR-0117), so the data changes go through the
`content-change` skill: a pull request there first, then this ticket's,
which moves `assets-private.rev`.

## Nick input

**Answer first:** 1326. Nothing to check after merge: verified by
tests; what's left to judge in play goes on 1064's checklist.

## Scope

**In:**
- Chapter 3's battle file(s), skirmishes and scenes' places name the
  bed 1326 picked.
- Story moments 1326 picked, set by the battle's triggers or the
  script's place directives (a new directive only if 1332 made one;
  never a script line, AGENTS.md rule 3).
- New sounds 1326 picked that 1332 didn't import: imported and
  credited (`THIRD_PARTY_ASSETS.md`).

**Out (do not do):** drifting effects, moving tiles, touches and
creatures (1372); the player itself (1332); other chapters; music;
script lines.

## Implementation steps

1. Read 1326's answer and 1332's data format.
2. Edit the private content (`content-change`); import anything new.
3. Run the gates with the private content checked out.

## Acceptance criteria

- [ ] A content test: every Chapter 3 place names the bed 1326
      picked (and none for a place picked "none").
- [ ] A test: the chapter's recorded playthrough replays identically.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Content: the chapter's data loads and names known beds.
- Integration: the battle's start begins its bed (1332's audio fake).

## Lesser targets

The data is the same everywhere; the tiers (1314) decide what plays.
GBA: no target.

## Completion notes
