---
id: "1085"
title: "Map editor, part 3: reinforcements, objectives, triggers and talk pairs"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1020"]
nick_input: none
completed:
---

# 1085 — Map editor: reinforcements, objectives, triggers and talk pairs

## Context

Split from 1020 by ticket 0135 (the two together are about 3,500 changed
lines, past the limit of about 3,000 for one PR; ADR-0101). 1020 gives the
editor units, a saved battle file (with its writer) and `Play`; this ticket
adds the rest of the battle file (`assets/battles/README.md`, ADR-0030,
ADR-0035).

## Nick input

None. Verified by tests and `cargo shuyi frame-png` pictures in the PR; what
is left to judge by playing goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- Reinforcements: a turn number and units, shown with a marker per turn.
- The objective and turn limit from the existing set; a scripted objective
  by name (1018) if that ticket is done, otherwise left out.
- Area triggers and talk pairs, naming scenes that exist in the bundle.
- Everything on the map goes through the map scene and skins (ADR-0038).

**Out (do not do):** writing dialogue (scenes are named, not written;
0723's tools cover writing); the bots' balance report (1021); chapters and
campaigns (1022).

## Implementation steps

1. Read 1020's editor modes and battle-file writer, ADR-0030 and ADR-0035.
2. Add the editing modes to the editor state (unit tests first), then the
   screen.
3. Snapshot tests of each mode.

Expected size: about 1,500 changed lines (0135; `cargo xtask pr-size`).

## Acceptance criteria

- [ ] A battle with a reinforcement, a turn limit, a trigger and a talk pair
      built in the editor (Harness test) saves, loads with no content errors
      and plays: the reinforcement arrives on its turn.
- [ ] Reinforcement markers are in the map scene (tests read the scene).
- [ ] No hard-coded keys or literal screen text.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: editor modes.
- Snapshot / integration: each mode; the build-and-play Harness test.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
