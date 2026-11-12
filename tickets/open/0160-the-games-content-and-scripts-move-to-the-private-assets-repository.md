---
id: "0160"
title: The game's content and scripts move to the private assets repository
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0156", "0157", "0158", "0162", "0163", "0164", "0165"]
nick_input: none
completed:
---

# 0160 — The game's content and scripts move to the private assets repository

## Context

Nick picked B+ in 0153 (`docs/research/0153-keeping-the-story-private.md`
§9): the story and the game's data leave the public repository, so a copy
of it gets the code and a small test game, not this game. This ticket
moves the content and is the last step. By then:

- 0156 has built the content layer (`assets-private/content/` over
  `assets/`, every content-loading CI job checks it out);
- 0157 (the scripts' tests) and 0162–0165 (the data's) have readied the tests
  (none quotes or draws real content; those that need it run on the
  private copy and skip without it, ADR-0118);
- 0158 has stopped the public web build and releases from carrying it.

**What moves** is everything that is this game's content: what a copy
would need to be this game. That covers chapters, battles, maps, world
maps, cinematics, openings, skirmishes, and every `.dlg` script, Chapter 1
included (0153 §3.1: 10 files, 5,980 lines, 308 scenes). From
`assets/data/` it covers characters, classes, items, skills, spells, arts,
AI, shops, camp, supports, names, new game, lessons and tips; also the
audio cue table, whose comments name plot, and the language packs' game
text.

**What stays** is engine-level, freely licensed, or shows no story:

- fonts;
- the sound and music files and their licences (`music/`, 48 tracks,
  CC0/CC-BY);
- palettes, the keymap, the UI text, credits;
- the public placeholder pictures that the bought art replaces;
- `assets/dialogue/README.md`, the `.dlg` format's reference (tests read
  it).

The session decides each remaining file by that rule and lists it in the
completion notes.

**The public files left must make a game that runs**: a build without the
checkout starts the test chapter (`assets-test/`, ADR-0057) as its new
game, so a fork or a worktree without access still builds, runs and passes
the tests that need no real content (0156's rule). Where a public file
needs a counterpart (for example `new_game.ron` pointing at the test
chapter, or an audio manifest of the public sounds), it goes in
`assets-test/`, or stays public as the default that `content/` replaces.

What the tests of mechanisms still read from the data that moves (0162's
completion notes list it): the classes, items, spells, skills and arts
the test files name (`exile`, `guard`, `archer`, `mage`, `raider`,
`brigand`, `swordsman`, `cleric`, `iron_sword`, `potion`, `fire`, …),
the `lead` character and its name, the support rules, and the music cues
`title`, `world_map` and the `skirmish` pool. 0162 ran the content
crate's tests with the story's files deleted and `new_game.ron` on the
test chapter: they passed, the tests of the content skipping. Run that
again here, for every crate, without the checkout. Also:
`dialogue/tests/presence.rs`'s `readme_if_examples_are_valid` reads real
characters through `assets/dialogue/README.md`'s examples.

## Nick input

None.

## Scope

**In:**
- The files above into the assets repository's `content/` (same paths as
  under `assets/`), through the `content-change` skill: an assets PR,
  then this PR, which deletes them here and moves `assets-private.rev`.
- The public fallback described above, and a test that a build without
  `private-content` starts the test chapter.
- Pointers: lines naming moved files (`assets/dialogue/ch0N.dlg`,
  `assets/chapters/…`, about 50 lines in 30 files for the scripts alone,
  0153 B1) name `assets-private/content/…`.
- `shuyi.ron`'s paths (`check_keys`, the Playdate list, the web build) and
  `crates/content/playdate-bundle.txt` read the layered view.
- `bench/*.bench.ron` name real battles: they run in the keyed CI job
  (0156) and keep naming ids only (0157's rule).
- `AGENTS.md`'s map of the repo; `docs/story/README.md`'s stub (0159) also
  names `content/`.
- The open tickets that edit scripts without writing words now go through
  `content-change` (0153 §3.3: 0852, 0853, 0854, and 0741, 0742 for
  Chapter 1). A line in each ticket's Context says so.

**Out (do not do):**
- Rewriting history (0153 §3.5).
- Changing any script's or data file's content (a move only; the writing
  pause holds).
- The writer's mirror (0152).

## Implementation steps

1. Write the size estimate first (rule 10). Deletions count: roughly
   15,000–20,000 lines deleted, and about 1,500 changed lines to review
   (the fallback, pointers, test data, tickets' lines, this ticket). It
   passes the 3,000-line warning by deletion; say so in the PR
   (ADR-0101). If the reviewable part passes 2,000, split off the
   pointers and the tickets' lines as a follow-up.
2. List every file of `assets/` as moves or stays, by the rule.
3. The public fallback and its test, on this repository first (it works
   before the move, since the layer is empty).
4. The assets PR, merged; then delete here, pointers, pin.
5. Check a fresh clone without access builds and starts the test chapter,
   in CI or a worktree without `assets-private/`. Never on a machine other
   than CI's or Nick's own worktrees.

## Acceptance criteria

- [ ] Every file listed as "moves" is in the assets repository's
      `content/` and not in this repository (`git ls-files assets` in the
      completion notes).
- [ ] With the private copy, every gate passes as before (CI).
- [ ] Without it, the workspace builds and a new game starts the test
      chapter (test).
- [ ] 0157's check finds no real line or map row in a tracked file
      outside `tickets/` and `docs/research/`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the fallback (a build without `private-content` loads the
  test chapter as the new game).
- Everything else is the existing suite, on the private copy in CI.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
