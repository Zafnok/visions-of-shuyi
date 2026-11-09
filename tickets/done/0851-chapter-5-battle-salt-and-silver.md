---
id: "0851"
title: "Chapter 5 battle: Salt and Silver, map, units, places and balance"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0747", "0509", "0013", "0070", "0071", "1007", "1008", "0319", "1070", "0323", "0608", "0076", "0520", "0523"]
nick_input: none
completed: 2026-11-08
---

# 0851 — Chapter 5 battle: Salt and Silver

## Context

Act 1, Chapter 5, from `docs/story/outline.md`:

- **Story battle:** "The Envoy's House": holding a harbour-front house at night against Hounds from the docks and the rooftops, for a number of turns.
- **Objective:** Survive for a number of turns (the Survive objective exists; the count is a *the agent's starting value* the bots tune).
- **Recruits:** Hedda Ravn (`shieldbearer`, Raider), loaned by Ragna "to make sure you get there".
- **New in this chapter:** nothing new.

0803 built Chapter 1 the same way (map, units, triggers, a winning
replay, a difficulty run); this ticket does it for Chapter 5, from its
beat sheet and script (0747). The balance comes from the bots: 0509's
`cargo shuyi autobalance` tunes the enemies until the persona bots land
in their bands (`docs/design/playtest-bots.md`), and 0013 has settled
the number scale first. As a world-map chapter it needs the world map (1007) and its skirmishes (1008), the new battle rules (0319), and the Act 1 world map's data (1070), whose story-battle node it fills in.

The new faces and combat pictures are 0070's picks; the scene and battle
music is 0071's.
0070 picked **Ragna**'s face: a generator face with dark-blonde hair in a bun and a traveller's coat (she never fights). **Hedda**'s combat picture is 0076's pick: elie automata's **Battle Axemaiden** (<https://elie-automata.itch.io/battle-axemaiden-animated-character-story-art>, free; AI-assisted, so the Steam page discloses it, 0903; the licence asks for the credit "elie automata"), in her three-quarter pose. Her face is a generator face to match it (long red ponytail, about 32, weathered; Nick chose it over the pack's own painted portraits), and she is now 32, not 44 (`shieldbearer.md`). Generator faces are built by Nick in the Character Generator from a parts list this ticket gives him (Nick, 0070: the agent's pasted generator mockups aren't reliable); Shironejiya pictures come from <http://hi79.web.fc2.com/material/monster_s/> and carry that site's credit text (`look-and-feel.md`, *Fighters with no Tiny Tales art*).

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
- **The map** `assets/maps/ch05.map` with the bought tilesets
  (ADR-0052 auto-tiling) and its glyph look; the tiles the battle needs.
- **The battle** `assets/battles/ch05.ron`: player slots
  (Preparations from Chapter 2 on), enemies from templates and named
  units, the objective, reinforcements and triggers the beat sheet
  calls for (talk, map areas, turn starts, boss and fall lines),
  `music: Cue(..)` from 0071, a seed.
- **The chapter** `assets/chapters/ch05.ron`: its scenes in order,
  `next: Some("ch06")` in `assets/chapters/ch05.ron`, and Chapter 4's `next` pointing here.
- **The world-map chapter** (1007's format): the chapter file says which
  nodes open and that this story battle ends it; the story-battle node
  1070 placed for Chapter 5 points at this battle (replacing 1070's
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
  `assets/dialogue/ch05.dlg`.
- **Battle notes** for the battle, where the design calls for them
  (`magic.md`).
- **Balance:** `cargo shuyi autobalance --stage ch05 --apply`, then
  `cargo xtask playtest ch05 --bot all`; the report in the PR.
- A winning-replay test and the content checks, as 0803 had.

**Out (do not do):**
- The script's words (0747); fixed skirmishes and towns (1071, 1070);
  side quests (1072, 1083, 1084); supports and camp events (1073).
- New rules in `core` (file a ticket; 0319 has the Act 1 ones).
- Changing earlier chapters' balance.

## Implementation steps

1. Read the beat sheet and script (0747), 0803 and its files, the
   design docs the extras touch, 0070's and 0071's answers.
2. Map first (`ascii-art` skill for the layout), then units and the
   battle file, then the chapter file.
3. Import the new art; wire the places and music into the script.
   When importing `scene_confide` and `battle_night_siege`, remove each from `RESERVED` in
   `crates/content/src/audio.rs`'s tests (its "drop it from
   RESERVED" assertion fails otherwise; 0252).
4. Autobalance; playtest; fix what the bots show (softlocks, a band
   missed).
5. `cargo shuyi frame-png` pictures of the map and one scene for the PR.

## Acceptance criteria

- [x] New Game reaches Chapter 5 through the chapters before it
      (harness test), and the battle can be won (winning-replay test).
- [ ] The bots land in their bands (`playtest-bots.md`); the report is
      in the PR. **Not met for Casual**: Normal and Hardcore are in band
      on seeds 1–100 and 101–200; Casual is too easy (86 and 80 against
      the hold band of 40–70% Nick set here). Ticket 0526.
- [x] Every scene's place exists and the script passes
      `cargo xtask check-script`.
- [x] Starting values and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the winning replay; New Game → Chapter 5 start; the
  scripted join (in a scene) or join-if-defeated of this chapter's recruit, if any (no recruiting by talking, 0069).
- Content checks: the map, battle and chapter files load and validate.

## Completion notes

**Done, but for Casual's band.** Chapter 5 plays from New Game through
Chapters 1 to 4: after Chapter 4's Hound post, the road south plays and
the Act 1 world map shows; the army walks through the Salt Road and
Saltmere (below) to the Envoy's House, where the house, the sea steps and
the attack play (Hedda joins), then Preparations, the hold, the morning,
the quay, the boat, the Hound post on the north road and "To be
continued" (Chapter 6 doesn't exist yet: 0852 points Chapter 5's `next`
at it).

- **Map** `assets/maps/ch05.map` (28×16, night): the envoy's tall
  narrow house in the middle, the street room behind the street door
  (13, 8), the stair (13, 5) up to the room at the top of the stairs and
  the roof hatch (14, 2) over it; the flat roofs of the terrace round it
  on the land side, with a stair down to the quay at each end; the quay
  and its casks; three piers, the wide one in front of the door the sea
  steps. Picture: `docs/img/ch05-map.png`.
- **Battle** `assets/battles/ch05.ron`: Survive 6 turns, the first hold.
  Every battle scene of `ch05.dlg` wired; two battle notes; music
  `battle_night_siege`.
- **Chapter** `assets/chapters/ch05.ron` on the Act 1 world map; Chapter
  4's `next` is `ch05`; the Envoy's House is `StoryBattle("ch05")`.
- **Units:** Hedda (`shieldbearer`, Raider) and the Hound officer
  (`hound_officer`, Swordsman) moved from `speakers` to `characters`;
  Ragna and the magistrate stay speakers.
- **Places:** `south_road`, `envoys_house` (the house and the morning),
  `saltmere_quay` (the quay and the boat), `north_road_post`; the sea
  steps and the attack use the battle's map.
- **Script:** the `@if` blocks the triggers made one-way are gone (the
  officer against Joss; the sixteen death and retreat lines), and Hedda's
  turn-1 block (she is always there on turn 1, so Hollis's line in its
  `@else` could never play and is gone); `@place` lines and Hedda's
  `@joins` added; the confession's music is `scene_confide`; the header
  says what 0851 wired. No other words changed; `cargo xtask
  check-script`: 0 problems.
- **Music:** `battle_night_siege` (Battle Music, Peter Eastman, the
  source's own loop, 0:04.6 to 1:28.6) and `scene_confide` (Soliloquy,
  Matthew Pablo), both CC-BY 3.0, downloaded with Nick's OK, credited in
  `audio.ron` and `THIRD_PARTY_ASSETS.md`, dropped from `RESERVED`.

**A fault of the script checker, fixed**
(`crates/content/src/dialogue/presence.rs`, with a test that fails
without it): a recruit who joins in a scene was taken as certain in
every scene of the chapter and its battle, because Joss joins after his
chapter's battle. Hedda joins before hers and can fall in it, so she is
now certain only where she can't be gone: the scenes before the battle,
and turn 1 if a slot places her.

**Holds' Casual band** (Nick, 2026-11-08, asked during this ticket): the
Casual bot guards only the lead, who waits the hold out indoors, so it
won 80–90% whatever changed. Nick: "maybe for casual we can aim for
40-70% band which would also necessitate making it harder". Recorded in
`playtest-bots.md` (*Holds have their own Casual band*) and
`crates/bots/targets.ron` (`hold_bands`, used for any story battle or
side quest won by Survive; skirmishes keep theirs). A hold has no
"Hardcore faster" check: it lasts its turns however it is played
(*the agent's reading*).

**Balance** (`cargo xtask playtest ch05 --bot all`; the army Chapters 1
to 4 leave and Hedda: Lead lv 7, Hedda lv 8, Hollis lv 10, Tamsin lv 10,
Aske lv 7, Maud lv 5, Rue lv 14, Joss lv 3):

| Persona | Band | As built (seeds 1–100) | Final, seeds 1–100 | Final, seeds 101–200 |
| ------- | ---- | ---------------------- | ------------------ | -------------------- |
| Casual | 40–70% (hold) | 78 ⚠ | **86 ⚠** | **80 ⚠** |
| Normal | 50–75% | 27 ✗ | **65 ✓** | **70 ✓** |
| Hardcore | 50–75% | 11 ✗ | **62 ✓** | **57 ✓** |

- `cargo shuyi autobalance --stage ch05` (47 evaluations in 40 minutes)
  found only every starting Hound at level 1, which left Casual at 78;
  rejected. By hand: 6 turns instead of 7 and no turn-5 wave; a harder
  officer starting at the far end of the wide pier (at the door on turn
  2); Tamsin's slot upstairs (she rode onto the quay and fell there).
  The variants tried are in `docs/playtesting.md`.
- **Casual is not in its band.** Nothing tried moved it under 78 without
  sinking Normal. Its lead falls only to the officer, and no bot stands
  a unit on the street door to stop him, so his strength moves all three
  bots together. Ticket 0526 (the bots, then a retune).

**Winning replay:** `crates/ui/tests/it/fixtures/ch05_winning.ron`, with
the army Chapters 1 to 4 leave and Hedda: held to the end of turn 6,
nobody lost. Two tests play it: the rules alone, and the real game from
New Game through the world map to "To be continued", Hedda in the army
at the end. Chapter 4's test now runs on into Chapter 5. The bots'
arrival has it for Chapter 6.

**Deviations:**

- **The art is follow-up 0866**, as 0848 to 0850 did: Hedda's and
  Ragna's faces are Nick's to build and the Axemaiden his to download.
  Until then both speak with an empty portrait frame, and Hedda uses the
  Raider's pictures.
- **`next: None`** in `ch05.ron` (the ticket said `Some("ch06")`): a
  chapter that doesn't exist can't be named; 0852 sets it.
- **The Salt Road passes through** (an `Empty` place, no longer
  `NotBuilt`) until 1071 builds it, and Saltmere and the Envoy's House
  open on `Cleared("veyne_hall")` instead of `Cleared("salt_road")`
  (a place to pass through can't be cleared). Noted in 1071.
- **Hedda joins in the attack scene** (`ch05_prebattle`), not on the
  quay: she fights the hold as a player unit (the beat sheet's
  recommendation), and a player slot needs her in the army. On the quay
  Ragna lends her for the road, as written.
- **The checker fix and the hold band** weren't in the ticket by name:
  the first was needed for the script to pass, the second is Nick's
  answer to a question the bots raised.

**Follow-up tickets:** 0866 (Chapter 5's art: Nick builds two faces and
downloads Hedda's picture first), 0526 (the Casual bot wins the hold too
often; no bot holds its door).

**The agent's starting values and rules** (the docs left them open; all
*tunable*, for the Act 1 playtest, 1064):

1. **Hedda:** Raider, level 8, 31 HP, Str 11, Dex 6, Spd 7, Def 8, Res
   2 (talent Def), a Steel Axe and an Iron Axe, Chain Mail. She joins
   the army as Ragna sends her down to the door, and fights the hold.
2. **The Hound officer:** Swordsman, level 9, 40 HP, Str 13, Dex 11,
   Spd 11, Def 8, Res 3, a Steel Sword and an Iron Sword, a Leather
   Vest; the boss, who comes at the party; he need not fall.
3. **Ragna is not on the map** (behind her table upstairs, off it): the
   hold can't be lost through her.
4. **The hold:** Survive 6 turns. 9 Hounds at level 8 to start (four on
   the quay, the officer and one more on the wide pier, three on the
   roofs); two by boat on turn 2, two over the roofs on turn 3, two by
   boat on turn 4. No chests; the win gives 2000 gold; pack cap 3.
5. **The music** is Battle Music's own loop (0:04.6 to 1:28.6): the
   piece's first 4.6 s and last 7.4 s are left out.
6. **Places' light:** the envoy's house at noon for both the afternoon
   visit and the first light after; the sea steps and the attack on the
   battle's map at night.

**Left to judge by playing (1064):** whether the hold feels tight enough
(the bots' Casual wins 80–86%); whether the roof hatch reads as a second
way in (a battle note points at it); whether the officer arriving on
turn 2 feels like "the boss comes with the docks"; Battle Music's loop
point; the four new places' look and light; Hedda joining before the
battle instead of on the quay; the walk from Veyne Hall to the Envoy's
House through the Salt Road and Saltmere until 1071 builds the road's
fight.
