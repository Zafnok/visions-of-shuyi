---
id: "0908"
title: "Polish before the Steam release: the title's sparks and the threat arrows' look"
type: tuning
milestone: M8 Release
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0908 — Polish before release: sparks and threat arrows

## Context

Chapter 1 playtest round 3 (0804, 2026-10-16). Two looks Nick is happy
with for now but wants tuned before release:

- The title's sparks (0811: 46 warm dots rising at 9 or 18 rows a second,
  swaying a cell or two, all *tunable*): "it looks fine for now. Probably
  before release we tune it".
- The threat arrows (0448: a 1-px red line with a dark edge each side,
  4-px barbs, stopping 3 px short of the tile middle; a 3×9 yellow `!`):
  "probably can be polished before final game release but these came out
  quite well for now."

"Before release" makes this a blocker of the Steam release work (0903,
`blocked_by` updated in the same PR).

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

**Decision** (`ask-nick`) on real frames and short recordings: 3–4
variants of each (sparks: count, speed, size, colour; arrows: thickness,
red, barb size, where the arrow starts), today's look among them.

## Scope

**In:** the variants; Nick's picks; setting the values (they're data or
constants already marked *tunable*); `look-and-feel.md` records them as
Nick's.

**Out (do not do):** new effects; when the arrows show (Nick: "Right",
settled).

## Acceptance criteria

- [ ] Nick picked a look for each; recorded.
- [ ] The values are set; the snapshots updated and reviewed.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the title with sparks (a fixed frame) and a map with arrows,
  per map skin (ADR-0038).

## Completion notes

*(Filled in by the session that completes the ticket.)*
