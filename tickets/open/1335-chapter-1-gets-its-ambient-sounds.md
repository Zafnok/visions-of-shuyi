---
id: "1335"
title: "Chapter 1 gets its ambient sounds"
type: content
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["1324", "1332", "1383"]
blocks: ["1064"]
nick_input: answer-first
completed:
---

# 1335 — Chapter 1 gets its ambient sounds

## Context

1324 records Nick's picks for Chapter 1's places (the battle (0803; the green before the chapel, 0741) and the scenes' places (0029, 0732-0734): the letter, the Red Company, after the fight, Veyne Hall): each
place's sound bed (no passing calls, 1322), whether it follows the light,
and any story moment that changes it. The player is built by 1332; this
ticket only sets Chapter 1's data.

Split from the chapter's single ambience ticket (2026-11-26): the
drifting effects, moving tiles and map touches are 1370's.

**1324's picks** (2026-11-26; the private repository's
`story/design/audio.md`, *Chapter 1's ambience*, has the why and Nick's
words; the sounds and licenses are in `docs/design/audio.md`, *Chapter
1's beds*). Each layer is levelled to the place's music like a music
track, then set by the dB given; "whole bed" is an extra offset on the
place's mixed bed. No bed follows the light; no passing calls.

| Place | Where it plays | Layers (dB) | Whole bed |
| ----- | -------------- | ----------- | --------- |
| The battle | battle `ch01` (its `ambience`) | meadow (Alcappuccino 416206) −6, river (bruno.auzet 690326) −6, wind in the leaves (Borgory 751473) −3 | 0 |
| The letter | `ch01_intro`, map `harrowby_yard` | dawn chorus (Synge101 611453) −9, fire crackle (`sfx/campfire_02.ogg`, the title's) 0 | 0 |
| The Red Company | `ch01_prebattle`, over the battle map: `@ambience` (1383) | river −4, wind in the leaves −4 | −6 |
| After the fight | `ch01_victory`, over the battle map: `@ambience` (1383) | wind in trees with crows (Cinetony 563572) −9, river −3 | −6 |
| Veyne Hall | `ch01_tbc`, map `veyne_study` | the hall's hush (Tonmeister88 557380) −8, fire crackle −10, a clock ticking (giddster 434841) −3 | 0 |

- **The 1322 Summer Meadow (`ambience/summer_meadow.ogg`,
  `bed_summer_meadow`) is replaced** by the Hungary meadow: Nick heard
  its insect buzz and rejected even a filtered copy. Remove it and its
  credit unless another place still uses it.
- **Before importing, check every file for hum or buzz** (steady narrow
  peaks under 700 Hz; Nick hears them) and high-pass the quiet ones
  (dawn chorus, clock, crows) at about 90 Hz.
- **One file per place, premixed** (my choice, technical): mix each
  place's layers at the levels above into one looped bed, so 1332's
  format (one cue per bed) holds as it is. Make the loop seamless (a
  crossfade at the seam).
- The fire and candle the maps lack are 1384, the bell 1385; neither
  blocks this ticket.

The chapter's battles, maps and scripts are in the private repository's
`content/` (ADR-0117), so the data changes go through the
`content-change` skill: a pull request there first, then this ticket's,
which moves `assets-private.rev`.

## Nick input

**Answer first:** 1324. Nothing to check after merge: verified by
tests; what's left to judge in play goes on 1064's checklist.

## Scope

**In:**
- Chapter 1's battle file names its bed; `ambience.ron`'s `places` gives
  `harrowby_yard` and `veyne_study` theirs; `ch01_prebattle` and
  `ch01_victory` get an `@ambience` directive after their `@place battle`
  (1383; a directive, no words changed).
- No story moment changes a bed in the battle (1324). The yard's bell
  is 1385.
- The sounds in the table above: imported, premixed per place and
  credited (`assets/audio/audio.ron`'s credits and
  `THIRD_PARTY_ASSETS.md`); the old Summer Meadow removed.

**Out (do not do):** drifting effects, moving tiles, touches and
creatures (1370); the player itself (1332); other chapters; music;
script lines.

## Implementation steps

1. Read 1324's answer and 1332's data format.
2. Edit the private content (`content-change`); import anything new.
3. Run the gates with the private content checked out.

## Acceptance criteria

- [ ] A content test: every Chapter 1 place names the bed 1324
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
