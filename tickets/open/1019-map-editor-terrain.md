---
id: "1019"
title: "Map editor, part 1: paint terrain and save a .map into a mod"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1014"]
nick_input: sign-off
completed:
---

# 1019 — Map editor: paint terrain

## Context

Maps are plain text today (`assets/maps/README.md`: a grid of legend
characters, tile features, a `look` header). Writing one by hand is how the
game's own maps are made, but modders (and Claude, for new chapters) work
faster painting with the same cursor and keys as the battle map. Nick asked
for "a bunch of dev tools" ([`docs/design/replayability.md`](../../docs/design/replayability.md)).

The editor is a screen in the game, reachable from the title menu's Mods
entry (1015) or, until that exists, the debug menu (ADR-0023). It writes
`.map` files into a mod folder through `app` (only `app` touches files,
ADR-0004).

## Nick input

**Sign-off:** Nick paints a small map, saves it, and says whether the
controls feel like the battle map's.

## Scope

**In:**
- A `MapEditor` screen: a new map of a chosen size or an existing map from
  the bundle; the battle cursor moves over it; a palette of the terrains in
  `assets/data/terrain.ron`; paint one tile, a line, or a filled rectangle;
  undo and redo; tile features as the `.map` format allows.
- The map is drawn **through the map scene and the map skins** like the
  battle map (ADR-0038), so it looks the same in the glyph look and the
  sprite look.
- Saving writes a valid `.map` (the same writer is used by tests: parse →
  write → parse gives the same map) into `mods/<chosen mod>/maps/`.
- New Actions only where no existing one fits, with defaults on both
  layouts and the controller (the `keyboard-input` skill).

**Out (do not do):** units, reinforcements, objectives and triggers (1020);
playing the map (1020); the web build.

## Implementation steps

1. Read ADR-0017, ADR-0038, ADR-0052 (the map's look), the
   `keyboard-input` skill, `assets/maps/README.md`, `crates/content/src/map.rs`
   and the battle screen's map view and cursor (`crates/ui/src/map_view*`,
   `screens/battle`).
2. Add a `.map` writer to `trpg-content` (`MapDef::to_text`) with a round
   trip property test.
3. Build the editor screen in `crates/ui/src/screens/map_editor/`; the
   editor's own state (tool, brush, undo stack) is plain data with unit
   tests; the screen only draws and maps Actions to edits.
4. Saving goes through a new request from `ui` to `app` (the same way saves
   go through `Storage`, or a small file-writing trait if the ADR from
   1011 named one).
5. Snapshot tests of the editor in both looks.

## Acceptance criteria

- [ ] Parse → write → parse of every map in `assets/maps/` gives the same
      `MapDef` (property / integration test).
- [ ] Painting, line, rectangle, undo and redo change the map as expected
      (unit tests on the editor state).
- [ ] The editor draws through the map scene (tests read the scene, not
      cells, per ADR-0038), with a snapshot per look.
- [ ] No hard-coded keys or literal screen text (`check-keys`,
      `check-text`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: editor state.
- Property: the `.map` writer round trip.
- Snapshot / integration: the editor screen in both looks; a Harness test
  that paints and saves to `MemoryStorage` or a test writer.

## Completion notes
