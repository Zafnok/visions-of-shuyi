---
id: "1040"
title: "Playdate: one-bit images made at build time and drawn as sprite items"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1032", "1036", "0062"]
nick_input: none
completed:
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

- [ ] `cargo xtask playdate-images` converts every image the Playdate
      bundle lists; none are committed.
- [ ] Tests for the converter, reader and renderer pass.
- [ ] A converted placeholder picture is shown in the PR (only
      placeholders or the public title art, never bought art).
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: conversion methods, reader round-trip, sprite drawing.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
