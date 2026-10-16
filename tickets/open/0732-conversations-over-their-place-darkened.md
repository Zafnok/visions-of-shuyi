---
id: "0732"
title: Draw each conversation over its place, darkened
type: feature
milestone: M6 Story & dialogue
model: opus-5.5
effort: high
status: todo
blocked_by: []
nick_input: none
completed:
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

- [ ] `@place` parses, round-trips through the `.dlg` writer, and a wrong
      map, an off-map tile or a misplaced `battle` fails `check-script`
      with the line number.
- [ ] `ch01_prebattle` and `ch01_victory` show the battle map, darkened,
      behind the talk, at Harrowby and at the toll-fort; in the glyph look
      too.
- [ ] A battle trigger scene (for example Harl's lines) shows the battle
      map behind it darkened.
- [ ] A scene with no place is drawn exactly as before (existing snapshots
      unchanged).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (content): parse, write back, each validator error.
- Unit (ui): the player's view carries the place and restores it on a log
  jump.
- Snapshot / integration: a scene with `@place` in the glyph look (the
  place's cells under the box); a battle trigger scene's darkened map. A
  test of what is shown reads the view or the scene; only the skin's
  look test reads cells and colours.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
