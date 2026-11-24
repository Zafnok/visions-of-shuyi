---
id: "1341"
title: "Chapter 7 gets its ambience, particles and sounds"
type: content
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["1330", "1332", "1334", "1315", "1316", "0853", "1090"]
nick_input: answer-first
completed:
---

# 1341 — Chapter 7 gets its ambience, particles and sounds

## Context

1330 records Nick's picks for Chapter 7's places (The Undercroft (0853): catacombs and narrow stairs under Ossary; the skirmishes *Pilgrims' Road* and *The Ossuary Gate* (1090); and the chapter's scenes): each
place's sound bed and passing calls, drifting effect, moving tiles,
touches and creatures, and any story moment that changes them. The
systems are built by 1332 (ambience), 1334 (map touches and creatures),
1315 (moving tiles) and 1316 (drifting effects); this ticket only sets
Chapter 7's data.

The chapter's battles, maps and scripts are in the private repository's
`content/` (ADR-0117), so the data changes go through the
`content-change` skill: a pull request there first, then this ticket's,
which moves `assets-private.rev`.

## Nick input

**Answer first:** 1330. Nothing to check after merge: verified by tests
and `frame-png` strips; what's left to judge in play goes on 1064's
checklist.

## Scope

**In:**
- Chapter 7's battle file(s), skirmishes and scenes' places name the
  ambience, drifting effect and creatures 1330 picked.
- Story moments 1330 picked, set by the battle's triggers or the
  script's place directives (a new directive only if 1332 or 1316 made
  one; never a script line, AGENTS.md rule 3).
- New sounds or effects 1330 picked that the systems' tickets didn't
  import: imported and credited (`THIRD_PARTY_ASSETS.md`).

**Out (do not do):** the systems themselves; other chapters; music;
script lines.

## Implementation steps

1. Read 1330's answer and the systems' data formats (1332, 1334, 1316).
2. Edit the private content (`content-change`); import anything new.
3. `frame-png` strips of each place; run the gates with the private
   content checked out.

## Acceptance criteria

- [ ] A content test: every Chapter 7 place names what 1330 picked
      (and nothing for a place picked "none").
- [ ] A test: the chapter's recorded playthrough replays identically.
- [ ] `frame-png` strips of each place in the PR (cropped, ADR-0118).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Content: the chapter's data loads and names known ambiences and
  effects.
- Integration: the battle's start begins its bed (1332's audio fake).

## Lesser targets

The data is the same everywhere; the tiers (1314) decide what plays.
GBA: no target.

## Completion notes
