---
id: "0125"
title: "README: Chapter 1 pictures, the opening intro, progress, the engine and the Playdate"
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-10-15
---

# 0125 — README: Chapter 1 pictures, the opening intro, progress, the engine and the Playdate

## Context

The root `README.md` was last refreshed by 0121 on 2026-10-05. About 60
tickets have landed since: the real Chapter 1 (0803) with the cast's
portraits (0706), the close-up combat scene (0413), the title art (0811)
and its intro cinematic (0817–0820, 0827, 0832, 0833), the Game Over
animation (0830), map zoom, lighting and colour themes, the dialogue log,
the tutorial question, player profiles, and two rounds of Nick's playtest.
Two new lines of work started: Shuyi Engine, the open-source engine split
out of the game (`docs/engine/`, 1101–1131), and the Playdate version of
Chapter 1 (`docs/design/playdate.md`, 1025–1053). The README still shows
test units on a test map and lists Chapter 1, the combat scene and the
end-of-battle music as "not in yet".

Nick asked (2026-10-15): "update readme with new graphics, opening intro,
progress, add the new roadmap items (engine, playdate), remove anything
outdated".

## Nick input

None.

## Scope

**In:**
- New screenshots in `docs/img/` with `cargo xtask frame-png` and the
  `private-assets` feature: the Chapter 1 battle, the combat scene, the
  title, frames of the intro cinematic, and the other pictures redone on
  Chapter 1 where they showed the test map.
- `README.md`: what is playable today, what is left before Nick's next
  playtest, progress (with rows for the engine and the Playdate), the
  roadmap with Shuyi Engine and the Playdate, and nothing that is out of
  date.
- Delete the pictures in `docs/img/` that nothing shows any more.

**Out (do not do):**
- No bought file, sheet or cut-out sprite in the repository (ADR-0040);
  only screenshots of the running game.
- No story spoilers past Chapter 1's opening; no game code; no new tooling.
- No rewrite of `docs/ROADMAP.md`.

## Implementation steps

1. Render the screenshots (commands in the Completion notes).
2. Count tickets per block in `tickets/open/` and `tickets/done/`.
3. Rewrite `README.md`.

## Acceptance criteria

- [x] Every image the README shows exists in `docs/img/`; those with
      bought art are screenshots of the game, nothing else.
- [x] The README's progress numbers match `tickets/` as of 2026-10-15
      (this ticket counted as done).
- [x] Every planned feature the README lists has a ticket or a design doc
      behind it.
- [x] No key is presented as fixed.
- [x] `cargo xtask ticket-lint`, `typos` and the `xtask` tests pass.

## Tests required

- None (docs only).

## Completion notes

- **Pictures** (`docs/img/`, 1600×1024 unless noted), all screenshots of
  the game made with `cargo run -q -p xtask --features private-assets --
  frame-png …` after `cargo xtask private-assets` (except
  `battle-glyphs`, made without the feature). They show the real
  Chapter 1 now, not test units. With `N="f f Down Down f f Down f"` (New
  Game, Classic, "familiar with this game", the lead as offered, Start),
  `B` = `--keys "$N" --wait 6 --keys "<80 × f>" --wait 3` (the opening
  scene clicked through to the battle), and
  `A="f Right Right Right Up Up f"` (turn 2: the lead steps up to the
  brigand who came down in the enemy phase):
  - `battle-forecast`, `battle-glyphs`: `$B --keys "Space f" --wait 15 --keys "$A" --wait 1.5 --keys "f Down" --wait 0.5`
  - `combat-scene`: the same, then `--keys "f f" --wait 1.0` instead of the last keys (the archer)
  - `battle-ranges`: `$B --keys f --wait 0.5`
  - `spell-menu`: `$B --keys "Down f f" --wait 1 --keys "Down f" --wait 0.5`
  - `dialogue`: `--keys "$N" --wait 6 --keys "<60 × f>" --wait 4`
  - `lead-select`: `--keys "f f Down Down f" --wait 0.5`
  - `title`: `--wait 3`
  - `key-bindings`: `--keys "Down Down f"`, thirteen `Down`s, then `f`
- **`intro.gif`** (800×496, 10 frames a second, 2.7 MB): the title's
  intro cinematic from 56 s to 74.5 s of the song (the party sets off, the
  brigand attacks the lead, the combat scene) and 116 s to 119 s (the logo
  and menu land), each frame `frame-png --scale 1 --wait <t>` (a wait over
  30 s split in pieces: the Harness allows 2000 frames per wait), the
  bottom 16 rows cut off (the debug key's hint), put together with one
  palette by Pillow. The cast's line-up (30.75 s to 50 s) was left out:
  its scrolling alone made 2.9 MB. Tamsin's combat shot (85.25 s) was left
  out too: riders have a stand-in picture until 0446.
- `preparations.png` is gone (Chapter 1 has no Preparations screen and the
  old one showed test units). `ch01-*.png` stay: 0803's notes name them.
- **README:** the hero is the intro GIF; the gallery is Chapter 1;
  "What you can play today" and "Not in yet" match `main` (Chapter 1, the
  combat scene, the end-of-battle music and Game Over rewind are in;
  what playtest round 3 waits for is listed as the next step); the
  progress table has rows for the work after Chapter 1, the Playdate
  (1025–1053) and Shuyi Engine (11xx in this repository), and calls the
  old "Engine: drawing, input, audio" row "Drawing, input and audio" so it
  isn't confused with Shuyi Engine; new sections for the Playdate (with
  0061's chosen mockup, `docs/design/playdate/q3r2-e-grey.png`) and Shuyi
  Engine; the intro cinematic, lighting, zoom, color themes and the
  glyph-or-pixel choice left "Planned" (done); the colorblind palette left
  it too (no ticket or design doc behind it). American spelling.
- **Progress counts** (2026-10-15, this ticket and 0840 counted): 228 of
  375. The "After Chapter 1" row is `10xx` without 1025–1053.
- **Found:** the Options box's last row sits on its bottom border (more
  so with a campaign loaded): ticket **0840**.
- Seen while capturing, not changed: after the enemy phase every enemy
  stays grey (acted) through the player's turn, so the turn-2 pictures
  show grey enemies. That is today's rule for acted sprite units
  (`look-and-feel.md`); Nick may want to look at it.
