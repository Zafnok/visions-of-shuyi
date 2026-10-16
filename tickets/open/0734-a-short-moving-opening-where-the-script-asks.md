---
id: "0734"
title: A short moving opening before a conversation, where the script asks
type: feature
milestone: M6 Story & dialogue
model: opus-5.5
effort: high
status: todo
blocked_by: ["0732", "0733"]
nick_input: sign-off
completed:
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

- [ ] `ch01_intro` and `ch01_tbc` open with their moving openings, then
      the talk over the darkened place; other scenes have none.
- [ ] The opening is about 1.5 s plus 0.5 s of darkening; Confirm skips it.
- [ ] Bad openings fail `check-script` with the line number.
- [ ] Works in the glyph look.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (content): parse, write back, each validator error.
- Integration (ui, Harness): the scene's first line appears after the
  opening's length; a Confirm press during the opening shows the first
  line at once; actors' positions read from the scene, not cells.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
