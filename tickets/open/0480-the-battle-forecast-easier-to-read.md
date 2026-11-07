---
id: "0480"
title: "Decide and build: a battle forecast that is easier to read"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: []
nick_input: decision
completed:
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

- [ ] Nick's pick is recorded in `look-and-feel.md`.
- [ ] The forecast shows each side's HP after the fight in the way he
      picked; a test on the view for a kill (after = 0), a double and a
      miss-free exchange.
- [ ] The glyph and Playdate skins paint it (snapshots; `frame-png`
      pictures in the PR).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `ForecastView` values for the three fights.
- Snapshot: the forecast snapshots updated; one new for a kill.

## Completion notes

*(Filled in by the session that completes the ticket.)*
