# Combat pictures (battlers)

In the combat scene each fighter is one still, full-body picture, moved by
the game (ADR-0056, `docs/design/look-and-feel.md`, *Combat screen*). A
battler is a **sidecar file** here, `<id>.ron`, that names one **PNG
image**. It is loaded by `trpg_content::battler` and validated by the
all-assets test (ADR-0005).

The files here are **placeholders**: plain figures made once for ticket
0413 so that a clone without the bought art builds, tests and runs. The
real pictures are bought art (ADR-0032): they are not in this repository
but in `assets-private/game/battlers/` (ADR-0040), where a file replaces
the one at the same path here. `cargo xtask battler-import` copies them
there from the bought packs and writes their sidecars; its table says
which picture each class and character gets (`cargo xtask battler-import
--list`).

## Whose picture a fighter gets

The first of these that exists:

1. its character's: `<character id>.ron` (the lead's is `lead_m.ron` or
   `lead_f.ron`, by the gender the player picked);
2. its class's: `<class id>.ron`;
3. `default.ron`, which must exist.

So a named character with no picture of their own looks like their class,
and a class with no picture yet shows the default figure. A class's bought
picture is the one the enemy's soldiers show (the Red Company's guard is
the Church Knight in red); a named character who shares it gets a row of
their own in the importer's table, recoloured (Hollis: the same knight in
blue).

## Format

```ron
// RON `//` comments are allowed.
(
    id: "brigand",
    image: "brigand.png",
    faces: Left,
    scale: (5, 4),
)
```

- `id`: the file stem.
- `image`: a PNG, relative to this folder, at its own pixel size.
- `faces`: `Left` or `Right`, the way the fighter looks in the file. The
  two fighters always face each other: the game mirrors a picture that
  looks the wrong way for the side it stands on.
- `scale` (optional, `(1, 1)` if left out): how big it is drawn next to a
  picture at `(1, 1)`, as numerator and denominator, from 1 to 2 times.
  `(5, 4)` is 1.25×. The scene then doubles everything.

**Checked by the loader**, all at once, each with its file: a RON syntax
error or an unknown field; `id` ≠ the file stem; an image that isn't a PNG
in this folder; a scale outside 1 to 2 times; a picture bigger than
240×144 px once scaled; no `default.ron`.

## Backdrops

`assets/battlebacks/battlebacks.ron` names the picture behind the
fighters, by the terrain the defender stands on:

```ron
(
    default: "fields.png",
    terrain: {
        "mountain": "mountains.png",
        "peak": "mountains.png",
    },
)
```

Every backdrop is a 336×248 PNG in `assets/battlebacks/`; the scene shows
its whole width and 192 rows of it at double size, and its two edges again,
mirrored, to fill the screen's width. The loader checks that each image is there
and that size, and that each terrain id exists. The files there are
placeholders too: the bought ones replace them by name, so the two share
this file.

## Seeing one

```bash
cargo xtask frame-png target/fight.png --keys "Down f Left f" --wait 0.5 --keys "f" --wait 0.5 --keys "f Right Right Right Up f" --wait 1.5 --keys "f f f" --wait 0.75
```

renders the Quick Battle's lord striking a brigand. Add `--features
private-assets` after `xtask` (`cargo run -p xtask --features
private-assets -- frame-png …`) to see the bought pictures.
