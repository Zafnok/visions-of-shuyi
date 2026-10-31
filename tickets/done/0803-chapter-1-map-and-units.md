---
id: "0803"
title: Chapter 1 content — map, roster, enemies, triggers, winning-replay test
type: content
milestone: M7 Chapter 1 & game flow
model: reasoning
effort: high
status: done
blocked_by: ["0009", "0016", "0316", "0410", "0412", "0501", "0503", "0706", "0707", "0710", "0714", "0716", "0801", "0807"]
nick_input: sign-off
completed: 2026-10-11
---

# 0803 — Chapter 1 content

## Context

Builds the actual first chapter from `docs/design/chapter-1.md` (0009), the
cast in `docs/story/characters/` (0701), the script (0707) and portraits (0706).
Proves the chapter is winnable with an automated replay.

**Blockers changed 2026-09-29:** 0411 (battle notes) was dropped, since
Chapter 1 has no battle notes (`chapter-1.md`, *Extras*). 0503 (bosses use
Combat Arts and actives) was added, so the boss behaves as designed before the
difficulty sim in step 7 is run and recorded.

**Blockers added 2026-10-01:** 0710 (`@music` in scenes) and 0807 (the
battle file's `music` field). This ticket's scope sets both for Chapter 1,
so both must exist first.

Also 0316 (non-attack skills cost uses per battle; Nick, 2026-10-01: the
playtest waits for it): the winning replay and the difficulty numbers must
be made with the final skill costs.

**Blocker added 2026-10-02:** 0714. The cast is in `characters.ron` as
**speakers** (ids that can talk but aren't units), so the script (0707)
could be checked before this ticket. Step 2 moves the ones who fight into
`characters`.

## Nick input

**Sign-off** happens in the playtest ticket 0804.

## Scope

**In:** `assets/maps/ch01.map`, `assets/chapters/ch01.ron`, real characters
in `characters.ron` (with class/level/stats/loadouts), enemy templates, trigger
wiring, New Game starting at ch01, a winning-replay test, a difficulty sim.
The battle file's `music` cue (0807) and the scenes' `@music` moods (0710),
chosen from the cue list in [`docs/design/audio.md`](../../docs/design/audio.md)
to fit the chapter's beats; ask Nick with the `ask-nick` skill if the beats
don't make it obvious.

**Out:** Chapter 2; balance changes to core formulas (file tickets instead).

## Implementation steps

1. **Map:** design `ch01.map` at the size in `chapter-1.md`, following FE map
   principles: a clear start area, 2–3 lanes/chokepoints, terrain that matters
   (forests for defence, a river with a bridge, a fort), the boss on a
   defensible tile (a fort; `terrain.md` has no gate or throne yet). Sketch it in the PR description.
2. **Roster:** real Chapter 1 characters from the story (replace
   `// PLACEHOLDER` entries; keep test fixtures in `tests/fixtures/` instead of
   shipped data). The five companions and the boss (`retainer`, `sergeant`,
   `poacher`, `keeper`, `heretic`, `red_captain`) are in the file's
   `speakers` list (0714): give each a `characters` entry and take the id
   out of `speakers` (an id in both is an error). Whoever doesn't fight in
   Chapter 1 stays a speaker. Each character's base stats, **talent** stat, starting
   weapon ranks and class records per `progression.md`; generic enemies use
   its generic-unit formula. Classes come from its class tree (which ones is
   `chapter-1.md`'s call: the lord's unique class (0016) + Rider, Archer,
   Cleric, Guard, Mage; tier-3 skills aren't designed yet, ticket 1001).
   Loadouts, starting weapon ranks, the pack cap, default pack and gold per
   `chapter-1.md` and `weapons-and-items.md` (`preparations: false`,
   objective `Rout`, no turn limit, difficulty `Normal`).
3. **Enemies:** count/mix per `chapter-1.md`; AI mix (mostly `Aggressive`, a
   `Guard` group near the boss, boss `Stationary`), sensible loadouts (mix
   weapon kinds so Nick's per-type traits show: a mounted enemy for spears,
   optionally one flyer for bows if the story allows; fliers are tier 3, so
   an early flier is an elite or one of a small number, per the enemy-flier
   rule in `progression.md`, and must stay beatable), one boss with
   a name and portrait. No elementals, reinforcements, villages, chests,
   shops, talk-recruits or battle notes in Chapter 1 (`chapter-1.md`).
4. **Triggers:** wire `ch01_*` scene ids from 0707 (intro, prebattle, boss
   engage, death quotes, victory, tbc; no talk-recruit in Chapter 1). The
   comment at the top of `assets/dialogue/ch01.dlg` lists every id with the
   trigger it is written for: also Hollis's first-turn line, Harl's
   half-HP line, his lines against the lead and against Tamsin, and a
   Casual retreat line next to each companion's Classic death quote. Aske
   counts "twelve men, no, fourteen" in `ch01_prebattle` and the lead may
   answer "Only fourteen?": pick an enemy count that fits.
   Once the companions are in the New Game roster the all-assets test
   checks who may be gone in each scene from these triggers (0715). Then
   delete `chapter_1_never_shows_a_companion_who_may_have_fallen`
   (`crates/content/src/dialogue/tests/presence.rs`, added by 0716): it
   runs the same check on a cast written by hand, and its `certain` map
   says which trigger each scene expects.
5. New Game → `ch01`.
6. **Winning replay test** `crates/ui/tests/it/ch01_winnable.rs` plus a `mod` line in
   `tests/it/main.rs` (or core): a
   hand-authored command list from the chapter seed that wins the map. Keep it
   in a readable `.ron`/text fixture. It must keep passing — if a later balance
   change breaks it, that change must update the replay consciously.
7. **Difficulty sim** (test marked `#[ignore]`, run manually):
   player side controlled by the same AI (Aggressive) over 200 seeds → print
   win rate and average player losses. Record numbers in Completion notes. Target:
   AI-player wins 30–70% (a thinking human should win comfortably; a mindless one
   shouldn't always win). Adjust enemy levels/placement to land in range.

## Acceptance criteria

- [x] Chapter 1 playable from New Game to "To be continued".
- [x] Winning replay test passes.
- [x] Sim numbers recorded and within target (or justified).
- [x] Map sketch + screenshot in the PR.

## Tests required

- Content validation (all-assets), winning replay, sim (ignored).

## Completion notes

**Done.** New Game now starts Chapter 1: `assets/maps/ch01.map` ("Harrowby
Ford", 24×16), `assets/battles/ch01.ron`, `assets/chapters/ch01.ron`, the
six and Harl as real characters in `characters.ron`, five Red Company
templates, every `ch01_*` scene wired to its trigger, and `@music` in the
four chapter scenes. The map sketch and the as-built list are in
`docs/design/chapter-1.md` ("As built"); the screenshot is
`docs/img/ch01-map.png`.

**Asked of Nick (2026-10-11, recorded in `magic.md` and `audio.md`):**

- Do Rue and Maud have their signature spells in Chapter 1? Nick left the
  first pick to the agent ("I can tell you if they're OP"). Both are in.
- Battle music: `battle_bright` (Nick, "A").
- The four scene cues: "I'll have to feel them out". They are in the build
  for the playtest, not approved.

**Difficulty sim** (the baseline bot, a mindless player, 200 seeds,
Classic; `cargo xtask playtest ch01 --runs 200`, or the ignored test
`chapter_1_difficulty` in `crates/bots/tests/baseline.rs`):

| | |
| - | - |
| Won | **74 of 200 (37%)**, inside the 30–70% target |
| Player units lost per try | 3.7 on average (Tamsin in 197 tries, Maud in 191, the lead in 126) |
| Turns | median 7, from 2 to 11 |

The bot charges its rider and its healer at the enemy alone, which is why
it loses so many; it never uses a potion or rewinds. The first layout
(mostly aggressive enemies, as step 3 asked) gave it 11%, lost by turn 3.
Levels were already 1, so the range was reached by **placement and
behaviour**, not by weakening anyone: three enemies come at once and the
rest hold their ground.

**Winning replay:** `crates/ui/tests/it/fixtures/ch01_winning.ron` wins on
turn 8 with nobody lost. Two tests play it: one against the rules alone,
one through the real game from New Game to "To be continued".

**the agent's starting rules** (the design docs were silent; all *tunable*,
for Nick to judge in 0804):

1. **Signature spells.** Rue's Cinder Oath: 9 power, 80 hit, the next tile
   only, 3 uses a battle (Fire is 5 / 90 / 1–2 tiles / 10). Maud's Vigil
   Light: heals 6 from up to 2 tiles, 5 uses (Heal is 10 / next tile / 8).
2. **Levels.** The lead, Aske, Maud and Rue start at level 1, Tamsin at 2,
   Hollis at 3 (he was a guard captain for twenty years).
3. **Stats.** Each of the six has their class's starting stats plus one to
   three points where the character is strong (Hollis is tougher, Tamsin
   faster, Aske more accurate, Rue's magic and Maud's resistance higher).
   The lead's talent is Str, as the lead's sheet says (the data had Spd).
4. **Gear.** Iron weapons for everyone; the lead also has a Steel Sword and
   Hollis a Steel Spear, so there is a second weapon to try. Hollis wears
   Chain Mail, the others a Leather Vest. Tamsin has a spear and a sword.
   Maud has Iron Gauntlets. **Rue has no sword**: she fights with spells
   only.
5. **Harl.** Brigand, level 5, 30 HP, Steel Axe and Iron Axe, on the fort
   tile of the toll-fort, never moving. As the boss he uses Combat Arts
   (Armor Cleave hurts Hollis badly). His sheet said he "uses an active
   skill"; the Brigand class has none, so he uses arts only (sheet fixed).
6. **The Red Company: 13 soldiers and Harl**, so Aske's "fourteen" is
   right. 5 Brigands, 3 Raiders, 3 Archers, 1 Rider (to show what spears do
   to horses, and what his spear does to Tamsin), 1 Guard (to show what
   magic does to armour). All level 1 but the three at the fort, level 2.
   **No flier**: a tier-3 unit would be far too strong here.
7. **Who attacks.** Only three come for the village from the start (two
   Brigands and an Archer). Everyone else holds a bridge, the fort road or
   the fort, and attacks whoever comes within reach. This is not the
   ticket's "mostly Aggressive"; see the sim above.
8. **The map.** Three ways over the river (a footbridge in the west, the
   road bridge, the toll bridge in the east), a walled fort with two ways
   in, and one fort tile in Harrowby next to where the party starts.
9. **Scene music** (not approved, see above): the letter `village_home`,
   the Red Company `dungeon_tense`, after the fight `scene_sad`, Veyne Hall
   `talk_antagonist`. The battle's own scenes don't change the music.

**Deviations from the plan:**

- **The winning replay is recorded, not hand-written.** The ignored test
  `ch01_winnable::record` searches for it (many tries at each turn, going
  back a turn if someone would fall) and writes the fixture. It knows the
  battle's luck, so it shows the chapter can be won, not how hard it is. A
  balance change that breaks the replay re-records it on purpose (the
  test file says how).
- **The test characters, test chapter and Quick Battle are still in the
  shipped data** (step 2 wanted them in `tests/fixtures/`). About 300
  places in the tests and the debug Quick Battle use them. Follow-up:
  ticket **0122**. For now the tests' New Game starts the test chapter
  through `NewGameDef::test_chapter()` (the `Harness` sets it;
  `Harness::with_shipped_story` starts the real story), so the flow and
  save tests don't depend on Chapter 1's content.
- **The sim test asserts the 30–70% range and prints nothing** (the lints
  forbid printing in tests); `cargo xtask playtest ch01 --runs 200` prints
  the numbers above.
- `cargo xtask frame-png` now starts the real story at New Game (it showed
  the test chapter).
- `chapter_1_never_shows_a_companion_who_may_have_fallen` is deleted, as
  step 4 says; `chapter_1s_cast_is_the_new_game_roster_and_its_triggers`
  checks that the real files give each scene the cast the script expects.

**For Nick when playing:** without the bought art the six companions' and
Harl's faces are empty frames (their portraits are in the private
repository, 0706). Numbers are deadly at this scale: a Brigand takes about
half the lead's HP in one hit, and the lead kills a Brigand in one round.
That is the placeholder number scale (ticket 0013), not something this
ticket changed.
