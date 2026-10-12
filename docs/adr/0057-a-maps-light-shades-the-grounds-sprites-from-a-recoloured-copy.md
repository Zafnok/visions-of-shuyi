# ADR-0057: A map's light is a name; the ground's sprites carry its shade and are drawn from a recoloured copy of the image

- **Status:** Accepted
- **Date:** 2026-10-12
- **Related tickets:** 0438, 0038, 0413, 0433, 0437, 0806
- **Extends:** ADR-0038 (§2: what a sprite item can say; §5: a map file's
  look data), ADR-0049 (how a sprite is painted), ADR-0052 (§3: the look a
  map file names), ADR-0056 (the backdrops' file)

## Context

- Nick decided lighting is set per map (`docs/design/look-and-feel.md`,
  *Battle map: bought tiles and unit sprites*): noon shows the bought
  tiles as they are, low light shows them darkened as in render C3, and
  units keep their own colours.
- C3 was made by moving each terrain pixel 35% of the way to its own grey
  and then multiplying red, green and blue by 0.62, 0.56 and 0.46. A
  multiply tint alone (what a textured quad's vertex colour gives) can't
  grey a colour.
- A sprite item already says how its pixels are coloured
  (`Paint`, ADR-0049): the image's own colours, one colour, or dimmed (a
  unit that has acted). `app` draws the last two from a **recoloured copy**
  of the image's texture, made on the CPU the first time a sprite needs it
  and kept; `cargo xtask frame-png` applies the same `Paint::apply` per
  pixel. Both therefore use one function and draw the same on every
  platform.
- The ground is many sprites of one image (ADR-0052); the units, the
  ranges' washes and the cursor are other items.
- `trpg_core::BattleDef` and `BattleMap` are what the rules, the bots and
  saves see. A light is look data and must not be in them (ADR-0038 §5).
- The combat stage's backdrops (ADR-0056) were bought at day, dusk and
  night; the public placeholders are day only.

## Decision

### 1. Lights are data

`assets/data/lighting.ron` is a map from a light's name to
`(grey: 0..1, tint: (r, g, b) each 0..1)`. `trpg_content::lighting` loads
it: every number from 0 to 1, and `"noon"` must exist (the light of a map
that names none). It ships `noon` (changes nothing), `dusk` (C3's numbers)
and `night`.

A loaded `Light` keeps its numbers as **thousandths** (`u16`), so it is
`Copy + Eq + Hash`: a frame item can carry it, a snapshot can print it,
and a renderer can key a cache by it. `Light::apply([r, g, b])` is the one
place that knows the sum: towards the pixel's own grey (the same grey as
`Rgb::dimmed`: 0.299 r + 0.587 g + 0.114 b), then times the tint, rounded.

### 2. A map names its light; a battle file may name another

- `MapLook` (ADR-0052 §3) gains `light`, `"noon"` by default:
  `look: (tiles: "indoor", light: "dusk")`.
- A battle file may say `light: "night"` to be fought in another light
  than its map's. The battle loader's `BattleDef` is `core`'s and doesn't
  get it: `Content::battle_lights` keeps it by battle id, and
  `Content::battle_look(&def)` is the map's look in the battle's light.
  The game flow gives that to the battle screen where it gave the map's
  look.
- A name that isn't a light is a content error at the name's line and
  column, in the map file or the battle file. The check needs the lights'
  file, so it runs after the maps load (`map::check_lights`), not in
  `parse_map`.

The light reaches the skin as part of `MapScene::look`: a **name**. The
screen says what; the skin says how (ADR-0038).

### 3. The shade is a `Paint`

`Paint::Shaded(Light)` joins `Image`, `Solid` and `Dimmed`.

- The sprite skin looks the scene's light up in the content and paints
  every sprite of the ground with it: each tile's own picture and the
  layers' pictures. A light that changes nothing (`Light::is_plain`) or a
  name the content doesn't have paints `Paint::Image`, so a map at noon is
  the frame it was before this ADR, item for item.
- What is painted over the ground is untouched: the ranges', flashes' and
  glow's washes (`Paint::Solid`), the path, the units, their bars and
  marks, the cursor. The glyph skin ignores the light, as it ignores the
  look.
- `app` draws a shaded sprite from a recoloured copy of the image, cached
  by `(image, light)`, exactly as it does a dimmed one. `frame-png` needs
  no change: it already calls `Paint::apply`.
- A snapshot line ends ` shade=350:620,560,460` (grey, then the tint, in
  thousandths).

### 4. The combat stage's backdrop follows the light

The backdrops' file may name pictures **drawn for a light**:
`lights: { "dusk": (default: …, terrain: {…}) }`. `Backdrops::in_light`
gives that light's picture if the file has pictures for it, else noon's
picture and the fact that it is noon's; the stage's view then carries the
light to shade it by (`StageView::shade`), and its skin paints the
backdrop `Paint::Shaded`. So the public placeholders need no dusk and
night files, and the bought build shows the artist's own dusk and night:
`cargo xtask battler-import` copies them and writes the backdrops' file
that names them into the private assets.

## Consequences

- A new light, or a different strength, is one line of
  `lighting.ron`: no code, no re-import.
- A shaded map costs one more texture the size of the tileset's image per
  light in use (one map is in one light, so one), made once. No shader:
  the web build and the native build run the same Rust for it.
- Every shaded ground sprite has a `shade=` token in snapshots, so a
  snapshot of a dusk map is as long as noon's and differs on those lines
  only.
- Two things can't be done this way and would need a real shader or a
  second pass: a light that changes **during** a battle smoothly (each
  step would be a new recoloured copy), and light sources or shadows
  (the shade is one recipe for the whole image). Both are out of 0438.
- A light's numbers are kept to three decimals; the file may write more
  but they are rounded.
- The battle-file light is read by parsing the battle file a second time
  for that one field. Three small files; if battles become many, the
  loader can return it beside the `BattleDef` instead.
- Colour themes (0806) recolour UI, never pictures; a light recolours
  pictures, never UI. They don't meet.

## Alternatives considered

- **A fragment shader in macroquad, with the same sum in `frame-png`**
  (the ticket's first suggestion): two implementations of the sum (GLSL
  and Rust) to keep equal, a material switch between shaded and unshaded
  sprites that breaks macroquad's batching mid-frame, and GLSL that must
  be written for both desktop GL and WebGL. The recoloured copy is already
  how `Dimmed` works and has neither problem.
- **The importer writes one pre-shaded copy of the tileset image per
  light** (the ticket's second suggestion): every new light or changed
  number means re-importing and pushing the private assets, the public
  fixture would need its own copies, and the game would hold lights it
  can't check against the data.
- **`shade: Option<…>` as a new field of `Sprite`**: a second way to say
  how a sprite is coloured beside `paint`, with combinations (shaded and
  solid?) that mean nothing. A paint is one choice.
- **A dark see-through rectangle over the map**: it can't grey a colour,
  and it would darken the units, which the design rules out; drawn under
  the units it would still tint the ranges.
- **The light as a field of `BattleDef`**: it would be in `core`, in the
  bots' inputs and in every suspended battle's save, for something no rule
  reads.
- **Lights as a fixed list in code, like the tile looks** (ADR-0052): the
  looks need pictures per name, so a list is honest there; a light is only
  numbers, and Nick is expected to ask for other lights and strengths.
