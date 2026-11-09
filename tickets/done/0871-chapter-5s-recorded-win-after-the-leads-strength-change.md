---
id: "0871"
title: Chapter 5's recorded win after the lead's strength change
type: bug
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-09
---

# 0871 — Chapter 5's recorded win after the lead's strength change

## Context

On `main` at `c6ee97b8`, `cargo test -p trpg-ui --test it ch05_winnable`
fails: `chapter_5_plays_from_new_game_to_to_be_continued` stops at
`crates/ui/tests/it/playthrough.rs:90` with the battle not won after the
recorded commands (`left: None`, `right: Some(Victory)`). Found while
working 1251, whose change doesn't touch it: the test fails the same way
with `main`'s own sources.

Very likely cause: PR #587 (0846, the lead's Str 7 to 8 in Chapter 1)
and PR #588 (0851, Chapter 5's battle and its recorded win,
`crates/ui/tests/it/fixtures/ch05_winning.ron`) were each green alone and
merged one after the other. Chapter 5's playthrough starts from the army
Chapters 1 to 4's recorded playthroughs leave, so a stronger lead changes
the fights the fixture was recorded against. `main`'s CI runs for both
merges were cancelled, so nothing caught it.

## Nick input

None. Verified by the test; nothing for the Act 1 playtest.

## Scope

**In:**
- Find which merge broke it (run the test at `60bc27dd` and at
  `305687bb`), and where the recorded commands stop matching (the first
  command the battle refuses, or the turn the outcome differs).
- Record Chapter 5's winning playthrough again, as `ch05_winnable.rs`'s
  doc comment says (`cargo test -p trpg-ui --test it
  ch05_winnable::record -- --ignored`), and read the new fixture's last
  lines: nobody lost, the house held until first light, the turn count
  close to 0851's.
- If Chapter 5 can no longer be won with nobody lost, say so in the
  completion notes with what changed, instead of loosening the test.

**Out (do not do):**
- Any balance change to Chapter 5 or to the lead (0846's numbers stay).
- Chapters 1 to 4's fixtures, unless their own tests fail too.

## Implementation steps

1. Run `cargo test -p trpg-ui --test it ch05_winnable` on `main`; confirm
   the failure. Bisect between `60bc27dd` and `c6ee97b8` to name the
   merge.
2. Re-record `crates/ui/tests/it/fixtures/ch05_winning.ron` with the
   ignored `record` test, and read its result lines.
3. Run the whole `ch0*_winnable` set and the `run-gates` skill.

## Acceptance criteria

- [x] `cargo test -p trpg-ui --test it ch05_winnable` passes on the
      branch.
- [x] The completion notes name the merge that broke it and the new
      fixture's result (who fell, how many turns).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: `ch05_winnable` with the re-recorded fixture.

## Completion notes

**The merge that broke it: #587 (0846, `305687bb`).** With 0846's two
data files (`characters.ron`, `ch04_winning.ron`) put back as they were
at `8ca9afce`, `ch05_winnable` passes on `main`. #588 (0851) was green
on its own.

**Why, and the deviation from step 2: the fixture was not re-recorded,
because it still wins.** 0846 made the lead stronger and re-recorded
Chapter 4's win, so Chapter 5 starts with a different army. Played side
by side, the rules alone and the real game agree on every command, and
the fixture's commands still win (`the_recorded_playthrough_holds_the_house_till_light`
passed on `main` the whole time). What changed is *how* they win:

- before 0846: all nine Hounds fall by the player's last command, so
  the battle ended as the player ended turn 6;
- after 0846: one Hound is still standing at that point, and the win
  comes as turn 6's last phase ends: the hold till first light, the
  battle's own objective.

`playthrough::play_in_game` stopped right after the last recorded
command, so it only worked for a battle won on the player's own move.
`playthrough::play` (the rules-only playback) and the recorder both play
the others' phases after the last command. `play_in_game` now does too
(`play_others_in_game`, the same loop it already ran before each
command). A fresh recording would have just been a different
playthrough that hit the same gap, so the fixture stays as it is.

**The fixture's result now:** Victory as turn 6 ends, nobody lost (all
eight, Hedda included, reach "To be continued"); one Hound left alive
when the house is held. Turn count unchanged from 0851 (6).

No balance change. All `ch0*_winnable` tests pass, through the rules and
through the game. Nothing for the Act 1 playtest.
