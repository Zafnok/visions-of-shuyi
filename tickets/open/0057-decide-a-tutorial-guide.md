---
id: "0057"
title: "Decide: a basic tutorial, maybe with a guide character"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
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
  through `docs/story/` (Claude writes the lines; Nick decides who).
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

- [ ] `chapter-1.md` records the answer and Nick's words.
- [ ] Follow-up tickets exist and 0804 waits for them.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None (design decision; docs and tickets only).

## Completion notes

*(Filled in by the session that completes the ticket.)*
