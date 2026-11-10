---
id: "0157"
title: Tests that need the real content run on the private copy, and no public file shows it
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0156"]
nick_input: none
completed:
---

# 0157 — Tests that need the real content run on the private copy, and no public file shows it

## Context

Nick picked B+ in 0153 (`docs/research/0153-keeping-the-story-private.md`
§9): the story and the game's data move into the private assets
repository (0159, 0160), on the content layer 0156 builds. The tests must
be ready first, because after the move:

- **Tests that need the real content** have to find it. 0156 makes CI
  check it out in every job that loads content. Locally and on forks,
  what such a test does without the checkout is 0156's ADR's rule, which
  this ticket applies to each test.
- **Public files must not show the content.** Several tests keep the real
  content in this repository:
  - picture tests (`.snap` files, 235 tracked) that draw real maps or
    quote real lines (at least 8 UI snapshots quote dialogue; 0153 §3.5);
  - Rust tests whose string literals quote real lines or describe the
    plot (for example `crates/content/src/dialogue/tests/knowledge.rs`,
    the finale's facts).

  Moving the content while those stay is a leak.

Counts from 0153 (2026-11-10):

- 78 `.rs` files call the shipped-content loaders (`load_embedded`,
  `with_shipped_story`, `load_with_scripts`, …);
- 64 name chapter ids (`"ch0N…"`);
- 17 name real scene ids, 6 of them later-chapter ones (`chapter/tests.rs`,
  `knowledge.rs`, `presence.rs`, `ch05_winnable.rs`, `skirmishes.rs`,
  `game_tools/tests.rs`).

Most tests already run on the test files (`assets-test/`, ADR-0057: about
300 places load them).

## Nick input

None.

## Scope

**In:**
- An inventory, kept in the completion notes, of every test that reads
  real content: real chapters, battles, maps, characters, classes, items,
  scripts or story documents, not `assets-test/`. Mark each:
  - **(a) keep:** it checks the real content (the content validates, the
    chapters can be won, the names registry). It stays as it is, runs in
    CI on the private copy and follows 0156's rule without it.
  - **(b) re-point:** it tests a mechanism and happens to use real content
    (a screen, a menu, a rule). Point it at the test files and add test
    data to `assets-test/` where needed.
- Every public file that shows real content after 0159 and 0160, made not
  to:
  - a picture test of real content is re-pointed (b), or its `.snap` moves
    to the assets repository, read through the layer (insta's snapshot
    path), if it must check the real look;
  - a literal quoting a real line, or describing plot, is replaced by a
    lookup or by test data.
- A rule in `docs/adr/` (an amendment to 0156's ADR, or a short ADR):
  public files may name content ids, as 0914 already allows, but never
  quote a line, draw a real map or describe plot. Also a check for it,
  `cargo xtask` or a test: a list of real lines and map rows read from the
  private copy, searched for in tracked files. It runs in a keyed CI job.

**Out (do not do):**
- Moving the content (0159, 0160) or the content layer (0156).
- Pages and the releases (0158).
- Rewriting tickets, ADRs or design docs that describe plot: 0159 handles
  the design docs' story sections; tickets stay as they are (0153 §3.5).

## Implementation steps

1. Write the size estimate first (rules 9 and 10). Expected about
   1,500–2,000 changed lines, not counting snapshots (`pr-size` skips
   them). If the inventory (step 2) shows more, **split now** with
   `write-ticket`: this ticket keeps the scripts' tests, and a new one,
   blocked by this one and blocking 0160, takes the data's.
2. The inventory: grep the loaders and ids above; for picture tests, read
   each `.snap` that isn't drawn from `assets-test/`.
3. Re-point the (b) tests, adding test data in `assets-test/`.
4. Move or re-point the picture tests of real content.
5. Replace quoting literals.
6. The rule and its check.

## Acceptance criteria

- [ ] The inventory is in the completion notes, each test marked (a) or (b).
- [ ] With the private checkout, every test passes as before (CI).
- [ ] Without it, the tests follow 0156's rule (the job list in its ADR).
- [ ] The check finds no tracked file that quotes a real line or a real
      map row, outside `tickets/` and the files 0159 and 0160 move (its
      output in the completion notes).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The re-pointed tests and their picture tests; the check's own test (a
  made-up line it finds, and one it doesn't).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
