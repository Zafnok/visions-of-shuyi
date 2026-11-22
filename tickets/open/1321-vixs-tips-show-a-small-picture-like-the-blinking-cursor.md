---
id: "1321"
title: "Vix's tips show a small moving picture of what they name"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["0469"]
nick_input: answer-first
completed:
---

# 1321 — Vix's tips show a small moving picture of what they name

## Context

Nick, after watching a new player play Chapter 1 (2026-11-22):

> "We should embed images or sprites into the guides (options menu) and
> vix tips where appropriate like for example she said "this is your
> cursor" why don't we show the blinking cursor (animated) in the tip bar
> itself"

0469 already covers pictures in Guides (map menu → Guides): a picture
per topic rendered from a small staged battle scene, with a decision on
stills vs short loops and the Playdate's version. Nick's note now says
he wants them **moving** where it helps, and **in the tip box too**.
0469's decision is widened to cover the tip box (its text updated in the
same PR as this ticket), and this ticket shows its pictures in the tip
box and the lesson box.

Today `TipView` (`crates/ui/src/tips/view.rs`) holds a title, an optional
portrait, a heading, text lines, `how` lines and the close hint: no
pictures. The tip box is painted by `tip_box::glyph::paint` and the
Playdate skin. Pictures follow ADR-0038 (sprite items or a frame of the
real scene painted by the skins, never per-pixel cells) and bought art
shows only in builds that have it (ADR-0040).

## Nick input

**Answer first:** 0469 (with the tip box added to its question: which
tips and lessons get a picture, where in the box, still or moving).
Nothing to check after merge: `frame-png` strips in the PR; what's left
to judge in play goes on 1064's checklist.

## Scope

**In:**
- `TipView` gains an optional picture: a staged scene id (0469's format)
  or a sprite item, with its motion computed from the time alone (the
  cursor's blink is the live cursor's own pulse).
- Tip and lesson data (`assets/data/tips.ron`, the lesson data) name the
  picture per entry; the entries 0469 lists, starting with "this is your
  cursor".
- The glyph and sprite skins of the tip box draw it; the box makes room
  for it without moving the text Nick already reviewed beyond a reflow.
- Fallback per 1314 (if done; else leave a `TODO(1314)`): the Playdate
  shows the still or nothing as 0469 decided.

**Out (do not do):** new tip text (dialogue writing is paused); Guides'
pictures (0469).

## Implementation steps

1. Widen 0469's question in its file (in this ticket's planning, before
   0469 is asked, if 0469 isn't done yet).
2. The `TipView` field; data fields; loader validation (an unknown
   picture id fails content validation).
3. Skins; the tip box's layout with a picture.
4. `frame-png` strips of the cursor tip at 0 and 500 ms.

## Acceptance criteria

- [ ] A view test: the cursor tip's view has its picture.
- [ ] A look test: the tip box draws the cursor picture, blinking at two
      times.
- [ ] Content validation fails on an unknown picture id (a test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the loader. View and look tests for the tip box.

## Lesser targets

Playdate: 0469's answer (a one-bit still or nothing). GBA: no target.

## Completion notes

