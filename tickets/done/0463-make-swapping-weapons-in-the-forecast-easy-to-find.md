---
id: "0463"
title: "Make swapping weapons in the forecast easy to find"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0480"]
blocks: ["1064"]
nick_input: decision
completed: 2026-11-17
---

# 0463 — Make swapping weapons in the forecast easy to find

## Context

Since 0442 there is no weapon list: Attack opens the forecast with the
equipped weapon, and ← / → swap weapons there (`docs/design/
combat-arts.md`, *The arts menu and the forecast*). Chapter 1 playtest
round 3 (0804, 2026-10-16), on the Arts menu:

> "so swapping weapons on forecast menu feels quite hidden as a feature it
> should be more obvious idk how also the guide should probably mention
> it"

Nothing on the forecast says ← / → do anything, and no lesson or tip
mentions it.

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

**Waits for 0480** (2026-11-07): a friend's playtest found the forecast
hard to read, and 0480 asks Nick for a new layout. The swap arrows or
hint go on that layout, so this ticket's mockups start from it.

## Nick input

**Decision** (`ask-nick`) on real frames of the forecast with the bought
art (a unit with two weapons, e.g. the lead with a sword and a lance if
one exists in Chapter 1, or the debug Quick Battle). Options, from real
games (check each before showing it):

- A. Arrows ◄ ► either side of the weapon's name, as Fire Emblem's
  forecasts do on the weapon row (Three Houses, Engage).
- B. A key hint on the forecast: "← → weapon" with the player's keys
  (key names drawn as 0250 decided: key caps, built by 0254; if 0254
  isn't done, plain text, and 0254 switches it).
- C. A line in the Attacking lesson: "Swap weapons: {CursorLeft} {CursorRight}"
  (only when the unit has 2+ weapons).
- D. A tip, the first time a unit with 2+ usable weapons opens a forecast.
- E. Combinations, or describe your own.

Nick said "the guide should probably mention it", so a lesson line or tip
(C or D) is likely part of the answer; ask which.

## Scope

**In:** the question; building the chosen look in the forecast view and
skins (ADR-0054, ADR-0038: the forecast is a screen view, not the map);
the lesson line or tip; `combat-arts.md` and `tutorial.md` updated.

**Out (do not do):** changing what ← / → do.

## Implementation steps

1. Mockups from the real forecast view; ask Nick; record the answer.
2. Build it; keys through the keymap (keyboard-input skill: never a
   hard-coded key).
3. Tests.

## Acceptance criteria

- [x] Nick picked; recorded in `combat-arts.md`.
- [x] The forecast shows the swap affordance only when there's more than
      one weapon to swap to (test on the view).
- [x] The lesson line / tip exists if picked (test).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the forecast view's swap flag.
- Snapshot: the forecast with and without a second weapon.

## Completion notes

Nick picked over two rounds of `frame-png` pictures of the real forecast
(a throwaway battle with the test lead's Iron Sword and Steel Sword; both
looks; an art's forecast and the swapped state):

- **Round 1:** "A + B but figure out a way to make it fit better within
  the column" (◄ ► on the weapon touched the enemy's column), and **a tip**
  (D) for the guide.
- **Round 2:** of smaller arrows either side, or a small pair before the
  name, he picked **the pair before the name**.

Recorded in `combat-arts.md` (*Showing that left / right swap the weapon
(0463)*), with pointers in `look-and-feel.md` (*Attack forecast*) and
`tutorial.md` (*Tips*).

Built:

- `ForecastView.swap`: the hint (`Left/Right weapon`, the player's keys
  through the keymap, as the help line names them) when `Targeting::
  can_swap`, else `None`. Filled in `BattleScreen::view`, beside the help
  line's `Left/Right swap`, which shares the key naming (`swap_hint`).
- Glyph skin: `swap_arrows`, a 5-pixel-tall `◂▸` pair of overlays in the
  empty cell before the attacker's weapon, and the hint centred in the
  bottom border, both in the highlight colour. The picture look uses the
  same panel.
- Playdate skin: the same mark in the cell before the weapon and the hint
  in the window's footer.
- The tip: `TipTrigger::FirstWeaponSwap` (the forecast is open and left /
  right swap), `weapon_swap` *Swapping weapons* in `tips.ron`, public and
  private (Zafnok/visions-of-shuyi-assets, branch `t0463-weapon-swap-tip`).

Tests: the view's flag on the lord (two swords) and the archer (one
bow); glyph and Playdate skin tests of the mark and hint, with and
without; a Playdate snapshot; the forecast snapshots of units with two
weapons now show the hint and the mark's overlays (the archer's, with
one bow, doesn't); the tip fires once with two swords and not with one.

*The agent's starting rules (Nick can veto; for the Act 1 playtest,
1064):*

- The hint reads `weapon` also when left / right swap between a weapon
  and a spell (after pointing at an enemy).
- The whole hint is in the highlight colour (the mockup had only the key
  names highlighted); 0254's key caps will style its key names.
- An art's forecast shows the mark and hint when the art can be used with
  more than one weapon.
- The tip's words: "This unit carries more than one weapon, and you can
  try each of them right here in the forecast. The numbers change to
  match before you commit to the attack." (`Swap weapon: {CursorLeft}
  {CursorRight}`), face `neutral`, listed in Guides after Weapon kinds.

Picture: `docs/screenshots/0463-forecast-swap.png` (the lord with two
swords, the knight with one spear, the Playdate), cropped to the
forecast.

The guide's new line has its voice clip (`vix_ccefa1ae`, Chatterbox,
`cargo xtask voice generate --scene vix`), committed with the tip in the
private repository.

Engine friction: estimate 80 production lines (a view field, two skins'
mark and hint, a tip trigger; judged by 0489's tip and 0480's skin
blocks), measured 73 added (0.9×), under 2×. PR size: 565 changed lines.
