---
id: "0734"
title: A short moving opening before a conversation, where the script asks
type: feature
milestone: M6 Story & dialogue
model: opus-5.5
effort: high
status: done
blocked_by: ["0732", "0733"]
nick_input: sign-off
completed: 2026-10-16
---

# 0734 — A short moving opening before a conversation, where the script asks

## Context

Nick's idea (0804, round 2), decided in 0029 (`docs/design/look-and-feel.md`,
*Places behind conversations*): "like even just Dace pacing a castle ...
and then flip to the conversation portraits with the background ...
darkened". A scene's script may ask for a short moving opening: the
characters move in the place at full brightness, then the place darkens
and the first line comes up. **Only where the script asks.** Nick found
the mockup's opening (3 s of movement, 0.5 s darkening) twice too long.

The title cinematic already moves units on a map: `MapPan` shots with
named `actions` of `Walk`/`Strike` (`crates/content/src/cinematic.rs`,
`cinematic/action.rs`; `assets/cinematics/title.ron`), drawn by
`crates/ui/src/cinematic.rs`. Today a `MapPan` only takes a battle
(`battle: "ch01"`, its units from the battle's setup). 0732 draws a place
(any `assets/maps/*.map`) behind a conversation; 0733 makes Chapter 1's
yard and study.

## Nick input

**Sign-off:** play Chapter 1 and look at: the places behind the talk, how dark they are, the two openings
(the lead and Hollis sparring; Dace pacing) and their length. Then the
four scene music picks (`docs/design/audio.md`, *Chapter 1's cues*),
which wait for this (Nick: "I want to see how the cutscenes would play out
with this ambience").

## Scope

**In:**
- A `.dlg` directive before a scene's first line: `@opening <cinematic id>
  <action>` naming an action in an `assets/cinematics/*.ron` file (or a
  new small format if that fits better; decide and record it). The
  action's actors are characters placed on the scene's place: the
  cinematic action file gains a way to place a character (by character
  id, drawn with their map sprite, or their glyph in the glyph look) at a
  tile of a non-battle map, and `Walk` moves them.
- Playback: the place at full brightness while the action plays, then a
  darkening over 0.5 s to 0.4 brightness (0732's constant), then the first
  line. Movement about **1.5 s** (*tunable*; Nick: half of the mockup's 3).
  Confirm skips to the first line; the skip-scene question still works.
- Facing: a walker faces the way they walk; a standing actor can be given
  a facing (sparring partners face each other).
- Chapter 1: `ch01_intro` opens on the lead and Hollis trading blows in
  the yard (each steps in and back); `ch01_tbc` on Dace pacing the study
  once. The female lead uses her own map sprite.
- The validator: the cinematic and action exist; every actor is a known
  character; the walk stays on the map.

**Out (do not do):**
- Openings for any other scene (the script adds them later, where it
  says something the words don't).
- Combat shots or strikes in openings.
- The title cinematic.

## Implementation steps

1. Read ADR-0060 (cinematics), ADR-0061 (walks and strikes in the map
   scene) and `assets/cinematics/README.md`, and decide how an opening
   names its place's actors; write it down there (a new ADR if it changes
   ADR-0061's format).
2. `crates/content`: the directive (parse, write back, check), and
   characters on a non-battle map in the action format.
3. `crates/ui`: the dialogue screen plays the opening before its first
   line, reusing the cinematic's map picture and walk code, then fades to
   0732's brightness. The opening is a view (actors' positions over time)
   painted by the map skins.
4. Chapter 1: the two openings in `assets/cinematics/` and their
   `@opening` lines in `ch01.dlg`.
5. Look at both in both map looks with `cargo xtask frame-png --wait`.

## Acceptance criteria

- [x] `ch01_intro` and `ch01_tbc` open with their moving openings, then
      the talk over the darkened place; other scenes have none.
- [x] The opening is about 1.5 s plus 0.5 s of darkening; Confirm skips it.
- [x] Bad openings fail `check-script` with the line number.
- [x] Works in the glyph look.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit (content): parse, write back, each validator error.
- Integration (ui, Harness): the scene's first line appears after the
  opening's length; a Confirm press during the opening shows the first
  line at once; actors' positions read from the scene, not cells.

## Completion notes

**Done.** `ch01_intro` opens on the lead and Hollis trading blows in the
yard at Harrowby, and `ch01_tbc` on Dace pacing the study at Veyne Hall;
then the place darkens and the talk comes up over it.

- **Format (ADR-0076)**: a small file of its own, `assets/openings/<id>.ron`
  (`assets/openings/README.md`): the map it plays on, its cast (character
  ids placed on tiles, each with an optional facing) and its walks, written
  as a cinematic action's entries (ADR-0061) and checked by the same code,
  now run on a `Stage` (a map and a cast) instead of a battle. No strikes.
  A speaker with no class (Dace) walks on foot. A walk gains an optional
  `facing` (stepping back while facing the opponent), honoured by the
  cinematics too.
- **Directive**: `@opening <id>` (`Step::Opening`), parsed, written back
  (round-trip property test), skipped by the passes over directives.
  Validator (`file:line` errors, through `check-script` too): only at a
  scene's top level, after a `@place <map>` (not `battle`/`none`), with a
  speech or narration line after it before the next one; the opening
  exists and is made for the map of the place in force; the opening file
  itself: id, map, cast (known characters, once each, on the map, on a
  tile they can stand on, not someone else's) and every walk rule.
- **Playback**: the dialogue screen plays it before the line after the
  `@opening`: the place at full brightness with the caption alone, the cast
  walking (`trpg_ui::cinematic::opening`: a pure function of the clock,
  reusing `action_state` and the walk gait), then 0.5 s of darkening to
  0.4 (`DARKEN_MS`, `PLACE_BRIGHTNESS`), then the line. Confirm (or End
  turn) goes straight to the line; Cancel asks the skip question; skipping
  the scene or jumping in the log passes over it. The cast is the place's
  map scene's units, painted by the map skins (their map picture, the
  lead's by the player's look; their label in the glyph look); a new
  `UnitView::marks` turns their HP bar and marks off.
- **Chapter 1**: `assets/openings/ch01_sparring.ron` (1.55 s: the lead
  steps in and back, Hollis steps in and back, the lead again; 0.2 s
  steps) and `ch01_pacing.ron` (1.45 s: across the rug and half way back),
  with their `@opening` lines in `ch01.dlg`.
- **Looked at** with `cargo xtask frame-png` (pictures with the bought art,
  and glyphs), both openings, mid-walk and with the talk after.
- **Tests**: content (parse, write back, every directive and file error,
  Chapter 1's openings), ui (the cast and its walks over time, the
  player's bookkeeping, the skin's bright-then-darkening frame) and
  `tests/it/openings.rs` (first line after the opening's length, Confirm
  and Cancel during it, positions read from the scene, the female lead, no
  opening elsewhere, a glyph snapshot). `places.rs`'s yard snapshot now
  shows the pair standing behind the first line.

**Deviations**

- A new small format rather than an action in a cinematic file (the
  ticket allowed it): a cinematic needs shots and a length an opening
  hasn't got (ADR-0076, *Alternatives*).
- The opening plays where the `@opening` line is, before the next line,
  not only before a scene's very first line: `ch01_intro`'s three captions
  stay on black (0733) and the opening comes with the yard.
- `cargo xtask frame-png` has no `--scene`; `ch01_tbc` was rendered with a
  temporary local hook, not committed.

**Follow-up**: 0735, Dace's own map picture (he has none: the picture
look shows the tileset's generic fallback for him; Nick picks one).

**For Nick** (sign-off): play Chapter 1 from New Game and look at the
yard after the three captions, and the study after the battle: the places
behind the talk, how dark they are, the two openings and their length.
Then the four scene music picks (`docs/design/audio.md`, *Chapter 1's
cues*), which waited for this.

*Claude's starting rules (Nick can veto):*

- After an opening its characters stay on the place where they stopped,
  darkened with it behind the talk, until the scene changes place.
- During an opening the place line (caption) stays at the top; the
  portraits and the text box wait.
- The people in an opening are on no side: in the glyph look their
  labels are in the neutral colour, and nobody has an HP bar.
- The sparring is three exchanges of 0.2 s steps (the lead, Hollis, the
  lead), each stepping back facing the other; Dace crosses the rug and
  stops half way back, facing the way he last walked.
- Cancel during an opening ends it and asks "Skip scene?"; answering no
  shows the first line.
