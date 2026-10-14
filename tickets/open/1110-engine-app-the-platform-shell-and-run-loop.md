---
id: "1110"
title: "shuyi-app: the platform shell and run loop; trpg-app becomes a small binary that picks the game"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1109"]
nick_input: none
completed:
---

# 1110 — shuyi-app: the platform shell and run loop

## Context

`crates/app` is already almost all platform (`docs/engine/boundary.md`,
*app*): rendering, keys, controllers (gilrs, the web pad plugin),
storage (files, `localStorage`), audio (sounds, music decoded on a worker
thread, voice), and the main loop. Its game leaks: the binary name, the
window title and icon, exe resource strings, storage names
`tactical-rpg` (`storage/native.rs:15`, `storage/web.rs:11`),
`Ctx::embedded`, the start screen, the release check on test files, and
`args.rs` (`--scene`, `--lead`).

## Nick input

None. The game must look, sound and save exactly as before.

## Scope

**In:**
- New crate `engine/crates/app` (package `shuyi-app`) with
  `pub fn run<G, A>(config: AppConfig, make: impl FnOnce(&Platform) ->
  shuyi_ui::Game<G, A>)`. `AppConfig`: window title, size, icon bytes,
  storage name (native folder and web prefix), the font atlas, extra
  checks to run at start (the game's release check).
- Move `render.rs`, `keys.rs`, `pads*`, `storage/`, `audio*` and the loop
  from `main.rs`. The web JS plugin names `trpg_pad_*` become
  `shuyi_pad_*` (update `web/` and `xtask web` in the same PR).
- `trpg-app/src/main.rs` keeps the `#[macroquad::main]` entry (macroquad
  needs it in the binary), builds `AppConfig` and calls `run`. `args.rs`
  stays the game's.
- The storage name stays `tactical-rpg` for this game so saves are
  found (test: the path is unchanged).

**Out (do not do):** new platform features (mouse, touch: 1121); a
headless runner (the Harness covers it).

## Implementation steps

1. Create the crate; move modules; parameterize the leaks.
2. Keep `#[macroquad::main]` in the binary; the engine's `run` is
   `async` and called from it.
3. Native and web builds; play the title, a battle and a save/load by
   hand, and through the browser pane on the web build.

## Acceptance criteria

- [ ] `trpg-app/src/main.rs` is under 120 lines.
- [ ] Saves written before the change load after it, native and web
      (same folder and prefix).
- [ ] `cargo build -p trpg-app --target wasm32-unknown-unknown` and the
      native build pass; the web build runs in the browser pane.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: storage path and prefix from `AppConfig`.
- Manual: native and web smoke test (listed in the PR).

## Completion notes
