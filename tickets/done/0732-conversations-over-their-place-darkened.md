---
id: "0732"
title: Draw each conversation over its place, darkened
type: feature
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
nick_input: none
completed: 2026-10-15
---

# 0732 — Draw each conversation over its place, darkened

## Context

Nick decided (ticket 0029, `docs/design/look-and-feel.md`, *Places behind
conversations*) that conversations stop being drawn on black: the talk is
drawn over its **place**, darkened. Away from a battle the place is a
top-down map laid out from the tilesets (Chapter 1's two are made by
0733); just before, during or just after a battle it is the battle map
itself. With *Map look: Glyphs* and on the Playdate the place is drawn in
glyphs. A short moving opening before some scenes is 0734.

How it works today (found while writing this ticket):

- `.dlg` directives: `Step` in `crates/content/src/dialogue.rs` (`Caption`,
  `Music`…), parsed by `directive()` in `dialogue/parse.rs`, checked in
  `dialogue/check.rs` (`check_scene`). The writer's reference is
  `assets/dialogue/README.md`.
- The dialogue screen has a view and a skin (ADR-0054): `DialogueView` in
  `crates/ui/src/screens/dialogue/view.rs`, painted by
  `screens/dialogue/glyph.rs`, which fills the screen with black (a
  full-screen scene) or only the rows under the text box (an overlay).
- Scenes during a battle are overlays (`DialogueScreen::told_overlay`, from
  `screens/battle/mod.rs` `next_scene`): the battle map shows around the
  talk, **not darkened**. A chapter's intro and victory scenes are played
  full-screen on black by `flow.rs` (`play`).
- A map is plain data (`MapScene`, `map_view/scene.rs`) painted by a map
  skin (`GlyphSkin`, `SpriteSkin`; `skin_for_look` in `map_view.rs`). Any
  `assets/maps/*.map` is a terrain-only map in `ctx.content.maps`.
  `GlyphBuffer::fade` darkens a whole frame, sprites and backdrop included
  (ADR-0060).
- The title cinematic's `MapPicture` (`crates/ui/src/cinematic.rs`) already
  draws a battle map as a backdrop behind other things.

## Nick input

None. (His sign-off on the look comes with 0734, when Chapter 1's scenes
have their places and openings.)

## Scope

**In:**
- A `.dlg` directive naming the scene's place, in force until the next
  one: `@place <map id> <x>,<y>` (a map in `assets/maps/`, centred on that
  tile), and `@place battle <x>,<y>` (the battle map of the battle this
  scene belongs to: the chapter's battle for its intro and victory scenes,
  the battle being played for a trigger scene). `@place none` goes back to
  black. A scene with no `@place` is unchanged (black, or, during a battle,
  the map as today but darkened, below).
- The validator: the map exists, the tile is on it, `battle` is only used
  in a scene that has a battle (a chapter's intro/victory scene or a
  battle trigger). Same `file:line: message` errors as the other
  directives.
- The place is drawn behind the talk at **0.4 brightness** (a named
  constant, *tunable*), through the map skins: the player's *Map look*
  picks the glyph or the sprite skin, as on the battle map. Nothing about
  the place's look goes in `core`, the bots or play records (ADR-0038).
  The place's view is plain data in `DialogueView` (the map, the centre),
  painted by the skin (ADR-0054); the dialogue skin keeps painting
  portraits, plates, box and caption over it.
- **During a battle**, a trigger scene (always an overlay) darkens the
  battle map behind it to the same brightness, units as they stand,
  whether or not the scene names a place.
- The place line (`@caption`) is unchanged.
- The dialogue log (0729) and the skip question show over the place as
  they show over black today.
- Chapter 1's script: `@place battle …` in `ch01_prebattle` (Harrowby, by
  the chapel) and `ch01_victory` (the toll-fort), with the tiles read from
  `assets/maps/ch01.map`. `ch01_intro` and `ch01_tbc` wait for 0733's
  places.
- The writer's reference (`assets/dialogue/README.md`, directives table)
  and `docs/story/writers-guide.md` if it lists directives.

**Out (do not do):**
- Chapter 1's own places (the yard, Veyne Hall): 0733.
- The moving opening: 0734.
- The Playdate's dialogue screen (1043 adds the place there).
- Bought painted backdrops behind talk (Nick: no).
- Lighting the place by its time of day: 0733 decides it per place, if the
  sprite skin's lighting (ADR-0058) can be named for a map.

## Implementation steps

1. `crates/content`: add `Step::Place(PlaceLine)` with `PlaceLine::{Map
   { map, at }, Battle { at }, None}`; parse `@place` in `directive()`;
   serialize it back (the `.dlg` writer next to `Caption`); skip it in the
   passes that skip directives (`presence.rs`, `line_id.rs`) and in the
   translation data if captions are listed there.
2. `check.rs`: the map exists in `refs.maps`, `at` is inside it; `battle`
   only where the scene has a battle. To know that, pass the scene's
   context in (the chapter files' `intro_scenes`/`victory_scenes`, the
   battle files' triggers); if that's not available to the check, check
   it in the all-assets test that loads chapters and battles.
3. `crates/ui/src/dialogue.rs` (the player): keep the current place like
   the caption, and expose it in the view (`DialogueView.place:
   Option<PlaceView>`, where `PlaceView` names the map and the centre in
   tiles). Jumping in the dialogue log restores the place the line had.
4. A `PlaceView` → `MapScene` builder (terrain only, no units, no cursor),
   in `map_view` next to the cinematic's `battle_scene`, and paint it
   with `skin_for_look(ctx's map look)` into the frame, then darken that
   region to `PLACE_BRIGHTNESS = 0.4`. Prefer a darkening that also
   darkens sprite items (as `GlyphBuffer::fade` does) over cell colours
   only. The dialogue's glyph skin then stops clearing to black where the
   place is.
5. `@place battle`: the flow (`flow.rs` `play`) gives the dialogue screen
   the chapter's battle map; a trigger scene in a battle gets the battle
   screen's own scene (units included) via the overlay path.
6. Battle overlays with no `@place`: darken the battle map drawn under the
   overlay (`ScreenStack::draw` draws the battle first) to the same
   brightness.
7. `ch01.dlg`: add the two `@place battle` lines. Read the scenes in the
   game (`cargo run -p trpg-app --features private-assets -- --scene
   ch01_prebattle`), both map looks.
8. Docs: the directive in `assets/dialogue/README.md`; an ADR only if the
   place's view/skin split differs from ADR-0054's pattern.

## Acceptance criteria

- [x] `@place` parses, round-trips through the `.dlg` writer, and a wrong
      map, an off-map tile or a misplaced `battle` fails `check-script`
      with the line number.
- [x] `ch01_prebattle` and `ch01_victory` show the battle map, darkened,
      behind the talk, at Harrowby and at the toll-fort; in the glyph look
      too.
- [x] A battle trigger scene (for example Harl's lines) shows the battle
      map behind it darkened.
- [x] A scene with no place is drawn exactly as before (existing snapshots
      unchanged).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit (content): parse, write back, each validator error.
- Unit (ui): the player's view carries the place and restores it on a log
  jump.
- Snapshot / integration: a scene with `@place` in the glyph look (the
  place's cells under the box); a battle trigger scene's darkened map. A
  test of what is shown reads the view or the scene; only the skin's
  look test reads cells and colours.

## Completion notes

**Done.** Conversations are drawn over their place, darkened.

- **`.dlg`**: `@place <map> <x>,<y>`, `@place battle <x>,<y>` and
  `@place none` (`Step::Setting(PlaceLine)` in `crates/content`), parsed,
  written back by `print_scene` (in the round-trip property test) and
  skipped by the passes over directives (line ids, presence, the
  translation data). The writer's reference (`assets/dialogue/README.md`,
  a new *Places* section, last so the README's numbered examples keep
  their place) and `docs/story/writers-guide.md` describe it; both
  examples are checked by tests.
- **Validator**: the map is one of `assets/maps/`, the tile is on it,
  `battle` is only in a scene a battle plays (a chapter's intro or victory
  scene, a battle trigger's scene), not a support conversation, and its
  tile is on that battle's map. `file:line: message` errors, through
  `check-script` too. They run after the battles and chapters load
  (`dialogue::check_places`, `PlaceRefs`; `lib.rs` `check_dialogue`), as
  the step's fallback said.
- **Player and view**: `DialoguePlayer` keeps the place like the caption
  (`View.place`); the log keeps players, so a jump restores it.
  `DialogueView.place` is the part of the map the whole screen shows, as
  a `MapScene` (plain data), built from a `map_view::Place` (a whole map,
  its look and its units) centred on the tile with `Camera::centred_on`.
- **Look**: the dialogue's glyph skin paints the place with `ctx.map_skin`
  (the player's *Map look*, at their map zoom) over the whole screen,
  `GlyphBuffer::fade`s it to `PLACE_BRIGHTNESS = 0.4` (sprites and
  backdrop too), then paints caption, portraits, plates and box on top at
  full brightness. An overlay with no place fades the battle map below to
  the same brightness. The log and the skip question are unchanged.
- **Where the battle comes from**: `DialogueScreen::with_battle(Place)`.
  The flow gives a chapter's scenes its battle's field (terrain only);
  a battle's trigger scene gets the battle as it stands
  (`BattleScreen::place`, units as drawn); the `--scene` preview finds the
  scene's battle (`Content::battle_of_scene`).
- **Chapter 1**: `@place battle 3,12` in `ch01_prebattle` (the green by
  the chapel, where the party starts) and `@place battle 20,3` in
  `ch01_victory` (the toll-fort, below its keep at 20,2).
- **Tests**: content (parse, errors, write back, every validator error,
  each scene's battle from the real files), ui (the player's place, the
  view's place and the log jump, the battle's place for a trigger scene,
  the look: place cells under the box, the overlay darkened), and
  `tests/it/places.rs` (New Game to `ch01_prebattle` with a snapshot in
  the glyph look, `ch01_victory`, a scene over the darkened battle map).
  The full-screen scene snapshots are unchanged (no existing scene names
  a place). The three snapshots of a scene over the battle map (the
  debug overlay, the rogue's line and its last words) keep every glyph;
  only their colour legend changed, to the darkened map's colours (and
  the combat stage's sprites under the rogue's lines list `bright=102`
  where they listed `opacity=102`).

- **The fade itself (ADR-0075, amends ADR-0060 §4)**: rendering
  `ch01_prebattle` with the bought art (`cargo xtask frame-png` with
  `private-assets`) showed the place at nearly full brightness: `fade`
  lowered each sprite's opacity, and the bought tiles are stacked
  sprites (ground, then edges and corners), whose see-through layers add
  back up. Sprites now have a `brightness` that `fade` lowers, and every
  renderer (`app`, `frame-png`, the one-bit frame) multiplies the
  sprite's colour by it; snapshots show ` bright=N`. Measured on the
  render: the battle map's grass, 89,207,147, is 36,83,59 behind the
  talk, 40%. The title cinematic's fade gets the same fix.

**Deviations**

- The step is `Step::Setting`, not `Step::Place`: `Step::Place` is
  already the portrait step (`@left`/`@right`).
- `DialogueView.place` holds the map scene to paint, not a map id and a
  centre: the skin paints a `MapScene` as the battle map's skins do, and
  `@place battle` in a battle needs the battle's live units, which no id
  names. `DialogueView` is `PartialEq` but no longer `Eq` (a map scene
  has fractions). The view/skin split is ADR-0054's, so no ADR for it;
  the fade change has its own (ADR-0075).
- A reply's reaction and an `@if` block must leave the same place on
  every way through, as they must the caption (a validator rule for the
  writer, like the caption's).
- A cinematic's talk shot still draws on black (out of the ticket; 0734
  is the moving opening).

**For Nick**: in the glyph look the whole of Chapter 1's map fits on the
screen, so the place is the whole field, darkened, with the talk over it;
with *Pictures* (the bought tiles) the screen shows the part around the
spot. Try `ch01_prebattle` and `ch01_victory`, and a battle line (Harl's)
over the darkened battle.

*the agent's starting rules (Nick can veto):*

- When the spot is near the map's edge, the view stops at the edge
  rather than showing black past it (as the battle camera does); a map
  smaller than the screen sits in the middle with black around it.
- A scene played during a battle covers the battle's key-help row below
  the text box with black, as before; with a place named, the place
  fills the whole screen.
- The battle's field behind a chapter's scenes before and after the
  battle has nobody on it.
