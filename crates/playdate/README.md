# `trpg-playdate`: the Playdate front end

The game on the Playdate ([ADR-0070](../../docs/adr/0070-playdate-front-end.md)).
A platform crate like `trpg-app`: it owns the SDK's update callback, the
clock, files and the allocator, and holds no game or screen logic.

So far (ticket 1031) it shows "Loading…", loads the game's data on the
game's own stack, then shows and logs how long that took and how much of
that stack it used. The game itself runs from ticket 1033.

## What is where

| Path | What |
| ---- | ---- |
| `shim/shim.c` | The SDK's entry point (`eventHandler`) and one small C wrapper per SDK call, compiled against the installed SDK's `pd_api.h` |
| `src/pd/` | The `sdk` feature: the Rust side of those calls, the allocator over the SDK's `realloc`, the panic handler, the game's own stack (`run`, `high_water`) and the entry points the shim calls. The only `unsafe` in the crate |
| `src/{heap,stack,text,report}.rs` | The plain, tested parts: allocator arithmetic, the stack's size and high-water mark, C strings, the report's text |
| `setup.c-LICENSE.md` | The license of the SDK's `setup.c`, which is linked into the game (0BSD) |

Without the `sdk` feature the crate builds on any machine, so
`cargo test --workspace` and `cargo clippy --workspace` never need the
SDK. `cargo xtask playdate` turns it on.

## The game's own stack

The Playdate gives the C stack about 10 KB; loading the game's data took
15 KB in ticket 1026. So every call from the update callback into game
code goes through `game_stack::run`, which runs it on a 64 KB block from
the heap (a few lines of Thumb assembly move the stack pointer there and
back). The block is filled with a pattern first; `game_stack::high_water`
counts how much of it was ever used, and a call that reaches the bottom
256 bytes stops the game with an error. The Simulator runs the game on
the PC's own (big) stack, so there nothing is measured.

## Install (once)

- The **Playdate SDK 3.1.2** from <https://play.date/dev/>. The Windows
  installer puts it in `Documents\PlaydateSDK`; anywhere else, set
  `PLAYDATE_SDK_PATH`.
- The **ARM compiler**, with MSYS2:
  `C:\msys64\usr\bin\pacman.exe -S mingw-w64-ucrt-x86_64-arm-none-eabi-toolchain`
  (Linux: `apt install gcc-arm-none-eabi`).
- **MinGW gcc** for the Simulator, which the game's Windows build already
  needs: `pacman -S mingw-w64-x86_64-gcc`.
- The device's Rust target: `rustup target add thumbv7em-none-eabihf`.

The compilers are found on `PATH`, else in MSYS2's default folders
(`C:\msys64\ucrt64\bin`, `C:\msys64\mingw64\bin`). No MSVC, no nightly,
no CMake.

## Build

```bash
cargo xtask playdate --release
```

builds `target/playdate/VisionsOfShuyi.pdx` for both the device and the
Simulator. `--device` or `--simulator` builds one half (the Simulator half
on Windows only). Without `--release` the Rust code is built unoptimized
(`playdate-dev` profile): fine in the Simulator, too slow and too deep for
the device's stack, so always use `--release` for the device.

## Run it

- **Simulator:** open `target/playdate/VisionsOfShuyi.pdx` with
  `PlaydateSimulator.exe` (in the SDK's `bin`), or drag it onto the
  Simulator. Its timings are the PC's, never the device's.
- **On a Playdate:** plug it in, unlock it, and in the Simulator with the
  game open choose *Device → Upload Game to Device*.
- **Logs:** the Simulator's console (the `>_` button under the screen)
  shows the game's log, and a plugged-in device's too. The report is also written to
  `load-report.txt` in the game's data folder: the SDK's
  `Disk/Data/com.zafnok.visionsofshuyi/` for the Simulator.
