---
id: "1020"
title: "Map editor, part 2: units, the battle file and playing the battle"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1019"]
nick_input: none
completed:
---

# 1020 — Map editor: units, the battle file and play

## Context

1019 paints terrain and saves a `.map`. A playable battle also needs a
battle file (`assets/battles/README.md`): player slots, enemies (template
or character, level, AI, boss), reinforcements by turn, the objective and
turn limit, dialogue triggers and talk pairs (ADR-0030, ADR-0035). This
ticket edits the units on top of the painted map, saves the battle file and
lets the modder play the battle straight from the editor. 1085 adds
reinforcements, the objective and turn limit, triggers and talk pairs (split
by ticket 0135: the two together are about 3,500 changed lines, past the
limit of about 3,000 for one PR; ADR-0101).

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Nick builds a small battle (a few enemies, Rout), plays it from the
> editor, and says what was hard to find.

## Scope

**In:**
- Placing and removing player slots and enemies on the map; picking a
  template or character, level, AI kind and boss flag from lists built from
  the loaded content (so mod-added classes appear).
- Saving the battle file next to the map; a battle-file writer in
  `trpg-content` with a round-trip test like 1019's.
- `Play`: validates and starts the battle in the normal battle screen with
  a temporary roster of the slotted characters; quitting returns to the
  editor.
- Everything on the map goes through the map scene and skins (ADR-0038).

**Out (do not do):** reinforcements, the objective and turn limit, triggers
and talk pairs (1085; until then a saved battle has the default Rout);
writing dialogue (0723's tools cover writing); the bots' balance report (1021); chapters and
campaigns (1022).

## Implementation steps

1. Read 1019's editor, ADR-0030, ADR-0035, `crates/content/src/battle*`,
   the flow screen that starts battles (`crates/ui/src/flow*`) and the
   Quick Battle path (which starts a battle without a campaign).
2. Add the battle-file writer with a round trip over `assets/battles/`.
3. Add the editing modes to the editor state (unit tests first), then the
   screen.
4. `Play` reuses the Quick Battle start path with the edited battle.
5. Snapshot tests of each mode.

Expected size: about 1,900 changed lines (0135; `cargo xtask pr-size`).

## Acceptance criteria

- [ ] Parse → write → parse of every file in `assets/battles/` gives the
      same battle (test).
- [ ] A battle built in the editor (Harness test) saves, loads with no
      content errors, and can be played to a Rout win by a scripted list of
      commands.
- [ ] Enemies and slots are in the map scene (tests read the scene).
- [ ] No hard-coded keys or literal screen text.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: editor modes.
- Property: the battle-file writer round trip.
- Snapshot / integration: each mode; the build-and-play Harness test.

## Completion notes
