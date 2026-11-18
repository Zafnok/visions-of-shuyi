---
id: "1286"
title: "The aimed spell, item or skill's preview in the battle side panel too"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0445"]
nick_input: decision
completed:
---

# 1286 — The aimed spell, item or skill's preview in the battle side panel too

## Context

While the player aims a spell, an item or a skill that has no combat
forecast, one line says what it will do: `Forest → Burning (1 round)`,
`Potion on Test Knight: HP 14 → 19` (`BattleScreen::message`, built by
`CastTargeting::preview` in `crates/ui/src/screens/battle/magic.rs`,
`ItemTargeting::preview` in `items.rs` and `SkillTargeting::preview` in
`skills.rs`). It is printed at the left of the bottom bar's toggles row.

Ticket 0445 kept it there (Nick, 2026-11-17, looking at a rendered frame
of Fire aimed at a forest): "I think as it is today but make it not
conflict with any new additions and it should also show in the right
panel somehow". The first half is 0445's; this ticket is the second:
the same preview in the side panel as well, while aiming.

Today the side panel, while aiming at a tile, shows the hover panel of
the tile (its terrain, and the unit on it if any): `panel_view` in
`crates/ui/src/screens/battle/mod.rs`, painted by `paint_panel` in
`glyph.rs`. The look is Nick's (`docs/design/look-and-feel.md`; he judges
pictures).

## Nick input

**Decision, before building.** Render options and ask Nick to pick
(`ask-nick`; frames from `cargo shuyi frame-png`; the Quick Battle's
Test Mage has Fire, Frost and Heal: `--battle quick --wait 3 --act
NextUnit --act Confirm --act Confirm --act CursorUp --act CursorUp --act
Confirm --act Confirm` aims Fire at a forest). Suggested options, not
decisions (render the glyph and the picture look):

- (a) a titled block at the top of the panel (the spell's name, then the
  preview line, wrapped to the panel), the tile's hover panel under it;
- (b) the preview in the panel's last rows, above `Gold`, the hover panel
  unchanged;
- (c) for a spell or item aimed at a unit, the unit's HP line shows the
  change (`HP 14 → 19`) where it already is, the preview line under it;
  at a tile, as (a).

No check after merge: verified by tests and `frame-png` pictures in the
PR; what is left to judge by playing goes on 1064's checklist.

## Scope

**In:**
- The preview shown in the side panel while aiming a spell, item or
  skill, as Nick picks; the bottom-bar line stays.
- `docs/design/look-and-feel.md` records his pick.

**Out (do not do):**
- The words of the previews (their text keys stay as they are).
- The combat forecast (attacks have their own panel).
- The bottom bar's layout (0445).

## Implementation steps

1. Options for Nick (above); wait for his answer.
2. Add the preview to the panel's view (a field of `PanelView` or a new
   part of `BattleView`, plain data), built in `view` from the same
   `preview` calls `message` makes; the glyph skin paints it (ADR-0054:
   the logic decides what, the skin how).
3. Wrap it to the panel's inner width with the existing wrap helper the
   panel uses for long lines.
4. Update the battle snapshots that aim a spell, an item or a skill;
   read each changed one.

Expected size: about 60 production lines (one view field, its paint, a
wrap), judged by the map-info box of the map menu (`map_info`,
`MapInfoView`).

## Acceptance criteria

- [ ] Aiming Fire at a forest in the Quick Battle, the side panel shows
      `Forest → Burning (1 round)` as Nick picked, and the bottom bar
      still shows it too.
- [ ] The same for an item (a potion on an ally) and a skill.
- [ ] A preview longer than the panel is wrapped, never cut or run over
      the border (a test).
- [ ] `look-and-feel.md` records Nick's pick.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the panel view while aiming a spell, an item and a skill, and
  while not aiming (no preview).
- Snapshot: the changed battle snapshots, reviewed; one with a long
  preview.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
