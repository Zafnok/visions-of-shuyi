---
id: "0833"
title: "Title cinematic: shorter talk, units move like the game, a combat scene"
type: content
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0832", "0728"]
nick_input: sign-off
completed: 2026-10-14
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

- [x] Shot 4 lasts 5 to 7 s.
- [x] Test: every walk moves at `WALK_TILES_PER_S` (within rounding) and no
      two walks overlap.
- [x] The cinematic has at least one combat-scene shot.
- [x] Every cut is on a measured change of the song (listed in the file's
      comment).
- [x] `title-screen.md` *As built* matches.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The speed / overlap test; existing cinematic tests and snapshots.

## Completion notes

**Done 2026-10-14.** Only content, tests and docs changed; no player or
game code.

**What changed in `assets/cinematics/title.ron`** (twelve shots now):

| # | Shot | Starts |
| - | ---- | ------ |
| 1–3 | Party pan, `ch01_intro` captions, six faces | unchanged |
| 4 | `ch01_prebattle`, Tamsin's news, 1 box | 0:50, now **6 s** (was 18.5) |
| 5 | Turn 1 on the map, 3× | 0:56 |
| 6 | **Combat scene**: the brigand attacks the lead, falls to his counter | 1:08.5 |
| 7 | Turn 2 on the map, 3× (pans from the road to the footbridge) | 1:14.5 |
| 8 | **Combat scene**: Tamsin attacks the footbridge's brigand, he falls | 1:25.25 |
| 9 | Aske crosses the footbridge, 4× | 1:31.25 |
| 10 | Turn 3, the road, 3× | 1:35.5 |
| 11 | The whole field, 2× | 1:44.5 |
| 12 | The logo | 1:57 |

- **Cuts on the song.** The song's loudness was measured again (a throwaway
  program, not in the repo). The new cuts: 56.0 (a note after a hush, six
  seconds into the 50.0 phrase), 74.5 (a beat), 91.25 (an accent in the
  loud part). The others are 0820's. All are listed at the top of the file.
- **Turns.** One unit walks at a time, then acts (attacks or waits); the
  party's phase, then the enemy's. Moves fall on the song's beats, about
  1.5 s apart. The freed 12.5 s of the talk went to the battle.
- **Combat scenes.** Every fight is now the combat scene (the game's
  default with combat animations on); the map's `Strike`s only happen
  while a combat shot is up, to leave the map's HP bars as the fight did.
  The rules give: brigand vs. lead with seed 10, the lead left 10/19 and
  the brigand falls (a critical counter); Tamsin vs. the footbridge
  brigand on the battle's own seed, Tamsin left 13/21 and he falls. Each
  fight takes 4.1 s of its 6 s shot.
- The 0728 script still opens `ch01_prebattle` with Tamsin's three-line
  news and `ch01_intro` with the three captions, before any choice; both
  shots show its current lines.
- **Tests:** `the_title_walks_one_unit_at_a_time_at_the_games_speed` and
  `the_titles_fights_end_before_their_cuts` (`crates/ui/src/cinematic/
  tests.rs`); the title's shot-by-shot test split into the storyboard
  (shots 2–4, 12, 1) and `the_battle_plays_as_turns_with_the_combat_scene`
  (shots 5–11), with new snapshots.
- `title-screen.md` *As built* and `assets/cinematics/README.md` updated.
- Watched as frames (`frame-png`), in the glyph look and with the bought
  art; not watched with sound.

**Deviations**

- **6 tiles a second, not 12.** `WALK_TILES_PER_S` (12) is the glyph
  look's speed. Every build Nick and players get has the bought art, and
  there a unit walks at `SPRITE_WALK_TILES_PER_S` = 6 tiles a second
  (Nick, 2026-10-04: 5 was too slow). The cinematic's walks are timed to
  that, and the test checks against it. In a build without the art the
  walks look half as fast as that build's battles.
- **The footbridge fight is Tamsin alone** (Aske follows her across).
  The storyboard has "two party units bring down a second enemy", but a
  combat scene always starts from the battle's starting HP: a second
  fight on the same brigand would show him unhurt. The same is why the
  brigand on the road attacks the lead (rather than the lead attacking
  first and Hollis finishing him).
- **Staging.** A combat shot puts its attacker on a tile of its own, but
  the target fights from where the battle starts. So the brigand's attack
  is staged at the lead's starting tile; both tiles are road, so the
  fight's numbers and backdrop are the same as where they stand on the map.
- Enemies that don't matter to the shots don't move in their phase (the
  second brigand south of the river, the guards), as in 0820.

**the agent's starting rules** (Nick can veto): who moves where each turn,
the order they move in, who fights whom, and the seed that makes the
brigand fall to the lead's counter; Tamsin alone at the footbridge.

**Follow-up tickets:** none.

**For Nick when he plays** (the title, left idle, sound on): is the talk
short enough at 6 seconds? Do the units move like the game? Do the two
combat scenes look right, and is holding the end of each fight for about
2 seconds OK?
