# ADR-0118: Tests of the real content, and what a public file may show

- **Status:** Accepted (0157); extends ADR-0117 and ADR-0057
- **Date:** 2026-11-12
- **Related tickets:** 0153, 0156, 0157, 0159, 0160, 0162

## Context

Nick picked B+ in 0153 (`docs/research/0153-keeping-the-story-private.md`
§9): the story and the game's data move into the private assets
repository, and the code stays public. ADR-0117 lays the private
`content/` over `assets/` in every build that has it checked out, and
every CI job that loads content checks it out.

Two things are left for the tests before 0159 and 0160 move the files:

- **A test of the real content has to find it.** After 0160, a checkout
  without access (a fork, a worktree that never ran `cargo shuyi
  private-assets`) builds on the public files and the test files
  (`assets-test/`, ADR-0057). A test that needs Chapter 3's scenes would
  fail there for a reason that has nothing to do with the code.
- **The public files must not show the content.** Tests kept real lines
  in string literals, in comments that told the plot, and in picture
  tests (`.snap` files of a dialogue screen or a title cinematic). Moving
  the scripts while those stay is a leak.

## Decision

### 1. Two kinds of test

Every test that reads the game's own content is one of two kinds:

- **(a) A test of the content.** It checks the real game: the content
  validates, a chapter's scenes are wired, the chapters can be won, the
  story's continuity holds. It stays on the real content and starts with

  ```rust
  if trpg_content::bundle::skip_without_the_game() {
      return;
  }
  ```

  `bundle::has_the_game()` is whether a chapter is in the bundle outside
  the test files. Without it the test prints why and returns: it
  **skips**. In CI (`CI` set when the crate is built) a missing content is
  a panic instead, so a CI job that forgot the checkout fails rather than
  skipping a gate. Until 0160 moves the files, the chapters are in
  `assets/`, so only the tests that need the private test data (§3) skip
  in a worktree without the checkout. The helpers are built only where
  the test files are (a debug build or `test-fixtures`), never in a build
  that ships.
- **(b) A test of a mechanism** (a screen, the dialogue log, a parser, a
  rule) that happened to use real content. It moves to the test files
  (`assets-test/`), with test data added there where it needs some, and
  then never skips.

A test that does both is split into the two.

### 2. What a public file may show

A tracked file may **name** a content id: a scene, chapter, battle, map
or character id, as 0914 and the public data already do. It may **not**:

- **quote a line** of a script: a caption, a speech, a narration or a
  reply, whole or in part;
- **draw a real map**: a row of a map's grid, or a picture test of one;
- **describe the plot**: who dies, who learns what, what a scene reveals,
  in a comment, a test's name or a message.

A test of the content that needs a line asks the content for it, by
scene and line, at run time. Its expectations that would tell the plot
are data in the private repository (§3).

Tickets, ADRs and design documents are outside this rule: tickets stay as
they are (0153 §3.5), and 0159 moves the story's documents and the story
sections of the design documents.

### 3. The content's own test data: `content/tests/`

What a test of the content checks it against, when saying it would tell
the story (the facts a scene must carry, the paths through the story),
lives in the private repository's `content/tests/`. The build never embeds
that folder (`crates/content/build.rs`, `CONTENT_TESTS`); the tests read
it from disk at `assets-private/content/tests/`, and skip without it as
§1 says. It is checked out with the rest of `content/`, so CI has it.

A picture test that must check the real look keeps its `.snap` there too,
in `content/tests/snapshots/`: `crates/ui/tests/it/main.rs`'s
`assert_private_snapshot!` points insta's snapshot path at it. Such a
test starts with `bundle::content_tests()`, which gives the folder, or
skips (and fails in CI) without it. 0157 moved seven: Chapter 1's places,
its intro's opening and the title's two shots of talk.

### 4. The check: `cargo xtask check-leaks`

`crates/xtask/src/leaks.rs` reads the scripts and maps as the build does
(the content's over `assets/`'s) and searches every tracked file for:

- any 7 words in a row of a line, by words (case, punctuation and line
  breaks aside, so a line wrapped in a text box is found); 6 found common
  phrases ("at the end of the") in 65 files;
- a whole line of 4 to 6 words, as written;
- a whole map row of 12 characters or more with three kinds of tile.

A token (`{n:keeper}`) splits a line, as the screen fills it in. Tickets
and the files 0159 and 0160 move are not searched (`MOVING`; 0160 empties
the list). It names each quote by its place and its source's, never the
text, because CI's logs are public. CI's `tickets` job, which checks the
content out, runs it on every pull request.

Plot told in other words is beyond a search; §2 holds it by review.

## Consequences

- After 0160, a fork's `cargo test` passes: the tests of the content
  skip, the tests of mechanisms run on the test files. CI runs them all.
- A new test of the content needs the guard; one that forgets it fails in
  a checkout without the content, which is where it is noticed.
- A test of the content that needs plot facts takes a pull request in
  the private repository for its data (the `content-change` skill), like
  a script change.
- The check misses a line quoted in fewer than 7 words that isn't a whole
  line, and a map drawn in glyphs (a snapshot shows tiles as the skin
  paints them, not the map's characters). Picture tests of real maps are
  0162's to move.
- Before 0160 the guard is never taken; 0160 is where it is first
  exercised, by running the tests without the checkout.

## Alternatives considered

- **`#[ignore]` the tests of the content, by a `cfg` each crate's build
  script sets.** Visible in the test output as ignored, but five crates
  would need a build script repeating the content crate's view of the
  bundle, and `cargo test -- --ignored` would run the other ignored
  tests too. Lost.
- **Leave the tests of the content failing without the checkout.** A
  fork could not tell a broken change from a missing folder. Lost.
- **Stand-in scripts in public with the same scene ids** (0153's design
  iii). The stand-ins name every scene of the story and keep its shape:
  more to keep in step, and the ids are all a test of a mechanism needs
  from the test files anyway. Lost.
- **Move the plot tests' code itself into the private repository**,
  included when present. Code in the data repository, linted only where
  it is checked out. Lost to data read by public code.
