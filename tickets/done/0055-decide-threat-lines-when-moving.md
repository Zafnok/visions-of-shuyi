---
id: "0055"
title: "Decide: showing which enemies can attack the tile you're moving to"
type: design-decision
milestone: Design decisions (Nick)
model: reasoning
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-12
---

# 0055 — Decide: threat lines when moving

## Context

Nick, playing Chapter 1 (ticket 0804, 2026-10-12):

> "when moving or proposing to move into an enemy's attack range, it should
> illuminate or draw slim arrows from enemy to the tile or unit you're
> moving to. Newer fire emblem games do this. So you don't need to rely on
> danger zone as much."

Today the only help is the danger zone key (`turn-structure.md`; it tints
every tile any enemy can reach) and picking an enemy to see its own range.
Neither says **which** enemies reach the tile under the cursor.

What the games do (check each before showing Nick; say so if unsure):

- **Fire Emblem: Three Houses** — while a unit is selected, a thin curved
  red line runs from each enemy that would target it to the unit at the
  tile the cursor proposes. It shows who the AI *will* attack, not only who
  could.
- **Fire Emblem Engage** — the same kind of lines, from every enemy that
  *can* reach the tile.
- **Fire Emblem: Awakening / Fates** — no lines; the danger zone, and one
  enemy's range can be pinned on.

## Nick input

**Decision** (`ask-nick`, with rendered mockups from the real battle map,
bought art, sent and never committed). Questions:

1. **What marks the enemies:** slim lines or arrows from each enemy to the
   tile; a glow or mark on each enemy that reaches it; both.
2. **Which enemies:** everyone who *could* attack that tile next enemy
   phase (Engage), or only those the AI *would* send at it (Three Houses;
   costs more to build and gives the AI's plan away).
3. **When:** only while a unit is selected and the cursor is on a tile it
   can move to; or also whenever the cursor rests on one of your units.
4. **Enemies off the screen:** the line runs to the screen's edge, or
   nothing.
5. **Is it a setting** (Options: on / off)? Don't add one unless he asks.
6. Whether an enemy that waits in place (Chapter 1's bridge and fort
   guards attack whoever comes within reach) counts: by the "could attack"
   rule it does.

Each with options from the games above plus "describe your own".

## Scope

**In:**
- The questions above, asked and recorded in
  `docs/design/turn-structure.md` (the rule) and
  `docs/design/look-and-feel.md` (the look).
- Update ticket 0448 with the answers.

**Out (do not do):**
- Building it: ticket 0448.
- Changing the danger zone.

## Implementation steps

1. Render the mockups and ask (`ask-nick`).
2. Record the answers; update 0448's steps to match.

## Acceptance criteria

- [x] Each question has Nick's answer in the design docs, quoted and dated.
- [x] 0448 says exactly what to build.

## Tests required

- None (documents only). `cargo xtask ticket-lint` passes.

## Completion notes

Asked 2026-10-12 with mockups drawn over a frame of the Chapter 1 battle
(bought art; sent to Nick, not committed). Nick's answers:

1. Look: "B + C": straight arrows and a mark over each enemy. He then asked
   for the mark to sit higher, clear of the enemy's head.
2. Which enemies: "A": everyone who could attack the tile.
3. When: "B": while proposing a move, and whenever the cursor rests on one
   of your units.
4. Off the screen: "A": the arrow comes in from the screen's edge.
5. A setting: not asked, as the ticket says; none.
6. Enemies that wait in place: they count (part of 2A as asked).

Recorded in `docs/design/turn-structure.md` and `look-and-feel.md`
(*Threat arrows* in both); 0448 updated.

*the agent's starting rules* (Nick can veto; listed in the design docs):

- "Can attack" is worked out like the danger zone, with the selected unit
  counted on the proposed tile.
- Pointing at an enemy you can attack: the arrows point at the tile you
  will attack from.
- The arrows stay while the action menu is open, and go when the forecast
  opens.
- Your units that have already acted show them too; green units don't.
- An enemy that can only heal gets no arrow.
- The glyph look gets the same arrows and marks (Nick saw only the
  bought-art look).
- The mark is drawn in front of a unit in the tile above, and moves down
  on the top row of the view. Nothing blinks or moves.

Deviation: questions 5 and 6 weren't asked as separate questions (5 by the
ticket's own instruction; 6 was put inside question 2's option A, with the
bridge Raider in the pictures). I said Fire Emblem Engage shows who "can"
attack with less than full certainty, and told Nick so.
