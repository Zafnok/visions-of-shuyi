---
id: "0057"
title: "Decide: a basic tutorial, maybe with a guide character"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-14
---

# 0057 — Decide: a basic tutorial

## Context

Nick, Chapter 1 playtest round 2 (0804, 2026-10-14): "we should have some
kind of basic tutorial, I am almost thinking of the annoying fairy from oot
or clippy or something... some tutorial mascot thing idk."

What teaches the player today (`docs/design/chapter-1.md`, *Teaching*;
ticket 0406): one-time **tips** (`assets/data/tips.ron`), a box shown the
first time something happens in a battle (first battle, first unit
selected, first enemy in reach, first forecast, low HP, level up, enemy
phase, danger zone), plus the **help bar** naming the keys. Nothing
explains Combat Arts, skills, magic, terrain, rewind or the threat arrows,
and no character speaks the tips.

## Nick input

**Decision**, with the `ask-nick` skill, with a real mockup of a Chapter 1
moment for each option (the bought portraits; `ascii-art` skill):

- **A. A guide character who speaks the tips** in a small box with a face:
  Navi in Ocarina of Time, Nell in Advance Wars' Field Training. Who: a party member (Hollis teaching the
  lead fits the story?), someone new, or a mascot. Story questions go
  through `docs/story/` (the agent writes the lines; Nick decides who).
- **B. Better tips, no character**: more triggers (arts, skills, magic,
  terrain, rewind, threat arrows), a *Tutorials* list to read them again
  (Fire Emblem Engage has one in its menu).
- **C. A short optional practice battle** before Chapter 1 (Fire Emblem:
  The Blazing Blade's Lyn chapters, Advance Wars' Field Training).
- **D. Teaching in the story's first battle itself**: scripted moments in
  Chapter 1 (FE Awakening's prologue).
- **E. Combinations**, and "describe your own."

Sub-questions: can tips be switched off in Options (on by default)? Is the
guide skippable for veterans (a question at New Game)? Which topics does
Chapter 1 teach?

Check every game example before using it.

## Scope

**In:**
- The question with mockups, Nick's answer recorded in `chapter-1.md`
  (*Teaching*) and, for a character, `docs/story/`.
- Follow-up ticket(s) to build it (`write-ticket` skill), added to 0804's
  `blocked_by` (Nick wants Chapter 1 to feel right before round 3).

**Out (do not do):**
- Building it.
- Writing the guide's lines (the follow-up's job).

## Implementation steps

1. Mockups; ask; record.
2. Write the follow-up tickets; add them to 0804's `blocked_by` and to
   `docs/ROADMAP.md`'s round 3 list.

## Acceptance criteria

- [x] `chapter-1.md` records the answer and Nick's words.
- [x] Follow-up tickets exist and 0804 waits for them.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None (design decision; docs and tickets only).

## Completion notes

Asked 2026-10-14 in two rounds, with mockups of Chapter 1's first turn
drawn over real frames with the bought art (sent to Nick, not committed):
today's tip; Hollis in a tip box with his face (A1) and in the conversation
box (A2); a *Guides* list (B); a practice-battle prompt (C); a scripted
step (D). The second round showed a shortlist of seven faces from the
bought packs (faces the cast already uses were left out) and the checklist
in the guide's box.

Nick's answers (verbatim in `docs/design/tutorial.md`):

1. New Game asks how familiar the player is: **Full** (the basic controls,
   optional for others), **Short** (only what differs from other strategy
   RPGs, forced), **None**.
2. A guide character: the **Fox Miko** face, **outside the story for
   now** (maybe tied in much later, like Angel in Borderlands 2). Not the
   Shuyi cameo, not Hollis.
3. **Guides** as a menu option.
4. Lessons as **checklists in any order**, **inside Chapter 1's battle**
   (2A), not a practice battle.
5. What's different here: the agent's list (keyboard controls, rewind, threat
   arrows, Combat Arts, terrain magic), accepted.
6. Tips switched off in Options: not separately; the level covers it.

Recorded in `docs/design/tutorial.md` (new), `chapter-1.md` (*Teaching*;
replaces "No forced tutorial steps"), `docs/design/README.md`, and
`docs/story/characters/guide.md` (new: outside the story, no name yet).

Follow-up tickets, all added to 0804's `blocked_by` and the ROADMAP's
round 3 list: **0455** the guide's face in the tip box, **0835** the
familiarity question at New Game, **0456** checklist lessons (blocked by
0455, 0835), **0457** the lessons (0456), **0458** new tips and the
Guides list (0455, 0456), **0731** the guide's name and lines (0457,
0458). 0058 (options per save) now also asks whether the level can be
changed later.

*the agent's starting rules* (Nick can veto; marked in `tutorial.md`):

- The familiarity question comes after Classic/Casual, before the lead.
- The level belongs to the campaign; old saves load as Full.
- At Short, the standard tips (unit selected, enemy in reach, forecast,
  low HP, level up, enemy phase, danger zone) don't show; they're in
  Guides.
- "Forced" means the turn can't end until the open lesson is done;
  everything else works under it.
- Each lesson shows once per campaign; rewinding doesn't undo a lesson.
- The rewind lesson opens and closes the rewind screen; no charge spent.
- Topics with a lesson get no tip; the new tips are skills, magic and
  terrain.
- Guides works at every level, and every topic can be read, seen or not.
- The guide's box title reads "Guide" until 0731 names the guide.

Found while making the mockups: the start tip is drawn under Hollis's
first-turn scene in Chapter 1 (both boxes at once). Its fix is in 0455.

Game examples checked 2026-10-14: Engage's menu entry is *Guides*
(System), not "Tutorials"; Blazing Blade's Lyn chapters are required on a
first playthrough; Awakening's prologue teaches with pop-ups, not with
Frederick; Field Training is 14 maps with Nell.
