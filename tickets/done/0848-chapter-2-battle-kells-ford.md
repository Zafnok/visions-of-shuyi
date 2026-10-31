---
id: "0848"
title: "Chapter 2 battle: Kell's Ford, map, units, places and balance"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0744", "0509", "0013", "0070", "0071", "0313", "0409", "0443", "0069", "0323", "0608", "0329", "0076"]
nick_input: none
completed: 2026-10-29
---

# 0848 — Chapter 2 battle: Kell's Ford

## Context

Act 1, Chapter 2, from `docs/story/outline.md`:

- **Story battle:** Kell's Ford: a river village split by a ford, houses to save before the raiders burn them (0069's *save the village*: a minimum to save, extra rewards past it), the garrison's watchtower across the water, a Hound lieutenant as boss.
- **Objective:** defeat the boss (the outline: "the chapter ticket decides"; a Defeat-unit objective exists) and save at least a minimum of the houses (`turn-structure.md`, *Save the village*; the minimum, the houses and the extra rewards are *the agent's starting values* for the bots to tune).
- **Recruits:** Joss Pellam (`defector`, Swordsman), joins in the scene after the battle (0069: no recruiting by talking).
- **Named boss (0743):** Lieutenant Silas Hendry (`hound_lieutenant`), who picked Joss for the company; sheet `docs/story/characters/hound_lieutenant.md`.
- **New in this chapter:** Preparations (0443's shop tab) for the first time, save the village (0313), a recruit who joins in a scene (0069).

0803 built Chapter 1 the same way (map, units, triggers, a winning
replay, a difficulty run); this ticket does it for Chapter 2, from its
beat sheet and script (0744). The balance comes from the bots: 0509's
`cargo shuyi autobalance` tunes the enemies until the persona bots land
in their bands (`docs/design/playtest-bots.md`), and 0013 has settled
the number scale first. It also needs the save-the-village rule (0313), the shop screen (0409), the Preparations shop (0443), and 0069's confirmation of what Chapter 2 introduces.

The new faces and combat pictures are 0070's picks; the scene and battle
music is 0071's.
0070 picked **Joss**: the Thief hero's face, bust and map sprite, and Shironejiya's `m_s_hero07` at 1.25× recoloured (brown hair, dark headband, red cloth turned Hound black). The **Hound lieutenant** (Silas Hendry) is 0076's pick: Stupid Creations' **TTMini: Bandit Brigand** (<https://stupidcreations.itch.io/ttmini-bandit-brigand>, $2.99, Nick buys it; AI-assisted, so the Steam page discloses it, 0903) in his three-quarter pose, and a generator face to match (shaggy red hair, full red beard), since he speaks; his sheet was rewritten to fit (`hound_lieutenant.md`). Generator faces are built by Nick in the Character Generator from a parts list this ticket gives him (Nick, 0070: the agent's pasted generator mockups aren't reliable); Shironejiya pictures come from <http://hi79.web.fc2.com/material/monster_s/> and carry that site's credit text (`look-and-feel.md`, *Fighters with no Tiny Tales art*).

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
- **The two special weapons** (0018, `combat-arts.md`, *Special weapons*):
  the Hound lieutenant is an axe user carrying the **Houndmaster's Axe**
  (he uses Pack Hunt; it is the player's when the battle is won), and the
  **Fordwarden's Spear** is one of the rewards for saving houses past the
  minimum. Both items come from 0329.
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
- **Balance:** `cargo shuyi autobalance --stage ch02 --apply`, then
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
3. Import the new art; wire the places into the script (the `@music`
   lines are already 0744's picks). When importing `battle_desperate`,
   remove it from `RESERVED` in `crates/content/src/audio.rs`'s tests
   (its "drop it from RESERVED" assertion fails otherwise; 0252). 0744
   wrote the script before this battle existed, so every companion line
   in a battle-trigger scene of `ch02.dlg` sits in an `@if` block for
   them; once a trigger names that companion (a death quote, a retreat
   line, the boss against Aske, Maud at a house) the validator reports
   the block as one that can only go one way: remove it. The `@if` in
   `ch02_first_turn` stays (a turn-start scene after Chapter 1 can't
   count on anyone). `defector`, `hound_lieutenant` and `hound_sergeant`
   are in `characters.ron`'s `speakers` for now; move the first two into
   `characters` here (the sergeant is 0849's). How Joss is on the map is
   open: the `ch02_refusal` scene is a turn-start scene drawn over the
   map, and the player never fights him (`docs/story/chapters/ch02.md`).
4. Autobalance; playtest; fix what the bots show (softlocks, a band
   missed).
5. `cargo shuyi frame-png` pictures of the map and one scene for the PR.

## Acceptance criteria

- [x] New Game reaches Chapter 2 through the chapters before it
      (harness test), and the battle can be won (winning-replay test).
- [x] The bots land in their bands (`playtest-bots.md`); the report is
      in the PR. (Seeds 1–100, as autobalance judges them; seeds 101–200
      put Hardcore 3 points over: see the Completion notes.)
- [x] Every scene's place exists and the script passes
      `cargo xtask check-script`.
- [x] Starting values and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the winning replay; New Game → Chapter 2 start; the
  scripted join (in a scene) or join-if-defeated of this chapter's recruit, if any (no recruiting by talking, 0069).
- Content checks: the map, battle and chapter files load and validate.

## Completion notes

**Done.** Chapter 2 plays from New Game through Chapter 1: the two
intro scenes, the first Preparations, the battle at Kell's Ford, the
herald, Joss joining on the far bank, Veyne Hall, then "To be
continued" (Chapter 3 doesn't exist yet: 0849 points Chapter 2's `next`
at it, as its ticket already says).

- **Map** `assets/maps/ch02.map` (24×16, dawn): the river across the
  middle, the ford as a road through it at x 9, the crown's bridge at
  x 19 with the walled watchtower beside it on the north bank, five
  houses (three south, two north), two already burnt. Picture:
  `docs/img/ch02-map.png`.
- **Battle** `assets/battles/ch02.ron`: defeat Hendry (the rest flee)
  and save at least 3 of 5 houses; Preparations; every scene id of
  `ch02.dlg` wired; music `battle_desperate`.
- **Chapter** `assets/chapters/ch02.ron`; Chapter 1's `next` is `ch02`.
- **Units:** Joss (`defector`) and Hendry (`hound_lieutenant`) moved
  from `speakers` to `characters`; five Hound templates.
- **Places:** `kells_road` (the halt), `kells_far_bank` (the join),
  `veyne_guardroom` (to be continued); the ford's edge and the herald
  use the battle's own map (`@place battle`).
- **Script:** the `@if` blocks the triggers made one-way are gone (the
  boss against Aske, Maud at a house, the ten death and retreat lines);
  the `@place` lines and `@joins defector` added. No words changed.
- **Music:** `battle_desperate` imported (Determined Pursuit by Emma_MA,
  CC0, downloaded from OpenGameArt with the user's OK, converted to OGG
  Vorbis 44.1 kHz) with its credit, and taken out of `RESERVED`.

**Two small mechanisms the chapter needed** (the design had decided
both; nothing built them yet):

- **`@joins <character>`**, a scene's recruit
  (`battle-scenes-and-recruitment.md`, *In a scene, on a story beat*):
  mirrors `@dies` (same rules; the join happens as the scene starts;
  `Campaign::joins_in_scene`). The script check treats a scene recruit
  as someone the army can lose from the next chapter on, and not in the
  chapter that recruits them (`Cast::with_joins`). Chapters 3–5's
  scripts already keep Joss in `@if` blocks, and pass.
- **`clear_items`** in a battle file, items for winning into the stock,
  beside `clear_gold`: how the Houndmaster's Axe "is the player's when
  the battle is won". The results screen doesn't list it (it doesn't
  list village rewards either).

**Balance** (`cargo shuyi autobalance --stage ch02 --apply`, then
`cargo xtask playtest ch02 --bot all`):

| Persona | Band | As first built | After the layout changes | Autobalanced, seeds 1–100 | Seeds 101–200 |
| ------- | ---- | -------------- | ------------------------ | ------------------------- | ------------- |
| Casual | 30–55% | 25 ✗ | 43 ✓ | **46 ✓** | 32 ✓ |
| Normal | 50–75% | 8 ✗ | 66 ✓ | **55 ✓** | 50 ✓ |
| Hardcore | 50–75% | 7 ✗ | 80 ⚠ | **75 ✓** | 78 ⚠ |
| Hardcore turns (vs Normal) | lower | 5 vs 4 ✗ | 6 vs 5 ✗ | 6 vs 4 ✗ | 6 vs 5 ✗ |

- As first built the bots lost the village, not the lead: they saved
  the two near houses, the south-east one burned on the Hounds' second
  turn and both north ones by their fifth. Lower levels didn't help
  (every Hound at level 1: 26 / 12 / 4 over 50 tries). Two layout
  changes did: the swordsman beside the west house moved to the north
  bank (Aske died there on turns 1–2), and only one of the two
  north-east Hounds is a raider, so the north-west house burns later.
- Autobalance's one change: every unnamed Hound a level down (2 → 1).
  As in Chapter 1 (0855), a lower enemy level made it *harder* for the
  bots (less EXP in the battle).
- "Hardcore faster than Normal" fails on this map: follow-up 0520 looks
  at it (saving houses may cost the careful bot turns).
- **The bots play with a weaker army than a player brings** (fresh
  character data, and no pack, since Preparations leaves the default
  pack empty). A player arrives about two levels up with Chapter 1's
  Potions, so Chapter 2 should play easier than these numbers.
  Follow-up 0520 gives the bots the army the chapters before leave, and
  re-tunes.

**Winning replay:** `crates/ui/tests/it/fixtures/ch02_winning.ron`, with
the army Chapter 1's recorded playthrough leaves: won on turn 3, nobody
lost, all 5 houses saved. Two tests play it: the rules alone, and the
real game from New Game through Chapter 1, Preparations, the battle and
Joss's join to "To be continued" (Joss in the roster, the axe in the
stock). The recorder moved from `ch01_winnable.rs` into a shared
`playthrough.rs` (it now also scores houses saved and burned);
Chapter 1's fixture didn't change. The Playdate's Chapter 1 test now
ends at Chapter 2's first scene.

**Deviations:**

- **The art is follow-up 0858**, not done here. Hendry's picture
  (TTMini: Bandit Brigand) isn't bought yet and his face is Nick's to
  build in the generator (0858 has the parts list); Joss's combat
  picture needs Shironejiya's `m_s_hero07`, whose site answered a script
  with only a placeholder page. One art PR through the private
  repository instead of two. Until then both speak with empty portrait
  frames and use their class's map sprite and combat picture.
- **`next: None`** in `ch02.ron` (the ticket said `Some("ch03")`): a
  chapter that doesn't exist can't be named; 0849 sets it.
- The Playdate ships only Chapter 1's music (`playdate-music.txt`), so
  Chapter 2 is silent there until a Playdate ticket adds its tracks.

**Follow-up tickets:** 0858 (Chapter 2's art; Nick buys Hendry's
picture and builds his face first), 0520 (the bots play a chapter with
the army the chapters before it leave; Chapter 2 re-tuned; 0849–0854
now wait for it).

**the agent's starting values and rules** (the docs left them open; all
*tunable*, for the Act 1 playtest, 1064):

1. **Joss is no unit on the map.** His refusal plays over the map as
   the Hounds' first turn starts (turn 1, enemy phase); the player
   never meets him in the battle.
2. **Five houses, save at least 3.** The 4th house standing gives the
   **Fordwarden's Spear**, the 5th an **Elixir**.
3. **The win gives 1200 gold and the Houndmaster's Axe** (a fresh one,
   not Hendry's worn copy). The other Hounds flee when he falls, and
   every house still standing then counts as saved.
4. **Pack cap 4** (Chapter 1: 3). The debug Quick Battle and the bots
   start with 3 Potions and 1000 gold, what Chapter 1 leaves.
5. **Hendry:** Raider, level 6, 30 HP, Str 10, Def 7, **Res 1**, the
   Houndmaster's Axe only, Chain Mail; he never moves from the ford's
   north end, with a guard either side (so Pack Hunt hits whoever comes
   up out of the water at +4).
6. **Joss:** Swordsman, level 3, talent Dex, 21 HP, Str 6, Dex 9, Spd 9,
   Def 4, Res 2; Iron Sword and Leather Vest.
7. **The Hounds:** 14, all level 1 after autobalance: 4 raiders who burn
   houses (two by the south houses, one on the bridge road, one from
   the north-east), 3 swordsmen, 2 archers, 2 guards with Hendry,
   1 rider, and 1 raider-class soldier holding the north-east.
8. **Maud's house line** plays at the house by the ford only (a trigger
   for each house would play it again at each).
9. **The race as built:** the two near houses burn on the Hounds' first
   turn unless evacuated or their raider killed; the south-east one on
   their second (Tamsin's job, as her first-turn line says); the big
   house by the water on their third; the north-west one later.

**Left to judge by playing (1064):** whether Hendry falls too fast to
magic (Res 1: the recorded playthrough has Rue finish him with an
Overcast Fire on turn 3, so his boss lines may barely play); whether the
race feels fair on a first try; whether 1200 gold and a pack of 4 suit
the first Preparations; the three new places' look and light; the map
at dawn.

Engine friction: estimate 140, measured 121 (0.9×), under 2×.
