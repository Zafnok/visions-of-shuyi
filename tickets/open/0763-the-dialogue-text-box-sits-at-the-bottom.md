---
id: "0763"
title: "The dialogue text box sits at the bottom of the screen, also in full screen"
type: bug
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0763 — The dialogue text box sits at the bottom of the screen

## Context

A friend's playtest of the Act 1 build, passed on by Nick (2026-11-07):

> "The Dialogue text box is moved up too much to the middle during full
> screen and should probably stay near the bottom"

Why it sits high:

- The game draws on a fixed 100×32 grid of 8×16 cells
  (`crates/ui/src/console.rs` ~:19). The dialogue screen puts the text box
  at rows 21–27 (`TEXT_BOX = Rect::new(0, 21, 100, 7)`, `TEXT_Y = 23`,
  `crates/ui/src/screens/dialogue/glyph.rs` ~:44–48; `BOX_PLACE` ~:188),
  under the portraits (rows 1–18, `FRAME_Y`, `FRAME`). **Rows 28–31 are
  empty** below it.
- The engine scales the grid by the largest whole number that fits and
  centres it with black bars (`layout()` in the engine's
  `crates/ui/src/frame/console.rs`; ADR-0063). The game's window is
  1640×1064 (`crates/app/src/main.rs` ~:33), so in a window there are
  almost no bars. In full screen at 2560×1440 the scale is still 2×
  (1600×1024): 208 px of black above and below. The box's bottom then
  ends about 77 % of the way down the screen, with four empty rows and a
  bar under it.
- The engine's text box grows upward for reply choices and keeps its
  bottom row (`box_rect`, engine `crates/ui/src/dialogue/text_box/
  glyph.rs`), so moving the bottom row moves the whole box.

The black bars themselves are the engine's scale mode: engine tickets
2333 (scale modes, integer or fit; After MVP) and 2376 (parked, the grid
grows by whole cells). Not this ticket.

## Nick input

None: he said where it goes ("near the bottom"). The portraits move down
with the box so they still stand on it (*the agent's starting rule*,
written in `look-and-feel.md`, on the Act 1 playtest's checklist,
1064). Verified by snapshots and `frame-png` pictures at full-screen
sizes in the PR.

## Scope

**In:**
- The text box's bottom row on the grid's last usable row (the box at
  rows 25–31, or 24–30 if a frame border needs the last row), with
  `TEXT_Y` and `BOX_PLACE` following.
- The portrait frames moved down by the same number of rows, keeping the
  gap between them and the box as it is.
- The **overlay** case (a battle's scene over the map, `view.overlay`):
  check what it paints below the box today (the code blanks those rows;
  the comment mentions the battle's key help). Keep the overlay working;
  if moving the box hides something that matters there, keep the
  overlay's box where it is and say so in the Completion notes.
- The caption, the place line, the log's key hint and anything else the
  dialogue screen paints near those rows: nothing overlaps (snapshot and
  picture check).
- The scene camera's debug layout (`crates/ui/src/debug/scene_camera.rs`
  ~:29, `LAYOUT`) if it copies the box's rows.
- `look-and-feel.md` (the dialogue layout lines, ~:774 and wherever it
  states rows).

**Out (do not do):**
- The engine's scale mode and black bars (engine 2333, 2376).
- The Playdate's dialogue screen (1043), which has its own layout.
- The dialogue log's own layout.

## Implementation steps

1. Read `dialogue/glyph.rs` (constants, `paint`, `paint_stage`,
   `paint_text_box`) and the overlay path.
2. Move the box and the frames; run the dialogue snapshots and look at
   every changed one.
3. `cargo shuyi frame-png` pictures of a two-portrait scene, a narration
   page and a reply choice, at the window size and at 2560×1440, before
   and after, in the PR.
4. Docs.

## Acceptance criteria

- [ ] The text box's bottom row is the last usable row of the grid
      (a test on the painted buffer: the box's border is on that row).
- [ ] With reply choices the box still fits (it grows upward) and
      nothing overlaps the portraits (snapshot).
- [ ] The overlay over the battle map still paints as before, or the
      Completion notes say why it differs.
- [ ] Before/after `frame-png` pictures at 2560×1440 are in the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the dialogue screen snapshots updated (each changed one
  looked at).
- Unit: the box's row on the painted buffer.

## Completion notes

*(Filled in by the session that completes the ticket.)*
