# Architecture Decision Records

Technical decisions live here. **Game-design** decisions (stats, magic, story)
are Nick's and live in [`docs/design/`](../design/README.md) instead.

An ADR is never edited to change its meaning after it is `Accepted`. To change a
decision, write a new ADR that supersedes it and set the old one's status to
`Superseded by ADR-NNNN`. Use the `write-adr` skill.

| ADR | Title | Status |
| --- | ----- | ------ |
| [0001](0001-record-architecture-decisions.md) | Record architecture decisions | Accepted |
| [0002](0002-language-rust.md) | Rust as the implementation language | Accepted |
| [0003](0003-rendering-glyph-grid-macroquad.md) | Glyph-grid rendering on macroquad | Accepted; images in the frame added by ADR-0038 |
| [0004](0004-crate-architecture.md) | Crate layering and deterministic core | Accepted; `app`'s one `unsafe` set out in ADR-0034 |
| [0005](0005-data-driven-content.md) | Data-driven content formats | Accepted |
| [0006](0006-input-actions-and-virtual-cursor.md) | Input actions, vim-style keymap, virtual cursor | Superseded by ADR-0015 |
| [0007](0007-testing-strategy.md) | Testing strategy | Accepted; behaviour tests read the map scene, per ADR-0038 |
| [0008](0008-ci-quality-gates.md) | CI and quality gates (free tier only) | Superseded by ADR-0014 |
| [0009](0009-distribution.md) | Distribution: Windows first, web, itch, Steam | Accepted |
| [0010](0010-ticket-workflow-and-model-routing.md) | Ticket workflow and model routing | Accepted; model routing and instruction entry point amended by ADR-0102 |
| [0011](0011-story-authoring-pipeline.md) | Story authoring pipeline with LLMs | Accepted |
| [0012](0012-visual-style.md) | Visual style: cells, tiles, color, portraits | Superseded by ADR-0018 |
| [0013](0013-licensing-and-third-party-policy.md) | Licensing and third-party policy | Accepted; art/audio licenses amended by ADR-0027; bought art allowed by ADR-0032 |
| [0014](0014-ci-gates-skip-docs-only-prs.md) | CI quality gates, skipping heavy jobs on docs-only PRs | Accepted |
| [0015](0015-input-actions-and-keymap-layouts.md) | Input actions, keymap layouts, virtual cursor | Accepted; per-key overrides superseded by ADR-0031; controller buttons added by ADR-0034 |
| [0016](0016-font-atlas-and-glyph-blitting.md) | Font atlas format and glyph blitting | Accepted |
| [0017](0017-screen-stack-and-frame-driver.md) | Screen stack, frame driver and test Harness | Accepted |
| [0018](0018-visual-style-v2.md) | Visual style v2 (after Nick's look sign-off) | Accepted; browsing cursor superseded by ADR-0024, acted-label rule by ADR-0029, portraits in part by ADR-0032 and ADR-0043 |
| [0019](0019-simulation-rng-and-serde-in-core.md) | In-crate PCG32 simulation RNG; serde derives in `core` | Accepted |
| [0020](0020-battle-state-serialisation.md) | Battle state saves its own data, not the content tables | Accepted |
| [0021](0021-skill-effects-as-data.md) | Skill effects are data, gathered into combat modifiers | Accepted |
| [0022](0022-combat-arts-and-attack-preview.md) | Combat Arts as weapon-input changes, and the attack preview | Accepted |
| [0023](0023-debug-tools-feature-for-pages-build.md) | A `debug-tools` cargo feature for the Pages build | Accepted |
| [0024](0024-cursor-as-pixel-overlays.md) | Draw the battle cursor as pixel overlays, with selectable styles | Accepted |
| [0025](0025-battle-event-playback-in-the-battle-screen.md) | Event playback runs inside the battle screen, as a mode | Accepted; playback keys changed in 0418 |
| [0026](0026-audio-cues-and-music-files.md) | Audio cues as data, played by `app`; music files beside the game | Accepted; licenses and credits amended by ADR-0027; native music loading amended by ADR-0028; track lengths and the music clock added by ADR-0037 |
| [0027](0027-audio-import-and-cc-by-3.md) | Importing third-party audio; CC BY 3.0 allowed | Accepted |
| [0028](0028-native-music-decoded-off-the-main-thread.md) | Native music is decoded on a worker thread, in its own quad-snd context | Accepted |
| [0029](0029-acted-units-keep-label-case.md) | Acted units are dimmed only; the label keeps its case | Accepted |
| [0030](0030-battle-dialogue-triggers.md) | Dialogue triggers are battle data; their scenes are events at their moment | Accepted |
| [0031](0031-player-key-bindings.md) | Player key bindings: slots, per-layout config, fixed Esc/Delete | Accepted; saved config now version 2 with the buttons, ADR-0053; a key on one action per group, ADR-0068; kept in a player profile, ADR-0069 |
| [0032](0032-bought-art.md) | Bought art is allowed; audio stays free | Accepted |
| [0033](0033-playtest-bots.md) | Playtest bots play through `core`'s commands, on reseeded copies, outside the game | Accepted |
| [0034](0034-controller-input.md) | Controller input: gilrs on native, a Gamepad API plugin on web | Accepted; buttons rebindable by ADR-0053 |
| [0035](0035-battle-and-chapter-files-and-the-flow-screen.md) | Battle and chapter files, a core `Campaign`, and one flow screen that hosts the others | Accepted; the flow hosting its screens (§3) superseded by ADR-0093 |
| [0036](0036-help-text-follows-the-device-and-own-font-glyphs.md) | Help text follows the device pressed last; our own glyphs join the font from a second BDF | Accepted |
| [0037](0037-music-clock.md) | A music clock: `app` reports what is sounding, `ui` wraps it at the track's length | Accepted |
| [0038](0038-graphics-are-a-skin.md) | Graphics are a skin: screens say what to show, a skin says how it looks | Accepted |
| [0039](0039-save-file-format.md) | Saves are versioned RON `SaveFile`s; a battle is saved as its history | Accepted; kept in a player profile, ADR-0069 |
| [0040](0040-private-assets.md) | Bought art lives in a private repository, pinned by commit and embedded by the `private-assets` feature | Accepted |
| [0041](0041-prs-up-to-date-before-merging.md) | PRs must be up to date with `main` and green before they merge (no merge queue) | Superseded by ADR-0042 |
| [0042](0042-prs-merge-on-green-without-updating.md) | PRs merge when their checks are green; they need not be up to date with `main` | Accepted |
| [0043](0043-png-portraits.md) | Portraits are PNG files named by a sidecar, drawn as one sprite at a whole scale | Accepted |
| [0045](0045-languages-text-by-key-and-line-ids.md) | Languages: text by key, dialogue lines by id, wide glyphs in two cells | Accepted (0233, 0717); amends ADR-0016 and ADR-0011 |
| [0046](0046-voice-clips-by-line-id.md) | Voice clips are files keyed by dialogue line id, generated now, replaceable by recordings | Accepted; amends ADR-0026 and ADR-0032 rule 2 |
| [0047](0047-pr-mutation-gate-in-shards-with-nextest.md) | The PR mutation gate runs in shards, with nextest as its test runner | Superseded by ADR-0073 |
| [0048](0048-backdrop-a-second-glyph-picture-panned-and-zoomed.md) | A backdrop: a second glyph picture behind the console, panned by the pixel and zoomed in whole steps | Accepted |
| [0049](0049-unit-sprites-from-their-own-files-and-the-game-picks-its-map-skin.md) | Unit sprites come from their own image files, a tileset may leave terrain to the glyph skin, and the game picks its map skin from what it has | Accepted; extends ADR-0038; the tileset file's writer changed by ADR-0052 |
| [0050](0050-player-settings.md) | Player settings: one saved RON record, read through `Ctx` | Accepted; where they are kept and saving at once superseded by ADR-0069 |
| [0051](0051-a-bought-works-credit-names-its-private-files.md) | A bought work's credit names the private files it covers, and a bought file without a credit is refused | Accepted; extends ADR-0032 and ADR-0040 |
| [0052](0052-terrain-pictures-between-tiles-in-layers.md) | Terrain is painted in layers of pictures drawn between tiles, chosen from the terrain grid; a map file names its look | Accepted (0437); extends ADR-0038 and ADR-0049 |
| [0053](0053-player-controller-buttons.md) | Player controller buttons: one shared set of slots in the saved key bindings (version 2) | Accepted; amends ADR-0031 and ADR-0034 |
| [0054](0054-screens-say-what-they-show-as-a-view-a-skin-paints-it.md) | A screen says what it shows as a view; a skin paints it | Accepted; extends ADR-0038 |
| [0055](0055-dialogue-lines-for-who-is-there.md) | Dialogue lines for who is there: `@if` blocks, resolved when the scene starts, and a check that nobody who may be gone is shown | Accepted (0715); extends ADR-0005 and ADR-0011 |
| [0056](0056-combat-pictures-a-sidecar-per-fighter-moved-by-code-on-a-stage.md) | Combat pictures: a sidecar per fighter, one sprite each, moved by code on a stage | Accepted (0413); follows ADR-0038, ADR-0043 and ADR-0054 |
| [0057](0057-test-files-in-their-own-embedded-directory.md) | Test files live in `assets-test/`, embedded only in debug and debug-tools builds | Accepted (0122); extends ADR-0005, ADR-0023 and ADR-0040 |
| [0058](0058-a-maps-light-shades-the-grounds-sprites-from-a-recoloured-copy.md) | A map's light is a name; the ground's sprites carry its shade and are drawn from a recoloured copy of the image | Accepted (0438); extends ADR-0038, ADR-0049, ADR-0052 and ADR-0056 |
| [0059](0059-title-art-a-generated-logo-picture-over-a-backdrop-per-launch.md) | Title art: a generated logo picture over a backdrop per launch | Accepted (0811); follows ADR-0038, ADR-0040, ADR-0054 and ADR-0056 |
| [0060](0060-cinematics-a-file-of-timed-shots-drawn-from-the-time-alone.md) | Cinematics: a file of timed shots, each frame drawn from the time alone | Accepted (0817); follows ADR-0005, ADR-0037, ADR-0038, ADR-0048 and ADR-0059 |
| [0061](0061-cinematic-action-authored-walks-and-strikes-in-the-map-scene.md) | Cinematic action: authored walks and strikes, put in the map scene from the action clock alone | Accepted (0827); extends ADR-0060, follows ADR-0038 |
| [0062](0062-cinematic-combat-shot-a-fight-the-rules-work-out.md) | Cinematic combat shot: a fight the rules work out, played back on the battle's own stage | Accepted (0832); extends ADR-0060, follows ADR-0004, ADR-0054 and ADR-0056 |
| [0063](0063-an-engine-and-a-game-one-boundary-now-two-repositories-later.md) | An engine and a game: one boundary in this repository now, two repositories later | Accepted (1101; Nick chose plan B, 2026-10-14); code moved 2026-10-19 (1131) |
| 0064 | Game styles: stages, two kinds of simulation, cameras and projections | Moved to [Shuyi Engine](https://github.com/Zafnok/shuyi-engine/tree/main/docs/adr) as its ADR-0003 (1101) |
| 0065 | Assets: standard files in, the engine's own data out; a deterministic movement kit, physics optional | Moved to [Shuyi Engine](https://github.com/Zafnok/shuyi-engine/tree/main/docs/adr) as its ADR-0004 (1101) |
| 0066 | AI-native tooling: one contract, three surfaces (CLI, MCP, skills); visual scripting as a view over text | Moved to [Shuyi Engine](https://github.com/Zafnok/shuyi-engine/tree/main/docs/adr) as its ADR-0005 (1101) |
| 0067 | Networking in layers: transports, sessions, sync models, services; determinism is the contract | Moved to [Shuyi Engine](https://github.com/Zafnok/shuyi-engine/tree/main/docs/adr) as its ADR-0006 (1101) |
| [0068](0068-key-groups-a-key-on-one-action-per-group.md) | Key groups: a key or button on one action per group, one press giving each group's | Accepted (0729); amends ADR-0031 and ADR-0053 |
| [0069](0069-player-profiles-a-prefix-per-profile-in-one-storage.md) | Player profiles: a key prefix per profile in the one storage, a list kept per computer | Accepted (0836); supersedes ADR-0050's storage part, amends ADR-0031 and ADR-0039 |
| [0070](0070-playdate-front-end.md) | The Playdate front end: stable Rust, the SDK's own C build, rules without `std` | Accepted (1026) |
| [0071](0071-sonar-backlog-becomes-batched-github-issues.md) | SonarCloud's backlog becomes batched GitHub issues, filtered by a checked-in policy | Accepted (0126) |
| [0072](0072-options-changes-previewed-then-kept-or-put-back.md) | Options changes are previewed, then kept or put back on leaving | Accepted (0839); amends ADR-0050 and ADR-0069 |
| [0073](0073-mutation-shards-build-incrementally-and-run-slow-tests-last.md) | Mutation shards build incrementally, run the slow tests last, and share one baseline | Accepted; its shard rule and shard time limit superseded by ADR-0100 |
| [0074](0074-the-one-bit-frame.md) | The one-bit frame: inks as reserved palette colours, a renderer to the Playdate's frame buffer | Accepted (1032) |
| [0075](0075-sprites-fade-by-brightness-not-opacity.md) | Sprites fade by brightness, not opacity | Accepted (0732); amends ADR-0060 §4 |
| [0076](0076-conversation-openings-a-cast-on-a-place-and-the-cinematic-walks.md) | Conversation openings: a cast on a place, and the cinematic walks | Accepted (0734); extends ADR-0061 |
| [0077](0077-the-one-bit-render-redraws-what-changed.md) | The one-bit render redraws only what changed, against a kept reference | Accepted (1055); extends ADR-0074 §4 |
| [0078](0078-input-actions-an-open-set-the-game-defines.md) | Input actions are an open set the game defines (`ActionSet`); layouts are named data | Accepted (1106); amends ADR-0015 and ADR-0031 |
| [0079](0079-button-chords-and-the-playdates-own-buttons.md) | Button chords (two pad buttons together), and the Playdate's own buttons | Accepted (1034); extends ADR-0034 and ADR-0053 |
| [0080](0080-screens-the-context-and-the-frame-loop-generic-over-the-game.md) | Screens, the context and the frame loop are the engine's, generic over the game (`Ctx<G>`, `GameData`, `GameHooks`) | Accepted (1109); amends ADR-0017 and ADR-0050 |
| [0081](0081-the-platform-shell-is-the-engines-the-game-gives-an-app-config.md) | The platform shell is the engine's (`shuyi-app`); the game gives an `AppConfig` and builds its `Game` on a `Platform` | Accepted (1110); amends ADR-0004 |
| [0082](0082-the-playdates-bundle-leaves-out-what-it-never-reads.md) | The Playdate's bundle leaves out what it never reads (`playdate-bundle.txt`), its layer says what it left out, and a Chapter 1 test proves the list | Accepted (1036) |
| [0083](0083-a-skin-set-in-the-context-picks-the-playdates-skins.md) | A skin set in the context (`SkinSet`) picks the Playdate's skins; its map skin paints the shared scene in one-bit inks | Accepted (1037); extends ADR-0038, ADR-0054, ADR-0070 §4 and ADR-0074 |
| [0084](0084-playdate-audio-formats.md) | Playdate audio: mono IMA ADPCM at 44.1 kHz, converted by `cargo xtask playdate` (sounds in memory, Chapter 1's music streamed, `playdate-music.txt`); the music clock is the file player's position | Accepted (1047) |
| [0085](0085-one-bit-pictures-made-when-the-playdate-build-is-built.md) | One-bit pictures: a `.1bit` format (engine), converted by the game's table when the Playdate's bundle is built and put at the PNG's path; the renderer maps the PNG's pixels onto the bigger picture | Accepted (1040) |
| [0086](0086-the-crank-as-buttons-and-actions-from-a-platform-menu.md) | The crank as two buttons (notches through the bindings), and actions from a platform's own menu (`Game::act`) | Accepted (1035); extends ADR-0034 and ADR-0079 |
| [0087](0087-engine-screens-take-the-games-rows-entries-and-hooks.md) | The engine's screens and widgets take the game's rows, entries and hooks (Key bindings rows, the picker's legend, `OptionRow`s, `DebugEntry`s, `DialogueHooks`, `AudioConfig`) | Accepted (1112); extends ADR-0054 and ADR-0080 |
| [0088](0088-the-grid-map-kit-a-scene-generic-over-the-games-units-skins-and-unit-painters.md) | The grid map kit: `MapScene<U>` of the game's units, ranges as named layers, `GridSkin`s that paint the ground with the game's `UnitPainter`, skin settings for colours and walk speeds, a `Projection` hook | Accepted (1115); extends ADR-0038, ADR-0049, ADR-0052, ADR-0058; the façade trait (§6) and `RangeKind` superseded by ADR-0094 |
| [0089](0089-shuyi-cli-a-project-file-and-the-games-tool-program.md) | Shuyi's tools: `shuyi-cli` (`cargo shuyi`), a project file (`shuyi.ron`) for the paths and settings, and the game's tool program (`GameTools`, here xtask) for the commands that run the game; the web shell is the engine's | Accepted (1117); extends ADR-0063, ADR-0038 §4, ADR-0081 |
| [0090](0090-the-playdates-bundle-has-a-private-layer.md) | The Playdate's bundle has a private layer with the `private-assets` feature (`cargo xtask playdate --private-assets`), and a touched-up `.1bit` beside a bought PNG is embedded as it is | Accepted (1061); extends ADR-0070 §3, ADR-0082, ADR-0085 |
| [0091](0091-the-playdates-screen-skins-and-what-its-options-leave-out.md) | The Playdate's screen skins: `Skins` picks a screen's painter by the skin set; the engine's Options and Key bindings take the game's (`painted_by`); with no keyboard Key bindings is the controller side alone and captures chords; the Playdate's Options leave out Layout, Fullscreen, Colors, Map look and zoom | Accepted (1045); extends ADR-0054, ADR-0083, ADR-0087 and ADR-0079 |
| [0092](0092-engine-friction-game-code-far-bigger-than-the-job-triggers-a-review.md) | Engine friction: game code far bigger than the job (3× an estimate written first, measured by `cargo xtask code-lines`) triggers a review with one of four outcomes; the engine never names a game concept, and the game never waits on the engine | Accepted (1151); extends ADR-0063 |
| [0093](0093-the-game-flow-pushes-its-screens-and-reads-what-they-pop-with.md) | The game flow pushes its screens over itself and reads what each pops with (engine 2306); Game Over is the battle's; the title refreshes when uncovered | Accepted (1158); supersedes ADR-0035 §3 |
| [0094](0094-a-map-skin-is-a-kit-grid-skin-and-its-unit-painter.md) | A map skin is a struct: one of the kit's grid skins and which of the game's unit painters goes with it, built each frame (engine 2313); the Playdate's is the kit's glyph skin in a one-bit `GlyphStyle`; ranges are the kit's `RangeName`s (`map_view::range`) | Accepted (1167); supersedes ADR-0088 §6 and its `RangeKind` (§2) |
| [0095](0095-playtest-personas-plan-their-turn-and-score-what-a-player-sees.md) | Playtest personas plan their own turn on the engine's turn planner (engine#2128) and score what a player sees (the danger zone and forecasts, never the enemy's mind); `Determinize` and `threat_forecast` for `BattleState`; settings and targets in `crates/bots/personas.ron` and `targets.ron` | Accepted (0506) |
| [0096](0096-a-watcher-on-the-engines-main-moves-the-pin-when-its-public-api-changes.md) | A watcher on the engine's `main` (`engine-watch.yml`, started by the engine's dispatch and hourly) keeps one `engine-pin` issue when a move is due: a public API change (the engine's `api:` label or a `pub` line), an unblocked game ticket, or engine code 48 hours old. The issue is its own ticket, worked by the `engine-pin` skill; every move reviews the engine PRs it passes (adopt, or not for the game) in `docs/engine/pins.md`, which `engine-pin --check` checks | Accepted (1190); extends ADR-0063 §8 and ADR-0071 |
| [0097](0097-the-engines-skill-pack-is-copied-into-the-game-at-the-pin.md) | The engine's skill pack is copied into `.claude/skills/shuyi-*` at the pin, not taken as the plugin (which follows the engine's `main`): `cargo xtask skill-pack` writes the pack the pinned `shuyi-cli` embeds, `--check` fails on a stale copy; `keyboard-input` and `run-gates` keep only the game's rules and link `shuyi-input` and `shuyi-gates` | Accepted (1192); extends ADR-0063 §8 |
| [0098](0098-markdown-the-code-reads-runs-its-crates-tests.md) | A Markdown file the code reads runs the tests of the crates that read it: its `read by` line in `changes.yml` feeds CI's `doc-tests` job (one OS, those crates only); xtask's `every_markdown_file_the_code_reads_is_tested` keeps the lines exact | Accepted (0252); extends ADR-0014 |
| [0099](0099-world-maps-a-file-per-act-rules-in-core-the-look-in-the-file-and-two-skins.md) | World maps: a file per act (`assets/worldmaps/`), its rules in core (`Campaign::world`, commands and events, what is open worked out, never stored), its look in the file, a `WorldMapView` painted by a glyph and a picture skin; chapters say `world_map`; `SavePoint::WorldMap` with no new save version; the map's shortcuts in a key group of their own | Accepted (1007); extends ADR-0005, 0035, 0038, 0039, 0054 |
| [0100](0100-mutation-shards-of-at-most-sixty-mutants-with-a-150-minute-limit.md) | Mutation shards of at most about 60 mutants: 1 shard up to 40 mutants, 4 up to 160, 8 up to 480, then one per 60, at most 20; each shard may run 150 minutes | Accepted (0133); supersedes ADR-0073 §1's shard rule and §3's time limit |
| [0101](0101-a-game-pr-stays-under-about-3000-changed-lines-by-estimate-and-warning.md) | A game PR stays under about 3,000 changed lines: planned at about 2,000 and split before coding, measured by `cargo xtask pr-size` (generated and binary files left out by `.gitattributes`), warned about by a Claude Code hook and on the PR, never failed; `engine-pin` and `sonar` PRs exempt | Accepted (0135); extends ADR-0010, 0092 |
| [0102](0102-portable-agent-workflows.md) | Portable agent workflows and capability routing | Accepted; model routing amended by ADR-0103 |
| [0103](0103-paired-model-recommendations.md) | Paired Claude and OpenAI model recommendations | Accepted |
| [0104](0104-random-skirmishes-overlay-cleared-places-and-rebuild-ordinary-battle-files.md) | Random skirmishes overlay cleared places and rebuild ordinary battle files | Accepted (1008); extends ADR-0099 |
| [0105](0105-no-build-embeds-a-readme-every-bundle-layer-is-a-generated-table.md) | No build embeds a README; every bundle layer is a table `build.rs` writes | Accepted (0123); extends ADR-0005, ADR-0040, ADR-0057, ADR-0070 |
| [0106](0106-a-forecast-carries-its-numbers-after-a-first-hit-debuff.md) | A forecast carries its numbers after a first-hit debuff | Accepted (0330); extends ADR-0022 |

Numbers 0044, 0050 and 0054 are taken by PRs open on 2026-10-04.

Template: [`0000-template.md`](0000-template.md).
