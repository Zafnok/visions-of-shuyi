---
id: "1304"
title: "Vix's Chapter 1 lessons in a better order, ticking what the player does"
type: feature
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
nick_input: none
completed: 2026-11-19
---

# 1304 — Vix's Chapter 1 lessons in a better order, ticking what the player does

## Context

Nick played the Chapter 1 web demo on 2026-11-20 as a writer applicant
would (0766's `before-sending.md`) and gave this feedback on Vix's tutorial
(his words, kept as he wrote them):

1. "I don't know why Vix's tutorials were collapsed they were better before
   at least the earlier ones like when it asks to press one key in each
   direction and tracks which have and have not been completed (or I guess
   this is just a complaint for the very first Vix tip bc she goes into it
   right after I think its wholly redundant)"
2. "Vix's attack tutorial doesn't track backing out as credit same as
   attack, only attack counts as credit"
3. "Vix's zoom tutorial should force a zoom out and a zoom in"
4. "Vix when describing acronyms like DEF and AVO and EXP should say the
   whole word not the shorted"
5. "Vix's moving tutorial should come before the attacking tutorial and
   track movement as a checkmark"
6. "enemy in reach tutorial should play probably at same time as movement
   tutorial or before attacking or before moving"
7. "swapping weapon vix tutorial should track progress"
8. "Vix's end turn tutorial should play earlier than exhausting all units
   since the player might wanna be tactical and not use all units every
   turn. Or it shouldn't play at all past turn 1 basically bc you know
   player already knows the info."

These are Nick's decisions; this ticket builds them. Where one is
ambiguous, the reading below is the agent's, recorded as a starting rule.

The lessons and tips are data: `assets-private/content/data/lessons.ron`
(cursor, keys, next_unit, moving, attacking, ending_turn, rewind,
threat_arrows, combat_arts, terrain_magic) and `tips.ron` (battle_start,
zoom, unit_selected, enemy_in_range, forecast, low_hp, level_up, terrain,
wading, weapon_kinds, weapon_swap, ...), with the rules in
`docs/design/tutorial.md` and the screen in
`crates/ui/src/screens/battle/` (`lesson_moment` and the tip moments). A
lesson is a checklist (items with `done_when`); a tip is one box with
nothing to tick (tutorial.md, *A tip or a lesson*).

This comes before the writer trial's first packets go out (0766). Vix's
tips and lessons in plain words are allowed during the dialogue writing
pause (0766, *The writing pause*): a changed line needs its voice clip made
again.

## Nick input

None: the decisions are above. What's left to judge by playing goes on the
Act 1 playtest's checklist (1064).

## Scope

**In** (each numbered to Nick's points):
1. The very first Vix box before the cursor lesson repeats what the cursor
   lesson then asks: drop that box (or fold its one useful sentence into
   the cursor lesson's text) so the turn starts with the checklist that
   ticks each direction. Find which box it is (`battle_start` tip or the
   lesson's own opening) by playing turn 1 at Full in the Harness.
2. **Attacking**: backing out of the forecast also ticks. Make it two
   items, "Attack: {Confirm}" (`Attacked`) and "Back out: {Cancel}" (a
   `done_when` for leaving the forecast without attacking, new if none
   exists), so either order works; the lesson closes when both are ticked.
3. **Zoom**: becomes a lesson with two items, zoom in and zoom out (each a
   `Pressed` of its action), shown when the zoom tip shows today.
4. Vix's text says **Defense**, **Avoid**, **Experience** (and any other
   short stat name in her lines: HP stays HP? check the game's own stat
   names in `assets/lang/en`) in full in what she *says*; the screen's
   column labels stay short. Change `tips.ron` and `lessons.ron` text, then
   make the changed lines' voice clips again (`cargo xtask voice`; if the
   voice service needs Nick's key, say so in the PR and list the lines).
5. **Moving** (the "Selecting a unit" lesson) comes before **Attacking**,
   and moving the unit is its own ticked item (it is: `UnitMoved`; check it
   ticks on screen and that the order holds when a unit could attack on
   turn 1).
6. The **enemy in reach** tip shows with or before the moving lesson, not
   after the attack.
7. **Swapping weapons** becomes a lesson that ticks: open a forecast with
   two weapons, swap with left/right (items for each, or one item "swap"
   that ticks on the first swap).
8. **Ending the turn**: shows on turn 1 right after the first unit has
   acted (not only when every unit has acted), and never after turn 1.
   Ending the turn early (with units left) still ticks it.

**Out:**
- New lessons or tips beyond these; Chapter 2+ tutorial.
- The tutorial levels (Full/Short/None) and Guides menu.

## Implementation steps

1. Expected size: data edits in two RON files, a few new `done_when`
   conditions and moment rules in the battle screen, about 150-250
   production lines, plus tests and clip regeneration.
2. Play turn 1 at Full in the Harness and list every Vix box in order,
   before changing anything (paste the order in the PR).
3. Make the changes 1-8, updating `docs/design/tutorial.md` where its rules
   change (record each as from Nick, 2026-11-20).
4. Tests: a Harness test of turn 1 at Full that checks the order (cursor
   lesson first with no box before it, moving before attacking, enemy in
   reach no later than moving, ending the turn after the first unit acts),
   the attack lesson ticking on back out, the zoom lesson ticking both
   ways, the swap lesson ticking, and no ending-turn lesson on turn 2.
5. Voice clips for every changed line (`cargo xtask voice status` must
   show none missing).
6. Run the `run-gates` skill; after merge, Nick re-runs the Demo workflow.

## Acceptance criteria

- [x] The turn-1 order test passes as above.
- [x] Attacking ticks on back out; zoom ticks in and out; swapping ticks.
- [x] Ending the turn shows after the first unit acts on turn 1 and never
      on turn 2 or later.
- [x] Vix's spoken lines say Defense, Avoid and Experience in full; every
      changed line has its clip.
- [x] `docs/design/tutorial.md` says the new rules.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Harness integration tests of the tutorial order and ticks (ADR-0007).

## Completion notes

**Turn 1 at Full before** (read from the code and the Chapter 1 tests):
the *Your move* tip; the cursor lesson; the keys; finding your units;
*Selecting a unit*; Attacking (once a unit that reaches an enemy is
selected); Ending the turn (only once every unit had acted). The *Moving
a unit*, *Enemy in reach* and *Swapping weapons* tips fired while a
lesson was up, missed their moment and came up only later, after
Attacking: why Nick saw "moving" and "enemy in reach" after the attack.
The zoom tip came back on the map after the lessons.

**After:** the cursor lesson (no box before it); the keys; finding your
units; *Moving a unit* (select, send to a blue tile, the menu; its line
says red is what the unit can hit, an enemy on red in reach); Attacking
(the forecast, then attack **or** back out); Swapping weapons (the first
forecast that swaps, ticked by a swap); Zooming the map (pictures only:
zoom out, zoom in); Ending the turn (turn 1, back on the map once the
first unit has acted; never after turn 1).

What was built, by Nick's point:

1. The `battle_start` tip is gone.
2. New `ForecastClosed` item (attacked from or backed out of); the
   Attacking lesson's second item uses it.
3. The zoom tip became the `zoom` lesson, items `ZoomedOut` and
   `ZoomedIn`. With glyphs it never opens, is put away if the look changes
   under it, and the box at the end of the lessons doesn't wait for it.
4. Vix's lines: Experience (level up), Defense and Avoid (terrain),
   Movement, Avoid and Defense (wading). Screen labels stay short.
5. and 6. *Moving a unit* lesson (was *Selecting a unit*) carries the
   removed *Moving a unit* and *Enemy in reach* tips' teaching.
7. The swap tip became the `weapon_swap` lesson, item `WeaponSwapped`; it
   opens only if a unit left could attack with two weapons or spells.
8. `ending_turn` starts on the new lesson trigger `FirstUnitActed` (turn
   1 only) and is put away after turn 1.

Voice: 7 clips made with the voice lab (`cargo xtask voice generate
--scene vix`), 10 stale ones removed; `voice status` shows none missing,
`voice audit` flags none.

Tests: the tips' mechanism tests used the removed tips as examples; they
now get them from a test fixture (`crates/ui/src/harness_tips.ron`, added
by the Harness and the unit tests' context, never in the game;
`with_shipped_story` keeps the game's own tips). The test language pack
translates the forecast tip instead of the start tip. The kept Quick
Battle session record was blessed (no start tip now).

Deviation: point 2 is built as one item that either ticks (Nick: "credit
same as attack"), not the draft's two items both needed.

*The agent's starting rules* (in `docs/design/tutorial.md`, for the Act 1
playtest, 1064):

- Backing out of the forecast and attacking tick the same item.
- Ending the turn steps aside for any lesson that comes up while it is
  open and can be done (Attacking, Swapping weapons, Zooming), and comes
  back after it, so it never holds the rest of turn 1's lessons.
- Zooming opens the first time the player is back on the map with no
  lesson but Ending the turn in play; only with pictures.
- Swapping weapons opens only when a unit still to act could swap.
- Vix's new lines (Moving a unit, Swapping weapons, Zooming, Ending the
  turn) are the agent's plain-words tutorial lines, allowed in the
  writing pause.

Left to judge by playing (1064): whether the zoom lesson lands at a good
moment on turn 1, and the new lines' feel.
