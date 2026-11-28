---
id: "0934"
title: "The demo keeps a language pack's entries for what it holds"
type: bug
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-27
---

# 0934 — The demo keeps a language pack's entries for what it holds

## Context

The Chapter 1 demo embeds a cut of the game's files (ticket 0914,
[ADR-0119](../../docs/adr/0119-the-chapter-1-demo-is-an-edition-built-from-a-cut-bundle.md);
`crates/content/demo/bundle.txt`, `crates/content/src/demo_bundle.rs`):
`data/names.ron` holds only the names Chapter 1 uses, and the later
chapters' scripts are left out.

A language pack isn't cut. 0725 laid a trial Chinese pack in (not
committed) whose `data.ron` had a `names.<id>` entry for every name, as
0726 will write it, and the demo's bundle no longer loaded:
`bundle::tests::the_demos_bundle_loads_chapter_1_alone_and_says_it_is_the_demo`
failed with 115 errors of the form `lang/zhhans/data.ron: "names.king":
no such key in the data`. A `data.ron` entry whose key the data lacks is
an error (`assets/lang/README.md`), and in the demo most keys are
lacking.

The pack's dialogue files have the other half of the problem: an entry
for a line of a script the demo leaves out is an orphan, which fails
nothing, so the demo would ship the Chinese of every later chapter. The
demo's export refuses a package that holds a later chapter's ids
(`shuyi.ron`, the edition's `disclosure`), and translated lines of later
chapters are the same leak.

No pack ships yet, so nothing is broken today. 0726 adds the first one
and can't pass the gates until this is done.

## Nick input

None. The demo shows Chapter 1 in every language the game has: Chapter 1
is what is translated first, and the demo is what a new player tries.
Verified by tests.

## Scope

**In:** the demo's cut of every pack (`lang/<code>/`), made at build time
from the files as they are, with no line to add to `bundle.txt` for each
language or each name.

**Out (do not do):** translating anything (0726); the pack's own
checks; the error's path, which named the trial's private `data.ron` as
`assets/lang/zhhans/data.ron` (write a ticket if it is still wrong once
a real pack is there); the test pack (`assets-test/`, not in the demo).

Expected size: about 80 production lines in `demo_bundle.rs` (one more
cut beside `keep`'s, which is about 120), and as many of tests.

## Implementation steps

1. In `demo_bundle.rs`, cut every `lang/<code>/data.ron` by the plan's
   own rules, with no new line in the list. An entry `(key: …, …)` stays
   when the data the demo holds has its key:
   - `names.<id>`: `<id>` is kept by the `keep data/names.ron` lines (a
     key keeps its `.` forms, as there);
   - `<table>.<id>.<field>` for a data file with `keep` lines
     (`characters`, and any other cut by then): `<id>` is kept by them;
   - `chapters.<id>.…` and `battles.<id>.…`: the file `chapters/<id>.ron`
     or `battles/<id>.ron` is in the demo;
   - any other key (a table the demo holds whole): stays.
2. Leave out `lang/<code>/dialogue/<file>.ron` when `dialogue/<file>.dlg`
   is left out. A pack may split its dialogue files another way
   (`assets/lang/README.md`); a file that matches no script is kept
   entry by entry, each entry staying when its key starts with a scene
   id of a script the demo holds. Captions (`caption.<scene>_…`) follow
   their scene.
3. `ui.ron` and `lang.ron` are kept whole.
4. Say so in `bundle.txt`'s header and `demo_bundle.rs`'s module comment.
5. If 0726 has merged first (it shouldn't: it is blocked by this), run
   the demo's export check too (`cargo shuyi export web --release
   --edition demo` is CI's: say so in the PR rather than run a release
   web build locally).

## Acceptance criteria

- [x] A pack `data.ron` with an entry for every id of the full `names.ron` is cut to the demo's names: a test in `demo_bundle.rs` on a sample text, with one key kept by its `.` form.
- [x] A pack's `dialogue/ch02.ron` is not in the demo's table, and `dialogue/ch01.ron` is, whole.
- [x] `the_demos_bundle_loads_chapter_1_alone_and_says_it_is_the_demo` passes with a pack laid in whose `data.ron` names every name (as a test's fixture text, or 0726's pack if it exists).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `demo_bundle.rs`, the cut of a pack's `data.ron` and the
  left-out dialogue files, on texts handed in (as its other tests do).
- Integration: the demo bundle's own load test, above.

## Completion notes

- `Plan::hold` (`demo_bundle.rs`) is told every file of the full game and
  reads the scene ids (`@scene` lines) of the scripts the demo holds;
  `build.rs` calls it in `write_demo`. Then `leaves_out` also leaves out
  `lang/<code>/dialogue/<file>.ron` when `dialogue/<file>.dlg` is left
  out, and `rewrite` cuts a pack's `data.ron` (an entry stays when the
  demo's data has its key) and any dialogue file that matches no script
  (by scene). `ui.ron`, `lang.ron` and a held script's file stay whole.
  `keep_entries` now shares its entry walk with the pack cut (`keep_by`).
- Deviation: the "file in the demo" rule is general, not only
  `chapters.` and `battles.`: a key `<table>.<id>.…` whose
  `<table>/<id>.ron` is a file of the game stays only when the demo holds
  it. That also cuts `worldmaps.<map>.<place>.name`, whose world maps the
  demo leaves out and which would otherwise fail the same way.
- A pack's `data.ron` with `characters.<id>.…` keys has none today
  (characters take their names from `names.<id>`); the rule is there for
  any table with `keep` lines.
- Tests: four unit tests on texts handed in, and
  `a_pack_naming_every_name_loads_in_the_demo`, which lays a fixture pack
  naming every name of the full game over `demo_layers()`: the cut pack
  loads, the uncut one fails with "no such key in the data". Checked
  once by hand with a trial pack in `assets/lang/zz/` (not committed):
  the demo's table held `data.ron` cut to 27 names and
  `dialogue/ch01.ron`, no `dialogue/ch02.ron`, and
  `the_demos_bundle_loads_chapter_1_alone_and_says_it_is_the_demo`
  passed.
- 0726 hasn't merged, so the demo's web export (`cargo shuyi export web
  --release --edition demo`) is left to CI.
- Friction: estimate 90, measured 129 (1.4×), under 2×.
- No gameplay rule decided.
