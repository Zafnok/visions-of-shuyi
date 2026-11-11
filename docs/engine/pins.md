# Engine pins

The ledger of the game's engine pin ([ADR-0096](../adr/0096-a-watcher-on-the-engines-main-moves-the-pin-when-its-public-api-changes.md)):
one section per pin move, newest last, whether the engine watcher filed it
(an `engine-pin` issue) or a ticket made it (`engine-change` §3.6 and §7).
The `engine-pin` skill writes each section.

A section is headed `## YYYY-MM-DD: <from 8 hex> → <to 8 hex> (N engine
PRs), #<game PR>`. Its table has a row for every *Breaking*, *Unblocks* and
*New for games* engine PR the move passed, each with a decision:

- *Breaking:* `fixed: <what the game changed>`; for a PR the report also
  lists under *New for games*, `fixed: …; adopt → NNNN` or `fixed: …; not for
  the game: <why>`;
- *Unblocks:* `unblocks NNNN`;
- *New for games:* `adopt → NNNN` (a game ticket that moves the game onto
  the engine's piece, or an open ticket changed to use it; `adopt → NNNN
  (after engine#MMMM)` when the engine must change first), or `not for the
  game: <why>`.

A line under the table counts the rest: `Nothing for the game: N PRs (#…,
#…)`.

`cargo xtask engine-pin --check` (in CI) fails when the last section isn't
the pin in `shuyi-engine.rev`, when a row has no decision, when an
`adopt → NNNN` names no ticket in `tickets/open/` or `tickets/done/`, or when
a row of the last section has only `fixed:` for a PR that also adds public
items (it asks GitHub about those rows). Older sections aren't asked about.

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

## 2026-10-30: 50ae33db → 1eaa1fa0 (13 engine PRs), #490

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #179 | 2344 | Looped sounds with handles; `AudioRequest` and `SoundRequest` gain variants and are `#[non_exhaustive]`, `pack::VERSION` 5 | fixed: nothing to change (the game's matches on them have wildcards or are `if let`, it bakes no packs and keeps no golden row of its manifest); not for the game: `docs/design/audio.md` has no looping sound, only music, so a loop waits on a design decision |
| #180 | 2395 | The Key bindings screen's glyph skin scrolls rows that don't fit | unblocks 1198 |
| #181 | 2394 | Sounds paused under a pause menu, audio held on suspend; `Audio` gains `suspend` and `resume` | fixed: nothing to change (the game has no `Audio` backend: the Playdate's handles requests itself); not for the game: pausing needs a looped sound (#179). `shuyi-app` now mutes the music and the voice on `Suspend`, with no game code |
| #182 | 2229 | Import settings beside a file (`.meta`); `RegisterError` gains variants | fixed: nothing to change (the game matches no `RegisterError`); not for the game: it bakes no packs, and its one setting per picture is `crates/content/src/playdate_images.rs`'s `Style` (see #190) |
| #183 | 3004 | `shuyi export <target>`; `contract::Op` gains `Export` | fixed: nothing to change (the game matches no `Op`); adopt → 0910 (after engine#3075) |
| #186 | 3048 | A game's identity in `shuyi.ron` and `AppInfo`; `BasicGame::new` takes it, `Config` and `TargetCheck` are `#[non_exhaustive]` | fixed: nothing to change (the game doesn't use `BasicGame` and builds no `Config` literal); adopt → 1199 |
| #187 | 3053 | Platform facts at run time; `Ctx::keyboard` is `ctx.platform.keyboard` | fixed: `ctx.platform.keyboard` in `crates/ui`, `crates/playdate` and `crates/xtask`, and their tests; adopt → 1200 |
| #190 | 3007 | One-bit sprites at bake time, per-target layers; `BakedFile`, `BakeInput` and `PackMade` are `#[non_exhaustive]` | fixed: the new `one-bit picture` kind and `onebit` section in the registry listing of `crates/content/src/formats/tests.rs` (the game bakes no packs and registers no `onebit` section); not for the game: Nick decided its pictures are stretched then dithered in a 4 × 4 Bayer (`docs/design/playdate.md`), which `crates/content/src/playdate_images.rs` does with the engine's `bitmap::convert`; the new ditherers are Atkinson and an 8 × 8 Bayer, and the bought art can't be in a layer of this repository (ADR-0040) |
| #193 | 2115 | Timers in steps or turns: `Countdown` and `Every` in `shuyi_core::timer` | not for the game: `crates/core` keeps no countdown in turns or steps today, and the screens' motion is drawn from the time alone. A rule that needs one later (a status that wears off) can use it |

Nothing for the game: 4 PRs (#184, #189, #191, #195). The skill pack's `shuyi-new-game`, `shuyi-ship`, `shuyi-stage-field` and `shuyi-stage-grid` gain identity, export, one-bit bakes and timers.

Also passed, unsorted by the report because its commit names no PR on the engine's side: engine #188 (3072), the game's icon in `shuyi.ron` and the window's icon from `AppInfo`: adopted with #186, in 1199.

## 2026-11-01: 1eaa1fa0 → 0ad02fa5 (13 engine PRs), #503

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #197 | 3010 | `shuyi-playdate`, the Playdate backend, and `shuyi export playdate`; the default notch is 45°, `ExportCx` gains fields, `Look` holds a `&dyn Images` | fixed: nothing to change (the game's keymap names its own 30° notch, it builds no `ExportCx` and moves no `Look` across threads); adopt → 1170 (after engine#3028) |
| #201 | 3029 | The audio player's rules shared by every backend, and the Playdate's audio; `Sdk` and `export::playdate::Dirs` gain members | fixed: nothing to change (the game implements no `shuyi_playdate::Sdk` and builds no `Dirs`); adopt → 1170 (after engine#3028) |
| #205 | 3080 | `shuyi web` builds in clean staging and replaces its folder; `export web` refuses stale files | adopt → 0910 (nothing in the game places files in `dist/web` by hand) |

Nothing for the game: 10 PRs (#196, #198, #199, #200, #202, #203, #204, #207, #208, #209). #198 and #209 are only the engine's tickets 3075 and 3113, which 0910 and 1201 still wait on. The skill pack's `shuyi-ship` gains the Playdate's package.

The notes on what the backend does went into 1170, 1196, 1048 and 1052. `cargo xtask engine-pin` didn't move the `shuyi` crate added by 1199, so the build held two engines; `crates/xtask/src/engine.rs` now lists it, and a test checks that list against the real `Cargo.toml`.

## 2026-11-01: 0ad02fa5 → 76e05909 (17 engine PRs), #513

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #215 | 2410 | `PadKind::Playdate` names `East` A, `West` B and `South` X | fixed: the Playdate's A and B moved to `East` and `West` in `crates/playdate/src/pads.rs`, `assets/data/keymap.ron`'s `playdate` table, `targets/playdate.ron` and their tests; the player sees the same names |
| #212 | 3075 | `shuyi export` builds named features and ships files beside the program; `ExportCx` gains fields | fixed: nothing to change (the game builds no `ExportCx`); adopt → 0910 |
| #210 | 3113 | A game's icon with pictures drawn per size | unblocks 1201 and its `engine-friction: engine#3113` marker |
| #214 | 3028 | The Playdate backend's game stack from the heap, and its high-water mark | unblocks 1170 |
| #211 | 3081 | `exposure-report.json` of every export, and a `disclosure` part that refuses a package | adopt → 1208 |
| #217 | 2812 | A new game's `AGENTS.md` points at the engine's docs (`{{engine.docs}}`) | not for the game: the placeholder is filled only when `shuyi new` writes a game, and this game's `AGENTS.md` was never written by it |
| #221 | 2220 | Hot reload of any asset kind, `shuyi run --watch` | adopt → 1209 |
| #206 | 2573 | Sound parity baselines and gates in the engine's CI; found that quad-snd's native loops aren't gapless | not for the game: the gates run on the engine's own probe and no game crate can call them. The game's music does loop natively, and only the engine's replacement (2210) can change that |

Nothing for the game: 9 PRs (#175, #192, #213, #216, #218, #219, #220, #222, #223). #216 is research whose handoff is parked in the engine (3083); #218 is a tutorial for new games; #222 widens the engine's API gate, so `Breaking:` lines now cover `shuyi-playdate`. The skill pack's `shuyi-ship` gains `icon_sizes`, `exposure-report.json` and the `disclosure` part.

## 2026-11-02: 76e05909 → 1e3d2d39 (16 engine PRs), #524

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #234 | 2518 | Session records of everything that reaches a game, at the platform; `Start` is `non_exhaustive` | fixed: nothing to change (the game builds `Start` with `Start::new`); adopt → 1210 |
| #238 | 3114 | A dialogue line carries a direction; a voice clip remembers it | fixed: the game's tests that build `Say` and `Narrate`, and 0752 here; adopt → 0721 |
| #236 | 2575 | The Harness starts a game from its real `Start` and plays a session record to any frame | adopt → 1210 |
| #232 | 2210 | `shuyi-sound`'s PCM and source core, with WAV and Vorbis decoders | not for the game: no backend plays through it yet, and the game plays sound only through a backend |
| #237 | 3030 | `shuyi-sound`'s mixer | not for the game: no backend plays through it yet (its own PR: quad-snd's path is unchanged) |
| #225 | 2242 | Hot reload of the pictures a backend uploaded | adopt → 1209 |

Nothing for the game: 10 PRs (#176, #185, #224, #226, #228, #229, #230, #231, #233, #235). #176 moves the engine's `image` crate; the game's pictures load the same, and the gates ran on it. #235 is research.

## 2026-11-03: 1e3d2d39 → 96e97bf0 (2 engine PRs), #529

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #239 | 2519 | Session records from real builds: debug builds keep their last sessions, the debug menu's "Save session" (a download on the web), and `shuyi run`, `scene` and MCP play them; `SceneOutput` and `StartOutput` gain `session` | fixed: nothing to change (the game builds neither output); adopt → 1210 (in this PR: the game's start moved to `trpg_ui::start`, `session_from` in the tools, the battle's state in the frame hash and a kept record) |

Nothing for the game: 1 PR (#244), a `pulldown-cmark` advisory in the engine's own tools. The skill pack's `shuyi-debug-replay` gains reproducing a bug from a session record.

## 2026-11-03: 96e97bf0 → 2d2166f6 (6 engine PRs), #537

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #250 | game 1086 | The literal-name check finds each name's form once per scene; the longest name once per scene and per pack file; `ScriptsCheck` checks scripts a scene at a time | adopt → 1086 (in this PR: `trpg_content::StagedLoad` and `dialogue::ScriptsLoad` check the Playdate's scenes over several update callbacks) |
| #240 | 3031 | The in-house sound backend behind `shuyi-app`'s `in-house-sound` (CPAL on Windows, a Web Audio worklet), off by default; `WavSource` and `VorbisSource` take what they read as a type | fixed: nothing to change (the game uses neither source); adopt → 1241 (after engine#3084) |

Nothing for the game: 4 PRs (#245, #247, #248, #249). #245 and #249 are the sound-parity gates' and the in-house web backend's CI, which the game doesn't build; #247 and #248 are engine tickets only.

## 2026-11-04: 2d2166f6 → 1a84a71e (2 engine PRs), 0826

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #253 | 3208 | Voices on or off and their volume are player settings: `Settings::voices` and `voice_volume` (0–100), `Ctx::has_voices`, voices off stops the clip playing, `OptionRow::voices` and `voice_volume`, `Harness::voice_volume`; `Ctx::voices_on`, `voice_volume`, `voice_gain` and the voice volume constants are gone | fixed: the game's tests read and change the settings; adopt → 0826 (in this PR: the saved settings' version 2, the Options rows and the New Game question) |

Nothing for the game: 1 PR (#251), the sound gate's retry on Windows timings, in the engine's own CI.

## 2026-11-04: 1a84a71e → b7a4674e (11 engine PRs), #547

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #256 | 2350 | The font atlas keeps the characters the text uses, and `shuyi check` fails on text it can't draw; `font_atlas::build_atlas` and `run` take an `AtlasChars` | fixed: the xtask's up-to-date atlas test passes `AtlasChars::default()`, the atlas of before; the engine's `.dlg` parser now takes any character but a control character, so the game reads `.dlg` files through its own `AsciiDlg` format and `dialogue::parse_dlg`, which keep ADR-0045's plain-ASCII text with the same errors; adopt → 1243 |
| #264 | 2301 | Wide glyphs, two cells per character, from a wide font in the atlas; `text_width` and `cut` take the widths, `FontAtlasDef` has `wide`, packs are version 8 | fixed: `trpg_content::lang::text_width` and `trpg_ui::screens::playdate::cut` measure with `CellWidths::NARROW`, the xtask icon test's atlas has `wide`; adopt → 0236 |
| #257 | 2137 | Real time's first step: `TickSim`, tick records that replay, `TickDriver` and its input latch; `FrameInput` gains presses, releases and `resumed` | not for the game: its rules are turns over `TurnRules`, with no stage that steps in fixed ticks |
| #260 | 2411 | A key or button let go while the game is suspended stays held after the resume: `InputState::release_all`, called on the first frame after `Game::set_resumed` | adopt → 1170 (the desktop and web builds get it through `shuyi_app::run`; the Playdate's shim never forwards `kEventResume`) |

Nothing for the game: 7 PRs (#246, #254, #255, #258, #259, #261, #265). #246, #258, #261 and #265 are the sound gate's and `shuyi-sound`'s mutation testing in the engine's own CI; #254, #255 and #259 are engine tickets only (#259's voice cast and clip plan are 1242's).

## 2026-11-05: b7a4674e → e2697683 (14 engine PRs), #557

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #267 | 2252 | The text checks measure with the font atlas's wide glyphs: `PackRules` and `TipRules` gain `widths`, `Names::longest` and `lines_at_longest` take them | fixed: `trpg_content`'s pack and tip rules and the talk shot's check pass `CellWidths::NARROW`, as `text_width` already did (the game's font has no wide glyphs); adopt → 0236 |
| #268 | 3042 | Focus on the desktop and the web: `FocusLost` and `FocusGained`, held keys and pads let go on focus loss, pads ignored without focus; `PadInput::poll` takes the focus | fixed: nothing to change (the game doesn't call `PadInput::poll`); adopt → 1246 |
| #269 | 3093 | A background policy the game sets: paused or muted while its window has no focus | adopt → 1246 (Nick chooses the policy) |
| #274 | 2399 | Pictures a skin paints at run time; `glyph_buffer::Item` gains `Painted` and is `#[non_exhaustive]` | fixed: the sprite skin's and the dialogue log's tests match `Item` with a `_` arm; not for the game: ADR-0038 keeps every picture a sprite item from an asset file, and the Playdate's one-bit pictures are baked (1221), never painted at run time |
| #271 | 2302 | Line breaking for text without spaces (Japanese), and `lang-status`'s report of translated text too long for its place; `word_wrap`, `TextBox::new` and `lang_status::report` take the widths | fixed: every `word_wrap` and `TextBox::new` in `crates/ui` and the xtask's `lang-status` tests pass `CellWidths::NARROW`; adopt → 0237 (its pin step struck) |
| #275 | 3046 | Crash reports: `shuyi-app`'s panic hook writes a report through the game's storage with the game's record, and the game hears of it at the next start | adopt → 1245 |
| #276 | 2226 | Language tags in BCP 47, a fallback chain per key, and a source language the game declares; `ENGLISH`, `LangCode::english` and `Lang::english` deprecated | fixed: the game reads its source language as `LangCode::default()` and its text by `source_text`; adopt → 0825 |
| #273 | 2566 | One input script for the Harness, the contract and the live link, with actions by name | fixed: nothing to change (the game builds no `Step` or `Script`); adopt → 1244 (its bench scenarios play actions by name) |
| #279 | 3094 | The grid template's crash record through `BasicGame` | not for the game: it isn't on `BasicGame` (its own `Game` and `AppConfig`); its record is 1245's |
| #278 | 2253 | `best_match`: the game's language that fits the system's languages | adopt → 0825 (its first-launch step uses it) |
| #266 | 3090 | The Playdate backend's run report | unblocks 1170 (its pin step struck) |
| #270 | 2398 | Stop music with a fade chosen per request | unblocks 1087 (its pin step struck) |
| #272 | 2109 | Ease curves in integers in `shuyi_core::ease` | unblocks 1228 (its pin step struck) |
| #277 | 3008 | `shuyi bench`: scenarios played headless, gated against every target's budgets | unblocks 1244 (its pin step struck) |

Nothing for the game: none.

## 2026-11-07: e2697683 → 02dbce1d (16 engine PRs), #572

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #283 | 2303 | Wide glyph sets per language, punctuation wide in one language only, Chinese line breaking; `FontAtlasDef` gains `wide_sets`, `LangInfo` `wide_font` and `wide_punctuation`, `font_atlas::run` takes `&[WideFont]`, packs are version 9 | fixed: the xtask icon test's atlas has `wide_sets`, the up-to-date atlas test passes no wide fonts (`&[]`); 0239's pin step struck |
| #298 | 2582 | The contract's answers list the sounds a game added at run time (`EventsOutput::sounds_added`) | fixed: nothing to change (the game builds no `EventsOutput`) |
| #295 | 2391 | One RGBA picture type: `glyph_tiles` and `font_atlas` give `shuyi_assets::image::Image`, `raster::of_painted` is a free function | fixed: nothing to change (the game's tools call none of them) |
| #291 | 2250 | A voice cast a voice tool can use: `Cast<S>` with `CastVoice<S>`, `cast_from_source` generic over the tool's settings | fixed: nothing to change (the game's voice tool reads its own cast today; 1242 moves it onto this) |
| #297 | 2544 | `bots` folded into `playtest`: `GameTools::bots` and `contract::bots` gone, a bot run is a `PlaytestPlan` with one entrant and no checks, a try gives `PlayedTry` | fixed: `Trpg::bots` became `Trpg::playtest_plan` (`plays::plan`: the `baseline` entrant in Classic on the stage asked for, else the story's first battle, no checks, each try's `PlayRecord` kept), autobalance's try is `play_try`, and the two 1193 tests run through `playtest`; adopt → 1240 (the record through `Entrant::record`, the copied loop deleted) |
| #280 | 3095 | A panic the game survives is reported but not offered as a crash: `CrashReport::fatal`, `offer_last_crash` offers only fatal reports | adopt → 1245 (its open PR's crash offer rests on `offer_last_crash`, which now skips a panic on another thread) |
| #284 | 2315 | The debug view: gizmos from `Screen::debug_draw` in channels, the debug menu's "Debug view", the grid kit's `grid` channel, saved under `debug_view` | fixed: the debug menu's new last entry moves the tests that reach a tool by Up (cinematic, scene camera, sprite test, voice test, map skin, the sessions' "Save session", the xtask camera frame), the debug menu snapshot, the unreadable-storage warnings (`debug_view`) and the kept Quick Battle session (written again: its script presses Up once more); adopt → 1252 |
| #285 | 2249 | Sounds a game makes at run time: `Samples`, `AudioQueue::add_sound` and `remove_sound` on every backend | not for the game: every sound it plays is an authored file with its credit in `audio.ron` (ADR-0013), and the Playdate's are baked at build time (1221); no design asks for a sound made while it runs |
| #300 | 2254 | A source language names its wide set and its punctuation's width (`SourceLang::with_wide_font`, `with_wide_punctuation`) | not for the game: its source language is English, which draws no wide glyph; Japanese and Chinese are packs with their own `lang.ron` (0236, 0239) |
| #301 | 2255 | Screens know when a voice clip has ended | adopt → 0863 (written here: 0859's auto-advance waits for the real end instead of its estimate) |
| #294 | 3210 | The options panel rises when its rows leave no room for the message | fixed: the four in-battle Options snapshots show the panel one row higher; the rise is one row early (a panel that fitted moves too), filed as engine#3212 |

Nothing for the game: 5 PRs (#281, #282, #290, #296, #299). #281, #282 and #290 are engine tickets and research only; #296 is the engine's own API listing in CI; #299 gives Hello, Grid and the grid template the debug view's grid channel (1252 does the game's).

## 2026-11-07: 02dbce1d → c1ba9ec8 (1 engine PR), #574

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #302 | 3211 | A tip keeps its instructions apart from its text: `Tip::how`, `TipLimits::how_lines` | fixed: the tips' `LIMITS` sets `how_lines: 0`; adopt → 0475 (Vix's spoken lines name no key, the keys go in a tip's `how`) |

Nothing for the game: none.

## 2026-11-10: c1ba9ec8 → ab593bb9 (50 engine PRs), #624

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #306 | 2581 | Scene data derived from the game's state; `contract::run::SceneOutput` is `#[non_exhaustive]` with `value` and `shape` | fixed: nothing to change (the game builds no `SceneOutput`) |
| #304 | 2576 | A misspelled action in a language file fails `shuyi check` (`text::check_key_placeholders`, `SH2008`) | fixed: nothing to change (the game isn't on `BasicData`, so its language files load as before); adopt → 0253 (the UI text's 126 key placeholders are checked by nothing today) |
| #307 | 2388 | One-bit roles learn what the inks do: `RolePatterns` is `#[non_exhaustive]` with `edge` and `invert`, `Base` gains `Dots`, packs are version 10 | fixed: the xtask's Playdate skin builds its roles with `RolePatterns::of`; unblocks 1170 |
| #318 | 2342 | Sound groups: a master volume and a volume per group on a loudness curve; `Settings`' three volumes, `FrameOutput`'s and the Harness's are gone | fixed: the settings' saved form is version 3 with `volumes`, and a version 1 or 2 save's linear volumes load where they sound the same (its 80, the old default, as the group's default); the Playdate front end plays at the frame's music and sound gains (`Audio::set_gains`); the tests read the gains (`Harness::gain`). The Options rows show a default volume as 97 where they showed 80, the same loudness, so their snapshots moved |
| #326 | 2258 | A game's own sound groups in the audio manifest; `MusicCue` gains `group`, packs are version 11 | fixed: the two test `MusicCue`s add `group: None` |
| #335 | 3097 | A game's load runs over several update callbacks on the Playdate; `Start::load` is a `Loader`, packs are version 13 | fixed: nothing to change (the game's `Start` is built with `Start::new`); unblocks 1170 |
| #331 | 3099 | Release exports strip their symbols, refuse the debug features, keep the crash record only when asked and never skip `wasm-opt` silently | fixed: `shuyi.ron`'s `export` says `wasm_opt: optional` (no runner installs Binaryen, and before it was skipped), and the crash reports keep the battle's record in a release build (`CrashReports::with_record_in_release`, as 1245 shipped them); adopt → 0916 (Binaryen on CI's and the release's runners, `optional` gone) |
| #332 | 2343 | Ducking: a sound group lowered while a voice plays, as rules in the audio manifest; packs are version 14 | fixed: nothing to change (the game bakes no pack); adopt → 0865 (its pin step struck) |
| #344 | 3303 | Every debug feature refused in the resolved export build; `live_link::FEATURE` and `ENGINE` gone, `Violation` gains `feature` | fixed: nothing to change (the game names neither constant and builds no `Violation`); 0914 says what it gives the demo |
| #345 | 2586 | The dialogue preview screen: a game's scenes from a watched folder, with switches, lines, problems and reload | fixed: nothing to change (only Hello, Grid's golden rows moved); adopt → 0764 (its pin step struck) |
| #346 | 3082 | Named demo editions from separate code and content roots; `ExportCx`, `ExportOutput`, `BuildInputs` and `web::Options` gain fields | fixed: nothing to change (the game builds none of them field by field); unblocks 0914 (its pin step struck) and 0764's edition clause |
| #352 | 3092 | Files dropped on the window, handed to the game as a bundle layer; `SystemEvent` is no longer `Copy` | fixed: nothing to change (the game copies no `SystemEvent`); adopt → 0767 |
| #303 | 2251 | Plan a language's voice clips and each voice's recording script | unblocks 1242 (its pin step struck) |
| #309 | 3096 | `shuyi export playdate` builds the game's Playdate package, with its features | unblocks 1170 (now `todo`: 2388, 3096 and 3097 are all pinned) |
| #329 | 2101 | A deterministic, sandboxed script runtime: `shuyi-script` | unblocks 1017 (its pin step struck) |
| #293 | 3069 | Files from the page: a web build lays the page's files over its own and sends its problems back | adopt → 0767 (the page can hand the web preview a writer's files) |
| #322 | 3085 | Live-link features refused in the resolved export build | adopt → 0914 (the demo's export is refused if a debug feature or the live link reaches it by any path) |
| #333 | 2585 | A dialogue preview's pieces: a scene's conditions, a player from any text box, a line's direction, a folder watched in every build | adopt → 0764 |
| #350 | 3304 | The bake takes an edition's content root, so a Playdate edition ships only its own files | adopt → 1170 (what the Playdate leaves out, by a content root rather than `playdate-bundle.txt`, if its data comes from the pack) |
| #349 | 2587 | A dialogue preview on the short path: a `BasicGame`'s preview binary in one call | not for the game: it isn't on `BasicGame` (its own `Game` and `AppConfig`); 0764 builds its preview on the pieces (#333, #345) |
| #348 | 3305 | `NOTICES.txt` lists the crates of the build as Cargo resolves it | adopt → 0914 (a demo without the default features names none of their crates) |
| #351 | 3306 | A Playdate edition's sounds and music come from its content root; the exporter's audio goes through `bake` | adopt → 1170 (the exporter asks `GameTools::audio` through `bake`) |
| #355 | 2588 | A dialogue preview on the web: `.dlg` files dropped on the page | adopt → 0767 |
| #353 | 3308 | Filling a rectangle keeps the items it misses as they were | unblocks 1261 (its pin step struck; the `bench` numbers are its) |
| #313 | 3212 | The options panel rises only when its message wouldn't fit | fixed: the four in-battle Options snapshots show the panel one row lower again, where it was before #294 (the engine ticket the pin move of 2026-11-07 filed) |

Nothing for the game: 25 PRs (#305, #312, #311, #310, #308, #314, #316, #315, #317, #324, #325, #327, #328, #321, #320, #323, #334, #338, #339, #340, #341, #343, #342, #347, #354). #305 and #327 are engine tickets only; #308, #316, #317, #334, #339, #341 and #347 are the engine's book, tutorial and site; #310 is the templates' web build; #311, #312, #315, #320, #321, #323 and #325 are the engine's own CI and tooling; #314, #324, #328 and #338 are tickets and research; #340, #342 and #343 are Sonar fixes in the web shell's scripts with no change in what they do; #354 stops a folder dropped on the Windows window from crashing, which the game reaches only through #352's drops (0767).

## 2026-11-10: ab593bb9 → edb6c888 (1 engine PR), #PRNUM

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #356 | 2071 | No tracked path over 150 characters, so a Windows checkout of the engine stays under 260; `cargo xtask check-paths` fails a new one | unblocks 1262 (this PR: the short `CARGO_HOME` on Windows CI is gone) |

Nothing for the game: none.
