# Playtest bots: player types, targets, autobalancing

Decided: 2026-09-30
Source: ticket 0033

## Nick's words

Original idea (before the ticket):

> "the casual is able to clear (assuming they play on casual mode) with
> probably some fallen units (which will come back since they play on casual
> mode). The normal player probably clears with none but takes a while and
> uses many consumables and the hardcore can clear quickly without many
> consumables used"

Also decided before the ticket: **bots never use rewind.**

Answers:

> **Q1–Q3** "1A 2A 3A but give statistics on how many units died/retreated
> for each epoch and attempt..."
>
> **Q4** "I'm thinking maybe 60% winrate is good enough... no consequences
> for death means it should still be a bit tight for the player... the # of
> deaths does not influence bot at all"
>
> **Q5** "nobody dying in 5/10 with max 3 items used"
>
> **Q6** "nobody dying in 3/10 with max 1 item used"
>
> "also -- we should ensure that we can backwards infer the level of
> difficulty onto a skirmish from these results if at all possible. Meaning, I
> tell you I want a skirmish on grid 10,10 of open world map that's lv35
> scaled difficulty, made up of a mix of units, you can generate one meeting
> that. Or a skirmish on grid 13,13 lv40 scaled difficulty showing up in ch2
> only made up wholly of mages, you can generate one for that."
>
> **Floor or band** "band"
>
> **Hardcore speed** "the point of automated balancing is I don't want to
> manually do this stuff.... so if there is a way to make it so hardcore
> player is able to do it much better then that's ok..."
>
> "also I realized.. I guess hardcore should play better than normal, and we
> don't have a fixed hard mode yet. So I think maybe hardcore should have a
> 50% WR too but it's more for seeing where veterans would end up"
>
> **Q7** "B to start, maybe C later" (targets per map tier; per-battle
> overrides maybe later)
>
> **Q8** "A" (no bot knows about surprise reinforcements)
>
> **Q9** "C - but autobalancing should mean this never happens. I will tell
> you I think the map is empty and I want a skirmish on this part of the grid
> and you will put one that makes sense for the lv I think of."
>
> **Q10** (which army the bots bring to a generated skirmish) "if I tell you
> put a lv35 battle here that means a lv35 battle...?"
>
> **Q11** (what autobalancing may change) "C for random skirmishes, A for
> fixed skirmishes (ie side quests)"

## The three player types

A **try** is one play of a battle from its first turn with fresh luck. A bot
plays each try once, never rewinds and never restarts (Nick, Q3: first try
only).

| Bot | Plays in | What it cares about while playing | A try is a **success** when… |
| --- | -------- | --------------------------------- | ---------------------------- |
| **Casual** | Casual mode | Winning and keeping the lord alive. **Other units falling doesn't matter to it at all** (Nick: "the # of deaths does not influence bot at all"). | it wins |
| **Normal** | Classic mode | Nobody dying. Spends items when that helps; doesn't hurry. | it wins, **nobody dies**, and it used **at most 3 items** |
| **Hardcore** | Classic mode | Nobody dying, few turns, few items. **Plays better than Normal** (a bigger thinking budget). It stands in for veterans, since there is no Hard mode yet. | it wins, **nobody dies**, and it used **at most 1 item** |

"Items" means consumables used by player units (Vulneraries, antidotes, …),
counted per try.

## Targets: a band per map tier

Each bot's **success rate** over many tries (100 by default) must land in a
band. Below the band = too hard (✗); above it = too easy (⚠). Nick picked
bands over floors because a battle "should still be a bit tight" even in
Casual mode.

Bands depend on the map's tier (the same tier that sets rewind charges,
`death-and-difficulty.md`). Nick fixed the **Normal tier** row; the other rows
are *Claude's starting values, tunable*, and per-battle overrides may come
later (Nick: "maybe C later").

| Map tier | Casual | Normal | Hardcore |
| -------- | ------ | ------ | -------- |
| Easy (e.g. random skirmishes) | 45–70% | 65–90% | 65–90% |
| **Normal** | 30–55% | **50**–75% | **50**–75% |
| Hard | 25–50% | 40–65% | 40–65% |
| Finale | 20–45% | 35–60% | 35–60% |

The lower edges on the Normal row for Normal and Hardcore are Nick's (5/10,
and Hardcore raised from 3/10 to 50%). The upper edges (+25 points) are
*Claude's starting values, tunable*.

**Casual's band is the Casual bot's, not a casual player's** (Nick,
2026-10-22, ticket 0506). Nick first set Casual's Normal row at 60–85%.
Once the bots played, the Casual bot turned out a weaker player than
Normal and Hardcore, as designed (it plans less and strays more), and
Nick kept it weaker: "a weaker player would naturally have a lower band
than a hardcore player who thinks more ... probably the band *should* be
low here for a casual bot, not bc we wanna punish noobs but bc if we make
the bot different it's not a casual player and thus have no persona for
one... we just have to read 30% as acceptable or whatever band number you
project we should aim for for a noob". The bot also never rewinds, where a
casual player has the map's rewind charges. So every Casual band is the
first one shifted down 30 points (*Claude's starting values, tunable*;
Nick's earlier row was 60–85 / 75–100 / 55–80 / 50–75). The floor that
blocks a change stays at 20%.

**Skirmishes have their own bands** (Nick, 2026-11-01, ticket 0510). The
first generated lv 35 battle lasted about 3 turns: the Casual bot won 95 of
100 tries while Normal finished with nobody dead in 55 and Hardcore in 70.
Casual only has to win, and in so short a fight making the enemies stronger
only stopped the others keeping everyone alive. Asked what a skirmish should
aim for, Nick picked a Casual band of its own, as in Fire Emblem Sacred
Stones, where a newer player almost always wins a side fight and keeping
everyone alive is the real test, and added: "normal and HC can also be
slightly easier for skirmishes".

| Skirmish tier | Casual | Normal | Hardcore |
| ------------- | ------ | ------ | -------- |
| Easy | 70–100% | 70–95% | 70–95% |
| Normal | 70–100% | 55–80% | 55–80% |

*Claude's starting values, tunable*: Casual's numbers, and "slightly easier"
as 5 points. Every fixed and random skirmish uses this table; a skirmish of
a higher tier keeps the table above, as do story battles and side quests.

**Hardcore is faster than Normal (no par numbers).** Nick doesn't want to set
a turn target per map by hand. The report compares the two bots
automatically: Hardcore's median turns must be **lower than Normal's**,
both over the tries each won with nobody dying (*Claude's reading*, ticket
0506: the engine's check takes one set of tries for both, and "nobody
died and won" is what both success rules share; with Hardcore's whole rule,
≤1 item, Normal, which spends its items, had no tries to compare). This checks that the veteran really plays better; it
is not a per-map par. If Hardcore is not faster, that's flagged as a problem
with the bots, not the map.

## Surprise reinforcements

**No bot knows about them** (Q8 A). Every bot plays like someone seeing the
map for the first time: while planning, it can't see reinforcements that
haven't arrived yet. An unfair ambush therefore shows up in the numbers.

## How Casual treats its units falling

Decided 2026-10-22 (ticket 0506). With falls costing it nothing at all,
the Casual bot spent its units recklessly, they retreated, and its lord
alone couldn't finish Chapter 1. Nick chose, from three options, **"units
are tools"**:

- A fall doesn't upset Casual: no restart, and in Casual mode the unit
  comes back after the battle. Its success is still only "it wins".
- But it doesn't throw units away either, because it needs them to win:
  a fall costs it a little, like losing a fighter, never as much as a
  win. *Claude's starting values, tunable:* a fall costs a tenth of a
  win, and leaving a unit where it could fall costs a third of that.

This refines Nick's earlier "the # of deaths does not influence bot at
all" (Q4): falls still don't count against Casual's success, and they
matter to how it plays only as fighting strength.

## What a bot knows about the enemy

Like a player: the **danger zone** and the **forecasts**, not the enemy's
mind. Nick (2026-10-21, ticket 0506): "they know the danger zone since the
UI makes it apparent, but they shouldn't know exactly who gets attacked
and if it will crit and what arts are used". So while planning, a bot may
see which enemies can reach which tiles and the numbers a forecast shows,
but it never plays the enemy's turn out with the enemy's real AI, and
never sees the luck to come.

Nick's own yardstick (2026-10-21): as an intermediate Fire Emblem player
he clears Chapter 1 as it is with nobody dying, using the danger zone
overlay and his rewinds. Bots never rewind, so they may land a little
lower, but a bot that does far worse is a weak bot, not a hard map.

## What the report shows

Per bot, per battle:

```
ch01 · Normal tier · 100 tries (seeds 1–100)

Casual   · Casual  · won 64/100                            band 60–85%  ✓
           retreats per try  0: 30  1: 21  2: 25  3+: 24   most: Kael 41×
Normal   · Classic · won, nobody died, ≤3 items: 47/100    band 50–75%  ✗ too hard
           deaths per try    0: 55  1: 30  2: 12  3+: 3    most: Mira 22×
           items per try     median 2  max 7
           turns             median 14
Hardcore · Classic · won, nobody died, ≤1 item: 52/100     band 50–75%  ✓
           turns             median 10 (Normal 14)          faster ✓

try 17 · Normal · lost (lord fell T9) · died: Mira T6, Kael T8 · items 2
…
```

- **Every try** is listed: result, who fell (died or retreated) and on which
  turn, items used, turns (Nick: "statistics on how many units
  died/retreated for each … attempt").
- **History across runs** ("each epoch", read as each batch of runs): every
  run of the report is kept, so a change shows as "Normal success 47% → 58%
  after the change". *Claude's reading of "epoch"; Nick may correct it.*

## What a miss does

- **Report only** for ✗ and ⚠ (Q9 C).
- **Blocks the change** only when a battle is basically unwinnable: the
  **Casual bot succeeds in under 20% of tries** (*starting value, tunable*).
- Nick expects this never to happen, because battles are **autobalanced**
  (below).

## Autobalancing

Nick doesn't want to tune battles by hand. A dev tool changes a battle until
all three bots land in their bands for its tier. The bots' own settings are
never changed to make a battle pass; only the battle changes.

What it may change (Q11):

| Battle kind | What autobalancing may change |
| ----------- | ----------------------------- |
| **Random skirmish** | Anything: enemy levels, count, classes, gear, positions, reinforcements, and the terrain. |
| **Fixed skirmish, side quest** | Only the enemies' numbers: level, count, gear, and which classes appear. The map, named units (bosses, characters) and objectives stay as authored. |
| **Story battle** | Same as fixed skirmishes. *Claude's reading: Nick grouped fixed skirmishes with side quests; story battles need their script intact, so they get the same limits. Nick may veto autobalancing story battles at all.* |

## Generating a skirmish to order

Nick tells Claude where and what, in words, for example:

> "put a lv 35 skirmish on grid 10,10 of the world map, mix of units"
>
> "a lv 40 skirmish on grid 13,13, only in Chapter 2, all mages"

Claude generates it with a dev tool and adds it to the game data:

- **"Lv 35" means a lv 35 battle** (Nick): the enemies are around lv 35 (the
  node's level marker shows 35, `world-structure.md`), and the bots play it
  with **a lv 35 army**: the units the player would have recruited by that
  point in the story, at lv 35, with ordinary gear for that point.
- Unit mix, place on the grid, and which chapters it appears in are taken
  from the request.
- The tool autobalances it (random-skirmish rules above) until the three bots
  land in the bands for its tier (skirmishes are Easy or Normal,
  `world-structure.md`).

## Open sub-questions (deferred)

- Per-battle overrides of the bands (Nick: "maybe C later").
- A "first-timer" bot (Q1 B), if ever wanted.
- Whether the bands feel right: compared against Nick's own play in 0508.
