---
id: "0740"
title: "Narration boxes are left-aligned like speech"
type: feature
milestone: M6 Story & dialogue
model: sonnet-5
effort: low
status: done
blocked_by: ["1177"]
nick_input: none
completed: 2026-10-21
---

# 0740 — Narration boxes are left-aligned like speech

## Context

Chapter 1 playtest round 3 (0804, 2026-10-16):

> "it's odd to me that narration dialogue boxes have their text centered
> while talking dialogue boxes have their text left aligned. Make it
> consistent, pref left alignment."

Since the carve (1112) the text box's painter is the engine's: its glyph
skin's `paint_lines` centres a narration box's lines across the box and
a single line vertically, in `TextDim`; speech starts at the box
place's `text_x` (`TEXT_X = 4` in the game's `BOX_PLACE`,
`crates/ui/src/screens/dialogue/glyph.rs`). The caption on the top row
(`paint_caption` there) is a different thing (a place line, `@caption`)
and stays centred unless Nick says otherwise. The title cinematic's talk
shots use the same painter (`crates/ui/src/cinematic/talk.rs`).

**The engine makes it a choice: engine#2305** (in
[Shuyi Engine's tickets](https://github.com/Zafnok/shuyi-engine/tree/main/tickets)):
the text box's `BoxPlace` gains how narration is aligned, `Centred` as
today or `Left` like speech. This ticket is the game's half: picking
`Left` everywhere the game places a text box.

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> The opening's narration reads left-aligned, like speech.

## Scope

**In:**
- Nothing for the pin: 1177 moves it past engine#2305 (check
  `shuyi-engine.rev` has it).
- Narration lines start at `TEXT_X` like speech, from the box's first
  text row (no vertical centring), keeping their `TextDim` colour
  (*Claude's starting rule*: the colour still tells narration apart):
  `Left` in the game's `BOX_PLACE`.
- The same in the title cinematic's talk shots (they paint with the
  dialogue screen's skin, so `BOX_PLACE` covers them: check it).
- The Playdate dialogue screen (1043) if it centres narration: same rule.
- `look-and-feel.md` (the dialogue screen's layout) updated.

**Out (do not do):** the engine's half (engine#2305); the top-row caption;
narration's colour.

## Acceptance criteria

- [x] A narration box's lines start at the speech column (test on the
      dialogue view's line positions, plus one snapshot).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: narration line x/y in the view or skin.
- Snapshot: a narration box, reviewed.

## Completion notes

- Moved the pin to the engine's `main` (8617026), which has engine#2305, and
  set `narration: NarrationAlign::Left` in the game's `BOX_PLACE`. Narration
  now starts at column 4 on the first text row, still `TextDim`. The title
  cinematic's talk shots and the log use the same painter, so they follow.
  The Playdate front end has no separate dialogue painter, so nothing to change.
- The new pin needed small fixes to build: `MapScene` gained a `more` field
  (two sites), `frame_png::run` takes a target, and the engine's standard
  formats list gained `pattern map`. Tests only, plus one pattern in a test.
- Tests: the dialogue glyph test and the cinematic shot test now check
  narration at the speech column; five snapshots re-reviewed (only the
  narration line moved). `look-and-feel.md` updated.
- Engine friction: estimate 5, measured 5 (1x), under 2x.
- No gameplay rules decided. Caption on the top row stays centred.
