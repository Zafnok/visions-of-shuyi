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

## 2026-10-26: 6f0a8209 → 883ad00a (6 engine PRs), #445

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #122 | 3065 | Memory tags: live bytes by frame phase, span and asset kind's decode | adopt → 1068 (its device measurement wraps the Playdate's `SdkAlloc` in `CountingAlloc::with_tags` and tags decodes with `format::watch_decodes`, instead of counting per file kind itself; nothing to delete: the game counts no heap yet) |
| #134 | 2554 | Heap snapshots and diffs in `run --perf`, MCP `perf` and a running build's trace | not for the game: tooling it gets through `cargo shuyi` and the MCP server at the pin; no game code or ticket snapshots the PC heap |
| #135 | 2555 | Perf over the live link: `shuyi attach --perf-follow`, MCP `perf` on an attached session | not for the game: tooling a debug build on `shuyi-app` gets at the pin; the game reads no frames over the link |

Nothing for the game: 3 PRs (#123, #136, #137). The skill pack's `shuyi-perf` gains its heap and running-build sections.

## 2026-10-27: 883ad00a → 1d284a18 (11 engine PRs), #451

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #138 | 2513 | `shuyi doctor` checks the C linker and system libraries; `Probe::read` | fixed: nothing to change (the game implements no `Probe`) |
| #139 | 3039 | Web builds carry a `NOTICES.txt`; `web::run` takes the game's works, `project::Config` gains `notices` | fixed: nothing to change (the game calls no `web::run` and builds no `project::Config` literal; `cargo shuyi web` loads `shuyi.ron`, which names no `notices`) |
| #144 | 2542 | The CLI's seams: each output schema's `op` is a const of its op | fixed: nothing to change (the game reads no output schema's `op`) |
| #148 | 2321 | A frame cap: `FrameOutput` is `#[non_exhaustive]` with `frame_cap` and `frame_cap_serial` | fixed: nothing to change (the game builds no `FrameOutput` and reads it field by field) |

Nothing for the game: 7 PRs (#140, #141, #142, #143, #145, #146, #147). The skill pack's `shuyi-ship` gains the web build's `NOTICES.txt`, and `shuyi-new-game` the Linux setup script.

Ticket 0909 (from #139): the web build's `NOTICES.txt` lists none of the game's own works yet; 0909 names the font, the music and their license texts in `shuyi.ron`'s `notices`. 0901 waits on it.

## 2026-10-28: 1d284a18 → c9fb84c9 (11 engine PRs), #465

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #149 | 2207 | Bundle layers from outside the binary; `bundle::Files` is `#[non_exhaustive]` with an `Owned` variant | fixed: nothing to change (the game matches no `Files` and builds its layers with `Layer::dir`, `Layer::table` and the builders) |
| #150 | 2332 | The raster: frame-png's `Painter` moved into `shuyi-ui`; `Image::decode` became `frame_png::decode_image` | fixed: `frame_png::decode_image` in `crates/xtask`'s `game_tools/tests.rs` |
| #152 | 2111 | Stamped records and saves: `pack::VERSION` 3 with `PackInfo::content`, `Layer::stands_for`, `ReplayRun::made` | fixed: nothing to change (the game bakes no packs and builds no `PackInfo`, `Layer` or `ReplayRun` literal) |
| #155 | 2240 | `dialogue::Named`: the hooks of a game whose only tokens are names | not for the game: its `Hooks` (`crates/ui/src/dialogue.rs`) also fill the lead's name and pronouns and play `@place` and `@opening` lines, so they stay its own `DialogueHooks` (they already fill names with `Names::substitute`) |
| #157 | 2568 | A game's start options checked once, on every tool path | not for the game: the tools at the pin check the options `Trpg` declares before its `start` runs; no game code or ticket changes |

Nothing for the game: 6 PRs (#151, #153, #154, #156, #158, #159). The skill pack's `shuyi-debug-replay` gains stamped records, and `shuyi-dialogue` the `Named` hooks.

Ticket 1195 (from #152): the game's tools give no `made` stamp yet, so its bots' play records replay unchecked against the build and content that made them; 1195 stamps them and passes the stamp on in `replay`.

Ticket 1196 (from #153, listed under nothing for the game by its `api: none` label): the engine's storage now runs a power-cut suite, and the Playdate's own `FileStorage` doesn't. Its write unlinks the old save before moving the new one in when the file system won't rename onto a file, so a power cut between the two loses the save. 1196 runs the suite on it and fixes what fails. Ticket 1197 widens this review after an audit of every section above: a breaking PR's features, and `api: none` PRs that change behavior, get decisions too.

## 2026-10-29: c9fb84c9 → 50ae33db (13 engine PRs), #482

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #162 | 2540 | frame-png scripts wind the crank: a `--crank DEGREES` step; `frame_png::Step` gains `Crank` and is `#[non_exhaustive]` | fixed: the game's own `--crank` option deleted from `crates/xtask/src/game_tools.rs` (the engine's parser now takes the flag first, so the game's could no longer be reached; the rewind picture of 1039's test comes from the engine's step); unblocks 1182, which deletes the rest (`--then-pad`) |
| #168 | 3056 | One-bit sprites drawn fast | unblocks 1078 |
| #174 | 2341 | Music crossfades and fade-ins: `AudioRequest::PlayMusic` gains `switch`, `MusicCommand::Start` gains `gain`, `AudioManifest` gains `music_fade_in_ms` and `music_overlap`, `pack::VERSION` 4 | fixed: `AudioRequest::play_music(cue, None)`, `MusicCommand::start(cue, 1.0)` and `{ cue, .. }` patterns in `crates/ui` and `crates/playdate`; the Playdate's backend starts a track at `Start`'s gain (always 1 with the default switch, so nothing sounds different); nothing else (the game builds no `AudioManifest` literal, bakes no packs and keeps no golden row of its manifest); unblocks 1080 |

Nothing for the game: 10 PRs (#161, #163, #164, #165, #166, #167, #169, #171, #172, #173). The skill pack is unchanged.

## 2026-10-30: 50ae33db → 1eaa1fa0 (13 engine PRs), #PRNUM

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #179 | 2344 | Looped sounds with handles; `AudioRequest` and `SoundRequest` gain variants and are `#[non_exhaustive]`, `pack::VERSION` 5 | fixed: nothing to change (the game's matches on them have wildcards or are `if let`, it bakes no packs and keeps no golden row of its manifest); not for the game: `docs/design/audio.md` has no looping sound, only music, so a loop waits on a design decision |
| #180 | 2395 | The Key bindings screen's glyph skin scrolls rows that don't fit | unblocks 1198 |
| #181 | 2394 | Sounds paused under a pause menu, audio held on suspend; `Audio` gains `suspend` and `resume` | fixed: nothing to change (the game has no `Audio` backend: the Playdate's handles requests itself); not for the game: pausing needs a looped sound (#179). `shuyi-app` now mutes the music and the voice on `Suspend`, with no game code |
| #182 | 2229 | Import settings beside a file (`.meta`); `RegisterError` gains variants | fixed: nothing to change (the game matches no `RegisterError`); not for the game: it bakes no packs, and its one setting per picture is `crates/content/src/playdate_images.rs`'s `Style` (see #190) |
| #183 | 3004 | `shuyi export <target>`; `contract::Op` gains `Export` | fixed: nothing to change (the game matches no `Op`); adopt → 0910 (after engine#3075) |
| #186 | 3048 | A game's identity in `shuyi.ron` and `AppInfo`; `BasicGame::new` takes it, `Config` and `TargetCheck` are `#[non_exhaustive]` | fixed: nothing to change (the game doesn't use `BasicGame` and builds no `Config` literal); adopt → 1199 |
| #187 | 3053 | Platform facts at run time; `Ctx::keyboard` is `ctx.platform.keyboard` | fixed: `ctx.platform.keyboard` in `crates/ui`, `crates/playdate` and `crates/xtask`, and their tests; adopt → 1200 |
| #190 | 3007 | One-bit sprites at bake time, per-target layers; `BakedFile`, `BakeInput` and `PackMade` are `#[non_exhaustive]` | fixed: the new `one-bit picture` kind in the registry listing of `crates/content/src/formats/tests.rs` (the game bakes no packs and registers no `onebit` section); not for the game: Nick decided its pictures are stretched then dithered in a 4 × 4 Bayer (`docs/design/playdate.md`), which `crates/content/src/playdate_images.rs` does with the engine's `bitmap::convert`; the new ditherers are Atkinson and an 8 × 8 Bayer, and the bought art can't be in a layer of this repository (ADR-0040) |
| #193 | 2115 | Timers in steps or turns: `Countdown` and `Every` in `shuyi_core::timer` | not for the game: `crates/core` keeps no countdown in turns or steps today, and the screens' motion is drawn from the time alone. A rule that needs one later (a status that wears off) can use it |

Nothing for the game: 4 PRs (#184, #189, #191, #195). The skill pack's `shuyi-new-game`, `shuyi-ship`, `shuyi-stage-field` and `shuyi-stage-grid` gain identity, export, one-bit bakes and timers.

Also passed, unsorted by the report because its commit names no PR on the engine's side: engine #188 (3072), the game's icon in `shuyi.ron` and the window's icon from `AppInfo`: adopted with #186, in 1199.
