---
id: "0088"
title: "Decide: dungeon delves (dark floors, the Way Down and the Way Out)"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0766"]
nick_input: decision
completed:
---

# 0088 — Decide: dungeon delves (dark floors, the Way Down and the Way Out)

## Context

Nick, during the 2026-11-16 hook research:

> "or if our open world had some dungeon delves with creepy music and
> many floors... maybe mystery dungeon like (though with SRPG grid
> rather than party based, and with escape tile rather than moving each
> turn to a staircase if that makes sense)"

> "probably a subset like 3-6 depending on dungeon"

> "by side mode its more like a node on the world map not a literal
> option on main menu"

On the "Torchlight" version, he said "this sounds fine":
- Floors are dark, and light behaves like an element: fire lights the
  area, water puts it out.
- The Way Down starts collapsing once someone steps on it. Anyone left
  behind starts the next floor alone, so it's a rescue, not a death.
- Loot comes home only if you leave by a Way Out.
- Linger too long on a floor and something wakes.

Delves are where the legendary beasts hide (0087). The beast waits on the
last floor of its dungeon. They also stand alone as world-map content.

**What came out of the research** (honest freshness, to show Nick):
- Final Fantasy Tactics' Deep Dungeon (1997) already has dark tactics
  floors with a hidden exit tile any unit can find.
- Disgaea's Item World has random floors, with exits only every 10 floors
  or by an item.
- Fire Emblem's Tower of Valni and Tactics Ogre's 100-floor Palace of
  the Dead are towers players call a grind when nothing is at stake.
- What's ours: light as an element, the collapsing Way Down with a
  rescue, the beast at the bottom, and the rival column.

**What exists:**
- Escape maps, where units that leave are safe (`death-and-difficulty.md`,
  0069).
- The world map's nodes, towns and skirmishes (`world-structure.md`,
  1007, 1008).
- The skirmish generator (0510, done) builds skirmishes while authoring,
  not while the game runs. A generator that runs in the game, from a
  saved seed, is new work for 1277 (an ADR). 0522, still open, adds
  layout templates to the authoring tool.
- A `dungeon_tense` music track (`audio.md`).
- Rewind charges and permadeath (`death-and-difficulty.md`).
- The run mode (0050) is undecided. A delve and a run can share floor
  generation, so the two answers should be kept apart and consistent.
- 0060 (free movement) decides whether the army walks around towns and
  dungeons outside battle. Nick, 2026-11-16: secrets belong to
  overworld free roam, not to battles. Delves stay grid floors whatever
  0060 decides.

This waits for the writer audition (0766), as Nick asked.

## Nick input

**Decision** (`ask-nick`): at most three questions per message, as
player situations with numbers, with options from real games plus
"describe your own". The answers go into a new `docs/design/delves.md`.

1. **The party:** the player picks 3–6 for each dungeon (the dungeon
   sets the size), or always 4. Are units that aren't picked unavailable
   until the party leaves?
2. **Floors:** how many per dungeon (for example 5–10), and generated
   each visit or fixed once generated, like Mystery Dungeon or Valni.
   How big is a floor?
3. **Dark and light:** how far each unit sees (for example 2 tiles), how
   long fire's light lasts, whether water puts light out, and whether
   ice keeps a faint glow. How are enemies outside the light shown: not
   at all, or as marks, such as footprints that fade?
4. **The Way Down:**
   - A. It starts collapsing once any unit steps on it, giving 3 turns
     to get everyone through. Anyone left behind starts the next floor
     alone, somewhere in the dark (the "Torchlight" rule).
   - B. The whole party must stand on it, like Seize.
   - C. Describe your own.
5. **The Way Out and loot:** a Way Out every third floor? Loot carried
   until you leave by one, with a few marked items kept anyway, like
   Lufia's blue chests? What's lost if the Lord falls?
6. **Lingering:**
   - A. After a set number of turns, something wakes on the floor (a
     powerful enemy), like Mystery Dungeon's wind or Persona's Reaper.
   - B. The floor closes and pushes you out.
   - C. Describe your own.
7. **Rewind and permadeath in a delve:** the same as any battle (per-floor
   charges), one pool for the whole delve, or another rule. Retry
   restarts the floor, not the delve?
8. **Coming back:** can a dungeon be re-entered, and are its floors new
   each time?
9. **The delve's mark on the world map:** the random skirmish's "?" on a
   grey plate is taken (`look-and-feel.md`, *The world map*). Ask with a
   mockup in the picture look.

## Scope

**In:**
- Running the questions and recording the answers, with every number
  *tunable*, in `docs/design/delves.md`, and the `docs/design/README.md`
  row.
- Bringing 1277–1279 and 1281's text in line with the answers. If an answer makes
  0050 (run mode) share a piece, add a line to 0050.

**Out (do not do):**
- Building anything (1277, 1278).
- The beasts themselves (0087).
- Free movement (0060).

## Implementation steps

1. Read `world-structure.md`, `death-and-difficulty.md`, `turn-structure.md`
   (escape objectives), `audio.md`, `replayability.md`, and tickets
   0050, 0060 and 0522.
2. Run the questions with `ask-nick`, using mockups in the picture look
   (a dark floor with lit circles and a collapsing Way Down).
3. Write `delves.md`, the README row and the follow-up changes, and move
   this ticket to `done/` in the same PR.

## Acceptance criteria

- [ ] `docs/design/delves.md` exists with Nick's words verbatim and the
      derived rules.
- [ ] 1277–1279 and 1281 match the answers.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None (a design decision).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
