# ADR-0076: Conversation openings: a cast on a place, and the cinematic's walks

- **Status:** Accepted
- **Date:** 2026-10-16
- **Related tickets:** 0734, 0029, 0732, 0733
- **Extends:** ADR-0061 (cinematic action: walks and strikes in the map
  scene), adding `facing` to a walk
- **Follows:** ADR-0038 (graphics are a skin), ADR-0054 (a screen's view
  and its skin), ADR-0060 (a frame from the time alone)

## Context

- Nick decided (0029, `docs/design/look-and-feel.md`, *Places behind
  conversations*) that a scene may open, **only where the script asks**,
  on its characters moving in its place at full brightness (the lead and
  Hollis sparring; Dace pacing), then the place darkens and the talk
  comes up. About 1.5 s of movement and 0.5 s of darkening.
- 0732 draws a conversation over its place (`@place <map> <x>,<y>`, a map
  of `assets/maps/`); 0733 made Chapter 1's yard and study.
- The title cinematic already moves units on a map: a `MapPan` shot's
  `action`, a list of `Walk`s and `Strike`s checked by
  `trpg_content::cinematic::action` and turned into the map scene's units
  by `trpg_ui::cinematic::action_state`, from the action's clock alone
  (ADR-0061). But a `MapPan` only shows a battle, and its actors are the
  battle's units where the battle file puts them.
- A cinematic file needs a length and at least one shot, and every named
  action list must be used by a shot. A place's map has no units, and some
  of the people a scene shows (Dace, the messenger) are speakers with no
  class or stats.
- Sparring partners step back while facing each other; ADR-0061's walker
  always faces the way it goes, and the camera once it stops.

## Decision

### 1. A small file format of its own: `assets/openings/<id>.ron`

```ron
(
    id: "ch01_sparring",
    map: "harrowby_yard",
    cast: [(who: "lead", at: (11, 8), facing: Right), (who: "retainer", at: (13, 8), facing: Left)],
    action: [(at: 0.0, do: Walk(unit: "lead", path: [(11, 8), (12, 8)], secs: 0.2)), …],
)
```

The ticket offered "an action in an `assets/cinematics/*.ron` file, or a
new small format if that fits better". An opening is not a cinematic: it
has no shots, music or length of its own, so putting it in a cinematic
file would mean a cinematic with no shots, or a dummy one. The file names
its own **map** and places its own **cast** (by character id, each with a
tile and an optional facing) on it; its **action** is ADR-0061's entries
as written, so the format, the checks and the playback are shared. The
format is described in `assets/openings/README.md`.

- `@opening <id>` in a `.dlg` (`Step::Opening`) plays it before the next
  text box. One argument; the directive's writer, parser and round trip
  are the other directives'.
- The opening names its map rather than taking the `@place` line's: each
  file is checked on its own when it loads, and the script's check is
  then only "the place in force is that map".

### 2. One checker for both: an action on a `Stage`

`action::check` now reads a list on a `Stage`: a map, a cast by name (each
with how it walks: a movement type and its name in a message), where
someone missing isn't, and whether strikes are allowed. A battle's stage
is its map and units as before (`Stage::battle`: every message is
unchanged); an opening's is the place's map and its cast, with **no
strikes** (out of 0734's scope: an opening shows people, not a fight). A
speaker without a class walks **on foot** (the terrain table's `foot`
movement type), so nobody walks through a wall or a desk.

The loader reports, with the file: the id, the map, an empty cast, a
cast member who isn't a character, is in it twice, stands off the map,
where they can't stand, or where someone else stands, and every walk
problem ADR-0061 lists. The dialogue checks report, with the `.dlg` line:
an `@opening` inside a reply or an `@if` block, without a `@place <map>`
in force, with nothing said after it, an unknown opening, or one made for
another map (`check_scene` for where it goes; `check_places`, after the
openings load, for the file and its map).

### 3. `facing` on a walk

`Walk(…, facing: Left)` keeps the walker facing that way all along and
after: ADR-0061's format gains an optional field, so every existing
action is unchanged, and `action_state` honours it for cinematics too.

### 4. Playback: the dialogue screen, a view, the map skins

- `DialoguePlayer` keeps the opening in force (until the next `@place`)
  for the view, and the one just reached for the screen to take, like
  `@music`. Skipping the scene, or jumping in the log, drops a due
  opening: it doesn't play.
- `trpg_ui::cinematic::opening` builds the cast as the map scene's units
  (`cast_units`: by character, so a skin with pictures draws their map
  picture, the lead's by the player's look; labels for the glyph look;
  no HP bar or marks) and puts them where the walks have them at a time
  (`opening_units`: `action_state`, then each walker keeps facing the way
  it last went). A pure function of the clock (ADR-0060 §2).
- `DialogueScreen` times it; the view's place carries the cast as its
  units (during the opening, and after it, where it left them, until the
  place changes), and `DialogueView::opening` says how far the place has
  darkened. The dialogue skin paints the place with the map skin at full
  brightness under the caption alone, fading to `PLACE_BRIGHTNESS` as it
  darkens; then the talk as before. Nothing about the look is in
  `content` or `core` (ADR-0038).
- `UnitView::marks` (new) turns off a unit's HP bar and effect marks; both
  skins honour it and the sprite skin's every-feature test lists it.
- Confirm (or End turn) ends the opening and shows the line; Cancel ends
  it and opens the skip question. The text waits while it plays.

## Consequences

- An opening is a few lines of data; Chapter 1 has two
  (`ch01_sparring`, `ch01_pacing`). Moving a place's furniture under a
  walk, or renaming a character, fails the content check.
- Its people have no side here: they are drawn in the neutral colour of
  the glyph look, with no HP bar. A speaker with no map picture of their
  own gets the sprite skin's fallback picture until the tileset names one
  for their character.
- The movement and its pace are authored (`secs`), not the battle map's
  walking speed: a pace that reads well in a 1.5 s opening is the
  author's to tune.
- Openings can't strike, change terrain or move the camera; each would be
  a new kind of entry or field, as ADR-0061 said of cinematics.
- The opening plays only before the line after the `@opening`; a scene
  that moves to another place can have another opening there.

## Alternatives considered

- **`@opening <cinematic> <action>`, an action list in a cinematic file**
  (the ticket's sketch). Needs a cinematic with no shots (or a dummy shot
  and length), a way to place characters on a non-battle map inside a
  format built around battles, and an exception to "every named list is
  used by a shot". The shared part, the walk entries, is shared anyway.
- **The opening takes its map from the `@place` line.** Then an opening
  file can't be checked alone, and its walks would be checked once per
  scene that uses it, reported against the script.
- **A cinematic shot played full-screen before the scene.** A hard cut to
  another screen and back; the design is the same place, darkening into
  the talk.
- **Walkers turn to the camera when they stop** (ADR-0061's rule). Two
  sparring partners would face the player between blows.
- **Strikes in openings.** Out of scope (0734): no combat in openings.
