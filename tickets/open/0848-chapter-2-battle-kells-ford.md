---
id: "0848"
title: "Chapter 2 battle: Kell's Ford, map, units, places and balance"
type: content
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["0744", "0509", "0013", "0070", "0071", "0313", "0409", "0443", "0069", "0323", "0608"]
nick_input: none
completed:
---

# 0848 — Chapter 2 battle: Kell's Ford

## Context

Act 1, Chapter 2, from `docs/story/outline.md`:

- **Story battle:** Kell's Ford: a river village split by a ford, houses to save before the raiders burn them (0069's *save the village*: a minimum to save, extra rewards past it), the garrison's watchtower across the water, a Hound lieutenant as boss.
- **Objective:** defeat the boss (the outline: "the chapter ticket decides"; a Defeat-unit objective exists) and save at least a minimum of the houses (`turn-structure.md`, *Save the village*; the minimum, the houses and the extra rewards are *Claude's starting values* for the bots to tune).
- **Recruits:** Joss Pellam (`defector`, Swordsman), joins in the scene after the battle (0069: no recruiting by talking).
- **New in this chapter:** Preparations (0443's shop tab) for the first time, save the village (0313), a recruit who joins in a scene (0069).

0803 built Chapter 1 the same way (map, units, triggers, a winning
replay, a difficulty run); this ticket does it for Chapter 2, from its
beat sheet and script (0744). The balance comes from the bots: 0509's
`cargo xtask autobalance` tunes the enemies until the persona bots land
in their bands (`docs/design/playtest-bots.md`), and 0013 has settled
the number scale first. It also needs the save-the-village rule (0313), the shop screen (0409), the Preparations shop (0443), and 0069's confirmation of what Chapter 2 introduces.

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
- **The map** `assets/maps/ch02.map` with the bought tilesets
  (ADR-0052 auto-tiling) and its glyph look; the tiles the battle needs.
- **The battle** `assets/battles/ch02.ron`: player slots
  (Preparations from Chapter 2 on), enemies from templates and named
  units, the objective, reinforcements and triggers the beat sheet
  calls for (talk, map areas, turn starts, boss and fall lines),
  `music: Cue(..)` from 0071, a seed.
- **The chapter** `assets/chapters/ch02.ron`: its scenes in order,
  `next: Some("ch03")` in `assets/chapters/ch02.ron`, and Chapter 1's `next` pointing here.
- **New units** in `characters.ron` (class, level, stats, loadout) for
  the recruits and named enemies; their faces, busts and combat pictures
  imported from the private repository with `cargo xtask
  portrait-import` / `battler-import` / `map-sprite-import`, as 0070
  recorded (push the private repository, `cargo shuyi private-assets
  --pin`, credits' `private` lists: CLAUDE.md, *Bought art*).
- **Conversation places**: the place maps the beat sheet names
  (`assets/maps/<place>.map`, as 0733 did for the yard and the study),
  the `place` directives and the `@music` lines in
  `assets/dialogue/ch02.dlg`.
- **Battle notes** for the battle, where the design calls for them
  (`magic.md`).
- **Balance:** `cargo xtask autobalance ch02 --apply`, then
  `cargo xtask playtest ch02 --bot all`; the report in the PR.
- A winning-replay test and the content checks, as 0803 had.

**Out (do not do):**
- The script's words (0744); fixed skirmishes and towns (1071, 1070);
  side quests (1072); supports and camp events (1073).
- New rules in `core` (file a ticket; 0319 has the Act 1 ones).
- Changing earlier chapters' balance.

## Implementation steps

1. Read the beat sheet and script (0744), 0803 and its files, the
   design docs the extras touch, 0070's and 0071's answers.
2. Map first (`ascii-art` skill for the layout), then units and the
   battle file, then the chapter file.
3. Import the new art; wire the places and music into the script.
4. Autobalance; playtest; fix what the bots show (softlocks, a band
   missed).
5. `cargo shuyi frame-png` pictures of the map and one scene for the PR.

## Acceptance criteria

- [ ] New Game reaches Chapter 2 through the chapters before it
      (harness test), and the battle can be won (winning-replay test).
- [ ] The bots land in their bands (`playtest-bots.md`); the report is
      in the PR.
- [ ] Every scene's place exists and the script passes
      `cargo xtask check-script`.
- [ ] Starting values and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the winning replay; New Game → Chapter 2 start; the
  scripted join (in a scene) or join-if-defeated of this chapter's recruit, if any (no recruiting by talking, 0069).
- Content checks: the map, battle and chapter files load and validate.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
