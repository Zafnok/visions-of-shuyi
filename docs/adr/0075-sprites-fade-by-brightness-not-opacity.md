# ADR-0075: Sprites fade by brightness, not opacity

- **Status:** Accepted
- **Date:** 2026-10-15
- **Related tickets:** 0732 (this: conversations over their place,
  darkened), 0817 and 0819 (the cinematic fade, ADR-0060), 0734 (the
  moving opening that darkens into the talk)

## Context

`GlyphBuffer::fade(brightness)` darkens a whole frame (ADR-0060 §4): cell
colours and rectangle colours are scaled, and each sprite's `opacity` is
scaled. ADR-0060 accepted the flaw that comes with the opacity: "where two
sprites overlap in a scene the lower one shows through a little during a
fade. Unseen in a short fade; a brightness applied by the renderers would
be the fix if it ever matters."

0732 holds a fade for a whole conversation: the place behind the talk at
0.4 brightness. The bought tileset lays each tile as several sprites (the
ground, then corners and edges over it), so at 40% opacity each layer lets
the one below show through and the layers add back up: a frame rendered
with the bought art showed the place at nearly full brightness. A
picture made of stacked sprites can't be darkened by opacity.

## Decision

- `Sprite` gets a **`brightness: u8`**: 255 = as the image and its paint
  have it, lower = that much darker, 0 = black where it shows.
  `Sprite::new` sets 255. It is separate from `opacity`, which stays how
  see-through the sprite is.
- `GlyphBuffer::fade` scales each sprite's `brightness`, no longer its
  `opacity` (cells, rectangles and the backdrop's copy as before). This
  amends ADR-0060 §4; the rest of ADR-0060 stands.
- Every renderer multiplies a sprite's colour by its brightness, after its
  paint: `app` scales the tint it draws the texture with, `frame-png`
  scales the tint it blits with, and the one-bit renderer (ADR-0074) keeps
  that share of the sprite's white pixels, by the same ordered dither as
  opacity.
- Snapshots list ` bright=N` on a sprite below 255 (`snapshot.rs`).

## Consequences

- A faded picture made of stacked sprites is exactly as dark as one made
  of a single sprite; the conversation's place, the battle map behind a
  battle's scene and the title's cinematic fade all darken evenly.
- A faded sprite no longer lets the cells under it show through: the
  frame goes to black, as ADR-0060 meant it to.
- Snapshots of faded frames show `bright=N` where they showed
  `opacity=N`.
- One more field on `Sprite` for every renderer to honour (a new renderer,
  such as the Playdate's, must apply it).

## Alternatives considered

- **Keep fading the opacity.** Stacked tiles stay nearly full brightness:
  the place behind a conversation wouldn't look darkened at all with the
  bought art.
- **A dark veil over the place** (a see-through black rectangle, or a
  black silhouette of each sprite). Rectangles have no opacity and there
  is no plain image to draw a veil sprite from; silhouettes over stacked
  sprites compound and darken unevenly; and a veil leaves the cells' own
  colours as they are, so tests and the one-bit frame wouldn't see it.
- **A brightness on `Backdrop` only** (painting the place into a picture
  first). Backdrop scenes are drawn sprite by sprite too, so the stacking
  is the same; and ADR-0060 had already ruled out a backdrop-only
  brightness.
