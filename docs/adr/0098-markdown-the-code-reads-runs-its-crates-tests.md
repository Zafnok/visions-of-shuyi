# ADR-0098: A Markdown file the code reads runs the tests of the crates that read it

- **Status:** Accepted
- **Date:** 2026-10-28
- **Related tickets:** 0252
- **Extends:** ADR-0014

## Context

ADR-0014 skips CI's build and test jobs on a pull request that changes
only docs-only paths (`*.md` anywhere, `docs/**`, `tickets/**`,
`.claude/**`). A push to `main` always runs everything.

Some tests read Markdown as data. `audio::tests::every_design_cue_is_in_the_manifest`
checks `docs/design/audio.md` against the audio manifest; others read
`docs/story/names.md`, `THIRD_PARTY_ASSETS.md` and the writer's guide.
PR #464 (ticket 0071) changed only `audio.md`, so its tests were skipped.
It merged green, and `main` went red on the merge commit, which left
every later PR red until ticket 0252 fixed it.

Running the whole CI on such a PR (three OSes, the web and Playdate
builds, coverage) would catch it, but only the tests of the crate that
reads the file can fail on a change to it.

## Decision

- In `.github/workflows/changes.yml`, each Markdown file the code reads is
  on a `read by <crates>` line of the `case`, before the docs-only
  patterns. A changed file on one of those lines adds its crates to a
  second output, `doc_crates`, and leaves `code` false.
- CI's `doc-tests` job runs when `code` is false and `doc_crates` isn't
  empty. On Ubuntu only, it runs `cargo test -p <crate>` for each of those
  crates, and no other job. `ci-result` needs it, so a failure blocks the
  merge. A PR with code runs the whole `test` job instead, which covers
  these tests.
- xtask's test `every_markdown_file_the_code_reads_is_tested` keeps those
  lines exact. It collects, per crate, every string literal outside a
  comment that ends in `.md` and contains a `/`. It strips the leading
  `../` and keeps those that name a file at the repository's root, under
  `assets/` or under `assets-test/`. Each such file must be on exactly one
  `read by` line, which names exactly the crates that name it. A file a
  literal names but no code reads is removed in the test, with a comment
  saying why.
- A new test that reads a Markdown file therefore fails until its file is
  on a line with its crate, and a stale line fails too.

## Consequences

- A PR that edits a design doc a test checks (most often `audio.md`) runs
  one crate's tests on one OS, a few minutes instead of under one. Every
  other docs-only PR still skips the build entirely.
- A test that reads Markdown through a path built at run time (a folder
  joined with a file name in two literals) isn't seen by the scan. Write
  the path as one literal.

## Alternatives considered

- **Count those files as code and run the whole CI**: this catches the
  break, but it builds three OSes, the web and the Playdate for a change
  only one crate's tests can see.
- **Run only the named test, not the crate's tests**: the build is most of
  the time anyway, and a list of test names would go stale faster than a
  list of crates.
- **A hand-kept list with no test**: the next test that reads a doc would
  bring the problem back without anyone noticing.
