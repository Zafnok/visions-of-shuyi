---
id: "0934"
title: "The demo keeps a language pack's entries for what it holds"
type: bug
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
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

- [ ] A pack `data.ron` with an entry for every id of the full `names.ron` is cut to the demo's names: a test in `demo_bundle.rs` on a sample text, with one key kept by its `.` form.
- [ ] A pack's `dialogue/ch02.ron` is not in the demo's table, and `dialogue/ch01.ron` is, whole.
- [ ] `the_demos_bundle_loads_chapter_1_alone_and_says_it_is_the_demo` passes with a pack laid in whose `data.ron` names every name (as a test's fixture text, or 0726's pack if it exists).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `demo_bundle.rs`, the cut of a pack's `data.ron` and the
  left-out dialogue files, on texts handed in (as its other tests do).
- Integration: the demo bundle's own load test, above.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
