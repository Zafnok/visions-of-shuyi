# ADR-0063: An engine and a game: one boundary in this repository now, two repositories later

- **Status:** Proposed (1101). It becomes Accepted when Nick approves the
  plan in `docs/engine/feasibility.md`; the first engine ticket that lands
  (1102) changes this line.
- **Date:** 2026-10-14
- **Related tickets:** 1101 (the study), 0059 (license, name, going
  public), 0060 (free movement in the game), 1102 (the boundary gate),
  1103–1117 (the carve), 1118 (the first example), 1130–1132 (the move to
  its own repository)
- **Extends:** ADR-0004 (crate layering), ADR-0013 (licensing),
  ADR-0038 and ADR-0054 (a view, a skin)

## Context

Nick, 2026-10-14: split the project into **an engine**, general purpose,
"free/open source" one day, which makes our game easy to build *and* makes
other people's 2D games easy to build (not only tactics games); and **the
game**, which stays paid and uses the engine "as an external dependency".

What the code looks like today (audit in `docs/engine/boundary.md`):

- ADR-0004's four layers already separate rules (`core`), data
  (`content`), screens (`ui`) and the platform (`app`); macroquad is only
  in `app`; `core` does no I/O and is deterministic. ADR-0038 and ADR-0054
  split *what to show* from *how it looks*. These are the hard parts of an
  engine boundary and they exist.
- But the layers are horizontal: each crate holds engine-like code and
  this game's code together. After cutting the mixed files apart: `core`
  is 3% genre-agnostic, 11% grid-tactics, 86% this game's rules;
  `content` 47% / 4% / 49%; `ui` 28% / 9% / 63%. `app` is almost all
  platform; half of `xtask` is generic tooling; `bots` is all game.
  Roughly a third of the codebase (about 60k of 180k lines, counting
  tests) belongs in the engine.
- `core` has no simulation trait: `Command`, `Event` and `BattleState`
  are concrete, and its grid and pathfinding code is typed over the
  game's map, tables and units. That part is rewritten generically, not
  moved. `content`'s loaders are almost all `load()` plus a pure
  `from_source(file, text)`, which is a clean seam.
- The couplings that stop a second game from using the engine today:
  1. `Action` is this game's closed enum (`trpg_content::keymap`), and
     keymaps, the rebinding screen, help text and tips are built on it.
  2. `Ctx` holds this game's `Content`, campaign mode, map skin and lead.
     Every `Screen` takes that `Ctx`.
  3. `Game::start` builds this game's title screen; `Game` opens the
     layout picker and the debug menu itself.
  4. The console is a fixed 100×32 cells of 8×16 pixels, in constants used
     by 15 files.
  5. The frame (`GlyphBuffer`) imports content types (`ImageId`, `Light`);
     `UiColor` is a closed list that includes tactics colors.
  6. `MapScene` and `MapSkin` carry battle units (`Faction`, classes, HP)
     and `trpg_core::Pos`; generic code reaches up into
     `screens::battle` (walk speeds, unit colors, `MAP_VIEW`).
  7. The types the engine's input, fonts, images, palette, audio and
     languages need live in `trpg-content`, next to the game's tables.
  8. The `Harness` is wired to the flow and battle screens and the test
     chapter.
- Nick's top priority today is Chapter 1 playtest round 3 ("my top
  priority to get all this feedback done", 2026-10-14). Most of its
  tickets change `ui`, the crate this split changes most.
- Every commit so far is Nick's or Claude's (plus Dependabot version
  bumps), and `LICENSE` section 7 gives Nick the rights to contributions,
  so the engine's files can be relicensed by Nick alone. Which license is
  his call (0059).

## Decision

### 1. Two products, one direction

- **The engine** knows nothing about Visions of Shuyi: no game crate, no
  file under `assets/`, `assets-test/` or `assets-private/`, no class,
  character, story or Combat Art. It ships only free, open material (the
  Terminus font; CC0 art for its samples).
- **The game** uses the engine the way any other game would. Its rules,
  data, story, screens, bought art and the private repository stay in the
  game.
- The engine includes an optional **tactics kit**: grid, tile movement
  and ranges, turns and phases, the battle cursor, grid map scenes and
  skins. It is the part of our game another grid-tactics game would want;
  the game's own rules (stats, Combat Arts, classes, magic, AI weights)
  are not in it. If Nick says the kit stays the game's (0059 question 2),
  it moves to the game side and nothing else in this ADR changes.

### 2. The layers stay ADR-0004's, split down the middle

Each game crate sits on the engine crate of the same layer:

```
engine-app    (window, blitting, input devices, audio backend, storage, run loop)  ◄── trpg-app (the binary: picks the game and its settings)
engine-ui     (frame, screens, input actions, widgets, skins, cinematics, Harness) ◄── trpg-ui  (this game's screens and skins)
engine-assets (bundle layers, images, fonts, palette, audio cues, languages,       ◄── trpg-content (this game's tables and files)
               keymaps, tilesets, importers, error reports)
engine-core   (RNG, simulation traits, tactics kit: grid, paths, turns; no I/O)    ◄── trpg-core (this game's rules)
engine-cli    (frame-png, font-atlas, check-keys, check-text, web, …)             ◄── xtask (this repository's own tools)
```

ADR-0004's rules hold for each engine crate exactly as for the game crate
beside it: `engine-core` is pure and deterministic; only `engine-app`
touches the platform; time is an argument; no `unsafe` outside
`engine-app`. A game crate may depend on the engine crate of its layer
and the ones below; an engine crate never depends on a game crate.

### 3. Where it lives until it has its own repository

- A top-level `engine/` folder holds everything that will become the
  engine's repository: `engine/crates/{core,assets,ui,app,cli}`, its
  `README.md`, `docs/`, `examples/` (sample games, ADR-0064) and
  `skills/` (ADR-0066). Package names `engine-core`, `engine-assets`,
  `engine-ui`, `engine-app`, `engine-cli`: a **working name**, renamed in
  one ticket when 0059 gives the real one.
- One Cargo workspace (`members = ["crates/*", "engine/crates/*"]`), one
  CI, one PR for a change that needs both sides. Nothing about our ticket
  workflow changes while the API is still moving.
- Engine files stay under `LICENSE` until 1130 adds the license Nick
  chose, in the engine folder only.

### 4. The boundary is a gate

`cargo xtask check-engine-boundary` (1102) fails when:

- an `engine-*` crate depends on a `trpg-*` crate, or on a third-party
  crate not on the engine's list in `engine/README.md`;
- a file under `engine/` names `trpg_`, `visions-of-shuyi`, `assets/`,
  `assets-private`, or an id from this game's data (class, character,
  terrain, item, cue ids read from the data files).

It runs in CI with the other `xtask` checks.

### 5. What goes where

`docs/engine/boundary.md` lists every module with its side (engine,
tactics kit, game, or "mixed: cut here"). The test for a new piece of
code: *would a platformer or a farming game want this unchanged?* →
engine. *Would another grid-tactics game?* → tactics kit. Otherwise →
game. When unsure, game: moving code down later is cheaper than pulling
game knowledge back out of the engine.

### 6. How code moves

- **One layer, one ticket, no change in behavior.** Snapshots must not
  change; tests move with their code. Each move first makes the generic
  part generic in place (an `ActionSet` trait instead of the closed
  `Action`, `Ctx<G>` instead of a `Ctx` holding `Content`), then moves it.
- **Game code keeps its names.** The game defines type aliases
  (`pub type Ctx = engine_ui::Ctx<Shuyi>`, `pub type FrameInput =
  engine_ui::FrameInput<Action>`) so the 40,000 lines of game screens
  change field paths, not signatures.
- **`core`'s tactics code is not moved as it is.** The kit gets a
  `TurnRules` trait (check, apply, legal commands, outcome) that
  `BattleState` implements in an `impl` block, and a Dijkstra over a cost
  closure that the game's `movement.rs` calls. The battle's own turn loop
  stays in the game.
- **Order** (tickets 1103–1117), bottom up, because each layer needs the
  one below: the asset types that the frame, input and audio need
  (`content` modules that never import `trpg_core`), then input made
  open (`ActionSet`), then the frame, then screens and `Ctx<G>`, then the
  platform shell and run loop, then the Harness, then the generic screens
  and widgets, then cinematics, then the tactics kit (`core`, then `ui`,
  then the bots), then the tools. 1118 then builds a small example game
  from engine crates alone.
- New generic work after a layer has moved is written in `engine/`
  directly.

### 7. When: never in front of the game

- **Now** (no conflict with open work): this ADR, the audit, the boundary
  gate (1102), and research spikes.
- **The carve (1103–1117) waits for 0804**, Nick's Chapter 1 playtest:
  every move touches files the playtest tickets touch, and the playtest's
  feel work is the top priority. No engine ticket is ever put on the
  Chapter 1 critical path.
- Engine-only features (other game styles, ADR-0064; networking,
  ADR-0067) are scheduled by Nick's say-so (`docs/engine/feasibility.md`,
  *What it costs the game*), not as a side effect of game tickets.

### 8. Moving to its own repository

The `engine/` folder becomes its own public repository (1131) when all
three hold:

1. Nick has chosen the license and name (0059) and 1130 has applied them;
2. an example game in `engine/examples/` runs and is tested without any
   game crate (1118);
3. the API has settled: over the last 30 days, fewer than 1 in 5 merged
   game PRs had to change a file under `engine/`.

Then: `git filter-repo --subdirectory-filter engine` keeps the engine's
history; the game depends on the engine by a pinned git revision
(`engine.rev`, like `assets-private.rev`, ADR-0040) and later by
crates.io versions; a game ticket that needs an engine change becomes an
engine PR first, then a game PR that moves the pin.

## Consequences

- The game gains from the carve even if the engine is never published:
  `Action`, `Ctx` and the frame become smaller, generic and tested on
  their own; the Harness and `frame-png` work for any screen.
- The carve is mechanical but wide: about a third of the codebase moves
  (`docs/engine/boundary.md`), and the `Action` and `Ctx` changes touch
  most screens. It costs about 15 tickets, run one at a time, during which
  other `ui` work must rebase.
- Until the engine has its own repository, its users are us. Claims that
  it supports other kinds of games are only true once a sample game of
  that kind runs in `engine/examples/` (ADR-0064).
- After the split, a change that needs both sides costs two PRs and a pin
  bump. Rule 8.3 delays that until it is rare.
- A second consumer (the samples, later other people's games) means the
  engine's API is a promise: breaking changes need a version bump and a
  note, which game tickets never needed.
- Each engine crate keeps the full gates (clippy pedantic, mutation
  testing, coverage); nothing in the engine is exempt because "it's just
  tooling".

## Alternatives considered

- **Two repositories now** — every engine-touching game ticket would be
  two PRs and a pin bump while the API changes weekly, during the
  playtest work Nick put first. Rejected; rule 8 gets there once it is
  cheap.
- **Keep one product; never split** — simplest, but loses Nick's goal,
  and the game's generic parts stay tangled with its rules (the closed
  `Action`, the game-wired `Ctx`), which already costs us in tests.
- **Build the engine on Bevy (as plugins)** — Bevy brings physics, tilemap
  and LDtk crates, an editor in progress, and a large audience. But it
  means rewriting `ui` and `app` around an ECS, the reasons ADR-0003 gave
  (compile times, WASM size) still hold, and our edge (state as plain
  data, a text-snapshot Harness, `frame-png`, a deterministic core) is
  harder to keep in an ECS world. A Bevy backend for `engine-app` stays
  possible later because the platform shell is thin.
- **Port the game to Godot and build tools for Godot instead** — a
  rewrite of ~180k lines, loses the Rust test and mutation gates, and
  gives Nick nothing he owns.
- **Engine files under an open license inside this repository forever** —
  it is step one (rule 3), but as the end state it mixes the engine's
  issues and contributors with a proprietary game's, and users cannot
  depend on a folder of someone else's game.
- **A finer split (a crate per subsystem, like Bevy's 50)** — more
  compile units and mutation shards for no gain at our size. Five engine
  crates mirror the four layers we already reason in; a crate can be split
  later when a user needs one part alone (for example only the
  frame and `frame-png`).
