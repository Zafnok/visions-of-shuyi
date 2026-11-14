# ADR-0125: The writer's preview is a package of its own

- **Status:** Accepted
- **Date:** 2026-11-14
- **Related tickets:** 1269; 0764 (the preview, ADR-0123), 1267 (the
  release dry run that found it)

## Context

ADR-0123 §1 made the writer's script preview `trpg-app`'s second program,
`story-preview`. Shuyi Engine's `cargo shuyi export` builds the program
named as the package, else the package's only one; an edition can name its
`bin`, but `shuyi.ron`'s `export` part cannot. With two programs in
`trpg-app` and none named `trpg-app`, every export without an edition `bin`
fails (`SH1002`): the release's three desktops and the demo's Windows
build. The game never waits on the engine (ADR-0092), so an engine
`export.bin` field is not the fix here.

## Decision

1. **The preview is the package `trpg-story-preview`** (`crates/story-preview`),
   whose one program is `story-preview`. `trpg-app` has one program again,
   `visions-of-shuyi`. This amends ADR-0123 §1 ("`trpg-app`'s second
   program"); the rest of ADR-0123 stands.
2. The preview shares the game's `AppInfo` by `#[path]` to
   `crates/app/src/app_info.rs` (one file, written by `cargo shuyi
   app-info`), and has the game's `debug-tools`, `private-assets` and
   `packed` features, forwarding to the same crates. Its Windows exe gets
   the game's icon from a `build.rs` of its own.
3. `shuyi.ron`'s `story-preview` edition names the package (`package:
   "trpg-story-preview"`) beside its `bin`; its pack is
   `trpg-story-preview.pack` (the engine names a pack after the package).
4. An xtask test reads `cargo metadata`: every package an export builds
   with no `bin` named has one program, or one named as the package. A
   second program in such a package fails the gates, not a release.

## Consequences

- The release and demo exports build the game's program again, with no
  engine change.
- A new program of the game goes in a package of its own, or the export
  part that builds it names its `bin`; the xtask test says so.
- One more crate in the workspace; it builds only what the preview uses.

## Alternatives considered

- **An engine `export.bin` field** — the right engine feature perhaps, but
  the game would wait on an engine PR and a pin move; the friction review
  can ticket it.
- **Rename `trpg-app`'s game program `trpg-app`** — the shipped program
  would change its name in every package, README and store page.
- **A `--bin` passed by release.yml and demo.yml** — the export takes none;
  and every new caller would have to remember it.
