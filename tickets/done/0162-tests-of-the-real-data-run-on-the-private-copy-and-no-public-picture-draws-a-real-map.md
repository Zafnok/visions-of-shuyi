---
id: "0162"
title: Tests of the real data in the content and core crates run on the private copy
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0157"]
nick_input: none
completed: 2026-11-12
---

# 0162 — Tests of the real data in the content and core crates run on the private copy

## Context

Split from 0157 before coding (rule 10): its inventory found about 120
code files and a dozen picture tests that read the real content, past one
PR. 0157 kept the **scripts' tests** (the ones that read the real `.dlg`
scenes and quote their lines) and set the rules; this ticket takes the
**data's**: tests that read the real chapters, battles, maps, world maps,
openings, cinematics, skirmishes, characters, classes and items.

0157 decided, in ADR-0118:

- **(a) a test of the content** stays on the real content and starts with
  `if trpg_content::bundle::skip_without_the_game() { return; }`: it
  skips without the checkout and fails in CI if the content is missing;
- **(b) a test of a mechanism** moves to the test files (`assets-test/`),
  with test data added there;
- a public file may name a content id, never quote a line, draw a real
  map or describe the plot;
- the content's own test data (a spec, a snapshot that must check the
  real look) goes in the private repository's `content/tests/`, which no
  build embeds; insta's snapshot path points there
  (`crates/ui/tests/it/places.rs` shows how).

After 0160 moves the content, a checkout without access has only the
public files and the test files, so every test here must either skip (a)
or run on the test files (b).

**Split again before coding (2026-11-12, rule 10).** The inventory
found about 70 files and 1,000 tests in six crates, plus about 140
picture tests. This ticket keeps the **content and core crates** and
adds the shared test data the later pieces use (a test opening, a test
world-map place with a named enemy); **0163** takes the unit tests in
`crates/ui/src/`, **0164** `crates/ui/tests/it/` and its pictures,
**0165** the bots, xtask, the Playdate crate and the benches. All four
block 0160; 0163–0165 are blocked by this one.

The files to sort (grep again, ids named in string literals):

- `crates/content/`: `chapter/tests.rs`, `battle/tests.rs`,
  `worldmap/tests.rs`, `cinematic/tests.rs`, `dialogue/tests/openings.rs`,
  `lesson/tests.rs`, `names/tests.rs` (the table, not `names.md`, which
  0159 handles), `lang/data_tests.rs`, `skirmish/tests.rs`, `map.rs`,
  `character.rs`, `tileset.rs`, `support.rs`, `trigger.rs`,
  `dialogue/tests/presence.rs` (`a_places_intro_counts_on_its_battles_named_enemies`,
  guarded by 0157 for want of a test place with a named enemy),
  `tests/it/{ai_soak,arts,promotion,skills,supports}.rs`;
- `crates/core/`: `camp/tests.rs`, `campaign/tests.rs`,
  `worldmap/tests.rs`, `lead/tests.rs`, `save/tests.rs`,
  `tests/it/save_format.rs` (ids only: check each needs no content).

The test characters (`assets-test/data/characters.ron`) use the real
classes and items (`exile`, `iron_sword`); after 0160 moves `classes.ron`
and `items.ron`, the test files need their own or a public base. That is
0160's public fallback, but this ticket's re-pointed tests must not
assume more of the real data than the test files keep.

## Nick input

None.

## Scope

**In:**
- The inventory of the files above, in the completion notes, each test
  marked (a) or (b).
- (a): the guard (ADR-0118 §1). (b): pointed at the test files, with test
  data added to `assets-test/` (a test opening, a test world map's
  places, …).
- The split (0163–0165), 0160's `blocked_by` and the roadmap.
- Comments and test names that tell the plot ("Joss joins in the scene
  after") reworded to say what the test checks (ADR-0118 §2).

**Out (do not do):**
- The scripts' tests, the guard itself, the ADR and the leak check
  (0157).
- The UI's, bots', tools' and Playdate's tests and every picture test
  (0163–0165).
- Moving the content (0160) or the public fallback it builds.
- Changing any real content.

## Implementation steps

1. Write the size estimate first (rules 9 and 10). If the inventory is
   past about 2,000 changed lines (snapshots aside), split again with
   `write-ticket` by crate (the UI tests on their own), each blocking 0160.
2. The inventory: grep the loaders (`load_embedded`, `with_shipped_story`,
   `story_only`) and the real ids; read each picture test of a real map.
3. Guard the (a) tests.
4. Re-point the (b) tests, adding test data.
5. Simulate the checkout's absence: the guard's tests and a run with
   `assets-private/content/` holding a stand-in `chapters/` cannot show it
   before 0160, so run the (b) tests with `story_only` where a test of the
   test files can, and say in the notes what 0160 must run.

## Acceptance criteria

- [x] The inventory is in the completion notes, each test marked (a) or (b).
- [x] Every (a) test starts with the guard; every (b) test reads only the
      test files and what 0160 keeps public (the data tables 0160 gives a
      public base are listed below and in 0160).
- [x] `cargo xtask check-leaks` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The re-pointed tests.

## Completion notes

**Split again first (rule 10).** The inventory found about 70 files and
1,000 tests in six crates, plus about 140 picture tests: far past one PR.
This ticket kept the content and core crates; **0163** (the UI's unit
tests in `crates/ui/src/`), **0164** (`crates/ui/tests/it/` and its
pictures, so "no public picture draws a real map" is 0164's now; the
title changed to say what this ticket does) and **0165** (bots, xtask, the
Playdate crate, the benches) are new, blocked by this one and blocking
0160. 0160's `blocked_by`, its Context and the roadmap name them.

**Estimate:** no production code (tests and test data only, so no
engine-friction ratio). PR size planned at about 900 changed lines (about
100 guards of 3 lines, the re-pointed tests, four tickets); measured
1,120 by `cargo xtask pr-size` before this note.

**Inventory**: (a) a test of the content, guarded; (b) a test of a
mechanism, on the test files; "any" reads whatever content is there and
holds without the story; "none" reads no content.

| File | Kind | What changed |
| ---- | ---- | ------------ |
| content `chapter/tests.rs` `the_test_chapters_and_new_game_load` (was `embedded_…`) | (b) | Split: the test chapters and test New Game |
| content `chapter/tests.rs` `the_games_chapters_and_new_game_load` | (a) | New, guard; the real chapters' wiring; plot comments gone |
| content `chapter/tests.rs` `a_new_campaign_starts_as_the_new_game_file_says` | (b) | The test New Game (lead and test knight), not Chapter 1's six |
| content `chapter/tests.rs` other 6 | (b)/any | Unchanged: test ids; `every_battle_starts_undecided` any |
| content `battle/tests.rs` `chapter_1_is_won_by_defeating_its_captain_and_the_rest_flee` | (a) | Guard; renamed, the boss named by id |
| content `battle/tests.rs` other 30 | (b) | Unchanged: test battles and maps (`bots_army`'s `"ch02"` is an unchecked string) |
| content `worldmap/tests.rs` `mod act1` (5 tests) | (a) | Guard each |
| content `worldmap/tests.rs` `fixed_skirmishes_validate_…` | (b) | A test battle given a theme in place of Chapter 1's (`problems_with`) |
| content `worldmap/tests.rs` `a_chapter_condition_must_name_a_chapter` | (b) | `test_wm2` for `ch02` |
| content `worldmap/tests.rs` other 14 | (b) | Unchanged |
| content `cinematic/tests.rs` `the_title_cinematic_is_as_long_as_its_song_…` | (a) | Guard; takes the "a build that ships has the title's alone" check from the test cinematic's test |
| content `cinematic/tests.rs` other 10 | (b) | Unchanged (the `title` music cue: 0160's public audio) |
| content `cinematic/tests/openings.rs` `the_test_opening_loads` | (b) | New test opening `assets-test/openings/test_opening.ron` on `test_buildings`, in place of `chapter_1_openings_load` |
| content `cinematic/tests/openings.rs` `chapter_1s_openings_are_on_its_places` | (a) | New, guard: ids only |
| content `cinematic/tests/openings.rs` the 4 check tests | (b) | On `test_buildings`, with `test_voice` (a new test speaker) for a classless walker |
| content `cinematic/tests/actions.rs` `units_are_those_on_the_battles_map_at_its_start` | (b) | A character enemy of `test_retreat` in place of Chapter 1's captain |
| content `cinematic/tests/actions.rs` `chapter_1_names_the_units_the_title_cinematic_moves` | (a) | Split out of `the_battles_name_their_enemies_by_unit_id`, guard |
| content `cinematic/tests/{actions,combat}.rs` other 19 | (b) | Unchanged |
| content `dialogue/tests/presence.rs` `a_places_intro_counts_on_its_battles_named_enemies` | (b) | A place on `test_retreat` (its rogue is a character); the test world's side quest listed |
| content `presence.rs` `the_cast_comes_from_…`, `every_kind_of_trigger_…` | (b) | `embedded()` keeps the test files' battles and chapters only; the story's recruit and its plot comment gone |
| content `presence.rs` `chapter_1s_cast_…` | (a) | Unchanged (0157's guard) |
| content `dialogue/tests/openings.rs` `chapter_1_opens_its_two_scenes_…` | (a) | Unchanged (0157's guard) |
| content `character.rs` `the_games_cast_is_its_characters_and_speakers` | (a) | Split out of `embedded_characters_load_and_make_units`, guard |
| content `character.rs` `embedded_characters_load_and_make_units` | (b)/any | Every loaded character and template is armed; the lords; the test mage's spells; the test speaker |
| content `character.rs` `the_embedded_table_has_the_test_entries_…` | any | The test speaker counted |
| content `class.rs` the 3 `embedded_*_match(es)_design_doc` | (a) | Guard |
| content `item/tests.rs` the 3 `*_match_the_design`, `seal_checks_…`, `w1`–`w4` | (a) | Guard |
| content `skill/tests.rs` the 3 design-table tests, `every_class_skill_exists_…` | (a) | Guard |
| content `art/tests.rs` the 6 design-table tests, `only_special_weapons_have_arts` | (a) | Guard |
| content `ai.rs` `embedded_file_holds_the_starting_values` | (a) | Guard |
| content `audio.rs` `the_playdates_music_is_chapter_1s` | (a) | Guard |
| content `lesson/tests.rs` `the_story_has_the_ten_lessons_of_the_design` | (a) | Guard |
| content `lesson/tests.rs` other 11 | (b)/any | Unchanged |
| content `lang/data_tests.rs` `the_games_data_offers_its_text` | (a) | Split out: skills, arts, the story's names, tips |
| content `lang/data_tests.rs` `the_embedded_content_offers_every_kind_of_data_text` | (b) | The kinds the test files use (a test name for the story's) |
| content `lang/data_tests.rs` other 22 | (b) | Unchanged (the test pack names real skills: 0160's base) |
| content `names/tests.rs`, `support.rs`, `tileset.rs`, `trigger.rs`, `skirmish/tests.rs`, `camp/tests.rs` | (b)/any | Unchanged but the names count (+ the test speaker) |
| content `map.rs` `every_map_round_trips` | any | No longer asks for `ch01` |
| content `bundle.rs` `story_only_hides_the_test_files_…` | (b) | A public file (`data/palette.ron`) in place of `maps/ch01.map` |
| content `lib.rs` `story_files_are_skipped_…`, `scripts_given_replace_the_bundles` | any | The New Game's first chapter is a chapter; errors under `assets*` |
| content `tests/it/all_assets_load.rs` `every_dialogue_line_has_a_unique_id` | any | "Some lines", not "more than 100" (the test files have 52) |
| content `tests/it/test_files.rs` (2 tests) | (a) | Guard: they compare the shipped story with the test build; the test speaker counted |
| content `tests/it/{arts,promotion,skills}.rs` (59 tests) | (a) | Guard each: the real tables' numbers |
| content `tests/it/{ai_soak,supports}.rs` | (b) | Unchanged: test map and pairs, the classes and items 0160 keeps |
| core `camp`, `campaign`, `worldmap`, `lead`, `save` tests, `tests/it/save_format.rs` | none | Unchanged: core reads no content; ids are plain strings |

**Without the checkout.** In a throwaway run (restored from git) every
story file was deleted from `assets/` (chapters, battles, maps, world
maps, openings, the title cinematic, skirmishes, every `.dlg`) and
`new_game.ron` pointed at the test chapter, as 0160's fallback will. The
content crate's tests then passed (647 + 68), about 100 of them skipping
by the guard; the four that failed at first are fixed above (`bundle.rs`,
`lib.rs` twice, `all_assets_load.rs`). The data tables stayed for that
run: the tests of mechanisms still read the classes, items, spells,
skills and arts the test files name, the `lead` character and its name,
the support rules and the music cues `title`, `world_map` and the
`skirmish` pool. 0160's Context now lists them and asks for the same run
on every crate.

**Deviations**

- **No new test world-map place**: the place test builds its place in
  the test (a side quest on `test_retreat`) and checks that the test
  world's own side quest is listed; no data was needed.
- **More files than listed**: the data tables' design tests (`class.rs`,
  `item`, `skill`, `art`, `ai.rs`, `audio.rs`) and four tests that only
  held with the story's files present.
- **A test speaker** (`test_voice`, in the test characters and names)
  for the opening tests' classless walker.
- **Not touched**: `readme_if_examples_are_valid` reads real characters
  through the public `.dlg` README's examples (0157's and 0160's; 0160
  names it).

**Notes for Nick:** nothing to check. No game rule was decided.
