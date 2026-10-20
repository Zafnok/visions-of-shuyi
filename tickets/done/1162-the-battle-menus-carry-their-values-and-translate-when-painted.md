---
id: "1162"
title: "The battle's menus carry their values and translate when painted: told(), IN_PLAY and the parallel lookup lists go"
type: feature
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: done
blocked_by: ["engine#2308", "1161"]
nick_input: none
completed: 2026-10-19
---

# 1162 — The battle's menus carry their values

## Context

The first engine-friction sweep (1153, `docs/engine/friction.md`) found
that the engine's `MenuItem.label` is a `String` baked when the menu is
built. The battle's input state (`crates/ui/src/screens/battle/mode.rs`)
is pure and doesn't know the player's language, so:

- it builds its menus in English (`IN_PLAY`, used 17 times) and
  `Mode::told()` rebuilds them translated for painting (about 45
  lines). `BattleScreen::menu_view` (`mod.rs`) calls `told` on every
  draw, which outside English clones the whole `Mode` and re-runs
  `skill_choices` each frame;
- seven menus keep a list beside the `Menu` of what each line stands
  for (`entries`, `choices`, `lines`, `groups.get(i)`) and look the
  chosen index up in it.

Engine ticket 2308 makes `Menu<T>` carry a value per item and lets a
label be a text key translated by the skin. 1161 first folds the five
list modes into one, so there is one place to change.

## Nick input

None.

## Scope

**In:**
- Move the pin past 2308 (`engine-change` skill, step 7) and fix what
  its breaking change touches in the game.
- The battle's menus are `Menu<T>` with keyed labels; `told()`,
  `IN_PLAY` and the parallel lists go; Confirm reads the chosen value.
- Other menus in the game that keep a parallel list
  (`grep -rn "MenuEvent::Chosen" crates/ui/src`) move too where it is
  a few lines each.

**Out (do not do):**
- Any change the player sees.

## Implementation steps

1. Write the estimate: about 100 lines removed, 30 added.
2. Move the pin; fix the build.
3. The battle's menus, then the others.

## Acceptance criteria

- [x] `grep -rn "fn told\|IN_PLAY" crates/ui/src` finds nothing of the
      battle's: `Mode::told` and `IN_PLAY` are gone. It still finds the
      dialogue screen's `DialogueScreen::told`/`told_overlay` and
      `dialogue::view::told` (they tell a scene; nothing to do with
      menus), left as they are.
- [x] Every snapshot is unchanged, including those in the `test`
      language.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The existing tests and snapshots, unchanged in what they check.

## Completion notes

- **Pin** moved to the engine's `main` (`939f023`), past 2308 and also
  2309–2311. Besides 2308's breaking change (menus and the text box's
  skins take the `Ctx`; labels are a `Label`), 2309 changed the
  question view (`AskView`: `keys`, `answers`), which the Playdate skins
  of Options and Key bindings read; fixed.
- **Battle** (`screens/battle/mode.rs`): the action menu is a
  `Menu<MenuEntry>` and the map menu a `Menu<MapEntry>`, both with keyed
  labels the skin words when it paints. The unit list is a
  `Menu<UnitId>`. A list the action menu opens is a `Menu<Line>`, where
  `Line` is an art, spell, skill, pack group or weapon to equip; the
  target pickers keep that menu (for Cancel) and the chosen line,
  instead of the menu, its lines and an index. `entries`, `units`, the
  lists' `kind` and every `get(i)` lookup are gone; Confirm reads the
  chosen value. `told()` and `IN_PLAY` are gone; `menu_view` no longer
  clones the mode or re-runs `skill_choices` each frame.
- **Deviation:** an art's, spell's, item's or unit's name is data text
  (`Words`), not a screen text key, so those lists can't hold keys. They
  are written in the player's language when they open: `step_with` and
  `Mode::tick` take the language (the screen gives its own), `step`
  stays English for the tests. A skill line's reason is now the
  `CostError` itself, worded in the menu. Only the Options screen
  changes the language, and it opens from the map menu, which is keyed,
  so no open list goes stale; a list open while the debug tools are
  toggled shows the old language until it reopens (debug only).
- **Other menus:** the title, Game Over, mode select, familiarity, the
  save prompt and the dialogue's replies carry their values; `pick_one`
  pops with the chosen value (no values list). Their labels stay text
  (already in the player's language when built).
- **Tests:** the two `told()` tests became "the action and map menus
  hold keys the skin tells" and "a list is written in the language it
  opens in". `a_menu_line_with_no_entry_behind_it_does_nothing` is
  gone: a line without a value can't be built any more. Test helpers
  `english`/`text_of` (menu labels) and `Values`/`Lines` (a menu's
  values) are new.
- **Engine friction:** estimate 30, measured 313 added (10.4×), 381
  removed: accept, no engine glue (see `docs/engine/friction.md`).
- No follow-up tickets. Nothing for Nick to check: nothing the player
  sees changed.
