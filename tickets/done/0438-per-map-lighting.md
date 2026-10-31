---
id: "0438"
title: "Per-map lighting: low-light maps are shaded, noon maps are not"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0437"]
nick_input: sign-off
completed: 2026-10-12
---

# 0438 — Per-map lighting

## Context

Nick, 2026-10-02 (ticket 0038), choosing between the bright bought tiles
(spike render C) and the same tiles darkened toward our earthy palette
(render C3): "Per-map lighting effects (i.e. if it takes place during what
would be a low light time, it can be shaded like with C3. Or if it's noon,
it can be like C/C2."

Rule, in `docs/design/look-and-feel.md` (*Battle map: bought tiles and
unit sprites*): lighting is set per map; noon shows the tiles as bought;
low light shows them darkened; unit sprites keep their own colours in
every light (*the agent's starting rule*, from how C3 was drawn).

How C3 was made (the look Nick approved): each terrain pixel was moved 35%
of the way to its own grey, then its red, green and blue were multiplied
by 0.62, 0.56 and 0.46. Units, the cursor, the HP bars and the UI were not
touched.

0437 draws the terrain with sprite items and gives a map file a `look`
header. A sprite item has no tint yet (ADR-0038 section 2; 0413 and 0436
add other sprite effects).

**From 0437 (2026-10-03, ADR-0052):**

- The whole `MapLook` (`trpg_content::map`) is already in the scene:
  `MapScene::look`. Add `light` to `MapLook` and it reaches the skin; the
  game flow gives the battle screen its map's look
  (`BattleScreen::with_look`), which is where a battle file's `light`
  override goes.
- The ground is painted in `crates/ui/src/map_view/sprite/ground.rs`:
  each tile's own picture, then the look's layers (pictures between
  tiles, and on tiles), all from the tileset's one image. Shade those.
  The tints painted after them (ranges, flashes, the cursor's glow) are
  sprites of that image too, in one colour (`Paint::Solid`): don't shade
  them.
- The importer is `cargo xtask tileset-import`
  (`crates/xtask/src/tileset_import.rs`); it already recolours tiles
  (`tint`), for the burning forest.

**Added 2026-10-11 (ticket 0413).** The combat stage's backdrops
(ADR-0056, `assets/battlebacks/battlebacks.ron`) are the bundle's **day**
pictures only. The bundle has each one at dusk and at night too. When a
map has a light, the stage should show that light's picture: give the
backdrops' file a picture per light (and `cargo xtask battler-import`'s
`BACKDROPS` the dusk and night files), and pass the map's light to
`stage::view` next to the ground.

## Nick input

**Sign-off:** one map rendered in each light, side by side, with bought
tiles and units (never committed), and the list of which light each
existing map got. He may ask for other lights or strengths: the values are
data.

## Scope

**In:**
- A small set of named lights as data, each a recipe for shading terrain.
- A map's light, named in its `look` header; default noon.
- The sprite skin shades terrain tiles by the map's light. Range tints,
  cursor, path, HP bars and unit sprites are drawn as before, on top.
- The combat scene's background (0413), if it uses the bought battle
  backgrounds: they come in day, dusk and night versions. Add a line to
  0413 (or to its follow-up if 0413 is done) that the background's version
  follows the map's light. Don't build it here.

**Out (do not do):**
- Lights that change during a battle, light sources, shadows, weather.
- Shading the glyph skin (it is already dark) or the UI.
- Shading unit sprites.
- Colour themes (0806): a theme never recolours a picture.

## Implementation steps

1. **Data** `assets/data/lighting.ron`: `{ "noon": (grey: 0.0, tint: (1.0,
   1.0, 1.0)), "dusk": (grey: 0.35, tint: (0.62, 0.56, 0.46)), … }`.
   Start with **noon** (as bought), **dusk** (C3's values above) and
   **night** (darker and bluer than dusk: *the agent's starting values*, e.g.
   grey 0.45, tint (0.40, 0.44, 0.58)). All *tunable*. Loader and validator
   in `trpg-content` (values in `0.0..=1.0`; `noon` must exist).
2. **Map look**: `look: (tiles: "outdoor", light: "dusk")` in the `.map`
   header (0437's `MapLook`); unknown light name is a validation error
   with file and line. A battle file (`assets/battles/*.ron`) may override
   its map's light with an optional `light:` so one map can be fought at
   two times of day; that field is look data too and stays out of
   `trpg-core`. Document both READMEs.
3. **Shading**. A multiply tint alone can't grey a colour, so choose one
   and record it in an ADR (`write-adr`):
   - *(preferred)* the sprite item gains `shade: Option<(grey, tint)>`,
     applied by `app`'s renderer (a small fragment shader in macroquad,
     the same maths in 0232's PNG renderer), one snapshot token per shaded
     sprite; or
   - the importer (0437) writes one pre-shaded copy of the tileset image
     per light into `assets-private/game/`, and the skin picks the image.
     Simpler to draw, but every new light means re-importing.
   Check the web build draws the same as native (in the browser pane,
   reading canvas pixels back as 0231 did).
4. **Skin**: `SpriteSkin` reads the scene's light (add it to `MapScene`
   as a plain name, ADR-0038: the screen says what, the skin says how)
   and shades terrain sprites only. `every_scene_feature_is_painted`
   (0433) gains the light.
5. **Content**: set each existing map's light. `test_small` stays noon.
   For Chapter 1 (0803, if done) take the time of day from
   `docs/story/chapters/ch01.md`; if the story doesn't say, leave noon and
   ask Nick in the sign-off. Don't invent a time of day.
6. Render the three lights with the bought tiles, **look at them**, send
   them to Nick.

## Acceptance criteria

- [x] A map with `light: "dusk"` draws its terrain shaded and its units
      unshaded (test on the sprite items: terrain sprites carry the shade,
      unit sprites don't).
- [x] A map with no `look` or no `light` draws exactly as before this
      ticket (snapshots unchanged).
- [x] Unknown light name: a validator error naming the file and the name.
- [x] The shading maths is one pure function with a unit test pinning C3's
      numbers on three sample colours, used by both renderers.
- [x] Glyph-skin snapshots unchanged; `the_skin_never_changes_the_game`
      passes with a shaded map.
- [x] Nick was sent the three renders and the per-map list.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `lighting.ron` loading and each error; `MapLook` with and without
  a light; the battle-file override; the shading function.
- Snapshot / integration: Quick Battle on the public fixture tileset under
  dusk; behaviour tests read the `MapScene`.

## Completion notes

**Done (2026-10-12).** A map is in a light: noon (as before), dusk or
night. With the bought tiles the ground is shaded by it; units, ranges,
the path, the cursor, HP bars and every panel keep their colours. The
combat stage's backdrop follows the light too.

**Which light each map got.** Both maps are at **noon**:

- `test_small` (the Quick Battle): noon, as the ticket says.
- `ch01` (Harrowby Ford): noon. `docs/story/chapters/ch01.md` puts the
  scene before the battle "at dawn" and the scene after it "at noon", so
  the battle is in the morning; the story never calls it low light. Not
  invented: **Nick, say if Chapter 1's battle should be at dusk** (one
  word in `assets/maps/ch01.map`).

**the agent's starting rules** (Nick can veto; all in `look-and-feel.md`):

- **Night** is darker and bluer than dusk (grey 0.45, tint 0.40 / 0.44 /
  0.58). Dusk is render C3's numbers exactly.
- The range colours, the path, the cursor and the HP bars aren't shaded
  (the ticket listed them; C3 left them alone).
- In a fight on the combat stage, the backdrop is the bundle's own dusk
  or night picture of the same place. A light the bundle has no picture
  for (none today; any light Nick adds later) shows the day picture
  shaded like the map's ground. The fighters aren't shaded.
- The glyph map looks the same in every light.

**What was built** (ADR-0058).

- `assets/data/lighting.ron` and `trpg_content::lighting`: the lights as
  data, each `(grey, tint)`; `Light::apply` is the one function that
  shades a colour, pinned on C3's numbers.
- A map's header names its light (`look: (light: "dusk")`); a battle file
  may name another (`light: "night"`). The battle's light isn't in
  `core`'s `BattleDef`: `Content::battle_lights` keeps it and
  `Content::battle_look` gives the game flow the map's look in it.
- `Paint::Shaded(Light)`: the sprite skin puts it on every picture of the
  ground. `app` draws it from a recoloured copy of the tileset image,
  like an acted unit's grey; `frame-png` already painted by
  `Paint::apply`. Snapshot token: ` shade=350:620,560,460`.
- The backdrops' file may name pictures per light; `cargo xtask
  battler-import` now also copies the bundle's dusk and night pictures and
  writes the file that names them into the private assets.
- `cargo xtask frame-png --light <name>`: fights every battle in that
  light, for looking.

**Deviations from the plan.**

- **No shader.** The ticket preferred a fragment shader. `app` already
  draws recoloured sprites from a copy of the image made on the CPU
  (ADR-0049), so the shade is one more `Paint`, with one implementation of
  the sum for the window, the web and `frame-png`. ADR-0058 has the
  reasons.
- The shade is a new `Paint`, not a `shade: Option<…>` field of `Sprite`.
- The light isn't a separate field of `MapScene`: it is `MapScene::look`'s
  `light`, a plain name, since the whole `MapLook` was already there.
- **The combat backdrops were built here**, not left to a follow-up: the
  note 0413 added to this ticket on 2026-10-11 asks for it, and it is
  newer than the Scope line that says not to.
- The light check of a map file runs after the maps load
  (`map::check_lights`), since `parse_map` has no lights' file; the error
  still names the file, line and column.
- **Web check not done by reading canvas pixels.** That check was for a
  shader, which could differ between WebGL and desktop GL. There is none:
  the recoloured copy is made by the same Rust on every platform, the
  path acted units have used on the web since 0436. The WASM build gate
  passes. Nick will see it on Pages; no map there is in low light yet.

**Follow-ups:** none created. Not done because out of scope: lights that
change during a battle, light sources, shadows, shading the glyph map.

**For Nick (sign-off).** Three renders of the Quick Battle map and of a
fight on the stage, at noon, dusk and night, with the bought art (sent in
the session, never committed). To see it in the game: put
`look: (light: "dusk")` in a map's header, or `light: "dusk"` in a battle
file, and run with `--features private-assets`. Things to judge: how dark
dusk and night are; whether Chapter 1's battle should be in low light; and
that the bundle's dusk backdrop is a bright sunset, brighter than our dusk
map (say if the shaded day picture would fit better).
