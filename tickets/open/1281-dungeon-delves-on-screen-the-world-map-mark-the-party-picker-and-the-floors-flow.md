---
id: "1281"
title: "Dungeon delves on screen: the world-map mark, the party picker and the floors' flow"
type: feature
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["1277"]
nick_input: answer-first
completed:
---

# 1281 — Dungeon delves on screen: the world-map mark, the party picker and the floors' flow

## Context

Nick's storefront-hook search (2026-11-16) settled the game's hook:
legendary beasts hidden at the bottom of dungeon delves, fought as a
raid. The chain is 0085–0088, 0772, 1091–1099 and 1275–1281. 1277
builds the delve's rules: `NodeKind::Delve`, `DelveDef` and `DelveState`,
the floor generator, the Way Down and the Way Out, and `start_delve`,
`finish_floor` and `leave_delve` on `Campaign`. It was split when it was
written, because rules and screens together were 2,700–3,200 changed
lines (ADR-0101). This ticket is the screen half, so a player can reach
a delve from the world map and play it through.

The rules come from 0088's answers in `docs/design/delves.md`. Where they
differ from this sketch, the decision wins.

**What it builds on:**
- The world map screen: `crates/ui/src/screens/world_map.rs` with
  `PlaceMark` and `FindKind` in `world_map/view.rs`, painted by
  `glyph.rs`, `picture.rs` and `playdate.rs`.
- The game flow: `crates/ui/src/flow.rs`.
- Screens are built as logic, then a view, then a skin (ADR-0054), as the
  engine's `screens/options.rs` is.
- Keys work through `Action`s only (the `keyboard-input` skill).
- No screen today lets the player pick who deploys, and a delve needs one.

## Nick input

**Answer first:** 0088 (the party's size, whether units left out are
unavailable, and whether Preparations comes before the first floor). The
delve mark's look is asked in a mockup before building, picture look
first, unless 0088 already settled it. The random skirmish's "?" on a
grey plate (`look-and-feel.md`, *The world map*) is taken, so the delve
mark must look different.

## Scope

**In:**
- `PlaceMark::Delve` and its panel lines in the glyph, picture and
  Playdate world-map skins. A `FindKind` row if 0088 wants one.
- The party picker: lists the roster and toggles units in and out within
  the size range, with a Playdate skin.
- The flow: `WorldEvent::DelveAhead` → the picker → Preparations (if
  0088 says so) → each floor's battle → `finish_floor` → the next floor
  or the world map. Leaving by a Way Out returns to the world map.
- Snapshots and frame-png pictures.

**Out (do not do):**
- The rules (1277), darkness (1278), hunts and lairs (1279, 1280).

## Implementation steps

**Size (ADR-0092):** about 400 production lines (comparable: one list
screen with toggles plus a new world-map mark, about 1008's screen
part); about 1,000 changed lines with tests.

1. Read `docs/design/delves.md`, 1277's completion notes and its code,
   `world_map.rs`, `flow.rs`, and one list screen in the engine.
2. **The mark:** `PlaceMark::Delve` in `world_map/view.rs`, drawn by the
   three world-map skins; its panel lines. If 1082 is done, add the
   delve's icon to its icon table. If not, add the delve to 1082's list.
3. **The party picker:** a screen (logic, then view, then skin, ADR-0054)
   that toggles units with `Confirm`, moves with the cursor Actions,
   leaves with `Cancel`, and refuses to start outside the size range
   (the view says why, through text keys). PC and Playdate skins
   (`crates/ui/src/screens/playdate/`). It goes on the game's screen list
   so `cargo shuyi check --target` sees it.
4. **The flow** in `crates/ui/src/flow.rs`: the path above. The module
   doc gets the delve's path.

## Acceptance criteria

- [ ] The world map shows the delve's mark in the glyph, picture and
      Playdate looks (snapshot tests), and frame-png pictures of them are
      in the PR.
- [ ] The party picker refuses a party outside the size range (screen
      test), and frame-png pictures of it are in the PR.
- [ ] A flow test enters the test delve from the test world map, picks a
      party, plays its floors with the bots and is back on the world map
      after the Way Out.
- [ ] `cargo shuyi check-keys` and `cargo shuyi lint` pass (no key named
      in code).
- [ ] What is left to judge by playing is in the Completion notes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the picker's toggles and size check.
- Snapshot / integration: the mark in three skins; the picker; the
  delve's flow end to end on test data.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
