---
id: "0466"
title: "A tip about the map zoom on the first turn"
type: content
milestone: M3 Battle UI
model: implementation
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0466 — A zoom tip on the first turn

## Context

The battle map starts at 2× and the Zoom map key (Q / P / right trigger)
switches to 1× (0439). Nothing in the tutorial mentions it. Chapter 1
playtest round 3 (0804, 2026-10-16), asked whether 2× was the right
start:

> "ensure the guide has a tip for zoom on first turn, and also we need the
> map scroll arrows to ensure players know when it's cut off from the
> whole picture. Otherwise 2x is ok to start on."

The scroll arrows are ticket **0449** (done); this ticket is the tip.
Tips: `assets/data/tips.ron` (0458's three are the model), Vix's voice
`docs/story/characters/guide.md`, `docs/design/tutorial.md`.

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

> Does the tip come on turn 1 at a good moment, and read in
> Vix's voice?

## Scope

**In:**
- A tip "Zooming the map" (*the agent's wording*), at **Full**, on the first
  turn of the first battle, after the turn-1 lessons (cursor → keys →
  next unit → selecting) and the start tip, so it doesn't add to the pile
  at the start (*the agent's starting rule*: on the first player phase when
  no lesson is open; if a trigger for that is missing in `tips.ron`, add
  one). Text names `{ZoomMap}` and says the view scrolls with the cursor.
- Only when the map is drawn with pictures (zoom does nothing in the
  glyph look). If the map look is glyphs, the tip waits until it's
  pictures.
- In Guides like every tip.

**Out (do not do):** the scroll arrows (0449); Short and None.

## Implementation steps

1. Add the tip to `tips.ron` (title, text, face per `guide.md`).
2. Add a trigger if needed; respect "tips wait while a lesson is open"
   (0456).
3. Tests.

## Acceptance criteria

- [ ] At Full, the tip shows on turn 1 after the turn-1 lessons close
      (test); never at Short or None.
- [ ] With the glyph look it doesn't show until pictures are on.
- [ ] It's listed in Guides.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: Chapter 1 at Full, turn 1, the tip's order after the
  lessons.

## Completion notes

*(Filled in by the session that completes the ticket.)*
