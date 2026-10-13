# Title art

The title screen's pictures (ticket 0811, ADR-0059;
`docs/design/look-and-feel.md`, *Title screen*). Each is drawn as a sprite
item (ADR-0038).

| File | What |
| ---- | ---- |
| `title.ron` | Names the logo and the backdrops |
| `logo.png` | The game's name as a picture, 52×31: our own, **generated** from `assets-src/title/logo.txt` and the palette. One of its pixels is drawn 8 screen pixels big |
| the other `*.png` | The backdrops, 336×248 each. Here they are **placeholders** (three flat bands of palette colours, generated). The bought pictures (Tiny Tales *Battlebacks Vol.1*) replace them by name in `assets-private/game/title/` (ADR-0040) |

Never edit the PNGs by hand. To regenerate them (after changing the logo's
text or the palette):

```bash
cargo xtask title-art
```

A test in `xtask` fails if a committed file differs from what the tool
makes. `cargo xtask title-art --bought` copies the bought backdrops from
`assets-private/library/` into `assets-private/game/title/`; commit and
push there, then `cargo xtask private-assets --pin`.

## `title.ron`

```ron
(
    logo: "logo.png",
    backdrops: ["fields_a_dusk.png", "fields_a_night.png"],
)
```

- `logo`: a PNG in this folder, at most 100×34 pixels (at 8 screen pixels
  each: the screen's width, and the rows above the menu).
- `backdrops`: PNGs in this folder, 336×248 each (a battle backdrop's
  size), each named once. The title shows one per launch, never the one it
  showed last. The list may be empty: the logo is then on plain black.

**Checked by the loader**, all at once: a RON syntax error or an unknown
field; an image that isn't a PNG in this folder; a logo that is too big; a
backdrop of another size or named twice.

## `assets-src/title/logo.txt`

Lines before `---` give each character a palette colour (`c cursor`);
lines after it are the rows of the picture, one character a pixel, `.` for
a clear one. Blank lines and lines starting with `#` are skipped. Only
palette names are used.

## Seeing it

```bash
cargo xtask frame-png target/title.png
```

Add `--prompt` for the "Press any key or button" state, and `--features
private-assets` after `xtask` (`cargo run -p xtask --features
private-assets -- frame-png …`) for the bought backdrops.
