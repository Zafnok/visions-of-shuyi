# ADR-0120: Release builds read `assets/` from a pack beside the program

- **Status:** Accepted (1265)
- **Date:** 2026-11-13
- **Related tickets:** 1265; engine 3300, 3304 (engine ADR-0110); 0910, 1208
  (the release export and its disclosure gate); 0914 (the demo edition)
- **Extends:** ADR-0005 (the bundle is embedded), ADR-0117 (the content laid
  over `assets/`), ADR-0119 (the demo's cut bundle); follows engine ADR-0110

## Context

Every build embeds each file of `assets/`, and of the private repository's
`content/` laid over it, in the program as written (ADR-0005, ADR-0117):
`trpg-content`'s `build.rs` writes tables of them, and `bundle::layers`
lays them out with the bought art over them and, in a debug build, the test
files under them (ADR-0057). The demo edition embeds its own cut table in
their place (ADR-0119).

Engine ADR-0110 gave a game's `packed` feature a meaning: `cargo shuyi
export <target> --release` on the desktops and the web turns it on, bakes
the game's files with its tools (`GameTools::bake`) into `<package>.pack`,
checks the pack against its sources (`bake --check`), searches it for the
build machine's paths, ships it beside the program, and lists every file of
it in the exposure report, held to `disclosure.allowed_pack_entries`.
`shuyi_app::pack_beside_program(name, stamp)` reads it.

The question the ticket asks is whether the game's layers fit one pack:

- **`assets/` and the content** are one view: the content replaces
  `assets/`'s file at a path, and nothing at run time asks which of the two a
  file came from (`bundle::is_private` names the bought art only). Baked
  through the bundle they are, they are one layer of files.
- **The bought art** (`private-assets`) is a compile-time choice of the
  program's build, and the export's tool program (xtask, which bakes) is
  built without it. The files are bought and never handed out raw (ADR-0040):
  a pack beside the program is one `unpack` from a folder of PNGs.
- **The test files** are never in a build that ships (ADR-0057).
- **The demo edition** bakes through its content root: the engine runs the
  tool program again with `SHUYI_CONTENT` (engine ADR-0101), which builds it
  with the `demo_edition` cfg and so with the demo's table.
- **The Playdate** bakes its own files (ADR-0070) and is out of scope.

So the engine's pack holds what the game needs; nothing in the engine is
missing.

## Decision

1. **A `packed` feature** on `trpg-app`, which turns on `trpg-content`'s.
   Built with it, the content crate embeds no table of `assets/`, the
   content or the demo's cut (`ASSETS`, `CONTENT`, `DEMO`), and reads that
   layer from the pack the program hands it (`bundle::use_pack`) under
   `assets/` (`DISPLAY_ROOT`), before anything reads a file. The bought art
   stays embedded over it with `private-assets`. Every other build (debug,
   tests, tools, the Pages web build, the Playdate) embeds as before.
2. **What is baked:** `bundle::pack_layers()`: the content over `assets/`,
   or in the demo edition the demo's table, without the bought art or the
   test files. Each file goes in as its own bytes (no kind is baked into the
   pack's binary form): the loaders read the same bytes either way, so the
   game plays the same, and baking a kind can come later without changing
   this.
3. **The stamp** is `bundle::PACK_STAMP`, `visions-of-shuyi <version>`: a
   pack of another version is refused and the player told to bake it again.
4. **The tools:** xtask's `Trpg::bake` gives `pack_layers` with that stamp
   for every target but the Playdate's, so `cargo shuyi bake` and `bake
   --check` work on the game.
5. **The program** (`crates/app/src/main.rs`) reads the pack with
   `pack_beside_program(CARGO_PKG_NAME, PACK_STAMP)` before the start. A
   pack that is missing, of another version or broken is the start's error,
   which the game shows in place of its title.
6. **The name** is the engine's, the package's: `trpg-app.pack`, beside
   `visions-of-shuyi.exe` (in `Contents/Resources` in the `.app`, beside
   `index.html` on the web).
7. **The disclosure parts** list `trpg-app.pack` beside the program and every
   file of the pack (`allowed_pack_entries`) for the full game and the demo;
   an xtask test holds the lists to `pack_layers` and the demo's bundle, as
   the voice clips' list is held to `voice/`. A file of the content is named
   there by its path, which a public file may do (ADR-0118: ids and file
   names, never a line).
8. **The package check** (`scripts/check-release-package.sh`) expects the
   pack in every desktop and web package and refuses a program that holds
   the first line of `assets/data/keymap.ron`.

## Consequences

- A release program carries no text of `assets/` and no path of it; the
  pack is beside it. A player who copies only the program sees why the game
  can't start.
- A new file of `assets/` or of the content needs its line in `shuyi.ron`
  (twice if the demo keeps it); the xtask test says which.
- A debug build, `cargo run`, the tests and the tools read the embedded
  files as before, with hot reload.
- The bought art is still inside the program: the release program shrinks
  by the size of `assets/`, not of the art.
- Building the workspace with `trpg-app/packed` would turn `packed` on in
  xtask's `trpg-content` too, and xtask could bake nothing; the export builds
  `-p trpg-app` alone, as ADR-0057 already asks of a build that ships.

## Alternatives considered

- **Bake the bought art into the pack too.** The program would lose the art's
  size, but the tool program would need the art feature to match the
  program's, and the art would ship as files anyone can list and copy.
- **Bake the RON files into the pack's binary forms** (engine `Packed`
  formats). Faster to load, but the game's loaders read RON text through
  their own parsers; each would need a format registered and its check.
  Raw bytes first; a later ticket can bake a kind when start time asks.
- **One pack per layer.** The engine reads one pack beside the program, and
  no run-time code tells `assets/` from the content.
- **Rename the package so the pack is `visions-of-shuyi.pack`.** Renames
  `trpg-app` across the workspace, CI and the docs for a file name only
  players see beside the program.
