---
id: "0164"
title: The UI integration tests of the real story run on the private copy, and no public picture draws a real map
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0162"]
nick_input: none
completed:
---

# 0164 — The UI integration tests of the real story run on the private copy, and no public picture draws a real map

## Context

Split from 0162 before coding (rule 10): see 0163's Context for the
split and ADR-0118's rules ((a) a test of the content keeps the real
content and starts with `if trpg_content::bundle::skip_without_the_game()
{ return; }`; (b) a test of a mechanism moves to `assets-test/`; a public
file names ids only, never a line, a real map or the plot; a picture that
must check the real look goes to the private `content/tests/snapshots/`).
Read 0157's and 0162's completion notes in `tickets/done/`.

This ticket takes **`crates/ui/tests/it/`** and its pictures
(`crates/ui/tests/it/snapshots/`, 131 files on 2026-11-12). From 0162's
inventory:

- `ch01_winnable.rs` … `ch05_winnable.rs`, `side_quests.rs`,
  `world_map.rs`, `ch01_lessons.rs`, `ch01_tips.rs`, `threat.rs`;
- `skirmishes.rs` (but `the_four_places_are_their_fixed_skirmishes`,
  already guarded by 0157; the file's plot comments are this ticket's);
- `battle.rs` (`the_start_tip_waits_for_the_first_turn_scene` needs a
  test battle with a turn-1 scene);
- `playdate.rs` (the `with_shipped_story` tests);
- `title_cinematic.rs` (all but `the_storyboard_plays_shot_by_shot`,
  0157's): its faces, map and logo shots;
- `openings.rs` (7 tests, guarded by 0157 for want of a test opening:
  0162 adds one, so re-point the mechanism tests to it);
- every other test that calls `Harness::with_shipped_story`.

**Picture tests:** every `.snap` that draws a real map, battle or world
map (read each: `flow`, `battle`, `cinematic`, `map_skin`,
`scene_camera`, `map_zoom`, `world_map`, `title`, `controller`, `save*`
and `options` ones start the real story in places). 0157's `cargo xtask
check-leaks` can't see them: a snapshot draws the tiles as the skin
paints them, not the map file's characters.

## Nick input

None.

## Scope

**In:**
- The inventory, in the completion notes, each test marked (a) or (b).
- (a): the guard. (b): pointed at the test files, with test data added
  to `assets-test/` (a test battle with a turn-1 scene, …).
- Each picture test of a real map re-pointed (b), or its `.snap` moved
  to the private repository's `content/tests/snapshots/` (through the
  `content-change` skill, `assert_private_snapshot!`) if it must check
  the real look.
- Comments and test names that tell the plot reworded (ADR-0118 §2).

**Out (do not do):**
- `crates/ui/src/` (0163); bots, xtask, Playdate, benches (0165);
  content and core (0162).
- Moving the content (0160) or the public fallback it builds.
- Changing any real content.

## Implementation steps

1. Write the size estimate first (rules 9 and 10). Past about 2,000
   changed lines (snapshots aside), split again with `write-ticket`
   (the winnable and story-flow tests apart from the picture tests),
   each blocking 0160.
2. The inventory: grep the loaders and real ids; read each picture test.
3. Guard the (a) tests; re-point the (b) tests, adding test data.
4. Move the picture tests that must check the real look; redraw the
   re-pointed ones from the test files.
5. Run the (b) tests with `story_only` where a test of the test files
   can, and say in the notes what 0160 must run.

## Acceptance criteria

- [ ] The inventory is in the completion notes, each test marked (a) or (b).
- [ ] Every (a) test starts with the guard; every (b) test reads only the
      test files and what 0160 keeps public.
- [ ] No tracked `.snap` under `crates/ui/tests/` draws a real map,
      battle or world map.
- [ ] `cargo xtask check-leaks` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The re-pointed tests and their picture tests.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
