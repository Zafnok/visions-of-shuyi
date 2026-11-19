---
id: "0480"
title: "Decide and build: a battle forecast that is easier to read"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
nick_input: decision
completed: 2026-11-16
---

# 0480 — A battle forecast that is easier to read

## Context

A friend's playtest of the Act 1 build, passed on by Nick (2026-11-07):

> "The UI for the battle forecast is hard to parse visually, and is not
> that intuitive i.e. for seeing how much expected HP enemy will have at
> the end"

What the forecast shows today (`docs/design/look-and-feel.md`, *Attack
forecast*, ~:616–651, decided in 0404; arts in `combat-arts.md`, *The
arts menu and the forecast*):

- each side: name, weapon, the weapon's durability (`20/20`), `HP <now>`,
  a 5-cell bar, Hit, Crit and an affinity mark;
- a numbered strike list (`8 dmg →` on the left, `← 8 dmg` on the
  right), doubles as extra rows, a skull on the killing strike, later
  strikes dimmed; totals as `17 ×2`.

What makes it hard:

- **HP after the fight is never a number.** The view has it
  (`ForecastHp { now, after, max }`, `crates/ui/src/screens/battle/view/
  forecast.rs` ~:88, filled in `screens/battle/forecast.rs` ~:77 and ~:85),
  but the glyph skin prints only `hp.now` (`glyph/forecast.rs` ~:168). The
  after-HP is only the shaded part of a 5-cell bar (`paint_bar`, ~:179–194),
  too coarse to read a number from.
- **The weapon's `20/20` sits right above the HP line** and reads like HP.
- The strike list makes the player add up the damage.

The snapshot that shows it:
`crates/ui/src/screens/battle/snapshots/trpg_ui__screens__battle__attack_tests__forecast_with_a_double_and_a_counter_snapshot.snap`.

Nick, 0404: "I like the bar representing HP in the forecast." Keep the
bar unless he drops it.

**Either order with 0333:** 0333 builds 0080's answer on armour (for
example a minimum hit) and may add a mark to the forecast for it. If
0333 is done, keep its mark in the new layout; if not, leave room for
one line and 0333 adds it.

## Nick input

**Decision** (`ask-nick`, before building), on `frame-png` pictures of the
real forecast with the private art, in both looks (glyphs and pictures),
for three fights: a plain exchange, a double with a counter, and a kill.
Options (check each game's claim before showing it; claim only what the
game really shows):

- A. **HP now → after as numbers** beside the bar (`HP 21 → 9`), as
  Fire Emblem's later forecasts do.
- B. **A longer bar** (10 or 20 cells) with the lost part blinking, as
  Into the Breach's health pips flash the damage to come.
- C. **Fewer rows:** the strike list folded into one line per side
  (`8 ×2`, a skull when it kills), durability moved beside the weapon's
  name or dropped.
- D. **Labelled columns** for both sides (HP, Dmg, Hit, Crit), as Fire
  Emblem's GBA forecast does.
- E. Combinations, or describe your own.

Ask two things: which layout, and whether the weapon's durability stays
on the forecast.

## Scope

**In:**
- The question, the answer and the mockups; `look-and-feel.md`, *Attack
  forecast*, updated with his words.
- Building the pick in the forecast's view and every skin of it: the
  glyph skin (`glyph/forecast.rs`), the picture look if it differs, and
  the Playdate skin (`playdate/forecast.rs`), which keeps its own fit rule
  (1075 sets Playdate line widths: if 1075 is open, tell it what changed).
- The forecast is a screen view (ADR-0054), not the map scene.

**Out (do not do):**
- What ← / → do and how swapping weapons is shown (0463, which waits for
  this ticket so its arrows go on the new layout).
- The skull as a sprite (0470); multi-target arts (0327) and marks (0328)
  rows: they follow whatever layout this picks.
- Changing combat numbers.

## Implementation steps

1. Read the done tickets 0404 and 0442, `view/forecast.rs`,
   `battle/forecast.rs`, `glyph/forecast.rs`, `playdate/forecast.rs`.
2. Render the options with `cargo shuyi frame-png` (throwaway skins in
   the scratchpad are fine for mockups); check every render (overlaps,
   numbers that match the fight). Ask Nick; expect more than one round.
3. Record the answer; change the view (fields the new layout needs) and
   the skins.
4. Tests: the view's numbers for each fight above; snapshots.

## Acceptance criteria

- [x] Nick's pick is recorded in `look-and-feel.md`.
- [x] The forecast shows each side's HP after the fight in the way he
      picked; a test on the view for a kill (after = 0), a double and a
      miss-free exchange.
- [x] The glyph and Playdate skins paint it (snapshots; `frame-png`
      pictures in the PR).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `ForecastView` values for the three fights.
- Snapshot: the forecast snapshots updated; one new for a kill.

## Completion notes

Nick picked over two rounds of `frame-png` pictures (three fights staged
in a throwaway battle, glyph and picture looks, a blinking GIF):

- **Round 1:** A + B (`HP 19 → 6` and a 12-cell bar whose lost part
  blinks), keep the strike list, and the uses "beside the weapon, or under
  when a art is used which drains dura, and then shows uses depleting like
  hp".
- **Round 2:** an art's cost as numbers only, `20 → 16` under the weapon
  (E; F, with a second blinking bar, was not picked); and "unless the HP
  is expected 0 it doesn't show 0 cells lit up".

Recorded in `look-and-feel.md` (*Attack forecast*, *HP after the fight,
and the weapon's uses (0480)*), with the examples in `combat-arts.md` and
`playdate.md` brought in line.

Built in both skins; the view needed no new fields (`ForecastHp.after`
and `SkillLineView.durability` already carried the numbers):

- `glyph/forecast.rs`: `Iron Sword 20` on the weapon's row; a row under
  the weapons only for an art's cost or `(broken)` (both columns move
  down with it); `HP 19 → 6` (the after number in its HP colour, `0` in
  `hp_low`); the 12-cell bar on its own row, the lost part shaded then
  full every half second (`lost_shaded`, from the clock as the skin's
  motion); the art's line only its name. The picture look uses the same
  panel.
- `playdate/forecast.rs`: the same lines in 10-cell columns (`Iron S…
  20`, `HP 19 → 11`, a 10-cell bar, the lost part a checker then black).
- `units.rs`, `lit_fill`: the forecast's bars keep one cell for a unit
  with HP (Nick's round-2 rule). The side panel's and the map's bars are
  unchanged.

**Follow-up: 0498.** 6 of the 14 weapons don't fit their name and uses
in a 13-cell column (Steel Sword 25 is 14). Nick, asked: "use shortened
names rather than truncated i.e. Stl Swo / probably need some mapping
from long to short name". That needs a short name in the item data and
its translation (a content change in the private repository) and his
pick of the names, so it is ticket 0498. Until then the name is cut with
`…` (`Steel Swo… 25`), as the Playdate's already were.

0333 is still open: the row under Crit (the affinity's, blank without
one) is where its mark can go. 1075 is open: its Context now says the
Playdate's forecast cuts a weapon's name to fit its uses.

*The agent's starting rules (Nick can veto; for the Act 1 playtest,
1064):*

- The blink is half a second shaded, then half a second full, starting
  shaded.
- `(broken)` shows on the row under the weapon (0404's starting rule,
  moved).
- A weapon's or spell's uses after its name are in the text colour (the
  name stays dimmed); a spell's uses show the same way (`Fire 10`).
- A long weapon name is cut with `…` to fit its uses, on the PC as on
  the Playdate, until 0498's short names.
- The Playdate blinks the lost HP between its checker and black.
- The one-lit-cell rule also holds for the HP a unit has now.

Pictures: `docs/screenshots/0480-forecast.png` (the three fights and
Guard Break) and `0480-forecast-playdate.png` (the double, in both halves
of the blink, and Guard Break), cropped to the forecast so no map of the
content is committed (ADR-0118).

Engine friction: estimate 120 production lines (two skins' side blocks,
judged by 0404's `paint_side` and `paint_bar`), measured 206 added (1.7×),
under 2×. PR size: 924 changed lines.
