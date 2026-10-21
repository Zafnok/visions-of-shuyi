---
id: "0739"
title: "Faces over a darkened place: how much the listener (and narration) dims them"
type: feature
milestone: M6 Story & dialogue
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0739 — Faces over a darkened place

## Context

Since 0732 a conversation is drawn over its place at 40% brightness
(`PLACE_BRIGHTNESS = 0.4`, `crates/ui/src/screens/dialogue/glyph.rs:50`).
Chapter 1 playtest round 3 (0804, 2026-10-16), asked whether the faces
were clear and the place readable:

> "the background should stay at this brightness (or maybe 50-60%) but
> why are the faces impacted? they are foreground this is weird."

What the code does (checked 2026-10-16):

- The place's darkening is **not** applied to the faces:
  `glyph::paint` paints the place, fades it (`buf.fade`, line 78), and
  only then draws the faces and the text box (lines 90–91).
- But the **listener's** face is dimmed by design: `LISTENER_DIM = 0.45`
  (`glyph.rs:45`), drawn at 140/255 opacity toward the frame's panel
  colour (`crates/ui/src/portrait.rs:43-45`, `90-94`). During
  **narration** there is no speaker, so **both** faces are dimmed
  (`glyph.rs:108`).
- `look-and-feel.md` (*Places behind conversations*, line ~1194) set the
  place "a little darker than the dimmed listener's portrait", so a
  dimmed face and the place look about equally dark. That's very likely
  what Nick saw.
- One real case of faces darkened with the background: the title
  cinematic's talk shots fade the whole frame, faces included
  (`crates/ui/src/cinematic.rs:574-594`).

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

**Decision** (`ask-nick`), on real frames with the bought portraits over
the yard and the study (both darkened), in each option:

1. **The listener's face:** A. not dimmed; the speaker is shown by the
   double frame and the lit name only (Fire Emblem Three Houses keeps
   both faces bright, check before citing); B. dimmed less (e.g. 80%
   instead of 55%); C. as today; D. your own.
2. **During narration:** A. faces at full brightness; B. faces hidden
   while a narration box is up; C. dimmed as the listener is; D. your own.
3. **The place's brightness:** stays 40% (his words), or 50% / 60% to try
   ("or maybe 50-60%"): show all three.
4. **The title cinematic's talk shots:** faces kept bright there too?

## Scope

**In:** the questions; changing `LISTENER_DIM` / narration dimming /
`PLACE_BRIGHTNESS` to the answers; the cinematic's talk shots drawing
faces after the fade if Nick says so; `look-and-feel.md` updated.

**Out (do not do):** the per-scene brightness for the study (0738);
the portraits' art.

## Acceptance criteria

- [ ] Nick answered; recorded.
- [ ] The dialogue view/skin draw faces per the answers (tests read the
      view's dim values, not cells, except the one look snapshot).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the view's per-face dim for speaker, listener and narration.
- Snapshot: a talk over a place, each reviewed.

## Completion notes

*(Filled in by the session that completes the ticket.)*
