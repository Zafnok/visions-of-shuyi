---
id: "0122"
title: Move the test characters, chapter and battles out of the shipped data
type: infra
milestone: M0 Foundation
model: opus-5.5
effort: high
status: todo
blocked_by: []
nick_input: none
completed:
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

- [ ] A release build of `trpg-app` without `debug-tools` embeds no test
      character, chapter, battle, map, scene, portrait, tileset or the
      `test` language pack (the test of step 4).
- [ ] The debug Quick Battle works in a debug build and with `debug-tools`.
- [ ] No test was removed or weakened.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the layer merges data tables by id; a clash is an error.
- Integration: step 4's test; the existing suites, unchanged in what they
  check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
