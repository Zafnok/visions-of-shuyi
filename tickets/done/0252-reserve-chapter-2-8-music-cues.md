---
id: "0252"
title: Reserve Chapters 2-8's music cues, and run CI on PRs that change Markdown the code reads
type: bug
milestone: M1 Engine
model: implementation
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-10-28
---

# 0252 — Reserve Chapters 2-8's music cues, and run CI on PRs that change Markdown the code reads

## Context

`main` has been red since PR #464 (ticket 0071, story-battle themes and
scene music for Chapters 2-8).
`audio::tests::every_design_cue_is_in_the_manifest` in
`crates/content/src/audio.rs` (ticket 0214) checks that every cue
[`docs/design/audio.md`](../../docs/design/audio.md) lists is in
`assets/audio/audio.ron`, and now fails with

```
left: ["battle_desperate", "battle_rival", "battle_night_siege", "battle_war",
       "battle_finale", "scene_departure", "scene_confide", "scene_lamplight",
       "holy_city", "scene_dread"]
right: []
```

0071 added those cues to `audio.md`'s *Music cues* and left importing
them to the chapter battle tickets (0848–0854, its *Out*), none of which
is done. Ticket 0251 fixed the same break for the world map's cues by
adding them to the test's `RESERVED` list and telling the importing
ticket to drop its cue from it; this ticket does the same.

Why the PR didn't catch it: under ADR-0014, CI skips its build and tests
on a pull request that changes only docs (`*.md`, `docs/**`, …), and #464
changed only `audio.md`. A push to `main` runs everything. Nick asked for
that loophole to be closed in this ticket too, running only the tests
that read the changed file, not the whole CI (ADR-0098).

## Nick input

None.

## Scope

**In:**
- Add the ten cues to `RESERVED` in `crates/content/src/audio.rs`, and
  name 0071 and 0848–0854 in its doc comment.
- In each chapter battle ticket that plays one of them, add to its import
  step: remove the cue from `RESERVED` when importing it (the test's
  "drop it from RESERVED" assertion fails otherwise):
  0848 `battle_desperate`; 0850 `battle_rival`; 0851 `scene_confide`,
  `battle_night_siege`; 0852 `battle_war`; 0853 `holy_city`,
  `scene_dread`; 0854 `battle_finale`, `scene_departure`,
  `scene_lamplight`.
- In `.github/workflows/changes.yml`, map each Markdown file the code
  reads to the crates that read it; a CI job `doc-tests` runs only those
  crates' tests on a docs-only PR that changes one; an xtask test keeps
  the map exact (ADR-0098, extending ADR-0014).

**Out (do not do):**
- Importing any track, its credits or `THIRD_PARTY_ASSETS.md` lines (the
  chapter tickets).
- Changing the test's checks.

## Implementation steps

1. In `crates/content/src/audio.rs`'s tests, grow `RESERVED` from
   `[&str; 6]` to `[&str; 16]`, appending the ten cues in `audio.md`'s
   order, and extend its doc comment.
2. Add the sentence above to step 3 of 0848, 0850, 0851, 0852, 0853 and
   0854.
3. In `changes.yml`'s `case`, before the docs-only patterns, one
   `read by <crates>` branch per set of crates, adding them to a new
   output `doc_crates`.
4. In `ci.yml`, a `doc-tests` job (Ubuntu, `cargo test -p` each of
   `doc_crates`) when `code` is false and `doc_crates` isn't empty;
   `ci-result` needs it.
5. In `crates/xtask/src/main.rs`'s tests,
   `every_markdown_file_the_code_reads_is_tested`: the `.md` paths in each
   crate's string literals that name a real file equal those branches.
6. ADR-0098, its row in `docs/adr/README.md`, a pointer in ADR-0014.

## Acceptance criteria

- [x] `cargo test -p trpg-content --lib audio::tests::every_design_cue_is_in_the_manifest` passes.
- [x] 0848, 0850–0854 say to drop their cues from `RESERVED` on import.
- [x] A docs-only PR that changes `docs/design/audio.md` runs
      `trpg-content`'s tests and no other build (`doc_crates`, `doc-tests`).
- [x] `every_markdown_file_the_code_reads_is_tested` passes, and fails
      when a file is missing, stale or mapped to the wrong crate.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the existing `every_design_cue_is_in_the_manifest`, unchanged;
  `every_markdown_file_the_code_reads_is_tested` (xtask).

## Completion notes

Done as planned: `RESERVED` in `crates/content/src/audio.rs` now holds the
ten Chapter 2-8 cues from 0071, and 0848, 0850, 0851, 0852, 0853 and 0854
each say to drop their cues from it on import (0849 imports none: Greywater
uses `battle_church_2`, already in the manifest). The test itself is
unchanged. No gameplay rules decided; nothing for the Act 1 playtest.

Then, at Nick's request, the CI loophole that let #464 merge green:
`changes.yml` maps eight Markdown files to the crates that read them (`THIRD_PARTY_ASSETS.md`,
`assets/dialogue/README.md`, `assets/maps/README.md`, `docs/design/audio.md`,
`docs/engine/pins.md`, `docs/story/names.md`,
`docs/story/writers-guide.md`, `tickets/README.md`). A docs-only PR that
changes one runs only those crates' tests, on Ubuntu (`doc-tests`), not the
whole CI (Nick: "it should just run the CI that is required"). xtask's new
test keeps the map equal to what each crate names, skipping comments. The scan found
`assets/maps/README.md`, which I had missed (a content test needs it to
exist). `assets/battlers/README.md` is a made-up file name in a test and
is excluded in the test. ADR-0098 records it.

Gates run locally: fmt; clippy for `xtask` and `trpg-content`; the tests
of both crates (the guard was also seen failing on a missing file and on a
file mapped to the wrong crate); `ticket-lint`; `typos`. The classify
script was run on a scratch repository (`audio.md`, `names.md` and the
writer's guide gave `doc_crates=trpg-content xtask`, `code=false`). The
rest (workspace tests, the WASM and Playdate builds) is left to CI; the
`doc-tests` job itself first runs on the next docs-only PR that changes one
of the files.
