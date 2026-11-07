---
id: "0482"
title: "An area skill's reach is shown on the map while it is chosen"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0482 — An area skill's reach is shown on the map while it is chosen

## Context

A friend's playtest of the Act 1 build, passed on by Nick (2026-11-07):

> "Inspire skill (and probably all skills similar) should have a range
> indicator rather than needing to work it out manually"

Inspire (`assets/data/skills.ron` ~:130): `Active(cost: Uses(2), effect:
Buff(area: Allies(radius: 2), mods: (combat: (hit: 10, avoid: 10))))`;
`docs/design/progression.md` ~:676: "allies within 2 tiles get hit +10
and avoid +10 until the start of this unit's next phase". Other actives
with an area of allies: the Str +2 buff (~:186, radius 1), Rally (~:264,
radius 2), the area heals (~:122 radius 1, ~:243 radius 2). The radius is
Manhattan distance and leaves out the user (`crates/core/src/battle/
skills.rs` ~:406–412, `allies_near`; `progression.md` ~:682).

Today these skills are used straight from the Skill menu, with no target
step and no map preview: `skill_choices`
(`crates/ui/src/screens/battle/skills.rs` ~:322) sets `needs_target` only
for Push (Shove). The menu describes the area only in text (`"ally
r{radius}"`, `assets/lang/en/ui.ron` ~:677). `add_ranges`
(`crates/ui/src/screens/battle/mod.rs` ~:2290–2343) tints move, attack,
danger, exit and the targets of the targeting modes, but has no area
case; the range kinds are in `crates/ui/src/map_view/scene.rs` ~:28–45
(DANGER, MOVE, ATTACK, HEAL, EXIT).

## Nick input

None: he asked for the indicator. The look reuses the map's existing
ally-range tint (HEAL), so no new look is invented; *the agent's
starting rule*, written in `look-and-feel.md` and put on the Act 1
playtest's checklist (1064). Verified by scene tests and `frame-png`
pictures in the PR.

## Scope

**In:**
- While an active skill whose effect has an area (`Allies(radius)` and
  any other area kind the skill data has) is **highlighted** in the
  Skill menu, the map scene shows the tiles it reaches (Manhattan
  `radius` around the user, leaving out the user's own tile), using the
  HEAL range kind; the allies who would get the effect are the ones
  standing in it.
- Moving the menu cursor off it clears the area; choosing it uses the
  skill as today.
- The same for an area spell or item, if the data has one that is used
  from a menu without a target step (check `spells.ron` and
  `items.ron`); if none, say so in the Completion notes.
- In the map scene (ADR-0038), so every map skin paints it: glyph,
  picture and Playdate.
- One line in `look-and-feel.md` (ranges) and in `progression.md` next to
  the auras rule.

**Out (do not do):**
- Passive auras (Leadership, `AllyAura`), which are never chosen from a
  menu; showing them is a separate question for Nick (ask in the Act 1
  playtest, 1064).
- A target step for these skills, or changing what they do.
- Area arts that strike several enemies (0327): they have their own
  target step.
- Outlines instead of tints (1213, on the kit's `RangeOutline`): if 1213
  is done, the area uses whatever ranges use then.

## Implementation steps

1. Read `skill_choices`, `add_ranges`, `allies_near` and the range kinds.
2. Give the Skill menu's highlighted choice its area (a radius or a set
   of tiles) in the battle's state for the view; one helper in core
   gives the tiles of an `Allies(radius)` area for a unit, so the
   preview and the effect never disagree (`allies_near` uses it).
3. In `add_ranges`, a case for the Skill menu: tint those tiles HEAL.
4. Tests: the scene's HEAL tiles for Inspire at radius 2 in the open, at
   a map edge, and with the user on its own tile left out; cleared when
   the cursor moves to a skill with no area.

## Acceptance criteria

- [ ] With Inspire highlighted in the Skill menu, the scene's HEAL range
      is exactly the tiles within Manhattan distance 2 of the user, not
      the user's tile (test reading the scene).
- [ ] Highlighting a skill with no area shows no HEAL range (test).
- [ ] The tiles come from the same core helper the effect uses (test:
      every ally in the shown area gets the buff after Confirm).
- [ ] `frame-png` pictures of Inspire highlighted, glyph and picture
      looks, in the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the core area helper.
- Integration: Harness tests on the debug Quick Battle with a unit that
  has Inspire (add one to the test characters if none has it).
- Snapshot: none new beyond the battle snapshots that change.

## Completion notes

*(Filled in by the session that completes the ticket.)*
