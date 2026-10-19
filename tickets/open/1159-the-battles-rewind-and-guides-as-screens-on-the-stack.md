---
id: "1159"
title: "The battle's rewind and Guides pop-ups as screens on the stack, with the engine's screen results"
type: feature
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: todo
blocked_by: ["engine#2306", "1158"]
nick_input: none
completed:
---

# 1159 — The battle's rewind and Guides as screens on the stack

## Context

The first engine-friction sweep (1153, `docs/engine/friction.md`)
measured the battle screen, `crates/ui/src/screens/battle/mod.rs`, at
2009 production lines against an estimate of 1700 (1.2×). Most of it is
the real job, but about 130 lines exist because the engine's
`Transition::Pop` carries no result:

- the rewind and Guides pop-ups are `Option` fields of `BattleScreen`
  (`rewind`, `guides`), not screens on the stack, because their answer
  has to change the battle below. So key priority is written out by
  hand (`overlay_key`, `progress_key`, `play_key`, and the order in
  `update`), `rewind_step` guesses the menu sound from the focus before
  and after (the engine's `Menu::handle_with_sound` does it), and
  `bare()` fills the 22 fields of `BattleView` by hand for the two
  pop-ups;
- `self.tips.set_under_scene(false)` at the start of every update and
  `set_under_scene(true)` when a scene is pushed rebuild "uncovered".

Engine ticket 2306 adds `Transition::PopWith`, `Screen::uncovered` and
`Screen::covered`. 1158 moves the pin past it first.

## Nick input

None.

## Scope

**In:**
- `RewindScreen` and `GuidesScreen` are pushed on the stack as overlays
  and pop with their effect (`RewindEffect`, `GuidesEffect`); the
  battle applies it in `uncovered`.
- The `rewind` and `guides` fields, their key routing and their
  branches in `bare()` and the view go. The rewind list uses
  `Menu::handle_with_sound` (or the list cursor, if 1160 is done).
- The tips' under-scene flag is set in `covered` and cleared in
  `uncovered`.
- The battle's `*_requested()` getters, if 1158 left them, become
  `PopWith` values the flow reads.

**Out (do not do):**
- Any change the player sees: the same keys open, move and close the
  pop-ups, the same sounds play, the same help lines show.
- The battle's other pop-ups (tips, notes, banner, progress): leave
  them unless one turns out to be the same few lines; then say so in
  the Completion notes.

## Implementation steps

1. Write the estimate: about 130 lines removed from `mod.rs`, a few
   added to `rewind.rs` and `guides.rs`.
2. Make each pop-up a `Screen` with `is_overlay`, and push it where the
   battle sets the field today.
3. Apply the effect in `uncovered`; delete the fields and routing.
4. Snapshots: the battle with each pop-up open must not change.

## Acceptance criteria

- [ ] `grep -n "rewind: Option\|guides: Option\|set_under_scene(false)" crates/ui/src/screens/battle/mod.rs`
      finds nothing.
- [ ] The rewind and Guides tests and snapshots pass unchanged.
- [ ] `cargo xtask code-lines` shows `trpg-ui` fell.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the existing ones for rewind and Guides, unchanged.
- Integration: open rewind, step back two turns, confirm; the battle's
  state is two turns back (an existing test if one does this).

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
