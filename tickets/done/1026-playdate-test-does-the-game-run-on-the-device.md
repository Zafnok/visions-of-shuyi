---
id: "1026"
title: "Playdate test: do the rules build for it, and how long does an enemy turn take"
type: research
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: done
blocked_by: []
nick_input: none
completed: 2026-10-15
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

- [x] An ADR under `docs/adr/` records the bindings, toolchain, licenses,
      per-crate `no_std` findings and enemy-turn timings, and is in the
      ADR index.
- [x] The `spike/playdate` branch is pushed and linked from the ADR. It
      builds a Playdate game that runs the Chapter 1 enemy phase in the
      simulator.
- [x] 1025 is updated with the findings.
- [x] `main` gains no Playdate code: `git diff main --stat` on this
      ticket's branch shows only `docs/` and `tickets/`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None on `main` (docs only). The spike's timings are recorded in the
  ADR. They are measurements, not tests.

## Completion notes

**Done.** [ADR-0070](../../docs/adr/0070-playdate-front-end.md) records
the answers; 1025's *Context* and *Scope* carry them; the code is on the
pushed, never-merged branch
[`spike/playdate`](https://github.com/Zafnok/visions-of-shuyi/tree/spike/playdate)
(`playdate/README.md` there says how to rerun it).

1. **Do the crates build for the device?** Yes, all three: `core`,
   `content` and `ui` build for `thumbv7em-none-eabihf` with `no_std` +
   `alloc` behind a default `std` feature (+140/−62, +318/−73 and
   +401/−109 lines), and their 703 + 657 + 1,844 tests pass unchanged with
   `std`. Beyond the expected path changes: `include_dir` is replaced by a
   generated file table, `LazyLock` by `once_cell`'s `OnceBox`, float
   rounding and trig by `libm`, two `HashMap`s by `hashbrown`.
2. **How long do enemy turns take?** Measured on Nick's Playdate (Rev B):
   a 14-enemy phase 1,162 ms, the average of 31 Chapter 1 phases 493 ms,
   the slowest single enemy decision 226 ms; loading every data file
   3,708 ms. The device is ~700× slower than the PC; the Simulator runs at
   PC speed.
3. **Bindings and toolchain:** none of the binding crates; stable Rust
   1.98.1, the SDK's own 0BSD C build support, MSYS2's `arm-none-eabi-gcc`
   and MinGW `gcc`, `pdc`. No MSVC, no nightly. SDK 3.1.2.

**Deviations.**
- The ticket expected to pick a binding crate. boozook's `cargo-playdate`
  (the likely pick) cannot build this workspace: it no longer compiles on
  current nightly, and on its pinned nightly its built-in Cargo cannot read
  edition-2024 manifests. The others are unmaintained or self-described as
  barely functional. The front end uses a thin C shim of its own instead
  (ADR-0070 §2).
- Nick sideloaded two builds (he has a Playdate, so no estimate was
  needed). The first crashed while loading: the game needs ~15 KB of stack
  and the device gives ~10 KB. The second ran the same code on a 256 KB
  stack from the heap and measured its use. ADR-0070 §4 makes that the
  front end's design.
- The device test loads the data with the game's own RON loader (once
  `content` built without `std`) rather than a pre-converted blob, so the
  load time is measured too. Only the 31 phase start states travel as a
  postcard file, made on the PC.
- Tools installed on Nick's PC for this (he agreed to each download): the
  Playdate SDK 3.1.2 (he ran the installer), MSYS2's
  `mingw-w64-ucrt-x86_64-arm-none-eabi-toolchain`, the Rust target
  `thumbv7em-none-eabihf`, Rust `nightly` and `nightly-2025-04-25`, and
  `cargo-playdate` 0.5.8 (to test it; not used by the decision).

**Follow-up tickets:** none created. 1025 writes the build tickets; its
Scope now lists the xtask/CI build, the own-stack front end, loading
time, and an optional AI speed-up (ADR-0070 §5).

**Gameplay rules decided:** none.

**For Nick:** nothing changes in the game. Enemy turns on the Playdate
will have a short pause before each enemy moves (about 0.1 s, at most a
quarter second), and the game will take a few seconds to start; both can
be cut later if they bother you when you play it.
