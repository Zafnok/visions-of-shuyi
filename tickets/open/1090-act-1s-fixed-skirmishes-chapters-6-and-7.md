---
id: "1090"
title: "Act 1's fixed skirmishes of Chapters 6 and 7"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1071", "0748", "0749", "0509", "0013", "0323", "0608"]
nick_input: none
completed:
---

# 1090 — Act 1's fixed skirmishes of Chapters 6 and 7

## Context

Split from 1071 before coding (ADR-0101): the eight fixed skirmishes and
the work they bring with them (the bots' arrival army and the chapters'
recorded playthroughs fighting the guards on the way) were well over
2,000 changed lines. 1071 built Chapters 4 and 5's four (*Aldwater Toll
Bridge*, *Millhaven Road* with Gil, *The Salt Road*, *Smugglers' Cove*)
and the pattern this ticket follows; this one builds the other four from
`docs/story/outline.md`:

| Chapter | Guards the path | Optional |
| --- | --- | --- |
| 6 | *The Burnt Mill* (to the plain; the first echoes) | *Brennish Outriders* |
| 7 | *Pilgrims' Road* (into Ossary) | *The Ossuary Gate* |

Their scenes are written (`ch06.dlg`: `ch06_burnt_mill_intro`,
`_victory`, `ch06_outriders_intro`, `_victory`; `ch07.dlg`:
`ch07_pilgrims_road_intro`, `_victory`, `ch07_ossuary_gate_intro`,
`_victory`). Their places are `Empty` with `NotBuilt` in
`assets/worldmaps/act1.ron`, with the conditions they open by already
written (1070).

Read 1071's Completion notes and files first: its battle files
(`assets/battles/aldwater.ron` and the rest), its tests
(`crates/ui/tests/it/skirmishes.rs`), and how a guarding fixed skirmish
joins the bots' arrival army (`crates/bots/src/arrival.rs`) and the
chapter tests.

What the scenes and the bible say the fights hold:

- *The Burnt Mill*: an **echo** (`bible.md`, *Echoes*: a fire elemental,
  `classes.ron`'s `fire_elemental`, absorbs Fire, weak to Ice) coming up
  the millrace, the first in the story. The outline's battle note for
  the Ashfields reads "Echoes of Vael: weak to frost, drink fire"; use
  the same one here, since it is the first meeting. No generic template
  of the class exists yet: add one (`characters.ron`'s `generics`), and
  0852 reuses it.
- *Brennish Outriders*: Brennish scouts, mounted, coming down the hill
  road; one is left on the ground at the end (the scene).
- *Pilgrims' Road*: a warden post at a wayside shrine between two walls,
  with pilgrims; the Vigil's `warden_guard` and `warden_archer` templates
  exist.
- *The Ossuary Gate*: wardens at a low gate in the hillside.

## Nick input

None. No check after merge (Nick, 2026-10-21): `frame-png` pictures of
each map and the bots' reports in the PR; levels, sizes and tiers are
*the agent's starting values* in the Completion notes for 1064's
checklist.

## Scope

**In:**
- Four maps (`assets/maps/`) and battle files (`assets/battles/`), as
  1071's: smaller than story battles, each with its tier, enemies from
  templates, a reward, `kind: FixedSkirmish`, `music: Pool("skirmish")`
  and a `bots_army`.
- Their nodes in `act1.ron`: `FixedSkirmish` with the scenes, `guards_path`
  as the table says, `NotBuilt` removed, the other conditions kept.
- A guarding skirmish joins the arrival chain the way 1071 made it.
  **Either order with the story battles works:** if 0852 (the Ashfields)
  is done, Chapter 6's recorded playthrough and the bots' arrival army
  now fight *The Burnt Mill* first, as 1071 did for Chapters 4 and 5
  (record the Chapter 6 playthrough again if it no longer wins); if not,
  0852 does it, and this ticket adds a line to 0852 saying so. The same
  for *Pilgrims' Road* and 0853.
- A player slot for each character in the army by then (Hedda, and Oriel
  if 0852 has made her a unit; else 0852 adds her slots).
- `cargo shuyi autobalance --stage <id> --apply` and `cargo xtask playtest
  <id> --bot all` for each; the reports in the PR.

**Out (do not do):**
- Chapters 4 and 5's skirmishes (1071), the story battles (0852, 0853),
  side quests (1083, 1084), random skirmishes, new rules.
- Stars on fixed skirmishes (0046's build tickets).

## Implementation steps

Expected size: about 1,100 changed lines, judged by 1071's four (maps and
battles about 70 lines each, a winning fixture each, the tests, the
ticket file).

1. Read 1071's files and notes, the four scenes, `world-structure.md`.
2. One skirmish at a time: map, battle, node, winning fixture.
3. The arrival chain, as the Scope says for each order.
4. Autobalance and playtest each; `frame-png` pictures.

## Acceptance criteria

- [ ] Each of the four can be entered from its node and won (a
      winning-replay test per battle, as 1071's).
- [ ] *The Burnt Mill* and *Pilgrims' Road* block their paths until won
      (test).
- [ ] The bots land in their bands for each; reports in the PR.
- [ ] Starting values are in the Completion notes (1064's checklist).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: winning replays; a guard blocking travel.
- Content checks for every new file.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
