# ADR-0060: Cinematics: a file of timed shots, each frame drawn from the time alone

- **Status:** Accepted
- **Date:** 2026-10-12
- **Related tickets:** 0817, 0036, 0818, 0819, 0820, 0827, 1010
- **Follows:** ADR-0005 (content is data, validated), ADR-0037 (the music
  clock), ADR-0038 (the map is painted by a skin), ADR-0048 (the backdrop
  and `cinema::view`), ADR-0057 (test files), ADR-0059 (`paint_logo`)

## Context

- Nick wants an intro cinematic on the title (0036,
  `docs/design/title-screen.md`, *Intro cinematic*): shots of a
  battlefield, the overworld, characters and conversation snippets, as
  long as the title song, looping with it, ending on the logo. He chose
  **hard cuts** between shots and **whole zoom steps**, one per shot.
- The title will run it in time with the music (0819): the music clock
  can lag, jump (a skip press goes to the song's silence) and wrap.
- The real cinematic (0820) and later shots (0818, 0827, 1010) must be
  changed without code.
- `Content` is compared with `Eq`, so it can hold no floats.
- The test battle and map are test files (ADR-0057): a build that ships
  doesn't have them, and no story file may name one.
- The battle map is whatever the map skin paints (ADR-0038), and a
  sprite skin has the player's own map zoom (0439).

## Decision

### 1. The file

`assets/cinematics/<id>.ron` (format: `assets/cinematics/README.md`):

```ron
(
    id: "title",
    length: Music("title"),      // or Seconds(10.0)
    shots: [
        (at: 0.0, shot: MapPan(battle: "ch01", from: (4, 9), to: (12, 3.5), zoom: 2)),
        (at: 6.0, shot: Logo),
    ],
)
```

- `at` is seconds from the start. A shot lasts until the next one's `at`;
  the last until the end. There is no fade or transition field: shots cut.
- `length` is `Music(cue)` (the cue's `length_ms`, so the file never
  repeats a number that `audio.ron` owns) or `Seconds(n)`, for a
  cinematic on no track. The ticket sketched a required `music:` field;
  `Seconds` was added because the only thing to test with would otherwise
  be a song of two minutes.
- A map point is in tiles, a whole number being a tile's **middle**, so
  it means the same place under every skin and tile size.
- `trpg_content::cinematic` loads them into `Content::cinematics`
  (`CinematicDef`), reporting every problem of every file at once: the id,
  the length, the shots' order (first at 0, each after the last, all
  before the end), and per shot kind its own (a pan: the battle exists,
  both points are on its map, the zoom is 1 to 4). Unknown fields are
  refused.
- Times are kept in whole milliseconds and points in thousandths of a
  tile (`TilePoint`): no floats in `Content`.
- The test cinematic is `assets-test/cinematics/test.ron`.

### 2. A frame is a pure function of the time

`trpg_ui::cinematic`:

- `Timeline::at_ms(t) -> ShotAt { index, progress }`, with `t` wrapped at
  the length. Whole milliseconds, so a time and the same time a length
  later give exactly the same answer.
- `CinematicPlayer::new(ctx, id)` builds once what the shots need (each
  battle's whole map with its units at their starting places, painted by
  the map skin into one `GlyphBuffer`).
- `CinematicPlayer::frame(ctx, t, screen) -> Frame` says what is shown,
  as plain data; `draw(ctx, t, brightness, buf)` paints it.
- **Nothing is carried from one frame to the next.** No shot may keep
  state that `update` advances; whatever moves in a shot (a pan, sparks,
  later a walking unit) is worked out from `t`. That is what lets the
  owner of the clock follow the music, jump and loop, and lets a test
  draw any moment. The player has no clock: the title (0819) gives it the
  music's, the debug tool its own.

### 3. The shots

- `MapPan`: the battle's picture as a backdrop over the whole frame
  (ADR-0048), its centre going from `from` to `to` by an ease-in-out of
  `progress`, placed by `cinema::view`. The picture is painted by
  `ctx.map_skin` **at the art's own size** (`shot_skin`), so the shot's
  `zoom` is the only zoom whatever the player's battle-map zoom is.
- `Logo`: `title::glyph::paint_logo`, the title's own function, over the
  backdrop the owner gives (`with_backdrop`: the one the title picked for
  this launch). Its sparks' time is `t`.
- A new shot kind is a variant of `Shot` in `content` with its validator
  rules, a `Built` and a `Frame` variant in `ui`, a row in the README,
  and a shot in the test cinematic.

### 4. Brightness is one fade of the whole frame

`draw` takes `brightness` (0 to 1) and ends with
`GlyphBuffer::fade(brightness)`: every cell's colours and every
rectangle's colour scaled, every sprite's `opacity` scaled, and the
backdrop's scene replaced by a copy faded the same way. It is for the
title's menu fading into the cinematic (0819); the file can't ask for it.

## Consequences

- 0820 and 1010 write the title's cinematic as data; 0818 and 0827 add
  shot kinds along the path in §3.
- A cinematic that names a test battle can only be a test file. The
  title's stand-in (0819) has to pan over a story battle.
- A sprite fades by its opacity, so where two sprites overlap in a scene
  (a unit on its tile) the lower one shows through a little during a
  fade. Unseen in a short fade; a brightness applied by the renderers
  would be the fix if it ever matters.
- Fading a pan copies the map picture each frame. A Chapter 1 map is a
  few hundred cells, and the fade lasts a moment.
- The player's pictures are painted when it is made: a change of map
  skin, colour theme or lead after that isn't followed until it is made
  again.
- Shots can't cross-fade or overlap. Nick ruled fades between shots out.

## Alternatives considered

- **Shots with a `duration` each.** Changing one shot's length would
  shift every later one off the music. Start times are what a storyboard
  timed against a song gives.
- **A player with its own clock and per-shot state** (`update(dt)`).
  Simple for a pan, but it can't jump to the song's silence or stay in
  step with a track that loads late, and snapshot tests would have to
  play up to the moment they want.
- **Floats in the definition.** `Content` would lose `Eq`, which its
  tests use.
- **`length_ms` in the file.** A second copy of the track's length, to
  drift from `audio.ron`.
- **Points in pixels.** They would mean another place under each skin
  and zoom.
- **A brightness field on `Backdrop`, applied by the renderers.** Three
  renderers (`app`, `frame-png`, snapshots) to change for one fade; the
  copy needs none.
- **Painting the map with the player's map zoom.** A shot at `zoom: 2`
  would be twice as close for a player at 2× as for one at 1×.
