---
id: "1403"
title: "The README's intro GIF from one frame-png run: the clip joiner of readme-pictures goes"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: blocked
blocked_by: ["engine#3446"]
nick_input: none
completed:
---

# 1403 — The README's intro GIF from one `frame-png` run

## Context

The third engine-friction sweep (1397, row C7) found that
`crates/tools/src/readme_pictures.rs` (ticket 1264, about 350
production lines) draws `docs/img/intro.gif` as four `frame-png --gif`
clips and joins them itself. The engine's GIF has four gaps that force
this:

- it stores every frame whole;
- it can't crop the help bar;
- it caps a run at 64 frames;
- it can't skip from 74.5 s to 114.7 s.

Engine ticket 3446 adds delta frames, `--crop`, `--span` and `--hold`.
`join`, `paint` and `Joiner` are marked
`engine-friction: engine#3446, moves (game 1403 deletes it)`.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine 3446 if it is behind (`engine-change` §7).
- `readme-pictures` runs one `frame-png --gif` with its two spans
  (56–74.5 s and 114.7–117.7 s at 10 frames a second), the crop of the
  bottom 16 rows and a 3 s hold.
- `join`, `paint` and `Joiner` go. The module keeps its usage text, its
  spans and the call.
- Redraw `docs/img/intro.gif`. Its frames look the same: the same
  moments and the same crop. Its size is about the same or smaller.

**Out (do not do):**
- Choosing other moments (ticket 0125 chose them).

## Implementation steps

1. Write the engine-friction estimate: about 40 lines kept, about 300
   removed.
2. Move the pin, if needed. Rewrite `command` and `draw` as one run.
3. Redraw the picture (needs `--features private-assets` and the bought
   art). Compare the frame count and size with the old one in the
   completion notes.

## Acceptance criteria

- [ ] `grep -n "fn join\|struct Joiner" crates/tools/src/readme_pictures.rs`
      finds nothing.
- [ ] The new `docs/img/intro.gif` has the same number of frames, give
      or take the merged repeats, and is no bigger than the old one.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The module's argument test (the spans, crop and hold handed to
  `frame-png`), replacing the joiner's tests.

## Completion notes

*(Filled in by the session that completes the ticket.)*
