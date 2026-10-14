# ADR-0063: An engine and a game: one boundary in this repository now, two repositories later

- **Status:** Accepted (Nick chose plan B, 2026-10-14). The same day the
  engine got its own repository for its plans, tickets and skills,
  [Shuyi Engine](https://github.com/Zafnok/shuyi-engine); §3 and §8 say
  how that fits.
- **Date:** 2026-10-14
- **Related tickets:** 1101 (the study), 0059 (license, name, going
  public), 0060 (free movement in the game), 1102 (the boundary gate),
  1103–1117 (the carve), 1118 (the first example), 1131 (the code moves
  to the engine's repository); the engine-only tickets moved to Shuyi
  Engine with their numbers (1119–1129, 1132–1135, 1140–1150)
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
shuyi-app    (window, blitting, input devices, audio backend, storage, run loop)  ◄── trpg-app (the binary: picks the game and its settings)
shuyi-ui     (frame, screens, input actions, widgets, skins, cinematics, Harness) ◄── trpg-ui  (this game's screens and skins)
shuyi-assets (bundle layers, images, fonts, palette, audio cues, languages,       ◄── trpg-content (this game's tables and files)
               keymaps, tilesets, importers, error reports)
shuyi-core   (RNG, simulation traits, tactics kit: grid, paths, turns; no I/O)    ◄── trpg-core (this game's rules)
shuyi-cli    (frame-png, font-atlas, check-keys, check-text, web, …)             ◄── xtask (this repository's own tools)
```

ADR-0004's rules hold for each engine crate exactly as for the game crate
beside it: `shuyi-core` is pure and deterministic; only `shuyi-app`
touches the platform; time is an argument; no `unsafe` outside
`shuyi-app`. A game crate may depend on the engine crate of its layer
and the ones below; an engine crate never depends on a game crate.

### 3. Where it lives: plans there, code here until the carve is done

- **The name is Shuyi Engine** (Nick, 2026-10-14; the engine
  repository's `docs/decisions.md`). Its crates are `shuyi-core`,
  `shuyi-assets`, `shuyi-ui`, `shuyi-app` and `shuyi-cli`, and its
  binary is `shuyi`.
- **The engine's own repository**, `github.com/Zafnok/shuyi-engine`
  (public, MIT OR Apache-2.0), holds three things from day one:
  - its ADRs ("engine ADR-NNNN"; this repository's proposals 0064–0067
    became its 0003–0006);
  - its engine-only tickets;
  - Nick's product decisions, its roadmap and its development skills.
- **The code is carved here first.** A top-level `engine/` folder holds
  the crates as they are carved (`engine/crates/{core,assets,ui,app,cli}`,
  then `engine/examples/`). It sits in one Cargo workspace (`members =
  ["crates/*", "engine/crates/*"]`) with one CI, so a change that needs
  both sides is one PR while the API is still moving.
- **Licensing while here.** Files in `engine/` stay under this
  repository's `LICENSE` while they are here. They become MIT OR
  Apache-2.0 when 1131 publishes them in the engine's repository. Nick
  owns them and chose that license. `LICENSE` itself is not changed.

### 4. The boundary is a gate

`cargo xtask check-engine-boundary` (1102) fails when:

- an `shuyi-*` crate depends on a `trpg-*` crate, or on a third-party
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
  (`pub type Ctx = shuyi_ui::Ctx<Shuyi>`, `pub type FrameInput =
  shuyi_ui::FrameInput<Action>`) so the 40,000 lines of game screens
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
- Engine-only features are **parked** in the engine's repository until
  the engine reaches parity with the game (plan B; its `docs/ROADMAP.md`):
  - other game styles (engine ADR-0003);
  - networking (engine ADR-0006);
  - visual scripting.

  They are never a side effect of game tickets. Each is unparked early
  only if the game needs it: 0060 for free movement, 0053 for online
  play.

### 8. Moving the code to the engine's repository

The `engine/` folder moves to Shuyi Engine (1131) once the carve is done
and an example game in `engine/examples/` runs and is tested without any
game crate (1118). The license and name are already settled.

How it moves:

1. `git filter-repo --subdirectory-filter engine` keeps the engine's
   history. It is merged into the engine repository's `main` with
   `--allow-unrelated-histories`, beside the plans already there.
2. The game depends on the engine by a pinned git revision
   (`shuyi-engine.rev`, like `assets-private.rev`, ADR-0040), and later
   by crates.io versions.
3. A game ticket that needs an engine change becomes an engine PR first,
   then a game PR that moves the pin.

Over the last 30 days, if 1 in 5 merged game PRs or more changed a file
under `engine/`, the API is still moving. In that case 1131 also sets up
a local checkout of the engine repository (git-ignored, at the pinned
revision, as `assets-private/` is) and a Cargo `[patch]` that points at
it, so a change to both sides stays one working copy.

## Consequences

- The game gains from the carve even if the engine is never published:
  `Action`, `Ctx` and the frame become smaller, generic and tested on
  their own; the Harness and `frame-png` work for any screen.
- The carve is mechanical but wide: about a third of the codebase moves
  (`docs/engine/boundary.md`), and the `Action` and `Ctx` changes touch
  most screens. It costs about 15 tickets, run one at a time, during which
  other `ui` work must rebase.
- Until the code moves, the engine's users are us. A claim that it
  supports another kind of game is true only once an example of that kind
  runs in its examples (engine ADR-0003).
- Plans and code live apart until 1131. The engine repository's tickets
  name this repository's as `game#NNNN`, and the carve's tickets stay
  here because they change this repository's code.
- After 1131, a change that needs both sides costs two PRs and a pin bump.
  The local checkout in §8 keeps that cheap while the API still moves.
- A second consumer (the samples, later other people's games) means the
  engine's API is a promise: breaking changes need a version bump and a
  note, which game tickets never needed.
- Each engine crate keeps the full gates (clippy pedantic, mutation
  testing, coverage); nothing in the engine is exempt because "it's just
  tooling".

## Alternatives considered

- **All code in two repositories now** — every engine-touching game
  ticket would be two PRs and a pin bump while the carve changes the API
  weekly, during the playtest work Nick put first. Rejected for the code;
  the engine's repository exists now for its plans (Nick, 2026-10-14),
  and §8 moves the code once the carve is done.
- **Keep one product; never split** — simplest, but loses Nick's goal,
  and the game's generic parts stay tangled with its rules (the closed
  `Action`, the game-wired `Ctx`), which already costs us in tests.
- **Build the engine on Bevy (as plugins)** — Bevy brings physics, tilemap
  and LDtk crates, an editor in progress, and a large audience. But it
  means rewriting `ui` and `app` around an ECS, the reasons ADR-0003 gave
  (compile times, WASM size) still hold, and our edge (state as plain
  data, a text-snapshot Harness, `frame-png`, a deterministic core) is
  harder to keep in an ECS world. A Bevy backend for `shuyi-app` stays
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
