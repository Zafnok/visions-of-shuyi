---
id: "0055"
title: "Decide: showing which enemies can attack the tile you're moving to"
type: design-decision
milestone: Design decisions (Nick)
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
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

- [ ] Each question has Nick's answer in the design docs, quoted and dated.
- [ ] 0448 says exactly what to build.

## Tests required

- None (documents only). `cargo xtask ticket-lint` passes.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
