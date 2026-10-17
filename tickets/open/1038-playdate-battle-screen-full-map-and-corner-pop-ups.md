---
id: "1038"
title: "Playdate: the battle screen's skin (the map fills the screen, corner pop-ups, menus, questions, banners)"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1037", "1035", "0242"]
nick_input: sign-off
completed:
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

**Sign-off:** Nick looks at the PR's `frame-png --playdate` pictures
(browsing with the pop-ups, the action menu, the map menu, the end-turn
question, a phase banner) and plays the first turns in the Simulator or
on his Playdate if he wants.

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

- [ ] In the Simulator, Chapter 1's first battle shows the map on the
      whole screen, the corner pop-ups follow the cursor and flip down
      near the top, and every menu and question opens and works.
- [ ] Pictures in the PR; look tests for each window.
- [ ] No PC snapshot changes.
- [ ] Nick's sign-off comment is in the completion notes (or "no comment
      yet").
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with the skin): each window; pop-up placement.
- Integration (harness, on the Playdate skin set): browse, open menus,
  end the turn; reads the view.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
