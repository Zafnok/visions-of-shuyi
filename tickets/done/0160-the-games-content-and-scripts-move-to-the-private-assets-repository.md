---
id: "0160"
title: The game's content and scripts move to the private assets repository
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0156", "0157", "0158", "0162", "0163", "0164", "0165", "0166"]
nick_input: none
completed: 2026-11-13
---

# 0160 — The game's content and scripts move to the private assets repository

## Context

Nick picked B+ in 0153 (`docs/research/0153-keeping-the-story-private.md`
§9): the story and the game's data leave the public repository, so a copy
of it gets the code and a small test game, not this game. This ticket
moves the content and is the last step. By then:

- 0156 has built the content layer (`assets-private/content/` over
  `assets/`, every content-loading CI job checks it out);
- 0157 (the scripts' tests) and 0162–0165 (the data's) have readied the tests
  (none quotes or draws real content; those that need it run on the
  private copy and skip without it, ADR-0118), and 0166 the comments
  outside them (none names the story's people or tells the plot);
- 0158 has stopped the public web build and releases from carrying it.

**What moves** is everything that is this game's content: what a copy
would need to be this game. That covers chapters, battles, maps, world
maps, cinematics, openings, skirmishes, and every `.dlg` script, Chapter 1
included (0153 §3.1: 10 files, 5,980 lines, 308 scenes). From
`assets/data/` it covers characters, classes, items, skills, spells, arts,
AI, shops, camp, supports, names, new game, lessons and tips; also the
audio cue table, whose comments name plot, and the language packs' game
text.

**What stays** is engine-level, freely licensed, or shows no story:

- fonts;
- the sound and music files and their licences (`music/`, 48 tracks,
  CC0/CC-BY);
- palettes, the keymap, the UI text, credits;
- the public placeholder pictures that the bought art replaces;
- `assets/dialogue/README.md`, the `.dlg` format's reference (tests read
  it).

The session decides each remaining file by that rule and lists it in the
completion notes.

**The public files left must make a game that runs**: a build without the
checkout starts the test chapter (`assets-test/`, ADR-0057) as its new
game, so a fork or a worktree without access still builds, runs and passes
the tests that need no real content (0156's rule). Where a public file
needs a counterpart (for example `new_game.ron` pointing at the test
chapter, or an audio manifest of the public sounds), it goes in
`assets-test/`, or stays public as the default that `content/` replaces.

What the tests of mechanisms still read from the data that moves (0162's
completion notes list it): the classes, items, spells, skills and arts
the test files name (`exile`, `guard`, `archer`, `mage`, `raider`,
`brigand`, `swordsman`, `cleric`, `iron_sword`, `potion`, `fire`, …),
the `lead` character and its name, the support rules, and the music cues
`title`, `world_map` and the `skirmish` pool. 0162 ran the content
crate's tests with the story's files deleted and `new_game.ron` on the
test chapter: they passed, the tests of the content skipping (0163, 0164
and 0165 did the same for theirs). Run that again here, for every crate,
without the checkout. Also:
`dialogue/tests/presence.rs`'s `readme_if_examples_are_valid` reads real
characters through `assets/dialogue/README.md`'s examples.

## Nick input

None.

## Scope

**In:**
- The files above into the assets repository's `content/` (same paths as
  under `assets/`), through the `content-change` skill: an assets PR,
  then this PR, which deletes them here and moves `assets-private.rev`.
- The public fallback described above, and a test that a build without
  `private-content` starts the test chapter.
- Pointers: lines naming moved files (`assets/dialogue/ch0N.dlg`,
  `assets/chapters/…`, about 50 lines in 30 files for the scripts alone,
  0153 B1) name `assets-private/content/…`.
- `shuyi.ron`'s paths (`check_keys`, the Playdate list, the web build) and
  `crates/content/playdate-bundle.txt` read the layered view.
- `bench/*.bench.ron` name real battles: they run in the keyed CI job
  (0156) and keep naming ids only (0157's rule).
- `AGENTS.md`'s map of the repo; `docs/story/README.md`'s stub (0159) also
  names `content/`.
- The open tickets that edit scripts without writing words now go through
  `content-change` (0153 §3.3: 0852, 0853, 0854, and 0741, 0742 for
  Chapter 1). A line in each ticket's Context says so.

**Out (do not do):**
- Rewriting history (0153 §3.5).
- Changing any script's or data file's content (a move only; the writing
  pause holds).
- The writer's mirror (0152).

## Implementation steps

1. Write the size estimate first (rule 10). Deletions count: roughly
   15,000–20,000 lines deleted, and about 1,500 changed lines to review
   (the fallback, pointers, test data, tickets' lines, this ticket). It
   passes the 3,000-line warning by deletion; say so in the PR
   (ADR-0101). If the reviewable part passes 2,000, split off the
   pointers and the tickets' lines as a follow-up.
2. List every file of `assets/` as moves or stays, by the rule.
3. The public fallback and its test, on this repository first (it works
   before the move, since the layer is empty).
4. The assets PR, merged; then delete here, pointers, pin.
5. Check a fresh clone without access builds and starts the test chapter,
   in CI or a worktree without `assets-private/`. Never on a machine other
   than CI's or Nick's own worktrees.

## Acceptance criteria

- [x] Every file listed as "moves" is in the assets repository's
      `content/` and not in this repository (`git ls-files assets` in the
      completion notes).
- [x] With the private copy, every gate passes as before (CI).
- [x] Without it, the workspace builds and a new game starts the test
      chapter (test).
- [x] 0157's check finds no real line or map row in a tracked file
      outside `tickets/` and `docs/research/`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the fallback (a build without `private-content` loads the
  test chapter as the new game).
- Everything else is the existing suite, on the private copy in CI.

## Completion notes

**What was done**

- **The move**: 80 files (11,009 lines) copied into the private
  repository's `content/` at the same paths, each blob hash checked equal
  to this repository's at `5f92a4d6`, then deleted here:
  every chapter, battle, map, world map, cinematic, opening, skirmish and
  `.dlg` script, the data tables (characters, classes, items, skills,
  spells, arts, AI, basic shop, camp, supports, names, new game, lessons,
  tips) and `audio/audio.ron`. No file's content changed.
- **What stays** (by the rule): fonts, sounds and their licences, the
  music (`music/`), palettes, terrain, lighting, the keymap, the UI text,
  credits, the placeholder pictures (battlers, battlebacks, portraits,
  title, images, icon), `NOTICES-game.txt`, every format's `README.md`.
  No language pack has game text in public (`lang/en/` is the UI's).
- **The public fallback (ADR-0121)**: a stand-in at each moved data
  table's path and the audio manifest's, which the content's file replaces
  whole. New Game is the test chapter (lead and `test_knight`); the cast is
  the lead, the guide and the skirmish generator's `skirmish_*` templates;
  the class tree, skills, spells, items and arts are whole but the
  personal weapons (a wielder is a story character) and their arts; the
  lessons and tips (the tutorial) are whole; camp has no event and
  supports no pair (the test files add theirs). Each has a note saying what
  it is and keeps the format's notes, with no note that names a chapter or
  a person. Built by a script from the moved files, so the entries are the
  real ones the tests of mechanisms were written against.
- **Without the checkout** `build.rs` writes the demo's tables empty and
  refuses the `demo` edition (the demo is Chapter 1 of the content).
- **The test**: `private_content::without_the_content_new_game_starts_the_test_chapter`
  loads the bundle a build without the checkout has (`layers_with(None)`)
  and starts New Game: the test chapter, `Mara` and `Test Knight`, no
  chapter of the game.
- **Tests**: guarded as tests of the content (ADR-0118 §1 (a)):
  `spells_match_the_design`, `table_holds_every_registered_id`,
  `readme_if_examples_are_valid`, xtask's
  `the_writers_guide_examples_have_no_problems` and
  `every_lesson_and_tip_the_guide_says_has_a_clip`. On the test cast
  (b): `speakers_who_are_not_units_may_be_placed_and_speak`,
  `a_blocks_character_must_be_known`. Ten tests that wrote `assets/data/…`
  as a moved file's shown path ask the bundle for it (0156's note).
- **A bug the move showed**: `cargo xtask skirmish`'s `beside` took the
  first path segment as the data folder, so with the content it would have
  written a new skirmish into `assets-private/maps/`; it now takes what is
  before `/worldmaps/` (test added).
- **Pointers**: lines naming a moved story file name
  `assets-private/content/…` (`playdate-music.txt`, the title screen's and
  three UI tests' comments, `docs/ROADMAP.md`, `docs/design/demo.md`,
  `title-screen.md`, `docs/img/README.md`, `chapters/README.md`). A moved
  data table's path still names its stand-in, so those lines stay.
  `shuyi.ron`'s `audio_manifest` (only used to name the file in the
  credits check's errors) names the content's; `check_keys`, the Playdate
  list and the web build already read the layered view (0156).
  The benches name battle ids only and run in the keyed job.
- **The catalogues**: the tables of the real places
  (`assets/maps/README.md`) and openings (`assets/openings/README.md`)
  moved to `content/maps/README.md` and `content/openings/README.md`.
- `check-leaks`: the `MOVING` list is gone (`UNSEARCHED`: tickets only).
- `AGENTS.md`'s map, `docs/story/README.md`, the private
  `content/README.md`; the `content-change` line in 0741, 0742, 0852,
  0853 and 0854's Context; ADR-0121.

**`git ls-files assets`** after the move (116 files): `NOTICES-game.txt`;
`audio/` (README, `audio.ron` stand-in, licences, 21 sounds);
`battlebacks/`, `battlers/`, `portraits/`, `title/`, `images/`, `icon/`
(placeholder pictures and their `.ron`s); `data/` (`credits`, `keymap`,
`lighting`, `palette`, `palettes/*`, `terrain`, and the stand-ins `ai`,
`arts`, `basic_shop`, `camp`, `characters`, `classes`, `items`,
`lessons`, `names`, `new_game`, `skills`, `spells`, `supports`, `tips`);
`fonts/`; `lang/en/ui.ron`; and the `README.md` of `battles/`,
`chapters/`, `cinematics/`, `dialogue/`, `maps/`, `openings/`,
`tilesets/`, `worldmaps/`, `lang/`.

**Without the checkout** (`assets-private/content/` renamed away, a
clean build): `cargo test --workspace --no-fail-fast` passed 4,714 tests,
none failed, the tests of the content skipping. **With it**: 4,714
passed, none failed.

**Deviations**

- **No `private-content` feature** exists (0156 made the checkout the
  switch), so the test lays the bundle out without the content rather
  than building without a feature.
- **The stand-ins are larger than "the test files' ids"**: the test
  tileset draws every class, the heal list's, spell texts' and special
  weapons' tests read the personal spells and the special weapons, and the
  guides' and tip box's tests read every lesson and tip. Their numbers are
  in the public design documents already; what a fork lacks is the story
  (chapters, battles, maps, scripts, the cast and the notes).
- **The size estimate** was written after the first build, not before
  (rule 10's step was skipped); see the PR's size line.

**Size**: production code, measured 26 added lines by `cargo xtask
code-lines` (the build's switch for the demo and the skirmish tool's
fix), about the 30 expected: under 2×. PR size 9,854 changed lines by
`cargo xtask pr-size`: 8,624 are the 65 files deleted here (the move),
and the part to review is about 1,230 (+587 −643, of which the stand-ins
+87 −535 against the moved files): under the 2,000 split point, so no
split (ADR-0101).

**Gates**: every command of `run-gates` passed locally with the checkout;
`check-leaks` found no line or map row in any tracked file but the
tickets.

**Follow-up**: **0167**, the public READMEs and docs outside the design
documents and ADRs that still name the story's people and places
(`docs/playtesting.md`, `assets/dialogue/README.md`'s examples,
`crates/ui/README.md`, `docs/ROADMAP.md`, …).

**Notes for Nick**: nothing to check, and no game rule was decided. A
copy of the public repository now plays a small test chapter; the game
itself builds from the private repository, as CI and your machine do.
