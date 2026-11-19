# Title art

The title screen's pictures (ticket 0811, ADR-0059;
`docs/design/look-and-feel.md`, *Title screen*), and the words of the Game
Over screen, which shows them under the logo (ticket 0830; *Game Over
screen*). Each is drawn as a sprite item (ADR-0038).

| File | What |
| ---- | ---- |
| `title.ron` | Names the logo, the backdrops and the chapter card's lettering |
| `logo.png` | The game's name as a picture, 52×31: our own, **generated** from `assets-src/title/logo.txt` and the palette. One of its pixels is drawn 8 screen pixels big |
| `game_over.png` | The words `GAME OVER` as a picture, 52×7, as wide as the logo: our own, **generated** from `assets-src/title/game_over.txt` and the palette, drawn at the logo's size |
| `chapter_name.png`, `chapter_name.ron` | The chapter title card's name lettering (ticket 0812, ADR-0128): Cinzel Bold at 40 px, every printable ASCII glyph white on clear, 16 cells a row, and the engine's atlas file (`FontAtlasDef`) giving each glyph's cell and inked columns. **Generated** from `assets-src/fonts/Cinzel-Bold.ttf` |
| `chapter_line.png`, `chapter_line.ron` | The same for the `Chapter 1` line above it: Cinzel Medium at 20 px, from `assets-src/fonts/Cinzel-Medium.ttf` |
| `campfire.png` | The campfire in the title's bottom-left corner (ticket 0922), four frames of 26×52 side by side: the orange fire with smoke of Idylwild's *Chromatic Campfires* (CC0, `THIRD_PARTY_ASSETS.md`), cut from its sheet. Not generated |
| the other `*.png` | The backdrops, 336×248 each. Here they are **placeholders** (three flat bands of palette colours, generated). The bought pictures (Tiny Tales *Battlebacks Vol.1*) replace them by name in `assets-private/game/title/` (ADR-0040) |

Never edit the PNGs by hand. To regenerate them (after changing the logo's
or the words' text, or the palette):

```bash
cargo xtask title-art
cargo xtask lettering
```

The first makes the logo, the words and the placeholder backdrops; the
second the chapter card's two sheets. A test in `xtask` fails if a
committed file differs from what the tools make. `cargo xtask title-art --bought` copies the bought backdrops from
`assets-private/library/` into `assets-private/game/title/`; commit and
push there, then `cargo shuyi private-assets --pin`.

## `title.ron`

```ron
(
    logo: "logo.png",
    game_over: Some("game_over.png"),
    backdrops: ["fields_a_dusk.png", "fields_a_night.png"],
    lettering: Some((heading: "chapter_name", line: "chapter_line")),
    campfire: Some((file: "campfire.png", frames: 4)),
)
```

- `logo`: a PNG in this folder, at most 100×34 pixels (at 8 screen pixels
  each: the screen's width, and the rows above the menu).
- `game_over`: optional; a PNG in this folder, at most as big as a logo.
  The words of the Game Over screen, drawn under the logo at its size and
  centred like it: make it as wide as the logo and the two line up.
  Without it the screen shows the words as text.
- `backdrops`: PNGs in this folder, 336×248 each (a battle backdrop's
  size), each named once. The title shows one per launch, never the one it
  showed last. The list may be empty: the logo is then on plain black.
- `lettering`: optional; the chapter title card's two glyph sheets by file
  stem here, `heading` for the chapter's name and `line` for `Chapter 1`
  above it. Each stem is a `.ron` and a `.png`. Without it the card prints
  its lines in the cell font.
- `campfire`: optional; the title's campfire (ticket 0922), a PNG here
  with `frames` pictures side by side, each the image's width over
  `frames`. Without it, or in a build that leaves the file out (the
  Playdate's, `crates/content/playdate-bundle.txt`), the title has no
  fire and no crackle; the embers still rise from where it would be.

**Checked by the loader**, all at once: a RON syntax error or an unknown
field; an image that isn't a PNG in this folder; a logo or Game Over
picture that is too big; a backdrop of another size or named twice; a
lettering sheet missing, without a printable ASCII glyph, with a cell
outside its image, or without its glyphs' `ink`; a campfire missing, or
not a whole number of its frames wide.

## `assets-src/title/logo.txt` and `game_over.txt`

Lines before `---` give each character a palette colour (`c cursor`);
lines after it are the rows of the picture, one character a pixel, `.` for
a clear one. Blank lines and lines starting with `#` are skipped. Only
palette names are used.

## Seeing it

```bash
cargo shuyi frame-png target/title.png
```

Add `--prompt` for the "Press any key or button" state, and `--features
private-assets` after `xtask` (`cargo run -p xtask --features
private-assets -- frame-png …`) for the bought backdrops.
