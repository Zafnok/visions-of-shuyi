---
id: "0833"
title: "Title cinematic: shorter talk, units move like the game, a combat scene"
type: content
milestone: M7 Chapter 1 & game flow
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0832", "0728"]
nick_input: sign-off
completed:
---

# 0833 — Title cinematic: moves like the game

## Context

Nick, Chapter 1 playtest round 2 (0804, 2026-10-14), on the intro cinematic:

> - "the dialogue scene with Tamsin and Hollis hangs uncomfortably long, it
>   should only be like 1/3 its current length."
> - "the battle scene has walkers moving at speeds that are not correct to
>   reality, and multiple units walking at same time. This is not the how
>   the game plays so it looks unnatural"
> - "the combat scene (2 battlers) is never shown, only some
>   in-battle-map fighting which also looks a bit odd."

`assets/cinematics/title.ron` (ticket 0820; storyboard and as-built table
in `docs/design/title-screen.md`):

- Shot 4, `Talk(scene: "ch01_prebattle", first: 0, count: 1)` (Tamsin's news
  to Hollis), runs from 50.0 s to 68.5 s: **18.5 s** for one box. A third is
  about 6 s.
- The `field` action's walks take 1 to 9 seconds over 1 to 9 tiles (about
  1 tile a second), and several units walk at once. In the game a unit
  walks at `WALK_TILES_PER_S` = 12 tiles a second
  (`crates/ui/src/screens/battle/walk.rs`), **one unit at a time**, and
  every move is followed by its action.
- Fights are only `Strike`s on the map; the combat scene (0413) is never
  shown. 0832 adds a shot that plays it.

The cuts sit on the song's changes (measured in 0820; listed at the top of
`title.ron`). 0728 rewrites the Chapter 1 script, so the `Talk` shots must
show its new lines.

## Nick input

**Sign-off:** watch the title cinematic (leave the title screen idle). Is
the talk short enough? Do the units move like the game? Does the combat
scene look right?

## Scope

**In:**
- Shot 4 cut to about 6 s; the freed time filled with another shot (a
  combat scene from 0832, or a longer march), with the cuts still on the
  song's changes (move a cut to a nearby measured change, not to an
  arbitrary time).
- The march and fights rewritten as turns: one unit walks at a time at the
  game's walk speed (`secs` = tiles ÷ 12), then acts; party moves first,
  then enemies, as in a phase. Pauses between units are allowed (the game
  has the player's think time).
- At least one fight shown with the combat-scene shot (0832), e.g. the lead
  and the brigand on the road (today's shot 6).
- The `Talk` shots checked against 0728's new script.
- `title-screen.md` (*As built*) updated.
- A test that every `Walk` in `title.ron` moves at the game's walk speed and
  no two walks overlap in time (so this can't come back).

**Out (do not do):**
- Changing the song, the logo shot or the player code (0832 does the
  combat shot).

## Implementation steps

1. Read `title.ron`, `title-screen.md` and 0820's completion notes (the
   measured song changes).
2. Re-time shot 4; re-plan the field action as turns; add the combat shot.
3. Add the speed / overlap test (in the cinematic's content tests or
   `crates/ui/src/cinematic/tests.rs`).
4. Watch it with `cargo run -p trpg-app --features private-assets` and
   look at frames with `frame-png`.

## Acceptance criteria

- [ ] Shot 4 lasts 5 to 7 s.
- [ ] Test: every walk moves at `WALK_TILES_PER_S` (within rounding) and no
      two walks overlap.
- [ ] The cinematic has at least one combat-scene shot.
- [ ] Every cut is on a measured change of the song (listed in the file's
      comment).
- [ ] `title-screen.md` *As built* matches.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The speed / overlap test; existing cinematic tests and snapshots.

## Completion notes

*(Filled in by the session that completes the ticket.)*
