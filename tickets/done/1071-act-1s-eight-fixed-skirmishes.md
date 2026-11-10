---
id: "1071"
title: "Act 1's fixed skirmishes of Chapters 4 and 5, with Gil's recruitment"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1008", "1070", "0746", "0747", "0748", "0749", "0509", "0013", "0070", "0323", "0608"]
nick_input: none
completed: 2026-11-09
---

# 1071 — Act 1's fixed skirmishes of Chapters 4 and 5

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

**Split (2026-11-09, ADR-0101):** this ticket builds Chapters 4 and 5's
four and makes a guarding skirmish part of the army a chapter arrives
with (the chapter tests' and the bots'); Chapters 6 and 7's four are
**1090**. The eight were well over 2,000 changed lines with that work.

1008 builds fixed skirmishes; 1070 places their nodes on the Act 1 map,
locked; the chapter scripts (0746–0749) write their short scenes,
including Gil's recruitment. This ticket builds the eight battles and
fills in their nodes. They use the `skirmish` music pool (1008), are
balanced by the bots (0509) after 0013's number scale, and Gil's face
and combat picture are 0070's pick.

**From 0850:** Gil is already a unit in `characters.ron` (a Brawler,
level 5, *Claude's starting values*: Veyne Hall has a slot for him and
his death and retreat lines), so this ticket places him on *Millhaven
Road* and may retune him; his art is 0862. *Aldwater Toll Bridge* is an
`Empty` place on the Act 1 map until this ticket, so the army can walk
through it to Veyne Hall: give it its kind (`guards_path: true`) here,
and Tamsin's side quest (`Cleared("aldwater")`) opens with it.

**From 0851:** *The Salt Road* is the same: an `Empty` place the army
walks through to Saltmere and the Envoy's House. Give it its kind
(`guards_path: true`), and put back `Cleared("salt_road")` as the
`opens_when` of Saltmere and the Envoy's House (both open on
`Cleared("veyne_hall")` until then, since a place to pass through can't
be cleared). The Salt Road's and the cove's scenes in `ch05.dlg` are
written and wait for their battles' triggers.

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

- [x] Each of the four (the other four are 1090's) can be entered from
      its node and won (a winning-replay test per battle; the guards and
      *Millhaven Road* through the real game too).
- [x] A guarding skirmish blocks its path until won (test).
- [x] Gil joins on *Millhaven Road* as the script says (test).
- [x] The bots land in their bands for each; reports in the PR.
      (Chapters 4 and 5 moved out of theirs: 0529.)
- [x] Starting values are in the Completion notes (1064's checklist).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: winning replays; Gil's recruitment; a guard blocking
  travel.
- Content checks for every new file.

## Completion notes

**Split first (ADR-0101).** The eight skirmishes, together with the work
they bring (the bots' arrival army and Chapters 4 and 5's recorded
playthroughs must fight the guards on the way), were well over 2,000
changed lines. So this ticket builds Chapters 4 and 5's four, and
Chapters 6 and 7's four are **1090** (blocked by this one; 1064, 0512,
0852, 0853 and the roadmap updated).

**What was done.**

- **Four maps and battles** (`assets/maps/`, `assets/battles/`, 18 × 12
  each), on the Act 1 map as `FixedSkirmish` places with their scenes
  from `ch04.dlg` and `ch05.dlg`:
  - *Aldwater Toll Bridge* (`aldwater`, guards the road to Veyne Hall,
    Easy, Rout): a Hound bar on the bridge, two Hounds on the near bank,
    archers and a rider on the far bank; the river is waded on foot only.
  - *Millhaven Road* (`millhaven_road`, optional, Normal): Gil in a ring
    of Hounds, the patrol's leader and a rider toward the town.
  - *The Salt Road* (`salt_road`, guards the way to Saltmere, Easy,
    Rout): a Hound post at the march stone and its hut.
  - *Smugglers' Cove* (`smugglers_cove`, optional, Normal, Rout): the
    beach under the cliffs, reached by the track or Aske's gorse path.
- **Saltmere and the Envoy's House open on `Cleared("salt_road")` again**,
  as 0851 asked; the cove opens with them.
- **Gil's recruitment** as 0069 and the script have it. *Millhaven Road*
  is won by beating the patrol's leader (`DefeatUnit`, `leftovers:
  Flee`), so Gil is an optional sub-boss: beat him first and he falls,
  says `ch04_prizefighter_fell` and joins after the fight; win without
  beating him and he runs with the Hounds and is missed. The victory
  scene's `@if prizefighter` picks the version. Two battle notes say so.
- **The patrol's leader** is a new nameless character, `patrol_leader`,
  named **"Corporal"** on his plate, like the Officer and the Knight (a
  one-word title: the names test reads two words as a person's name).
- **The presence check** (ADR-0055) now knows the world maps' places: a
  place's intro scene can count on the named enemies its battle fields
  (`Cast::with_places`, `worldmap::place_scenes`). That is what lets Gil
  speak in `ch04_millhaven_road_intro` before he is in the army.
- **The guards are part of the army a chapter arrives with.** Every
  player must win them, so:
  - the bots' arrival army (`crates/bots/src/arrival.rs`) wins them
    first, each with its recorded playthrough (`guards_by_chapter`: the
    guards each chapter's army must win on the way, from the map's
    routes);
  - a guard is itself played by the bots with the army its chapter
    brings (no `bots_army`);
  - Chapters 4 and 5's tests win them on the way, through the real game
    too, and their playthroughs were recorded again.
- **The recorder** (`playthrough::record_where`) can be told which wins
  count, so *Millhaven Road*'s playthrough beats Gil before the leader.

**Deviations.**

- **Gil's art isn't here:** it is 0862's (*Chapter 4 art: Gil and Dace*),
  as the ticket's note from 0850 says.
- **Gil fights the party only.** The rules have no three-sided fights, so
  the Hounds around him are on his side.
- **The smugglers use the privateers' templates** (sailors who are
  nobody's soldiers, 1072) instead of new ones.
- **Chapters 4 and 5 moved out of their bands** once their armies win the
  guards first. Their tuning is **0529** (below).

**The agent's starting values** (for 1064's checklist):

- Tiers: the guards **Easy**, the optional ones **Normal**.
- Rewards: Aldwater **400** gold (the toll-chest), Millhaven Road **300**,
  the Salt Road **500**, Smugglers' Cove **800, an Elixir and two
  Potions** (its loot).
- Levels, after the autobalance:
  - Aldwater: Hounds lv 9, the swordsman lv 11.
  - Millhaven Road: the ring's Hounds lv 1, Gil lv 5, the Corporal lv 6.
  - The Salt Road: Hounds lv 11–12, the guard at the stone removed.
  - The cove: privateers lv 12–13.
- The bots' army for the optional ones (`bots_army`): Chapter 4's at lv 6,
  Chapter 5's at lv 8.

**To judge by playing (1064):**

- Whether *Millhaven Road*'s level marker misleads. It reads **Lv 2**,
  since the ring's Hounds are lv 1, but Gil and the Corporal are the
  fight.
- Whether the guards feel like a toll on the road or a chore.
- Whether "Corporal" reads well.

**The bots** (100 tries, seeds 1–100; Casual / Normal / Hardcore success;
skirmish bands Easy 70–100 / 70–95 / 70–95, Normal 70–100 / 55–80 /
55–80):

| Battle | Army | Before the last autobalance | After it |
| --- | --- | --- | --- |
| Aldwater Toll Bridge | after Chapter 3 | 98 / 96 / 96 | **100 / 89 / 90** ✓ |
| Millhaven Road | reference lv 6 | 91 / 26 / 29 | **93 / 56 / 66** ✓ |
| The Salt Road | after Chapter 4 | 99 / 82 / 89 | **99 / 82 / 89** ✓ |
| Smugglers' Cove | reference lv 8 | 100 / 93 / 88 | **98 / 67 / 77** ✓ |

Aldwater and the Salt Road were first balanced against a reference army.
That made the Salt Road's Hounds give the real, stronger army too much
EXP, so both were balanced again with the army that really arrives. The
Salt Road's "as built" row is with that army, after its first
autobalance; it needed no further change.

Chapters 4 and 5 after this change (story bands 30–55 / 50–75 / 50–75,
Chapter 5's Casual 40–70):

| Battle | Before | After |
| --- | --- | --- |
| Chapter 4 | 52 / 61 / 63 | 59 ⚠ / 58 / 51 |
| Chapter 5 | 58 / 58 / 69 | 77 ⚠ / 85 ⚠ / 13 ✗ |

Chapter 5 is still won 98% of the time by every bot. Its Hardcore loses
someone more often than Normal and uses all three items (its success
allows one). **0529** finds why and brings both chapters back inside
their bands. The CI playtest fails only under 20% Casual, so nothing is
blocked meanwhile.

**Pictures in the PR:** `docs/img/1071-aldwater.png`,
`-millhaven_road.png`, `-salt_road.png`, `-smugglers_cove.png` (each
battle at its start, the glyph look).

**Follow-ups:** **1090** (Chapters 6 and 7's skirmishes), **0529**
(Chapters 4 and 5's bands).

**Sizes.** Engine friction: estimate 50 production lines (the guard loop
in the bots’ arrival, judged by its per-chapter loop), measured 172
(3.4×). Review: outcome 4, accept: all game logic, no glue around the
engine; the estimate missed the bots’ army for a guard fight itself and
the presence rule for places (`docs/engine/friction.md`). PR size:
estimate 1,500, measured 2,275: past the 2,000 a ticket is planned
at, after the split; the recorded playthroughs (about 450 lines, four new
and two recorded again) are most of the difference.

