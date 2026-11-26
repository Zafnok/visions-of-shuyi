---
id: "1376"
title: "Chapter 7's maps get their drifting effects and touches"
type: content
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["1368", "1334", "1315", "1316", "0853", "1090"]
blocks: ["1064"]
nick_input: answer-first
completed:
---

# 1376 — Chapter 7's maps get their drifting effects and touches

## Context

1368 records Nick's picks for Chapter 7's battle maps (The Undercroft (0853): catacombs and narrow stairs under Ossary; the skirmishes *Pilgrims' Road* and *The Ossuary Gate* (1090)): each
map's drifting effect and density, moving tiles, touches and creatures,
and any story moment that changes them. The systems are built by 1334
(map touches and creatures), 1315 (moving tiles) and 1316 (drifting
effects); this ticket only sets Chapter 7's data.

Split from the chapter's single ambience ticket (2026-11-26): the sound
beds are 1341's.

The chapter's battles and maps are in the private repository's
`content/` (ADR-0117), so the data changes go through the
`content-change` skill: a pull request there first, then this ticket's,
which moves `assets-private.rev`.

## Nick input

**Answer first:** 1368. Nothing to check after merge: verified by
tests and `frame-png` strips; what's left to judge in play goes on
1064's checklist.

## Scope

**In:**
- Chapter 7's battle file(s) and skirmishes name the drifting effect,
  moving tiles and creatures 1368 picked.
- Story moments 1368 picked, set by the battle's triggers (a new
  trigger kind only if 1316 or 1334 made one).
- New effects 1368 picked that the systems' tickets didn't import:
  imported and credited (`THIRD_PARTY_ASSETS.md`).

Anything on the map goes in the map scene and is painted by the map
skins (ADR-0038); this ticket only names data the systems already read.

**Out (do not do):** sound beds (1341); the systems themselves;
other chapters; script lines.

## Implementation steps

1. Read 1368's answer and the systems' data formats (1334, 1315,
   1316).
2. Edit the private content (`content-change`); import anything new.
3. `frame-png` strips of each map; run the gates with the private
   content checked out.

## Acceptance criteria

- [ ] A content test: every Chapter 7 map names what 1368 picked
      (and nothing for a map picked "none").
- [ ] A test: the chapter's recorded playthrough replays identically.
- [ ] `frame-png` strips of each map in the PR (cropped, ADR-0118).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Content: the chapter's data loads and names known effects and
  creatures.
- Scene: a map's scene holds its picked effect (read from the scene,
  not cells).

## Lesser targets

The data is the same everywhere; the tiers (1314) decide what plays
(the Playdate shows the map still). GBA: no target.

## Completion notes
