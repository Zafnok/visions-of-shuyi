---
id: "0242"
title: "The battle screen's panels and the dialogue screen: say what they show as a view, painted by a skin"
type: feature
milestone: M1 Engine
model: opus-5.5
effort: high
status: todo
blocked_by: ["0241"]
nick_input: none
completed:
---

# 0242 — Battle panels and the dialogue screen as view + skin

## Context

Last of three tickets that give every screen the split the Options screen
has (ticket 0805, Nick 2026-10-04: a bought UI pack must be able to
replace the look "without affecting logic").
[ADR-0054](../../docs/adr/0054-screens-say-what-they-show-as-a-view-a-skin-paints-it.md)
is the pattern; Options is the worked example; 0240 and 0241 did the
menus.

The battle *map* is already a skin (ADR-0038: `MapScene`, `MapSkin`). What
is left is everything the battle screen draws around and over the map,
straight into the buffer, and the dialogue screen.

## Nick input

None. Nothing the player sees changes.

## Scope

**In:**
- The battle screen's own drawing in `crates/ui/src/screens/battle/`: the
  side panel (`panel.rs`), the help bar and message row (`mod.rs`), the
  action, weapon, spell, skill, item and map menus and the unit list, the
  end-turn, restart and suspend questions, the objective page, the
  forecast (`forecast.rs`), the unit info screen (`info.rs`), the fight
  box (`playback.rs`'s drawing), the EXP bar and level-up page
  (`progress.rs`'s drawing), phase and outcome banners (`banner.rs`),
  battle notes (`notes.rs`), the rewind screen (`rewind.rs`), tips
  (`crate::tips::draw_tip`).
- `DialogueScreen` (`crates/ui/src/screens/dialogue.rs`): the text box,
  speaker, caption, choices and the skip question. `DialoguePlayer`
  already gives a `View` of what is said; this adds what the *screen*
  adds (how much is revealed, the open choice menu, the question).
- Removing `Menu::draw` once nothing calls it.

**Out (do not do):**
- The map itself, its skins, `MapScene` (ADR-0038: done).
- Any change to what is shown, its text, keys, timing or layout.
- A second skin or a way to choose skins.
- The combat scene with full-body art (0413): *either order works.* If
  0413 is done, its scene is already data painted by the map skins or its
  own painter: leave it. If not, 0413 builds on the fight box's view from
  this ticket.

This is large. Split it at the start rather than half-way: (a) panel,
help bar, menus, questions, objective, banners, notes, tips; (b) forecast,
info, fight box, EXP and level-up, rewind; (c) dialogue. Do (a) here and
write (b) and (c) as tickets of this pattern, each blocked by the one
before, unless (a) turns out small enough to take (b) with it.

## Implementation steps

1. `battle/view.rs`: `BattleView` holding the map area's `MapScene`
   (unchanged) and an `Option` or list for each thing in scope that is on
   screen this frame: `panel`, `help`, `status`, `toast`, `menu` (a
   `MenuView` with its anchor tile), `question`, `objective`, `banner`,
   `notes`, `tip`, … Each is plain data built from the battle state and
   the screen's `Mode`; values are meanings (an HP bar is `{ hp, max }`,
   a forecast line is numbers and flags).
2. `battle/glyph.rs`: `paint(ctx, &view, buf)`, which asks the map skin to
   paint the scene (as `draw` does today) and paints the rest. Layout
   constants (`layout.rs`'s rects, box sizes, columns) belong to the skin.
3. `BattleScreen::view(&self, ctx)`; `draw` becomes one call. The
   screen's existing accessors used by tests (`help`, `status`, `banner`,
   `toast`, …) stay and feed the view.
4. Dialogue: `dialogue/view.rs` extends what `DialoguePlayer::current()`
   gives with the revealed length, the choice `MenuView` and the skip
   question; `dialogue/glyph.rs` paints it (portraits through
   `portrait::draw_portrait`).
5. Tests: behaviour tests read the view (or the scene, as now); tests of
   the look move to the skin's module. The many battle tests that read a
   help line or a panel row through cells (`row(&h, 30)`, `panel(&h, 5)`)
   read the view's field instead; keep a few in the skin's tests to pin
   where those rows are painted.

## Acceptance criteria

- [ ] `battle/mod.rs`, `battle/mode.rs` and `dialogue.rs` don't import
      `UiColor`, `Rect`, `BoxStyle` or `Cell`; each `draw` is one call to
      its skin (extend 0240's grep test).
- [ ] No snapshot file changes.
- [ ] The view has unit tests for each thing in scope appearing and
      disappearing (a menu opens, a banner expires, the forecast for a
      target, …).
- [ ] `Menu::draw` is gone.
- [ ] `cargo xtask check-keys` and `check-text` pass; the `check-text`
      limit isn't raised.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the view's content per state; the skin's placement.
- Property: none.
- Snapshot / integration: the existing snapshots, unchanged; Harness tests
  read the view for behaviour.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
