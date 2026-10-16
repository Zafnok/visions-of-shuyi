---
id: "1103"
title: "shuyi-assets: the asset bundle as layers the game gives, plus file errors and RON loading"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: done
blocked_by: ["1102"]
nick_input: none
completed: 2026-10-16
---

# 1103 — shuyi-assets: the bundle as layers the game gives

## Context

First move of the carve (ADR-0063 §6). `trpg-content`'s bundle
(`crates/content/src/bundle.rs`) layers embedded folders (the game's
assets, then the private art, then test files) with logic that is
game-agnostic, but the folders are written into `include_dir!` calls and
a static `LAYERS` (`bundle.rs:20-92`), two cargo features and
`build.rs` are tied to them, and every loader calls free functions
(`bundle::file()`, `files_in()`). `docs/engine/boundary.md`, coupling 9.

It waited for 0804 (it touches every loader in `content`, which the
playtest tickets also changed). Since playtest round 3 (2026-10-16) the
carve comes first: Nick, "we should prio engine carve. Make it obvious
engine carve is now the critical path." Round 4 of 0804 now waits for
the carve, so this no longer waits for 0804.

## Nick input

None.

## Scope

**In:**
- New crate `engine/crates/assets` (package `shuyi-assets`), depending
  only on allowed crates (1102's list).
- `shuyi_assets::bundle`: `Layer { name, dir: &'static Dir<'static>,
  private: bool, test: bool }`, `Bundle::new(Vec<Layer>)`, and the
  existing behavior as methods: `file`, `files_in` (merged, sorted,
  de-duplicated), `display_path`, `is_private`, `story_only`,
  `test_part`. A process-wide `Bundle` is registered once
  (`shuyi_assets::bundle::install(Bundle)`, a `OnceLock`) so the
  `&'static` API loaders rely on stays.
- `shuyi_assets::{error, ron}`: `ContentError` with file:line:col
  (`content/src/error.rs`), `parse_ron<T>` (`ron_loader.rs`), and the
  `line_of` helper now in `content/src/terrain.rs:285`.
- `trpg-content` keeps its `include_dir!` calls, features and `build.rs`
  (they are the game's folders), builds its `Vec<Layer>` in one function
  (`trpg_content::layers()`), installs it in `load_embedded`, and
  re-exports the moved items under their old paths so other crates
  don't change.

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6). 1028's generated file table (the `no_std` bundle) and 1036's Playdate bundle list become layers of the new bundle.

**Out (do not do):** disk layers and hot reload (1124); mods (1011);
moving any other loader (1104, 1105).

## Implementation steps

1. Create the crate and add it to the boundary README's crate list.
2. Move the bundle logic and its fixture tests (`bundle.rs:399-645`);
   tests that read the real game files stay in `trpg-content`.
3. Move `error.rs`, `ron_loader.rs`, `line_of`; re-export.
4. Make `trpg-content` install its layers; run every content test.
5. `cargo xtask check-engine-boundary` passes.

## Acceptance criteria

- [x] `shuyi-assets` builds and tests alone (`cargo test -p
      shuyi-assets`) with no game file.
- [x] Every existing test passes unchanged; no snapshot changes.
- [x] `cargo build -p trpg-app --target wasm32-unknown-unknown` passes.
- [x] `docs/engine/boundary.md` rows for `bundle.rs`, `build.rs`,
      `error.rs`, `ron_loader.rs` updated to say where they now live.
- [x] All gates in the `run-gates` skill pass, including
      `check-engine-boundary`.

## Tests required

- Unit: bundle layering over fixture directories (moved tests).
- Integration: the game's `all_assets_load` still passes.

## Completion notes

- New crate `engine/crates/assets` (`shuyi-assets`): `bundle` (`Bundle`,
  `Layer`, `Files`, `install`, `installed`, `get_or_install`,
  `story_only`), `error` (`ContentError`, `ContentErrors`, `line_of`) and
  `ron` (`parse_ron`, plus `load_ron(&Bundle, path)`). Default `std`
  feature; builds `no_std` for the Playdate, and CI's `no-std` job and the
  `run-gates` list now build it.
- `trpg-content` keeps its `include_dir!` calls, both features and
  `build.rs`; `bundle::layers()` (re-exported as `trpg_content::layers`)
  builds its `Vec<Layer>`, `bundle::install()` installs it, and the old
  free functions, constants, `error` and `ron_loader` paths stay as thin
  wrappers or re-exports, so no other crate changed.
- `bundle_table.rs` is gone: 1028's generated table is now the bundle's
  one layer without `std` (`Layer::table`). The test that keeps it equal
  to `include_dir` compares the two layers.
- Fixture tests moved to the engine with neutral layer names (`base`,
  `bought`, `tests`, `bought-tests`): the boundary gate bans the game's
  folder names there. Tests that read the game's files stay in
  `trpg-content`. New engine tests: a table layer answers as the same
  directory, the bundle installs once, `load_ron` over a bundle.
- Checked by hand: `cargo test -p trpg-content --features private-assets
  --test it private_assets::` passes with the private repository at the
  pinned revision (what Pages runs).

Deviations from the plan:

- `Layer` holds `files: Files` (an `include_dir` directory or a sorted
  table) instead of `dir: &'static Dir`: `include_dir` needs `std`, and
  the ticket asks for the `no_std` table to be a layer.
- `story_only` is a free function, not a method: it hides test layers
  from the thread, for every bundle.
- The process-wide bundle is a `once_cell::race::OnceBox`, not a
  `OnceLock`, so it builds without `std`. `once_cell` (MIT OR Apache-2.0,
  already a workspace crate) is added to the engine's allowed list.
- The game's bundle is installed on the first file read as well as at the
  start of `load_embedded` (and `load_with_scripts`): many tests, the app
  and the tools read files without loading everything first.
- `is_private` asks the bundle's private layers instead of two fixed
  names. Same answer for every name `display_path` can give.
- A path no layer has is shown under the first layer that is neither
  private nor test files (`assets/` for the game, as before).

Follow-up: 1056 (the debug Playdate build's `pdc` step crashes, on
`main` too; the release build packs fine).

`trpg-content` no longer depends on `thiserror` (its errors are the
engine's now).

No gameplay rules decided. Nothing changes for Nick when playing.
