---
id: "1209"
title: See a changed game file in the running debug build
type: feature
milestone: "Engine: carve"
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["engine#2220"]
nick_input: none
completed:
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

- [ ] With the headless backend's script, a changed palette file is seen by
      the game on the next frame.
- [ ] A changed `.dlg` shows the next time its scene plays, without a rebuild.
- [ ] A broken file keeps the old value, and its error names the file and line.
- [ ] A build without the debug tools embeds the same files as before and
      never asks for changes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the first three criteria through the Harness and the headless
  backend's `change_file`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
