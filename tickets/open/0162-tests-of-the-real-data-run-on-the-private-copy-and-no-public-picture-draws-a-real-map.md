---
id: "0162"
title: Tests of the real data run on the private copy, and no public picture draws a real map
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0157"]
nick_input: none
completed:
---

# 0162 — Tests of the real data run on the private copy, and no public picture draws a real map

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

The files to sort, from 0157's inventory (2026-11-12; grep again, ids
named in string literals):

- `crates/ui/tests/it/`: `ch01_winnable.rs` … `ch05_winnable.rs`,
  `skirmishes.rs` (but `the_four_places_are_their_fixed_skirmishes`,
  0157's), `side_quests.rs`, `world_map.rs`, `ch01_lessons.rs`,
  `ch01_tips.rs`, `threat.rs`, `battle.rs` (`the_start_tip_waits_for_the_first_turn_scene`
  needs a test battle with a turn-1 scene), `playdate.rs` (the
  `with_shipped_story` tests), `title_cinematic.rs` (all but
  `the_storyboard_plays_shot_by_shot`, 0157's), and every test that calls
  `Harness::with_shipped_story`;
- `crates/ui/src/`: `map_view/playdate/tests.rs` (`harrowby_ford()`),
  `map_view/place.rs`, `cinematic/opening/tests.rs` (`ch01_sparring`,
  `ch01_pacing`), `screens/dialogue/tests.rs` (opening `ch01_sparring`),
  `screens/world_map/tests.rs`, `screens/battle/{doable,magic_tests,
  lesson_tests,lessons,art_tests,threat_tests,stage/tests}.rs`,
  `screens/preparations/tests.rs`, `audio.rs`, `tips.rs`;
- `crates/content/`: `chapter/tests.rs`, `battle/tests.rs`,
  `worldmap/tests.rs`, `cinematic/tests.rs` and `tests/openings.rs`,
  `lesson/tests.rs`, `names/tests.rs` (the table, not `names.md`, which
  0159 handles), `lang/data_tests.rs`, `skirmish/tests.rs`, `map.rs`,
  `character.rs`, `tileset.rs`, `support.rs`, `trigger.rs`,
  `dialogue/tests/presence.rs` (`the_cast_comes_from_…`,
  `every_kind_of_trigger_…`), `tests/it/{ai_soak,arts,promotion,skills,
  supports}.rs`;
- `crates/core/`: `camp/tests.rs`, `campaign/tests.rs`,
  `worldmap/tests.rs`, `lead/tests.rs`, `save/tests.rs`,
  `tests/it/save_format.rs` (ids only: check each needs no content);
- `crates/bots/`: `army.rs`, `arrival/tests.rs`, `balance/tests.rs`,
  `playtest/tests.rs`, `skirmish/tests.rs`, `testkit.rs`,
  `tests/baseline.rs`;
- `crates/xtask/`: `playtest/tests.rs`, `skirmish/tests.rs`,
  `game_tools/balance/tests.rs`, `game_tools/tests.rs` (the battle half),
  `battler_import.rs`, `tileset_import/tests.rs`;
- `crates/playdate/src/chapter_1.rs` (plays Chapter 1 on the Playdate's
  bundle);
- `bench/*.bench.ron`: real battles by id, in the keyed CI job (0160
  keeps them naming ids only).

Picture tests: every `.snap` that draws a real map, battle or world map
(about 140 under `crates/ui/tests/it/snapshots/` come from tests that
start the real story; read each). 0157's `cargo xtask check-leaks` can't
see them: a snapshot draws the map's tiles as the skin paints them, not
the map file's characters.

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
  data added to `assets-test/` (a test opening, a test battle with a
  turn-1 scene, a test world map's places, …).
- Each picture test of a real map re-pointed (b), or its `.snap` moved to
  the private repository's `content/tests/snapshots/` (through the
  `content-change` skill) if it must check the real look.
- Comments and test names that tell the plot ("Joss joins in the scene
  after") reworded to say what the test checks (ADR-0118 §2).

**Out (do not do):**
- The scripts' tests, the guard itself, the ADR and the leak check
  (0157).
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
5. Move the picture tests that must check the real look.
6. Simulate the checkout's absence: the guard's tests and a run with
   `assets-private/content/` holding a stand-in `chapters/` cannot show it
   before 0160, so run the (b) tests with `story_only` where a test of the
   test files can, and say in the notes what 0160 must run.

## Acceptance criteria

- [ ] The inventory is in the completion notes, each test marked (a) or (b).
- [ ] Every (a) test starts with the guard; every (b) test reads only the
      test files and what 0160 keeps public.
- [ ] No tracked `.snap` draws a real map, battle or world map.
- [ ] `cargo xtask check-leaks` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The re-pointed tests and their picture tests.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
