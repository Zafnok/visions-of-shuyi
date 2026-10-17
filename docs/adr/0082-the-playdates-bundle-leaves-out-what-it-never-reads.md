# ADR-0082: The Playdate's bundle leaves out what it never reads, and says so

- **Status:** Accepted
- **Date:** 2026-10-17
- **Related tickets:** 1036, 1103, 1047, 1049
- **Builds on:** ADR-0070 §5 (load only what the Playdate uses, behind a
  loading card, before any binary format), ADR-0005 (the embedded
  bundle) and 1103's bundle as layers.

## Context

The Playdate build embeds `assets/` as one table that `trpg-content`'s
`build.rs` writes, because `include_dir` needs `std` (ADR-0070 §3). Until
1036 that table held every file: the notes for whoever edits the data
(`README.md` files, license texts), the PC's window icons and the sound
effects, which the Playdate does not play until 1047 (and will likely
play in another format). They cost about 360 KB of `pdex.bin` and RAM.

Two things make leaving a file out more than deleting a table row:

- Loaders check what they read. The audio manifest's check reports a
  sound file that is not in the bundle as missing, so a smaller bundle
  must be able to say "left out on purpose" apart from "missing".
- Nothing tells us what the Playdate reads. The list must come from the
  game itself and stay right as `assets/` grows.

The ticket also asks for the slowest files on the device, which needs a
time for every file the loader reads, without a clock in `content`
(ADR-0004).

## Decision

1. **One list, of what is left out.** `crates/content/playdate-bundle.txt`
   names what the Playdate leaves out of `assets/`: a file, a folder
   (`audio/sfx/`) or every file of a name (`**/README.md`), with `#`
   comments. `build.rs` writes the Playdate's table (`playdate_files.rs`)
   and the paths it left out (`playdate_left_out.rs`); a line that
   matches no file stops the build. It is a list of what is *left out*,
   so a new data file reaches the Playdate by default.
2. **The layer says what it left out.** The engine's `Layer` gains
   `leaving_out(paths)` and `Bundle::left_out(path)`. Without `std` the
   game's one layer is the Playdate's (`bundle::playdate_layers`), with its
   list. A loader that would report a left-out file missing does without
   it instead: the audio check (`audio::Files::sound_left_out`) skips the
   sound files' format check, keeping the manifest's own checks. The PC's
   and the web's layers leave nothing out, so they check as before.
3. **The test is the list's proof.** `crates/playdate/src/chapter_1.rs`
   plays Chapter 1 through the front end's own start (`start.rs`,
   `card.rs`) with its thread reading the Playdate's bundle (the engine's
   `reading`, `std` only) and a watcher on every read (the engine's
   `watch`). It fails if the game reads a file the list left out, or if
   the bundle holds a file nothing read. The `playdate-bundle` feature of
   `trpg-content` builds the Playdate's layers with `std` for it; no PC
   or web build turns it on.
4. **The device times every read.** The front end installs a watcher
   (`bundle::watch`, set once) that notes each read with the SDK's clock
   while loading. A file's time is from its read to the next; the start's
   steps that read nothing are marked by name. The log names the ten
   slowest; `load-report.txt` holds all.

## Consequences

- The Playdate's `pdex.bin` went from 2,847,117 to 2,487,387 bytes. The
  load time barely changes: what is left out is not parsed (1026 found
  the parsing is the time), so ADR-0070 §5's fallback (a binary format at
  build time) is decided by the device numbers this measures.
- Adding a file to `assets/` that the Playdate never reads fails the
  Chapter 1 test until it is listed. That is the point, but it is a
  second place to touch.
- The full-colour pictures, the PC's colour themes and the PC font's PNG
  stay in the Playdate's bundle: their loaders check them against other
  data (portraits, battle art, Options' themes), and leaving them out
  needs those loaders to do without, which 1040 (one-bit pictures) and
  1045 (the Playdate's Options) change anyway. The test lists them as
  read, so it will say when they stop being read.
- The engine's bundle gains three small, general pieces: a layer that
  leaves files out, a watcher of reads, and a thread reading another
  bundle. A watcher costs one atomic load per read when none is set.
- 1049 (one build per language) adds its languages' packs to the list.

## Alternatives considered

- **A list of what the Playdate keeps.** Every new data file would be
  missing on the Playdate until someone listed it, and the game would
  fail to load there only. The test already catches files nothing reads.
- **Make the loaders tolerate any missing file.** The PC's checks would
  stop catching a deleted file; "left out" has to be the build's own
  statement.
- **A cargo feature per left-out group** (`no-sounds`, `no-icons`).
  More code paths in `content` for what is a list of paths.
- **Log the lookups in the Simulator by hand and keep the list from that
  run.** The ticket suggested it; the same Rust code runs in the test, on
  every PR, so the list cannot drift.
- **Time each loader inside `content`.** `content` has no clock
  (ADR-0004); the watcher gives the front end the times without one.
