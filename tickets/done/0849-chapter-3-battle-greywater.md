---
id: "0849"
title: "Chapter 3 battle: Greywater, map, units, places and balance"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0745", "0509", "0013", "0070", "0071", "0037", "0322", "0069", "0323", "0608", "0520", "0523"]
nick_input: none
completed: 2026-11-04
---

# 0849 — Chapter 3 battle: Greywater

## Context

Act 1, Chapter 3, from `docs/story/outline.md`:

- **Story battle:** Greywater Abbey: a causeway to an island abbey, wardens and Hounds on the walls, the scriptorium as the Seize point, chests in the reliquary.
- **Objective:** Seize the scriptorium (the Seize objective exists).
- **Recruits:** none.
- **Named boss (0743):** Prior Edwin Sallow (`greywater_prior`), who dies or surrenders when beaten (Nick, 2026-10-28: "can say dies or surrenders"; this ticket picks). Sergeant Ned Tolley (`hound_sergeant`), an old Veyne guardsman, commands the Hounds guarding him: a named enemy, not the boss. Sheets in `docs/story/characters/`.
- **New in this chapter:** a Seize objective and chests; after this chapter the Act 1 world map opens.

0803 built Chapter 1 the same way (map, units, triggers, a winning
replay, a difficulty run); this ticket does it for Chapter 3, from its
beat sheet and script (0745). The balance comes from the bots: 0509's
`cargo shuyi autobalance` tunes the enemies until the persona bots land
in their bands (`docs/design/playtest-bots.md`), and 0013 has settled
the number scale first. It also needs 0037's building tiles (decided: `gate` and `throne`, built by 0322, for the abbey's gate and the scriptorium's Seize point; no tile heals) and 0069's confirmation of what Chapter 3 introduces.

The new faces and combat pictures are 0070's picks; the scene and battle
music is 0071's.
0070 made **the prior** Greywater's boss (not the Hound captain): Shironejiya's dark-robed priest `m_s_mage_dark_m01` at 1.25×, and a face only if he speaks outside the fight. Generator faces are built by Nick in the Character Generator from a parts list this ticket gives him (Nick, 0070: the agent's pasted generator mockups aren't reliable); Shironejiya pictures come from <http://hi79.web.fc2.com/material/monster_s/> and carry that site's credit text (`look-and-feel.md`, *Fighters with no Tiny Tales art*).

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
- **The map** `assets/maps/ch03.map` with the bought tilesets
  (ADR-0052 auto-tiling) and its glyph look; the tiles the battle needs.
- **The battle** `assets/battles/ch03.ron`: player slots
  (Preparations from Chapter 2 on), enemies from templates and named
  units, the objective, reinforcements and triggers the beat sheet
  calls for (talk, map areas, turn starts, boss and fall lines),
  `music: Cue(..)` from 0071, a seed.
- **The chapter** `assets/chapters/ch03.ron`: its scenes in order,
  `next: Some("ch04")` in `assets/chapters/ch03.ron`, and Chapter 2's `next` pointing here.
- **New units** in `characters.ron` (class, level, stats, loadout) for
  the recruits and named enemies; their faces, busts and combat pictures
  imported from the private repository with `cargo xtask
  portrait-import` / `battler-import` / `map-sprite-import`, as 0070
  recorded (push the private repository, `cargo shuyi private-assets
  --pin`, credits' `private` lists: CLAUDE.md, *Bought art*).
- **Conversation places**: the place maps the beat sheet names
  (`assets/maps/<place>.map`, as 0733 did for the yard and the study),
  the `place` directives and the `@music` lines in
  `assets/dialogue/ch03.dlg`.
- **Battle notes** for the battle, where the design calls for them
  (`magic.md`).
- **Balance:** `cargo shuyi autobalance --stage ch03 --apply`, then
  `cargo xtask playtest ch03 --bot all`; the report in the PR.
- A winning-replay test and the content checks, as 0803 had.

**Out (do not do):**
- The script's words (0745); fixed skirmishes and towns (1071, 1070);
  side quests (1072, 1083, 1084); supports and camp events (1073).
- New rules in `core` (file a ticket; 0319 has the Act 1 ones).
- Changing earlier chapters' balance.

## Implementation steps

1. Read the beat sheet and script (0745), 0803 and its files, the
   design docs the extras touch, 0070's and 0071's answers.
2. Map first (`ascii-art` skill for the layout), then units and the
   battle file, then the chapter file.
3. Import the new art; wire the places and music into the script.
4. Autobalance; playtest; fix what the bots show (softlocks, a band
   missed).
5. `cargo shuyi frame-png` pictures of the map and one scene for the PR.

## Acceptance criteria

- [x] New Game reaches Chapter 3 through the chapters before it
      (harness test), and the battle can be won (winning-replay test).
- [x] The bots land in their bands (`playtest-bots.md`); the report is
      in the PR.
- [x] Every scene's place exists and the script passes
      `cargo xtask check-script`.
- [x] Starting values and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the winning replay; New Game → Chapter 3 start; the
  scripted join (in a scene) or join-if-defeated of this chapter's recruit, if any (no recruiting by talking, 0069).
- Content checks: the map, battle and chapter files load and validate.

## Completion notes

**Done.** Chapter 3 plays from New Game through Chapters 1 and 2: the
boathouse, the causeway, Preparations, the battle at Greywater Abbey,
the scriptorium, the causeway at night, the kitchen, Veyne Hall's yard,
then "To be continued" (Chapter 4 doesn't exist yet: 0850 points
Chapter 3's `next` at it).

- **Map** `assets/maps/ch03.map` (28×16, dawn): the shore at the left,
  the causeway two wide over the lake with a landing halfway, the gate
  (two `gate` tiles) between two towers, the courtyard, the scriptorium
  over the gate with the prior's seat (a `throne`, the Seize point) at
  its west end, the reliquary with three chests; a wader can go round by
  the shallows to a breach in the west wall. Picture:
  `docs/img/ch03-map.png`.
- **Battle** `assets/battles/ch03.ron`: seize the prior's seat; every
  scene id of `ch03.dlg` wired; one battle note (only the exile seizes,
  and the prior sits on the seat); music `battle_church_2`.
- **Chapter** `assets/chapters/ch03.ron`; Chapter 2's `next` is `ch03`.
- **Units:** the prior (`greywater_prior`) and Ned Tolley
  (`hound_sergeant`) moved from `speakers` to `characters`; two warden
  templates (`warden_guard`, `warden_archer`); the Hounds' own.
- **Places:** `greywater_boathouse` (the boathouse), `greywater_causeway`
  (the causeway at night), `greywater_kitchen` (the kitchen),
  `veyne_yard` (to be continued); the causeway's foot and the
  scriptorium use the battle's own map (`@place battle`).
- **Script:** the `@if` blocks the triggers made one-way are gone (Ned
  against Hollis, the prior against Maud, the twelve death and retreat
  lines); the `@place` lines added. No words changed.

**Two faults of the bots on a Seize map** (Chapter 3 is the first story
battle with one), fixed in `crates/bots/src/persona.rs` with a test each
that fails without the fix:

- **A persona never seized.** The order (ADR-0095, ADR-0112) lists each
  unit's escape early on an escape map; a `Seize` fell among "the rest",
  past the places tried, so a lead standing on the tile never took it
  (Normal and Hardcore ran to the turn cap). A seize is now listed with
  the escapes.
- **The lead walked away from the causeway.** A seize's `progress`
  counted the tiles to the Seize point as the crow flies: from the shore
  the corner at (3, 3) is nearest the scriptorium across the lake, so the
  Hardcore lead went there on turn 1 and stayed, every step out along the
  causeway scoring as going back. It now counts walking (the flow field,
  as `approach` already did); as the crow flies only when nobody can walk
  there. Neither change touches a battle without a Seize, so Chapters 1
  and 2 play as before. The replay recorder (`playthrough.rs`) steers the
  lord to the Seize point the same way.

**Balance** (`cargo xtask playtest ch03 --bot all`; the army Chapters 1
and 2's recorded playthroughs leave, `trpg_bots::arrival`):

| Persona | Band | As built (30 tries) | Seize fixed | Walking | Ned, prior down | Pack cap 3 | Seeds 1–100 | Seeds 101–200 |
| ------- | ---- | ------------------- | ----------- | ------- | --------------- | ---------- | ----------- | ------------- |
| Casual | 30–55% | 7 ✗ | 9 ✓ | 7 ✗ | 16 ✓ | 9 ✓ | **37 ✓** | **34 ✓** |
| Normal | 50–75% | 0 ✗ | 0 ✗ | 3 ✗ | 3 ✗ | 22 ✓ | **67 ✓** | **62 ✓** |
| Hardcore | 50–75% | 0 ✗ | 3 ✗ | 16 ✓ | 20 ✓ | 20 ✓ | **52 ✓** | **66 ✓** |
| Hardcore turns (vs Normal) | lower | – | 11 vs 12 ✓ | 9 vs 11 ✓ | 9 vs 11 ✓ | 9 vs 11 ✓ | **9 vs 10 ✓** | **9 vs 11 ✓** |

- `cargo shuyi autobalance --stage ch03` (50 tries, 30 evaluations, run
  after the two fixes) found no change to the unnamed enemies that
  brought Casual or Normal into their bands, and changed nothing. What
  held them back was the named units and the pack: Ned on the gate and
  the prior on his seat (autobalance never touches named units), and
  Normal winning 27 of 30 but drinking all **4** Potions in the pack,
  where its success allows 3. With a pack of 3 it wins on 3.
- The unnamed enemies stay at level 6, as first built.
- Casual still stalls in about 40 of its 100 tries: it loses the army at
  the gate and the lead waits alone, out of reach of the prior (a
  failure either way; its success rate is in band).

**Winning replay:** `crates/ui/tests/it/fixtures/ch03_winning.ron`,
with the army Chapters 1 and 2's recorded playthroughs leave (Joss
joined): won on turn 8, nobody lost, the lead seizing the prior's
seat after Rue's fire felled him. Two tests play it: the rules alone, and the real
game from New Game through Chapters 1 and 2 to "To be continued".
Chapter 2's test now runs on into Chapter 3. The bots' arrival
(`trpg_bots::arrival`) has it for Chapter 4.

**Deviations:**

- **The art is follow-up 0861**, not done here, as 0848 did with 0858:
  the prior's and Ned's faces are Nick's to build in the generator (0861
  has the parts lists), and the prior's combat picture comes from the
  Shironejiya site that answered a script with a placeholder page. Until
  then both speak with empty portrait frames and use their class's map
  sprite and combat picture.
- **`next: None`** in `ch03.ron` (the ticket said `Some("ch04")`): a
  chapter that doesn't exist can't be named; 0850 sets it.
- **The objective line reads `Seize the throne`**: a Seize names its
  tile's terrain. Follow-up 0331 lets a battle name it (`the
  scriptorium`), as 0324 did for an escape's exit.
- **The bots' two fixes** were not in the ticket's scope by name; its
  step 4 says to fix what the bots show, and nothing could be balanced
  without them.

**Follow-up tickets:** 0861 (Chapter 3's art: Nick builds the two faces
first), 0331 (a Seize objective names its place).

**The agent's starting values and rules** (the docs left them open; all
*tunable*, for the Act 1 playtest, 1064):

1. **Only the lead may seize** (Fire Emblem's Seize, `terrain.md`: "Act
   1 uses only Fire Emblem's Seize"), and the prior **sits on the seat**,
   so he is beaten before it is taken (and the script, which has him
   dead, always holds). No turn limit.
2. **The prior:** Mage, level 6, 24 HP, Mag 10, Def 2, Res 8, Fire and
   Frost, a Linen Robe; he never leaves his seat (+3 Def, +30 Avoid on
   the throne tile).
3. **Ned Tolley:** Swordsman, level 5, 26 HP, Str 8, Def 6, an Iron
   Sword and a Leather Vest, on the gate (+3 Def, +20 Avoid); a named
   enemy, not a boss: killing him isn't needed.
4. **The abbey:** 12 unnamed enemies at level 6: two Hounds on the
   causeway's landing, a warden on the gate beside Ned, a Hound archer in
   each tower, four in the courtyard (a warden, a raider, a rider, a
   warden archer), a Hound and a warden at the top of the scriptorium's
   stair, a warden archer in the reliquary. All hold their ground
   (`Guard`, the tower archers `Stationary`) but the archers' reach.
5. **The chests:** 800 gold, an Elixir (one of Act 1's handful), a
   Focus Charm.
6. **The win gives 1500 gold.** Pack cap **3** (Chapter 2: 4).

**Left to judge by playing (1064):** whether the causeway and the gate
feel like a siege or a slog; whether the prior falls too fast (Rue's
fire) for his lines to play; whether a pack of 3 after Chapter 2's 4
reads as a choice; whether the chests are worth the detour; the four new
places' look and light; `Seize the throne` until 0331.
