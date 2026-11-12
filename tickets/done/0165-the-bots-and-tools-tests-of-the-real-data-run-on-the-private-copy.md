---
id: "0165"
title: The bots' and tools' tests of the real data run on the private copy
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0162"]
nick_input: none
completed: 2026-11-12
---

# 0165 — The bots' and tools' tests of the real data run on the private copy

## Context

Split from 0162 before coding (rule 10): see 0163's Context for the
split and ADR-0118's rules ((a) a test of the content keeps the real
content and starts with `if trpg_content::bundle::skip_without_the_game()
{ return; }`; (b) a test of a mechanism moves to `assets-test/`; a public
file names ids only, never a line, a real map or the plot). Read 0157's
and 0162's completion notes in `tickets/done/`.

This ticket takes the dev tools and the Playdate crate. From 0162's
inventory (2026-11-12; grep again for `load_embedded`, `story_only`,
`with_shipped_story` and real ids in string literals):

- `crates/bots/`: `army.rs`, `arrival/tests.rs`, `balance/tests.rs`,
  `playtest/tests.rs`, `skirmish/tests.rs`, `testkit.rs`,
  `tests/baseline.rs`;
- `crates/xtask/`: `playtest/tests.rs`, `skirmish/tests.rs`,
  `game_tools/balance/tests.rs`, `game_tools/tests.rs` (the battle half;
  0157 re-pointed the `--scene` half), `battler_import.rs`,
  `tileset_import/tests.rs`;
- `crates/playdate/src/chapter_1.rs` (plays Chapter 1 on the Playdate's
  bundle: a test of the content);
- `bench/*.bench.ron`: real battles by id, in the keyed CI job (0160
  keeps them naming ids only): check each names ids only and say so.

Most bot tests that play a real chapter to a win are tests of the
content (a); those that check how a bot or a tool works (b) can play the
test chapter and battles of `assets-test/`.

## Nick input

None.

## Scope

**In:**
- The inventory, in the completion notes, each test marked (a) or (b).
- (a): the guard. (b): pointed at the test files, with test data added
  to `assets-test/`.
- Comments and test names that tell the plot reworded (ADR-0118 §2).

**Out (do not do):**
- The UI (0163, 0164); content and core (0162).
- Moving the content (0160) or the public fallback it builds.
- Changing any real content, or the bots' recorded playthroughs.

## Implementation steps

1. Write the size estimate first (rules 9 and 10). Past about 2,000
   changed lines, split again with `write-ticket` (bots apart from
   xtask), each blocking 0160.
2. The inventory: grep, then read each test.
3. Guard the (a) tests; re-point the (b) tests, adding test data.
4. Run the (b) tests with `story_only` where a test of the test files
   can, and say in the notes what 0160 must run.

## Acceptance criteria

- [x] The inventory is in the completion notes, each test marked (a) or (b).
- [x] Every (a) test starts with the guard; every (b) test reads only the
      test files and what 0160 keeps public.
- [x] `cargo xtask check-leaks` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The re-pointed tests.

## Completion notes

**Estimate:** no production code (tests only, so no engine-friction
ratio). PR size planned at about 700 changed lines; measured 374 by
`cargo xtask pr-size` before this note. No split needed.

**What was done**

- `crates/bots/src/testkit.rs`: `test_story()`, the test files' story for
  the bots' tests of mechanisms: the content with the test files'
  chapters and battles only, a New Game of `test_wm1` → `test_wm2` →
  `test_wm3` with the lead and the test knight, and an in-memory scene in
  `test_wm1`'s victory in which `test_rogue` joins (no `.dlg` written).
- Each test below guarded (a) or re-pointed (b); comments and names that
  told who joins where, or named the story's people and places, reworded
  to ids.

**Inventory**: (a) a test of the content, guarded; (b) a test of a
mechanism, on the test files; "any" reads whatever content is there and
holds without the story; "none" reads no content (ids as strings only).

| File | Test | Kind | What changed |
| ---- | ---- | ---- | ------------ |
| bots `army.rs` | all 12 (`reference_army_levels_and_roster`, `a_recruit_counts_…`, the growth, spell, promotion, gear, shop, pack, story-chapters and fielding tests) | (b) | On `test_story()`: chapters `test_wm1`/`test_wm2` (shop 4), `test` (shop 1), `quick` (shop 5, steel); the recruit is the test rogue; battle `test` |
| bots `arrival/tests.rs` | `nothing_comes_before_the_first_chapter` (was `…_chapter_1`) | (b) | The test story's first battle |
| | `chapter_2_is_played_with_the_army_chapter_1_leaves`, `the_guards_on_the_way_are_won_first`, `the_army_is_the_same_every_time`, `a_recruit_fights_the_chapters_after_their_join_on_the_way` | (a) | Guard: they play the story's recorded playthroughs; names of people and places gone from comments and messages |
| | `a_battle_no_chapter_fights_has_no_arrival` | (b) | `quick` on the test story |
| | `the_pack_takes_the_stock_first_…`, `a_full_pack_buys_nothing_more` | (b) | Battle `test`, its default pack emptied first |
| | `whoever_a_scene_has_recruited_by_the_chapter_joins` | (b) | The test story's recruit, before and after its scene |
| bots `balance/tests.rs` | `the_enemy_numbers_come_in_order_unnamed_enemies_only` | (b) | Its Chapter 1 half moved out; `test`'s named brigand has no knob |
| | `chapter_1s_named_enemies_have_no_knobs` | (a) | New, guard; the boss's name read from the content |
| | `autobalance_never_touches_named_units_map_or_objectives` | (b) | `test_retreat` (a named boss) in place of `ch01` |
| | other 9 | (b) | Unchanged: `quick`, `test_random_meadow`, own battles on `test_small` |
| bots `playtest/tests.rs` | `a_hold_is_a_battle_won_by_surviving` | (b) | `test_hold` and `test_retreat` |
| | other | (b) | Unchanged: `quick` and made-up stages |
| bots `skirmish/tests.rs` | all | (b) | On `test_story()`: chapters `test_wm2`/`test_wm3`; the property test's slots are the army's roster, not "6 or more" |
| bots `testkit.rs`, `persona/tests.rs`, `bot`, `measures` | all | (b) | Unchanged: the test map and battles (`test_small`, `test_hold`, `quick`) |
| bots `tests/baseline.rs` | `baseline_bot_plays_a_battle_to_the_end` | any | Unchanged: every battle there is |
| | `chapter_1_difficulty` (ignored) | (a) | Guard |
| | `chapter_1_is_won_by_defeating_its_captain_and_the_rest_flee` (was `…_when_harl_falls_…`) | (a) | Guard; the boss named by id |
| | the personas' and rule battles' tests | (b) | Unchanged: test battles |
| xtask `playtest/tests.rs` | `a_battle_with_nothing_before_it_is_played_with_a_fresh_army` (was `chapter_1_is_played_with_the_army_it_always_was`) | (b) | Battle `test` |
| | `chapter_2_is_played_with_the_army_chapter_1_leaves` | (a) | Guard |
| | argument, report and history tests | none | Unchanged: `ch01` as a string in a made-up record |
| xtask `skirmish/tests.rs` | `chapters_are_numbers_or_ids` | (b) | The test files' chapters (`5` → `quick`, `4` → `test_wm1`) |
| | `a_skirmish_end_to_end_…` | any | The army's chapter is the story's last, read from the content, not `ch05` |
| | `without_a_map_it_goes_on_the_story_s_first_…` | (a) | Guard; it alone copies `assets/worldmaps/act1.ron` into its sandbox (the others' sandbox no longer does) |
| | other | (b) | Unchanged: the test world |
| xtask `game_tools/balance/tests.rs` | `the_plan_names_the_battle_…` | (b) | Lists `test_too_hard`, not `ch01` |
| | other | (b) | Unchanged: `test_too_hard` |
| xtask `game_tools/tests.rs` | `a_session_opens_on_the_battle_asked_for`, `the_playdate_session_plays_on_its_console` | (b) | Battle `test` in place of `ch04` |
| | `the_bench_scenarios_play_what_they_say` | (a) | Guard: the benches play the game's battles and scenes |
| | `the_game_opens_on_a_battle_or_a_scene`, `replay_refuses_a_record_that_isnt_one` | none | Unchanged: ids parsed, never loaded |
| | `bots_play_three_games_with_records` (`bot_run`) | any | Unchanged: the New Game's first battle |
| xtask `battler_import.rs` | `the_games_table_names_real_ids_once_each_…` | (a) | Guard: the importer's table names the game's classes and characters |
| | `the_backdrops_file_is_the_public_one_…` | any | Unchanged: terrain and lighting, which stay public |
| xtask `tileset_import/tests.rs` | both that load content | any | Unchanged: terrain (public); classes and characters only as the tileset's references |
| playdate `chapter_1.rs` | `chapter_1_plays_through_on_the_playdates_bundle_…` | (a) | Guard; "the boss's last words" |
| playdate `pictures.rs` | `every_portrait_has_a_one_bit_face_in_each_portrait_style` | (a) | Guard: the Playdate's bundle is Chapter 1's (`playdate-bundle.txt`), and without the game it can't load (found by the run below; not in the ticket's list) |
| `bench/*.bench.ron` | 6 scenarios | (a) | Ids only (`side_company_ledger`, `ch01_intro`, none for the options menu), as 0160 keeps them; comments no longer name a character, a side quest's title or what the narration shows |

**Without the story.** In a throwaway run (restored from git) every
story file was deleted from `assets/` (chapters, battles, maps, world
maps, openings, cinematics, skirmishes, every `.dlg`) and `new_game.ron`
pointed at the test chapter, as 0160's fallback will. Then
`cargo test -p trpg-bots -p xtask -p trpg-playdate`: bots 112 + 4 pass
(1 ignored), Playdate 69 pass, xtask 495 of 496 pass. The one failure is
`check_script`'s `the_writers_guide_examples_have_no_problems`, which
reads `docs/story/` (0159's, as 0157's notes say). The first such run
also failed the bench test, the Playdate portraits test and the xtask
skirmish sandbox, fixed above.

**What 0160 must keep public, or give a counterpart, for these tests of
mechanisms:** besides 0162's list, the basic shop (`basic_shop.ron`:
potions and iron by chapter 2, steel by chapter 5, which the reference
army, pack and arrival tests read), the generic `skirmish_*` templates
in `characters.ron` (both skirmish generators' tests), `test_rogue`'s
character (the test recruit) and the AI values (`ai.ron`, every bot).
0160 runs the throwaway run above again, for every crate.

**Deviations**

- One file not in the list: `crates/playdate/src/pictures.rs`, found by
  the run without the story.
- No test data added to `assets-test/`: the test story is built in
  `testkit.rs` from the test world's three chapters, and its recruit's
  scene is made in memory, so no `.dlg` was written (dialogue writing is
  paused).
- `arrival.rs`'s recorded playthroughs are still the real chapters':
  the tests that play them are (a). Moving them would need a test
  playthrough, and the ticket keeps the recorded playthroughs as they are.

**Notes for Nick:** nothing to check. No game rule was decided.
