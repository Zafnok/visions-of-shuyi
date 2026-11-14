---
id: "1209"
title: See a changed game file in the running debug build
type: feature
milestone: "Engine: carve"
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["engine#2220"]
nick_input: none
completed: 2026-11-14
---

# 1209 — See a changed game file in the running debug build

## Context

Engine PR #221 (2220), passed by the pin move of 2026-11-01: hot reload of
any asset kind. With debug tools on, `Platform::changed_files` reports what
changed on disk once a frame; `shuyi_assets::hot::Hot<T>` keeps a value that
reloads and keeps the old one when the new file is broken;
`Hook::Reloaded`, `Game::reloaded` and `Ctx::note_reload` tell the game, and
`shuyi-app` draws the error over the frame. `shuyi run --watch` runs it, with
`shuyi.ron`'s `assets` part naming the folders. A shipped build never polls.

Today a change to a `.dlg` scene, a map, a palette or a battle file in
`assets/` needs a rebuild and a walk back to that screen, because
`crates/content/src/bundle.rs` embeds the files at build time and the
content is loaded once. The polling only sees folders read with
`read_folder`, so the game's embedded bundle needs a debug path to the disk
first. A changed PNG isn't redrawn yet (engine 2242).

**Either order with 0764:** 0764 builds a script preview program for
writers that reads the scripts from a folder and reloads them. If 0764
is done, reuse its folder reading for the debug build where it fits; if
not, nothing to do.

## Nick input

None.

## Scope

**In:**
- With the debug tools on, the game's content can come from `assets/` and
  `assets-test/` on disk, so `changed_files` sees them. Shipping builds keep
  the embedded bundle alone.
- Start with the kinds that repay it: palettes and dialogue scenes, which
  are read when a screen opens. A changed file shows on the next frame or
  the next time its screen opens, and its error is drawn over the frame with
  the old value kept.
- `shuyi run --watch` works on this repository (`shuyi.ron`'s `assets`).
- Write down in the completion notes what stays out and why: a battle's rules
  data must not change under a running battle (ADR-0004: its state changes
  only by `Command`), so maps and battle files reload at most on the next
  battle.

**Out (do not do):**
- Reloading rules data in a running battle.
- Pictures, until engine 2242.
- Anything in a shipping build; the Playdate and web front ends.

## Implementation steps

1. Read the engine's chapter 5 and ADR-0071, and Hello, Grid's palette.
2. Give the debug build's content a path to the disk.
3. Move palettes and dialogue onto `Hot<T>` or `Game::reloaded`.
4. Point `shuyi.ron`'s `assets` at the folders; try `shuyi run --watch`.

## Acceptance criteria

- [x] With the headless backend's script, a changed palette file is seen by
      the game on the next frame.
- [x] A changed `.dlg` shows the next time its scene plays, without a rebuild.
- [x] A broken file keeps the old value, and its error names the file and line.
- [x] A build without the debug tools embeds the same files as before and
      never asks for changes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the first three criteria through the Harness and the headless
  backend's `change_file`.

## Completion notes

**What was done** (ADR-0124):

- **The debug build reads its folders from disk under `run --watch`.**
  `crates/app/src/main.rs` (`read_from_disk`): with debug tools on and
  `SHUYI_WATCH` set, the app reads `assets/`, `assets-test/` and the
  content (`assets-private/content/`) with `shuyi_app::read_folder`,
  beside the folder `SHUYI_WATCH` names, and `bundle::read_from_disk` puts
  each in its embedded table's place (same order, private and test as
  before). The bought art stays embedded. `shuyi.ron` names `assets` for
  `run --watch`.
- **Read as a build embeds it.** The engine's `read_folder` keeps every
  file, and `lang/README.md` stops the content loading. `bundle::Folder`
  keeps the watched layer and a second layer of the files `build.rs` would
  embed (no READMEs, ADR-0105; no content `tests/`, ADR-0118), and
  `bundle::take_changes` copies a change across before anything reloads.
  The app's test loads the content from the real folders and finds it equal
  to the embedded content.
- **`trpg_ui::hot::HotContent`**, a module `start::start` installs in a
  build with debug tools: when a theme's palette or a `dialogue/*.dlg`
  changed, it loads the whole content again and takes the palettes (the
  theme in use repainted from the next frame) and the dialogue table
  (looked up each time a scene plays). A failed load keeps the old ones and
  notes its errors under `content`; the platform draws them over the frame
  until a load succeeds. A full load takes a fraction of a second (the five
  Harness tests, ten loads, run in 0.2 s), so the lighter
  `Hot<PaletteDef>` route wasn't needed, and every check across files
  still runs.
- Tried for real: the debug game started with `SHUYI_WATCH` (what `run
  --watch` sets), a `@endif` removed from `assets-test/dialogue/test.dlg`
  on disk drew `assets-test/dialogue/test.dlg:11: @if has no @endif` at the
  top of the window within a second, and putting it back cleared it.

**What stays out, and why:**

- **Maps, battles, chapters, classes, items and every other table** keep
  what the game started with. A battle's state changes only by `Command`
  (ADR-0004) and its record must replay, so rules data can't change under
  a running battle; maps and battle files could reload at most at the next
  battle, which is later work.
- **Pictures**: the bought art stays embedded and no picture is reloaded
  (engine 2242 is done in the engine but not adopted here).
- **A scene already on screen** keeps the copy it opened with; the change
  shows the next time it plays.
- **Shipping builds, the web and the Playdate**: no folder is read (the
  web and Playdate have none), the module isn't installed without debug
  tools, and the engine never asks for changes then.

**Deviations:**

- `cargo shuyi run --watch` couldn't start the game while this was
  worked: since 0764 `trpg-app` had two programs and the engine refused to
  choose (SH1002), which broke the desktop exports too. I tried it as
  `SHUYI_WATCH=<repo>/assets cargo run -p trpg-app --bin visions-of-shuyi`
  and filed the bug; 1269 (#690) fixed it on `main` meanwhile (the
  preview is its own package), so the ticket I filed was dropped, and
  `run --watch` itself was tried after merging it.
- The README filter (`bundle::Folder`) is a workaround for `read_folder`
  having no way to leave files out; an engine `read_folder` with a filter
  would remove it.
- 0764's folder reading (`read_watched_folder` and the engine's
  `DialogueFolder`) didn't fit: they watch a folder in every build and
  read it as the only scripts, where the game needs its whole content.
  This uses `read_folder`, which is watched only with debug tools.

**Engine friction:** estimate 160 production lines (hello-grid's
`hot_palette.rs`, about 50 lines for one palette, plus reading three
folders in `main` and a lookup of which layer a folder replaces), measured
180 (1.1×). Signal: a workaround for an engine assumption (the README
filter), reviewed: engine ergonomics, shipped and followed up by **1270**
(marked `// engine-friction: 1270`).

**Gameplay rules decided:** none.

**Follow-up tickets:** 1270 (a folder reader that leaves files out).
