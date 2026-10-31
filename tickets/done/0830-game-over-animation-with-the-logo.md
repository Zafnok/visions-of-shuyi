---
id: "0830"
title: "Game Over screen: an animation with the game's logo"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0811"]
nick_input: decision
completed: 2026-10-13
---

# 0830 — Game Over animation with the logo

## Context

Nick, playing Chapter 1 (ticket 0804, 2026-10-12):

> "game over and title screen should both have a nice animation w our logo
> (I think there are open tickets for this)"

The title has its tickets: the logo (0811) and the intro cinematic that
ends on it (0819, 0820). **Game Over had none.** Today it is the heading
`GAME OVER` and a menu (`Rewind` while a charge is left, `Retry Battle`,
`Title`) on a plain screen, with the `defeat` music carrying on from the
banner (`docs/design/audio.md`).

Code: `crates/ui/src/screens/game_over.rs` builds a `GameOverView`;
`game_over/glyph.rs` paints it (ADR-0054). The logo is whatever 0811
makes, so this ticket waits for it.

**Added 2026-10-12 (0811 is done):** the logo is a sprite,
`assets/title/logo.png` (52×31, drawn 8× by
`title::glyph::logo_sprite`; ADR-0059;
`docs/design/look-and-feel.md`, *Title screen*). The title shows it over a
bought backdrop with sparks drifting up (`title::glyph::paint_logo`);
whether Game Over uses the backdrop and the sparks too is part of the
question for Nick. Idle motion is drawn only while `Ctx::idle_motion`
is on.

## Nick input

**Decision, asked first** (`ask-nick`; show moving mockups or frame
strips, several options, expect rounds). Nick said only "a nice animation
w our logo"; everything else is open, so don't pick for him:

- **What moves:** the logo fades in over the darkened battle map; the logo
  breaks or drains of colour; the words `GAME OVER` form and the logo sits
  small under them; something of his own. Draw on real games for each
  option (e.g. Fire Emblem's plain fade to `GAME OVER`, Dark Souls' `YOU
  DIED` band, Into the Breach's timeline collapse) and say honestly what
  each is.
- **How long** before the menu can be used, and whether a press skips it.
- Whether the lost battle stays visible behind it.

## Scope

**In:**
- The animation Nick picks, on the Game Over screen, in time with nothing
  (the `defeat` music keeps playing as it does today).
- A press skips to the finished screen with the menu *(the agent's starting
  rule, unless Nick answers otherwise: nobody should wait to retry)*.
- `docs/design/look-and-feel.md` records his answers.

**Out (do not do):**
- The title's logo and cinematic (0811, 0819, 0820).
- The `DEFEAT` banner, the music, the menu's items and what they do.
- "To be continued" (same module; leave it).

## Implementation steps

1. Ask Nick (above); record the answers.
2. `GameOverView` gains what the animation needs as plain data (how far
   along it is, 0–1; whether the menu is shown). The screen advances it
   with `input.dt` only; no clock outside `FrameInput`.
3. `game_over/glyph.rs` paints the view. The logo is drawn the way 0811
   draws it (reuse its item or function; don't copy the art). If it is a
   picture, it is a sprite item from an asset file (ADR-0038).
4. Any press during the animation jumps to its end; that press does
   nothing else (it must not pick `Rewind`).
5. The "Reduce motion" or animation-speed settings, if `options.md` has
   them by then, apply here as they do elsewhere.

## Acceptance criteria

- [x] `look-and-feel.md` records Nick's pick, with the date.
- [x] Test: the view goes from start to finished over the decided time,
      and the menu can't be used before the end.
- [x] Test: a press during the animation finishes it and picks nothing.
- [x] Snapshots of the first, a middle and the last frame.
- [x] `Rewind`, `Retry Battle` and `Title` behave as before.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration (`crates/ui/tests/it/`): the two behaviour tests, reading the
  view.
- Snapshot: three frames of the glyph skin.

## Completion notes

**Nick's answers (2026-10-13, two rounds of renders; recorded in
`docs/design/look-and-feel.md`, *Game Over screen*):** the battle fades to
black; the logo at its title size and place, gray; the words `GAME OVER`
in gold under it, as wide as the logo ("A but align the GAME OVER with the
logo better", then "A2"); black behind; two seconds; any press skips.

**Done:**
- `GameOverScreen` counts the animation's time from `input.dt` alone
  (`ANIMATION_SECONDS` = 2). `GameOverView` gained `progress` (0 to 255);
  `menu` and `help` are `None` until the end. A press before the end
  (any key or controller button) jumps to it and is used up.
- `game_over/glyph.rs` paints it: what the buffer holds fades to black
  (the flow draws the lost battle under the screen while it still shows),
  the logo fades in through `title::glyph::logo_sprite`, painted dimmed,
  then the words.
- The words are a picture, `assets/title/game_over.png` (52×7), generated
  by `cargo xtask title-art` from `assets-src/title/game_over.txt` like the
  logo, and named by `title.ron`'s new optional `game_over` field. Our own
  lettering: no third-party asset, no credit needed. Without the picture
  the screen prints the heading as text.
- Tests: unit tests of the screen and of the skin; in
  `crates/ui/tests/it/flow.rs`, the two behaviour tests through the real
  game, with snapshots of the first, a middle and the last frame.

**Deviations:**
- Step 5: `options.md` has no "Reduce motion" or animation-speed setting,
  so there was nothing to apply. `Ctx::idle_motion` is for motion that
  never ends (the title's sparks); this animation runs once, so it doesn't
  depend on it.
- The three frame snapshots are of the real flow (the lost battle is in
  the first), not of the skin alone; the skin's own tests read sprites and
  cells.
- Tests that pressed keys the moment Game Over opened now press once more
  first (the skip), or wait two seconds.

**Gameplay rules the agent decided (starting rules, Nick may veto):**
- Backing out of the rewind screen returns to Game Over as it was left:
  the animation doesn't play a second time.
- The skip makes no sound.
- The menu appears at once at the end of the two seconds; it doesn't fade.
- The time is split so: the battle is gone after 0.7 s, the logo fades in
  from 0.6 s to 1.4 s, the words from 1.1 s to 1.9 s.
- `GAME OVER` reads the same in every language until a language brings
  its own picture (follow-up below).

**Follow-up tickets:** 0831 (a language can bring its own Game Over
words, or show its text).

**For Nick:** lose a battle (or let the turn limit run out) and watch Game
Over open; press a key half way through to see the skip.
