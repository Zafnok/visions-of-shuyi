# ADR-0070: The Playdate front end: stable Rust, the SDK's own C build, rules without `std`

- **Status:** Accepted
- **Date:** 2026-10-15
- **Related tickets:** 1026 (this test), 1025 (the build tickets), 0061
  (Nick's Playdate decisions), 1108 and 1110 (engine carve)

## Context

Nick, 2026-10-15: "I wanna rush a playdate port!!! how can we get ch1 on
playdate asap?!" Ticket 1026 tested whether the game's code runs on the
device, how long the enemy AI takes there, and which tools to build with.
The test is on the throwaway branch
[`spike/playdate`](https://github.com/Zafnok/visions-of-shuyi/tree/spike/playdate)
(never merged; its `playdate/README.md` says how to rerun it).

**The device and SDK** (checked 2026-10-15):

- Playdate SDK **3.1.2** (2026-09-16). Windows installer
  `PlaydateSDK-3.1.2.exe`, 45 MB, from `download.panic.com`; Linux tarball
  34 MB. The C build support it ships (`C_API/buildsupport/setup.c`,
  `link_map.ld`, `common.mk`) is under **0BSD**. The SDK itself is under
  Panic's SDK license: free and royalty-free, games are ours to sell
  anywhere; the SDK may not be redistributed, decompiled, or used for
  other platforms; "Playdate" or "Panic" may not be in the game's name
  without permission. Panic's Catalog store has its own terms (0061 asks
  Nick where to sell).
- CPU: ARM Cortex-M7 at 168 MHz. Rev A is an STM32F746 with a
  single-precision FPU; Rev B (Nick's, `target=h7d1`) is an STM32H7B0.
  16 MB of RAM holds the game's code and heap. Screen: 400 × 240, 1-bit.
- The C stack is small: Playdate developers report about **10 KB** on the
  device (there is no official figure; the `STACK_SIZE=61800` in old
  makefiles is said to be unused).
- On Windows, Panic's C instructions use Visual Studio (MSVC) and CMake
  for the Simulator build. Nick's machine has no MSVC (CLAUDE.md).

**The Rust bindings** (all under licenses ADR-0013 allows):

| Crate | Last release | What happened |
| ----- | ------------ | ------------- |
| `playdate` / `playdate-sys` / `cargo-playdate` (boozook), MIT OR Apache-2.0 | 0.5.8, 2025-03 | `cargo-playdate` no longer compiles on today's nightly (an unstable API changed). Built with its pinned `nightly-2025-04-25`, its built-in Cargo (1.81) **cannot read edition-2024 manifests**, which is every crate in this workspace. Its prebuilt bindings stop at SDK 2.6.2 (3.1.2 needs bindgen and libclang). Packaging needs symlink rights (Windows Developer Mode). |
| `crankstart` (pd-rs), MIT | 0.1.2, 2023-09 (git active) | Describes itself as "barely functional"; git path dependency, the `crank` tool, nightly. |
| `craydate`, MIT/Apache-2.0 | 0.1.7, 2022-04 | Unmaintained. |
| `playdate-rs`, MIT OR Apache-2.0 | 0.0.15, 2023-09 | Unmaintained. |

**The measurements.** The spike made `trpg-core`, `trpg-content` and
`trpg-ui` build for `thumbv7em-none-eabihf` with `#![no_std]` + `alloc`,
then ran a Playdate program that calls `trpg_content::load_embedded()` on
the device (every data file parsed from RON, as the game does) and runs
the AI phases of Chapter 1: 31 phase start states from five baseline-bot
tries (seeds 1–5), each phase run three times, the fastest kept.

| | PC (host) | Simulator | Playdate Rev B |
| --- | --- | --- | --- |
| Load all game data | 5 ms | 5 ms | **3,708 ms** |
| Turn 1 enemy phase (14 enemies, 15 commands) | 1.7 ms | 2 ms | **1,162 ms** |
| Slowest enemy phase | 1.8 ms | 2 ms | 1,162 ms |
| Average enemy phase (31 phases) | 0.72 ms | 1 ms | **493 ms** |
| Slowest single enemy decision | 0.38 ms | 0.4 ms | **226 ms** |
| Stack used (load + AI) | — | — | **15,256 bytes** |

- The device is about **700× slower** than the PC on both the loading and
  the AI, far more than the clock speeds suggest (a guess from clocks
  alone would be 30–100×). The Simulator runs the code natively on the
  PC, so **its timings say nothing about the device**: only device runs
  count.
- The first device build ran on the system stack and **crashed while
  loading**; the same code on a 256 KB stack taken from the heap finished
  and used 15 KB of it. The crash is consistent with the ~10 KB stack.
- `f64` maths (the AI's scores) is done in software: the build targets
  Rev A's single-precision FPU.
- The device binary (`pdex.bin`) is 1.6 MB, with the ~1 MB of embedded
  `assets/`. The link pulled in nothing from newlib or libgcc: only our
  code, Rust's `compiler_builtins` (MIT/Apache-2.0) and `setup.c` (0BSD).

## Decision

### 1. Toolchain: stable Rust and the SDK's own C build, GNU tools only

- **Rust:** the repository's pinned stable toolchain (1.99.1) and the
  tier-2 target `thumbv7em-none-eabihf` (its `core`/`alloc` come
  prebuilt; no nightly, no `build-std`). Flags:
  `-C target-cpu=cortex-m7 -C target-feature=-fp64 -C relocation-model=pic`
  (single-precision FPU for Rev A; position-independent, so no MOVW/MOVT
  address pairs, which `pdc` cannot relocate; the SDK's C build uses
  `-mword-relocations` for the same reason). Release profile with
  `panic = "abort"`, `lto = true`, `codegen-units = 1`. The front end is a
  `staticlib`.
- **Device link:** `arm-none-eabi-gcc` from MSYS2
  (`mingw-w64-ucrt-x86_64-arm-none-eabi-toolchain`, GCC 16.1) compiles
  the SDK's `setup.c` and our C shim with the flags of the SDK's
  `common.mk`, and links them with the Rust library through
  `link_map.ld` and `--emit-relocs` into `pdex.elf`.
- **Simulator:** MSYS2's MinGW `gcc` links the same shim, `setup.c` and an
  `x86_64-pc-windows-gnu` build of the library into `pdex.dll` (with
  `-D_WINDLL`). The Simulator (an MSVC program) loads it fine. A
  `no_std` library on that target needs an empty `rust_eh_personality`.
- **Packaging:** the SDK's `pdc` packs both into one `.pdx` that runs in
  the Simulator and on the device. An `xtask` command does all of this
  (1025 writes it from `spike/playdate`'s `build.sh`).
- **CI** (1025): Linux runners install `gcc-arm-none-eabi` and download the
  SDK's Linux tarball from Panic on each run (the SDK is never committed:
  its license forbids redistributing it).

### 2. No binding crate: a thin C shim of our own

The front end calls the SDK through a small C file (`eventHandler`, plus
one wrapper per SDK call it uses: draw, buttons, crank, files, timer,
`realloc`, log), compiled against the installed SDK's real `pd_api.h`. The
Rust side declares those wrappers in an `extern "C"` block. The headers
stay the source of truth for the API's layout, a new SDK is picked up by
recompiling, and nothing depends on a binding crate's upkeep or nightly.
The SDK headers are never copied into the repository.

### 3. Rules without `std`: a default `std` feature per crate

`trpg-core`, `trpg-content` and `trpg-ui` get a default `std` feature;
with it off they build with `#![no_std]` + `extern crate alloc`. What the
spike changed (line counts from `git diff --shortstat` on the branch;
about 195 of the added lines are the same two-line prelude import):

| Crate | Changed lines | What it takes |
| ----- | ------------- | ------------- |
| `core` | +140 / −62 in 33 files | `std::` paths to `core::`/`alloc::` (all of `BTreeMap`, `Arc`, `Rc`, `fmt`, `iter`, `mem`, `cmp`, `cell`, `error::Error` exist there); a crate prelude (`Vec`, `String`, `Box`, `ToString`, `ToOwned`, `vec!`); `serde` with `default-features = false, features = ["derive", "alloc"]` at the workspace, `std` turned on by the `std` features. |
| `content` | +318 / −73 in 49 files | As `core`, plus: `include_dir` does not build without `std`, so a `build.rs`-generated sorted table of `(path, include_bytes!(…))` replaces it (same public functions; the test files and private layer stay `std`-only); `LazyLock` → `once_cell::race::OnceBox`; `f32`/`f64` `round` → `libm`; `Path::extension` → a string split; `ron` and `thiserror` with `default-features = false` (both support `no_std`). |
| `ui` | +401 / −109 in 120 files | As `core`, plus: 26 float calls (`round`, `floor`, `sin`, `cos`, `hypot`, `rem_euclid`) through a `libm`-backed trait used only without `std`; `HashMap` (2 uses) → `hashbrown`; one `LazyLock` → `OnceBox`. |

New crates for this: `libm`, `once_cell`, `hashbrown`, all MIT OR
Apache-2.0. With `std` on (every existing build), all 703 + 657 + 1,844
tests of the three crates pass unchanged on the branch. A CI job
(`cargo build --no-default-features --target thumbv7em-none-eabihf` for
the three crates) keeps them building once the first build ticket lands.

### 4. The front end's crate and how it fits ADR-0004 and ADR-0054

- A new crate **`crates/playdate`** (`trpg-playdate`), beside `app`. Like
  `app` it is a platform crate: it owns the run loop (the SDK's update
  callback), buttons and crank turned into `Action`s through the
  player's keymap (`keyboard-input` skill), the clock (`dt` from the
  SDK's timer), storage (the SDK's file calls), sound, and the allocator.
  It contains no game or screen logic, and like `app` it may hold the
  workspace's `unsafe` (ADR-0004 rule 5): the FFI, the allocator and the
  stack switch.
- `core`, `content` and `ui` run unchanged apart from §3. Screens build
  their views and `MapScene` as today (ADR-0038, ADR-0054). The Playdate's
  look is **new skins**: a 1-bit `paint` per view and a Playdate map skin,
  sized for 400 × 240 and drawn from 0061's answers, chosen where `draw`
  chooses today's glyph skin. The front end then draws the frame to the
  1-bit screen.
- **The game runs on its own stack.** The front end allocates a stack
  from the heap (64 KB: four times the 15 KB measured) and enters it with
  a few lines of inline assembly around each call into the game. Shrinking
  every deep call path under 10 KB instead would be a hunt with no end.
  Each build ticket that adds work on the device rechecks the high-water
  mark (the spike's painted-stack measurement).

### 5. What the numbers mean for Chapter 1

- **Enemy turns:** a 14-enemy phase thinks for 1.2 s in all, 80 ms per
  enemy on average and 0.23 s at worst, spread over the enemies' moves.
  Each enemy's move is already animated one by one, so this is a short
  pause before each enemy acts. Playable for Chapter 1 as it is.
  `next_command` replans every enemy for each command (the phase costs
  about n² decisions), so planning once per phase is the obvious speed-up
  if 1025's tickets or Nick's play-through find the pauses too long. It
  must keep the AI's choices identical (determinism, ADR-0004).
- **Loading:** 3.7 s to parse every data file at start-up. 1025 starts by
  loading only what the Playdate version uses (0061: one build per
  language, so one language pack instead of all of them), behind a
  loading card, and measures again. Converting the data to a binary format at build time
  is the fallback if it is still too slow.
- Timings come from the device only, never the Simulator (§Context).
  Nick's Playdate is Rev B; Rev A is unmeasured (its RAM is said to be
  faster).

### 6. The engine carve (ADR-0063)

- **1108 (console size):** until it lands, the console is a fixed 100 × 32
  cells. The Playdate screens need their own size, so 1025 makes the
  Playdate skin tickets wait for 1108's `ConsoleSpec` (or do that part of
  1108 first) rather than fork the constants. After 1108 the front end
  passes its own `ConsoleSpec`.
- **1110 (platform shell):** the Playdate front end stays a **second front
  end beside the engine's app**, not part of it: no `std`, no threads, a
  C entry point and a callback loop leave little to share. What it shares
  is `Game::frame` (input events and `dt` in, a frame out), which 1110
  keeps platform-neutral.
- **The carved crates** (`shuyi-*`) keep the `std` feature and the
  `no_std` CI build from §3, so the engine stays buildable for the device.
  The engine's public repository may hold a Playdate front end too, as
  long as no SDK file is committed.

## Consequences

- No MSVC, no nightly, no CMake, no bindgen: Nick's machine already has
  every tool, and CI's Linux runners can build the device binary.
- We maintain the C shim and the FFI ourselves: a few dozen small
  functions, checked by the C compiler against the SDK's headers.
- `core`, `content` and `ui` gain a `std` feature and a crate prelude; code
  in them must not use `std`-only items outside `#[cfg(feature = "std")]`
  (the `no_std` CI build catches it). Float maths in `content` and `ui`
  goes through `libm`.
- `include_dir` stays for the `std` build only; the `no_std` bundle is a
  generated table. Two code paths for one bundle until the engine's
  bundle (1103, the bundle as layers) replaces both.
- Device performance needs device runs; the Simulator is only for
  correctness. Every 1025 ticket with device-visible work says how it was
  measured on Nick's Playdate.
- ADR-0013: the Playdate SDK is a platform SDK like Steamworks (free,
  royalty-free, not redistributed); `setup.c` is 0BSD. Both are listed
  in `THIRD_PARTY_ASSETS.md` by the first build ticket that ships them.

## Alternatives considered

- **boozook's `playdate` crates with `cargo-playdate`:** the most complete
  bindings, but on 2026-10-15 the tool does not build on current nightly,
  and with its pinned nightly it cannot read edition-2024 manifests, so it
  cannot build this workspace at all; plus stale prebuilt bindings,
  bindgen/libclang for SDK 3.x, and Developer Mode for symlinks. Revisit
  if it gets a release for SDK 3 and edition 2024.
- **`crankstart`:** self-described as barely functional, nightly, a git
  dependency and an extra tool.
- **Hand-written Rust copies of `pd_api.h`'s structs** (no C at all):
  removes the shim, but a mistake in a transcribed struct of function
  pointers fails silently at run time, and every SDK update means
  re-checking the layout by hand. The C compiler does that for free.
- **Panic's CMake + Visual Studio build:** needs MSVC on Nick's machine.
- **WSL with the Linux SDK:** works, but the Simulator would run in WSL's
  window system; the MinGW path keeps everything native and is no harder.
- **Run the game on the system stack and cut stack use:** 15 KB measured
  against ~10 KB available; RON parsing and the AI recurse through deep
  generic code, and every new feature would risk a crash only the device
  shows.
- **Estimate device speed from the Simulator and the clock speed** (the
  ticket's fallback): the measured slowdown (~700×) is several times what
  clocks predict, so an estimate would have been badly wrong.
