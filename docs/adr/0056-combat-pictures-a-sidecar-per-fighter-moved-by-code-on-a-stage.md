# ADR-0056: Combat pictures: a sidecar per fighter, one sprite each, moved by code on a stage

- **Status:** Accepted
- **Date:** 2026-10-11
- **Related tickets:** 0413, 0021, 0038, 0040, 0404, 0711, 0438
- **Follows:** ADR-0032 (the art is bought), ADR-0038 (a picture is a
  sprite item), ADR-0040 (bought files live in the private repository),
  ADR-0043 (the same shape as portraits), ADR-0051 (a bought file needs a
  credit), ADR-0054 (a view, painted by a skin)

## Context

- Nick wants the combat screen to show the two fighters full-body
  (`look-and-feel.md`, *Combat screen*). The art is one **still** picture
  per hero or class, bought (Tiny Tales, Shironejiya, later Pixel Flag),
  and **moved by the game**: run up, flash, shake, fade.
- The pictures come from several artists: different sizes (48 to 128 px
  wide), different facings (Tiny Tales looks right, Shironejiya left), and
  Nick sized each source so the fighters stand equally tall (1×, 1.25×,
  1.5×). Shironejiya's are drawn on a flat background colour.
- In ticket 0413 Nick chose where the scene goes: **the map pane becomes
  the stage** at double size, with a bought backdrop, the numbers in a
  plate per fighter and the result over the target's head.
- 0404's playback already lays every strike on a timeline of beats
  (intro, the striker's name flashing, the result while HP drains, a gap,
  the falls, the outro), times the sounds on it, and is what "hold Confirm
  = faster" and "Cancel = skip" act on.
- The bought files can't be in this repository, and every gate must pass
  without them (ADR-0040).

## Decision

### 1. A battler is a sidecar and one image

`assets/battlers/<id>.ron`:

```ron
(
    id: "brigand",
    image: "brigand.png",
    faces: Left,
    scale: (5, 4),
)
```

`id` is the file stem: a class id, a character id (`lead_m` / `lead_f` for
the lead), or `default`. `faces` is how the file is drawn; `scale`
(optional, `(1, 1)`) is a fraction from 1 to 2. `trpg_content::battler`
loads them after the image table into `Content::battlers` and reports
every problem of every file at once (`assets/battlers/README.md` lists the
checks). `content` decodes no pixels.

**A fighter's picture** is the first that exists of: its character's (the
lead's by gender first), its class's, `default` (which must exist). So a
new class or character needs no code and no data change to appear: it
shows the default until its sidecar is added.

**Backdrops** are one file, `assets/battlebacks/battlebacks.ron`: a
`default` image and a map from terrain id to image, all 336×248. The
fight's backdrop is the one of the terrain the defender stands on. The
map's light (0438) isn't in it yet.

### 2. Public placeholders, bought files over them

Every sidecar and image the gates need is a plain placeholder in
`assets/` (a figure per Chapter 1 class, `default`, two backdrops). The
bought ones are at the **same paths** in `assets-private/game/`, so they
replace them in a `private-assets` build, and the backdrops' file is
shared. Character pictures (`lead_f`, `heretic`, `red_captain`) exist only
privately; without them those fighters show their class.

`cargo xtask battler-import` writes the private files from a table in
`crates/xtask/src/battler_import.rs` (who gets which bought file, its
facing, its scale): it copies the image, makes the flat background of a
"keyed" picture clear, and writes the sidecar. Nothing is edited by hand
in `assets-private/game/battlers/`.

### 3. The stage is a view of the playback; a skin paints it

`screens/battle/stage.rs` builds a `StageView` from the `Playback` each
frame (ADR-0054): the backdrop, the two `FighterView`s (name, faction, HP
as shown now, numbers, picture, `Pose`), who stands on the left, whether
they fight hand to hand, the result text and its target, and whether the
stage flashes. `stage/glyph.rs` paints it over the map view: the backdrop
and each picture as **one sprite item**, mirrored with `flip_x` to face
its opponent, the plates and labels as cells. Tests of what a fight shows
read the view; only the skin's tests read sprites and cells.

With combat animations off (`Settings::combat_animations`) there is no
view and 0404's box is drawn instead, as before.

### 4. Motion is pure functions of the playback's clock

`stage/motion.rs` turns "which beat, how far into it" into a `Pose`
(`lunge`, `dodge`, `shake`, `flash`, `fade`), with no unit of length; the
skin multiplies by its pixel constants. The run-up fills the beat in
which the striker's name flashes, so the strike lands exactly when the
result and its sound do; nothing was added to the timeline and the
speed-up and skip need no change. The timings are constants in `ui`, not
per-image data.

No new sprite feature was needed: a hit is `Paint::Solid(white)`, a fall
is `opacity`, and the critical's white flash is the backdrop drawn again
as a solid white shape at part opacity.

## Consequences

- Adding a fighter's picture is a row in the importer's table (bought) or
  a sidecar and a PNG (ours); swapping an artist is the same.
- Fractional scales (1.25×, then doubled) are drawn with nearest-pixel
  sampling, so some picture pixels are 2 screen pixels and some 3. Nick
  chose those sizes on mockups drawn the same way.
- The stage hides the map while a fight plays. A dialogue scene triggered
  inside a combat (a boss's line, last words) now plays over the stage.
- Named characters without their own art look like their class until a
  recoloured picture is imported for them (the importer has no recolour
  step yet).
- A picture taller than the ground line allows (138 px at double size)
  would be cut at the scene's top; the loader's 240×144 limit is looser
  than that so the big *Gods and Gallants* pictures can be tried later.
- A second look for the stage (a pack's panels for the plates) is another
  `paint` for the same view.

## Alternatives considered

- **The battlers in the tileset file, like the map's units (ADR-0049).**
  One file for everything a unit looks like, but a tileset is a map skin
  the player can switch off (glyph map), while the stage is the same in
  both; and the ticket asked for a sidecar per class, like portraits.
- **Per-image motion data (frames, anchors, keyframes).** The pictures
  are stills; the same few motions fit all of them. Timing in code keeps
  every fight consistent and the files trivial.
- **Animating with the timeline (new beats for run-up and return).** It
  would change every fight's length and the sounds' places for a look.
- **Per-pixel flash (a tint parameter on `Sprite`).** Not needed: a solid
  silhouette reads as a hit flash, as in the games Nick named.
- **The box over the map, art at its own size.** Shown to Nick; he chose
  the stage (the run-up has room, the art reads at double size).
