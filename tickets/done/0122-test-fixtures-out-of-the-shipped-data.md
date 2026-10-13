---
id: "0122"
title: Move the test characters, chapter and battles out of the shipped data
type: infra
milestone: M0 Foundation
model: opus-5.5
effort: high
status: done
blocked_by: []
nick_input: none
completed: 2026-10-12
---

# 0122 — Move the test characters, chapter and battles out of the shipped data

## Context

Ticket 0803 put the real Chapter 1 into the game: `assets/data/new_game.ron`
starts `ch01`, and `characters.ron` has the real cast. Its step 2 also said
"keep test fixtures in `tests/fixtures/` instead of shipped data". 0803 did
not do that part: about 300 places in the tests, and the debug Quick Battle
(title screen, debug builds and the Pages build), use the test characters
and files below, so moving them is a refactor of its own.

What is still embedded in the game (`assets/`, so in every release build)
but is only for tests and the debug Quick Battle:

- `data/characters.ron`: characters `test_lord`, `test_knight`,
  `test_archer`, `test_scout`, `test_rogue`, `test_mage`; generics
  `test_brigand`, `test_raider`, `test_frost_elemental`.
- `data/names.ron`: their names. `data/supports.ron`: the two test pairs.
- `chapters/test.ron`, `chapters/quick.ron`, `battles/test.ron`,
  `battles/quick.ron`, `maps/test_small.map`.
- `dialogue/test*.dlg`, `portraits/test_lord*`, `portraits/test_knight*`,
  `lang/test/`, `tilesets/test*`.

The tests of the game flow already start a new game in the test chapter
through `trpg_content::NewGameDef::test_chapter()` (set by
`trpg_ui::harness` and `screen::tests::ctx`), not through `new_game.ron`.

Loading is described in ADR-0005 (data files) and ADR-0035 (battles and
chapters).

## Nick input

`None.` One thing changes for Nick: the debug Quick Battle. See step 1.

## Scope

**In:**
- A way for tests (and debug tools) to load content that has the test
  files, while a release build of the game embeds only the story's files.
- Moving the files listed above out of what a release build embeds.
- The debug Quick Battle keeps working in debug builds and the Pages build.

**Out (do not do):**
- Changing what any test checks.
- Changing Chapter 1's data.
- The mod layers (1012–1014): if they are done first, use them; this ticket
  does not wait for them.

## Implementation steps

1. Decide how the test files are layered over the story's, and write an ADR
   (`write-adr` skill). Two candidates: a second embedded directory
   (`assets-test/`) merged over `assets/` behind a cargo feature that the
   test builds and the `debug-tools` feature turn on (the way the
   `private-assets` feature overlays `assets-private/game/`, ADR-0040); or
   `trpg_content::load_with_scripts`-style loading of extra files given by
   the caller. Data tables (`characters.ron`, `names.ron`, `supports.ron`)
   are single files, so the layer must merge entries by id, not replace
   the file. The Quick Battle must keep working where it is offered today
   (`Ctx::debug_tools`).
2. Move the files; split the test entries of the three data tables into
   the test layer.
3. Make every test load the layered content: `trpg_content::load_embedded`
   callers in tests, `Ctx::embedded` in `trpg_ui::harness` and
   `screen::tests::ctx`, the bots' and xtask's tests.
4. Add a test that a build without the test layer has no id starting
   `test_` among its characters, generics, chapters, battles, maps and
   scenes, and no `quick` chapter.
5. Update the READMEs in `assets/` that describe the test files.

## Acceptance criteria

- [x] A release build of `trpg-app` without `debug-tools` embeds no test
      character, chapter, battle, map, scene, portrait, tileset or the
      `test` language pack (the test of step 4).
- [x] The debug Quick Battle works in a debug build and with `debug-tools`.
- [x] No test was removed or weakened.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the layer merges data tables by id; a clash is an error.
- Integration: step 4's test; the existing suites, unchanged in what they
  check.

## Completion notes

**What was done** (ADR-0057):

- The test files moved to **`assets-test/`**, at the paths they had under
  `assets/` (`assets-test/README.md` lists them). Also moved, though the
  ticket's list didn't name it: `images/test_card.png` (the sprite test
  card), a test file like the rest.
- `trpg_content::bundle` embeds `assets-test/` as a layer after `assets/`
  in a **debug build**, or with `trpg-content`'s new **`test-fixtures`**
  feature (turned on by `trpg-ui`'s `debug-tools` and `harness`). That is
  the same condition as `Ctx::debug_tools`, so the Quick Battle is offered
  exactly where its files are. A release build of `trpg-app` without
  `debug-tools` embeds none of them.
- `characters.ron`, `names.ron` and `supports.ron`: the test entries are in
  `assets-test/data/` and are added to the story's by id; an id (or a
  name, or a pair) the story has too is an error.
- **No test changed what it loads**: `load_embedded` gives a test build the
  layered content, so step 3 needed no edit at the ~300 call sites. Only
  expected error paths changed (`assets/…/test…` → `assets-test/…`).
- Step 4's test is `crates/content/tests/it/test_files.rs`. It loads the
  content with the test files hidden (`bundle::story_only`) and checks
  there is no `test` id or file and no `quick` chapter; a second test
  checks that the test files change nothing of the story's content.
- `crates/app/src/main.rs` refuses to compile a release build that would
  embed the test files by accident (`cargo build --release --workspace`
  shares `xtask`'s features with the game).
- `cargo xtask test-card` / `test-tileset` write to `assets-test/`;
  `check-script` and `check-keys` read both directories.

**Checked by hand:** `cargo build --release -p trpg-app` and a search of
the `.exe`: no test character, name, scene, map or pack is in it. What is
left are mentions in the embedded READMEs, in comments and in a credit's
path list (ticket 0123).

**Deviations:**

- **The private repository changed too.** Its `game/` held the bought
  portraits of `test_lord` and `test_knight`; `game/` is embedded whole,
  so a sold build would have embedded them and then failed to load (their
  characters are gone from it). They moved to a new folder `test/` there
  (commit `36eafea`), which is embedded only with the test files;
  `cargo xtask private-assets`, the CI action and `--pin`'s credit check
  know the folder, and `assets-private.rev` moved. The Pages build's Quick
  Battle shows the same portraits as before.
- `crates/app`'s tests can no longer run in the **release** profile
  (`cargo test --release -p trpg-app`): the Harness brings the test files
  in and the guard above stops the build. Nothing runs them that way.

**Follow-up ticket:** 0123 (the asset READMEs are embedded in the game).

**For Nick:** nothing changes in what you play. The debug Quick Battle is
where it was (your own builds and the Pages build); a sold build no longer
carries the test characters and maps.

No gameplay rule was decided in this ticket.
