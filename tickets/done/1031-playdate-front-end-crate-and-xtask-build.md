---
id: "1031"
title: "Playdate: the front-end crate, the game on its own stack, and `cargo xtask playdate` with a CI build"
type: infra
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1029"]
nick_input: none
completed: 2026-10-15
---

# 1031 — The Playdate front end's crate and build

## Context

[ADR-0070](../../docs/adr/0070-playdate-front-end.md) decided how the
Playdate build works: stable Rust, the SDK's own C build support
(`setup.c`, `link_map.ld`, 0BSD), a thin C shim of our own (no binding
crate), MSYS2's `arm-none-eabi-gcc` for the device and MinGW `gcc` for the
Simulator's `pdex.dll`, packed by `pdc` (§1, §2). The game needs ~15 KB of
stack and the device gives ~10 KB, so the front end runs the game on a
stack it allocates (§4). The spike's `playdate/bench/build.sh` and
`shim/main.c` on
[`spike/playdate`](https://github.com/Zafnok/visions-of-shuyi/tree/spike/playdate)
do all of this in a throwaway form.

This ticket makes the real crate and build, with just enough inside to
prove them: it loads the game's data on the device and shows how long
that took and how much stack it used. The game itself runs in 1033.

## Nick input

None to decide. Nick sideloads one build to his Playdate (see
*Measured on Nick's Playdate*).

## Scope

**In:**
- New crate `crates/playdate` (package `trpg-playdate`), a `staticlib`
  (and `rlib` for host checks). `no_std` + `alloc`. Depends on
  `trpg-core`, `trpg-content`, `trpg-ui` with `default-features = false`.
  ADR-0004: like `app` it is a platform crate and may hold `unsafe` (the
  FFI, the allocator, the stack switch); no game or screen logic.
- `crates/playdate/shim/shim.c`: `eventHandler` (exported for the
  Simulator with `__declspec(dllexport)` under `_WINDLL`), the update
  callback, and one small wrapper per SDK call used so far (`realloc`,
  log, error, timer, `getCurrentTimeMilliseconds`, `drawText`, `clear`,
  file open/read/write/close). Compiled against the installed SDK's
  `pd_api.h`; the SDK's headers are never copied into the repository.
- Rust side: the `extern "C"` declarations, a `GlobalAlloc` over the
  SDK's `realloc`, a `#[panic_handler]` that calls the SDK's `error` with
  the message, and a `rust_eh_personality` stub for the Simulator build
  (ADR-0070 §1).
- **The game's own stack** (ADR-0070 §4): 64 KB allocated from the heap
  once; every call from the update callback into game code goes through
  a `run_on_stack(f)` that switches the stack pointer with a few lines of
  inline assembly (Thumb on the device; on the Simulator, x86-64, either
  the same idea or a plain call: the Simulator's stack is the PC's). The
  stack is painted with a pattern first; `stack_high_water()` reports the
  deepest byte used. Log it once a second while debugging and on demand.
- What runs in this ticket: on start, show "Loading…" with the SDK's
  `drawText` (temporary, replaced by 1036's card), call
  `trpg_content::load_embedded()` on the game's stack, then show the load
  time (ms) and the stack high-water mark, and log both.
- `cargo xtask playdate [--release] [--device|--simulator|--both]`
  (default both): builds the Rust library for each target, compiles and
  links the C side with the flags from the spike's `build.sh` and the
  SDK's `common.mk`, writes `pdxinfo` (name, author, bundle ID
  `com.zafnok.visionsofshuyi` or similar, version from the workspace),
  and runs `pdc` into `target/playdate/VisionsOfShuyi.pdx`. Finds the SDK
  from `PLAYDATE_SDK_PATH`, else `~/Documents/PlaydateSDK`; finds the
  compilers from `PATH`, else MSYS2's default places
  (`C:\msys64\ucrt64\bin\arm-none-eabi-gcc`,
  `C:\msys64\mingw64\bin\gcc`). Clear errors naming what is missing and
  how to install it.
- The build flags from ADR-0070 §1: `-C target-cpu=cortex-m7
  -C target-feature=-fp64 -C relocation-model=pic`; a `playdate` release
  profile with `panic = "abort"`, `lto = true`, `codegen-units = 1` (in
  the workspace `Cargo.toml`; don't change the existing profiles).
- **CI job `playdate`** (`.github/workflows/ci.yml`, Linux runner):
  installs `gcc-arm-none-eabi`, downloads the SDK's Linux tarball from
  Panic (`download.panic.com`, version pinned to 3.1.2 in one place;
  cached by version; never committed: its license forbids redistributing
  it), runs `cargo xtask playdate --device` and checks `pdex.elf` was
  linked and `pdc` produced the `.pdx`. On Linux the device build is
  enough (the Simulator `.dll` is Windows-only; `pdc` and `pdex.elf` are
  the point). Wire it into `ci-result` and the `changes` filter.
- Workspace hygiene: `cargo test --workspace`, `cargo clippy --workspace`
  and the WASM build keep working (e.g. `test = false`, the panic
  handler and allocator under `cfg(target_os = "none")` or
  `cfg(not(test))`). `cargo xtask check-engine-boundary` and
  `check-keys` pass (no key or button names in this ticket).
- `THIRD_PARTY_ASSETS.md`: rows for the Playdate SDK (platform SDK, not
  shipped in the repository; Panic's SDK license) and `setup.c` (0BSD,
  linked into the game), per ADR-0070 *Consequences* and ADR-0013.
- `crates/playdate/README.md`: what to install (SDK 3.1.2, MSYS2
  `mingw-w64-ucrt-x86_64-arm-none-eabi-toolchain`,
  `rustup target add thumbv7em-none-eabihf`), how to build, how to run in
  the Simulator, how Nick sideloads (Simulator: *Device → Upload Game to
  Device*) and where device logs show (the Simulator's console while the
  device is plugged in).
- `CLAUDE.md`'s repo map: a `crates/playdate` row.

**Out (do not do):**
- Running screens, drawing frames, input, saves (1033 and later).
- The 1-bit renderer (1032).
- Loading less data (1036).

**Carve (1110):** the Playdate front end stays a second front end beside
the engine's app (ADR-0070 §6). If 1110 has landed, nothing here changes;
if not, 1110's note already says to keep this crate building.

## Implementation steps

1. Read ADR-0070 and the spike's `playdate/bench/` (`build.sh`,
   `shim/main.c`, `src/lib.rs`: the allocator, the stack switch and the
   painted-stack measurement are there).
2. Create the crate, the shim, the allocator, panic handler and
   `run_on_stack`.
3. Write `xtask playdate` from `build.sh` (Rust, not bash; follow
   `xtask web`'s shape). Test its pure parts (flag lists, `pdxinfo` text,
   SDK path lookup) with unit tests.
4. CI job; check it runs green on the PR.
5. Build both targets on Nick's machine; run in the Simulator; ask Nick
   to sideload.

## Measured on Nick's Playdate

Nick sideloads `VisionsOfShuyi.pdx` and pastes the Simulator console. The
PR lists: load time on the device (1026 measured 3,708 ms) and the stack
high-water mark (1026: 15,256 bytes; it must stay well under 64 KB).

## Acceptance criteria

- [x] `cargo xtask playdate` builds `target/playdate/VisionsOfShuyi.pdx`
      on Nick's machine (GNU tools only, no MSVC, no nightly).
- [x] The `.pdx` runs in the Simulator and shows the load time and stack
      mark.
- [ ] On Nick's Playdate it loads without crashing; the numbers are in
      the PR.
- [ ] The CI `playdate` job passes.
- [x] `THIRD_PARTY_ASSETS.md` lists the SDK and `setup.c`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `xtask playdate`'s pure helpers (flags, `pdxinfo`, SDK lookup).
- CI: the device build.
- Manual: Simulator run; one device run (numbers in the PR).

## Completion notes

**Done.** New crate `crates/playdate` (`trpg-playdate`) and
`cargo xtask playdate [--release] [--device|--simulator|--both]`:

- `shim/shim.c`: `eventHandler` (exported for the Simulator under
  `_WINDLL`), the update callback, and one wrapper per SDK call
  (`realloc`, log, error, the millisecond clock and the elapsed timer,
  `clear`, `drawText`, file open/read/write/close), compiled against the
  installed SDK's `pd_api.h`.
- `src/pd/` (the `sdk` feature, the only `unsafe`): the `extern "C"`
  side, a `GlobalAlloc` over the SDK's `realloc` (padding for alignments
  above 8), a `#[panic_handler]` that shows the message with the SDK's
  `error` without allocating, `rust_eh_personality` for the Simulator,
  and the game's own stack: 64 KB from the heap, painted, entered with
  Thumb assembly by `game_stack::run`; `game_stack::high_water()` reads
  the mark; a call that reaches the bottom 256 bytes stops the game with
  an error. On the Simulator `run` is a plain call (the PC's stack) and
  nothing is measured.
- What runs: "Loading…" (the system font has the `…`), then
  `trpg_content::load_embedded()` on the game's stack, then the load time
  and the stack mark on screen, in the log, and in `load-report.txt` in
  the game's data folder (for reading a Simulator run without its
  console). In debug builds the mark is logged once a second when it
  changed.
- The plain parts (`heap`, `stack`, `text`, `report`) build without the
  SDK and are tested on the host; `cargo test/clippy --workspace` never
  need the SDK.
- `xtask playdate` is a list of steps (`plan`) run by `execute`: the
  tests check every command, flag and path without the SDK. Workspace
  profiles `playdate` (`--release`: `panic = "abort"`, fat LTO, one
  codegen unit, line tables kept in `pdex.elf`) and `playdate-dev`.
- CI job `playdate` (Linux): `gcc-arm-none-eabi`, the SDK's Linux tarball
  from Panic (version 3.1.2 pinned in the job's env, cached by version),
  clippy with the SDK glue, `cargo xtask playdate --release --device`, and
  checks for `pdex.elf`, `pdex.bin` and `pdxinfo`. In `ci-result`.
- Measured on Nick's machine: `pdex.bin` 1.49 MB; nothing linked from
  newlib or libgcc. Simulator: data loaded in 6 ms (PC speed; the stack is
  measured on the device only).

**Deviations.**

- The crate is a `staticlib` only, not also an `rlib`: Cargo skips LTO
  for a library that is both, and without LTO the device link pulled in
  libgcc's unwinder and newlib's `abort`. Nothing needs the `rlib`.
- An unoptimized device build keeps ARM unwind tables that name the
  unwinder's personality routines, so `src/pd/panic.rs` defines empty
  `__aeabi_unwind_cpp_pr0`/`pr1`/`pr2` (never called: nothing unwinds).
- The Playdate SDK is listed in a new *Platform SDKs (not shipped)*
  section of `THIRD_PARTY_ASSETS.md`, not as a table row: every row needs
  a credit whose license is one of ADR-0013's, and nothing of the SDK
  ships. `setup.c` has a row, a hidden credit and its license text
  (`crates/playdate/setup.c-LICENSE.md`).
- The SDK glue (`crates/playdate/src/pd/**`) is excluded from mutation
  testing: only the `sdk` feature compiles it, so every mutant there
  would count as missed. CI's `playdate` job builds it.
- The Simulator's `pdex.dll` is stripped (14 MB to 3 MB), because the
  `.pdx` carries it to the device too.
- `run-gates` gained the Playdate clippy line; the lint-drift test now
  covers both platform crates.

**For Nick.** On the Playdate the game's name is "Visions of Shuyi",
its author "Nick Wentz" and its bundle ID `com.zafnok.visionsofshuyi`
(its data folder on the device; it should not change once a build is
out). Say if you want other names. No gameplay rules were decided.

**Follow-ups.** None new: 1032 (renderer), 1033 (the game runs), 1036
(loading card, less data) are already ticketed.
