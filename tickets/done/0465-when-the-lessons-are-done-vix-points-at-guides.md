---
id: "0465"
title: "When the lessons are done, Vix points at Guides and offers a fresh start"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0464"]
nick_input: none
completed: 2026-11-01
---

# 0465 — When the lessons are done, Vix points at Guides and offers a fresh start

## Context

Chapter 1 playtest round 3 (0804, 2026-10-16). Two of Nick's answers
meet at the same moment, the end of the lessons:

On Guides (map menu → Guides, 0458), asked if it was easy to find:

> "hard to find, the guide should say something like "you're on your own
> for the rest, if you need any help, here is where you find the Guide.
> Good luck!" something like that..."

On the lessons' moments:

> "Also, after the tutorial tips all play, we should ask if the player is
> OK with their position or wants to retry battle with no penalty, or
> retry tutorial with no penalty."

Lessons: `assets/data/lessons.ron`, `crates/ui/src/screens/battle/
lessons.rs`, `docs/design/tutorial.md`. Vix's voice: `docs/story/
characters/guide.md` (0731: dry, calm, short; no jokes). Restart Battle
already exists in the map menu.

After 0464, so a lesson that can't be done doesn't hold things up first.

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Finish the lessons at Not familiar and at Familiar with
> other strategy RPGs; is the moment right, the wording right, and do the
> three answers do what he meant?

## Scope

**In:**
- **When:** the moment the last lesson the campaign's tutorial level
  shows is done (*the agent's starting rule*). At Full that may be a later
  turn (spells on tiles, Combat Arts); if Nick wanted it straight after
  turn 1–2's lessons, that's his call in the Act 1 playtest (1064). At None, never.
- **What:** one guide box, in Vix's voice and with her face, wrapping as
  0459 does. It says she's done, that Guides in the map menu (named with
  the player's key, `{Cancel}`) has everything, and asks with three
  answers (*the agent's wording, Nick can change*):
  - "Keep going" (the cursor starts here: the safe answer, Nick's rule of
    non-destructive defaults);
  - "Restart the battle" (back to turn 1, lessons stay done);
  - "Restart with the lessons" (back to turn 1, this battle's lessons
    are shown again).
- **"No penalty":** a restart from here spends no rewind charge, counts
  no restart in any results or stats the game keeps, and keeps nothing
  from the try (EXP, items used). Check what Restart Battle does today
  and say in the completion notes what differs.
- Write Vix's line per `guide.md`'s rules; text ids in the language files
  (ADR-0045).
- Update `tutorial.md`.

**Out (do not do):**
- Pictures in Guides (0469).
- Any change to which lessons exist.

## Implementation steps

1. Find where the last lesson closes; add the "all done" event.
2. Build the box with the three answers (reuse the guide box from 0455 /
   0459 and an existing yes/no question's widget, as a view and a skin,
   ADR-0054).
3. Wire the two restarts (reuse Restart Battle; for "with the lessons",
   clear this battle's lesson ids from the campaign before restarting).
4. Tests.

## Acceptance criteria

- [x] At Full and Short, the box shows right after the last lesson
      closes, once per campaign; never at None (tests).
- [x] "Keep going" closes it; the battle goes on.
- [x] "Restart the battle" goes back to turn 1 with the lessons done; no
      rewind charge or restart is counted (test).
- [x] "Restart with the lessons" goes back to turn 1 and the lessons show
      again (test).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: battle harness on Chapter 1, finishing every lesson, then
  each of the three answers.
- Snapshot: the box (bought-art build not needed; the public build shows
  no face).

## Completion notes

When every lesson the campaign's tutorial level shows is done, Vix's box
comes up over the map: where Guides is (with the player's key), and three
answers. Text ids are in `assets/lang/en/ui.ron` and the test pack; the view
is `LessonsEndView`, painted by `tips/glyph.rs` on both the desktop and the
Playdate battle skins. `tutorial.md` has the rule.

**"No penalty", checked against Restart Battle:** nothing differs; both
restarts are the same restart. It rebuilds the battle from its setup, so every
rewind charge is back and nothing from the try is kept (EXP, items used, the
state). The game keeps no count of restarts anywhere. As with Restart Battle,
the battle's opening scene and notes play again, a battle with Preparations
goes back to them as the player left them, and playtime keeps counting. The
only addition: "Restart with the lessons" also takes this battle's lessons out
of the campaign's done list.

**A bug this would have hit, fixed here:** the campaign took in the lessons
only when the *number* of them changed. Taking one back as the answer went in
left the number the same, so its saves would have kept the old list. It now
compares the lists (`flow.rs`, with a test).

**Starting rules (*the agent's starting rule*), for the Act 1 playtest
(1064):**

- The box waits until the map is back to browsing, not the instant the last
  lesson closes. If the last lesson is opening Arts, the player finishes that
  unit's action first.
- Once per campaign: after "Restart with the lessons", finishing them again
  doesn't ask again.
- Cancel does nothing on the box; only an answer closes it.
- Only a real campaign has the box; the debug Quick Battle doesn't.
- Vix's words, with her `sly` face: "That's the lessons / That's all I've got.
  You're on your own now. / Need a reminder? Guides, in the map menu (d). / A
  fresh start costs nothing, if you want one." Nick's "Good luck" is left out
  to stay within her one-short-lines voice; he can ask for it back.

**To judge in the playtest:** the ticket's rule means the box can come late.
Spells that change tiles only counts once the player has aimed one, so someone
who never opens the Heretic's spells on a forest doesn't see the box in Chapter
1 at all, and it then comes in whichever later battle finishes the list, where
"Restart with the lessons" re-shows only that battle's lesson. If Nick wants it
straight after turn 1-2's lessons, that's his call in 1064.

**Limit:** "this battle's lessons" means those finished on this battle screen.
Lessons finished before an earlier restart of the same battle, or before a
suspend and continue, stay done and don't show again.

**Deviations:** the ticket asks for `frame-png` pictures in the PR; the PR has
the box's text snapshot instead, which is the same glyph frame. The public
build has no face, and the box's fit beside the face is checked by test.
At Full, the integration test leaves out the three lessons that need their own
set-up (threat arrows, Arts, tile spells), which the Short test plays.
