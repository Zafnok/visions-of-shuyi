---
id: "0060"
title: "Decide: does the army ever walk around freely (towns, the world, dungeons)?"
type: design-decision
milestone: Design decisions
model: reasoning
effort: medium
status: todo
blocked_by: ["1064"]
nick_input: decision
completed:
---

# 0060 — Decide: does the army ever walk around freely?

## Context

Nick, 2026-10-14, asking for the engine split (ticket 1101,
`docs/engine/feasibility.md`): the engine should support a top-down
**free-movement** mode as well as the grid, "and transitions between the
game types as dev wants (i.e. we ourselves might transition between 1 and
2 for open world section)".

What is decided so far points the other way, so this needs Nick's answer
before anything is built for the game:

- [`world-structure.md`](../../docs/design/world-structure.md) (0008):
  "This is an SRPG/TRPG not a traditional open world RPG so we need to
  keep the pace in mind." After Chapter 3 the army moves between **nodes**
  on a world map (FE Sacred Stones). On Q3 Nick said recruitable
  wanderers "might be inside towns?", which is the one place walking
  around was hinted at.
- 1007 builds the world map as nodes and menus, and 1003 builds camp as
  screens.

Why it matters beyond the game: the engine's free-movement mode
(engine ADR-0003; engine tickets 1119–1125, parked in [Shuyi
Engine](https://github.com/Zafnok/shuyi-engine/blob/main/docs/ROADMAP.md)) is paid for by the game only if
the game uses it. If the answer is "never", that work stays parked until
the engine reaches parity with the game (plan B).

## Nick input

**Decision**, with the `ask-nick` skill. Ask after the Act 1 playtest
(1064). It said "after the Chapter 1 playtest" (0804), because round 3's
feel work was his top priority (2026-10-14); 0804 was folded into 1064
on 2026-10-21.

## Scope

**In:** the questions below; the answer recorded in `world-structure.md`
(a *Walking around* part) and its `docs/design/README.md` row; changes to
1007 and 1003 that the answer calls for (for example 1007 gains a town
scene that is walked through, or nothing changes), and unparking engine
tickets 1119–1125 if the answer is B–D.

**Out (do not do):** building any of it; changing the world map's node
design (that is 0008's answer and stays unless Nick changes it here).

## Implementation steps

1. Read `world-structure.md`, `supports.md` (camp), `chapter-1.md` and
   `docs/engine/feasibility.md` (*Game styles*).
2. Ask, as player situations with an example (feedback: plain starting
   rules), at most three questions per message:
   1. **Is there anywhere the player walks a character around freely?**
      - **A. Nowhere: menus and nodes only** — FE Sacred Stones, Tactics
        Ogre Reborn, Wargroove. Between battles everything is a menu or a
        node on the map. Keeps the pace tight, which 0008 asked for.
      - **B. Towns and camp only** — FE Three Houses' monastery, Triangle
        Strategy's exploration before a battle: walk the lead around a
        small place, talk to people, find items, then back to nodes and
        battles.
      - **C. Towns, camp and some dungeons or fields** — FE Echoes'
        dungeons: walk through a place, touching an enemy starts a grid
        battle on that ground.
      - **D. An open world map walked freely** — Unicorn Overlord: the army
        walks the overworld; battles start where it meets enemies.
      - **E. Describe your own.**
      Recommendation: **A or B**: A keeps 0008's pace; B is the smallest
      step that uses the engine's free-movement mode, in places where pace
      matters least.
   2. Only if B–D: **what does walking look like?** The lead alone, or the
      army following in a line (Three Houses: the lead alone; Octopath: the
      lead alone; Unicorn Overlord: one marker for the army). Glyphs or the
      bought map sprites (the sprites already walk, 0440).
   3. Only if B–D: **which act first?** Never in Chapters 1–3 (they are
      linear, 0008), so the earliest is the first world map (after
      Chapter 3).
3. Record Nick's words verbatim and the rules in `world-structure.md`.
4. Update 1007 and 1003 (a walked-through town or camp, or no change),
   and the *Game styles* table in `docs/engine/feasibility.md` (whether
   the game uses free movement). If the answer is B–D:
   - unpark engine tickets 1119–1125 in Shuyi Engine, with a small PR
     there that follows its `tickets/README.md`, *Parked tickets*;
   - write the game ticket for the first walked-through place, waiting
     for 1131 and noting the engine tickets it needs.

## Acceptance criteria

- [ ] Nick's answer, verbatim, is in `world-structure.md`.
- [ ] 1007, 1003, `docs/engine/feasibility.md` and (if B–D) the engine
      tickets 1119–1125 match the answer; `cargo xtask ticket-lint`
      passes.

## Tests required

None (docs only).

## Completion notes

*(Filled in by the session that completes the ticket.)*
