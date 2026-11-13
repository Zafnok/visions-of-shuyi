---
id: "0164"
title: The UI integration tests of the real story run on the private copy, and no public picture draws a real map
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0162"]
nick_input: none
completed: 2026-11-12
---

# 0164 — The UI integration tests of the real story run on the private copy, and no public picture draws a real map

## Context

Split from 0162 before coding (rule 10): see 0163's Context for the
split and ADR-0118's rules ((a) a test of the content keeps the real
content and starts with `if trpg_content::bundle::skip_without_the_game()
{ return; }`; (b) a test of a mechanism moves to `assets-test/`; a public
file names ids only, never a line, a real map or the plot; a picture that
must check the real look goes to the private `content/tests/snapshots/`).
Read 0157's and 0162's completion notes in `tickets/done/`.

This ticket takes **`crates/ui/tests/it/`** and its pictures
(`crates/ui/tests/it/snapshots/`, 131 files on 2026-11-12). From 0162's
inventory:

- `ch01_winnable.rs` … `ch05_winnable.rs`, `side_quests.rs`,
  `world_map.rs`, `ch01_lessons.rs`, `ch01_tips.rs`, `threat.rs`;
- `skirmishes.rs` (but `the_four_places_are_their_fixed_skirmishes`,
  already guarded by 0157; the file's plot comments are this ticket's);
- `battle.rs` (`the_start_tip_waits_for_the_first_turn_scene` needs a
  test battle with a turn-1 scene);
- `playdate.rs` (the `with_shipped_story` tests);
- `title_cinematic.rs` (all but `the_storyboard_plays_shot_by_shot`,
  0157's): its faces, map and logo shots;
- `openings.rs` (7 tests, guarded by 0157 for want of a test opening:
  0162 adds one, so re-point the mechanism tests to it);
- every other test that calls `Harness::with_shipped_story`.

**Picture tests:** every `.snap` that draws a real map, battle or world
map (read each: `flow`, `battle`, `cinematic`, `map_skin`,
`scene_camera`, `map_zoom`, `world_map`, `title`, `controller`, `save*`
and `options` ones start the real story in places). 0157's `cargo xtask
check-leaks` can't see them: a snapshot draws the tiles as the skin
paints them, not the map file's characters.

## Nick input

None.

## Scope

**In:**
- The inventory, in the completion notes, each test marked (a) or (b).
- (a): the guard. (b): pointed at the test files, with test data added
  to `assets-test/` (a test battle with a turn-1 scene, …).
- Each picture test of a real map re-pointed (b), or its `.snap` moved
  to the private repository's `content/tests/snapshots/` (through the
  `content-change` skill, `assert_private_snapshot!`) if it must check
  the real look.
- Comments and test names that tell the plot reworded (ADR-0118 §2).

**Out (do not do):**
- `crates/ui/src/` (0163); bots, xtask, Playdate, benches (0165);
  content and core (0162).
- Moving the content (0160) or the public fallback it builds.
- Changing any real content.

## Implementation steps

1. Write the size estimate first (rules 9 and 10). Past about 2,000
   changed lines (snapshots aside), split again with `write-ticket`
   (the winnable and story-flow tests apart from the picture tests),
   each blocking 0160.
2. The inventory: grep the loaders and real ids; read each picture test.
3. Guard the (a) tests; re-point the (b) tests, adding test data.
4. Move the picture tests that must check the real look; redraw the
   re-pointed ones from the test files.
5. Run the (b) tests with `story_only` where a test of the test files
   can, and say in the notes what 0160 must run.

## Acceptance criteria

- [x] The inventory is in the completion notes, each test marked (a) or (b).
- [x] Every (a) test starts with the guard; every (b) test reads only the
      test files and what 0160 keeps public.
- [x] No tracked `.snap` under `crates/ui/tests/` draws a real map,
      battle or world map.
- [x] `cargo xtask check-leaks` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The re-pointed tests and their picture tests.

## Completion notes

**Estimate:** no production code (tests, test fixtures' comments and
pictures), so no engine-friction ratio. PR size planned at about 1,100
changed lines (about 60 guards, five test files re-pointed, the plot
reworded in eleven files, three tickets); no split needed. Measured:
1,326 changed lines by `cargo xtask pr-size` before this note.

**How the inventory was made.** The grep the Context names, then 0162's
throwaway run before any change: every story file deleted from `assets/`
(chapters, battles, maps, world maps, openings, the title cinematic,
skirmishes, every `.dlg`) and `new_game.ron` on the test chapter, then
`cargo test -p trpg-ui --features harness --test it`. Before: 61 of 460
failed, and one picture (`playdate_battle`) changed. After this ticket:
all 460 pass there, the (a) tests skipping, and every public picture
comes out the same; with the story, all 460 pass too. `assets/` was
restored from git after the run.

**Inventory**: (a) a test of the content, guarded; (b) a test of a
mechanism, on the test files; "none" reads nothing that moves. Every
other file of `crates/ui/tests/it/` is (b) or none and unchanged: it
passed with the story deleted, its pictures the same.

| File | Kind | What changed |
| ---- | ---- | ------------ |
| `ch01_winnable.rs` (2), `ch02_winnable.rs` (2), `ch03_winnable.rs` (2), `ch04_winnable.rs` (3), `ch05_winnable.rs` (3) | (a) | Guard each; the rules-alone tests renamed `the_recorded_playthrough_wins_<battle>` (they named a boss, a place or what happens); comments, messages and the recorded fixtures' headers name ids, never a person, a place or who joins |
| `side_quests.rs` (4) | (a) | Guard; `…_takes_the_barn` → `…_wins_the_company_ledger`, `…_frees_the_boat` → `…_wins_letters_home`; ids in the comments |
| `skirmishes.rs` (5) | (a) | Guard (the first was 0157's); `…_beats_gil_and_he_joins` → `…_wins_millhaven_road_with_its_recruit`, `…_takes_the_cove` → `…_wins_smugglers_cove`, `gil_joins_on_millhaven_road_…` → `millhaven_road_plays_from_the_map_to_the_map_with_its_recruit` |
| `world_map.rs` `act1_*` (3) | (a) | Guard; `act1_start`'s picture (Act 1's world map) moved private |
| `world_map.rs` other 8 | (b) | Unchanged: the test world maps |
| `ch01_lessons.rs` (20) | (a) | Guard: the game's own lessons as its first battle teaches them; the mechanism is tested on the test lesson (`lessons.rs`, `screens/battle/lesson_tests.rs`). The end box's picture (over Chapter 1's map) moved private |
| `ch01_tips.rs` (3) | (a) | Guard; the mage named by id |
| `threat.rs` `chapter_1_names_who_can_attack_…` | (a) | Guard: `turn-structure.md`'s example on Chapter 1 |
| `threat.rs` other | (b) | Unchanged: the Quick Battle |
| `battle.rs` `the_start_tip_waits_for_the_first_turn_scene` | (b) | The test chapter's battle, given a turn-1 trigger in memory (the battle-trigger test scene `test_turn_3`) and no battle notes, as Chapter 1 has |
| `playdate.rs` (10: the D-pad walk-through, New Game to the first scene, Options, the one-bit frames, A + B, the battle screen, the crank, the map menus, the system menu) | (b) | `with_shipped_story` dropped: the test chapter; `the_d_pad_…_into_chapter_ones_first_battle` → `…_into_the_first_battle`. `playdate_battle`'s picture redrawn on `test_small` |
| `playdate.rs` `the_title_cinematic_plays_to_its_menu_on_the_playdate` | (b) | `test_title`, given in memory a talk shot (`test_intro`), a faces shot (the lead and the test knight) and its pan again, so every kind of shot is drawn |
| `playdate.rs` `chapter_ones_scenes_play_…` | (a) | Unchanged (0157's guard) |
| `title_cinematic.rs` (10: left alone, going idle, the load wait, no music, the presses, the skips, the silence, the timer, back from a battle) | (b) | `test_title` as the title's (`with_test_title`, as 0163's unit tests); the frames and the fight's sounds are the test cinematic's |
| `title_cinematic.rs` `the_storyboard_plays_shot_by_shot`, `the_battle_plays_as_turns_with_the_combat_scene` | (a) | `content_tests()` (the guard and the private folder); their 10 public pictures (faces, map, logo) moved private; the characters' names read from the names table, never written |
| `openings.rs` (7) | (b) | Re-pointed to `test_opening` (0162's), given in memory to `test_intro` and to the test scene with a reply choice (`@place test_buildings` and `@opening`, no new line written). The pacing test is gone (the test opening's step back covers facing); `an_opening_after_a_line_waits_for_it` is new. The picture is public again, of the test opening, and its private copy deleted |
| `places.rs` | (a)/(b) | Unchanged (0157's) |
| `demo.rs` (3, from 0914, merged in) | (a) | Unchanged: guarded already; its three pictures (the demo's title menu, end card and export question) draw no map |
| `playthrough.rs` | — | Two comments named a place and a person |

**Pictures.** 120 `.snap` files are left under `crates/ui/tests/it/`. All
came out the same with the story deleted, so none draws a real map,
battle or world map. Moved to the private `content/tests/snapshots/`
(Zafnok/visions-of-shuyi-assets#3), each compared with
the public one byte for byte before deleting it: the title cinematic's
10, Chapter 1's lessons end box, `act1_start`. Redrawn from the test
files: `playdate_battle`, `the_opening_in_glyphs`.

**What 0160 must keep or run.** Besides 0162's, 0163's and 0165's lists:
the test scenes `test_intro`, `test`, `test_victory`, `test_turn_3`, the
opening `test_opening` on `test_buildings`, the cinematic `test_title`
(and the `title` cue's 133.743 s), the test battle and the Quick Battle.
Step 5's `story_only` hides the test files, not the story, so the
deletion run above is what checks the (b) tests: run it again there.

**Deviations**

- **No test data added to `assets-test/`**: the turn-1 scene, the
  openings and the Playdate's talk and faces shots are given in memory to
  the test files that exist (dialogue writing is paused; no `.dlg` line
  was written).
- **`ch01_lessons.rs` stays (a)**, not re-pointed: its 20 tests play the
  game's own lessons on Chapter 1's map, tile by tile; the lessons'
  mechanism already has 28 tests on the test lesson.
- **Found, not done:** comments outside `crates/ui/tests/it/` still name
  the story's people and some tell who dies or joins (62 places, e.g.
  `content/src/dialogue/deaths.rs`). Follow-up **0166**, blocking 0160.
- Two stale pictures with no test (`map_skin`'s
  `quick_battle_under_the_sprite_skin*`) draw the Quick Battle only;
  left as they are.

**Notes for Nick:** nothing to check. No game rule was decided.
