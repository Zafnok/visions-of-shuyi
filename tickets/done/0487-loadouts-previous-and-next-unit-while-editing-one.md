---
id: "0487"
title: "Loadouts: previous and next unit while editing one"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-10
---

# 0487 — Loadouts: previous and next unit while editing one

## Context

Nick's playtest of the Act 1 build (2026-11-08):

> "In the Loadout in preparation, there should be a keybind when you are
> setting up a character to quickly swap between them (i.e. q/e)"

Preparations → Loadouts today (`crates/ui/src/screens/preparations.rs`,
`handle()` ~:730–820): the focus goes Tabs → Units → Slots → Stock. Up and
Down in Units pick the unit (~:775, which also puts the slot back to the
first), Confirm goes to its Slots (~:779), Confirm on a slot to the Stock
for it, and Cancel steps back one level (~:784, ~:799, ~:806). To change
the next unit's gear the player backs out twice, moves down, and goes in
twice again.

The screen's actions (`crates/ui/src/screens/actions.rs` ~:119,
`PREPARATIONS`) are the cursor, Confirm, Cancel and Options. The game
already has **Previous unit** and **Next unit** actions (`PrevUnit`,
`NextUnit`, `crates/content/src/keymap.rs` ~:74–77), used on the battle
map, the unit info screen (`battle.help.info`: "{NextUnit} next unit ·
{PrevUnit} previous"), target picking and the world map. Their default
keys are Nick's (`docs/design/controls.md`; `assets/data/keymap.ron`):
`a` / `s` right-handed, `;` / `l` left-handed, the shoulder buttons on a
pad, and `West+DpadLeft` / `West+DpadRight` (B + left / right) on the
Playdate. Nick wrote "q/e" as an example of a key pair: the ticket uses
these two actions, so the player's own keys work and the help bar names
them. No default changes (a default key is Nick's call, `keyboard-input`
skill).

## Nick input

None. Verified by the Harness and view tests below and a `frame-png`
picture of the Slots focus with its help bar in the PR. Whether it feels
right goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- `PrevUnit` and `NextUnit` added to `PREPARATIONS` and handled on the
  Loadouts tab in the Units, Slots and Stock focus: the previous / next
  unit of `units()`, wrapping at both ends, with the move sound.
- **The focus level stays** (Slots stays Slots, Stock stays Stock), and so
  does the cursor row: the slot row index (clamped to the new unit's
  slots), and in Stock the stock row index (clamped to the new list). If
  the new unit's stock list for that slot is empty, the focus goes back to
  Slots, as Confirm would have refused it (`NOTHING_IN_STOCK`).
- The message line clears as on a cursor move.
- The help lines of Units, Slots and Stock (`prep.help.units`,
  `prep.help.slots`, `prep.help.stock`, `ui.ron` ~:368–370) name the two
  keys through the keymap: e.g. `{PrevUnit} {NextUnit} unit`, as the world
  map's `{PrevUnit} {NextUnit} places`.
- On the Playdate (keymap's `playdate` table), B + left / right work the
  same; the chord's B doesn't also back out (the engine holds B's Cancel
  back until it is let go alone, `keyboard-input` skill, *The Playdate*).
- `docs/design/weapons-and-items.md`, *On the Preparations screen*
  (~:351): one sentence for the keys.

**Out (do not do):**
- Changing any default key or adding an action.
- The Pack and Shop tabs (no unit there).
- The Loadouts lines' content (0478, 0479) and moving the lists onto the
  engine's panels (1238). Either order with each: they don't touch
  `handle()`'s unit stepping; if 1238 is done first, step the unit in the
  panel's list it made.
- The Playdate skin of Preparations (1046). Either order: the view
  doesn't change; if 1046 is done, check its help line fits.

## Implementation steps

1. Read the `keyboard-input` and `shuyi-input` skills and `handle()`.
2. Write the failing Harness test first: in Slots on unit 1's armour
   slot, Next unit shows unit 2's armour slot.
3. Add the actions to `PREPARATIONS` and a `step_unit(forward)` in the
   screen that keeps the focus and rows as above.
4. Update the three help keys; check the help bar fits on every pad
   (`the_longest_help_bar_fits_on_every_pad`, `tests/it/controller.rs`).
5. Doc sentence; snapshots.

## Acceptance criteria

- [x] In Slots, Next unit moves to the next unit with the same slot row
      highlighted; Previous unit back (Harness test with the default
      right-handed keys, `crates/ui/tests/it/`, or a view test in
      `crates/ui/src/screens/preparations/tests.rs`).
- [x] In Stock, Next unit keeps Stock and the row (clamped); with no stock
      rows for the new unit it lands on Slots (test).
- [x] From the last unit Next unit wraps to the first, and Previous from
      the first to the last (test).
- [x] Rebound keys work: with `PrevUnit` bound to another key, that key
      steps the unit and the help bar names it (test).
- [x] The Playdate's B + right steps the unit and doesn't back out of
      Slots (test with `use_playdate_buttons`).
- [x] `cargo shuyi check-keys` passes; no key is named in code or text.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit / view: the focus and rows after each step.
- Integration: the Harness tests above, the controller help-bar test.
- Snapshot: the Loadouts help line.

## Completion notes

- `PrevUnit` / `NextUnit` are in `PREPARATIONS` and handled on the
  Loadouts tab in the Units, Slots and Stock focus by
  `PreparationsScreen::step_unit`: wraps at both ends, move sound, the
  message clears, the focus stays; the slot row is kept (clamped), the
  stock row too (clamped), and an empty stock for the new unit's slot
  lands on Slots. In Units the slot row goes back to the first, as Up and
  Down there already do.
- Help lines `prep.help.units/slots/stock` (and the test pack) read
  `{Cursor} choose · {PrevUnit} {NextUnit} unit · …`; snapshots updated.
- `docs/design/weapons-and-items.md`: one bullet under *On the
  Preparations screen*.
- Tests: three view tests in `preparations/tests.rs` (slot row kept,
  wrapping, stock clamp / fallback), two Harness tests in
  `tests/it/preparations.rs` (default keys with the help lines, a rebound
  Next unit key), and a Playdate test (`B+→` / `B+←` step the unit in
  Slots without backing out; B alone still backs out; the help line fits
  50 columns).
- The battle map's `the_longest_help_bar_fits_on_every_pad` is unchanged:
  the Preparations help lines stay well under it.
- No gameplay rule was decided. For the Act 1 playtest (1064): whether
  stepping the unit feels right in the gear lists.
