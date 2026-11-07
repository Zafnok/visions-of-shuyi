---
id: "0078"
title: "Decide: how a unit's stats are easy to see in battle"
type: design-decision
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0078 — Decide: how a unit's stats are easy to see in battle

## Context

A friend's playtest of the Act 1 build, passed on by Nick (2026-11-07):

> "Make stats easier to see, rather than needing to go to unit info or
> leveling up (we can workshop ideas)"

Today, on the battle map:

- **The side panel** (hovering a unit; `crates/ui/src/screens/battle/
  panel.rs`, `PanelView` and `PanelUnitView` in `view.rs` ~:151, ~:175)
  shows the terrain (name, Def/Avo, heals), the unit's name, faction,
  `Class Lv N`, HP now/max, and gold. **No stats and no weapon.**
- **Unit info** (the Unit info key or the map menu; `battle/info.rs`,
  `view/info.rs`) shows everything, on its own screen.
- Stats also show on the level-up screen.

**The docs disagree with the game.** `docs/design/look-and-feel.md`
~:522–523 says the side panel shows "name, class, level, HP, stats,
weapon, terrain. That's where class and full numbers are read." ~:778 says
it "shows stats only (Nick dropped the mini portrait)". The panel built
since shows neither stats nor weapon. Say so to Nick plainly: nobody
decided to leave them out.

## Nick input

**Decision** (`ask-nick`). Several rounds are likely ("we can workshop
ideas"). Show `frame-png` pictures of the real battle map with the
private art, both looks, a hovered ally and a hovered enemy, for each
option. Check every real game's claim before showing it; claim only what
the game really does.

- A. **The side panel shows the stats** (Str Mag Dex Spd Def Res, Mov)
  and the equipped weapon, as `look-and-feel.md` already says.
- B. **Combat numbers instead of raw stats**: Atk, Hit, Avoid, Crit and
  attack speed with the equipped weapon (the numbers a fight uses), as
  some Fire Emblem stat pages list beside the raw stats.
- C. **A key that flips the panel** between short (today) and full
  stats, without leaving the map (the key is rebindable, keyboard-input
  skill).
- D. **A bigger hover window** over the map near the unit, while the
  cursor rests on it.
- E. Combinations (e.g. A on the panel and B in the forecast), or
  describe your own.

Ask too: the same for enemies as for allies (yes / no)? And the
Playdate (its panel is small): the same pick, or the flip key only?

## Scope

**In:** the question, the mockups, and the answer recorded in
`look-and-feel.md` (*Side panel*), replacing the lines that disagree.

**Out (do not do):** building it (0484); the forecast (0480); unit info.

## Implementation steps

1. Read `look-and-feel.md` (*Side panel*, the lines above), `panel.rs`,
   the Playdate panel skin (`playdate.rs` ~:193).
2. Render the options (scratchpad renderer or throwaway skins with
   `cargo shuyi frame-png`); check each render (no invented stat, numbers
   that match the unit).
3. Ask Nick; iterate; record his words.

## Acceptance criteria

- [ ] Nick's answer is recorded verbatim in `look-and-feel.md`, and the
      lines that disagreed with the game are fixed.
- [ ] 0484's text is updated with what to build.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

None (a decision); `cargo xtask ticket-lint`.

## Completion notes

*(Filled in by the session that completes the ticket.)*
