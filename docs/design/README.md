# Game design decisions

Nick decides how the game plays. Each `00xx` ticket asks him (using the
`ask-nick` skill: options from real games + "describe your own"), and records
the answer here as a concrete, implementable rule set. Implementation tickets
treat these files as the source of truth, and never invent answers that aren't
here.

| Topic | File | Ticket | Status |
| ----- | ---- | ------ | ------ |
| Stats & combat maths | [`stats-and-combat.md`](stats-and-combat.md) | 0001 | ✅ decided 2026-09-25 |
| Turn structure | [`turn-structure.md`](turn-structure.md) | 0002 | ✅ decided 2026-09-25 |
| Act 1's battle rules: escape, save the village, bosses who retreat or can't be killed, Hollis's death, side-quest rewards, recruiting, what each chapter introduces | [`turn-structure.md`](turn-structure.md) (*Act 1's battle rules*), `death-and-difficulty.md`, `world-structure.md` (*Side quest rewards*), `battle-scenes-and-recruitment.md`, `docs/story/outline.md` | 0069 | ✅ decided 2026-10-22 (everyone escapes one by one, no turn limit; Dace retreats at the killing blow (Ch 4) and is downed but stays (Ch 8); Hollis dies in the closing scene in every mode; rewards: personal weapon, signature upgrade, rare stock or gold, no stat boosters, can be missed (with a deadline reminder); no talk-recruit; villages only as a story race) |
| Weapons, gear, items, shops | [`weapons-and-items.md`](weapons-and-items.md) | 0003, 0408 (Preparations) | ✅ decided 2026-09-25 (Preparations: bring only your own items, retry returns there, 2026-10-01) |
| The Preparations shop: basic items and tiers | [`weapons-and-items.md`](weapons-and-items.md) | 0044 | ✅ decided 2026-10-21 (tiers by chapter, consumables + Iron/Steel gear + repairs, Potion 20 / Hi-Potion 40, rare Elixir; green units carry their own items) |
| Four armour weights (cloth, leather, chain, plate) and who wears them | [`weapons-and-items.md`](weapons-and-items.md) | 0073 | ✅ decided 2026-10-21 (four weights by material; each class has a max weight and wears anything lighter; the lord never wears plate; four starter pieces from Chapter 2) |
| The Steel armour sold from Chapter 5 | `weapons-and-items.md` | 0074 | ⏳ asked next (0073: Iron Plate from Ch 2, a Steel Plate later) |
| Combat Arts (and active-skill costs) | [`combat-arts.md`](combat-arts.md) | 0014 | ✅ decided 2026-09-25 (non-attack actives cost uses per battle, 2026-10-01; arts in their own menu, 2026-10-12, ticket 0442) |
| Combat Arts for ranks C–S, special weapons | `combat-arts.md` | 0018 | ⏳ after playtest 0804 |
| Magic & healing | [`magic.md`](magic.md) | 0004 | ✅ decided 2026-09-25 (spells after a class change and terrain magic details revised 2026-09-27; casting on the battle screen added 2026-10-01) |
| Level ups, classes, class tree | [`progression.md`](progression.md) | 0005, 0017, 0019 | ✅ decided 2026-09-25 (fliers moved to tier 3+ by 0017; no per-class stat caps, 0019, 2026-09-28; seals only between battles, 0603, 2026-09-30) |
| Death, rewind, difficulty, saving | [`death-and-difficulty.md`](death-and-difficulty.md) | 0006 | ✅ decided 2026-09-25 |
| Setting, tone & the lead | [`setting-and-tone.md`](setting-and-tone.md) (+ [`docs/story/beats.md`](../story/beats.md)) | 0007 | ✅ decided 2026-09-25 |
| World structure, saving on the world map, camp | [`world-structure.md`](world-structure.md) | 0008 | ✅ decided 2026-09-26 |
| Chapter 1 scope | [`chapter-1.md`](chapter-1.md) | 0009 | ✅ decided 2026-09-25 |
| The lord's unique class line | [`progression.md`](progression.md) (*The lord's line*) | 0016 | ✅ decided 2026-09-25 |
| Supports & relationships | [`supports.md`](supports.md) | 0010 | ✅ decided 2026-09-25 |
| Look & feel | [`look-and-feel.md`](look-and-feel.md) | 0011, 0404 (forecast), 0021 (bought art), 0038 (bought tiles and sprites on the battle map), 0039 (sprite units and dialogue busts) | ✅ decided 2026-09-25 (forecast 2026-09-27; bought portraits and battle art 2026-09-30; art bought, and the battle map drawn with it, per-map lighting and a zoom key 2026-10-02; walking sprites, effect arrows, the glyph look as an option and busts at 4× in dialogue 2026-10-02) |
| Fighters with no Tiny Tales art | [`look-and-feel.md`](look-and-feel.md) (*Fighters with no Tiny Tales art*) | 0040, 0035 (Harl) | ✅ decided 2026-10-04 (Shironejiya for foot fighters, Pixel Flag's Paladin for riders) |
| Sprite units: the zoom key (the outline alone, no corner mark: decided 2026-10-03, 0436) | `look-and-feel.md` (*Zoom*), `controls.md` (*Zoom map*) | 0035, 0439 | ✅ decided 2026-10-12 (starts at 2×; `Q` / `P`, right trigger, optional) |
| Music & sound effects | [`audio.md`](audio.md) | 0020 | ✅ decided 2026-09-28 (banter track, plain-spell crit, fliers and later places still open) |
| Title screen: the logo, its backdrops and the sparks | [`look-and-feel.md`](look-and-feel.md) (*Title screen*) | 0811 | ✅ decided 2026-10-12 (block lettering over a bought backdrop that changes each launch, drifting sparks, no line under the title) |
| Game Over screen: the opening animation | [`look-and-feel.md`](look-and-feel.md) (*Game Over screen*) | 0830 | ✅ decided 2026-10-13 (the battle fades to black, the gray logo then gold `GAME OVER` under it, two seconds, any press skips) |
| Descriptions in lists: what the highlighted skill or art does | [`look-and-feel.md`](look-and-feel.md) (*Descriptions in lists*) | 0429 | ✅ decided 2026-10-14 (a row under the list inside the menu box, white text, the cursor stops on greyed lines) |
| Places behind conversations | [`look-and-feel.md`](look-and-feel.md) (*Places behind conversations*) | 0029 (built by 0732, 0733, 0734) | ✅ decided 2026-10-15 (the talk over its place, darkened; a short moving opening where the script asks; the battle map for talk around a battle) |
| Title | [`title.md`](title.md) | 0012 | ✅ decided 2026-09-29 (*Visions of Shuyi*) |
| Title screen: "Press any key or button" | [`title-screen.md`](title-screen.md) | 0034, 0032, 0226 | ✅ decided 2026-09-30 (every build, keys and buttons, since 0032; the keyboard and controller pictures 2026-10-04) |
| Title screen: intro cinematic | [`title-screen.md`](title-screen.md) (*Intro cinematic*) | 0036 | ✅ decided 2026-10-03 (an off switch in Options: later; skip sound 0041; overworld shot 1010) |
| Pointing at a hurt ally to heal it | [`turn-structure.md`](turn-structure.md) (*Pointing at an ally*), [`magic.md`](magic.md) | 0451 | ✅ decided 2026-10-14 (closest tile any heal reaches, then a list of the heals) |
| Threat arrows: which enemies can attack the tile you're moving to | [`turn-structure.md`](turn-structure.md) (*Threat arrows*), [`look-and-feel.md`](look-and-feel.md) (*Threat arrows*) | 0055 (built by 0448) | ✅ decided 2026-10-12 |
| Arrows at the map's edges where the map goes on past the view | [`look-and-feel.md`](look-and-feel.md) (*Arrows at the map's edges*) | 0449 | ✅ decided 2026-10-21 (a big arrow fading slowly, only while browsing; cursor yellow in the glyph look) |
| Tutorial: how familiar you are, the guide, checklist lessons, Guides | [`tutorial.md`](tutorial.md) (+ `chapter-1.md`, *Teaching*) | 0057 (built by 0455, 0456, 0457, 0458, 0731, 0835) | ✅ decided 2026-10-14 |
| Terrain: movement costs & bonuses | [`terrain.md`](terrain.md) | 0301, 0037 | ✅ decided 2026-09-26 (house, gate and throne tiles, capturing, healing tiles: 2026-10-22, ticket 0037) |
| Controls, key layouts & controller | [`controls.md`](controls.md) | 0015, 0030, 0032 | ✅ decided 2026-09-25 (rebinding keys 0030, 2026-09-29; controller 0032, 2026-09-30) |
| The Options screen: speeds, volumes, asking first; where options are kept (player profiles) | [`options.md`](options.md) | 0805, 0058 (built by 0836, 0837, 0838, 0839) | ✅ decided 2026-10-04 (player profiles hold the options and saves, 2026-10-14) |
| Battle scenes, talking & recruitment | [`battle-scenes-and-recruitment.md`](battle-scenes-and-recruitment.md) | 0705 | ✅ decided 2026-09-29 (quest recruitment later) |
| Playtest bots: player types, targets, autobalancing | [`playtest-bots.md`](playtest-bots.md) | 0033, 0506 | ✅ decided 2026-09-30; what bots know of the enemy and Casual's falls 2026-10-21/22 |
| AI voices, Japanese and Chinese options, a hired writer later | [`voices-languages-and-script.md`](voices-languages-and-script.md) | 0042 (Japanese), 0043 (voices), 0045 (Chinese) | ⏳ direction set 2026-10-03 (Chinese added 2026-10-04); details open |
| The Playdate version of Chapter 1: screen size, layout, no colour, buttons and crank, what it keeps, where it is sold | [`playdate.md`](playdate.md) | 0061, 0062, 0066 (built by 1025's tickets) | ✅ decided 2026-10-15 (today's 8×16 cells, FE-GBA corner pop-ups, your units black and enemies grey, A+B ends turn and the crank rewinds, everything kept with one build per language, itch.io first then Catalog; one-bit pictures: Bayer-dot faces at 2× with a style toggle, fighters on white ground under a dithered horizon, card = dithered backdrop + logo, icon = the S; the guide's box: a side window away from the cursor, 1× face, lesson text and checklist at once) |
| Number scale, stat ceilings, strike rule & level cap | [`stats-and-combat.md`](stats-and-combat.md) | 0013 (built by 0323, 0608) | ✅ decided 2026-10-22 (FE-sized; fixed ceilings HP 200 / stats 100; extra strikes need a gap or a share of the enemy's speed, whichever is bigger; level cap 99 for Act 1, about 1 level per battle) |
| Replayability: stars and Replay, Visions, rewards, Ironman, a run mode, a randomizer, replays and sharing, multiplayer, modding | [`replayability.md`](replayability.md) | 0046–0054 | ⏳ direction set 2026-10-05; details after playtest 0804 (multiplayer after launch) |

Each file starts with `Decided: YYYY-MM-DD`, `Source: ticket NNNN`, and Nick's
words verbatim, followed by the derived rules (numbers marked *tunable* where
Claude chose a starting value).
