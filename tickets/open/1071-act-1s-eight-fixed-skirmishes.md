---
id: "1071"
title: "Act 1's eight fixed skirmishes, with Gil's recruitment"
type: content
milestone: Post–Chapter 1
model: reasoning
effort: high
status: todo
blocked_by: ["1008", "1070", "0746", "0747", "0748", "0749", "0509", "0013", "0070", "0323", "0608"]
nick_input: none
completed:
---

# 1071 — Act 1's eight fixed skirmishes

## Context

`docs/story/outline.md` names two fixed skirmishes for each of Chapters
4–7 (`world-structure.md`: "authored like story battles (map, enemies,
possibly a short scene) but smaller"; some guard a path and must be won
to pass):

| Chapter | Guards the path | Optional |
| --- | --- | --- |
| 4 | *Aldwater Toll Bridge* (to Veyne Hall) | *Millhaven Road*: a Hound patrol, and a prize-fighter betting he can beat all of them: **Gil Parrow** joins here |
| 5 | *The Salt Road* (to Saltmere) | *Smugglers' Cove*, with loot |
| 6 | *The Burnt Mill* (to the plain; the first echoes) | *Brennish Outriders* |
| 7 | *Pilgrims' Road* (into Ossary) | *The Ossuary Gate* |

1008 builds fixed skirmishes; 1070 places their nodes on the Act 1 map,
locked; the chapter scripts (0746–0749) write their short scenes,
including Gil's recruitment. This ticket builds the eight battles and
fills in their nodes. They use the `skirmish` music pool (1008), are
balanced by the bots (0509) after 0013's number scale, and Gil's face
and combat picture are 0070's pick.

## Nick input

None. No check after merge (Nick, 2026-10-21): `frame-png` pictures of
each map and the bots' reports in the PR; levels, sizes and tiers are
*the agent's starting values* in the Completion notes for 1064's
checklist.

## Scope

**In:**
- Eight maps (`assets/maps/`) and battle files (`assets/battles/`),
  smaller than story battles, each with its tier (`world-structure.md`:
  normally Easy or Normal), its enemies from templates, its reward
  (`world-structure.md`; *Smugglers' Cove* "with loot"), and its scene
  triggers from the scripts.
- **Gil's recruitment** on *Millhaven Road*: as Chapter 4's script
  (0746) wrote it (he joins if defeated, 0069: win without beating him and he is missed); his unit in
  `characters.ron`, his art imported as 0070 recorded (private
  repository, `--pin`, credits).
- The eight nodes in 1070's map point at these battles; the guarding
  ones block the path until won (1008).
- `cargo shuyi autobalance --stage <id> --apply` and `cargo xtask playtest
  <id> --bot all` for each; the reports in the PR.

**Out (do not do):**
- Story battles (0850–0853), side quests (1072, 1083, 1084), random skirmishes
  (1070's pool), new rules.
- Stars on fixed skirmishes (0046's build tickets).

## Implementation steps

1. Read 1008's format and test skirmishes, 1070's map, the scenes in
   0746–0749, `world-structure.md`.
2. One skirmish at a time: map, battle, scene triggers, node.
3. Gil's unit and art.
4. Autobalance and playtest each; `frame-png` pictures.

## Acceptance criteria

- [ ] Each of the eight can be entered from its node and won (a
      winning-replay test per battle, or one test that plays them all).
- [ ] A guarding skirmish blocks its path until won (test).
- [ ] Gil joins on *Millhaven Road* as the script says (test).
- [ ] The bots land in their bands for each; reports in the PR.
- [ ] Starting values are in the Completion notes (1064's checklist).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: winning replays; Gil's recruitment; a guard blocking
  travel.
- Content checks for every new file.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
