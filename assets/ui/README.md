# Window frames

The picture look's window frames (ticket 1380;
`docs/design/look-and-feel.md`, *Windows (the picture look)*): which
picture frames each window, where its corners end, the colour its inside
is filled with, and the plate a title sits on. Read by
`trpg_content::frames`; the skins ask for each frame by name.

| File | What |
| ---- | ---- |
| `frames.ron` | The frames and plates, by name. Ours: names and numbers |

The pictures are **bought art** (ADR-0040), in the private assets'
`game/ui/`, cut from the bought packs by `cargo xtask ui-import` (its list:
`assets-src/ui/pictures.ron`). A build without them has no frames, and every
window keeps its glyph lines whatever the *Window look* option says.

## `frames.ron`

```ron
(
    frames: {
        "panel": (
            picture: "panel.png",                      // in ui/
            borders: (left: 22, top: 33, right: 22, bottom: 32),
            fill: "#3a2c36",                           // the inside
            outset: 5,                                 // console px past the window
        ),
    },
    plates: {
        "title": (picture: "plate.png", caps: (left: 16, right: 16), pad: 6, text_top: 8),
    },
)
```

- A **frame** is its picture cut in nine slices: `borders` are where its
  corners end, in the picture's pixels; the corners show once and the
  edges stretch between them. Only its border is drawn: the window's
  inside is its cells, filled with `fill`. `outset` (default 0) draws the
  frame that many console pixels past the window on each side, so wide
  corners keep off the text. A frame is drawn at one console pixel per
  picture pixel (the font's pixel).
- A **plate** is two end caps (`caps`, in the picture's pixels) and the
  body between them, stretched to the title's width plus `pad` console
  pixels on each side. The title's row is `text_top` picture pixels below
  the plate's top.
- Each name the windows ask for must be there, and no other
  (`FRAME_NAMES`, `PLATE_NAMES`): `panel` (the side panel and the battle's
  boxes), `menu` (the unit menu and its sub-menus), `map_menu`, and the
  plate `title`. A picture's borders or caps must leave room between
  them.
