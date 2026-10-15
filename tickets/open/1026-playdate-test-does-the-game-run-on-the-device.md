---
id: "1026"
title: "Playdate test: do the rules build for it, and how long does an enemy turn take"
type: research
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1026 — Playdate test: do the rules build for it, and how long does an enemy turn take

## Context

Nick, 2026-10-15: "I wanna rush a playdate port!!! how can we get ch1 on
playdate asap?!" He chose to unpark the port (1025) and run this test
first, ahead of the engine carve (1103–1131). The carve stays as planned;
this ticket does not wait for it and does not change it.

What a first look found (1025, *Context*; check it again here):

- `trpg-core` depends only on `serde`. Its `std` uses are `BTreeMap`,
  `BTreeSet`, `Arc`, `Rc`, `fmt`, `iter::once`, `mem::take` and
  `std::error::Error`. All of these exist in `core`/`alloc`
  (`core::error::Error` is stable since Rust 1.81). `std::time::Instant`
  appears only in tests (`ai/tests.rs`, `movement/tests.rs`).
- `trpg-content` needs more: `ron`, `include_dir`, `thiserror`,
  `std::sync::LazyLock` (`lang.rs:15`), `std::fs` (`audio.rs:745,761`,
  `credits.rs:676`, likely debug or tool paths) and `std::thread::spawn`
  (`bundle.rs:395`).
- `trpg-ui` uses `Rc`, `RefCell`, `Any` and `VecDeque` (all in
  `core`/`alloc`), plus `ron` and `thiserror`.
- The Rust Playdate bindings (`crankstart`, `craydate`, and boozook's
  `playdate` crates, built with `cargo-playdate`) build a `#![no_std]`
  game with an allocator. Some need nightly Rust. The device is a
  168 MHz ARM Cortex-M7 with 16 MB of RAM. Check all of this against the
  current SDK and crate docs.
- The embedded assets (`assets/`) are about 1 MB.

This ticket answers three questions. The port's ADR records the answers,
and 1025 uses them to write the build tickets:

1. Can `core`, `content` and `ui` build for the device (no `std`, with
   `alloc`)? What would it take, crate by crate?
2. How long do the enemy AI's turns in Chapter 1 take on the device?
3. Which binding crate and toolchain do we use, under ADR-0013's
   licenses, on Nick's Windows machine with the GNU toolchain (no MSVC)?

## Nick input

`None.` If Nick owns a Playdate, the session may ask him (in plain words,
no technical questions) to sideload one test build and read out the
timing it shows on the screen. This is optional: the simulator plus a
measured scale factor is enough to finish.

## Scope

**In:**
- Re-check the facts in 1025's *Context* and the list above against the
  current Playdate SDK, its license, and the binding crates' docs and
  licenses.
- A **throwaway spike branch** (`spike/playdate`, never merged) that:
  - adds a `std` feature to `trpg-core` (on by default) and makes the
    crate build with `#![no_std]` + `alloc` when it is off;
  - builds a minimal Playdate game that embeds the Chapter 1 battle data,
    runs the enemy phase of Chapter 1's battles from their start states,
    and shows the time per enemy turn on screen;
  - tries `trpg-content` and `trpg-ui` the same way and lists what
    breaks (it does not need to fix it all).
- ADR `docs/adr/NNNN-playdate-front-end.md` (`write-adr` skill), with:
  - the chosen bindings, SDK version and toolchain, and their licenses;
  - the per-crate `no_std` findings, with an estimate of changed lines;
  - the enemy-turn timings (simulator, and device if measured);
  - how the Playdate front end fits ADR-0004 and ADR-0054: a new crate
    beside `app` that paints the same views with its own 1-bit skins;
  - what changes when engine carve tickets 1108 (console size) and 1110
    (platform shell) land;
  - a link to the spike branch.
- Update 1025 with what the ADR found, plus anything that changes the
  build tickets it will write.

**Out (do not do):**
- Merging any Playdate code, or `no_std` changes, into `main`. 1025's
  build tickets do that properly.
- Layouts, fonts or skins for the 400 × 240 screen. That is 0061 (design)
  and then 1025's tickets.
- Any change to the desktop or web game.
- The engine carve (1103–1131).
- Requiring MSVC locally. If a tool needs it, record that and find a
  GNU-toolchain or WSL path.

## Implementation steps

1. Read 1025, ADR-0004, ADR-0013, ADR-0038, ADR-0054, ADR-0063 and
   `docs/engine/boundary.md`.
2. Look up the current Playdate SDK (version, license, Windows install,
   the ARM toolchain it needs) and the Rust bindings: `playdate` /
   `cargo-playdate` (boozook), `crankstart`, `craydate`. Compare them on
   license (ADR-0013 allowed list), upkeep (last release), nightly or
   stable Rust, Windows GNU support, and simulator support. Pick one.
   **Ask Nick before downloading the SDK.** Name the file, the source and
   its size (safety rule for downloads).
3. On `spike/playdate`, make `trpg-core` build with `--no-default-features`
   for the device target: `#![cfg_attr(not(feature = "std"), no_std)]`,
   `extern crate alloc`, and `alloc::`/`core::` paths in place of `std::`.
   Check that `serde` still builds with `default-features = false,
   features = ["alloc", "derive"]`. Record the changed line count.
4. Write the smallest Playdate game that loads Chapter 1's battle data.
   Embed the RON as bytes, and parse it on the host into a `postcard`
   blob if `ron` will not build. Then step the enemy phase with
   `trpg_core`'s AI and draw the time per turn on screen. Run it in the
   simulator. If Nick has a device, measure there too. If not, time the
   same turns on the host and in the simulator, and estimate the device
   time from the published CPU speed. Say clearly that it is an estimate.
5. Try `trpg-content` and `trpg-ui` with `no_std`. For each error group,
   list the cause and a fix (for example: `LazyLock` → `once_cell` with
   `critical-section`; `std::fs` paths behind the `std` feature;
   `thiserror` 2 with `default-features = false`).
6. Write the ADR (step "In" above). Update 1025 and its
   `## Completion notes`-to-be with the numbers. Keep the spike branch
   pushed and link it.
7. Run `cargo xtask ticket-lint` and the `run-gates` skill. Only docs and
   tickets change on this ticket's branch.

## Acceptance criteria

- [ ] An ADR under `docs/adr/` records the bindings, toolchain, licenses,
      per-crate `no_std` findings and enemy-turn timings, and is in the
      ADR index.
- [ ] The `spike/playdate` branch is pushed and linked from the ADR. It
      builds a Playdate game that runs the Chapter 1 enemy phase in the
      simulator.
- [ ] 1025 is updated with the findings.
- [ ] `main` gains no Playdate code: `git diff main --stat` on this
      ticket's branch shows only `docs/` and `tickets/`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None on `main` (docs only). The spike's timings are recorded in the
  ADR. They are measurements, not tests.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
