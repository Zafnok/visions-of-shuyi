# `trpg-playdate`: the Playdate front end

The game on the Playdate ([ADR-0070](../../docs/adr/0070-playdate-front-end.md)).
A platform crate like `trpg-app`: it owns the SDK's update callback, the
clock, files and the allocator, and holds no game or screen logic.

It shows "Loading…", loads the game on the game's own stack and runs it
(ticket 1033). The screens still wear the PC's skins, drawn into a
50 × 15 console, so most are cut off until their Playdate skins land
(1037 on).

## What is where

| Path | What |
| ---- | ---- |
| `shim/shim.c` | The SDK's entry point (`eventHandler`) and one small C wrapper per SDK call, compiled against the installed SDK's `pd_api.h` |
| `src/pd/` | The `sdk` feature: the Rust side of those calls, the allocator over the SDK's `realloc`, the panic handler, the game's own stack (`run`, `high_water`), the SDK's files behind `storage::Files` (`files.rs`) and the entry points the shim calls (`entry.rs`: the frame loop). The only `unsafe` in the crate |
| `src/pads.rs` | The buttons: the SDK's button bits as a `PadKind::Playdate` controller. The only file here that names buttons (`check-keys`) |
| `src/storage.rs` | The game's `Storage` on the Playdate's files |
| `src/screen.rs` | Copying the one-bit frame onto the SDK's frame buffer, changed rows only |
| `src/{heap,stack,text,report}.rs` | The plain, tested parts: allocator arithmetic, the stack's size and high-water mark, C strings, the logged reports |
| `setup.c-LICENSE.md` | The license of the SDK's `setup.c`, which is linked into the game (0BSD) |

Without the `sdk` feature the crate builds on any machine, so
`cargo test --workspace` and `cargo clippy --workspace` never need the
SDK. `cargo xtask playdate` turns it on.

## The frame loop

Each SDK update (`src/pd/entry.rs`):

1. `dt` is the SDK's elapsed-time timer since the last update (it starts
   after loading, so the first frame does not count the load).
2. The buttons become this frame's `RawInputEvent`s (`src/pads.rs`). A
   button pressed and let go between two updates counts as held for one
   frame, so a quick tap is never lost while a frame takes long.
3. `Game::frame(&events, dt)` runs on the game's own stack.
4. Only when the frame's `GlyphBuffer` differs from the last one drawn,
   `trpg_ui::onebit` renders it and the rows that changed are copied into
   the SDK's frame buffer (`getFrame`, `markUpdatedRows`).

**Frame rate: 30.** `setRefreshRate(30)`, the SDK's own default, to
start: the game's animations are timed by `dt`, not by frames, and on the
device a frame that renders costs the full 400 × 240 one-bit draw. The
device numbers (below) say whether to change it.

The game starts with the Playdate's console (`ConsoleSpec::PLAYDATE`) and
palette, `Keyboard::Absent` (no layout picker) and the Playdate pad as the
device help text names; tips, lessons and idle motion on, as on the PC.
The debug tools are off: they are made of the test files, which a build
without `std` does not have (ADR-0057). Sound is ignored until 1047; a
game that quits keeps its last frame (the system's Menu button leaves
it). Choosing a colour theme in Options would swap in that theme's PC
palette until 1045 takes the theme out of the Playdate's Options.

## Buttons

The D-pad, A and B are one controller of `PadKind::Playdate`, through the
player's pad bindings like any controller (ADR-0034): A reports as the
binding position `South` and B as `East`, so the default `pad` table of
`assets/data/keymap.ron` makes the D-pad the cursor, A Confirm and B
Cancel. Help text names them `A`, `B` and the D-pad. Chords and the crank
come with 1034 and 1035.

## Storage

The game's `Storage` is one `<key>.ron` file per key in the game's data
folder (`Disk/Data/com.zafnok.visionsofshuyi/` in the Simulator), written
to `<key>.ron.tmp` first and then moved over the old file. Profiles and
saves are the PC's (ADR-0069). The debug build runs `trpg-ui`'s storage
conformance suite on the real files at start, in the `storage-check`
folder (left empty), and logs `storage: the conformance suite passed`; a
failure stops the game with the error.

## What it measures

Once a minute the front end logs the screen on top, the frames a second,
`Game::frame`'s and the render-and-copy's min / avg / max milliseconds,
and the game stack's high-water mark (device only), in every build: the
device needs the release build. On start it logs, and writes to
`load-report.txt`, how long loading the game took.

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
  Simulator. The arrow keys are the D-pad, the `S` key is A and the `A`
  key is B. Its timings are the PC's, never the device's.
- **On a Playdate:** plug it in, unlock it, and in the Simulator with the
  game open choose *Device → Upload Game to Device*.
- **Logs:** the Simulator's console (the `>_` button under the screen)
  shows the game's log, and a plugged-in device's too. The load report
  is also written to `load-report.txt` in the game's data folder: the
  SDK's `Disk/Data/com.zafnok.visionsofshuyi/` for the Simulator.
