---
id: "1116"
title: "Tactics kit: playtest bots over any TurnRules, and the .map text format"
type: infra
milestone: "Engine: carve"
model: implementation
effort: medium
status: done
blocked_by: ["1114"]
nick_input: none
completed: 2026-10-16
---

# 1116 — Kit: bots over any `TurnRules`; the `.map` format

## Context

The playtest bots (ADR-0033, `crates/bots`) are all this game's today:
they use `BattleState`, `Command`, `AiWeights` and the rest. The shape
is generic and about 200 lines: a `PlayerBot::choose` trait, a
`play_battle` loop, and the measures pattern. After 1114 it can be
written over `TurnRules`.

The `.map` text format (`content/src/map.rs`: a RON header, `---`, one
character per tile, a legend, a look and light) is the kit's. Its Shop
and Chest features (`map.rs:136-157`) are the game's.
`docs/engine/boundary.md`, *bots* and *content*.

## Nick input

None.

## Scope

**In:**
- `shuyi_core::bots` holds three things:
  - `trait Bot<R: TurnRules> { fn choose(&mut self, &R) -> R::Command }`
  - `play<R>(rules, bots_by_side, limit) -> Report` (outcome, turns,
    commands)
  - a seeded random bot over `legal_commands()`
- `trpg-bots` keeps its bots and measures and uses the generic loop.
- `shuyi_assets::grid_map` holds the `.map` reader and printer, with a
  hook for the game's tile features. The game keeps Shop and Chest as
  its features. The printer round-trips: parse → print → parse gives
  the same map.

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6).

**Out (do not do):** the map editor (1019); bot personas (0506).

## Implementation steps

1. Write the generic loop. Port `trpg-bots`. `bots/tests/baseline.rs`
   gives the same results.
2. Move the `.map` format with a feature hook. Every map in `assets/`
   round-trips (property test over the real maps in the game's tests;
   fixture maps in the engine's).

## Acceptance criteria

- [x] A toy `TurnRules` game (tic-tac-toe in the engine's tests) is
      played to the end by two random bots, deterministically per seed.
- [x] The game's bot baseline is unchanged.
- [x] Every `.map` round-trips.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the loop with the toy game.
- Property: `.map` round trip.

## Completion notes

**What was done.**

- `shuyi_core::bots` (engine): `Bot<R: TurnRules>` (`choose(&mut self,
  &R) -> R::Command`), `play(rules, bots_by_side, limit) -> Report`
  (outcome, turns, commands) and `play_observed`, which also hands every
  applied command and its events to a callback (how a game builds its
  measures), `PlayError` (command, side, turn, error) and `RandomBot`, a
  seeded pick from `legal_commands()`. Tested over tic-tac-toe: two random
  bots play every seed to the end, the same seed gives the same game, all
  three endings happen, each side's bot is asked only for its side, the
  limit, a refused command, a missing bot.
- `trpg-bots` uses the loop: `play_battle` is `play_observed` with the
  player's bot on the Player phase and the AI (a `BaselineBot` with the
  enemy weights) on the Enemy and Other phases; `BattleMeasures` is the
  observer. `PlayerBot` is now any `Bot<BattleState>` (a blanket impl), so
  `xtask playtest` is unchanged.
- `shuyi_assets::grid_map` (engine): the `.map` reader (`parse`) and
  printer (`print`), `Legend<T>`, `MapLook` with its tile look and light,
  and the header position helpers. The game gives the terrain lookup (a
  closure), a `MapFormat` (its tile looks and size limit) and its feature
  type through the `MapFeature` hook (serde, plus an optional file-only
  `problem`). `NoFeature` is for a game without features.
- `trpg_content::map` keeps its API (`MapDef`, `MapLegend` = `Legend<TerrainId>`,
  `MapLook`, `parse_map`, `print_map`, `TILE_LOOKS`, `DEFAULT_TILES`) and
  its Shop and Chest features (`FeatureDef` implements `MapFeature`),
  the light and feature checks.

**Deviations.**

- `TurnRules` doesn't say whose move it is or which turn it is, and the
  loop needs both (which bot to ask, the limit, `Report::turns`). A small
  second trait, `shuyi_core::Sides: TurnRules` (`side_to_move`, `turn`),
  says it; `BattleState` implements it with its phase and turn.
- `play` takes `&mut R`, so the caller keeps the final state.
- The format-only tests of `content/src/map.rs` moved to the engine with
  the code (over fixture maps and a fixture `Sign` feature), with the
  round-trip property over generated maps now with features too. The
  game's tests keep the shop, chest and light tests and gain
  `every_map_round_trips` over every real map (`assets/` and the test
  files).
- The kit's map defaults are `"outdoor"` and `"noon"`;
  `content::lighting::NOON` and `content::map::DEFAULT_TILES` now name
  the kit's constants, so they can't drift.
- `shuyi-assets` now depends on `shuyi-core` (the `Grid` a map is read
  into), keeping its `no_std` build.

**Baseline unchanged.** `cargo xtask playtest ch01 --runs 200 --json` on
the code before and after the port: the two reports are the same in every
field but the date and speed (all 200 tries: result, turns, every fall,
items, commands). `baseline.rs` passes unchanged.

**Model:** worked on Opus 5.5 (the ticket says implementation).

**Gameplay rules decided:** none. Every battle plays identically.
