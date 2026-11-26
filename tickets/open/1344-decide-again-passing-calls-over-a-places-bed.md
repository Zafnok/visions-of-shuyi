---
id: "1344"
title: "Decide again: passing calls over a place's bed"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: blocked
blocked_by: ["1332", "1333", "1334", "1315", "1316", "1335"]
nick_input: decision
completed:
---

# 1344 — Decide again: passing calls over a place's bed

## Context

1322 (2026-11-23, `docs/design/audio.md`, *Ambience*) gave each place a
sound bed under its music and asked about **passing calls**: single
sounds at random gaps over the bed (a bird, a dog far off, a bell), as
Stardew Valley drops a lone cricket chirp or a frog's croak over its
loops. Nick heard them on the listening page over two rounds, from one every
8–20 s to one every 90–180 s, with two bells, and said:

> I think the calls are a good idea but not in a battle and not at this
> frequency and the chapel bell is too loud and long for a call

> for now I don't like the randomness and loudness of the calls but they
> might be revisited later so keep that part open as to revisit after
> several more life tickets (the ones which make the game less amateur and
> more polished)

So battles never get calls, and nothing else gets them until this ticket
asks again, once the game has more of its "life" built: the ambience
player and Chapter 1's beds (1332, 1335), the moving tiles and drifting
effects (1315, 1316) and the hit bursts and map touches (1333, 1334).
The calls heard (authors, pages, licenses) are listed in `audio.md`,
*Ambience*. The 1322 page's source is gone (throwaway); the published
page (https://claude.ai/artifact/T6YeygJ9ubs3apgU1Hmx4e) still plays them.

The 13xx block holds this game's "more life" tickets (1322–1377), so
this decision takes a 13xx number (`tickets/README.md`, *Numbering*).

## Nick input

**Decision** (`ask-nick`; a listening page over the game as it is then,
with a slider per layer: music, bed, calls, as Nick asked for 1322). His
two dislikes are the brief: **the randomness** and **the loudness**.
Options to hear, checked against the game before they are shown:

- A. **Still none.**
- B. **Calls at set moments, not random**: one when a scene or the world
  map opens, or on a turn's start, never on top of a line.
- C. **Rare and soft random calls** where a bed plays outside battles
  (a scene's place, the world map, towns, camp): far quieter than 1322's
  (e.g. the bed's own level), one every 90–180 s or rarer.
- D. Describe your own.

Then, for the picked option: where (scenes, world map, towns, camp), how
loud against the bed, and the Playdate (the same, or none).

## Scope

**In:** the listening page (throwaway, never committed); the answer in
`audio.md` (*Ambience*: replace "No passing calls, for now", and the open
sub-question); a feature ticket to build it (`write-ticket`) if Nick
picks any calls.

**Out (do not do):** calls in a battle (Nick: "not in a battle"); building
anything; changing a bed or its volume (1322); a chapter's own picks
(1324–1331, 1362–1369).

## Implementation steps

1. Read `audio.md` (*Ambience*, its appendix rounds) and play the game's
   beds as 1332 and 1335 built them.
2. Pick candidate calls on Freesound and OpenGameArt (CC0 or CC-BY 4.0,
   ADR-0013), short and soft: 1322's were too loud and its bell too long.
3. Build the listening page: a frame of a scene's place or the world map
   (`frame-png`), the music, the bed and the calls, each with a slider.
4. Ask; record Nick's words and the rules in `audio.md`; file the build
   ticket if any.

## Acceptance criteria

- [ ] `docs/design/audio.md`, *Ambience*, says what passing calls are now
      (or that there are none), with Nick's words quoted.
- [ ] Every call Nick picked has its author, page and license recorded.
- [ ] If he picked calls, a feature ticket builds them, blocked by this
      one.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

None (a decision).

## Lesser targets

The Playdate follows the answer; GBA: no target (1314).

## Completion notes
