---
id: "0336"
title: "Deep water that only fliers and swimmers cross"
type: feature
milestone: M2 Core rules
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["0099"]
nick_input: answer-first
completed:
---

# 0336 — Deep water that only fliers and swimmers cross

## Context

From Nick's notes from a new player's Chapter 1 session (2026-11-22): shallow water
keeps today's rule, deep water is crossed only by flying or swimming
units. 0099 decides the open parts (a new tile or `sea`, who swims, ice,
which maps). This ticket builds what it decides.

Today: `assets/data/terrain.ron` holds the terrains and their costs per
movement type; `crates/core/src/terrain.rs` has `MovementTypeId(u8)` and
`move_cost` (`None` = can't enter); the loader is
`crates/content/src/terrain.rs`. Four movement types: foot, mounted,
armored, flying. Ice turns water and sea to ice (0310). The tileset skin
is `assets-src/tilesets/tiny_tales.ron`, packed by `cargo xtask
tileset-import` (0437); the glyph look's terrain glyphs are in the
terrain table (ADR-0018, ADR-0038).

## Nick input

**Answer first:** 0099. Nothing to check after merge: verified by the
tests below and `frame-png` pictures in the PR; anything left to judge
by playing goes on 1064's checklist.

## Scope

**In:**
- If 0099 picks a new tile: `deep_water` in `terrain.ron` (costs and
  avoid per 0099), its glyph, its tile in `tiny_tales.ron`, its one-bit
  Playdate look (`assets/data/palettes/playdate*.ron` pattern).
- If 0099 picks a swimmer: a `swimming` movement type in `terrain.ron`
  with a cost on every terrain, and the class or skill that uses it.
- Ice on deep water as 0099 says.
- The AI and bots read move costs through core already; check they
  treat deep water as a wall for non-swimmers (a test).
- The map edits 0099 lists, through the `content-change` skill (the
  maps are in `assets-private/content/`).

**Out (do not do):** animated water (1315); a new class's art beyond
what Tiny Tales has (file a ticket if missing); rebalancing a chapter
beyond moving its tiles (each chapter's balance ticket, or the bots'
bands, report it).

## Implementation steps

1. Data: the terrain row(s) and movement type in `assets/data/terrain.ron`;
   load and validate in `crates/content/src/terrain.rs` (every terrain has
   a cost for every movement type).
2. Core: nothing new if costs stay table-driven; add tests in
   `crates/core` that a foot unit can't path through deep water and a
   flier (and a swimmer) can.
3. Look: the tile in `assets-src/tilesets/tiny_tales.ron` (pick from the
   bundle's `Standard_Coast` deep tiles), re-run `cargo xtask
   tileset-import`; the glyph and both palettes; the Playdate pattern.
4. Info: the terrain window names it (text by key, `assets/text/`).
5. Maps: per 0099, via `content-change`.
6. Run the bots on every changed map (`cargo shuyi playtest`) and put
   the bands in the PR.

## Acceptance criteria

- [ ] A core test shows foot, mounted and armored units can't enter deep
      water; flying (and swimming, if added) can, at 0099's cost.
- [ ] `cargo shuyi frame-png` pictures of a river with shallow and deep
      stretches, in the picture look, the glyph look and the Playdate
      skin, are in the PR.
- [ ] Ice on deep water behaves as 0099 decided (a test).
- [ ] The bots' bands on changed maps are in the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: move costs and pathing through deep water per movement type.
- Property: no path for a non-swimmer ever ends on or crosses deep water.
- Snapshot: the terrain window naming deep water.

## Lesser targets

The Playdate draws terrain one-bit by pattern: deep water needs a
pattern clearly different from shallow water's. GBA has no target yet:
nothing to do.

## Completion notes

