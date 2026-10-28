# ADR-0098: A Markdown file the code reads is not docs-only

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

## Decision

- `.github/workflows/changes.yml` names the Markdown files the code reads
  on one `case` line (`code (read by code)`), before the docs-only
  patterns. A pull request that changes one of them counts as code and runs
  every job.
- xtask's test `every_markdown_file_the_code_reads_counts_as_code` keeps
  that list exact. It collects every string literal in `crates/` that ends
  in `.md` and contains a `/`. It strips the leading `../` and keeps those
  that name a file at the repository's root, under `assets/` or under
  `assets-test/`. That set must equal the list. A file a literal names but
  no code reads is removed in the test, with a comment saying why.
- A new test that reads a Markdown file therefore fails until the file is on
  the list, and a stale entry fails too.

## Consequences

- A PR that edits a design doc a test checks (most often `audio.md`) runs
  the full CI, a few minutes instead of under one. Every other docs-only PR
  still skips it.
- A test that reads Markdown through a path built at run time (a folder
  joined with a file name in two literals) isn't seen by the scan. Write
  the path as one literal.

## Alternatives considered

- **Run every test on every PR**: this gives up ADR-0014's savings on the
  many design, story and ticket PRs that touch no file the code reads.
- **Always run only `trpg-content`'s tests on docs-only PRs**: this costs a
  build on every docs PR and still misses the files xtask and engine-watch
  read.
- **A hand-kept list with no test**: the next test that reads a doc would
  bring the problem back without anyone noticing.
