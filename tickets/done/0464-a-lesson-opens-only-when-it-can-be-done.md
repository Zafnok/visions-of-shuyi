---
id: "0464"
title: "A lesson opens only when it can be done that turn"
type: bug
milestone: M3 Battle UI
model: opus-5.5
effort: medium
status: done
blocked_by: []
nick_input: sign-off
completed: 2026-10-17
---

# 0464 — A lesson opens only when it can be done that turn

## Context

Checklist lessons (0456, 0457; `assets/data/lessons.ron`,
`crates/ui/src/screens/battle/lessons.rs`, `docs/design/tutorial.md`).
Chapter 1 playtest round 3 (0804, 2026-10-16), at Not familiar (Full):

> "so the Arts lesson coming right after the Attack lesson is where the
> tips first start feeling disjointed since Attack feels forced and
> probably is a shit move 1. For instance, I moved Tamsin to attack a
> bandit, now the tip says open arts, but this is impossible with every
> other unit on turn 1 since they're all out of range. It probably
> stresses out a new player. Probably if the tip cannot be achieved then
> it shouldn't yet show."

And on turn 1's load: "just make sure the tips are actually achievable on
the turn they present themselves otherwise it's ok".

What happens: Tamsin's action menu had usable Arts (`ArtsAvailable`), so
the Combat Arts lesson was started while the Attacking lesson was open,
and waited. When Attacking closed (Tamsin attacked), Combat Arts opened,
but Tamsin had acted and no other unit could reach an enemy. Units were
still left to act, so the "put away" rule (no unit left) didn't apply,
and the turn was held by a lesson nobody could finish.

**Comes before the Playdate playtest.** Round 3's tickets wait for the
carve (1118) and the Playdate port (1050), Nick's priority. This one is
an exception he chose (2026-10-16): the Playdate runs the same tutorial
code, so 1050 waits for it, and it waits for nothing (the carve doesn't
move the battle screen's tutorial code). The engine carve is still the
critical path (`docs/ROADMAP.md`); this is a small fix to fit in beside it.

## Nick input

**Sign-off:** play turn 1 at Not familiar as he did (Tamsin attacks
first); the Arts lesson doesn't open until a unit can use an art.

## Scope

**In:**
- A waiting lesson **re-checks its trigger when its turn to open comes**.
  If what started it is no longer true (the unit whose menu had Arts has
  acted; the spell list with a tile spell is closed and that unit has
  acted; no enemy is in reach any more), it doesn't open; it waits for its
  trigger to fire again, in this turn or a later one.
- An open lesson that needs a unit to act is **put away as soon as no
  unit left to act can do it this turn** (not only when no unit is left
  at all). "Can do it" per lesson: Attacking, some unit left could reach
  an enemy (the same reach the danger zone and threat arrows use); Combat
  Arts, some unit left has an art usable from a tile it can reach; Spells
  that change tiles, some unit left could aim a tile spell; Moving and
  Threat arrows, any unit left. *Claude's starting rule*; say so at
  sign-off.
- Lessons whose items need nothing of a unit (cursor, keys, next unit,
  ending the turn, rewind) are unchanged.
- Update the comment at the top of `lessons.ron` and `tutorial.md`.

**Out (do not do):**
- The order of the lessons, their text, their levels.
- What happens when all lessons are done (0465).

## Implementation steps

1. Write the failing test first: Chapter 1, Full, turn 1. Select Tamsin,
   open the forecast on the road brigand (Attacking opens), attack; assert
   the Combat Arts lesson is not open and End turn isn't refused by it
   while the other units can't reach anyone.
2. Give each unit-needing lesson a "can be done now" check, computed from
   the battle state (core reach queries, not cells).
3. Re-check at open time and after every unit's action.
4. Make the test pass; add one per lesson kind.

## Acceptance criteria

- [x] The repro above: the Arts lesson doesn't open after Tamsin's attack
      (test).
- [x] An open unit-needing lesson closes without a tick (and comes back
      later) the moment no unit left can do it (tests per lesson kind).
- [x] Lessons that need no unit behave as before (existing tests pass).
- [x] `tutorial.md` and the `lessons.ron` header say the new rule.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the per-lesson "can be done now" check.
- Integration: the battle harness on Chapter 1 at Full, turn 1, as in
  step 1.

## Completion notes

- New `crates/ui/src/screens/battle/doable.rs`: `Doable`, what the
  player's units left to act could still do this turn, from the core's
  reach and the menus' own checks (an enemy in reach, an art usable from a
  tile it can stop on, a spell it could aim at a tile from one, or just a
  unit left). Each answer is worked out the first time a lesson asks and
  kept until the battle changes (reset after every command and rewind), so
  the art and spell checks don't run every frame.
- `LessonState::put_away` became `settle(turn, can)`: while the lesson in
  front has an unticked item nobody left can do, it leaves play. An open
  one is put away for the turn (as before); one not open yet is dropped
  and comes back the next time its trigger fires, this turn or later. The
  battle screen settles after absorbing triggers each frame and again just
  before a lesson is marked open (so a lesson that comes to the front
  mid-frame is checked before it opens). Only in the player's phase.
- The Attacking trigger and the "can attack" check share one function
  (`doable::reaches_enemy`), so the lesson opens and leaves on the same
  reach.
- Tests: unit tests of `Doable` on the shipped Chapter 1 and of `settle`
  for each item kind; integration tests on Chapter 1 at Full (Nick's repro:
  the Sergeant aims, backs out to his menu, attacks; Combat Arts doesn't
  open and End turn isn't refused; and Attacking put away when the
  Sergeant waits instead). Both fail with the old rule.
- Deviation: the ticket says a waiting lesson "re-checks its trigger". It
  re-checks whether it can be done instead (the same check as the put-away
  rule): the triggers are moments (a menu open), which have always passed
  by the time the lesson ahead closes, so checking them literally would
  drop every waiting lesson.

**Claude's starting rules** (for Nick to agree or veto):
- A lesson that asks a unit to do something opens only if some unit still
  to act this turn could do it; otherwise it waits until it starts again
  (that turn or a later one).
- An open one is put away, unfinished, the moment nobody left could do
  what it still asks, and comes back on a later turn.
- "Could do it": Attacking, some unit left could reach an enemy; Combat
  Arts, some unit left has an art usable from a tile it can reach; Spells
  that change tiles, some unit left could aim a spell at a tile; Selecting
  a unit and Threat arrows, any unit left.
