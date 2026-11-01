# `trpg-playdate`: the Playdate front end

The game on the Playdate ([ADR-0070](../../docs/adr/0070-playdate-front-end.md)).
A platform crate like the engine's `shuyi-app`: it owns the SDK's update callback, the
clock, files and the allocator, and holds no game or screen logic.

From its first frame it shows the loading card, the game's name centred
in the game's font (ticket 1036, `src/card.rs`); then it loads the game
on the game's own stack and runs it (ticket 1033). The screens are drawn
into a 50 × 15 console, each in its Playdate skin where it has one (the
map, 1037; New Game, Options, Key bindings and Profile, 1045); the others
still wear the PC's skins and are cut off until theirs land (1038 on).

Its pictures are one-bit ones (ticket 1040, ADR-0085): `trpg-content`'s
`build.rs` converts the portraits, combat pictures and backdrops by the
game's table (`crates/content/src/playdate_images.rs`) and puts each at
its PNG's path in the Playdate's bundle; `src/pictures.rs` reads them
once at start for the renderer. A picture still a PNG (the title's logo,
the map's) is drawn as a grey box. `cargo xtask playdate-images` writes
the same pictures as files to look at, into `target/playdate/images/`.

## The Playdate's bundle

The game's data is `assets/` without what the Playdate never reads:
the notes (`README.md`, license texts), the PC's icons and the sound
files, which the Playdate plays from its own converted copies in the
`.pdx` (see *Sound and music*). `crates/content/playdate-bundle.txt` lists what is left
out; `trpg-content`'s `build.rs` embeds the rest in the build without
`std`, and the bundle knows what it left out, so the audio manifest's
sounds go unchecked instead of missing (ticket 1036). `src/chapter_1.rs`
plays Chapter 1 through `src/start.rs` against that bundle: every file
the game reads must be in it, and every file in it must be read. A new
file under `assets/` that the Playdate never reads fails it until it is
listed.

## What is where

| Path | What |
| ---- | ---- |
| `shim/shim.c` | The SDK's entry point (`eventHandler`) and one small C wrapper per SDK call, compiled against the installed SDK's `pd_api.h` |
| `src/pd/` | The `sdk` feature: the Rust side of those calls, the allocator over the SDK's `realloc`, the panic handler, the game's own stack (`run`, `high_water`), the SDK's files behind `storage::Files` (`files.rs`) and the entry points the shim calls (`entry.rs`: the frame loop); with the `bench` feature, the bench build's extra renders (`bench.rs`). The only `unsafe` in the crate |
| `src/audio.rs` | Playing the frame's sounds and music (ticket 1047, ADR-0084): the PC player's rules over a `Speaker` trait, which `src/pd/sound.rs` puts on the SDK's sample and file players |
| `src/pads.rs` | The buttons: the SDK's button bits as a `PadKind::Playdate` controller. The only file here that names buttons (`check-keys`) |
| `src/menu.rs` | The game's entries in the system menu: the choice the SDK calls back with, kept until the next update, and their names |
| `src/storage.rs` | The game's `Storage` on the Playdate's files |
| `src/screen.rs` | Copying the one-bit frame onto the SDK's frame buffer, changed rows only |
| `src/card.rs` | The loading card, and the game's font read before the rest of the data |
| `src/pictures.rs` | The one-bit pictures the renderer draws, read once from the bundle |
| `src/start.rs` | The game's context as the Playdate starts it: console, palette, buttons, no keyboard |
| `src/chapter_1.rs` | The test that plays Chapter 1 against the Playdate's bundle |
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
   frame, so a quick tap is never lost while a frame takes long. Then the
   crank's turn since the last update (`getCrankChange`, `isCrankDocked`)
   becomes its notches' presses (`RawInputEvent::from_crank`, ADR-0086).
3. An entry of the system menu chosen since the last update goes to the
   game (`Game::act` with its action), then `Game::frame(&events,
   dt)` runs, both on the game's own stack, told first where the music is
   (the file player's position), and its sounds and music commands are
   played (`src/audio.rs`). The "Game::frame" time counts both.
4. Only when the frame's `GlyphBuffer` differs from the last one drawn,
   `trpg_ui::onebit` redraws the parts that differ (`render_changed`,
   ADR-0077; the first frame whole) and the rows that changed are copied
   into the SDK's frame buffer (`getFrame`, `markUpdatedRows`).

**Frame rate: 30.** `setRefreshRate(30)`, the SDK's own default, to
start: the game's animations are timed by `dt`, not by frames. The
device numbers (below) say whether to change it.

The game starts with the Playdate's console (`ConsoleSpec::PLAYDATE`) and
palette, `Keyboard::Absent` (no layout picker) and the Playdate pad as the
device help text names; tips, lessons and idle motion on, as on the PC.
The debug tools are off: they are made of the test files, which a build
without `std` does not have (ADR-0057). A game that quits keeps its
last frame (the system's Menu button leaves it), its music stopped.
Its Options have no colour theme, window, map look or layout to choose,
and its Key bindings show the Playdate's buttons alone (ticket 1045,
ADR-0091).

## Buttons

The D-pad, A and B are one controller of `PadKind::Playdate`, through the
player's pad bindings like any controller (ADR-0034): A reports as the
binding position `East` and B as `West`, as the engine's profile has
them, so `keymap.ron`'s `playdate` table (since 1034, with A + B and the
B chords) makes the D-pad the cursor, A Confirm and B Cancel. Help text
names them `A`, `B` and the D-pad.

**The crank** (ticket 1035, ADR-0086) is two more buttons of that pad: a
notch wound back (`notch_degrees` in `keymap.ron`'s `crank`, 30) presses
and lets go of `CrankBack`, a notch forwards `CrankForward`; the
`playdate` table binds them to stepping the rewind. A docked crank does
nothing.

## The system menu

The game adds three entries to the system menu (`src/menu.rs`): Danger
Zone, Dialogue Log and Options, named in the player's language (renamed
when it changes). Choosing one hands the screen on top the same action
as in the game (ADR-0086); where that screen has nothing to do with it,
nothing happens.

## Storage

The game's `Storage` is one `<key>.ron` file per key in the game's data
folder (`Disk/Data/com.zafnok.visionsofshuyi/` in the Simulator), written
to `<key>.ron.tmp` first and then moved over the old file, with the old
value kept in `<key>.ron.bak`, so a save survives a power cut (the engine's
`power_cut_suite`; `src/storage.rs` says what it promises). Profiles and
saves are the PC's (ADR-0069). The debug build runs `trpg-ui`'s storage
conformance suite on the real files at start, in the `storage-check`
folder (left empty), and logs `storage: the conformance suite passed`; a
failure stops the game with the error.

## What it measures

Once a minute the front end logs the screen on top, the frames a second,
`Game::frame`'s and the render-and-copy's min / avg / max milliseconds,
and the game stack's high-water mark (device only), in every build: the
device needs the release build.

On start it logs when the loading card was shown, how long loading the
game took, how many files it read, the ten slowest and when the game's
first frame (the title) was shown, all in milliseconds after the game
started:

```
Loading card shown … ms after start
Game data loaded in … ms
Game stack used: … of 65,536 bytes
Bundle: … reads of … files, 0 not in the bundle
Slowest files, ms to the next read: dialogue/ch01.dlg …, (the title) …, …
Sounds: 21 files loaded
First frame of the game shown … ms after start
```

A file's time runs from its read to the next one: the parsing and
checking of what it holds, roughly (a loader may read several files
before parsing them). The steps that read no file are named in brackets:
`(the player's settings and saves)`, `(the sounds)`, `(the title)`. `load-report.txt`
holds the same lines and every file read with its time; the debug build
also logs each, and each file read once the game runs.

**The bench build** (`cargo xtask playdate --release --bench`, ticket
1055) packs `target/playdate/VisionsOfShuyi-bench.pdx`: after each frame
drawn it also renders the frame whole with the fast one-bit renderer and
with 1032's (`onebit::reference`), times every phase of both (ground,
backdrop, backgrounds, under items, edges, glyphs, over items, invert)
and checks all three frames are the same. Its minute report adds:

```
Bench, 1032's render: min … / avg … / max … ms, n = …
Bench, 1032's render by phase, avg / max ms: ground … / …, backdrop …
Bench, fast render: min … / avg … / max … ms, n = …
Bench, fast render by phase, avg / max ms: ground … / …, backdrop …
Bench: 0 frames unlike 1032's render, 0 redrawn unlike a whole render
Bench: the music played this minute
```

The extra renders slow the game down, so it is for measuring only; its
"Render and copy" line still times only what the normal build does. It
pauses the music every other minute (ticket 1047): the fast render's
time in a minute with music and in one without is what playing music
costs.

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
the device's stack, so always use `--release` for the device. The debug
`pdex.elf` has its unwind index sections (`.ARM.exidx.*`) taken out after
linking (`arm-none-eabi-objcopy`, beside `arm-none-eabi-gcc`): unoptimized,
there is one per function, more sections than an ELF file can number
plainly, and the SDK's `pdc` (3.1.2) crashes on it (ticket 1056).

`--private-assets` builds with the bought art in `assets-private/game/`
over the placeholders (fetch it with `cargo shuyi private-assets`;
ADR-0040, ADR-0090): its portraits, fighters and backdrops converted to
one-bit, or a touched-up `<stem>.1bit` beside a PNG taken as it is.

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

## Sound and music

(ticket 1047, [ADR-0084](../../docs/adr/0084-playdate-audio-formats.md))

`cargo xtask playdate` has the engine's bake
(`shuyi_cli::export::playdate::audio`, ticket 1203) convert every sound
file of `assets/audio/audio.ron` and the music listed in
`crates/content/playdate-music.txt` (Chapter 1's) to mono IMA ADPCM WAVs
at 44.1 kHz in `Source/audio/` and `Source/music/`; `pdc` makes them
`.pda`s. Nothing is committed: the converted files are cached in
`target/playdate/audio-v<n>/` under their source's checksum. A new track for the Playdate is a line in
`playdate-music.txt`; the Chapter 1 test fails while Chapter 1 asks for
a track the list leaves out.

The sounds are loaded into memory behind the loading card and play on 8
sample players; each track streams from its file in a file player, at
most two at once during a fade. Volumes are the cue's times the player's
Sound or Music option, as on the PC. The music clock (the title
cinematic's) is the file player's position. Once a minute the log says
which track plays and whether it ran out of data:

```
Music: battle_bright, no gaps
```
