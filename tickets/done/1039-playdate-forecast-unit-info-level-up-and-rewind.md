---
id: "1039"
title: "Playdate: the forecast pop-up, unit info, the fight box, EXP and level-up, and the rewind screen"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: done
blocked_by: ["1038", "1035", "0248"]
nick_input: none
completed: 2026-10-22
---

# 1039 — Forecast, unit info, level-up and rewind on the Playdate

## Context

`docs/design/playdate.md`, *Layout*: **the forecast** is a window over
one side of the map (the side away from the two units), with the same
contents as the PC forecast (`docs/design/look-and-feel.md`, *Attack
forecast*): both sides' HP, Hit, Crit, the strikes in order, the kill
skull. Picture: `docs/design/playdate/q2-a-forecast.png`. The PC side
panel's other contents (stats, weapons, chapter, turn, rewinds) are on
the **Unit info** screen and in the map menu (*Claude's starting rule*).

0242 made the battle screen's views; ticket 0248 gives the forecast, unit
info (`info.rs`), fight box (`playback.rs`), EXP bar and level-up page
(`progress.rs`) and rewind screen (`rewind.rs`) theirs. **This ticket
needs 0248 done too.** 1038 painted the rest
of the battle screen; 1035 made the crank drive the rewind screen.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Nick looks at the PR's pictures (a forecast with a kill
> skull, Unit info for one of his units and one enemy, a level-up, the
> rewind list) and, if he wants, plays a fight in the Simulator or on his
> Playdate.

## Scope

**In:**
- Playdate paints, chosen by `SkinSet`, sized for 50 × 15:
  - **forecast window** on the side away from the two units, with every
    line of the PC forecast;
  - **Unit info** as a full-screen page (the side panel's new home for
    a unit): what the PC info screen and side panel show for the unit
    (stats, weapons, items, skills), paged with the cursor if it doesn't
    fit 15 rows. The chapter, turn and rewind charges go in the map
    menu's header (1038's map menu; add them here if 1038 didn't).
    *Claude's starting rule, from the design's "on the Unit info screen
    and in the map menu": the unit's own things on Unit info, the
    battle's in the map menu;*
  - **fight box** (the strikes as they play, when the combat picture is
    off or skipped);
  - **EXP bar and level-up page**;
  - **rewind screen**: the list of past actions as a window over one
    side, the map behind showing the battle as it was (as on the PC),
    the charges line and the "Use 1 of N charges?" question.
- If a view lacks something the Playdate needs, add it to the view for
  both skins.
- Look tests with the skin; behaviour tests read the views.

**Out (do not do):**
- The combat picture (1041).
- Any change to the PC skin or to what the PC shows.

**Carve (1108, 1112):** either order works; follow where the frame and
widgets went.

## Implementation steps

1. Read the design doc, 0242's (b) ticket's views, `forecast.rs`,
   `info.rs`, `playback.rs`, `progress.rs`, `rewind.rs`.
2. Each paint with its look test.
3. `frame-png --playdate` pictures for the PR.
4. Simulator: attack, see the forecast, level up, rewind with the crank.

## Acceptance criteria

- [ ] In the Simulator: a forecast before an attack, Unit info from the
      map menu, a level-up, and a rewind by crank all show and work.
      *All but the level-up, which takes 100 EXP to reach (see
      Simulator below); its look tests cover it.*
- [x] Pictures in the PR (all but a level-up, for the same reason); look
      tests for each.
- [x] No PC snapshot changes.
- [x] Starting rules and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with the skin): forecast placement on both sides; each page.
- Integration (harness, Playdate skin set): attack with forecast, info,
  rewind; reads the views.

## Completion notes

**Done.**

- **Five Playdate paints**, chosen by the skin set, one module each in
  `battle/playdate/`: `forecast.rs`, `info.rs`, `fight_box.rs`,
  `progress.rs` (EXP bar, level-up and class pages) and `rewind.rs`. The
  battle's Playdate skin calls them instead of the PC's paints; the
  rewind screen's `draw` is `SKINS.paint(...)` (`rewind::SKINS`) and its
  map is sized by `layout::map_area` (the whole screen on the Playdate,
  `MAP_VIEW` on the PC, as before).
- **What the views gained, for both skins** (the PC paints ignore them):
  the forecast's two units' tiles (`ForecastView::tiles`), the info
  screen's page (`InfoView::page`, always 0 on the PC) and the map menu's
  turn and rewind charges (`BattleView::map_info`). The page turns with
  Left and Right in the battle screen (`info::turn_page`, as many pages
  as `layout::info_pages` says: 1 on the PC, so nothing changes there).
  The charges line is now one function (`rewind::charges_text`) shared by
  the rewind screen and the map menu.
- **Shared with the PC, no PC change:** the rewind list's wrapping and
  scrolling take a width (`glyph::rewind::wrap_to`, `first_shown`).
- **Tests:** look tests for each paint (`battle/playdate/*/tests.rs`, and
  the fight box and map menu in `battle/playdate/tests.rs`): the
  forecast's placement on both halves, its rows, the kill skull in pixels
  through the one-bit renderer, the rules and notes cut when too tall;
  each info page; the EXP bar, the level-up page (with and without a
  face) and the class box; the rewind list, its scrolling and its
  question. An integration test plays the Quick Battle on the Playdate's
  skins and reads the views: the map menu's turn and charges, Unit info's
  pages, the forecast clear of the target, the fight skipped to the EXP
  bar, the crank's rewind screen and its question
  (`tests/it/playdate.rs`). No PC snapshot changed.
- **Pictures:** `docs/design/playdate/1039-*.png` (`cargo shuyi
  frame-png --playdate`, ×2). The rewind screen opens only by the crank
  on the Playdate and the engine's frame-png script has no crank step, so
  the game's frame-png gained two options, `--crank DEGREES` and
  `--then-pad BUTTONS`, played after the steps (`crates/xtask/src/
  game_tools.rs`).

**Simulator** (`cargo xtask playdate --release --simulator`, driven by
window messages to the Simulator's own window and controls, Chapter 1's
first battle): the map menu showed `Turn 1` in its frame and `3 of 3
charges left` under it; Unit Info opened Ellery's info, right turned the
three pages and wrapped, down showed Aske on the same page, B closed it;
with Aske moved and waiting, the crank (undocked, wound back with the
Simulator's crank control) opened the rewind window over the map showing
Aske back where it stood, A asked "Use 1 of 3 charges?" with `A rewind ·
B back` inside, and A rewound; Tamsin moved beside a brigand and Attack
showed the forecast on the right half, the two units on the left. **A
level-up was not reached in the Simulator** (it takes 100 EXP, several
fights): the level-up page is covered by its look tests, with and without
a face, and is not in the pictures for the same reason (frame-png can't
reach one quickly either).

**Engine friction:** estimate 450 production lines (five paints of
existing views and a few view fields, judged by 1038's battle skin, 364
lines for a dozen windows), measured 776 (1.7×), under 2×. The signal: the
game's `--crank` / `--then-pad` frame-png options work around the
engine's frame-png script having no crank step (outcome 1, engine
feature; leak test passed). Shipped with the workaround marked
`// engine-friction: engine#2540`; engine ticket 2540
([Zafnok/shuyi-engine#66](https://github.com/Zafnok/shuyi-engine/pull/66))
and game ticket 1182 (deletes it, moves the pin). Row in
`docs/engine/friction.md`.

**Deviations.**
- The forecast window is **half the screen wide** (25 cells), not as
  wide as its lines like the mockup: a wider window's two places overlap
  in the middle of the screen, where it then covers both units.
- **No chapter in the map menu:** the ticket names "the chapter, turn and
  rewind charges", but the PC's battle screen shows no chapter; the turn
  and the charges are there.
- **Unit info has no "items" line:** the PC's info screen shows the
  weapons, armor, accessory and spells, not the pack; the Playdate shows
  the same.
- The **fight box** shows only when the combat picture is off or skipped
  mid-fight (with combat animations off a fight is over at once, as on
  the PC), so the integration test checks the fight skipped to the EXP
  bar and the box has its look test.

**Follow-ups:** engine#2540 (a crank step in frame-png's script) and **1182** (the game's frame-png uses it and drops its own `--crank` and `--then-pad`; blocked by engine#2540). Neither is on the Act 1 critical path.

***Claude's starting rules*** (in `docs/design/playdate.md`, *Layout*;
for 1064's checklist, Nick can veto):
- The forecast is a window half the screen wide down the half away from
  the two units, from the top; with one unit in each half, on the
  attacker's half so the target stays in view. Every line of the PC's;
  the HP bar is 4 cells (kept black, would-be-lost a checker, the rest
  white); the strikes after a kill are grey dots; when the rows don't fit,
  the rules go first, then the last notes.
- Unit info is a full-screen window with the unit's name in its frame,
  three pages that left and right turn (wrapping, `◀ 1/3 ▶` in the
  frame): the unit and its stats and effects; its arms, gear, ranks and
  affinities; its skills and spells. It opens on page 1; up and down
  change the unit and keep the page. A raised or lowered stat is the bar.
  No portrait placeholder.
- The map menu has the turn in the top of its frame and the rewind
  charges in its bottom.
- The fight box, the EXP bar and the class box are windows across the top
  of the screen (as on the PC); the striker's name is the bar while it
  flashes.
- The level-up page is a window in the middle: the face at 1× top left,
  the banner, name and what changed beside it, the stats two to a row,
  each gain the bar; without a face the words take its place.
- The rewind list is a window half the screen wide down the right side,
  full height; the highlighted entry is the bar; "Use 1 of N charges?" is
  a question in the middle with its buttons named inside; no help line
  otherwise.

**Left to judge by playing (1064):** whether the forecast covering the
attacker when the two units stand either side of the screen's middle is
acceptable; whether three info pages and turning them with left and
right feel right, and what each page should hold; whether the 4-cell HP
bars read on the device; whether the rewind window hiding the right of
the map (the battle as it was) is a problem; the level-up face at 1×
(the one-bit faces are 1042's).
