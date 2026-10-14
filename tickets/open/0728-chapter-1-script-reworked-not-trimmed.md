---
id: "0728"
title: Rework the Chapter 1 script as scenes, not line by line
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: high
status: todo
blocked_by: []
nick_input: sign-off
completed:
---

# 0728 — Rework the Chapter 1 script

## Context

Nick, Chapter 1 playtest round 2 (0804, 2026-10-14), about 0724's rewrite:

> "the rewrite was too literal. It took the lines line by line and sucked
> out the quips rather than reworking the script. Aske says 'fourteen men
> in the fort...' the line is so literal and boring.... I don't like the
> original quips but this is just matter of fact and weird."

0724 cut the jokes from `assets/dialogue/ch01.dlg` one line at a time
(its completion notes list every changed line). What's left reads as
reports of facts. Nick wants neither the old quips nor flat lines: scenes
rewritten so people talk like people under threat, with character in how
they say things rather than jokes on top.

Read first: the `story-writing` skill and ADR-0011; `docs/story/` (bible,
`chapters/ch01.md` beat sheet, `beats.md`, the characters' sheets, which
0724 gave sample lines); `docs/design/setting-and-tone.md` (*Where the jokes
go*, *Language*: American English); 0724's and 0707's completion notes. Claude writes all the lines; Nick
decides only story questions.

## Nick input

**Sign-off:** read or play the Chapter 1 scenes. Do the people sound like
people? Is Aske's report of the fort something she'd say?

If the rework needs a change to a story beat (`beats.md`), ask Nick with the
`ask-nick` skill before writing it.

## Scope

**In:**
- Rewrite `ch01.dlg` scene by scene: each scene's purpose (from the beat
  sheet) first, then the lines. Lines may be merged, reordered, cut or
  added; who speaks when may change.
- Keep: every scene id and trigger, every `@choice` and the story facts it
  sets, the `@if` parts (0715/0716), the music and caption directives
  unless a scene's purpose changes, the facts the map depends on (Aske's
  count must still match the fourteen of the fort, `chapter-1.md`).
- The tone rule stays: the opening and the battle are serious; no quips
  under threat. Serious is not flat: voice comes from word choice,
  rhythm, what each person notices and what they don't say.
- Update the beat sheet, the ledger and the sheets' sample lines if they
  change.
- The title cinematic (`assets/cinematics/title.ron`) shows some of these
  lines by box number (`Talk` shots): check them after the rewrite and fix
  `first` / `count` if the boxes moved (0833 reworks the cinematic; say in
  the Completion notes which boxes it shows now).
- List the lines with `cargo xtask lines <scene>` and keep the content checks green (line
  ids change; voice clips don't exist yet for Chapter 1).

**Out (do not do):**
- New story beats or changes to what happens (ask Nick first).
- Other chapters.
- The dialogue screen or engine.

## Implementation steps

1. Read everything in *Context*. For each scene write down its purpose and
   what each speaker wants in it (in the beat sheet, `chapters/ch01.md`).
2. Rewrite the scenes. Read each aloud in your head as the character; if a
   line only states a fact, give it the speaker's angle on that fact.
   Example of the direction (not a line to use): Aske's count could say
   what kind of hunter she is and what she thinks of the Red Company,
   not just the number.
3. Run the gates (the content checks read the script) and the Chapter 1
   tests (the winning replay plays through the scenes).
4. In the Completion notes, quote three before/after pairs, including
   Aske's fort line.

## Acceptance criteria

- [ ] Every `ch01_*` scene rewritten; scene ids, triggers, choices and
      facts unchanged (the content tests pass).
- [ ] Beat sheet, ledger and sheets updated where they changed.
- [ ] The cinematic's `Talk` shots still show the opening lines they meant
      to (or the notes say what 0833 must change).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Existing content and flow tests (script parsing, triggers, the Chapter 1
  replay). Snapshots that show script text are reviewed and accepted.

## Completion notes

*(Filled in by the session that completes the ticket.)*
