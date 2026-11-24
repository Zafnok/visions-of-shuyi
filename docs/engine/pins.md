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
  the game: <why>`. A PR with only `Behaviour:` lines (engine ADR-0108: a
  change the game sees without a break) is in this table too: `fixed:` says
  whether the game keeps its own rule or takes the engine's, and how
  (`fixed: nothing to change (…)` names what was searched);
- *Unblocks:* `unblocks NNNN`;
- *New for games:* `adopt → NNNN` (a game ticket that moves the game onto
  the engine's piece, or an open ticket changed to use it; `adopt → NNNN
  (after engine#MMMM)` when the engine must change first), or `not for the
  game: <why>`.

A line under the table counts the rest: `Nothing for the game: N PRs (#…,
#…)`.

`cargo xtask engine-pin --check` (in CI) fails when the last section isn't
the pin in `shuyi-engine.rev`, when a row has no decision, when an
`adopt → NNNN` names no ticket in `tickets/open/` or `tickets/done/`, when
a row of the last section has only `fixed:` for a PR that also adds public
items, or when an engine PR the last section's move passed has a `Breaking:`
or `Behaviour:` line and no row (it asks GitHub about those PRs, ticket
1254). Older sections aren't asked about.

The report (`cargo xtask engine-pin --report`, and the issue) also lists
the adoption tickets still open, once each with their age, *idle* past 5
days, and under *Workarounds the pin ends* each `engine-friction:
engine#NNNN` marker in `crates/` whose engine ticket the move passes, and
each open PR that adds one for such a ticket or one already pinned. The
pin PR removes each marker, or its row names the ticket that does.

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

## 2026-11-10: ab593bb9 → edb6c888 (1 engine PR), #627

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #356 | 2071 | No tracked path over 150 characters, so a Windows checkout of the engine stays under 260; `cargo xtask check-paths` fails a new one | unblocks 1262 (this PR: the short `CARGO_HOME` on Windows CI is gone) |

Nothing for the game: none.

## 2026-11-11: edb6c888 → 97313b9e (5 engine PRs), #638

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #359 | 3309 | Files the player keeps: `Ctx::offer_file` and `Ctx::ask_for_file`, answered as system events, and where a platform's storage lives; the web shell's `download.js` is `player-files.js` | fixed: the release package check names `player-files.js` in the web shell (and `page-files.js`, shipped since #293 and never listed); unblocks 0872 and 0873 (engine#3309 out of their `blocked_by`, their pin steps struck for it; engine#2138 still blocks both) |
| #362 | 2389 | The inks removed: one-bit screens draw only by roles' patterns; `shuyi_assets::ink` and `PaletteDef::from_ink_source` gone, the renderer's `_with` forms are its only forms | fixed: `Ink` and the Playdate palette's loader move into `trpg_content::ink` (the game's palette file names inks; `Ink::patterns` and `Ink::is_white` read the engine's patterns), the `render*_with` calls are `render*`, and the tests that drew with inks alone draw with the Playdate's pattern map; no frame or snapshot changed; adopt → 1263 (the palette drops the inks and the map alone draws it, about 110 lines removed) |

Nothing for the game: 3 PRs (#358, #360, #361). #358 is engine tickets only; #360 and #361 are the engine's tutorial tries and book.

## 2026-11-11: 97313b9e → 85e8d5c5 (2 engine PRs), #641

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #367 | 2138 | Save bundles: a storage's keys written as one checked text file, read back with a verdict per entry, and applied all or nothing | unblocks 0872 and 0873 (engine#2138 out of their `blocked_by`, their pin steps struck) |

Nothing for the game: 1 PR (#363), the engine's own ticket skills.

## 2026-11-11: 85e8d5c5 → 234de697 (12 engine PRs), #646

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #377 | 2514 | Frames over time from `frame-png`: `--every` and `--count` make a sheet labelled with times, an APNG or a GIF; the MCP `frame_png` tool returns the sheet | fixed: nothing to change (the game builds no `frame_png::Options` or MCP input literal; its `cargo shuyi frame-png` has the options); adopt → 1264 (the README's intro GIF, made by hand with Pillow in 0125, remade by one command) |
| #376 | 3310 | A desktop keeps its storage in the folder the game names: My Games on Windows, the old folder moved there once | adopt → 0875 (its folder and move are the engine's `AppConfig::desktop_folder`; the ticket sets it instead of writing its own) |
| #374 | 2066 | A `Behaviour:` line for a change a game sees without a break | adopt → 1254 (engine#2066 is done: it reads the line) |

Nothing for the game: 9 PRs (#366, #364, #365, #330, #288, #357, #370, #375, #368). #366 is the engine's tutorial playground; #364, #365, #368, #370 and #375 are the engine's own ticket lint, examples, gates, CI and Dependabot; #330, #288 and #357 are dependency bumps with no change the game sees.

## 2026-11-13: 234de697 → a434b973 (21 engine PRs), #649

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #378 | 3300 | Desktop and web exports ship a baked pack beside the program when the package has a `packed` feature; `web::Options` and `ExportOutput` gain `pack`; every web build ships `pack-file.js` | fixed: the release package check names `pack-file.js` in the web shell (the game builds neither struct field by field); adopt → 1265 (the game's release builds read its files from a pack instead of embedding them as text) |
| #386 | 3033 | In-house web saves: the engine's own `storage.js` in place of `quad-storage.js`, and a `localStorage` that throws answers `StorageError::Backend` | fixed: `quad-storage.js`'s rows leave `THIRD_PARTY_ASSETS.md` and `credits.ron`, `sapp_jsutils.js`'s row says what uses it now, and the release package check names `storage.js` and no longer asks `NOTICES.txt` for quad-storage; adopt → 1266 (a browser that refuses storage shows the save error, checked in the browser pane) |
| #390 | 3312 | A save bundle holds each RON entry as RON over lines; `SAVE_BUNDLE_VERSION` is 2 and a version 1 file is refused | fixed: nothing to change (the game writes and reads bundles only through `SaveBundle`, and its tests build theirs) |
| #395 | 3213 | A screen can keep a set of voice clips loaded, beside the clips coming next (`Ctx::keep_voices`) | unblocks 0496 (engine#3213 out of its `blocked_by`, its pin steps struck; the `engine-friction: engine#3213` mark is its to replace) |
| #380 | 3089 | The in-house Windows backend's audio thread runs as a "Pro Audio" task, a frame's sounds start on one sample, and the output counts its dry callbacks | adopt → 1241 (its check of the loops reads the dry callbacks; the game turns the backend on there) |
| #379 | 2589 | `frame-png --gif`: frames over time as a GIF with an exact palette, never quantized | adopt → 1264 (the README's intro is remade with `--gif`) |
| #385 | 2811 | `frame-png --preview`: an example's animated card for the engine's website, from its `shuyi.ron` | not for the game: it makes the engine website's Examples cards, and the game is no engine example (its README animation is 1264's) |
| #391 | 2590 | `shuyi new`'s `private-files` part: a new game's private repository, read by `BasicGame::private` | not for the game: the part is written only by `shuyi new`, and `BasicGame` isn't the game's (its private layer is ADR-0117's) |
| #393 | 3313 | `shuyi bench`'s web package number leaves out `web.music` and `web.voice`; its table gains a "Whole package" column | adopt → 1249 (this is its engine change: its engine and pin steps struck) |

Nothing for the game: 12 PRs (#381, #383, #382, #384, #387, #389, #388, #392, #396, #394, #397, #398). #389 and #392 hold the tickets of #390 and #395; #387 and #388 are engine tickets only; #381, #383, #384, #396 and #398 are the engine's tutorial, book and playground; #382 is the engine's own flaky-gate ledger; #394 and #397 change the engine's reusable playtest workflow and the template's CI, neither of which the game's CI calls.

## 2026-11-14: a434b973 → 4ce11a3b (14 engine PRs), #683

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #399 | 2591 | `shuyi repos` makes a game's public and private repositories on GitHub, with read-only keys, secrets and a ruleset; `doctor`'s `Check` is `#[non_exhaustive]` with `optional` | fixed: nothing to change (the game builds no `doctor::Check` and implements no `Probe`); not for the game: its repository and its private assets repository exist already, with their keys and secrets (ADR-0040, ADR-0117), and `shuyi repos` makes the template's layout, not the game's |
| #407 | 3050 | Release a game from a tag: `export --plan`, `Exporter`'s `host()`, `container()` and `ci()`, the template's `release.yml` with `SHA256SUMS` and attestations | fixed: nothing to change (the game implements no `Exporter`); adopt → 1267 (the game's `release.yml` takes its version check and matrix from `export --plan`, dropping its bash check and its copy of the sniper image's digest, and records each package's SHA-256) |
| #406 | 2597 | `shuyi new` asks whether to include the skills' worked examples; `check-skills` passes a skill without its example that says so | not for the game: the game wasn't made by `shuyi new`, and `skill-pack` copies the pack with its examples, so its `check-skills` is unchanged |
| #411 | 2816 | `shuyi reveal`: a verifiable kit of the approved captures, builds and receipts before the first public footage | adopt → 1268 (a kit before the Steam and itch.io pages and their footage go public; 0901 and 0903 say so) |
| #410 | 2828 | Exercises in the book: `run_game_checked`, `PageCheck` and the page plugin's checks | not for the game: the game runs on the long form, not `run_game`, and has no book page to answer |
| #416 | 2598 | A crash report's `frames`: return addresses as offsets into the executable on Windows and Linux | adopt → 0918 (its engine half is this PR: its engine steps struck; left are the check on the game's release package and `docs/releasing.md`) |
| #404 | 2592 | `shuyi new` asks whether to put the game on GitHub, each repository's visibility and which files stay private, then runs `shuyi repos` (a `Behaviour:` line only; its row added by 1254, whose check asks for one) | fixed: nothing to change (only `shuyi new` run in a terminal asks the new question, and the game wasn't made by `shuyi new` and never runs it: searched `crates/`, `.github/` and `scripts/` for `shuyi new`) |

Nothing for the game: 7 PRs (#400, #405, #409, #408, #412, #413, #415). #400, #405 and #413 are the engine's book and playground; #415 changes `shuyi new`'s template's dependency budget, which the game wasn't made by; #409 and #412 are the engine's own xtask and worktrees; #408 restates the MCP server's move-aside promise and its test, with its code unchanged.

## 2026-11-15: 4ce11a3b → cfd79d10 (22 engine PRs), #700

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #417 | 3302 | A closed build's copy marked for one recipient in a signed ledger (`export --recipient`); packs are version 15 and take files' bytes in any order; `ExportOutput` gains `recipient`, `ExportResult` gains `Detected` | fixed: nothing to change (the game builds no `ExportOutput` and matches no `ExportResult`; its packs are baked by each export; searched `crates/` and `shuyi.ron` for the old pack refusal, `pack::VERSION`, `ExportOutput`, `ExportResult` and `recipient`: only `crates/xtask`'s pack test, which opens a fresh bake); adopt → 1272 (press and playtest copies marked for their recipient) |
| #425 | 3311 | The storage's web log lines print their error, not a literal `{e}` | fixed: nothing to change (takes the engine's; the game matches neither line: searched `crates/` for "smoke check failed" and "will not persist") |
| #428 | 3401 | `private-assets --pin`'s refusal names the private repository's pull request and `switch --detach origin/main` | fixed: nothing to change (takes the engine's; `content-change` already lands the private repository through a pull request, and nothing in `crates/`, `docs/` or the skill says `push origin HEAD:main`) |
| #429 | 2599 | On Windows, `mcp`'s moved-aside copy is one file, and the server waits up to two seconds to delete it | fixed: nothing to change (takes the engine's; searched `crates/`, `scripts/` and `run-gates` for `shuyi-running` and `move_aside`: no hit) |
| #435 | 3402 | New games' compression hook, `repos` setting `core.hooksPath`, and `doctor`'s `git-hooks` and `target-compression` checks | fixed: nothing to change (the game has no `scripts/git-hooks` and never runs `shuyi repos`; `doctor`'s new checks are optional); adopt → 1271 (the game's worktrees compress their 10+ GB target folders by a git hook) |
| #436 | 2257 | The dialogue and tip checks' errors say "cells" where they count cells | fixed: takes the engine's; the game's tests of those errors say "cells", and the game's own lead-line rule (`crates/content/src/dialogue/rules.rs`), which counts cells too, says "cells" with them |
| #437 | 2256 | `> (): text` parses as narration with no direction, and `print_scene` writes it for narration that opens with `(` and holds `):` | fixed: nothing to change (takes the engine's: ADR-0045's `AsciiDlg` checks the text, not the direction; no narration in a `.dlg` of `assets/`, `assets-test/` or the content has text opening with `(` (its one `> (slowly, tired):` is a direction), and the game's `print_scene` round trips pass) |
| #438 | 2082 | The frame's and widgets' types move under ADR-0039 (`#[non_exhaustive]` with constructors); a sprite of a `Paint` the backend doesn't know is drawn in its own colours | fixed: `Sprite`, `Overlay`, `MenuView`, `AskView`, `GridView` and `SkipView` built with `new` and assigned, `PanelLook`'s `with_border` and `with_title`, the Playdate's `ONE_BIT` skin from `GlyphSkin::new` and `GlyphStyle::PLAIN`'s `with_` methods, in `crates/ui` (the sprite test's `with` helper for its changed sprites); the game paints only the engine's `Paint`s |
| #439 | 2412 | Every pad event carries its pad: `PadDown` and `PadUp` with its id, `PadRemoved`, and the merge of pads moved into `Game` | fixed: the Playdate's crank events and the tests' pad events carry pad 0 (`crates/playdate`'s `pd/entry.rs`, the tests in `crates/ui` and `crates/playdate`); `game.rs`'s test of `from_pads` now expects each pad's events with its id and the `PadRemoved`s, the merge being `Game`'s (behaviour unchanged: the app's front end goes through `from_pads` and `Game`) |

Nothing for the game: 13 PRs (#419, #420, #421, #422, #423, #424, #426, #427, #430, #431, #432, #433, #434). #426 ships the pack skill `shuyi-private-files`, copied here by `skill-pack`: adopt → 1273 (`content-change` keeps only the game's rules on top of it). #421 is the engine's own compression hook and gate (the game's is 1271's). #419 is research; #420, #423, #424, #427, #430, #431 and #434 are the engine's own tests, versions, PR size, gates, build profiles, aliases and MCP command; #422 its merge commits and `.codex/config.toml`; #432 and #433 hold tickets only.

## 2026-11-15: cfd79d10 → e074d3ec (6 engine PRs), #706

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #441 | 2087 | `run_app` starts a game of its own shape from a plain `fn main`; `window_conf`, `run` and `keys::to_key` private; `shuyi::files!` embeds a layer in place of an `include_dir::Dir`; ADR-0119 lists the outside crates the API may name | fixed: `crates/app` and `crates/story-preview` call `shuyi_app::run_app(config(), start)` from a plain `fn main`, with no `#[macroquad::main]` or `window_conf`; `story-preview` drops its `macroquad` dependency, and `app` keeps it for the `--watch` log line and the window icon test, which now decodes the icon's three pictures at 16, 32 and 64 pixels as the engine does before giving the window the icon; nothing to change for `Layer::dir` (the game embeds no `include_dir::Dir`: its layers are generated tables, ADR-0105) |
| #442 | 3214 | `DialoguePreview` opens its tabs on Scenes, on the scene just played, when a scene ends (a `Behaviour:` line only) | fixed: nothing to change (takes the engine's: Nick asked for it on 2026-11-15, ticket 0770, which this move unblocks for its game test; searched `crates/` for a test of the preview's tab after a scene ends: none) |
| #445 | 2084 | The sound, music and cinematic types move under ADR-0039 (`#[non_exhaustive]` with constructors) | fixed: `AudioRequest::play_sound` and `play_voice`, `MusicClock::new`, `MusicCue::new` (`with_looped` for the Playdate's played-once track) and `ShotDef::new` in `crates/ui`, `crates/content` and `crates/playdate`; the Playdate's `PlaySound` match takes `..`, and its `MusicCommand` match a `_` arm that plays nothing, as before for every command it knows |

Nothing for the game: 3 PRs (#440, #443, #444). #440 keeps the engine's templates' manifests out of cargo's scan of the engine as a git dependency, which ends the `{{crate_name}}-rules` error cargo printed on every build of the game, with nothing for the game to change; #443 and #444 are the engine's own exercises and API check.

## 2026-11-16: e074d3ec → 1ccb1acc (13 engine PRs), #715

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #448 | 2083 | The input and controller types move under ADR-0039 (`#[non_exhaustive]` with constructors); `PadKind::ALL` is a slice | fixed: `StickDef::new` in `crates/content`'s keymap test, `crates/playdate`'s pad and its test, and `crates/ui`'s `game.rs` test; `PadState::new` in `crates/playdate`'s `pads::state` and that test (sticks at rest, as `Default` gave); the game matches no `Key`, `Button`, `UiAction`, `RawInputEvent`, `PadKind`, `Device` or `Input` without a `_` arm and never reads `PadKind::ALL` as an array |
| #451 | 2085 | Target profiles, packs, language packs, credits and bot settings move under ADR-0039; `PackRules::new` and `with_variants` are `const` | fixed: the game's `const RULES` in `crates/content/src/lang.rs` built with `PackRules::new(…).with_variants(&[MALE, FEMALE])`, the same rules; not for the game: what it adds is the constructors themselves, which this fix uses, and no other type of the PR is built by the game (it reads its target profiles, credits and packs from files) |
| #452 | 2136 | The grid cursor's pulse, music fades and `Pan` on `shuyi_core::ease`, equal to before but in the last bits (a `Behaviour:` line only) | fixed: takes the engine's; no snapshot moved (the title's `cinematic.rs` pans included); the kept Quick Battle session (`crates/ui/tests/it/sessions/quick_battle.session.ron`) is blessed again: from frame 16, where Quick Battle fades the title's music out, each frame's hash holds the fade's gain, which moved in its last bits; its input is unchanged and it plays to the same battle (and the bless writes the record format's `file_drops` stamp) |
| #456 | 2347 | Styled text: key names in `Key` spans in every hint (`StyledText`, `ctx.text_styled`, `print_styled`, the palette's `key`); the engine's screens' help and questions hold them | fixed: the game's views keep their text with no key roles: `WaitView::log_hint` and the profiles' help built from `text_with(…).into()`, the Playdate skins and the class change's question print `.plain()`, tests compare `.plain()`, and the tests of the engine's yes-or-no line expect its key spans; takes the engine's no-break spaces in its own screens' help (the game's `{Action}` names had them already); adopt → 0254 (its key caps turn on the styled text in every hint; engine#2347 out of its `blocked_by`, which keeps engine#3215) |
| #453 | 2078 | `shuyi_cli::worktrees::seed`: a new worktree's `target/` started from a warm one's | adopt → 1274 |
| #454 | 2076 | `shuyi_cli::worktrees::trim`: the incremental caches of idle worktrees deleted | adopt → 1274 |
| #457 | 2092 | `trim`'s `stale_hashes`: older hashes of a crate in `deps/` deleted | adopt → 1274 (the game's `crates/xtask` wraps the three, or takes engine#3403's `cargo shuyi` commands if they land first, and `clean-merged-targets` may go) |
| #459 | 2785 | The template's and Hello, Grid's hints in key spans, with a `key` colour (a `Behaviour:` line for `shuyi new`) | fixed: nothing to change (only a game made by `shuyi new` gets it, and the game wasn't; its own hints are 0254's) |

Nothing for the game: 5 PRs (#446, #447, #449, #450, #458). #450 is engine ticket 3215 alone (key caps), which 0254 still waits for; #446 and #447 are the engine's own compile-time budget and disclosure test; #449 keeps the `in-house-sound` feature doing nothing, with no change the game sees; #458 removes the engine's research scripts.

## 2026-11-17: 1ccb1acc → e738ffc2 (18 engine PRs), #732

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #460 | 3403 | `cargo shuyi trim-targets` and `seed-target --into`; `COMMANDS` a slice; six new problem codes | fixed: nothing to change (searched `crates/` for `COMMANDS`: the game never names it; the game calls no `worktrees::trim`; the template's `AGENTS.md` is a new game's); adopt → 1274 (its text now uses the two commands and writes no wrapper) |
| #462 | 2086 | shuyi-cli's contract types under ADR-0039, `Op::ALL` a slice, `CheckInput::project`, `font_atlas::run(&RunOptions)`, `rules` a list | fixed: the atlas test in `crates/xtask/src/game_tools/tests.rs` calls `font_atlas::run` with `RunOptions`, and the fence test reads `rules` as `["trpg-core"]` (the same one package; `shuyi.ron` loads unchanged); the game builds no contract type by literal and matches none; its `check --target` tests call `check::targets` with the project as before |
| #463 | 3215 | Key caps (box, bold, capitals, room) a theme turns on; `FontAtlasDef::ink`; pack format 16 | fixed: `crates/xtask/src/icon.rs`'s test atlas adds `ink: None`; `assets/fonts/atlas.ron` made again with `cargo shuyi font-atlas` (its `ink` block added; `atlas.png` and `atlas_1bit.bin` unchanged); the game commits no pack (`cargo shuyi export` bakes one); the game's `key_caps` stays off, so its text measures and draws as before (0254 turns it on) |
| #468 | 3216 | A bold face in the font atlas (`font-atlas --bold`); `CellWidths::ink(bold)`; pack format 17 | fixed: the same test atlas adds `bold` and `bold_ink`; the game never calls `ink()`; its atlas is made without `--bold`, so nothing else moves (0254 adds Terminus Bold) |
| #469 | 3044 | The window opens at the size and in the mode the player left it in; `Window::console` (a `Behaviour:` line, and an addition) | fixed: takes the engine's (searched `crates/` for a storage key `window`: none; the game's fullscreen setting, `crates/ui/src/settings.rs`, still wins from its first game frame, as only a game frame's `fullscreen` is applied); adopt → 1283 (`WINDOW` from `Window::console`, not `1640, 1064` by hand) |
| #471 | 2348 | Plural, ordinal and select messages (a subset of ICU MessageFormat); `notices::FILES` an array of 4 | fixed: nothing to change (the game never names `notices::FILES`'s type; every `NOTICES.txt` now lists CLDR's `plural.rs` under Unicode-3.0, a license `deny.toml` and ADR-0013 allow; the notices test passes); adopt → 1282 (the five one-and-many key pairs of `assets/lang/en/ui.ron` become plural messages and their `n == 1` branches go) |
| #474 | 3314 | The system's cursor hides when the mouse is still and in fullscreen (a `Behaviour:` line only) | fixed: nothing to change (takes the engine's; searched `crates/` for `show_mouse`, `set_cursor_grab` and `mouse_position`: none, the game's cursor is the grid's) |
| #475 | 2094 | A new game's `setup-linux.sh` retries `apt-get` (a `Behaviour:` line only) | fixed: nothing to change (only a game made by `shuyi new` has the script; the game has no `scripts/setup-linux.sh`) |
| #476 | 3218 | Typed text arguments (`ToArg`, `&Args`), filled by the plural rules of the text's language; `{device}` and `{target}` filled by `Ctx` | fixed: `crates/ui`'s `tips::fill_text`, `fill_values` and `Words::text_with` take `&Args` and the language their text is in (`Lang::text_in` for a pack, `words::SOURCE_LANG`, `en`, for the data's English, a test that it is the engine's default); the screens' value arrays are `&dyn ToArg`, `SaveError`'s storage error is passed as its text and `class_level`'s level as `impl ToArg`; takes the engine's message parsing (searched `assets/lang`, `assets-test/lang` and the content's `lang/` for `''` and `'{`: none, so every text fills as before); `{target}` in the rewind list's lines is passed by its caller (`rewind.rs`, through `Words`, not `Ctx`), and no game text has `{device}`; its cost on every fill (about 80k instructions per Playdate frame, the cursor sweep 2,050,631 over its 2,000,000 budget) is left to engine#3219 and 1285: until then the cursor sweep is in CI's `KNOWN_OVER` (Nick, 2026-11-17) |
| #477 | 3217 | The cursor's four keys as one picture (`TextRole::Arrows`, `KeyCapStyle::with_arrows`); a cursor on other keys reads `w a s d` | fixed: takes the engine's: the left-handed layout's cursor reads `w a s d` (no-break spaces) in every help bar and tip, and `w a s d` in the layout picker's legend; the tests that expected `wasd` (`crates/ui`'s battle, guides, tips, credits, lead select, options, title and layout picker tests, and `tests/it`'s) and six snapshots (the arts and skill menus, the log, the layout picker twice, Options' picker) changed for that alone; the game filters no span by `TextRole::Key`; adopt → 0254 (its key caps draw the arrow cluster: engine#3215 out of its `blocked_by`, its pin step struck) |
| #478 | 2262 | The bundle's reads hand out a `Cow`, not `&'static`; `AppConfig::files` a reader of `Cow` | fixed: `crates/content`'s `bundle::file`, `bytes` and `test_part` keep handing out `&'static` through `lent` (every layer of the game's lends, `Cow::Borrowed`, so nothing is copied; a file a layer read on demand would be leaked, which 1284 removes); `crates/app` and `crates/story-preview` give `AppConfig::new` a closure that lends; `take`'s comparison, the bundle's and xtask's tests and `playdate_images` deref the `Cow` |
| #479 | 2263 | A pack read file by file (`pack::Index`, `Source`, `Index::into_layer`); pack format 18 | fixed: nothing to change (the game commits no pack; its packed build's `Pack::open` and the pack test read format 18 as baked; the bundle asking each layer for a file first answers the same for its layers, all read whole); adopt → 1284 (the packed build reads its pack file by file once its loaders hold no `&'static`, measured first) |

Nothing for the game: 6 PRs (#461, #464, #466, #467, #470, #473). #461 ships the pack skill `shuyi-worktrees`, copied here by `skill-pack`, which 1274 follows; #464 and #473 are the research behind 2262 and 2263 (#478, #479); #466 and #470 research a code pad in a web page; #467 holds tickets only.

## 2026-11-18: e738ffc2 → 60a7ca47 (23 engine PRs), #761

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #481 | 2265 | Pack version 19: `Bake::first` lays named folders first, each folder's span in the index, `Bundle::arrived` over a source | fixed: nothing to change (the game commits no pack: `cargo shuyi export` bakes one at the engine's version and the packed build's pack test bakes and opens it; its `pack_bake` names no folder first, so `shuyi bake` answers as before); adopt → 1298 (after engine#3318) |
| #482 | 2261 | Messages checked when the languages load; `LangStatus` non-exhaustive; `lang-status`'s missing forms | fixed: takes the engine's checks; the test pack (`assets-test/lang/test/ui.ron`, tag `test`, whose plurals have only `other`) drops the `one` branches of its five plurals, which were never picked, so it fills as before; every other text of the game, its content and its tips loads under the new checks (the tests); the game builds no `LangStatus` |
| #494 | 2095 | The font atlas's description, `Sound` and `DoctorInput` under ADR-0039 | fixed: `FontAtlasDef::new` in `crates/xtask/src/lettering.rs` (its `ink` assigned after) and in `icon.rs`'s test atlas; the game builds no `FontInk`, `Sound` or `DoctorInput` |
| #497 | 3317 | A picture's texture made when first drawn; `Renderer::new` takes no image table | fixed: nothing to change (the game never calls `Renderer::new`; takes the engine's: searched `crates/` and `docs/adr/` for a rule on when textures are made or on an image's decode error: none, and every picture of the game is embedded, so none waits for bytes) |
| #498 | 3407 | Template jobs with the private files cache no build; `shuyi repos` gives Dependabot no key (`Behaviour:` lines only) | fixed: nothing to change for the caches (every job of the game's that checks out the private files already has `cache-targets: false`); keeps the game's own `.github/actions/private-assets` for now, which `shuyi repos` didn't make; 1297 takes the engine's rule that Dependabot's runs get no key and build on the public files |
| #501 | 3316 | A load in steps counts its units and waits for the platform (`Progress::Of`, `Progress::Wait`), the card's bar; a recording counts a load's waits as one frame | fixed: nothing to change (searched `crates/` for `load_steps`, `reloaded_at` and `Progress::`: the game's desktop and web load runs in one frame and nothing waits, so its kept Quick Battle session replays unchanged); adopt → 1170 (its load in steps on the engine's `load_steps`, the card's bar only with Nick's yes) |
| #485 | 3219 | Plain text fills without parsing a message | unblocks 1285 (its pin step struck, `engine#3219` out of its `blocked_by`) |
| #496 | 3223 | Threat arrows as arcs, with a frame on the threatened tile | adopt → 0908 (its open PR #745 offers the three shapes in Options, arcs the default; it rebases on this pin rather than moving it) |
| #499 | 2266 | A language's texts parsed once, at load (`Lang::form_in`, `data_form_in`, `message::Form`) | adopt → 1299 (the game's own fills, `Words::text_with` and the tips', fill text they hold and parse a message on every call) |
| #503 | 2268 | A message's walk borrows its values | adopt → 1299 (the game's fills take it at the pin with no change, all through `&dyn ToArg`; 1299 measures it with 2266's) |

Nothing for the game: 13 PRs (#401, #403, #414, #480, #483, #484, #488, #489, #491, #492, #493, #500, #502). #401 moved the engine to Rust 1.100, so the game's toolchain moves with it (`rust-toolchain.toml`, `rust-version`, ADR-0070's line), clippy 1.100's new `map_or_default` and `needless_range_loop` findings are fixed as it suggests, and cargo 1.100's unused workspace dependencies (`hashbrown`, `thiserror`) are removed; #483 is ticket 3219's file, #485's ticket; #491 finds the Windows window once, with nothing the game sees; #403, #414 and the rest are the engine's book, CI, skills and dependency bumps.

## 2026-11-18: 60a7ca47 → c0da2ec7 (4 engine PRs), #767

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #504 | 2098 | Public API with no second user hidden or moved out before the first release | fixed: nothing to change (searched `crates/` for `recipient`, `PackMark`, `pack::Raw`, `image::mark`, `run_game_checked`, `PageCheck`, `page_check` and `report_check`: none) |
| #506 | 3406 | A `shuyi new` game's tag release is a draft unless `PUBLISH_ON_GITHUB`, its debug files in `<tag>-debug` (a `Behaviour:` line only) | fixed: nothing to change (keeps the game's own rule: its `release.yml` isn't the template's, makes no GitHub Release since 0920 and keeps the debug files in a private repository; searched `.github/workflows/` for `PUBLISH_ON_GITHUB` and `-debug`: none) |
| #505 | 3224 | Ranges that spread out when opened, a wave of light, one outline and relief (`RangeShow`, `MapScene::ranges_shown`) | fixed: the sprite skin's `every_field_is_listed` names `ranges_shown` (the game sets none and draws the tint, as before); adopt → 0923 (its open PR #766 offers the looks in Options; it rebases on this pin rather than moving it) |
| #507 | 0923 | The Options panel may rise to the screen's top row (`api: none`, in `shuyi-ui`'s Options skin) | fixed: nothing to change (the game's Options rows fit below row 1 today, so its panel and snapshots are unchanged; 0923's new row is what needs the room); adopt → 0923 |

Nothing for the game: 0 PRs.

## 2026-11-19: c0da2ec7 → d10d1950 (3 engine PRs), #770

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #509 | 3318 | The web page fetches the `.wasm` beside the pack and streams the pack into the game in the order it was baked; `pack_beside_program` returns `OpenedPack`; `Bundle::arrived` and `Bake::first` answer for a file too | fixed: `OpenedPack::into_layer` compiles unchanged in `crates/app` and `crates/story-preview`; keeps the game's rule that its load reads every file at once: the game's start and the story preview's wait behind the card until the whole pack has arrived (`trpg_ui::start::when_files_arrived` over `Start::in_steps`, `Progress::Wait` until `trpg_content::bundle::arrived()`), as the page used to before it started the game; the rest (pack plugin version 2, the `.wasm` preloaded, `Index::read`'s span check) is the engine's web page and reader, which the game takes (`scripts/check-release-package.sh` names `pack-file.js`, still shipped); adopt → 1298 (lays the title's folders first and waits only for them; its pin step struck, `engine#3318` out of its `blocked_by`) |
| #510 | 3225 | The options glyph skin's rows scroll when they don't fit above the message (`api: none`, in `shuyi-ui`'s Options skin) | fixed: nothing to change (the game's public build's Options rows fit, so its panel and snapshots are unchanged); unblocks 1301 (its engine step and pin step struck: the picture build's two more rows now scroll) |

Nothing for the game: 1 PR (#508, the engine's book and its playground Codespace, which the game doesn't build).

## 2026-11-19: d10d1950 → 7a7d7b0c (1 engine PR), #773

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #511 | 3226 | The options glyph skin's value column leaves room for the longest label and its arrow (`api: none`, in `shuyi-ui`'s Options skin) | unblocks 1302 (this PR: the title's Options snapshot moves every value one cell right, as "Voice on hover in the log" is 25 cells; `engine#3226` out of its `blocked_by`) |

Nothing for the game: 0 PRs.

## 2026-11-20: 7a7d7b0c → 50472bbc (1 engine PR), #779

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #512 | 3505 | The web page takes the keys inside a site's frame without a click: at load, when the frame is first shown, and when the site fullscreens it (`api: none`, the shell `shuyi web` writes) | fixed: nothing to change (the game ships the engine's page unchanged; searched `crates/`, `assets/` and `scripts/` for `glcanvas`: none, the game has no page of its own); adopt → 0933 (this PR: the demo and the `story-trial` preview get it at their next build) |

Nothing for the game: 0 PRs.

## 2026-11-23: 50472bbc → 7d60a21a (25 engine PRs), #808

| Engine PR | Ticket | What | Decision |
| --- | --- | --- | --- |
| #522 | 3041 | Log sinks in shipped builds: `logs/latest.log` and three kept runs, the panic in the log, the browser's console; macroquad's `log-rs` feature, so `macroquad::logging::warn!` is the `log` crate's | fixed: `crates/app`'s line for a folder `--watch` can't read is an `eprintln!` (it runs in `startup()`, before `run_app` installs the engine's logger, so a `log` line there would go nowhere; it printed on stderr before and still does), and `macroquad` is a dev-dependency of `crates/app` only (the window icon's test); takes the engine's log file, line format, panic line and crash-report lines (searched `crates/`, `scripts/` and `.github/` for a reader of the old `{category}: ` lines and the `audio: ` and `storage: ` prefixes: none); adopt → 1345 (the docs and bug-report texts name the log file) |
| #519 | 2239 | A language pack's speech and narration lines get the text box's page check | fixed: nothing to change (takes the engine's: the `test` pack and the content's languages load under it, the tests; the game has no page check of its own for translated lines) |
| #523 | 2133 | The bots' settings refuse unknown fields and have schemas, `Band::distance`, `bot_seed` for an entrant's bots | fixed: nothing to change (`crates/bots/personas.ron` names only known fields and loads, the bots' tests; the game's band distance is already the engine's, `crates/bots/src/balance.rs`; the template's seed is a new game's); adopt → 1346 (the personas' `SEED_SALT` gives way to `bot_seed`) |
| #524 | 3324 | `SHUYI_LOG`, the log's filter from the environment; `log_sink::install` takes the filter | fixed: nothing to change (the game never calls `log_sink::install`: searched `crates/` for `log_sink` and `LogFilter`: none; takes the engine's `SHUYI_LOG`, unset by default); 1345 names it for testers |
| #527 | 2215 | `shuyi-embed` writes a game's file table at build time; `include_dir` and `Files::Dir` gone | fixed: nothing to change (the game matches no `Files` and embeds no `include_dir::Dir`: its layers are `crates/content/build.rs`'s generated tables, ADR-0105; searched `crates/` for `Files::Dir` and `include_dir`: none); adopt → 1347 (`build.rs`'s walk and tables on `shuyi_embed`, with its case and UTF-8 checks) |
| #525 | 2413 | `shuyi-app` reads the mouse and touches: the desktop and web builds send pointer events | fixed: keeps the game's rule that a click doesn't dismiss the title's "press any key or button" (`docs/design/title-screen.md`; `docs/design/controls.md`, keyboard only), which the engine's `Game` ends at a pointer press since #521: the title puts the wait back when the frame's only press is the pointer's (`crates/ui/src/screens/title.rs`, `clicked_alone`; tests `a_click_does_not_end_the_prompt` and `title::a_click_does_not_dismiss_the_prompt`); takes the engine's elsewhere (searched `crates/` for `pointer()`, `point(` and `tile_at`: no screen of the game reads the pointer; only the engine's debug menu, never shipped, takes clicks); not for the game: it is played from the keyboard alone by Nick's design (`docs/design/controls.md`, "Keyboard only") |
| #530 | 3411 | The contract's wall times rounded where they are measured | fixed: nothing to change (keeps the game's own: its `playtest` wrapper, `crates/tools/src/playtest.rs`, times its tries itself and writes its own `Played::seconds_per_try`; it builds no contract answer) |
| #528 | 2397 | Music loop regions: an intro once, then a region of the track; pack version 20 | fixed: nothing to change (the game commits no pack, `cargo shuyi export` bakes one; no cue of the game names a region and its music plays on quad-snd, which plays the whole file as before; the Playdate's music clock is its own, `crates/playdate/src/audio.rs`); adopt → 1348 (after 1241: the tracks whose sources give a loop point play their intro once, then the loop) |
| #536 | 2345 | Cue rules as data (pick, random volume, gaps); `Player::load` takes no seed; frame hashes cover the sounds; pack version 21 | fixed: the kept Quick Battle session (`crates/ui/tests/it/sessions/quick_battle.session.ron`) is blessed again: the hashes of the frames that play a menu sound moved, as each now covers the sounds sent, and its input is unchanged; nothing else to change (the game calls no `Player::load` or Playdate `audio::player`, and builds no `SoundRequest`; no cue has a rule, so every cue plays as before, its file now picked from `ctx.music_seed`); adopt → 1349 (gaps on the cursor and menu ticks, no footstep twice) |
| #537 | 2234 | JPEG, WebP, QOI and JPEG XL pictures drawn and FLAC and MP3 sounds played; the manifest takes them | fixed: nothing to change (takes the engine's texts: searched `crates/` for "must be .ogg" and "no image pixels": none; every picture of the game is a PNG and every sound an Ogg or WAV); adopt → 1348 (the music that came as MP3 or FLAC ships as its source when that is smaller than its re-encode) |
| #541 | 2333 | Scale modes, integer or fit, drawn sharp; `project::Scale` an alias of `ScaleMode`, `Factor` a re-export | fixed: nothing to change (the game names neither `project::Scale` nor `contract::target::Factor`: searched `crates/`; `shuyi.ron`'s targets are `scale: integer`, which `check --target` checks as before); unblocks 1320 (its pin step struck, `engine#2333` out of its `blocked_by`) |
| #535 | 3347 | A graphics tier in each target profile and `PlatformFacts`; `SHUYI_GRAPHICS`; pack version 22 | fixed: the game's two copies of the Playdate's profile (`targets/playdate.ron`, `crates/ui/src/playdate_target.ron`) say `graphics: Reduced`, as the engine's does, so `the_playdates_facts_are_the_engines` holds; nothing reads the tier yet, so nothing draws otherwise; adopt → 1314 (its open PR #802 gives every moving look its stills; it rebases on this pin rather than moving it) |
| #542 | 3419 | `run`'s answer lists the plays the cue rules dropped; contract version 55 | fixed: nothing to change (the game builds no `RunOutput` and no cue has a gap); adopt → 1349 (its tests read the dropped plays) |
| #547 | 2609 | The hosted preview's action API, `shuyi preview` | fixed: nothing to change (no skill of the game names `preview`; `check-skills` passes); adopt → 1350 (with #546) |
| #521 | 2405 | Mouse and touch for the engine's menus and grid maps (`PointerInput`, `Menu::point`, `GridSkin::tile_at`, the Harness's `click`) | not for the game: it is played from the keyboard alone by Nick's design (`docs/design/controls.md`; `docs/design/title-screen.md`, "the game has no mouse controls"); the pointer's one effect on it, the title's wait, is #525's row |
| #529 | 2203 | JPEG, WebP, QOI, JPEG XL, FLAC and MP3 decoders, each behind a feature | adopt → 1348 (with #537) |
| #546 | 2602 | The hosted preview's worker (`preview-worker`, `Session::for_visitor`) | adopt → 1350 (after engine#2611: whether the game offers a hosted preview is Nick's, with the page and its cost) |
| #545 | 3243 | A range tint's strength a game sets for each skin (`RangeShow::with_strength`) | adopt → 1319 (its open PR #801 files 1343, which sets the picture look's tints to 150%; this move is 1343's pin step) |

Nothing for the game: 7 PRs (#517, #518, #531, #532, #533, #543, #544). #518 gives the template's private-files action `ssh` in the sniper image; the game's `release.yml` already installs `openssh-client` there itself. #517 and #531 are the engine's API check; #532 is research; #533 and #543 hold tickets only; #544 fixes a test of the engine's.
