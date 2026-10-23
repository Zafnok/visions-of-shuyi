---
id: "0854"
title: "Chapter 8 battle: The Last Watch, map, units, places and balance"
type: content
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["0750", "0509", "0013", "0070", "0071", "1007", "1008", "0319", "1070", "0323", "0608"]
nick_input: none
completed:
---

# 0854 — Chapter 8 battle: The Last Watch

## Context

Act 1, Chapter 8, from `docs/story/outline.md`:

- **Story battle:** "Saltmere Docks": hold the dock for a number of turns while the ship loads, against Hounds from three streets; Dace is the boss and can't be killed; he retreats.
- **Objective:** Survive for a number of turns, with Dace unkillable by 0319's rule (at 0 HP he is down: grayed out, no actions, untargetable, blocking his tile); Hollis is deployed, fights, and dies in the closing scene in every mode (0069, `turn-structure.md` and `death-and-difficulty.md`).
- **Recruits:** none.
- **New in this chapter:** nothing new.

0803 built Chapter 1 the same way (map, units, triggers, a winning
replay, a difficulty run); this ticket does it for Chapter 8, from its
beat sheet and script (0750). The balance comes from the bots: 0509's
`cargo shuyi autobalance` tunes the enemies until the persona bots land
in their bands (`docs/design/playtest-bots.md`), and 0013 has settled
the number scale first. As a world-map chapter it needs the world map (1007) and its skirmishes (1008), the new battle rules (0319), and the Act 1 world map's data (1070), whose story-battle node it fills in.

The new faces and combat pictures are 0070's picks; the scene and battle
music is 0071's.

## Nick input

None. No check after merge (Nick, 2026-10-21: he checks only design and
the full Act 1 playtest, 1064). Verification is by tests, `cargo shuyi
frame-png` pictures of the map in the PR and the bots' reports. Levels,
enemy counts, map size and turn counts are *Claude's starting values*
("for the chapter tickets", the outline), tuned by the bots; list them
in the Completion notes for 1064's checklist. A design question the
docs don't answer (a rule, not a number) goes to Nick with `ask-nick`
before building.

## Scope

**In:**
- **The map** `assets/maps/ch08.map` with the bought tilesets
  (ADR-0052 auto-tiling) and its glyph look; the tiles the battle needs.
- **The battle** `assets/battles/ch08.ron`: player slots
  (Preparations from Chapter 2 on), enemies from templates and named
  units, the objective, reinforcements and triggers the beat sheet
  calls for (talk, map areas, turn starts, boss and fall lines),
  `music: Cue(..)` from 0071, a seed.
- **The chapter** `assets/chapters/ch08.ron`: its scenes in order,
  no `next` (1065 ends the act after it), and Chapter 7's `next` pointing here.
- **The world-map chapter** (1007's format): the chapter file says which
  nodes open and that this story battle ends it; the story-battle node
  1070 placed for Chapter 8 points at this battle (replacing 1070's
  stand-in). Nodes, towns and skirmishes themselves are 1070 and 1071's.
- **New units** in `characters.ron` (class, level, stats, loadout) for
  the recruits and named enemies; their faces, busts and combat pictures
  imported from the private repository with `cargo xtask
  portrait-import` / `battler-import` / `map-sprite-import`, as 0070
  recorded (push the private repository, `cargo shuyi private-assets
  --pin`, credits' `private` lists: CLAUDE.md, *Bought art*).
- **Conversation places**: the place maps the beat sheet names
  (`assets/maps/<place>.map`, as 0733 did for the yard and the study),
  the `place` directives and the `@music` lines in
  `assets/dialogue/ch08.dlg`.
- **Battle notes** for the battle, where the design calls for them
  (`magic.md`).
- **Balance:** `cargo shuyi autobalance --stage ch08 --apply`, then
  `cargo xtask playtest ch08 --bot all`; the report in the PR.
- A winning-replay test and the content checks, as 0803 had.

**Out (do not do):**
- The script's words (0750); fixed skirmishes and towns (1071, 1070);
  side quests (1072); supports and camp events (1073).
- New rules in `core` (file a ticket; 0319 has the Act 1 ones).
- Changing earlier chapters' balance.

## Implementation steps

1. Read the beat sheet and script (0750), 0803 and its files, the
   design docs the extras touch, 0070's and 0071's answers.
2. Map first (`ascii-art` skill for the layout), then units and the
   battle file, then the chapter file.
3. Import the new art; wire the places and music into the script.
4. Autobalance; playtest; fix what the bots show (softlocks, a band
   missed).
5. `cargo shuyi frame-png` pictures of the map and one scene for the PR.

## Acceptance criteria

- [ ] New Game reaches Chapter 8 through the chapters before it
      (harness test), and the battle can be won (winning-replay test).
- [ ] The bots land in their bands (`playtest-bots.md`); the report is
      in the PR.
- [ ] Every scene's place exists and the script passes
      `cargo xtask check-script`.
- [ ] Starting values and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the winning replay; New Game → Chapter 8 start; the
  scripted join (in a scene) or join-if-defeated of this chapter's recruit, if any (no recruiting by talking, 0069).
- Content checks: the map, battle and chapter files load and validate.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
