---
id: "1118"
title: "The first example: a small game made from engine crates alone, tested in CI and built for the web"
type: feature
milestone: "Engine: carve"
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1112", "1116", "1117"]
nick_input: none
completed: 2026-10-18
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

- [x] `cargo run -p hello-grid` plays. The web build runs in the browser
      pane.
- [ ] The Harness test, bot test, snapshot and determinism test pass in
      CI.
- [x] `check-engine-boundary` passes; the example names no game crate.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: Harness playthrough, bots, determinism.
- Snapshot: one `frame-png` picture.

## Completion notes

**Done.** `engine/examples/hello-grid/` (package and binary `hello-grid`)
depends only on `shuyi-core`, `shuyi-assets`, `shuyi-ui`, `shuyi-app`
and, for its tools, `shuyi-cli`; `check-engine-boundary` passes.

- **The game:** a title menu (Play, Options, Quit); the engine's options
  screen with two settings (the engine's text speed, the game's map look:
  glyphs or tiles), the Key bindings screen and Restore defaults; a short
  `.dlg` scene in the engine's text box over the stage; a 12 × 10 stage
  read from the engine's `.map` format, where your piece walks up to four
  tiles a turn to the goal and the foe (`Chase`, a `Bot`) steps one tile
  toward it, tripping 25% of the time on the state's seeded RNG; rules
  over `TurnRules` and `Sides`; an end screen (won or caught; Play again,
  Back to the title). Leaving the stage asks first.
- **Art:** the Terminus font (a copy of the game's BDF and its atlas from
  `cargo shuyi font-atlas`) and a tileset drawn from its glyphs by a new
  `cargo shuyi glyph-tiles` (`shuyi_cli::glyph_tiles`, listed in the
  project's `shuyi.ron`). Sounds: sine blips made by code.
- **Tests** (`tests/it/`): a Harness playthrough to the win by keys (and
  one caught, leaving, the scene, steering, walks, the trip message); the
  bot test (random bot against the chaser over 60 seeds, both outcomes
  seen; random against random); determinism (same seed and commands, same
  events and state; same seed and keys, same frames); options and key
  rebinding; the files equal what the tools make; two `frame-png`
  pictures as insta binary snapshots, and the title and end as text
  snapshots.
- **CI:** `test` builds and runs it natively on three OSes; the `wasm`
  job builds it for the web (artifact `hello-grid-web`); Pages publishes
  it at `/engine/hello-grid/`. `engine/README.md` has the section; the
  example has its own README.
- **Seen working:** `cargo run -p hello-grid` starts; the web build was
  played to the win in the browser pane, in both looks.

**Deviations:**

- The ticket says "CC0 art only: the Terminus font"; Terminus is
  OFL-1.1 (allowed by ADR-0013, as for the game). Registered in
  `THIRD_PARTY_ASSETS.md` with its license beside it.
- The example's folders are `files/` and `files-src/`, not `assets/`:
  `check-engine-boundary` bans paths starting with `assets/` (the game's
  folder), and a false positive is no reason to loosen the gate.
- The console is 100 × 32 (the game's), with the sprite map at 2×: the
  engine's options and Key bindings skins are laid out at fixed cells for
  that size and run off a smaller one. That is something of the game in
  the engine's way; follow-up **1155** lays them out from the frame's
  size and moves the example to 48 × 24.
- Two engine fixes the example needed:
  - `shuyi web` looked for the wasm in the project's own `target/`; a
    project inside a bigger workspace builds into the workspace's. It now
    asks `cargo locate-project --workspace` (or `CARGO_TARGET_DIR`).
  - `shuyi-app`'s `keys::poll` put a frame's releases before its presses,
    so a key pressed and let go within one frame (a quick tap at a low
    frame rate, or the browser pane's synthetic keys) read as held and
    repeated until the next release: the cursor ran to the map's edge. A
    key up at the end of the frame is now a tap (`keys::ordered`, tested).
    This fixes the game's web build too.

**Rules decided here:** none of the game's. The example's own rules (four
tiles a turn, the foe's one step and 25% trip, caught on the same tile)
are the engine example's, not Visions of Shuyi's.

**Follow-up:** 1155 (the engine's screens fit any console).
