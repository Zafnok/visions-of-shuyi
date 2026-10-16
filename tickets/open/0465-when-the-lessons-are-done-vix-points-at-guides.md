---
id: "0465"
title: "When the lessons are done, Vix points at Guides and offers a fresh start"
type: feature
milestone: M3 Battle UI
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0464", "1118", "1050"]
nick_input: sign-off
completed:
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

## Nick input

**Sign-off:** finish the lessons at Not familiar and at Familiar with
other strategy RPGs; is the moment right, the wording right, and do the
three answers do what he meant?

## Scope

**In:**
- **When:** the moment the last lesson the campaign's tutorial level
  shows is done (*Claude's starting rule*). At Full that may be a later
  turn (spells on tiles, Combat Arts); if Nick wanted it straight after
  turn 1–2's lessons, that's his call at sign-off. At None, never.
- **What:** one guide box, in Vix's voice and with her face, wrapping as
  0459 does. It says she's done, that Guides in the map menu (named with
  the player's key, `{Cancel}`) has everything, and asks with three
  answers (*Claude's wording, Nick can change*):
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

- [ ] At Full and Short, the box shows right after the last lesson
      closes, once per campaign; never at None (tests).
- [ ] "Keep going" closes it; the battle goes on.
- [ ] "Restart the battle" goes back to turn 1 with the lessons done; no
      rewind charge or restart is counted (test).
- [ ] "Restart with the lessons" goes back to turn 1 and the lessons show
      again (test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: battle harness on Chapter 1, finishing every lesson, then
  each of the three answers.
- Snapshot: the box (bought-art build not needed; the public build shows
  no face).

## Completion notes

*(Filled in by the session that completes the ticket.)*
