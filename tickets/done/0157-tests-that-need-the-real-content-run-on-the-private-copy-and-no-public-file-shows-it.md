---
id: "0157"
title: Tests that need the real content run on the private copy, and no public file shows it
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0156"]
nick_input: none
completed: 2026-11-12
---

# 0157 — Tests that need the real content run on the private copy, and no public file shows it

## Context

Nick picked B+ in 0153 (`docs/research/0153-keeping-the-story-private.md`
§9): the story and the game's data move into the private assets
repository (0159, 0160), on the content layer 0156 builds. The tests must
be ready first, because after the move:

- **Tests that need the real content** have to find it. 0156 makes CI
  check it out in every job that loads content. Locally and on forks,
  what such a test does without the checkout is 0156's ADR's rule, which
  this ticket applies to each test.
- **Public files must not show the content.** Several tests keep the real
  content in this repository:
  - picture tests (`.snap` files, 235 tracked) that draw real maps or
    quote real lines (at least 8 UI snapshots quote dialogue; 0153 §3.5);
  - Rust tests whose string literals quote real lines or describe the
    plot (for example `crates/content/src/dialogue/tests/knowledge.rs`,
    the finale's facts).

  Moving the content while those stay is a leak.

Counts from 0153 (2026-11-10):

- 78 `.rs` files call the shipped-content loaders (`load_embedded`,
  `with_shipped_story`, `load_with_scripts`, …);
- 64 name chapter ids (`"ch0N…"`);
- 17 name real scene ids, 6 of them later-chapter ones (`chapter/tests.rs`,
  `knowledge.rs`, `presence.rs`, `ch05_winnable.rs`, `skirmishes.rs`,
  `game_tools/tests.rs`).

Most tests already run on the test files (`assets-test/`, ADR-0057: about
300 places load them).

## Nick input

None.

## Scope

**In:**
- An inventory, kept in the completion notes, of every test that reads
  real content: real chapters, battles, maps, characters, classes, items,
  scripts or story documents, not `assets-test/`. Mark each:
  - **(a) keep:** it checks the real content (the content validates, the
    chapters can be won, the names registry). It stays as it is, runs in
    CI on the private copy and follows 0156's rule without it.
  - **(b) re-point:** it tests a mechanism and happens to use real content
    (a screen, a menu, a rule). Point it at the test files and add test
    data to `assets-test/` where needed.
- Every public file that shows real content after 0159 and 0160, made not
  to:
  - a picture test of real content is re-pointed (b), or its `.snap` moves
    to the assets repository, read through the layer (insta's snapshot
    path), if it must check the real look;
  - a literal quoting a real line, or describing plot, is replaced by a
    lookup or by test data.
- A rule in `docs/adr/` (an amendment to 0156's ADR, or a short ADR):
  public files may name content ids, as 0914 already allows, but never
  quote a line, draw a real map or describe plot. Also a check for it,
  `cargo xtask` or a test: a list of real lines and map rows read from the
  private copy, searched for in tracked files. It runs in a keyed CI job.

**Out (do not do):**
- Moving the content (0159, 0160) or the content layer (0156).
- Pages and the releases (0158).
- Rewriting tickets, ADRs or design docs that describe plot: 0159 handles
  the design docs' story sections; tickets stay as they are (0153 §3.5).

## Implementation steps

1. Write the size estimate first (rules 9 and 10). Expected about
   1,500–2,000 changed lines, not counting snapshots (`pr-size` skips
   them). If the inventory (step 2) shows more, **split now** with
   `write-ticket`: this ticket keeps the scripts' tests, and a new one,
   blocked by this one and blocking 0160, takes the data's.
2. The inventory: grep the loaders and ids above; for picture tests, read
   each `.snap` that isn't drawn from `assets-test/`.
3. Re-point the (b) tests, adding test data in `assets-test/`.
4. Move or re-point the picture tests of real content.
5. Replace quoting literals.
6. The rule and its check.

## Acceptance criteria

- [x] The inventory is in the completion notes, each test marked (a) or (b).
- [x] With the private checkout, every test passes as before (CI).
- [x] Without it, the tests follow 0156's rule (the job list in its ADR).
- [x] The check finds no tracked file that quotes a real line or a real
      map row, outside `tickets/` and the files 0159 and 0160 move (its
      output in the completion notes).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The re-pointed tests and their picture tests; the check's own test (a
  made-up line it finds, and one it doesn't).

## Completion notes

**Split first (rule 10).** The inventory found about 120 code files and a
dozen picture tests that read real content: past one PR. As step 1 says,
this ticket kept the **scripts' tests** (those that read the real `.dlg`
scenes or quote their lines), the rule and the check; **0162** (new,
blocked by this one, blocking 0160) takes the **data's** (real chapters,
battles, maps, world maps, openings, cinematics, characters; the picture
tests of real maps). 0160's `blocked_by` and the roadmap name it.

**What was done**

- **ADR-0118**: (a) a test of the content stays on it and starts with
  `trpg_content::bundle::skip_without_the_game()` (skips without the
  content, panics in CI, where every job that runs it checks it out); (b)
  a test of a mechanism moves to `assets-test/`. A public file may name
  an id, never quote a line, draw a real map or describe the plot. The
  content's own test data goes in the private `content/tests/`, never
  embedded.
- **`bundle.rs`**: `has_the_game()` (a chapter in the bundle outside the
  test files), and, in builds with the test files only,
  `skip_without_the_game()` and `content_tests()` (the private test
  data's folder, or a skip). `build.rs` leaves `content/tests/` out of
  every table (PC and Playdate), and the layer-is-its-folder test knows.
- **`cargo xtask check-leaks`** (`crates/xtask/src/leaks.rs`): reads the
  scripts and maps as the build does (content over `assets/`) and
  searches every tracked file for any 7 words in a row of a line (by
  words, so a line wrapped in a snapshot's text box is found), a whole
  line of 4–6 words as written, and a whole map row (12+ characters, three
  kinds of tile). Tickets and the files 0159 and 0160 move are skipped
  (`MOVING`). It names places, never text (CI's logs are public). In CI's
  `tickets` job (which checks the content out and always runs) and in
  `run-gates`. 6 words found common phrases in 65 files; 7 found only real
  quotes.
- **The private repository**: `content/tests/knowledge.ron` and seven
  snapshots in `content/tests/snapshots/`, and `content/README.md`
  (Zafnok/visions-of-shuyi-assets#2).
- Tickets: 0159 also moves `assets-src/audio/voice-audition.html` (it
  plays real Chapter 1 lines: a story document); 0162 written; 0160 and
  the roadmap chained.

**Inventory of the scripts' tests**

(a) keep, on the real content; (b) re-point, to the test files.

| Test | Kind | What changed |
| ---- | ---- | ------------ |
| content `dialogue/tests/knowledge.rs`, the ten path tests | (a) | One test, `every_path_keeps_what_the_party_knows`, reads the facts, phrases and paths from the private `knowledge.ron`; the code names no plot. Three new mechanism tests on the test scene |
| content `knowledge.rs` `the_facts_are_read_from_the_words` | (b) | Made-up phrases; the real ones are the spec's `reads` |
| content `dialogue/tests/presence.rs` `chapter_1s_cast_is_the_new_game_roster_and_its_triggers` | (a) | Guard |
| content `presence.rs` `a_places_intro_counts_on_its_battles_named_enemies` | (a) for now | Guard; plot comment gone. Its place is the story's world map: re-pointing needs a test place with a named enemy (0162) |
| content `dialogue/tests/openings.rs` `chapter_1_opens_its_two_scenes_away_from_the_battle` | (a) | Guard |
| content `lib.rs` `place_checks_know_each_scenes_battle_and_join_the_dialogue_errors`, `a_scenes_battle_is_its_chapters_or_its_triggers` | (b) | The test scenes and map `test_small` only |
| content `tests/it/all_assets_load.rs` (3 tests) | (a) | None: they check whatever content is there, and pass on the public files too |
| content `names/tests.rs`, xtask `check_script` `the_writers_guide_examples_have_no_problems` | (a) | None here: they read `docs/story/`, which 0159 moves (its Context names both) |
| ui `dialogue_log.rs` (11 tests, 2 snapshots) | (b) | On the test scene `test`; snapshots redrawn from it |
| ui `places.rs`: Chapter 1's prebattle, victory, intro, last scene, darkening, victory fade | (a) | Guard; neutral names and comments; the quoted line is gone (the snapshot checks it); 4 snapshots moved private |
| ui `places.rs` `the_battle_map_behind_a_battles_scene_is_darkened` | (b) | Already on the test files |
| ui `openings.rs` (7 tests) | (a) for now | Guard; lines read from the content (`shows_line`), never quoted; neutral names; its snapshot moved private. Re-pointing needs a test opening (0162) |
| ui `title_cinematic.rs` `the_storyboard_plays_shot_by_shot` | (a) | Guard; its two talk snapshots moved private (the faces, map and logo ones are 0162's) |
| ui `scene_preview.rs` (5 tests) | (b) | `test_intro` |
| ui `skirmishes.rs` `the_four_places_are_their_fixed_skirmishes` | (a) | Guard (the file's plot comments are 0162's, with its other tests) |
| ui `playdate.rs` `chapter_ones_scenes_play_to_their_end_on_the_playdates_skins` | (a) | Guard |
| xtask `check_script` `the_repository_scripts_have_no_problems` | (a) | Guard; finds Chapter 1's script under either root |
| xtask `check_script` `a_script_of_the_game_is_checked_once_under_its_own_name` | (b) | `assets-test/dialogue/test.dlg` |
| xtask `voice/tests.rs` `the_story_scenes_are_the_scenes_of_assets_dialogue` | (a) | Guard |
| xtask `voice/tests.rs` two punctuation tests | (b) | Made-up lines in place of four fragments of real ones |
| xtask `main.rs` `lines_lists_a_scene_and_fails_on_an_unknown_one` | (b) | `test_intro`, `test_victory` |
| xtask `game_tools/tests.rs` `a_session_opens_on_the_battle_asked_for` | (b) for `--scene` | `test_intro`; its battle half is 0162's |
| app `args.rs` scene tests | none | Parse an id only; read nothing |
| `bench/dialogue-portrait.bench.ron` | (a) | None: an id in the keyed bench job (0160 keeps ids only) |
| `.claude/skills/story-writing` | — | Two real replies quoted as examples replaced by made-up ones |

Tests that name a real script only as a path string, or load the whole
bundle and work on any content, are unchanged: `the_staged_load_gives_the_same_content`,
`lines_know_their_dlg_file`, `scripts_given_replace_the_bundles`, the
content-layer temp-repository tests, the Playdate read report.

**`cargo xtask check-leaks`** before: 25 quotes in 13 files (the dialogue
log's test and snapshots, the places', the title cinematic's,
`knowledge.rs`, the story-writing skill). After:

```
check-leaks: OK, 1685 tracked files quote no line or map row of the game's
```

**Deviations**

- **Kept, not re-pointed, where the test files have nothing like it**:
  the openings tests need a test opening, and the place cast test a test
  world-map place with a named enemy. Both are data, so 0162 re-points
  them; until then they are guarded tests of Chapter 1, with no line
  quoted and their pictures private.
- **`content/tests/` is a new folder** in the private repository, which
  the ticket didn't name: the plot facts of the continuity test had to
  live somewhere private, and CI's sparse checkout of `content/` already
  fetches it.
- **No snapshot names a real test any more**: the moved ones were made
  again in the private folder under the tests' new names; each picture
  was compared byte for byte with the public one before deleting it.

**Without the checkout.** Before 0160 the chapters are still in
`assets/`, so nothing skips; the guard's own tests read made-up bundles
(`the_game_is_here_when_a_story_layer_has_a_chapter`,
`without_the_game_a_test_skips_but_fails_in_ci`). The private test data's
tests skip in a worktree without the checkout (`content_tests()`), and
fail in CI if it is missing.

**Size:** estimate about 450 production lines (the check, judged by
`mentions.rs` and `content_layer.rs`, and the guard); measured 301 by
`cargo xtask code-lines` (0.7×, under 2×). PR size: planned about 2,000
after the split, measured about 2,300 (the continuity test's rewrite was
bigger than planned); under the 3,000 limit, and the split already made
is the clean one.

**Notes for Nick:** nothing to check. No game rule was decided.
