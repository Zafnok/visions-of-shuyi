---
id: "1162"
title: "The battle's menus carry their values and translate when painted: told(), IN_PLAY and the parallel lookup lists go"
type: feature
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: todo
blocked_by: ["engine#2308", "1161"]
nick_input: none
completed:
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
  its breaking change touches in the game. *Done by 1163*: the pin is at
  `939f023` and the build was adapted the smallest way, which this
  ticket undoes: `MenuEvent::Chosen(())` read through `menu.focus()`,
  and `LabelText::as_str` (`widgets/menu.rs`, in the prelude) reading a
  label back as text.
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

- [ ] `grep -rn "fn told\|IN_PLAY" crates/ui/src` finds nothing.
- [ ] Every snapshot is unchanged, including those in the `test`
      language.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing tests and snapshots, unchanged in what they check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
