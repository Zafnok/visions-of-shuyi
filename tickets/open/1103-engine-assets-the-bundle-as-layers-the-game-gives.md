---
id: "1103"
title: "shuyi-assets: the asset bundle as layers the game gives, plus file errors and RON loading"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: todo
blocked_by: ["0804", "1102"]
nick_input: none
completed:
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

Waits for 0804: it touches every loader in `content`, which the playtest
tickets also change.

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

- [ ] `shuyi-assets` builds and tests alone (`cargo test -p
      shuyi-assets`) with no game file.
- [ ] Every existing test passes unchanged; no snapshot changes.
- [ ] `cargo build -p trpg-app --target wasm32-unknown-unknown` passes.
- [ ] `docs/engine/boundary.md` rows for `bundle.rs`, `build.rs`,
      `error.rs`, `ron_loader.rs` updated to say where they now live.
- [ ] All gates in the `run-gates` skill pass, including
      `check-engine-boundary`.

## Tests required

- Unit: bundle layering over fixture directories (moved tests).
- Integration: the game's `all_assets_load` still passes.

## Completion notes
