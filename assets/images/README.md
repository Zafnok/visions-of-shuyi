# Images (`.png`)

Pictures the game draws as **sprite items** (ADR-0038): a screen puts a
`Sprite` in the frame that names an image by its path here, and `app` draws
it from a texture. No picture is ever drawn as per-pixel cells or
rectangles.

Every `*.png` under `assets/` (in any directory, the font atlas aside) is in
the image table, `Content::images` (`trpg_content::image`), under its path
relative to `assets/`, e.g. `images/effect_marks.png`. The all-assets test
(ADR-0005) fails on a file named `.png` that isn't a PNG, an empty image, or
one with a side over 4096 pixels.

This directory is for images that belong to no other kind of asset. A kind
with its own directory and sidecar files (portraits, 0711; combat pictures,
0413; tilesets, 0433) keeps its PNGs there.

| File | What |
| ---- | ---- |
| `effect_marks.png` | 14×7: the arrows a sprite map skin puts on a unit under a timed effect (ADR-0049, `docs/design/look-and-feel.md`): an **up arrow** for a bonus, then a **down arrow** for a penalty, each 5×5 with a 1-pixel dark edge. Our own picture, in the palette's `effect_bonus`, `effect_penalty` and `black`. Generated; its tool was deleted (ticket 0191): `git show 61d537d2:crates/xtask/src/effect_marks.rs` |
| `edge_marks.png` | 42×13: the arrows a sprite map skin puts at the edges of the map's view where the map goes on past it (ticket 0449, `docs/design/look-and-feel.md`, *Map edges*): a right, a left, a down and an up arrow, each 6 deep and 11 across with a 1-pixel dark edge. Our own picture, in the palette's `cursor` and `black`. Generated; its tool was deleted (ticket 0191): `git show 90f97cf4:crates/xtask/src/edge_marks.rs` |
| `key_arrows.png` | 24×16: the arrow keys as one picture, four 8×8 key caps in an upside-down T, each with its arrow, which a hint draws in place of the word `arrows` for the cursor on the arrow keys (ticket 0254, `docs/design/look-and-feel.md`, *Key names in hints*). Our own picture, drawn by hand in the default theme's `key_bg`, `key_edge` and `key` (the same in every theme) |

The test image is a test file, in `assets-test/images/` (ADR-0057):

| File | What |
| ---- | ---- |
| `test_card.png` | 16×16 test image: a one-pixel white border around four coloured quadrants (red and green above blue and yellow), so a flip, a crop and a scale each show. Used by tests and the "Sprite test" debug tool. Generated; its tool was deleted (ticket 0191): `git show 61d537d2:crates/xtask/src/test_card.rs` |

The arrows and the test card are **generated**; never edit them by hand.
Their tools were deleted once the pictures stopped changing (ticket 0191):
each row names the commit that last had its tool. To make one again (the
arrows after changing their palette colours), restore the tool from that
commit with `git show <commit>:crates/xtask/src/<tool>.rs`, with its `mod`
line and its wrapper in `crates/xtask/src/main.rs` at the same commit,
and run it.

A test in `trpg-content` (`image.rs`) fails if an arrow picture has a
pixel that isn't one of its palette colours, so a palette change that
leaves them stale is caught.

## Rules for images

- PNG only, at most 4096×4096.
- Draw at whole-number scales: `app` samples the nearest pixel, so 1×, 2×,
  3×… stay sharp and other scales don't.
- Only images under a licence ADR-0013 allows, each listed in
  `THIRD_PARTY_ASSETS.md`. Bought art stays out of this repository
  (ADR-0032); a public placeholder stands in for it here.
