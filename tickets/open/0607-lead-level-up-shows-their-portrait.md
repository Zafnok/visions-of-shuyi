---
id: "0607"
title: The lead's level-up and promotion pages show the lead's portrait
type: bug
milestone: M5 Progression
model: sonnet-5
effort: medium
status: todo
blocked_by: []
nick_input: sign-off
completed:
---

# 0607 — The lead's level-up page shows their portrait

## Context

Nick, Chapter 1 playtest round 2 (0804, 2026-10-14): "seems main
character has no lv up portrait idk why".

Cause (found while ticketing): the level-up and promotion pages look the
portrait up by the unit's raw character id. The lead's id is `"lead"`
(`trpg_core::lead::LEAD_ID`), but the lead's portraits are stored as
`lead_m` and `lead_f` (`assets/portraits/`). Every other screen maps the id
first with `LeadProfile::portrait_for` (`crates/core/src/lead.rs`):
the dialogue screen (`screens/dialogue.rs`, `draw_side`), the cinematic
faces (`cinematic/faces.rs`), the combat stage (`screens/battle/stage.rs`)
and the map sprites (`map_view/sprite/units.rs`). The progress pages don't:

- `crates/ui/src/screens/battle/progress.rs`, `Progress::pages`: fills
  `LevelUpPage.character` / `PromotionPage.character` with the raw id.
- `progress.rs`, `draw`: `p.character.as_ref().and_then(|c| portraits.get(c))`
  for both pages, so the lead gets the `battle.portrait` placeholder box.
- `draw` is called from `screens/battle/mod.rs` (level-up in battle),
  `screens/results/glyph.rs` (the results screen) and
  `screens/class_change.rs` (the promotion result). All three fail for the
  lead.

## Nick input

**Sign-off:** level the lead up in Chapter 1 (or on the results screen):
the lead's face shows, the one picked on the lead-select screen.

## Scope

**In:**
- The lead's portrait (the gendered one, `ctx.lead.portrait_for`) on the
  level-up page, the promotion page, in battle, on the results screen and
  on the class-change result.
- A failing test first that shows the bug.

**Out (do not do):**
- The unit info screen's placeholder box (it shows the placeholder for
  every unit; not this bug).
- Any change to the pages' layout.

## Implementation steps

1. Write the failing test: a `Progress` built for a unit whose character
   is `LEAD_ID`, drawn with a portraits map holding `lead_m`; it shows the
   portrait, not the placeholder (read the page's view or the sprite items
   drawn, per ADR-0038, not cells).
2. Map the id when the pages are built: give `Progress::pages` (or its
   caller) the `LeadProfile` and store `lead.portrait_for(id)` in
   `LevelUpPage.character` / `PromotionPage.character`; or pass
   `&LeadProfile` to `draw` and map at the lookup. Pick the one that keeps
   the three call sites simplest; all three have `ctx.lead`.
3. Cover the three call sites (in battle, results, class change) with one
   test each, or one shared test of the function they all call plus a
   Harness test through the real game for one of them.

## Acceptance criteria

- [ ] A test with the lead (male and female profile) fails before the fix
      and passes after it.
- [ ] Harness: the lead levels up in a battle and the level-up page shows
      `lead_m` (or `lead_f` for a female lead).
- [ ] The results screen and the class-change result show it too (test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the page's portrait id for the lead and for another character.
- Snapshot / integration: Harness level-up of the lead.

## Completion notes

*(Filled in by the session that completes the ticket.)*
