---
id: "0163"
title: The UI screens' unit tests of the real data run on the private copy
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0162"]
nick_input: none
completed:
---

# 0163 — The UI screens' unit tests of the real data run on the private copy

## Context

Split from 0162 before coding (rule 10). 0162's inventory of the tests
that read the real data (chapters, battles, maps, world maps, openings,
cinematics, skirmishes, characters, classes, items) found about 70 files
and 1,000 tests in six crates, plus about 140 picture tests: past one
PR. 0162 kept the content and core crates' tests and adds the shared test
data (in `assets-test/`); this ticket takes the **unit tests inside
`crates/ui/src/`**; 0164 the UI's integration tests and their pictures;
0165 the bots, xtask, the Playdate crate and the benches.

The rules are ADR-0118's (read it, and 0157's and 0162's completion
notes, in `tickets/done/`):

- **(a) a test of the content** stays on the real content and starts with
  `if trpg_content::bundle::skip_without_the_game() { return; }`;
- **(b) a test of a mechanism** moves to the test files (`assets-test/`),
  with test data added there;
- a public file may name a content id, never quote a line, draw a real
  map or describe the plot (comments and test names included);
- a picture test that must check the real look keeps its `.snap` in the
  private repository's `content/tests/snapshots/` (through the
  `content-change` skill); `crates/ui/tests/it/places.rs` shows how.

After 0160 moves the content, a checkout without access has only the
public files and the test files, so every test here must either skip (a)
or run on the test files (b). 0160 also moves `classes.ron`, `items.ron`
and the other data tables and builds a public fallback for the test
files; a re-pointed test must not assume more of the real data than the
test files keep.

The files, from 0162's inventory (2026-11-12; grep again for the loaders
`load_embedded`, `story_only`, `Harness::with_shipped_story` and real ids
in string literals: `ch01`…`ch08`, `act1…`, the battle, map, opening,
character, class and item ids of `assets/`):

- `map_view/playdate/tests.rs` (`harrowby_ford()`), `map_view/place.rs`;
- `cinematic/opening/tests.rs` (`ch01_sparring`, `ch01_pacing`);
- `screens/dialogue/tests.rs` (opening `ch01_sparring`);
- `screens/world_map/tests.rs`;
- `screens/battle/{doable,magic_tests,lesson_tests,lessons,art_tests,threat_tests,stage/tests}.rs`;
- `screens/preparations/tests.rs`;
- `audio.rs`, `tips.rs`;
- any other test under `crates/ui/src/` the grep finds, and the `.snap`
  files under `crates/ui/src/**/snapshots/` that draw a real map, battle
  or world map (read each).

0162 adds a test opening and a test world-map place with a named enemy
in `assets-test/`; use them, and add what else a re-pointed test needs.

## Nick input

None.

## Scope

**In:**
- The inventory of the files above, in the completion notes, each test
  marked (a) or (b).
- (a): the guard. (b): pointed at the test files, with test data added
  to `assets-test/`.
- Each picture test under `crates/ui/src/` of a real map re-pointed (b),
  or its `.snap` moved private if it must check the real look.
- Comments and test names that tell the plot reworded to say what the
  test checks (ADR-0118 §2).

**Out (do not do):**
- `crates/ui/tests/it/` (0164); bots, xtask, Playdate, benches (0165);
  content and core (0162).
- Moving the content (0160) or the public fallback it builds.
- Changing any real content.

## Implementation steps

1. Write the size estimate first (rules 9 and 10). Past about 2,000
   changed lines (snapshots aside), split again with `write-ticket`
   (screens/battle on its own), each blocking 0160.
2. The inventory: grep, then read each test.
3. Guard the (a) tests.
4. Re-point the (b) tests, adding test data.
5. Move or redraw the picture tests of real maps.
6. Run the (b) tests with `story_only` where a test of the test files
   can, and say in the notes what 0160 must run.

## Acceptance criteria

- [ ] The inventory is in the completion notes, each test marked (a) or (b).
- [ ] Every (a) test starts with the guard; every (b) test reads only the
      test files and what 0160 keeps public.
- [ ] No tracked `.snap` under `crates/ui/src/` draws a real map, battle
      or world map.
- [ ] `cargo xtask check-leaks` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The re-pointed tests and their picture tests.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
