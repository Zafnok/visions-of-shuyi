---
id: "0191"
title: The frozen picture generators are deleted, and each picture says how it was made
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-18
---

# 0191 — The frozen picture generators are deleted, and each picture says how it was made

## Context

The [friction sweep of 2026-11-17](../../docs/research/friction-2026-11-17.md)
(*Processes that slow us down*, "One-shot generators kept forever")
proposed deleting six generators in `crates/xtask/src/` whose output is
committed: `edge_marks`, `effect_marks`, `test_card`, `test_units`,
`test_auto` and `title_art` (about 1,850 lines, 36 tests). Each one is
built, tested and measured for coverage on every code push.

Each was re-checked on 2026-11-17 (lines / tests / commits / last
change, from `wc -l`, `grep -c '#\[test\]'` and `git log`). Three can
go; three are still re-run:

| Tool | Lines / tests | Commits, last | Inputs | Verdict |
|---|---|---|---|---|
| `test_card` | 69 / 2 | 3, 61d537d2 (10-18) | none: the engine's fixed pixels (`shuyi_cli::test_card`) | Delete |
| `edge_marks` | 217 / 5 | 1, 90f97cf4 (10-21, #382) | the palette's `cursor` and `black` (`COLORS`, line 28) | Delete |
| `effect_marks` | 210 / 6 | 3, 61d537d2 (10-18) | the palette's `effect_bonus`, `effect_penalty`, `black` | Delete |
| `test_units` | 311 / 6 | 5, 61d537d2 (10-18) | the terrain, classes, palette and font atlas (`assets-src/README.md`) | **Keep**: `test-tileset` runs it (`main.rs` line 455), and 1226 edits it |
| `test_auto` | 573 / 9 | 5, 61d537d2 (10-18) | the same | **Keep**: run by `test-tileset`; #418 (10-23, new terrain) re-made its output; 1226 edits it |
| `title_art` | 465 / 8 | 4, 61d537d2 (10-18) | `assets-src/title/logo.txt`, the palette | **Keep**: re-run when the game's name or the logo changes (0906 checks the title's trademark), and `--bought` copies the bought backdrops (ADR-0059) |

The three kept are out of this ticket. The three deleted are 496 lines
and 13 tests, plus their wrappers and tests in `main.rs` (`test_card`,
`edge_marks`, `effect_marks`, and their `*_rejects_args` tests).

Each tool has a test that the committed picture is what the tool makes
(`committed_image_is_what_the_tool_makes` and its twins), which catches
a palette change that left a picture stale. For the two palette
pictures that guard stays, cheaper: a test that every opaque pixel's
color is one of the palette's named colors.

This bucket of the sweep (process and tooling): 0188 (split xtask), 0189
(a rules-text budget), 0190 (retire the Sonar issue lane), this ticket,
and the amended 0145, 0147, 0181, with 0151, 1174, 1257 and 1239 folding
into 0181.

Either order:

- 1235 (one report helper for the xtask commands) names the
  `test-card`, `edge-marks` and `effect-marks` wrappers. If this lands
  first, 1235 has three fewer to convert.
- 0188 (split xtask) moves the generators to the tool program. If this
  lands first, there are three fewer files to move.
- 1212 uses the engine's sprite-test screen with the test card; the
  picture stays, so either order works.
- 1213 draws the edge arrows from the kit's skins with
  `EDGE_MARKS_PATH`; the picture stays.

## Nick input

None.

## Scope

**In:**
- Delete `crates/xtask/src/test_card.rs`, `edge_marks.rs` and
  `effect_marks.rs`, their `mod` lines, their `main.rs` wrappers, their
  `USAGE` lines and their tests in `main.rs`.
- Before deleting, record each tool's last commit:
  `git log -1 --format=%h -- crates/xtask/src/<tool>.rs` (today
  61d537d2, 90f97cf4 and 61d537d2; recompute, it may have moved). In the
  picture's README row, say it is a generated picture whose tool was
  deleted, and how to get it back:
  `git show <hash>:crates/xtask/src/<tool>.rs`.
  - `assets/images/README.md`: `effect_marks.png`, and a new row for
    `edge_marks.png` (it has none today). The paragraph "Both are
    generated; never edit them by hand…" and its commands change to say
    so.
  - `assets-test/README.md` (`test_card.png`) and `assets-src/README.md`
    (the `test-card` and `effect-marks` rows).
- `assets/data/palette.ron` lines 46-48: the comment "rerun `cargo xtask
  effect-marks` after changing them" becomes "the arrows are baked into
  `images/effect_marks.png`: restore its tool (`assets/images/README.md`)
  when changing them". A shorter note beside `cursor` (line 54) says
  `images/edge_marks.png` bakes it too.
- In `trpg-content` (beside the image table's tests): a test that every
  opaque pixel of `effect_marks.png` and `edge_marks.png` is one of the
  palette colors its tool used.
- One-line amendments where an ADR names a deleted command: ADR-0057
  (line 46, `test-card`) and ADR-0089 (lines 59 and 83); and
  `docs/engine/boundary.md` line 270 (`effect-marks`).

**Out (do not do):**
- `test_units`, `test_auto`, `title_art`, `test_tileset`, `sfx`, `icon`,
  `readme_pictures` and `voice_test` (still re-run; see the table).
- Changing any picture.
- Moving the kept generators into a crate of their own (0188 decides
  where the tool program lives).

## Implementation steps

1. Record the three hashes.
2. `git rm` the three files; remove the `mod` lines (`main.rs` lines 16,
   17 and 37), the dispatch arms (lines 120, 122, 123), the functions
   (`test_card`, `edge_marks`, `effect_marks`) and their tests
   (`test_card_rejects_args`, `edge_marks_rejects_args` and
   `effect_marks_rejects_args`, `main.rs` lines 1530, 1598 and 1604),
   and the three `USAGE` lines.
3. Check that nothing outside `main.rs` uses the deleted modules
   (`grep -rn "edge_marks::\|effect_marks::\|test_card::" crates`; on
   2026-11-17 only `main.rs` lines 436, 592 and 609 do). The palette
   names they read (`edge_marks.rs`'s `COLORS`, `effect_marks.rs`'s
   `Colors::embedded`) become the new test's input.
4. Write the color test and the README, palette and ADR lines.
5. `grep -rn "test-card\|edge-marks\|effect-marks" --exclude-dir=target
   --exclude-dir=tickets .` finds only the README notes and the ADR
   amendments.

Expected size: about 600 lines deleted, about 60 added.

## Acceptance criteria

- [x] The three files are gone, and `cargo xtask test-card` prints the
      unknown-command usage.
- [x] Each of the three pictures' README rows names its tool's last
      commit, and `git show <hash>:crates/xtask/src/<tool>.rs` prints the
      tool.
- [x] The color test fails when `effect_bonus` in `palette.ron` is
      changed (tried locally, noted) and passes as committed.
- [x] The grep of step 5 finds only the notes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the new palette-color test for the two pictures.
- The image table's existing all-assets test still loads all three
  pictures.

## Completion notes

- Deleted `test_card.rs`, `edge_marks.rs` and `effect_marks.rs` with their
  `mod` lines, dispatch arms, wrappers, `USAGE` lines and `*_rejects_args`
  tests (222 production lines and 13 + 3 tests). `cargo xtask test-card`
  now prints `unknown command: test-card` and the usage (exit 2).
- Last commits, recomputed: `test_card` 61d537d2, `edge_marks` 90f97cf4,
  `effect_marks` 61d537d2; `git show <hash>:crates/xtask/src/<tool>.rs`
  prints each. The README rows of `assets/images/`, `assets-test/` and
  `assets-src/` name them; `edge_marks.png` got its row.
- New test `image::tests::baked_pictures_are_drawn_in_their_palette_colors`
  in `trpg-content`: every opaque pixel of the two arrow pictures is one of
  its tool's palette colours, each colour is used, and no pixel is partly
  see-through. Tried locally: `effect_bonus` set to `#70d8e9` fails it
  ("images/effect_marks.png has [112, 216, 232] …"); as committed it passes.
  `trpg-content` gains `shuyi-assets` with `png` as a dev-dependency to
  decode them.
- Deviations: ADR-0049 §5 also named `effect-marks` and its "test fails if
  the committed file isn't what the tool makes", so it got the same
  one-line amendment; `docs/engine/boundary.md` line 264 (`test-card`) too,
  beside line 270. The doc comments of `TEST_CARD_PATH`,
  `EFFECT_MARKS_PATH` and `EDGE_MARKS_PATH` no longer name the commands.
- The step-5 grep finds only the ADRs' original text with its amendments
  and the two boundary-table notes.
- Engine friction: none, production code only removed (0 added).
- No follow-up tickets. No gameplay rules decided.
