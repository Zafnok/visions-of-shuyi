# Engine-shaped code left in the game: audit, 2026-11-27

16 read-only auditors, one slice each, covered every game crate against the
pinned engine (`230080d`) and its `tickets/`. I re-checked the headline
claims (ticket blockers, the engine functions named) by hand. The line
counts are the auditors' estimates of production lines saved in the game.
They are not measurements.

Nothing here is filed yet. Under `engine-friction`'s *Sweep*, this would
become a sweep ticket (1153's shape) that logs and files these findings.

## A. Already ticketed and now unblocked: about 6,000 lines

The engine has shipped every piece these tickets wait on. Each ticket's
`blocked_by` is met at the pin, but none of them has been worked yet.

| Game ticket | What it deletes | Lines | Checked |
| ----------- | --------------- | ----- | ------- |
| 1170 (then 1204–1206) | The Playdate front end: the engine's `crates/playdate` has the driver, backend, heap, stack, storage, audio and panic pieces | ~4,800 (crate is 5,288) | engine#2388, 3096 and 3097 done |
| 1180 | The bots' weights, threats, goal distance, candidate order and targets (`persona.rs`, `playtest.rs`, `movement.rs` 452-634). It should also take `record.rs` `seize_field` | ~450 | all 7 engine blockers and 0506 done |
| 1207 (after 1180) | The game's own `playtest` command | ~1,000. It needs `--merge` in the engine first (C17) | — |
| 1274 | `xtask/clean_targets.rs`, which copies `shuyi trim-targets --merged`. The ticket's "~200" estimate is stale: the whole file goes | ~405 (+613 tests) | engine `cli/src/worktrees/trim.rs` |
| 1219 | `content/tileset/look.rs`, onto the engine's `TerrainTilesDef`/`LookDef` | ~330 | no blockers |
| 1222 | The portrait picture set, onto `assets/picture_set.rs` | — | no blockers |
| 1228 | The fight stage's and AI pan's eases and lerp, onto `shuyi_core::ease` | ~45 | engine#2109 done |
| 1182 | `--then-pad` and `After` in `game_tools.rs` | ~20 | engine#2540 done |

## B. Copies of what the engine already has, with no ticket: about 250 lines

These need no engine work: delete the game's copy and call the engine's.

- 5 hand-made yes/no and choice questions (end turn, restart, suspend,
  the rewind confirm, the lessons end) → `widgets::Ask`. The game
  already uses `Ask` in `shop.rs`. ~40–70.
- Copies of `cells`/`text_cells` in 7 skins → `GlyphBuffer::text_cells`.
  The friction row for 0236 said the engine covers it, but the copies
  were never deleted. ~20.
- Preparations' `stock_window_start` → `ListCursor::window_by_heights`.
  ~12.
- `sfx::wav_bytes` → `shuyi_assets::audio::pcm16_wav`. ~20.
- Hand-written picks in `skirmish.rs` and `level_up` →
  `RandomSource::index`/`pick`/`weighted_index`. These make the same
  draws, so replays stay equal. ~15.
- `hp_fill`/`lit_fill` → `frame::meter_fill`. This is a small look
  change: 99/100 HP would show one empty cell. ~20.
- `tips/glyph.rs` hand-drawn titled box → `GlyphBuffer::panel`. ~10.
- `font::text_width` → `lang::text_width`. ~6.
- `bot_wins` threads → `cli::pool::run`. ~10.
- `frame-png --playdate` → implement `GameTools::frame_one_bit`. ~15.
- `movement::area_tiles` → `threat::strike_area_from`. ~20.

## C. New engine pieces: about 5,000 lines

All of these pass the leak test. Under the friction rule, each engine
ticket needs Nick's OK. They are ranked by lines saved.

| # | Engine piece (generic name) | Where | Lines | Evidence |
| - | --------------------------- | ----- | ----- | -------- |
| 1 | Private-content leak checks: no `.dlg` phrase or `.map` row in public files, no private CI job caching `target/` | cli `check` | ~400 | `xtask/leaks.rs`, `workflow_guard.rs` |
| 2 | An edition's cut list: leave out, keep entries, swap text | cli `export --edition` | ~350–450 | `content/demo_bundle.rs` |
| 3 | Per-change private pin files merged into one pin (ADR-0135's own "alternatives" names this) | cli `private-assets` | ~300–400 | `xtask/assets_pin.rs` |
| 4 | Corner-sheet recipes from Tiled terrain sets, 3×3 patches and RPG Maker quarters | assets `corners` | ~300 | `tools/tileset_import.rs` |
| 5 | Moving terrain tiles, plus a timed frame clip. engine#2359 says "no game asks", which is stale: 1315 and 1378 shipped a copy | tactics kit | ~300 | `content/tileset/frames.rs`, `trees.rs` |
| 6 | Incremental threat queries ("what threatens P if it stood on T") | tactics kit | ~240 | `core/movement.rs` 668-961, `battle/threat.rs` |
| 7 | GIF options for `frame-png`: delta frames, crop, spans, hold | cli | ~220 | `tools/readme_pictures.rs` |
| 8 | A sprite piece painter: pictures taller than their tile, y-order, a clip so they never hide the piece above, gliding | tactics kit | ~200 | `map_view/sprite/units.rs` |
| 9 | A styled-text help bar: styled wrap that keeps key caps whole, joined hints | ui | ~200 | `widgets/help.rs` |
| 10 | Dialogue backlog with jump-back, auto-advance and preloading the lines that can come next (re-scope parked engine#2723 and #2725) | ui dialogue | ~200 | `screens/dialogue/history.rs`, `dialogue.rs` |
| 11 | Pixel edits on `Image`: crop, cell, blit, flip, key-out, recolour, halve | assets | ~200 | 5 importers in `tools/` |
| 12 | A music pool rotation: play, fade, a quiet gap, the next track | ui audio | ~170 | `screens/camp/music.rs` and 2 setups |
| 13 | Profile export/import on `ProfilesScreen`, export file names and the "saved to" notice | ui | ~260 | `profile.rs`, `save_files.rs` |
| 14 | A default `as_any`/`as_any_mut` on `Screen` (31 + 7 hand copies) | ui | ~150 | every screen |
| 15 | Enemy-phase move playback: pan, highlight, walk, hurry | tactics kit | ~140 | `battle/ai_phase.rs` |
| 16 | A full-screen question that pops with the chosen value (5 screens) | ui | ~120 | `screens/mod.rs` `pick_one` |
| 17 | `playtest --merge`: join split seed ranges. This blocks 1207 | cli | ~85 | `tools/playtest.rs` 456-541 |
| 18 | Stills for moving looks by graphics tier | assets target | ~100 | `content/motion.rs` |
| 19 | A first-launch language picker, with the machine-translation note | ui | ~100 | `language_list.rs` |
| 20 | A walker over a `.dlg` scene's steps with their lines (5 copies), and a plain-ASCII text policy instead of filtering engine errors by their message text | assets dialogue | ~130 | `content/dialogue/*` |
| 21 | Movement that stops on entry (zone of control), plus a check of a drawn path's cost | tactics kit | ~90 | `core/movement.rs` |
| 22 | Checking a map's lights and props against its tileset | tactics kit | ~90 | `content/map.rs` 162-260 |
| 23 | A looping ambience bed with crossfade, hush and owner (4 screens) | ui audio | ~80 | `ui/ambience.rs` |
| 24 | Text padded or cut to a cell width (found by 3 auditors: ~30 call sites) | assets lang | ~40 | `screens/mod.rs` `pad_cells`/`clip` |

Smaller items (each under about 60 lines) are in the auditors' notes:
- an `ActionSet` macro
- a voice-manifest writer
- save-bundle checks for the engine's own keys
- key placeholders in every language pack
- `Pos` arithmetic and `TileRect`
- `Rect::centred_in` (20 copies)
- a harness snapshot of one area
- a cinematic clock that follows the music
- a delayed sound cue
- digit grouping
- date helpers
- `keep_voices` de-duplication

## D. Bug found on the way

`battle/art_list.rs:525-527` pads Combat Art names with
`format!("{name:<NAME_W$}")`, which counts characters, not cells, so
Chinese names break the arts menu's columns. Ticket 0263 fixed the other
menus and missed this one.

## E. Genuinely game logic (checked, leave it)

- The rules crate's combat, progression, items, shops, supports and arts.
- The battle `Mode` machine and the forecast.
- Every view struct and skin painter that names units, factions or stats.
- Campaign flow, the world map's travel, and camp sections.
- The cinematic's shot kinds.
- The voice-lab pipeline.
- story-import and engine-watch, which run on this game's own process.

Motion, camera, cursor, path and save are already thin layers over the
engine's grid kit.
