---
id: "0485"
title: "The end-of-lessons box asks only in Chapter 1"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0485 — The end-of-lessons box asks only in Chapter 1

## Context

Nick's playtest of the Act 1 build (2026-11-08):

> "Vix's final tutorial for terrain effects appears in ch3 seemingly and
> out of nowhere after completing just that tutorial then it asks if you
> wanna restart battle, and this feels weird, that option should probably
> only appear in ch1, even though Vix might have tutorials later. Probably
> determine some cutoff in ch1 that she asks it"

The box is ticket 0465's (done): heading "That's the lessons", answers
Keep going / Restart the battle / Restart with the lessons
(`assets/lang/en/ui.ron` ~:464–469). Code:

- `crates/ui/src/screens/battle/lessons.rs`: `END_ANSWERED` (~:25),
  `EndAnswer` (~:30), `LessonState::end_due` (~:115): due when the
  campaign has a tutorial level, hasn't answered the box, and every lesson
  that level shows (and that may start in this battle, `starts_in`) is
  done.
- `crates/ui/src/screens/battle/mod.rs`: `lesson_moment` (~:1046),
  `lessons_end_shown` (~:1091: the map idle, nothing queued, no tip due,
  `end_due`), `lessons_end_key` (~:1106).

**Why it came in Chapter 3.** Two lessons wait for their first chance:
`combat_arts` (`ArtsAvailable`) and `terrain_magic` ("Spells that change
tiles", `TileSpellAvailable`: a spell list that can aim at a tile;
`assets/data/lessons.ron` ~:159–181). If the player never opens Rue's spell
list next to a forest in Chapters 1 and 2, `terrain_magic` is the last
lesson left, it opens in Chapter 3, and closing it makes `end_due` true:
the box asks to restart a Chapter 3 battle. `docs/design/tutorial.md`
~:120–133 says so on purpose ("it can come on a later turn or in a later
battle").

**A second cause, same rule.** If a winning move ticks the last lesson,
the battle is won before the map is idle, so the box waits and comes at
the next battle's first idle moment.

Nick decided the direction: the box asks only in Chapter 1, at a cutoff.
Lessons themselves still show in later battles.

## Nick input

None: Nick set the direction. The exact cutoff is *the agent's starting
rule* below, written into `tutorial.md` with his words, and goes on the
Act 1 playtest's checklist (1064). Verified by the tests below.

**The agent's starting rule (the cutoff).** The box is asked only in
Chapter 1's battle. It comes at the first quiet moment (the map idle on
the player's phase, no lesson open or about to open, no tip due) once
**either** every lesson the tutorial level shows is done, **or** turn 3's
player phase has begun, whichever is first, whatever lessons are left.
Lessons not done yet still show in later battles when their chance comes,
but the box never comes again outside Chapter 1. If Chapter 1 is won
before then (the bots' fastest median is 8 turns, `docs/playtesting.md`
~:197), it is never asked.

Example: a player at Short finishes the keys, threat arrows and Combat
Arts lessons on turn 1 and Rewind on turn 2, but never opens Rue's spells
by a forest. Turn 3 begins; when the map is idle, Vix's box comes ("That's
the lessons"). In Chapter 3, Rue's spell list by a forest opens "Spells
that change tiles"; closing it asks nothing.

Why turn 3: the turn-1 lessons and Rewind (turn 2, `SecondTurn`) are done
by then, so the box doesn't cut in early, and every Chapter 1 run reaches
it.

## Scope

**In:**
- The cutoff as data in `assets/data/lessons.ron`: a top-level field, e.g.
  `end_box: (battle: "ch01", from_turn: 3)` (the battle where the box may
  come, and the turn from which it comes whatever is left). The loader
  (`crates/content/src/lesson.rs`) reads it; the header comment explains
  it; content validation checks the battle id exists. The test lessons
  (`assets-test/data/lessons.ron`) get their own value for the tests.
- `LessonState::end_due` takes the battle and the turn: false outside
  `end_box.battle`; true there when every shown lesson is done, or the
  turn is at least `from_turn` and no lesson is open or waiting to open
  this moment.
- `lessons_end_shown` keeps its other conditions (idle map, no tip, no
  queue); it also waits while a lesson is open (`lessons.is_open()`).
- `docs/design/tutorial.md` ~:120–133: Nick's words (2026-11-08) and the
  rule above, replacing "it can come on a later turn or in a later
  battle".

**Out (do not do):**
- The box's words, answers or look (0465's, Nick's).
- Moving the box onto the engine's question widget (1185; either order:
  if 1185 is done, change `end_due` where it lives then).
- When `terrain_magic` or `combat_arts` open (they still wait for their
  chance).
- Vix's arts lesson (0490) and the effectiveness tip (0489): either
  order, they add lessons/tips that this rule covers as it is.

## Implementation steps

1. Write the failing tests first (below): the box shows in a battle other
   than Chapter 1 today.
2. Add `end_box` to the lessons file, its type and loader, and its
   validation.
3. Change `end_due` (and its callers in `battle/mod.rs`) to the rule.
4. Update the tests that assumed the old rule (`lessons.rs` ~:455–506,
   `crates/ui/tests/it/lessons.rs`, `ch01_lessons.rs`) and the doc.

## Acceptance criteria

- [ ] In a battle other than `end_box.battle`, with every lesson done and
      the box unanswered, `lessons_end_shown` is false (unit test in
      `lessons.rs`, and a Harness test where the last lesson closes in a
      second battle: no box).
- [ ] In Chapter 1 at Short, with `terrain_magic` not done, the box shows
      at the first idle moment of turn 3 (Harness test in
      `crates/ui/tests/it/ch01_lessons.rs`).
- [ ] In Chapter 1, finishing every lesson on turns 1–2 still shows the box
      then (the existing acceptance tests keep passing:
      `the_box_comes_after_the_last_lesson_and_keep_going_closes_it`).
- [ ] The box doesn't show while a lesson is open on turn 3 (test).
- [ ] A lesson finished after the cutoff (e.g. `terrain_magic` in another
      battle) still closes and is kept; no box (test).
- [ ] `tutorial.md` states the rule with Nick's words.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `end_due` for each case (battle, turn, all done, open lesson) in
  `crates/ui/src/screens/battle/lessons.rs`.
- Integration: the Harness tests above (`ch01_lessons.rs`, `lessons.rs`).
- Content: the validation of `end_box`.

## Completion notes

*(Filled in by the session that completes the ticket.)*
