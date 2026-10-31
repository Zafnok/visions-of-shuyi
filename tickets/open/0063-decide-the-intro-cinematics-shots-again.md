---
id: "0063"
title: "Decide: rework the intro cinematic's shots (opening narration, faces, the 4x shot, the ending)"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0063 — Decide: rework the intro cinematic's shots

## Context

Chapter 1 playtest round 3 (0804, 2026-10-16). The small fixes Nick asked
for are **0843**. Beyond them, he wrote this about the intro cinematic as
a whole:

> "some parts of the intro still don't make sense... we start with
> narration and the narration ends with "somewhere... chops wood" and it's
> supposed to be exciting? and then the portraits fly by, it's so
> disjointed. The whole cinematography feels a bit off here. Also when it
> zooms on the lead and the archer(?) then it looks like a romantic moment
> in the middle of a battle it's very odd. And the narration at the
> beginning is thinking you read slow asf. Probably it should use the
> cutscene tech we made a few tickets ago -- for showing a dimmed
> background and animations outside of battle...? Also the music closes
> with a swell crescendo thing and everybody is just chilling totally
> stationary it's very anti-climactic. Anyway. We can park this work til
> after engine carve and playdate since it's not integral to ch1 but it
> definitely feels way off so I expect some parked tickets for it."

Today's storyboard (`assets/cinematics/title.ron`, 12 shots;
`docs/design/title-screen.md`, *Intro cinematic*):

| Start | Shot | Nick's remark |
| ----- | ---- | ------------- |
| 0:00 | Party pan, 3× | |
| 0:15 | `ch01_intro`'s 3 captions, ~5 s each | ends on chopping wood, not exciting; reads too slow |
| 0:30.75 | Six faces slide past | "portraits fly by, it's so disjointed" |
| 0:50 | Tamsin's news | (0843 makes it 7.5 s) |
| 0:56 | Turn 1, 3× | |
| 1:08.5 | Combat: brigand vs lead | |
| 1:14.5 | Turn 2, 3× | |
| 1:25.25 | Combat: Tamsin vs brigand | |
| 1:31.25 | Aske crosses the footbridge, **4×**, next to Tamsin | "looks like a romantic moment" (he read it as the lead and the archer) |
| 1:35.5 | Turn 3, 3× | |
| 1:44.5 | The whole field, 2× | the song swells and "everybody is just chilling totally stationary" |
| 1:57 | Logo | (0843 moves it to 1:55.8) |

"The cutscene tech" is 0732–0734: a place drawn darkened behind talk,
with a short moving opening (actors walking on the place).

**Waits for the engine carve and the Playdate port** (Nick: "We can park
this work til after engine carve and playdate"), so `blocked_by` has 1118
and 1050. The Playdate's own cinematic (1044) is built before this; this
ticket's answers say whether the Playdate version changes too.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

**Decision**, with the `ask-nick` skill and real frames or short recorded
mockups of each option (an Artifact page that plays them). Options draw on
real games' openings; check each reference before showing it, and say
honestly when unsure.

## Questions

1. **The opening (0:15).** Options, for example:
   - A. Cut the captions; open on the yard at dawn with the sparring
     opening (0734's tech), then go straight to the battle.
   - B. Keep narration but write new lines for the intro (a hook, not the
     chapter's quiet opening), and show them faster.
   - C. Captions over moving places (the yard, Veyne Hall, the toll-fort)
     instead of black.
   - D. Describe your own.
   Sub-question: how fast should captions go (seconds per caption)?
2. **The faces (0:30.75).** For example:
   - A. Drop the slide; introduce each character in action on the map
     (a short shot of each doing their thing).
   - B. Each face over its own place, one at a time, with a name.
   - C. Keep the slide but slower, and only the four leads.
   - D. Describe your own.
3. **The 4× shot of Aske and Tamsin (1:31.25).** For example: A. a wider
   shot (3×) with the enemy in it; B. replace it with another fight; C.
   keep the zoom but on a unit acting alone; D. your own.
4. **The ending (1:44.5 to the logo).** The song builds to its biggest
   moment there. For example: A. the party charges the bridges on the
   swell; B. Harl steps out of the fort on the swell, then the logo; C. a
   last combat scene on the swell; D. your own.
5. **The Playdate:** do the answers apply to its cinematic too (1044)?

Then write the build ticket(s) the answers need (0843's file is the
model), blocked by this one, and add them to 1064's `blocked_by` if Nick
wants them before the Act 1 playtest.

## Scope

**In:** the questions above; recording the answers in
`docs/design/title-screen.md` (*Storyboard*); writing the build tickets.

**Out (do not do):** building any of it; Tamsin's picture (0446); the
small fixes (0843).

## Acceptance criteria

- [ ] Nick answered questions 1–5; his words are in `title-screen.md`.
- [ ] The build ticket(s) exist, with `blocked_by` set; `cargo xtask
      ticket-lint` passes.

## Tests required

- None (decision).

## Completion notes

*(Filled in by the session that completes the ticket.)*
