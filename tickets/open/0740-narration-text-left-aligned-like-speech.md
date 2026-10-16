---
id: "0740"
title: "Narration boxes are left-aligned like speech"
type: feature
milestone: M6 Story & dialogue
model: sonnet-5
effort: low
status: todo
blocked_by: ["1118", "1050"]
nick_input: sign-off
completed:
---

# 0740 — Narration boxes are left-aligned like speech

## Context

Chapter 1 playtest round 3 (0804, 2026-10-16):

> "it's odd to me that narration dialogue boxes have their text centered
> while talking dialogue boxes have their text left aligned. Make it
> consistent, pref left alignment."

Today (checked 2026-10-16), `paint_lines` in
`crates/ui/src/screens/dialogue/glyph.rs:252-270` centres a narration
box's lines (`TEXT_BOX.x + (TEXT_BOX.w - len) / 2`, lines 263–266) and a
single line vertically (line 261), in `TextDim`; speech starts at
`TEXT_X = 4` (line 267). The caption on the top row (`paint_caption`,
lines 95–100) is a different thing (a place line, `@caption`) and stays
centred unless Nick says otherwise. The title cinematic's talk shots use
the same painter (`crates/ui/src/cinematic/talk.rs:67-83`).

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

## Nick input

**Sign-off:** the opening's narration reads left-aligned, like speech.

## Scope

**In:**
- Narration lines start at `TEXT_X` like speech, from the box's first
  text row (no vertical centring), keeping their `TextDim` colour
  (*Claude's starting rule*: the colour still tells narration apart).
- The same in the title cinematic's talk shots (same painter).
- The Playdate dialogue screen (1043) if it centres narration: same rule.
- `look-and-feel.md` (the dialogue screen's layout) updated.

**Out (do not do):** the top-row caption; narration's colour.

## Acceptance criteria

- [ ] A narration box's lines start at the speech column (test on the
      dialogue view's line positions, plus one snapshot).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: narration line x/y in the view or skin.
- Snapshot: a narration box, reviewed.

## Completion notes

*(Filled in by the session that completes the ticket.)*
