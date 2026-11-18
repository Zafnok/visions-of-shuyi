---
id: "0739"
title: "Faces over a darkened place: how much the listener (and narration) dims them"
type: feature
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-11-18
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

- [x] Nick answered; recorded.
- [x] The dialogue view/skin draw faces per the answers (tests read the
      view's dim values, not cells, except the one look snapshot).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the view's per-face dim for speaker, listener and narration.
- Snapshot: a talk over a place, each reviewed.

## Completion notes

Asked 2026-11-18 with real frames (picture look, bought portraits) of
the yard and the study: the listener not dimmed / 80% / as today (55%);
faces during narration at full brightness / hidden / dimmed; the place at
40% / 50% / 60%. Nick kept all three as they are:

> "I have no problem with the listener face as it is today. the concern is
> that the ambient lighting bleeds into the dialogue boxes. like in the
> study" · "again no concern here" · "this is not the problem. the problem
> is the study has its dialogue boxes being a ridiculously low
> brightness." (seen in the web build)

- **No code change.** `LISTENER_DIM` (0.45), the narration dimming and
  `PLACE_BRIGHTNESS` (0.4) stay. The existing tests already pin them:
  `the_speaker_is_bright_and_the_listener_dimmed_and_mirrored`,
  `narration_dims_both_and_starts_at_the_speech_column` and
  `the_place_is_painted_by_the_map_skin_darkened_under_the_talk`
  (`crates/ui/src/screens/dialogue/glyph/tests.rs`), and the place
  snapshots of `crates/ui/tests/it/places.rs`.
- **Question 4 (the title cinematic) was not asked:** its talk shots are
  painted by the dialogue skin, so they follow the same rules, and the
  title fades the cinematic only while it comes in from black at its
  start, over a map shot. No talk shot is ever faded as a whole (the
  ticket's reading of `cinematic.rs` predates the engine carve).
- **What Nick actually saw** is a web-only bug: the study's night light
  darkens the faces and text boxes. `cargo shuyi frame-png` (picture,
  glyph and Playdate looks) draws them as bright as over the yard, so the
  game's view and skin are right and the fault is in the live renderer.
  Filed as **0774** (added to 1064's `blocked_by` and the roadmap).
- Recorded in `docs/design/look-and-feel.md`, *Places behind
  conversations*, "The faces over a darkened place".
