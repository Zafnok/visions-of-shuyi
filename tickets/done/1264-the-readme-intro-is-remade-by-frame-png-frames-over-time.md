---
id: "1264"
title: "The README's intro animation is remade by one frame-png command, not by hand"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-13
---

# 1264 — The README's intro animation is remade by one frame-png command, not by hand

## Context

The README's hero picture, `docs/img/intro.gif` (2.7 MB, 10 frames a
second), is the title's intro cinematic. 0125 made it by hand: one
`cargo shuyi frame-png --wait <t>` per frame, waits over 30 s split in
pieces, the bottom 16 rows cut off, then the frames put together by a
Pillow script that was never kept. Nobody can remake it after the intro,
the art or the title changes without redoing that work.

Engine PR #377 (engine ticket 2514), in the pin since the pin move of
2026-11-11, gives `frame-png` frames over time: `--every SECONDS
--count N` after the steps draws `N` frames, as a sheet labelled with
their times, or as an APNG that loops forever with `--apng`. Engine PR
#379 (engine ticket 2589), in the pin since the pin move of 2026-11-13,
adds `--gif`: a looping GIF with the frames' exact colours, never
quantized or dithered (a frame of more than 256 colours is refused and
`--apng` suggested), each frame shown for its delay to the nearest
hundredth of a second. The game's `cargo shuyi frame-png` has the
options already.

## Nick input

None: the picture shows the same moments of the intro; the README's
look is 0125's.

## Scope

**In:**
- Remake the intro animation with `frame-png --every … --count …
  --gif` (`--apng` if a frame has more than 256 colours), built with `--features private-assets` as
  0125's was. `--count` is at most 64 and `every × (count − 1)` at most
  2000 frames: the intro's two stretches (56 s to 74.5 s, 116 s to
  119 s) at 10 frames a second need more than 64 frames, so either draw
  it in several clips and join them, at fewer frames a second if it
  still reads well, or file the engine ticket that lifts the cap
  (`engine-change`, "ticket and follow-up") and keep the GIF until it
  lands.
- The debug key's hint row: leave it out by the command's own options
  if one exists (a release build, `--crop`), otherwise say what is left.
- The command(s) written down where the next person finds them (the
  README's picture notes, or a `cargo xtask readme-pictures` if more
  than one command), with every other README picture's command from
  0125's notes beside it.
- The README points at the new file; the old GIF deleted if replaced.

**Out (do not do):**
- New moments or a new look for the intro or the gallery.
- Animated store-page media (Steam, itch.io): the store pages are 0903
  and later tickets.

## Implementation steps

1. Read `cargo shuyi frame-png --help` and 0125's notes (the waits and
   cuts).
2. Draw the two stretches with `--every` and `--count`; compare with
   the GIF frame by frame for the same moments; check the file size
   (0125 kept it under 3 MB).
3. Write the commands down; swap the README's picture.

## Acceptance criteria

- [x] The README's intro animation is made by the written-down
      command(s) alone, with no hand step.
- [x] It shows the same moments as `intro.gif` and is no bigger than
      about 3 MB.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None: a picture for the README. The PR shows the new file.

## Notes

- Found by the pin move of 2026-11-11 (`docs/engine/pins.md`, engine
  #377).

## Completion notes

- **One command:** `cargo run -q -p xtask --features private-assets --
  readme-pictures` (`crates/xtask/src/readme_pictures.rs`) writes
  `docs/img/intro.gif`, with no hand step. It runs `frame-png --scale 1
  --every 0.1 --count N --gif` four times: 56.0–62.3 s, 62.4–68.7 s,
  68.8–74.4 s and 114.7–117.7 s of the title song (the 64-frame cap, and
  one run can't skip from 74.5 s to 114.7 s). Then it joins the clips
  into one GIF that loops, with the `gif` crate the engine writes them
  with. Every frame keeps its exact colours (its own palette), stores only
  the part that changed (the engine's GIF stores each frame whole: about
  6 MB for the four clips), and a repeated frame is shown as one longer
  frame. The menu is held 3 s before the loop, as in 0125's.
- **The hint row:** `frame-png` has no option to leave it out (no
  `--crop`, and its Harness always has the debug tools), so the join cuts
  the bottom 16 rows (the help bar and the `F2 debug` hint), as 0125 did:
  800×496.
- **Same moments, today's intro:** 2.85 MB (0125's: 2.7 MB), 24.5 s.
  Compared with the old GIF moment by moment. The march and the brigand
  match. The road fight now ends about 1.4 s later in the clip, and the
  lead is left on 11 HP, not 10: 0843 retimed the fights after 0125. The
  logo now cuts in at 115.8 s (0843), not about 117 s, so the last clip
  starts at 114.7 s, not 116 s. That keeps 0125's moments: about a
  second of the whole field, then the logo and the menu land. The title
  menu now shows Vix's "Still Player 1?" note beside it (added after
  0125).
- **Deviation:** the second stretch is 114.7–117.7 s, not the ticket's
  116–119 s, for the reason above.
- **Written down:** `docs/img/README.md`, the README's picture notes:
  the intro's command and what it does, and 0125's commands for every
  other README picture, in today's spelling (`frame-png` through xtask).
  Those were not drawn again (out of scope): the notes say to check each
  one against the game first.
- The README already points at `docs/img/intro.gif`, now replaced in
  place: no other change there.
- No engine ticket filed. The 64-frame cap doesn't matter here (two
  stretches always need two runs). Storing only the changed part of each
  frame in the engine's `--gif` would make the join's encoding
  unnecessary; that is an engine idea for Nick to OK, not needed here.
- Engine friction: estimate 150, measured 290 (1.9×), under 2×.
