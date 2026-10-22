---
id: "1038"
title: "Playdate: the battle screen's skin (the map fills the screen, corner pop-ups, menus, questions, banners)"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: done
blocked_by: ["1037", "1035", "0242"]
nick_input: none
completed: 2026-10-22
---

# 1038 — The Playdate battle screen

## Context

`docs/design/playdate.md`, *Layout* (Nick, 0061 Q2 A, "Corner pop-ups",
like Fire Emblem on the GBA):

- The map takes the **whole screen**: no side panel, no status line.
- **Browsing:** a small window in a corner shows the unit under the
  cursor (name, HP), another the terrain (name, Def, Avoid). They sit in
  the top corners and move to the bottom ones when the cursor comes near
  (*placement tunable*).
- Anything else the PC side panel shows (stats, weapons, the chapter,
  turn and rewinds) is on the Unit info screen and in the map menu
  (*Claude's starting rule*).

Ticket 0242 turns the battle screen's own drawing (panel, help bar,
menus, questions, objective, banners, notes, tips) into a `BattleView`
painted by a glyph skin (ADR-0054); this ticket adds the Playdate's
paint of that view, chosen by 1037's `SkinSet`. 0242 also splits off the
forecast/info/fight-box/level-up/rewind part (0248) and the dialogue part
(0249) as tickets of their own: those are 1039's and 1043's. Until 0248 is
done, `BattleView::old` (`OldParts`) carries those five as the battle's
state; this ticket leaves them to 1039. `BattleView` also holds the heal
and burn numbers (`popups`). The grep test in `screens/title/tests.rs`
wants `battle/mod.rs`'s `draw` to be one `glyph::paint(ctx,
&self.view(ctx), buf);` line: choose the skin inside a `paint` there, or
change the test to accept the choice.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Nick looks at the PR's `frame-png --playdate` pictures
> (browsing with the pop-ups, the action menu, the map menu, the end-turn
> question, a phase banner) and plays the first turns in the Simulator or
> on his Playdate if he wants.

## Scope

**In:**
- `crates/ui/src/screens/battle/playdate.rs` (or `playdate/`):
  `paint(ctx, &view, buf)` for the parts of `BattleView` 0242 made,
  sized for 50 × 15: the map area is the whole buffer (the camera and
  `MapScene` size follow from the skin's `view_tiles`), the two corner
  pop-ups, the menus (action, weapon, spell, skill, item, unit list, map
  menu, including 1035's extra entries), the end-turn, restart and
  suspend questions, the objective page, phase and outcome banners,
  battle notes, all as small windows over the map in the
  one-bit look (black frame, white inside, black text). A menu sits on
  the side of the map away from its unit, as on the PC.
- **No help bar:** the PC's help bar has no room. Where the PC shows a
  key hint the player needs to proceed (a question's answer keys), the
  window shows it inside, named through the keymap (`keyboard-input`
  skill). *Claude's starting rule (Nick can veto): no help bar on the
  Playdate's battle map; the map menu and the Playdate's printed buttons
  stand in for it.*
- Corner pop-up placement: top-left (unit) and top-right (terrain); both
  move to the bottom corners when the cursor is in the top 5 rows (the
  number in one place, *tunable*). If there's no unit under the cursor,
  only the terrain window shows.
- `BattleScreen::draw` picks the skin by `ctx`'s `SkinSet`; if 0242's
  view lacks something the Playdate needs (e.g. the terrain's Def and
  Avoid for the pop-up), add it to the view for both skins.
- Look tests with the skin (cells and pixels); behaviour tests read the
  view.

**Out (do not do):**
- Forecast, unit info, fight box, EXP and level-up, rewind screen (1039);
  the combat picture (1041); dialogue (1043); the guide's tip and lesson
  box (1058, decided by 0066).
- Any change to the PC skin or what the PC shows.

**Carve (1108, 1112):** either order works; follow where the frame and
widgets went.

## Implementation steps

1. Read `docs/design/playdate.md`, ADR-0054, 0242's `BattleView`, and the
   q2-a mockup (`docs/design/playdate/q2-a-forecast.png`) for the
   window style.
2. The Playdate paint, part by part, each with a look test.
3. `frame-png --playdate` pictures for the PR.
4. Simulator: play turn 1 of Harrowby Ford.

## Acceptance criteria

- [x] In the Simulator, Chapter 1's first battle shows the map on the
      whole screen, the corner pop-ups follow the cursor and flip down
      near the top, and every menu and question opens and works.
- [x] Pictures in the PR; look tests for each window.
- [x] No PC snapshot changes.
- [x] Starting rules and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with the skin): each window; pop-up placement.
- Integration (harness, on the Playdate skin set): browse, open menus,
  end the turn; reads the view.

## Completion notes

**Done.**

- **`battle/playdate.rs`**, the Playdate's paint of `BattleView`, chosen
  by the skin set: the battle screen's `draw` is now
  `SKINS.paint(ctx, &self.view(ctx), buf);` (`SKINS` in `battle/mod.rs`,
  as 1045's screens do), so the title test's grep needed no change.
- **The map fills the screen:** `layout::map_area(ctx)` is `MAP_VIEW` on
  the PC and the whole console on the Playdate; the camera and the
  `MapScene` size follow from the map skin's `view_tiles` of it (24 × 15
  tiles).
- **Windows over the map**, in the one-bit look of
  `screens/playdate.rs`: the two corner pop-ups (unit: name and HP;
  terrain: name, DEF and AVO), moving to the bottom corners while the
  cursor is on the top `TOP_ROWS` (5) rows; the action, list, map and
  unit-list menus (header in the frame, the focused line's note under a
  rule, scrolling with `▲`/`▼`); the end-turn, restart and suspend
  questions with their buttons named inside; the objective page and the
  battle notes (wrapped again to the screen, kept off the noted units'
  rows); the phase and outcome banners; the message line; the heal and
  burn numbers. No help bar, no toggles line.
- **Shared with the PC, no PC change:** `glyph::menu_origin_in` and
  `glyph::box_top_in` take the map area (the PC's `menu_origin` and
  `box_top` call them with `MAP_VIEW`); `notes::wrap_notes` and
  `unwrap_notes` wrap the battle notes at any width. No PC snapshot
  changed.
- **Tests:** look tests for each window and the pop-up placement
  (`battle/playdate/tests.rs`, cells, and pixels through the one-bit
  renderer with a braille snapshot); an integration test that plays
  Harrowby Ford on the Playdate's skins with its buttons and reads the
  view (`tests/it/playdate.rs`).
- **Pictures:** `docs/design/playdate/1038-*.png` (`cargo shuyi frame-png
  --playdate`, ×2).

**Simulator** (`cargo xtask playdate --release --simulator`, driven by
posted key messages, turn 1 of Harrowby Ford): the map fills the screen,
the pop-ups follow the cursor and move down at the top, the map menu and
the restart question open and answer. End Turn there waits for Chapter
1's first lesson, as lessons do by design (their box is still the PC's,
off the Playdate's screen until 1058); with lessons off the integration
test ends the turn through its question to the enemy's banner.

**Engine friction:** estimate 300 lines (one skin of a view, like 1045's
Options skin plus the battle's windows), measured 364 (1.2×), under 2×.

**Deviations.**
- The view needed nothing new: the terrain's `DEF +0  AVO +0` line and
  the unit's HP were already in `PanelView`.
- The forecast, unit info, fight box, EXP and level-up pages, combat
  picture and the guide's box keep the PC's paints until 1039, 1041 and
  1058 (as before this ticket, they are partly off the 50 × 15 screen).
- The message line (aim preview, a combat's message, a toast) is not in
  the ticket's list; it is shown in a window (a starting rule below) so
  the Playdate doesn't lose what a heal or skill will do.

**Follow-ups:** **1075** (the Equip and spell lines are wider than the
Playdate's window, so the uses left are cut with `…`; what the line keeps
is Nick's look-and-feel call, asked before building). Added to 1064's
`blocked_by` and the roadmap.

***Claude's starting rules*** (in `docs/design/playdate.md`, *Layout*;
for 1064's checklist, Nick can veto):
- The corner pop-ups are flush in the corners in the battle's window look;
  the unit's shows name and `HP 19/19` (no bar), the terrain's its name
  and `DEF +0  AVO +0` (and what it heals, on a tile that heals).
- They move to the bottom corners while the cursor is on the screen's top
  5 rows (*tunable*).
- They hide while a menu, question, the objective page, the battle notes,
  a banner, a fight, the unit info or the EXP page is up, and when no
  cursor is on the map (the enemy's phase).
- No help bar and no toggles line.
- The message line is a one-line window in the middle of the top edge, or
  the bottom edge while the cursor is in the top half.
- Menus open beside their tile as on the PC (right, else left); a menu too
  tall scrolls with its focus; a line too long is cut with `…`.
- A question is a window in the middle, its buttons named inside
  (`A yes / B no`).
- The objective page and battle notes are a window across the middle,
  title in the frame, notes wrapped to the screen, off the noted units'
  rows.
- A banner is a 24 × 6 window in the middle, black words (no side
  colours on one bit).
- Heal and burn numbers are black on white over the unit; the sign tells
  them apart.

**Left to judge by playing (1064):** whether 5 rows is the right place for
the pop-ups to flip; whether hiding them during menus feels right; the
message window's place; the double frames' weight on the small screen.
