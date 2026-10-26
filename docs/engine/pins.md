# Engine pins

The ledger of the game's engine pin ([ADR-0096](../adr/0096-a-watcher-on-the-engines-main-moves-the-pin-when-its-public-api-changes.md)):
one section per pin move, newest last, whether the engine watcher filed it
(an `engine-pin` issue) or a ticket made it (`engine-change` §3.6 and §7).
The `engine-pin` skill writes each section.

A section is headed `## YYYY-MM-DD: <from 8 hex> → <to 8 hex> (N engine
PRs), #<game PR>`. Its table has a row for every *Breaking*, *Unblocks* and
*New for games* engine PR the move passed, each with a decision:

- *Breaking:* `fixed: <what the game changed>`;
- *Unblocks:* `unblocks NNNN`;
- *New for games:* `adopt → NNNN` (a game ticket that moves the game onto
  the engine's piece, or an open ticket changed to use it; `adopt → NNNN
  (after engine#MMMM)` when the engine must change first), or `not for the
  game: <why>`.

A line under the table counts the rest: `Nothing for the game: N PRs (#…,
#…)`.

`cargo xtask engine-pin --check` (in CI) fails when the last section isn't
the pin in `shuyi-engine.rev`, when a row has no decision, or when an
`adopt → NNNN` names no ticket in `tickets/open/` or `tickets/done/`.

## 2026-10-25: baseline → 3f2256d4

The pins before this one were moved by tickets (1177 and 1188, both
2026-10-22) without this review. The engine watcher's first report starts
from here.

## 2026-10-25: 3f2256d4 → b4591a81 (29 engine PRs), #425

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #88 | 2013 | Window, AppConfig, BoxPlace, MapScene and others moved onto constructors | fixed: `Window::new(..).with_icon`, `Icon::new`, `AppConfig::new` and `VoiceFiles::new` in `crates/app`; `BoxPlace::new(..).with_narration` and `MapScene::new` in `crates/ui`, whose `every_field_is_listed` (`map_view/sprite.rs`) now checks the scene's and the cursor's fields by name, since they can't be destructured |
| #90 | 2024 | The engine's other config and views moved onto constructors | fixed: `Looks`, `MapColors`, `TerrainGlyphs`, `TextView`, `WaitView`, `CursorView` and `RowValue` built with their constructors in `crates/ui`; `Look::new` in `crates/app`; `Files`, `SoundCue`, `MapFormat`, `TerrainTiles`, `DialogueTable`, `ImageTable` and `ImageInfo` in `crates/content`, `crates/ui`, `crates/playdate` and `crates/xtask`; `Check::rate`, `Check::median` and a `_` arm in `crates/bots` |
| #91 | 2025 | The contract's inputs and the game tools' types moved onto constructors | fixed: `GameOption`, `GameLang`, `Shot`, `Scene` and `Script` built with their constructors in `crates/xtask` |
| #93 | 2358 | Modules join a running game | fixed: nothing to change (the game builds no `RunOutput`) |
| #97 | 2003 | Modules: the catalog, the `shuyi` crate, `shuyi modules` and `shuyi add` | fixed: nothing to change (the game calls no `BasicGame::game` and matches no `contract::Op`) |
| #98 | 2501 | Templates as data behind `shuyi new` | fixed: nothing to change (the game calls no `NewInput`) |
| #101 | 1134 | The engine's skill pack | fixed: nothing to change (the game builds no `Catalog`, `Plan` or `NewOutput`); taking the pack is 1189's |
| #105 | 2533 | `shuyi playtest` keeps a history and says what changed | fixed: nothing to change (the game builds no `Played`, `PlaytestOutput` or `PlaytestArgs`) |
| #106 | 2021 | A reusable playtest workflow | fixed: nothing to change (the game builds no `Fill`) |
| #77 | 2546 | The MCP server follows the session's checkout | unblocks 1189 |
| #102 | 2131 | Threat areas in the tactics kit | unblocks 1180 |
| #104 | 2132 | A planner's candidate order from the game's kinds of command | unblocks 1180 |
| #76 | 2545 | A running MCP server never blocks the next build | adopt → 1189 (its `mcp install` writes `MCP_TIMEOUT`, and its check by hand covers the case 2545 couldn't: server and tool program one binary) |
| #78 | 2511 | The short path: engine defaults, a prelude, `run_game`, `BasicTools` | not for the game: it is on the long form with every part its own (data, settings, voices, themes, its window and tools); the short path is for new games |
| #95 | 2372 | A module's debug menu entries | not for the game: the game has no modules |

Nothing for the game: 14 PRs (#75, #79, #80, #83, #84, #89, #92, #94, #99, #100, #103, #111, #112, #113). #83 and #89 moved the engine to Rust 1.99, so the game's toolchain moves with it (`rust-toolchain.toml`, `rust-version`), and clippy 1.99's new `assert_is_empty` lint is fixed as it suggests in the game's tests.

## 2026-10-25: b4591a81 → 555bba71 (2 engine PRs), #432

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #115 | 2559 | check-skills checks a project's own skills without asking for a worked example | unblocks 1194 |

Nothing for the game: 1 PR (#114).

## 2026-10-25: 555bba71 → d04bc2d1 (2 engine PRs), #434

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #117 | 2521 | The live link: `shuyi attach` and MCP read a running windowed build | fixed: nothing to change (the game matches no `contract::Op`) |
| #116 | 3063 | Frame times and spans as data: the platform's clock, `Ctx::span`, `App`'s record of each frame | adopt → 1057 (its bench build times a battle frame's parts with the engine's clock, spans and `FrameRecord`s instead of marks of its own; nothing to delete: the game times no part of a frame yet, and the device report's whole-frame and render times stay) |

Nothing for the game: 0 PRs.

## 2026-10-26: d04bc2d1 → 6f0a8209 (6 engine PRs), #440

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #118 | 2560 | MCP's session attached to a running build over the live link | fixed: nothing to change (the game builds no `StartInput` or `StartOutput`) |
| #119 | 2553 | Perf traces agents read: `shuyi run --perf`, an MCP `perf` tool | fixed: nothing to change (the game matches no `contract::Op`) |
| #120 | 3064 | The heap counter in debug builds | fixed: nothing to change (the game names no `DEFAULT_CRATES`; its only `#[global_allocator]` is the Playdate crate's, which doesn't link `shuyi-app`) |

Nothing for the game: 3 PRs (#86, #87, #121). The skill pack gains `shuyi-perf`.
