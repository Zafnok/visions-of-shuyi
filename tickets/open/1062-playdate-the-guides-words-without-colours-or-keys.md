---
id: "1062"
title: "Playdate: the guide's tips and lessons don't talk about colours, a mouse or keys"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["1060"]
nick_input: decision
completed:
---

# 1062 — Playdate: the guide's words without colours or keys

## Context

Found while making 0066's mockups (2026-10-18). Some of the guide's
words are only true on the PC:

- Tip `unit_selected` (`assets/data/tips.ron`): "Blue is where it can go,
  red is where it can attack. Steer to a blue tile…". The Playdate has no
  colour: move range is light dots, attack range diagonal lines
  (`docs/design/playdate.md`, *Telling things apart without colour*).
- Tip `enemy_in_range`: "That enemy on a red tile?…".
- Lesson `keys` ("Played on the keys"): "No mouse here; it's all keys.
  And every key can be changed in Options…", with the item "Open Options
  from the map menu: {Cancel}". The Playdate has buttons and a crank, and
  Options is in its system menu (*Buttons and the crank*).

1060 makes a lesson's `{Action}` name the Playdate's way to do it and
counts it; it rewords nothing ("Out"). 1058 fits the box to the screen;
it changes no words. The agent writes every line
(`docs/story/writers-guide.md`); whether the `keys` lesson exists on the
Playdate at all is a tutorial design question (which lessons exist,
`docs/design/tutorial.md`), so it goes to Nick.

## Nick input

**Decision** (`ask-nick` skill), one question before building: on the
Playdate, does the "Played on the keys" lesson (a *different here*
lesson, shown at Full and Short) A) go away, B) become a lesson about the
Playdate's own controls (A + B ends the turn, the crank rewinds, the
system menu holds Options), or C) describe your own. Record the answer in
`docs/design/tutorial.md` (a *On the Playdate* paragraph under *The
lessons*). No check after merge (Nick, 2026-10-21): `cargo shuyi
frame-png --playdate` pictures of the tips and lessons of Chapter 1's
first battle go in the PR, and the rest goes on 1064's checklist.

## Scope

**In:**
- A way for a tip's or lesson's text (and an item's label) to have a
  Playdate wording beside the PC's, in the data (`tips.ron`,
  `lessons.ron`, and the language files of each language), chosen by
  the device as the button names are. Decide the shape (an optional
  `playdate_text`, or a text key per device) and record it in an ADR if
  it is a new data pattern (`write-adr`).
- Playdate wordings for `unit_selected` and `enemy_in_range` that name
  the marks (dots, lines), and whatever Nick decides for `keys`.
- A test that walks every tip and lesson on the Playdate's context and
  finds no colour word ("blue", "red", "green", "yellow") and no "mouse"
  or "key"/"keys" in the text shown.

**Out (do not do):**
- The PC's words (unchanged; tests).
- The box's look (1058) and the `{Action}` placeholders (1060).
- New tips or lessons beyond what Nick decides for `keys`.

## Implementation steps

1. Read `crates/ui/src/tips/view.rs`, `crates/ui/src/tips.rs`
   (`fill_placeholders`), how `ctx.words()` gives `tip_text` and
   `lesson_text`, and how the device is known (`ctx.device`).
2. Ask Nick the question above; record it.
3. Add the Playdate wording to the data and the lookup; write the lines
   (`docs/story/writers-guide.md`, Vix's voice in
   `docs/story/characters/guide.md`), in American English.
4. The walking test; `cargo shuyi check-text` and `lang-status` stay
   green (each language gets the new lines or is reported missing as
   usual).

## Acceptance criteria

- [ ] `docs/design/tutorial.md` records Nick's answer for the `keys`
      lesson on the Playdate.
- [ ] On the Playdate's context no tip or lesson of Chapter 1 shows a
      colour word, "mouse" or "key(s)" (a test).
- [ ] No PC text changes (snapshots unchanged).
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: the lookup picks the Playdate wording on the Playdate and the
  PC's elsewhere.
- Integration: the walk over every tip and lesson on the Playdate's
  context.
- Manual: none (left-overs go to 1064).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
