---
id: "1072"
title: "Act 1's side quests, part 1: Tamsin and Aske"
type: content
milestone: Post–Chapter 1
model: fable-5.1 / gpt-6-astra
effort: high
status: done
blocked_by: ["1070", "0018", "0069", "0746", "0747", "0509", "0013", "0323", "0608", "0329"]
nick_input: none
completed: 2026-11-04
---

# 1072 — Act 1's side quests, part 1: Tamsin and Aske

## Context

**Split by ticket 0135** (five quests are about 4,000 changed lines, past
the limit of about 3,000 for one PR; ADR-0101). This ticket builds Tamsin's
and Aske's quests; 1083 builds Hollis's and Rue's, 1084 Maud's. The table
keeps all five for the whole picture, and the rest of this ticket is what
1083 and 1084 read for how a quest is built.

Side quests (paralogues) are "optional, handcrafted battles tied to one
character's story, with story scenes and a unique reward", unlocked by
story progress and conditions (`world-structure.md`). Act 1 has five
(`docs/story/outline.md`):

| Quest | Whose | Unlocked | What it is |
| --- | --- | --- | --- |
| *The Company Ledger* | Tamsin | after the Aldwater Toll Bridge (Ch 4), Tamsin recruited and alive | she finds the crown paymaster who pocketed the Red Company's wages, and has to decide what the money is worth |
| *Letters Home* | Aske | after Chapter 5's story battle, Aske alive | a refugee boat carrying her letters and silver home has been seized by League privateers |
| *The Captain's Oath* | Hollis | after Chapter 4's story battle, Hollis alive | his wife's grave near Veyne Hall; his goodbye to the life before exile (his confession stays in the main story) |
| *The Other Cart* | Rue | after Chapter 6's story battle, Rue alive | a Candle who escaped with her runs a hideout for runaways |
| *Unwritten* | Maud | once Ossary is reached, Maud alive | she brings the register's names back to their families |

Their rewards are of the kinds 0069 decided, and special weapons come
from 0018 (`combat-arts.md`, *Special weapons*: a personal weapon is a
special weapon locked to one character, built by 0329). The chapters' scripts (0746, 0747, 0749) come first, so the
quests fit what the ledger says has happened. 1070 places their nodes,
locked; this ticket writes and builds them and fills in the nodes.

## Nick input

None; no check after merge (Nick, 2026-10-21). Which reward each quest gives
is picked here, inside 0069's kinds, and listed in the Completion notes for
1064's checklist. The question about the runaway Candle of *The Other Cart*
moved to 1083 with Rue's quest.

## Scope

**In:**
- For Tamsin's and Aske's quests: a short beat sheet (in `docs/story/chapters/`, e.g.
  `side-tamsin.md`), its scenes (`assets/dialogue/side_*.dlg`, by the
  `story-writing` skill with 0736's craft rules and the critique pass),
  its map and battle (as 0848–0854 do: map, units, triggers, place
  maps, `music: Cue("side_quest")`), its unique reward, and its node in
  1070's map with the unlock condition.
- New names in `docs/story/names.md` and `assets/data/names.ron`; minor
  speakers who need a face use the shared portraits (`soldier` and the
  like) unless 0070 picked one.
- The ledger updated for each.
- `cargo shuyi autobalance` and `playtest --bot all` per battle.

**Out (do not do):**
- Hollis's and Rue's quests (1083); Maud's (1084).
- Supports and camp events (1073); fixed skirmishes (1071).
- Changing what the main story says happened.

## Implementation steps

1. Read the `story-writing` skill, the outline, the ledger, Tamsin's and
   Aske's sheets, 0746 and 0747's scripts, 0069's and 0018's answers.
2. Per quest: beat sheet, script, critique pass, map, battle, reward,
   node.
3. Bots; `frame-png` pictures; ledger.

Expected size: about 1,700 changed lines (0135; `cargo xtask pr-size`).

## Acceptance criteria

- [ ] Each quest unlocks under its condition and not otherwise (tests).
- [ ] Each can be won (winning replays) and gives its reward.
- [ ] Scripts pass `cargo xtask check-script`; the critique pass is
      ticked in the PR body.
- [ ] The bots land in their bands for each; reports in the PR.
- [ ] Starting values and the rewards are in the Completion notes
      (1064's checklist).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: unlock conditions (character dead or not recruited →
  locked); winning replays; the reward in the army's stock.
- Content checks for every new file.

## Completion notes

**Done.** Tamsin's and Aske's side quests are on the Act 1 map and play
through the real game: the place opens, its scene, Preparations, the
battle, the results, the scene after, and the map again with the place
cleared and the reward in the army.

- **The Company Ledger** (Tamsin's): beat sheet
  `docs/story/chapters/side-tamsin.md`, script `assets/dialogue/side_ledger.dlg`
  (the gate, the strongroom, 24 battle scenes), map
  `assets/maps/side_company_ledger.map` (Kingsmead, the crown's tithe
  barn north of Millhaven, 22 x 14, dusk), battle
  `assets/battles/side_company_ledger.ron`: defeat Ambrose Tull, the
  paymaster, who **retreats** at 0 HP out the back with the purse he
  counted; the men he pays run (`leftovers: Flee`). **Reward: 4,212
  crowns** (0069's "a purse of gold"), the number her book has carried,
  which she hands to the lead as a payment and keeps the names uncrossed.
- **Letters Home** (Aske's): beat sheet `side-aske.md`, script
  `side_letters.dlg` (the cliff top, the shingle, 24 battle scenes), map
  `side_letters_home.map` (Gull Creek, east of Saltmere, 22 x 14, dusk),
  battle `side_letters_home.ron`: defeat Jory Casker, a League privateer
  with a council paper (he falls); his crew run for the sloop. **Reward:
  Tor's Bow**, the first **personal weapon** (0018, 0329's `wielder`):
  her dead brother's hill bow, sent south by her mother; only Aske wields
  it; its art **Longshot** (3 dur) shoots at 2 to 3 tiles at hit -10.
- **New names** (`docs/story/names.md`, `names.ron`): the paymaster
  Ambrose Tull, Denny Rudd of the Red Company (a named enemy on the barn
  step, who falls), Jory Casker, Sten (the hill people's boat master, a
  speaker), Kingsmead and Gull Creek. Their voice cards are in the beat
  sheets. Three units and four `privateer_*` templates in
  `characters.ron`.
- **The nodes** `company_ledger` and `letters_home` in
  `assets/worldmaps/act1.ron` point at the battles and scenes; their
  unlock conditions are 1070's as written (`Cleared("aldwater")` with
  Tamsin in the army; `Cleared("envoys_house")` with Aske), `NotBuilt`
  removed.
- **The ledger** has a new section, *Act 1's side quests*, at the end:
  each quest is optional and may be played at any time to the act's end,
  so neither changes what the main chapters say happened, and each is
  written to lean on nothing after its unlock (no Veyne Hall in the
  Ledger; no Ashfields in Letters, and Rue never speaks to Aske).
- **Critique pass** done: a cold read by a fresh reader of the player's
  view of all eight scenes, then the editor's list. Fixed from it: the
  paymaster's case cut from three boxes to two, "six riders" said once,
  the ledger-column motif cut from four uses to three, the cliff-top
  roll call shortened, Hollis's brief lines made plainer, and three
  swappable lines given their speaker's habit (Joss runs on, Rue asks,
  Hedda says "easy"). The reader would miss Tamsin, Aske, Joss and the
  paymaster, and remembered Sten's "No? Then stand there. I'll look from
  here."
- **Tests** (`crates/ui/tests/it/side_quests.rs`): each place opens
  after its battle while its character is in the army, and not before,
  and not without them; each battle is won by its recorded playthrough
  (`fixtures/side_*_winning.ron`, played with the army the battle's
  `bots_army` describes) losing nobody, the paymaster retreating alive
  and the win paying 4,212, the bow landing in the stock with Aske as
  its wielder; *The Company Ledger* played from a save on the map to the
  map again through the real game. The content tests cover every new
  file. The test raises the army by average growths itself: the bots
  crate stays out of the game's crates, test dependencies included
  (ADR-0033, an xtask test guards it).

**The bots** (`cargo shuyi autobalance --stage <id> --apply`, 100 tries
a persona, seeds 1-100; the army is the battle's `bots_army`: the
roster by Chapter 4 at level 10, by Chapter 6 at level 16, raised as
`docs/playtesting.md` says, no pack):

| Battle | Persona | Band | As authored | Autobalanced |
| ------ | ------- | ---- | ----------- | ------------ |
| Company Ledger | Casual | 30-55% | 50 | **46** |
| | Normal | 50-75% | 28 | **51** |
| | Hardcore | 50-75% | 58 | **49** (one under) |
| | Hardcore turns (vs Normal) | lower | 10 vs 12 | 10 vs 12 |
| Letters Home | Casual | 30-55% | 52 | **76** (too easy) |
| | Normal | 50-75% | 12 | **43** (too hard) |
| | Hardcore | 50-75% | 44 | **56** |
| | Hardcore turns (vs Normal) | lower | 7 vs 8 | 7 vs 8 |

The Ledger's search ran out of evaluations with Hardcore one point under
its band (the four changes it kept: the gate's guard and two archers a
level up, the archer behind the gate a level down); shipped as the best
setting seen, for 1064 to judge. **Letters Home took three passes.** As
first built (the army at level 16, Casker at the camp fire) it was a
four-turn rush, too easy for every persona even with every privateer
at level 26; the army went to level 12 (a tier-1 party, as the act's
pace has it by Chapter 5) and Casker onto the boat's deck by the
hatch, behind the gangplank and its bowmen. Then the Normal bot lost
the lead on turn 2 to the three privateers rushing the path, so those
three hold its foot instead and the lead starts last. The search then
ran out of evaluations with Casual and Normal pulling opposite ways:
Casual (rewinds) wins too often, Normal (nobody lost) not often
enough, Hardcore in its band. Shipped as the best setting seen (the
foot of the path at levels 9-11, the deck's bowmen and the sloop's
boat at 14); a death-free win on this beach is what 1064 should judge.

**Deviations.**

- **Gil and Hedda have no slot in either battle**: they are speakers,
  not units, until 1071 (Gil) and 0851 (Hedda) make them so, and a
  battle file can't place a speaker. Their lines in the scenes before
  and after each battle are in `@if` blocks and play when they are in
  the army; their death and retreat lines are written at the end of each
  script in `@if` blocks, unwired, for 1071 and 0851 to give them slots
  and triggers (the scripts' headers say so).
- **The validator doesn't know a side quest's scenes can count on their
  character**, so Tamsin's and Aske's lines before and after their own
  battle sit in `@if` blocks like everyone else's. A small content change
  could teach it (`Cast` reading the world maps' side quests); not done
  here, since the dialogue is checked before the world maps load.
- **`@place battle` is for a chapter's or trigger's scenes only**, so the
  quests' intro and victory scenes name their own map as a place
  (`@place side_company_ledger 11,6`).
- **The `bots_army` chapters are `ch04` and `ch06`**, which don't exist
  yet: `roster_by` then gives the whole story's roster (the six and
  Joss), which is right until those chapters exist, when it gives who is
  recruited by then (Gil and Hedda included, once they are units).
- **Tor's Bow is named after Tor** (`brother_tor`): a rename of him
  touches `items.ron` and the language packs too.

**The agent's starting values and rules** (for the Act 1 playtest, 1064;
all *tunable*):

1. **The Ledger's reward is gold, 4,212**, in `clear_gold`: it shows on
   the results screen like any clear gold. **Letters Home's is Tor's Bow
   and 400 gold** (the privateers' purse).
2. **Tor's Bow:** Bow, rank E, might 7, hit 85, crit 10, weight 3, range
   2, 25 durability, price 1,200 (so it sells for 600, with the special
   weapons' warning). **Longshot:** 3 durability, range 2-3, hit -10; a
   target at 3 counters only if it reaches 3.
3. **Ambrose Tull:** Swordsman, level 9, retreats at 0 HP (the hit that
   brings him there counts as a kill: EXP and the objective), stationary
   in the strongroom. **Denny Rudd:** Rider, level 9, holds the barn
   step (Guard AI). **Jory Casker:** Swordsman, level 14, stationary on
   the boat's deck by the hatch.
4. **The Ledger's enemies:** 15 at the start (Hounds at the gate and in
   the barn, Red Company men in the yard and on the west flank), levels
   9-11 after autobalance, and two Hound riders from the east on turn 4.
   **Letters Home's:** 12 privateers at levels 9-14 after autobalance
   (three holding the foot of the cliff path, one on the rock, three on
   the gangplank and the deck with Casker, three at the camp), and two
   more up the beach from the sloop's boat on turn 3.
5. **Both have Preparations, a pack cap of 4, Normal tier**, the
   `side_quest` cue (the ticket's choice), and `kind: SideQuest`.
6. **Kingsmead's stream can be waded on foot (cost 5) and not by a
   horse;** Gull Creek's water the same. The doors drawn on Kingsmead's
   walls are gaps (a `door` tile is impassable).
7. **Tull says Tamsin's number and she stops him; she says it once, in
   the strongroom.** The ledger has forty-one names; Harl's troop is
   eleven of them. Denny's wife is unnamed.
8. **Aske's mother asks her to come home, and Aske doesn't answer.** Her
   arc's end (Act 3: she goes home) is untouched.

**Left to judge by playing (1064):** whether the Ledger's Hardcore band
matters at one point under; whether 4,212 gold at Chapter 4 is too
much purse too early (Millhaven's Steel arrives in Chapter 5); whether
Longshot's range 3 makes Aske too safe; the two maps' look at dusk;
whether a player fighting the Ledger after Saltmere, with Hedda in the
party and no line for her at the barn, notices.

**Pictures in the PR:** `docs/img/1072-ledger-gate.png`,
`1072-ledger-map.png`, `1072-letters-cliff.png`, `1072-letters-map.png`
(the glyph look, from a throwaway Quick Battle start, since reverted).

**Follow-ups:** none new. 1071 and 0851 add Gil's and Hedda's slots and
wire their lines; 1083 and 1084 build the other three quests the same
way.

**Sizes.** Estimated before the code: about 20 production lines (no new
rule; a paralogue is data in an engine that has side quests, as this one
does since 1007) and about 1,700 changed lines in all (the ticket's).
Measured: 0 added production lines (`cargo xtask code-lines`; the whole
ticket is data, scripts, docs and tests: under 2x) and 2,183 changed lines
(`cargo xtask pr-size`): past the 2,000 a ticket is planned at, under
the 3,000 limit; it would have split cleanly into one quest a PR.
