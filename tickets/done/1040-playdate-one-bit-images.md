---
id: "1040"
title: "Playdate: one-bit images made at build time and drawn as sprite items"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1032", "1036", "0062"]
nick_input: none
completed: 2026-10-17
---

# 1040 — One-bit images for the Playdate

## Context

The Playdate draws pictures as one-bit sprite items (ADR-0038: a picture
is a sprite item from an asset file; 1032's ADR says how the one-bit
renderer draws them). Today's pictures are colour PNGs decoded by `app`
with macroquad: title backgrounds and logo (`assets/title/`), battle
backgrounds (`assets/battlebacks/`), combat pictures
(`assets/battlers/`, real ones bought in `assets-private/game/battlers/`),
portraits (`assets/portraits/`, bought faces in the private
repository). The device should not decode PNGs (ADR-0070: the device is
~700× slower than the PC).

0062 decides how they look (the dither style per kind of picture). This
ticket builds the converter and the format; 1041, 1042 and 1044 use it
for each kind of picture.

## Nick input

None. The look is 0062's; this ticket applies it.

## Scope

**In:**
- **A one-bit image format** (pixels and a transparency mask, packed
  one bit per pixel in rows; a small header with the size), and its
  reader in `trpg-content` (`no_std`), so the one-bit renderer (1032)
  draws it as a sprite item.
- **`cargo xtask playdate-images`** (also run by `xtask playdate`):
  converts a picture to one-bit with the method 0062 picked for its kind
  (a table: kind → method and settings), writing the result where the
  Playdate bundle (1036) embeds it. Pictures from `assets/` convert into
  a generated folder (e.g. `target/playdate/images/`), never committed;
  pictures from `assets-private/game/` convert into the private
  checkout's own generated folder, never into this repository (ADR-0040).
- **The methods** (`playdate.md`, *One-bit pictures*): enlarge to the
  size shown first (pixel doubling for whole sizes, a smooth resample
  for 1.25×), then the brightness stretch and a 4 × 4 Bayer dither at
  screen pixels; optionally a brightness curve, rounding to N levels,
  and the art's own dark lines drawn on top (styles A+ lines, F+ lines,
  G). **Portraits are converted once per style** (A, A+ lines, F+
  lines, G) for 1042's Options toggle. The listener's fade is drawn at
  run time, not converted.
- **Which images:** the ones the Playdate build uses (1036's list): the
  Playdate bundle names the one-bit file in place of the PNG, so screens
  and sidecars (`.ron`) keep naming the same image id.
- Builds without the private art use the placeholders in `assets/`
  (converted the same way), as the PC build does.
- Tests: the converter's methods on small test images (pinned pixels);
  the reader round-trips; the renderer draws a converted test image.
- Credits: converting bought art is a modification its licence allows
  (ADR-0032); the credits entries stay as they are.

**Out (do not do):**
- Hand touch-ups of particular pictures (1042 does the portraits'),
  and wiring each kind of picture into its screen (1041, 1042, 1044).
- Changing any PC picture.

**Carve (1104):** either order works. If 1104 moved images and the
palette to the engine, the one-bit format is an engine image format and
the converter an engine tool; the method table stays the game's.

## Implementation steps

1. Read 0062's answers in `docs/design/playdate.md`, ADR-0038, ADR-0040,
   1032's ADR, `crates/content/src/image.rs`.
2. Format, reader, converter methods with tests.
3. `xtask playdate-images` and its call from `xtask playdate`.
4. Simulator: show one converted title background through a debug scene
   or `frame-png --playdate`.

## Acceptance criteria

- [x] `cargo xtask playdate-images` converts every image the Playdate
      bundle lists; none are committed.
- [x] Tests for the converter, reader and renderer pass.
- [x] A converted placeholder picture is shown in the PR (only
      placeholders or the public title art, never bought art).
- [x] All gates in the `run-gates` skill pass, and the CI `playdate` job
      (`cargo xtask playdate --release` builds locally; CI on the PR).

## Tests required

- Unit: conversion methods, reader round-trip, sprite drawing.

## Completion notes

**What was done** (ADR-0085):

- **The format** (`shuyi_assets::bitmap`, re-exported as
  `trpg_content::bitmap`, the engine's since 1104 carved images): a
  `.1bit` file, a 16-byte header with the size and the PNG's own size,
  then one or more styles' rows and a shared mask, the Playdate's bitmap
  layout. The reader checks every field and names what is wrong.
- **The converter** (`shuyi_assets::bitmap::convert`): enlarge (pixels
  doubled for a whole scale, a smooth resample otherwise), brightness
  curve, the 3% stretch, rounding to N levels, the 4 × 4 Bayer dither at
  screen pixels, the art's dark lines on top. Pinned-pixel tests for each
  step.
- **The game's table** (`crates/content/src/playdate_images.rs`):
  portraits at 2× in styles A, A+ lines, F+ lines and G (that order is
  1042's setting); combat pictures at twice their sidecar's scale (the
  1.25× ones resampled smoothly) in A; battle and title backdrops
  lightened, in A, enlarged to the screen's 400-pixel width.
- **Into the Playdate's bundle**: `crates/content/build.rs` converts them
  and puts each one-bit file at its PNG's path in the Playdate's table
  (1036's), so screens and sidecars name the same image. The engine's
  image table reads a one-bit file's PNG size, so loaders check as
  before. The PNGs no longer reach the device.
- **The renderer** maps a sprite's PNG pixels onto the bigger picture
  (`Image::sample`, also in the reference renderer; the property tests
  draw an enlarged picture with both). The front end reads every picture
  once at start (`crates/playdate/src/pictures.rs`).
- **`cargo xtask playdate-images`** writes the same pictures to look at
  (`.1bit` and a PNG per style) into `target/playdate/images/`, and the
  bought art's into `assets-private/generated/playdate-images/` when it is
  checked out. `frame-png --playdate` now draws them.
- Shown in the PR: `docs/screenshots/1040-playdate-one-bit-placeholders.png`
  (the public placeholders only).
- Step 4 (seeing one): `cargo xtask frame-png --playdate` of the title
  draws the converted backdrop (cut off by the PC's title skin until
  1044). The Simulator was already open on Nick's machine, so it wasn't
  driven. `pdex.bin` is 2,496,466 bytes (1036: 2,487,387): the one-bit
  pictures are about the PNGs' size.

**Deviations:**

- The pictures are converted **when the Playdate's bundle is built**, by
  `build.rs`, not by `xtask playdate-images` into a folder the build then
  embeds, and `xtask playdate` doesn't run it: that way the tests (the
  Chapter 1 test on the Playdate's bundle included) read the same files
  the device gets, and no stale folder can ship. `playdate-images` makes
  the same files to look at and for 1042's work.
- The **bought art can't reach the Playdate yet**: its bundle has no
  private layer (ADR-0070 §3). New ticket **1061** adds it (and lets
  1042's touched-up files win over converting); 1041 and 1042 now wait
  for it.
- The title's logo and Game Over words are not dithered: 0062 decided
  the logo as solid white letters with a black shadow and rim, which is
  1044's to make. They stay PNGs (grey boxes on the Playdate until then).

**the agent's starting rules** (Nick can veto; all *tunable*, the screens'
tickets may change them):

- Backdrops (title and battle) are lightened with the same brightness
  curve as the line styles (0.8), and made 400 pixels wide (the screen's
  width) before they are dithered.
- A dark line is an art pixel 30% darker than the brightest pixel next to
  it, or under 15% bright; the edge of a figure counts as next to white,
  so its outline is kept.

**Follow-up ticket:** 1061 (the bought art in the Playdate's bundle).

**Notes for Nick:** nothing to play yet: the Playdate's screens still
wear the PC's skins, so the pictures show cut off until 1041, 1043 and
1044 lay them out. The placeholder backdrops are flat bands of colour,
so they come out as flat patterns; the bought ones have gradients.
