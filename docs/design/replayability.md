# Replayability, extra modes, multiplayer and modding

Decided: 2026-10-05 (the direction; the details are open, see below)
Source: Nick, in conversation (a brainstorm on what sets the game apart);
tickets 0046–0053 settle the details

## Nick's words

Claude pitched ideas the repo had no ticket for. Nick on them (2026-10-04):

> I think 3 sounds ok, it's a way to make it a bit roguelike in a small way
> and roguelikes are pretty hot... idk about specifics yet, but if it's
> optional to take a challenge in some way (ie star ratings, with 1 star =
> clear, and the extra stars are challenges) or if we have a card system
> like take a card from a selection of 3 or skip it
>
> and it gives a drawback or condition but an extra reward it might be good
>
> I don't think we can do 4. the nemesis system is copyrighted or something
>
> #5 is ok, if we have a seeded ghost or something that is proven to win
> #6 sure
> #7 sure
> #8 yea let's do it, plan a bunch of dev tools tbh, if we can have a
> scripting DSL or something for easy modding support etc
>
> I don't love 1 and 2 but I don't dismiss them either....
>
> and my idea was somehow having multiplayer but it might be hard to make it
> work in a good way...

(3: optional vows sworn before a battle for a reward. 4: enemy officers who
come back and remember you. 5: the Hardcore bot's turns as "the veteran's
time", and watching it play. 6: the skirmish generator as a post-game mode.
7: shared seeds, replays, daily maps. 8: player-made campaigns and the tools
for them. 1: permadeath that feeds the Door Vow. 2: restraint as a way to
play: routing, surrender, letting a boss flee.)

On the follow-ups (2026-10-05):

> yea stars are a good idea, and if the player wants to replay a map before
> continuing it should be on ending screen
>
> the continue button should pop up a screen saying move on? or something to
> confirm bc player might have lost units, might have used too many
> consumables, might have not gotten 3 stars. even if they got 3 stars they
> might have used more consumables than planned so that option should always
> be there
>
> cards/visions sure... don't necessarily need to be in lore but it could be
> doable... and yea the reward should be something more meaningful than not
> using rewinds but not mandatory to clear the game

On the other roguelike ideas (1: heat for random skirmishes, 2: a run mode
built on the skirmish generator, 3: a randomizer, 4: an ironman option,
5: a daily seeded run):

> 1. sure
> 2. sure, as an extra option on main menu, with its own save slots
> 3. sure
> 4. sure, as another option in the new game (classic / casual / ironman -
> we don't need ironvow naming ironman is universal)
> 5. maybe, but later. im less excited about this one

On multiplayer (1: hotseat, 2: play by link, 3: ghost armies, 4: co-op
skirmishes, 5: live online PvP):

> 1. sure, but need to think about how we set up opposing armies (does it
> use save files, do we just let ppl pick units, is there an army budget,
> who goes first, how are unit placements determined)
> 2. maybe
> 3. yea it's decent, need army budgets though or tiers like pokemon
> showdown to balance
> 4. maybe... but it might just be a nuisance... though I guess it might be
> ok.
> 5. I do want this but I agree... tho doesn't steam provide servers somehow
>
> for modding, I think I agree with everything here...

Then:

> replay - yes the whole attempt is undone
> victory - after continue
> ironman... I guess we need to be consistent... ironman hides everything
> even rewind
> stars - we can have fixed stars on story / fixed skirmish and these won't
> have visions. visions replace stars for random skirmishes as opt in choice
> of 3. as for rewards, TBD, we can make a ticket exploring options but
> since we have the split they should pay out slightly differently but not
> game breakingly so
>
> for multiplayer we'll have a 00 ticket for designing through these options
> and their tradeoffs etc. before implementation details. and this will all
> be much after single-player launch anyway.

## What this decides

None of this is needed for Chapter 1. Each part waits for the Chapter 1
playtest (0804); multiplayer waits for the single-player launch.

### Stars (story battles and fixed skirmishes)

- **Story battles and fixed skirmishes have fixed stars.** One star is a
  clear; the extra stars are challenges. Which challenges, and how many
  stars: ticket **0046**.
- **They never have Visions.**

### Replaying a won map, and "Move on?"

- After a won battle, the end of the battle offers **Replay** next to
  **Continue**, so the player can play the map again before going on.
- **A replay undoes the whole attempt** (Nick): deaths, EXP, items, gold,
  support points, everything that attempt changed.
- **Continue always asks first** ("Move on?" or similar), whatever happened:
  the player may have lost units, used more consumables than they wanted, or
  missed stars, and even a three-star win can have cost too much. The
  question is never skipped.
- **The victory scenes play after Continue**, not before the choice.
- The details (what the question lists, where a replay starts): 0046.

### Visions (random skirmishes)

- **Random skirmishes have Visions instead of stars**: an **opt-in** choice
  of three cards, or skip. A card brings a drawback or condition and an
  extra reward.
- The name may or may not be part of the lore ("don't necessarily need to
  be in lore but it could be doable").
- Which cards, how they're drawn, and the rest: ticket **0047**.
- **Heat for random skirmishes** (Hades' Pact of Punishment: turn the
  difficulty up for better loot) is part of the same question (0047).

### Rewards for stars and Visions

- **More meaningful than the unused-rewind EXP bonus**, but **never
  needed to finish the game**.
- Stars and Visions **pay out slightly differently**, not in a way that
  breaks the game.
- What they pay: ticket **0048**, which explores the options first.

### Ironman

- A **third mode at New Game**: Classic / Casual / **Ironman** (Nick: the
  name is "Ironman", which players know everywhere).
- **Ironman hides everything that undoes a result, rewind included**
  (Nick: "we need to be consistent"). *Claude's reading, to confirm in
  0049:* no rewind, no Replay, no Restart battle and no Retry Battle;
  fallen units die as in Classic.
- The rest (what a defeat does, saving, switching mode): ticket **0049**.

### The run mode

- A **roguelike run mode**: chains of generated, bot-balanced skirmishes,
  drafting recruits and Visions. It's **an extra option on the title
  menu**, with **its own save slots**.
- Everything else: ticket **0050**.

### The randomizer

- A **randomizer** (classes, growths, recruit order and so on, from a
  shareable seed). Yes (Nick). What it changes and where it's offered:
  ticket **0051**.

### Replays, the veteran's ghost and sharing

- **The bots' proven wins become something players see**: Nick is fine with
  it "if we have a seeded ghost or something that is proven to win".
- **Shared seeds and replays:** yes (Nick, "#7 sure").
- How, and where: ticket **0052**.
- **A daily seeded run:** "maybe, but later". Not ticketed.

### Multiplayer

- Nick wants multiplayer. Hotseat (two players on one machine): yes, once
  the army setup is designed. Ghost armies (fight someone's army run by the
  AI): "decent", with army budgets or tiers to balance it. Play by link and
  co-op skirmishes: maybe. Live online PvP: wanted, if it can be done well.
- **First a decision ticket on the options and their trade-offs** (0053),
  then implementation tickets. **All of it after the single-player launch.**

### Modding and dev tools

- **Yes** (Nick: "plan a bunch of dev tools tbh, if we can have a scripting
  DSL or something for easy modding support"; on Claude's outline of a mod
  folder, an in-game map editor, a small scripting language, the bots as a
  balance tester for modders, and a mod policy: "I agree with everything
  here"). How it's built is technical: Claude's tickets and ADRs, no
  decision ticket. Whether mods may use the game's own art is a licensing
  question and stays Nick's call (ADR-0013, `LICENSE`).

## Not taken

- **Enemy officers who come back and remember you** (the Nemesis system):
  ruled out. Warner Bros. holds a US patent on it (granted 2021). Scripted
  recurring villains, which every story game has, are unaffected.
- **Permadeath that feeds the Door Vow** and **restraint as a way to play**
  (routing, surrender, letting a boss flee): Nick doesn't love them but
  doesn't dismiss them either. Not ticketed; the story pipeline may raise
  them again.

## Still open

| Question | Ticket |
| -------- | ------ |
| Stars, Replay and the "Move on?" question | 0046 |
| Visions and heat for random skirmishes | 0047 |
| What stars and Visions pay | 0048 |
| Ironman: defeat, saving, switching mode | 0049 |
| The run mode | 0050 |
| The randomizer | 0051 |
| Replays, the veteran's ghost, sharing a battle | 0052 |
| Multiplayer: modes, armies, fairness, servers | 0053 |
