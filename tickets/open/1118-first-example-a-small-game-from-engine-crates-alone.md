---
id: "1118"
title: "The first example: a small game made from engine crates alone, tested in CI and built for the web"
type: feature
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1112", "1116", "1117"]
nick_input: none
completed:
---

# 1118 — The first example: a small game from engine crates alone

## Context

ADR-0063 §8.2: the engine moves to its own repository only once an
example game runs and is tested without any game crate. engine ADR-0003 §1:
the README claims a style only when its example passes CI. This is the
first example. It proves the carve (1103–1117) left nothing of our game
in the engine's way.

## Nick input

None.

## Scope

**In:**
- `engine/examples/hello-grid/`, a binary crate depending only on
  `shuyi-*`. It has:
  - a title menu;
  - an options screen with two settings;
  - key rebinding;
  - a short `.dlg` scene;
  - a 12×10 grid stage where the player moves one piece to a goal tile
    while one enemy piece steps toward it, using `TurnRules`;
  - a win screen.
- It ships with CC0 art only: the Terminus font, plus a tileset made by
  `shuyi-cli` from glyphs (as `xtask test-tileset` does).
- Tests:
  - a Harness test that plays it to the win;
  - a bot test (random bots over `TurnRules`);
  - a `frame-png` picture checked as a snapshot;
  - a determinism test (same seed and commands, same result).
- CI builds it native and for the web, and Pages publishes the web
  build at `/engine/hello-grid/`.
- `engine/README.md`: how to build and run it; the crates; what the
  example shows.

**Out (do not do):**
- other styles (engine tickets 1119–1128, parked);
- `shuyi new` (engine ticket 1129);
- moving the code to the engine repository (1131).

## Implementation steps

1. Write the example as its own crate in the workspace.
2. Add its tests and CI and Pages steps.
3. Write the README section.

## Acceptance criteria

- [ ] `cargo run -p hello-grid` plays. The web build runs in the browser
      pane.
- [ ] The Harness test, bot test, snapshot and determinism test pass in
      CI.
- [ ] `check-engine-boundary` passes; the example names no game crate.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: Harness playthrough, bots, determinism.
- Snapshot: one `frame-png` picture.

## Completion notes
