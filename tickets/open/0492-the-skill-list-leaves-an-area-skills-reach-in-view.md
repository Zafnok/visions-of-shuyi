---
id: "0492"
title: "The Skill list leaves an area skill's reach in view"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0482"]
nick_input: decision
completed:
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

- [ ] With Inspire focused in the Skill menu on the Quick Battle, no tile
      of the reach is under the list window (test reading where the
      window is painted against the scene's heal range).
- [ ] `frame-png` pictures of Nick's pick, both looks, in the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the placement function for a reach beside a map edge.
- Snapshot / integration: the battle snapshots that change.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
