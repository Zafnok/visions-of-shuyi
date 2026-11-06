---
id: "0850"
title: "Chapter 4 battle: The Hound's Country, map, units, places and balance"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0746", "0509", "0013", "0070", "0071", "1007", "1008", "0319", "1070", "0323", "0608", "0520", "0523"]
nick_input: none
completed: 2026-11-05
---

# 0850 — Chapter 4 battle: The Hound's Country

## Context

Act 1, Chapter 4, from `docs/story/outline.md`:

- **Story battle:** "Veyne Hall": storming the lead's childhood home; Hollis and the lead know the old servants' passages; Dace is the boss and escapes at low HP.
- **Objective:** as the outline's battle needs, with Dace retreating at the killing blow by 0319's rule; his retreat counts as defeating him and wins the battle (0069, `turn-structure.md`, *A boss who retreats*).
- **Recruits:** Gil Parrow (`prizefighter`, Brawler), recruited in the *Millhaven Road* skirmish (1071 builds the battle; this script writes his scenes).
- **New in this chapter:** the world map, a town (Millhaven), fixed skirmishes (*Aldwater Toll Bridge*, guarding the path; *Millhaven Road*, optional), the first side quest (Tamsin's, 1072).

0803 built Chapter 1 the same way (map, units, triggers, a winning
replay, a difficulty run); this ticket does it for Chapter 4, from its
beat sheet and script (0746). The balance comes from the bots: 0509's
`cargo shuyi autobalance` tunes the enemies until the persona bots land
in their bands (`docs/design/playtest-bots.md`), and 0013 has settled
the number scale first. As a world-map chapter it needs the world map (1007) and its skirmishes (1008), the new battle rules (0319), and the Act 1 world map's data (1070), whose story-battle node it fills in.

The new faces and combat pictures are 0070's picks; the scene and battle
music is 0071's.
0070 picked **Gil**: generator face B (slicked-back black hair, a scar over one eye, a goatee) and Sagak Art's "8-Bit Heroes 2: Fighter" at 1.75× (Nick buys it, $3). **Dace** keeps his face (0706) and the dark knight (0040). Generator faces are built by Nick in the Character Generator from a parts list this ticket gives him (Nick, 0070: the agent's pasted generator mockups aren't reliable); Shironejiya pictures come from <http://hi79.web.fc2.com/material/monster_s/> and carry that site's credit text (`look-and-feel.md`, *Fighters with no Tiny Tales art*).

## Nick input

None. No check after merge (Nick, 2026-10-21: he checks only design and
the full Act 1 playtest, 1064). Verification is by tests, `cargo shuyi
frame-png` pictures of the map in the PR and the bots' reports. Levels,
enemy counts, map size and turn counts are *the agent's starting values*
("for the chapter tickets", the outline), tuned by the bots; list them
in the Completion notes for 1064's checklist. A design question the
docs don't answer (a rule, not a number) goes to Nick with `ask-nick`
before building.

## Scope

**In:**
- **The map** `assets/maps/ch04.map` with the bought tilesets
  (ADR-0052 auto-tiling) and its glyph look; the tiles the battle needs.
- **The battle** `assets/battles/ch04.ron`: player slots
  (Preparations from Chapter 2 on), enemies from templates and named
  units, the objective, reinforcements and triggers the beat sheet
  calls for (talk, map areas, turn starts, boss and fall lines),
  `music: Cue(..)` from 0071, a seed.
- **The chapter** `assets/chapters/ch04.ron`: its scenes in order,
  `next: Some("ch05")` in `assets/chapters/ch04.ron`, and Chapter 3's `next` pointing here.
- **The world-map chapter** (1007's format): the chapter file says which
  nodes open and that this story battle ends it; the story-battle node
  1070 placed for Chapter 4 points at this battle (replacing 1070's
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
  `assets/dialogue/ch04.dlg`.
- **Battle notes** for the battle, where the design calls for them
  (`magic.md`).
- **Balance:** `cargo shuyi autobalance --stage ch04 --apply`, then
  `cargo xtask playtest ch04 --bot all`; the report in the PR.
- A winning-replay test and the content checks, as 0803 had.

**Out (do not do):**
- The script's words (0746); fixed skirmishes and towns (1071, 1070);
  side quests (1072, 1083, 1084); supports and camp events (1073).
- New rules in `core` (file a ticket; 0319 has the Act 1 ones).
- Changing earlier chapters' balance.

## Implementation steps

1. Read the beat sheet and script (0746), 0803 and its files, the
   design docs the extras touch, 0070's and 0071's answers.
2. Map first (`ascii-art` skill for the layout), then units and the
   battle file, then the chapter file.
3. Import the new art; wire the places and music into the script.
   When importing `battle_rival`, remove it from `RESERVED` in
   `crates/content/src/audio.rs`'s tests (its "drop it from
   RESERVED" assertion fails otherwise; 0252).
4. Autobalance; playtest; fix what the bots show (softlocks, a band
   missed).
5. `cargo shuyi frame-png` pictures of the map and one scene for the PR.

## Acceptance criteria

- [x] New Game reaches Chapter 4 through the chapters before it
      (harness test), and the battle can be won (winning-replay test).
- [x] The bots land in their bands (`playtest-bots.md`); the report is
      in the PR. (Every band on seeds 1–100 and 101–200; the "Hardcore
      faster" check is the bots', 0525.)
- [x] Every scene's place exists and the script passes
      `cargo xtask check-script`.
- [x] Starting values and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the winning replay; New Game → Chapter 4 start; the
  scripted join (in a scene) or join-if-defeated of this chapter's recruit, if any (no recruiting by talking, 0069).
- Content checks: the map, battle and chapter files load and validate.

## Completion notes

**Done.** Chapter 4 plays from New Game through Chapters 1 to 3: after
Chapter 3's save prompt the valley road plays, the Act 1 world map
shows, the army walks to Veyne Hall (through the Aldwater stand-in,
below), the rise and the gate play, then Preparations, the battle, the
stable, the Hound post and "To be continued" (Chapter 5 doesn't exist
yet: 0851 points Chapter 4's `next` at it).

- **Map** `assets/maps/ch04.map` (29×16, dawn): the road south of the
  gate (13, 12) with the guardroom's two forts behind it; the yard and
  the stable stalls; the hall along the north wall, Dace at its west
  end; the kitchen beside it, joined by the stair at (16, 3); the long
  wall on the east with the laundry door at (21, 10) and the servants'
  passage up to the kitchen; the river meadow, the mill pond and the
  river beyond. Picture: `docs/img/ch04-map.png`.
- **Battle** `assets/battles/ch04.ron`: defeat Dace, who retreats at 0 HP
  (`defeat: Retreats`), which wins; his Hounds run (`leftovers: Flee`).
  Every battle scene of `ch04.dlg` wired; two battle notes; music
  `battle_rival`.
- **Chapter** `assets/chapters/ch04.ron` on the Act 1 world map; Chapter
  3's `next` is `ch04`; Veyne Hall's place is `StoryBattle("ch04")`.
- **Opening scenes** (new, a little code): a world-map chapter's scenes
  when it begins, before its map shows (`opening_scenes`), for the
  valley road, which the beat sheet plays "when the chapter opens". The
  chapter format had no place for them: `intro_scenes` play on arriving
  at the story battle. Checked like the other scenes, and only on a
  world-map chapter (`assets/chapters/README.md`).
- **Units:** Dace (`rival`, Shadowblade) and Gil (`prizefighter`,
  Brawler) moved from `speakers` to `characters`: Veyne Hall has a slot
  for Gil and his death and retreat lines, which need him to be a unit.
- **Places:** `valley_road`, `veyne_rise`, `veyne_stable` (the yard and
  the stable), `millhaven_post`; the gate uses the battle's map.
- **Script:** the `@if` blocks the triggers made one-way are gone (Dace
  against Hollis, Rue and Joss; the fourteen death and retreat lines);
  `@place` lines added; the header says what 0850 wired. No words
  changed; `cargo xtask check-script`: 0 problems.
- **Music:** `battle_rival` imported (A Slave To No One, Jonathan Shaw,
  CC-BY 3.0, downloaded with Nick's OK), credited in `audio.ron` and
  `THIRD_PARTY_ASSETS.md`, dropped from `RESERVED`.

**A fault of the bots, fixed** (`crates/bots/src/arrival.rs`, with a
test that fails without it): the arrival army recruited only for the
chapter being played, so Chapter 4's army replayed Chapter 3 without
Joss (who joins after Chapter 2) and stopped with "no unit with id 7".
Each chapter on the way now gets its recruits before its battle.

**Balance** (`cargo xtask playtest ch04 --bot all`; the army Chapters 1
to 3 leave: Lead lv 5, Hollis lv 8, Tamsin lv 9, Aske lv 6, Maud lv 5,
Rue lv 12, Joss lv 3):

| Persona | Band | As built (seeds 1–100) | Final, seeds 1–100 | Final, seeds 101–200 |
| ------- | ---- | ---------------------- | ------------------ | -------------------- |
| Casual | 30–55% | 68 ⚠ | **52 ✓** | **40 ✓** |
| Normal | 50–75% | 53 ✓ | **61 ✓** | **60 ✓** |
| Hardcore | 50–75% | 63 ✓ | **63 ✓** | **56 ✓** |
| Hardcore turns (vs Normal) | lower | 7 vs 7 ✗ | 7 vs 7 ✗ | 7 vs 7 ✗ |

- `cargo shuyi autobalance --stage ch04` (12 evaluations before its
  time ran out) found no change to the unnamed enemies that helped. The
  three bots fail the same way here (the lead falls to Dace), so they
  move together; what separated them was a Dace hard to put down rather
  than hard-hitting (30 HP, Str 10, Def 6 → 40 HP, Str 11, Def 8) and
  his riders back on turn 7 (three) instead of turn 4 (two). The
  variants tried are in `docs/playtesting.md`.
- The `lower ✗` is the bots' by `playtest-bots.md`: ticket 0525.

**Winning replay:** `crates/ui/tests/it/fixtures/ch04_winning.ron`,
with the army Chapters 1 to 3 leave: won on turn 5, nobody lost, Dace
retreating. Two tests play it: the rules alone, and the real game from
New Game through the world map to "To be continued". Chapter 3's test
now runs on into Chapter 4. The bots' arrival has it for Chapter 5.

**Deviations:**

- **The art is follow-up 0862**, as 0848 and 0849 did: Gil's face is
  Nick's to build and his combat picture Nick's to buy; Dace's dark
  knight is ready in the private repository and goes in the same push.
  Until then Gil speaks with an empty portrait frame, and both use their
  class's pictures.
- **`next: None`** in `ch04.ron` (the ticket said `Some("ch05")`): a
  chapter that doesn't exist can't be named; 0851 sets it.
- **Aldwater Toll Bridge passes through** (an `Empty` place, no longer
  `NotBuilt`) until 1071 builds it: it is on the only road to the hall,
  so without it Chapter 4 couldn't be reached. Tamsin's side quest
  (`Cleared("aldwater")`) opens with 1071. Noted in 1071.
- **Gil became a unit here**, though 1071 says it makes him: the hall's
  triggers need him. Noted in 1071.
- **The bots' arrival fix** wasn't in the ticket by name; step 4 says to
  fix what the bots show, and nothing could be measured without it.
- **The opening scenes** are a small format and flow change the ticket
  didn't list; the beat sheet's first scene had nowhere else to go.

**Follow-up tickets:** 0862 (Chapter 4's art: Nick builds Gil's face and
buys his picture first), 0525 (Hardcore no faster than Normal at Veyne
Hall).

**The agent's starting values and rules** (the docs left them open; all
*tunable*, for the Act 1 playtest, 1064):

1. **Dace:** Shadowblade, level 8, 40 HP, Str 11, Dex 13, Spd 13, Def 8,
   Res 4, a Steel Sword and Iron Gauntlets, a Leather Vest. He holds the
   lord's end of the hall and comes at whoever reaches him (`Guard`).
2. **When Dace retreats, his Hounds run** (`leftovers: Flee`, as Chapter
   1's Harl): the battle ends at once either way.
3. **Gil:** Brawler, level 5, 27 HP, Str 8, Spd 10, Def 5, Iron
   Gauntlets and a Leather Vest; his slot at Veyne Hall stays empty
   unless he joined.
4. **The hall:** 12 unnamed Hounds at level 7: a guard on the gate, an
   archer in each guardroom fort, a raider, a swordsman and a rider in
   the yard, a guard on the hall's front door, two sentries in the river
   meadow, a raider in the kitchen, a guard and an archer with Dace. On
   turn 7 three riders come up the road behind the party.
5. **No turn limit, no chests.** The win gives 1800 gold; pack cap 3.
6. **The music** is A Slave To No One's first 4:30, faded out over 6 s
   (the piece is 5:51, over the 4.5-minute budget, with no quiet place
   to end on).
7. **The valley road plays before the world map** (as the beat sheet
   says); the rise and the gate when the army reaches the hall.

**Left to judge by playing (1064):** whether the laundry-door way reads
as the way in (Hollis's first-turn line and a battle note point at it)
or the gate is simply quicker; whether Dace's retreat feels earned or
abrupt; whether riders at the party's back on turn 7 feel fair; the
4:30 cut of the music and its fade; the four new places' look and light
(the yard after the battle in noon light, the gate scene at dawn); the
world map's first appearance after Chapter 3, with only Veyne Hall and
Millhaven to go to until 1071.
