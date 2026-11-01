# ADR-0105: No build embeds a README; every bundle layer is a table `build.rs` writes

- **Status:** Accepted
- **Date:** 2026-11-01
- **Related tickets:** 0123
- **Extends:** ADR-0005, ADR-0040, ADR-0057 (how their directories are
  embedded), ADR-0070 §3 (the table layer, until now only without `std`)

## Context

`include_dir!` embedded each asset directory whole, with the `README.md` of
every folder: about 67 KB of notes on tickets, tools and test files in the
sold `.exe`, which no game code reads. `include_dir!` has no filter, and a
filter on the bundle's reads would leave the bytes in the binary.

`crates/content/build.rs` already wrote a table of `include_bytes!` lines for
`assets/`, for the Playdate, and a test kept it equal to `include_dir!`'s
files. `shuyi_assets::bundle` reads a table layer exactly as it reads a
directory.

## Decision

1. **Every layer is a table.** `build.rs` writes one for `assets/`,
   `assets-test/`, `assets-private/game/` and `assets-private/test/`;
   `bundle.rs` includes each under the same `cfg` as before. `include_dir` is
   no longer a dependency of the game's crates.
2. **`build.rs`'s `walk` skips every file named `README.md`**, so the rule
   holds for all tables, the Playdate's too. `playdate-bundle.txt` loses its
   `**/README.md` line, and its `**/` form, which nothing else used.
3. **The READMEs stay beside the files they describe.** A test that runs a
   README's examples reads it with `include_str!` from the repository.
4. **A test compares** the `assets/` and `assets-test/` tables with the
   folders on disk, file for file and byte for byte, but the READMEs.

The merging of layers stays in the engine's tested `Bundle`; the build script
only lists one folder at a time, so ADR-0040's reason against a merging build
script does not apply.

## Consequences

- The sold game holds only files that aren't notes, and nothing is copied at
  build time.
- With `private-assets`, a missing `assets-private/test/` now gives an empty
  table where it used to stop a debug build from compiling; the test files'
  placeholders are then read instead.
- A file in an asset folder that isn't called exactly `README.md` is still
  embedded: notes go in that name.
- A rule in a build script is out of mutation testing's reach; the test of
  decision 4 is what guards it.

## Alternatives considered

- **Skipping READMEs in the bundle's reads and listings:** an engine change
  that leaves the bytes in the binary, which is the problem.
- **Copying the filtered folders to `OUT_DIR` for `include_dir!`:** works, but
  copies the assets on every build and keeps a second way of embedding beside
  the tables.
- **Moving the READMEs to `docs/assets/`:** out of the ticket's scope, and the
  notes are most useful beside the files.
