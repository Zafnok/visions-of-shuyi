---
id: "0737"
title: "Write the Chapter 1 script again from the beats, on a blank page"
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: high
status: todo
blocked_by: ["0736"]
nick_input: none
completed:
---

# 0737 — Write the Chapter 1 script again from the beats

## Context

Chapter 1 playtest round 3 (0804, 2026-10-16). Nick on the script after
0728's rework:

> "Fable still did it line by line. Still 14 archers stuff... We need to
> toss the script. We can keep the beats we agreed on but the whole script
> is fairly mid imo."

> Hollis in the yard: "like I said, this is AGAIN a line by line rewrite.
> Toss the whole thing."

> Aske's fort report: "again, rewrite". Tamsin over Harl: "we'll rewrite
> all this crap". Tamsin's news: "it's ok".

And on pace: the chapter drags "In the conversations".

What stays (Nick's answers in round 3): Hollis's "Aye", the spelling
"Greywater Abbey" and Tamsin's "Your Former Grace" (recorded in
`setting-and-tone.md` by 0804's round 3 PR). The story beats
(`docs/story/beats.md`, `docs/story/chapters/ch01.md`) stay as decided.

**0736 comes first:** it researches the craft and writes it into the
story-writing skill, with the rule that a rewrite starts on a blank page.

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16:
"ch1 playtest 4 come after those milestones with update script"), so
`blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

The ticket's `model` follows ADR-0010 (creative writing → fable-5.1).
Nick named Fable as the writer of 0728's rewrite; if he runs this one on
another model, that's his call.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Does the chapter sound like a
> game he wants to play, and do the conversations still drag? (Asked first
> in 1064, as round 4 of the Chapter 1 playtest would have.)

## Scope

**In:**
- Every scene in `assets/dialogue/ch01.dlg`: `ch01_intro`,
  `ch01_prebattle`, the battle's lines (first turn, Harl, death quotes,
  retreat lines), `ch01_victory`, `ch01_tbc`.
- Write each scene **from the beat sheet, without reading its current
  lines first** (0736's rule). Keep only what the format needs: scene
  ids, `@` directives (music, place, opening, caption), reply choices that
  the story gates read, and the line ids voice clips use, if any are kept.
- Shorter: keep inside 0736's scene-length budget.
- The three kept words (above).
- The title cinematic reuses lines from `ch01_intro` and `ch01_prebattle`
  (`assets/cinematics/title.ron`, its `talk` shots). Update the box
  indices it plays, so it still shows the three captions and Tamsin's
  news (or its new equivalent).

**Out (do not do):**
- Changing a story beat, a character's arc, who lives or dies, or a
  reply's consequences. If the beats can't work without a change, stop
  and ask Nick with `ask-nick`.
- The intro cinematic's cinematography (**0063**).
- Voices (0722 regenerates them after this).

## Implementation steps

1. Read 0736's craft section and `docs/story/beats.md`,
   `docs/story/chapters/ch01.md`, the character sheets and
   `setting-and-tone.md`. Do **not** open `ch01.dlg`'s text yet.
2. Draft each scene in a scratch file from the beats.
3. Run the story-writing skill's critique pass on each scene.
4. Now open `ch01.dlg` and carry over only the structure listed in Scope
   (ids, directives, replies the gates read). Replace the text.
5. Fix the title cinematic's talk shots if box indices moved.
6. Run the content checks and the dialogue tests; update snapshots that
   show script text, after looking at each.
7. Completion notes: the box count per scene before and after, and two
   or three lines per scene that show the new voice.

## Acceptance criteria

- [ ] Every Chapter 1 scene is newly written; no line of the old script
      survives except the three kept words.
- [ ] Each scene is within 0736's length budget; total box count is lower
      than today's.
- [ ] Story gates, replies and the Classic/Casual line variants (0715,
      0716) still work: their tests pass.
- [ ] The title cinematic still shows the intended boxes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the existing dialogue and Chapter 1 flow tests pass; the
  content validation of `ch01.dlg` passes.
- Snapshot: updated where script text shows, each one reviewed.

## Completion notes

*(Filled in by the session that completes the ticket.)*
