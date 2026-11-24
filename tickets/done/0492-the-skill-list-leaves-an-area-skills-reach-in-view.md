---
id: "0492"
title: "The Skill list leaves an area skill's reach in view"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0482"]
blocks: ["1064"]
nick_input: decision
completed: 2026-11-17
---

# 0492 — The Skill list leaves an area skill's reach in view

## Context

Ticket 0482 tints an area skill's reach (Inspire, Rally, Sanctuary,
Benediction) with the heal range while its line is focused in the Skill
menu. Its `frame-png` pictures (`docs/img/0482-inspire-glyphs.png`,
`docs/img/0482-inspire-pictures.png`) show the Skill list window, placed
one cell right of the unit by the menu placement rule (`menu_origin_in`,
`crates/ui/src/screens/battle/glyph.rs` ~:184, ticket 0454), covering
about half of the radius-2 diamond: the player sees only the left half of
the reach, which is what the friend's playtest asked to see (0482's
Context).

Where the list goes is a look decision (`docs/design/look-and-feel.md`),
so it is Nick's.

## Nick input

**Decision**, asked before building with the `ask-nick` skill, with
`frame-png` pictures of each option over the Quick Battle in both looks
(glyphs and pictures). Options to show, from real games:
- A: the list goes beside the reach, not beside the unit (outside the
  diamond, left or right, whichever fits);
- B: the list stays, the camera pans so the reach is clear of it;
- C: the list goes to a fixed corner of the map view while an area skill
  is focused (Fire Emblem: Three Houses keeps its command list to one
  side);
- D: describe your own.

## Scope

**In:**
- The placement Nick picks, for the Skill list while a line with an
  area is focused, on the PC glyph and picture skins and the Playdate
  (`menu_origin_in` with `SCREEN`).
- One line in `look-and-feel.md` next to 0482's line.

**Out (do not do):**
- The placement of other lists and menus (Arts, Magic, Item, the action
  menu): unchanged unless Nick says otherwise.
- What the reach shows or how it is tinted (0482, 1213).

## Implementation steps

1. Render the options with `cargo shuyi frame-png` (picture look:
   `cargo run -q -p xtask --features private-assets -- frame-png …`
   after `cargo shuyi private-assets`) and ask Nick (`ask-nick`).
2. Implement the pick where the battle skins place a list
   (`paint_menu_beside`, `glyph.rs`; the Playdate's in `playdate.rs`),
   taking the focused choice's `SkillChoice::area` from the view.
3. Tests and the doc line.

## Acceptance criteria

- [x] With Inspire focused in the Skill menu on the Quick Battle, no tile
      of the reach is under the list window (test reading where the
      window is painted against the scene's heal range).
- [x] `frame-png` pictures of Nick's pick, both looks, in the PR.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the placement function for a reach beside a map edge.
- Snapshot / integration: the battle snapshots that change.

## Completion notes

**Nick's pick:** A, beside the reach (2026-11-17). He was shown, over the
Quick Battle in both looks, the list as today, A beside the reach, B kept
in place with the map slid left, and C in the map view's top-right
corner.

**Done.**
- View: `BattleMenuView::clear_of`, the tiles a menu keeps clear of
  besides its own: the focused area skill's reach in the Skill list,
  empty for every other menu and line. One helper,
  `BattleScreen::reach_shown`, gives it to the view and to `add_ranges`'s
  tint, so the list and the tint never disagree.
- Skins: `beside_all` (the smallest rectangle holding the unit's tile and
  the reach's cells) and `menu_anchor` (`glyph.rs`); the PC skins (glyph
  and picture, which share the menu painting) and the Playdate place the
  list with 0454's `menu_origin_in` beside that rectangle instead of the
  unit's tile: right of the reach, or left where it would leave the map
  view, its first item level with the reach's top row. Reach tiles the
  camera doesn't show are left out of the rectangle.
- Doc: one line in `look-and-feel.md` under 0482's.
- Tests: `a_menu_goes_beside_the_reach_it_keeps_clear_of` (the placement
  in the open and at the view's right edge, and no reach = beside the
  tile); `the_skill_list_keeps_clear_of_the_focused_reach` (reads the
  window's border in the frame against the scene's heal range on the
  Quick Battle); `a_skill_with_no_area_leaves_the_list_beside_the_unit`.
  No snapshot changed.
- Pictures: `docs/img/0492-inspire-glyphs.png`,
  `docs/img/0492-inspire-pictures.png` (a game screenshot, as 0482's).

**For the Act 1 playtest (1064):** the list hops two tiles sideways when
the focus moves onto an area skill and back (Nick saw this downside in
the question and picked A).

**Engine friction:** estimate 30 production lines (a field on the menu
view and a bounding rectangle fed to 0454's placement rule in two skins),
measured 35 added (1.2×): under 2×.
**PR size:** estimate 250, measured about 210 (`cargo xtask pr-size`).
