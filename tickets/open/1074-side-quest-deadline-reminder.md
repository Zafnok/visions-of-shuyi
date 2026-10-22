---
id: "1074"
title: "Show when a discovered side quest closes (a deadline reminder on the world map)"
type: feature
milestone: Post–Chapter 1
model: sonnet-5
effort: medium
status: todo
blocked_by: ["1007"]
nick_input: none
completed:
---

# 1074 — Show when a discovered side quest closes

## Context

Ticket 0069 (2026-10-22) decided that a side quest, and its unique reward,
can be **missed for good**: it needs its character recruited and alive, and
an act's world map closes when the act ends (`docs/design/world-structure.md`,
*Side quest rewards* and *Continents and acts*). Nick: "missed for good, with
some reminder / tracker if the side quest is discovered "deadline in XYZ
time"".

So once a side quest has been **discovered** (unlocked and shown on the
world map), the player is told when it closes, e.g. `Closes when Act 1
ends`. In Act 1 every side quest stays open until the act ends (the outline,
Chapter 6: "Act 1's side quests stay open"), but a later quest may close
earlier (after a given story battle), so the deadline is data on the node.

1007 builds the world map's nodes, unlock conditions and map menu; this
ticket adds the deadline to side-quest nodes and shows it.

## Nick input

None. Verified by tests and `cargo shuyi frame-png` pictures in the PR.
How the reminder looks and reads in play goes on the Act 1 playtest's
checklist (1064).

## Scope

**In:**
- A side-quest node's **deadline** in the world-map data (`content`):
  `ActEnd` (the default) or `AfterStoryBattle(<node id>)`, validated (the
  node exists, is a story battle, and comes after the quest unlocks).
- `core` closes the quest when its deadline passes (the node becomes
  cleared/unplayable, an event says so) and reports, for each discovered,
  unplayed quest, its deadline (a plain-data query, ADR-0004).
- The reminder: on the world map, a discovered side quest's info line shows
  `Closes when Act 1 ends` / `Closes after <story battle name>`; the map
  menu gets a short list of open side quests with their deadlines. Built as
  a view (plain data) painted by the skin (ADR-0054); the node's mark goes
  in the world-map scene (ADR-0038). Text through the language packs.
- A quest whose character dies (Classic) closes at once, with the same event.

**Out (do not do):**
- The Act 1 side quests themselves (1072) and their node placement (1070).
- Any new rule for missing a recruit (optional recruits like Gil have no
  reminder; Nick only asked for side quests).
- Changing when side quests unlock.

## Implementation steps

1. Read `world-structure.md` (*Side quest rewards*), 1007's node data and
   world-map state, and its screen's view/skin split.
2. Add the deadline to the side-quest node data, its validation and
   `assets/` README docs; default `ActEnd`.
3. In `core`: close quests when their deadline passes or their character
   dies, through `Command` → `Event`; a query listing discovered open
   quests with deadlines.
4. In `ui`: the info line and the map-menu list (view + skin), text keys in
   the language packs (`cargo shuyi check-text`).
5. Expected size: about 150 production lines (one data field and its check,
   one rule with an event, one list view and its skin), judged by 1007's
   town-node info line.

## Acceptance criteria

- [ ] A discovered side quest shows its deadline on the world map and in
      the map-menu list (snapshot).
- [ ] A quest closes after its `AfterStoryBattle` node is won, and at once
      when its character dies in Classic (unit tests, named after the rule).
- [ ] A world map with a deadline naming an unknown or earlier node is
      refused by content validation.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: deadline passing, character death, the query; validation errors.
- Property: a closed side quest never becomes playable again.
- Snapshot / integration: the info line and the map-menu list.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
