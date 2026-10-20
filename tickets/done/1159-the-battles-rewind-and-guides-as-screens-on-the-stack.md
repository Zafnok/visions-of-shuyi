---
id: "1159"
title: "The battle's rewind and Guides pop-ups as screens on the stack, with the engine's screen results"
type: feature
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: done
blocked_by: ["engine#2306", "1158"]
nick_input: none
completed: 2026-10-19
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

1158 (done) left the battle with Game Over: a defeat's banner pushes it
over the battle, the battle reads `GameOverChoice` in `uncovered`
(`Rewind` calls `rewind_from_defeat`), and backing out of the rewind
screen pushes it again, skipped. The getters went: the battle pops with
`BattleEnd`. Keep that behavior when the rewind screen moves onto the
stack (ADR-0093).

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

- [x] `grep -n "rewind: Option\|guides: Option\|set_under_scene(false)" crates/ui/src/screens/battle/mod.rs`
      finds nothing.
- [x] The rewind and Guides tests and snapshots pass unchanged (the
      snapshots are untouched; the tests' assertions are, their plumbing
      finds the screens on a stack: see below).
- [ ] `cargo xtask code-lines` shows `trpg-ui` fell. **Not met:** 311
      added, 244 removed (+67 net). See below.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the existing ones for rewind and Guides, unchanged.
- Integration: open rewind, step back two turns, confirm; the battle's
  state is two turns back (an existing test if one does this).

## Completion notes

**Done.** `RewindScreen` and `GuidesScreen` are screens of their own,
pushed over the battle. The rewind screen pops with its `RewindEffect`
(`Close` or `Rewind(point)`) and the battle carries it out in
`uncovered` (`rewound`); closed on a battle still decided, it came from
Game Over, which opens again as it was left (1158's behavior, now without
the `RewindFrom` field: only a lost battle can have the screen open with
an outcome). The battle's `rewind` and `guides` fields, `rewind_step`,
`guides_step`, their branches in `play_key`, `help`, `scene` and `view`
(and `bare()`, and `BattleView`'s `rewind` and `guides` fields) are gone;
`play_key` returns the screen to open (`Opened`: Options, Guides,
Rewind) and `open` pushes it once the lessons have ticked. The rewind
list is on the engine's `ListCursor` and plays its sound where each key
is handled (no more guessing it from the focus before and after). The
tips' flag is now `covered`: set in `covered`, cleared in `uncovered`,
for any screen over the battle (a scene, the rewind screen, Guides,
Options, Game Over), which is what the old per-frame reset rebuilt.

**Deviations.**

- **Not overlays.** Both screens paint the whole screen, so they are
  opaque: the rewind screen paints its own map (the battle as it was
  before the highlighted action, seen through the battle's camera and
  look, `MapFrame`), and the Guides box already covered the map and the
  side panel. Drawing the battle under them would be thrown away (and
  costs time on the Playdate).
- **Guides pops plainly.** Closing it changes nothing in the battle (its
  map menu stays open under it), so it pops with no value;
  `GuidesEffect` stays as the list's step result.
- **The lesson over the rewind screen** is the battle's lesson box as the
  screen opened, handed to it (`with_lesson`): nothing can tick it while
  the screen is open, and the closing tick happens in `uncovered`.
- **Whole-screen views.** To keep ADR-0054's `glyph::paint(ctx,
  &self.view(ctx), buf)` (the title test checks it), each screen's
  `view(ctx)` is now its whole screen (`RewindScreenView`,
  `GuidesScreenView`: the list, the help, the debug hint, and for rewind
  the map and the lesson); the list views are `list_view`.
- **Tests.** Unit tests that read `battle.rewind()`/`guides()` drive the
  battle on a real stack (`testing::Stacked`, which derefs to the battle
  but never `DerefMut`, so a helper can't update the battle past the
  screens over it); integration tests find the screens with
  `h.screen::<RewindScreen>()`. Assertions are unchanged, except: a
  Rewind, Confirm, Confirm sent in one frame is now one key a frame; a
  "Rewind does nothing here" check reads the transition (`answer`); the
  stack shows `rewind` on top where it used to show `battle`; and one tip
  test gives the battle a frame to take in a tip fired by hand.

**Player-visible differences (both tiny).** Keys pressed in the same
frame after the one that opens the rewind screen or the Guides list are
dropped, as for every pushed screen (Options, scenes, Game Over): only
two presses within one frame, or the crank wound several notches in the
frame it opens the rewind (it opens on the newest; the next notches step
it a frame later). And the battle waits under them as it does under
Options: a toast still up when Guides opens has its time left after.

**Friction: 2.1×, accepted** (estimate 150 added lines, measured 311;
row in `docs/engine/friction.md`). The battle lost about the 130 lines
the sweep counted, but each pop-up now pays what every screen pays (a
`Screen` impl, a whole-screen view, its painter) and the rewind screen
carries the battle's camera to paint its own map, so `trpg-ui` rose by
67 lines net instead of falling. The other pop-ups (tips, notes, banner,
progress) were left as they are: none is the same few lines.

**Nick:** nothing to decide, and nothing should look different: open
the rewind screen (r) and the Guides list (map menu) in a battle on the
Pages build and check they work as before.
