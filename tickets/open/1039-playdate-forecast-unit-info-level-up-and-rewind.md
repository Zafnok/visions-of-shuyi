---
id: "1039"
title: "Playdate: the forecast pop-up, unit info, the fight box, EXP and level-up, and the rewind screen"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1038", "1035", "0248"]
nick_input: sign-off
completed:
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

**Sign-off:** Nick looks at the PR's pictures (a forecast with a kill
skull, Unit info for one of his units and one enemy, a level-up, the
rewind list) and, if he wants, plays a fight in the Simulator or on his
Playdate.

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
- [ ] Pictures in the PR; look tests for each.
- [ ] No PC snapshot changes.
- [ ] Nick's sign-off comment is in the completion notes (or "no comment
      yet").
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with the skin): forecast placement on both sides; each page.
- Integration (harness, Playdate skin set): attack with forecast, info,
  rewind; reads the views.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
