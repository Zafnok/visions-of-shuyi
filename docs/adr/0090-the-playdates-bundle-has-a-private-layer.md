# ADR-0090: The Playdate's bundle has a private layer, and a touched-up `.1bit` beside a PNG wins

- **Status:** Accepted
- **Date:** 2026-10-18
- **Related tickets:** 1061 (1040, 1041, 1042)

## Context

The Playdate's bundle is a table of files that `crates/content/build.rs`
writes, because `include_dir` does not build without `std` (ADR-0070
§3). That table was made from `assets/` only, so "the test files and
private layer stay `std`-only" and a Playdate build only ever showed the
public placeholders. ADR-0085 made the Playdate's pictures one-bit when
the build is built and noted that bought art needs a private layer.
1041 and 1042 need the bought faces, fighters and backdrops on the
Playdate to be checked, and 1042 touches some up by hand.

## Decision

1. **A private layer in the Playdate's bundle**, with the
   `private-assets` feature (ADR-0040) only. `build.rs` writes
   `playdate_private_files.rs`, a table of `assets-private/game/` made as
   the public one is: the rules of `playdate-bundle.txt` that match a
   file leave it out (a rule that matches no bought file is no error),
   and every picture of a kind in `playdate_images.rs` is converted into
   `OUT_DIR/onebit-private/`. A fighter's scale is its sidecar's in the
   private folder, else the one in `assets/`, as the layered bundle
   reads it. Without the feature the table is empty and never included.
2. `bundle::playdate_layers()` puts `Layer::table(PRIVATE_DISPLAY_ROOT,
   PLAYDATE_PRIVATE).private()` over the public layer, as `layers()`
   does on the PC. Without `std`, `layers()` is `playdate_layers()`. The
   test files stay `std`-only.
3. **A touched-up picture wins.** A `<stem>.1bit` file beside a PNG of a
   one-bit kind in `assets-private/game/` is embedded as that PNG's
   picture as it is, instead of converting the PNG, and the `.1bit` file
   is not in the table at its own path (`playdate_images::touch_ups`).
   `cargo xtask playdate-images` copies it to its output instead of
   converting, and draws only the styles it holds, so what it shows is
   what ships.
4. `trpg-playdate` has a `private-assets` feature forwarding to
   `trpg-content`'s; `cargo xtask playdate --private-assets` turns it on
   (any build, the bench build too), into the same `.pdx` name.

## Consequences

- The Simulator and the device can show the bought art; 1041 and 1042
  can be signed off on it.
- No gate reads `assets-private/` (ADR-0040): the gates build without
  the feature, and the touch-up rule is tested on lists of paths and on
  a scratch folder.
- The bought PNGs the Playdate keeps as PNGs (the map's tiles and
  sprites) go into the private layer too, as on the PC, until a ticket
  lists them in `playdate-bundle.txt` or gives them a one-bit kind.
- Touch-ups only work in the private layer; a public picture is always
  converted from its PNG.

## Alternatives considered

- **Embed `assets-private/generated/playdate-images/`** (what
  `playdate-images` writes). A build would depend on a tool run before
  it, and a stale folder would ship stale pictures (ADR-0085 turned this
  down for the public pictures too).
- **Touch-ups as a separate folder** (`assets-private/game-playdate/`).
  A second tree to keep in step with the PNGs it replaces; beside the PNG,
  the touch-up and its source are seen together and the credit's path
  covers both.
