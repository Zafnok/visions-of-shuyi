---
id: "0121"
title: "README: new screenshots, progress and planned features"
type: infra
milestone: M0 Foundation
model: opus-5.5
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-10-05
---

# 0121 — README: new screenshots, progress and planned features

## Context

The root `README.md` was last refreshed by 0111 on 2026-09-30. About 80
tickets have landed since: the spell menu, the animated enemy phase, the
whole game flow with saves, options, key and controller rebinding, the
Chapter 1 script, and the bought pixel art. Its "not in yet" list named
things that are in, its progress table was out of date, and its
screenshots were SVGs made by a script that was never kept.

Nick asked (2026-10-04): "update readme with new screengrabs, roadmap
progress, etc. cut any sections no longer relevant. show planned features
without being overly technical."

## Nick input

None at the start. When the first version showed only the glyph look, Nick
asked "what about our sprites?" and, told that the repo's rule forbade
committing screenshots of the bought art, decided (2026-10-05):

> I think screenshots will be necessary. as long as we're not distributing
> the *assets* then we are ok. We'll need to include the assets in game
> screenshots anyway for steam store page... we just don't distribute the
> raw files at any point

## Scope

**In:**
- New screenshots in `docs/img/`, made with `cargo xtask frame-png`
  (0232). First without the `private-assets` feature; then Nick asked for
  the sprites (see *Nick input*), so the battle and dialogue pictures are
  made with it.
- The bought-art rule in `CLAUDE.md`, the `ascii-art` skill and
  `frame-png`'s texts: screenshots of the game may be committed.
- `README.md`: what is playable today, progress, what is left for
  Chapter 1, and the planned features in plain words.
- Cut what a visitor doesn't need: the Releasing steps move to
  `docs/releasing.md`; the Play section repeated the link at the top.
- `docs/ROADMAP.md`: the Chapter 1 critical path table lists only what is
  still open.

**Out (do not do):**
- No bought file, sheet or cut-out sprite in the repository (ADR-0040).
- No story spoilers; no game code; no new tooling (only the wording of
  `frame-png`'s warning changes).
- No rewrite of the rest of `docs/ROADMAP.md`.

## Implementation steps

1. Render the screenshots (commands in the Completion notes).
2. Count tickets per block in `tickets/open/` and `tickets/done/`.
3. Rewrite `README.md`; move Releasing to `docs/releasing.md`.
4. Bring the critical path table in `docs/ROADMAP.md` up to date from
   0804's `blocked_by` list.

## Acceptance criteria

- [x] Every image the README shows exists in `docs/img/`; those with
      bought art are screenshots of the game, nothing else.
- [x] The README's progress numbers match `tickets/` as of 2026-10-04
      (this ticket counted as done).
- [x] Every planned feature the README lists has a ticket or a design doc
      behind it.
- [x] No key is presented as fixed (the README names only the default
      style and says everything can be rebound).
- [x] `cargo xtask ticket-lint`, `typos` and the `xtask` tests pass.

## Tests required

- None (docs only).

## Completion notes

- Seven PNGs in `docs/img/` (1600×1024), replacing the three SVGs.
  `battle-forecast`, `battle-ranges`, `spell-menu` and `dialogue` show the
  bought art (`cargo run -q -p xtask --features private-assets --
  frame-png …`, after `cargo xtask private-assets`); `battle-glyphs` is
  `battle-forecast` without the feature; the two menu screens look the
  same either way. All show test units; the lead's face is the public
  placeholder until 0706. To make them again, from the repo root
  (`B="Down f Left f f"`, Quick Battle with its banner closed):
  - `battle-forecast`, `battle-glyphs`: `--keys "$B f Right Right Right Up f" --wait 0.8 --keys "f f" --wait 0.3`
  - `battle-ranges`: `--keys "$B w f" --wait 0.3`
  - `spell-menu`: `--keys "$B Down f Right Right Up Up f" --wait 0.8 --keys f --wait 0.3`
  - `dialogue`: `--keys "f f Up f" --wait 3`
  - `preparations`: `--keys "Down f f f"`
  - `key-bindings`: `--keys "Down Down f"`, ten `Down`s, then `f`
- README: the level-up picture is gone (no short script reaches that
  screen); the per-block codes (`00xx`…) and the technical bullets under
  "How it's built" are gone; "Play" and "Releasing" are gone as sections
  (the link is at the top, the steps are in `docs/releasing.md`).
- The planned features come from `docs/ROADMAP.md`,
  `docs/design/replayability.md`, `docs/design/world-structure.md` and
  `docs/design/voices-languages-and-script.md`, and each has an open
  ticket.
- `docs/ROADMAP.md`: the critical path is now three rows (0022–0024, 0413,
  0706, 0822 → 0803, 0809 → 0804). The paragraphs above the table are
  history and were left as they are.
- **The bought-art rule changed** (Nick's words above): the files, and
  pictures that hand the art out (a sheet, a cut-out sprite, a mockup
  pasted together from the files), still never enter this repository; a
  screenshot of the running game may. `CLAUDE.md`, the `ascii-art` skill
  and `frame-png`'s help and warning say so now. No ADR was changed:
  ADR-0032 and ADR-0040 speak of the files, not of screenshots.
- The progress numbers are a snapshot and will go stale again.
