---
id: "0442"
title: Combat Arts in their own menu, out of the forecast
type: feature
milestone: M3 Battle UI
model: opus-5.5
effort: high
status: done
blocked_by: []
nick_input: sign-off
completed: 2026-10-12
---

# 0442 — Combat Arts in their own menu, out of the forecast

## Context

Nick, 2026-10-03, while answering ticket 0430 (which keys swap the weapon in
the forecast):

> "like three houses, but up/down should not pick an art. the arts should not
> be in the same menu. we should have an arts menu. that was something i didnt
> like about the current quick battle setup anyway"

Today (ticket 0414, `docs/design/combat-arts.md` → *Forecast display*) the
attack forecast has a list beside it: `Attack`, the weapon's Combat Arts, the
unit's combat actives. Up / down move through it while a target is chosen
([`art_list.rs`](../../crates/ui/src/screens/battle/art_list.rs),
`Targeting` in [`attack.rs`](../../crates/ui/src/screens/battle/attack.rs),
`step_targeting` in [`mode.rs`](../../crates/ui/src/screens/battle/mode.rs)).
A spell's forecast has the same list with the spell actives
([`magic.rs`](../../crates/ui/src/screens/battle/magic.rs), `CastTargeting`).

Ticket 0430 made pointing at an enemy open the forecast at once, with left /
right swapping the weapon or spell (`Targeting::options`, `Targeting::swap`;
`docs/design/turn-structure.md` → *Pointing at an enemy*). It left the arts
list where it was.

In Fire Emblem: Three Houses, `Attack`, `Magic` and `Combat Arts` are three
entries of the action menu; `Combat Arts` lists the arts, then the target is
picked with the forecast.

## Nick input

**Ask first, with the `ask-nick` skill, before building** (rendered mockups,
`feedback-nick-visual-reviews`), then record the answers in
`docs/design/combat-arts.md` (*Forecast display*) and
`docs/design/turn-structure.md`:

1. Where the arts menu is: an `Arts` entry in the action menu (Three Houses),
   or something else.
2. How an art is reached after **pointing at an enemy** (0430 opens the
   forecast directly): a key in the forecast that opens the arts menu, or only
   through Cancel → action menu → `Arts`.
3. Where the combat **actives** go (Close Shot, Overcast…): in the same arts
   menu, or their own.
4. What up / down do in the forecast once the list is gone (change target?).
5. Whether left / right swap the weapon in **every** forecast, also the one
   opened from `Attack` (which would make the weapon list before it
   unnecessary), and the same for `Magic` and its spell list.

**Sign-off:** in Quick Battle, use an art with the lord and Overcast with the
mage, by pointing at an enemy and from the action menu.

## Scope

**In:**
- The arts menu Nick chooses, for weapons' arts and for the combat actives.
- The forecast without the arts list; its keys as Nick answers.
- Help bars through the keymap (`keyboard-input` skill).
- The design docs above.

**Out (do not do):**
- New arts, costs or rules of arts (`core` is unchanged; every check stays
  `BattleState::preview_attack`, ADR-0004).
- Describing arts when choosing (ticket 0429): if 0429 is done first, keep its
  descriptions in the new menu; if this one is done first, add a line to 0429
  that the list it describes is the arts menu.
- AI changes.

## Implementation steps

1. Ask Nick (above) and record the answers.
2. `mode.rs`: a `MenuEntry` and `Mode` for the arts menu as answered, built
   from `art_choices` (`art_list.rs`); choosing a line opens `Targeting` with
   that technique fixed.
3. `attack.rs`, `Targeting`: drop the list (`choices`, `list`, `move_list`,
   `has_list`) or keep only the chosen technique; the same for
   `CastTargeting` in `magic.rs`.
4. `mod.rs`: draw the arts menu; remove the list beside the forecast;
   `help_targeting` without `art`.
5. Update the tests of 0414 (`art_tests.rs`), 0410 (`magic_tests.rs`), 0428 and
   0430 (`mode.rs`, `attack_tests.rs`) to the new flow.

## Acceptance criteria

- [x] Nick's answers are in `docs/design/combat-arts.md` and
      `docs/design/turn-structure.md`, in his words.
- [x] Test: an art is chosen in the arts menu, then its target, and the attack
      command carries that art.
- [x] Test: the forecast shows no arts list, and up / down do what Nick chose.
- [x] Test: a dimmed art (`broken`, `2 uses left`) can't be chosen and shows
      its reason in the menu.
- [x] Test: Cancel from the forecast of an art goes back to the arts menu,
      then the action menu.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the menu's lines and the mode transitions.
- Snapshot / integration: a harness script with the default keys using an art
  and a spell active; review the changed forecast snapshots.

## Completion notes

**Nick's answers** (asked before building, with rendered mockups,
2026-10-12; recorded in `docs/design/combat-arts.md` → *The arts menu and the
forecast*, `turn-structure.md` → *Pointing at an enemy* and `magic.md`):

- How do you get to the arts menu? "A. Action menu only" (like Three Houses;
  after pointing at an enemy: Cancel, then `Arts`).
- Where do the class actives go? "C. Spell actives in spell list" (weapon
  actives in `Arts`; the spell list gets `Fire + Overcast` lines; no `Arts`
  for mages).
- What do the keys do in the forecast? "A. Same everywhere" (left/right swap
  the weapon in every weapon forecast, up/down change the target, no weapon
  list after `Attack`).

**Done.**

- `Arts` is an entry of the action menu under `Attack` (`MenuEntry::Arts`,
  `Mode::ArtsMenu`). Its lines (`art_lines` in `art_list.rs`) are the arts
  of every weapon the unit carries, then its combat actives that swing a
  weapon. A line nobody can be attacked with is dimmed with the reason
  (`broken`, `out of range`). Choosing a line opens the forecast with that
  art fixed (`Targeting::art`); Cancel goes back to the arts menu, then to
  the action menu on `Arts`.
- The forecast has no list (`Targeting` lost `choices`, `list`, `move_list`,
  `has_list`; it has one `technique`). Up / down and the unit keys change
  the target; left / right swap the weapon whenever several can make the
  attack.
- `Attack` opens the forecast at once with the equipped weapon
  (`Targeting::attack`). The weapon list (`Mode::WeaponMenu`) is gone.
- Spell actives are lines of the spell list (`SpellChoice::active`):
  `Fire`, then `Fire + Overcast  +1 use`, dimmed with the reason when the
  spell can't pay (`1 uses left`).
- Help bars come from the keymap as before; the `art` hint is gone.
- No `core` change (one doc comment's pointer). Every check is still
  `BattleState::preview_attack`.
- Ticket 0429 has a note that the list it describes is now the arts menu
  and the spell list's lines.

**Deviations from the steps as written:**

- Step 2 said the arts menu is built from `art_choices`, which worked per
  target. The menu is opened before a target is known, so it is built by
  the new `art_lines` (per weapon, over every unit) and `art_choices` is
  gone.
- The weapon list was removed and spell actives moved to the spell list:
  both from Nick's answers, not in the steps.
- The spell-active line shows only `Fire + Overcast  +1 use`, without the
  spell's uses and numbers (they are on the `Fire` line above): with them
  the list was too wide to fit beside the unit.

**Rules I decided** (*Claude's starting rules*, Nick can veto):

- `Arts` is listed when the unit has an art or a weapon active for a weapon
  it carries, and dimmed when none can be used on anyone from that tile.
- `Attack` and pointing at an enemy are plain attacks only. An enemy only
  an art reaches is attacked through `Arts`. Example: an archer with a
  brigand right beside it: `Attack` is dimmed, the menu opens on `Arts`,
  Close Shot hits it. (Before, pointing at that brigand opened the forecast
  with Close Shot already chosen.)
- An art's forecast opens with the equipped weapon if it can use the art on
  someone, else the first weapon that can. Left / right swap only between
  weapons that can; the art stays chosen.
- From `Attack`, left / right swap between weapons only (spells are under
  `Magic`). After pointing they swap between weapons and spells, as before.
- `Magic` keeps its spell list, and a spell's forecast opened from it
  doesn't swap.
- A spell cast with an active targets enemies only (no tiles).
- A line with nobody in reach is dimmed with `out of range`.

**For Nick when playing** (sign-off, Quick Battle):

- The lord: point at the near brigand, Confirm. The forecast opens with no
  list; left / right swap Iron and Steel Sword. Cancel, then `Arts`, Guard
  Break, Confirm twice.
- The mage: `Magic`, then `Fire + Overcast`.
- On the small test map the arts menu and the spell list open to the left
  of the unit, as the old weapon list did, because they don't fit on its
  right.

**Follow-up tickets:** none.
