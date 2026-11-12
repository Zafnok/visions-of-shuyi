---
id: "0163"
title: The UI screens' unit tests of the real data run on the private copy
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0162"]
nick_input: none
completed: 2026-11-12
---

# 0163 — The UI screens' unit tests of the real data run on the private copy

## Context

Split from 0162 before coding (rule 10). 0162's inventory of the tests
that read the real data (chapters, battles, maps, world maps, openings,
cinematics, skirmishes, characters, classes, items) found about 70 files
and 1,000 tests in six crates, plus about 140 picture tests: past one
PR. 0162 kept the content and core crates' tests and adds the shared test
data (in `assets-test/`); this ticket takes the **unit tests inside
`crates/ui/src/`**; 0164 the UI's integration tests and their pictures;
0165 the bots, xtask, the Playdate crate and the benches.

The rules are ADR-0118's (read it, and 0157's and 0162's completion
notes, in `tickets/done/`):

- **(a) a test of the content** stays on the real content and starts with
  `if trpg_content::bundle::skip_without_the_game() { return; }`;
- **(b) a test of a mechanism** moves to the test files (`assets-test/`),
  with test data added there;
- a public file may name a content id, never quote a line, draw a real
  map or describe the plot (comments and test names included);
- a picture test that must check the real look keeps its `.snap` in the
  private repository's `content/tests/snapshots/` (through the
  `content-change` skill); `crates/ui/tests/it/places.rs` shows how.

After 0160 moves the content, a checkout without access has only the
public files and the test files, so every test here must either skip (a)
or run on the test files (b). 0160 also moves `classes.ron`, `items.ron`
and the other data tables and builds a public fallback for the test
files; a re-pointed test must not assume more of the real data than the
test files keep.

The files, from 0162's inventory (2026-11-12; grep again for the loaders
`load_embedded`, `story_only`, `Harness::with_shipped_story` and real ids
in string literals: `ch01`…`ch08`, `act1…`, the battle, map, opening,
character, class and item ids of `assets/`):

- `map_view/playdate/tests.rs` (`harrowby_ford()`), `map_view/place.rs`;
- `cinematic/opening/tests.rs` (`ch01_sparring`, `ch01_pacing`);
- `screens/dialogue/tests.rs` (opening `ch01_sparring`);
- `screens/world_map/tests.rs`;
- `screens/battle/{doable,magic_tests,lesson_tests,lessons,art_tests,threat_tests,stage/tests}.rs`;
- `screens/preparations/tests.rs`;
- `audio.rs`, `tips.rs`;
- any other test under `crates/ui/src/` the grep finds, and the `.snap`
  files under `crates/ui/src/**/snapshots/` that draw a real map, battle
  or world map (read each).

0162 adds a test opening and a test world-map place with a named enemy
in `assets-test/`; use them, and add what else a re-pointed test needs.

## Nick input

None.

## Scope

**In:**
- The inventory of the files above, in the completion notes, each test
  marked (a) or (b).
- (a): the guard. (b): pointed at the test files, with test data added
  to `assets-test/`.
- Each picture test under `crates/ui/src/` of a real map re-pointed (b),
  or its `.snap` moved private if it must check the real look.
- Comments and test names that tell the plot reworded to say what the
  test checks (ADR-0118 §2).

**Out (do not do):**
- `crates/ui/tests/it/` (0164); bots, xtask, Playdate, benches (0165);
  content and core (0162).
- Moving the content (0160) or the public fallback it builds.
- Changing any real content.

## Implementation steps

1. Write the size estimate first (rules 9 and 10). Past about 2,000
   changed lines (snapshots aside), split again with `write-ticket`
   (screens/battle on its own), each blocking 0160.
2. The inventory: grep, then read each test.
3. Guard the (a) tests.
4. Re-point the (b) tests, adding test data.
5. Move or redraw the picture tests of real maps.
6. Run the (b) tests with `story_only` where a test of the test files
   can, and say in the notes what 0160 must run.

## Acceptance criteria

- [x] The inventory is in the completion notes, each test marked (a) or (b).
- [x] Every (a) test starts with the guard; every (b) test reads only the
      test files and what 0160 keeps public.
- [x] No tracked `.snap` under `crates/ui/src/` draws a real map, battle
      or world map.
- [x] `cargo xtask check-leaks` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The re-pointed tests and their picture tests.

## Completion notes

**Estimate:** no production code (tests, test data and comments), so no
engine-friction ratio. PR size planned at about 800 changed lines (about
45 tests); no split needed.

**How the inventory was made.** The grep the Context names, then 0162's
throwaway run: every story file deleted from `assets/` (chapters, battles,
maps, world maps, openings, the title cinematic, skirmishes, every `.dlg`)
and `new_game.ron` on the test chapter, then `cargo test -p trpg-ui
--lib`. Before: 43 of 1,812 failed. After this ticket: all 1,812 pass,
the guarded ones skipping (then `assets/` was restored from git). The
data tables (classes, items, spells, names, tips, audio) stayed for that
run, as in 0162's; the tests below that read them are listed too.

**Inventory**: (a) a test of the content, guarded; (b) a test of a
mechanism, on the test files; "none" reads no content (a content id as a
plain string at most); "any" reads whatever is there and holds without
the story.

| File | Kind | What changed |
| ---- | ---- | ------------ |
| `cinematic/opening/tests.rs` (4 tests) | (b) | On the test opening `test_opening`; `sparring_partners_…` is `a_step_back_keeps_facing_the_other_way`; the walker test adds a step up in the test, so the way it last went isn't the camera's |
| `cinematic/tests.rs` the title cinematic's walks, fights and shot times (3) | (a) | Guard |
| `cinematic/shot_tests.rs` `a_face_has_the_short_name_…` | (b) | The test speaker `test_voice`, its short form set in the test |
| `map_view/place.rs` (2) | (b) | The map `test_small` and the battle `test` |
| `map_view/playdate/tests.rs` `harrowby_fords_opening_view` | (b) | `the_quick_battles_opening_view`: the Quick Battle; the picture of the real map deleted, the test battle's drawn |
| `screens/dialogue/tests.rs` `a_speaker_who_is_not_a_unit_…` | (b) | `test_voice` |
| `screens/dialogue/tests.rs` `the_view_shows_the_place_…`, `a_jump_in_the_log_…`, `the_keys_after_…_an_opening_wait`, `fading_in_needs_a_place_…` | (b) | `test_small`, the battle `test`, `test_buildings` with `test_opening` |
| `screens/dialogue/glyph/tests.rs` (3 place tests) | (b) | `test_small`; a checked cell moved onto the smaller map |
| `screens/dialogue/playdate/tests.rs` (3 place tests) | (b) | `test_small` |
| `dialogue/tests.rs` `no_embedded_scene_shows_someone_who_is_gone` | (a) | Guard (it plays every scene of the game for the real cast); a plot comment gone |
| `dialogue/tests.rs` `the_place_lasts_until_the_next_place_line` | none | Test ids in place of `ch01` |
| `screens/battle/doable.rs` (3) | (b) | The Quick Battle (three units with arts in reach, a mage with a tile spell and none); `chapter_1_turn_1_…` is `turn_1_can_do_…` |
| `screens/battle/magic_tests.rs` `cinder_oath_shows_no_counter_in_chapter_1` | (b) | `a_spell_the_defender_cant_counter_shows_no_counter`: the Quick Battle's mage, its Fire given `no_counter` in the test, against the plain Fire's counter |
| `screens/battle/art_tests.rs` `opening_the_unit_menu_searches_the_move_once` | (b) | The Quick Battle's archer (no skills, like the bench battle's rider) |
| `screens/battle/threat_tests.rs` (2 path-search tests) | (b) | The Quick Battle; the bench files still time the biggest battle (0165's) |
| `screens/title/tests.rs` (15), `title/playdate/tests.rs` (2) | (b) | `with_test_title`: the new test cinematic `test_title` (timed to the `title` song, its fight at 69.95 s and logo at 115.8 s, on the battle `test`) as the title's |
| `screens/world_map/tests.rs` `the_cursors_keys_…_reach_every_place` | split | (b) the test maps; (a) new `act_1s_world_map_lets_the_cursor_reach_every_place`, guard |
| `audio.rs` `a_pool_pick_…`, the clock, `every_track_in_a_pool_…` | (b) | A made-up manifest of three tracks |
| `audio.rs` `the_victory_pool_is_thirteen_stings_…` | (a) | Guard |
| `tips.rs` `the_map_pick_tips_name_the_select_key` | (a) | Guard |
| `tips.rs` (2 more), `input.rs`, `map_view.rs`, `map_view/glyph/units.rs`, `portrait/tests.rs`, `screen.rs`, `screens/battle/notes.rs` | any | Unchanged: the keymap, font, palette or test files only |
| `screens/battle/lesson_tests.rs`, `lessons.rs` | none | `another_battle` in place of an unread `ch01` |
| `screens/battle/stage/tests.rs` picture test | none | Its made-up battler ids are test ids; character comments neutral |
| `screens/preparations/tests.rs`, `screens/battle/act1_tests.rs` | (b) | Unchanged: already the Quick Battle |
| `flow.rs`, `flow/tests.rs`, `screens/battle/flight.rs` | — | Comments that named who dies, joins or falls, reworded |

**Pictures.** Of the 112 `.snap` files under `crates/ui/src/`, only the
Playdate's Harrowby Ford drew a real map; it is redrawn on the Quick
Battle. The other 111 came out the same with the story's files deleted,
so none draws a real map, battle or world map.

**What 0160 must keep or run.** The tests of mechanisms here read, of the
data that moves: the classes `exile`, `guard`, `archer`, `mage`, the spell
`fire`, the `title` music cue (its 133.743 s: `test_title` and the title's
tests time to it), the `lead` character and its name. Run `cargo test -p
trpg-ui --lib` without the checkout once the files have moved, as above.
Step 6's `story_only` hides the test files, not the story, so the
deletion run is what checks the (b) tests here.

**Deviations**

- **A new test cinematic** (`assets-test/cinematics/test_title.ron`): the
  title screen's 17 tests play the title's cinematic in time with its
  song, which the test files had nothing like.
- **Plot comments outside tests** (`flow.rs`, `flight.rs`) were reworded
  too: they are under `crates/ui/src/` and named who dies.
- **Seen, not changed:** a unit whose menu has a Skill entry runs 3 path
  searches when its menu opens, one without runs 1 (the art test above).
  It may be the skills' own reach (a move after acting); not looked into.

**Notes for Nick:** nothing to check. No game rule was decided.
