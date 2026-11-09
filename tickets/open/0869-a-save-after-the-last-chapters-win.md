---
id: "0869"
title: "A save after the last chapter's win"
type: bug
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0869 — A save after the last chapter's win

## Context

Nick's playtest of the Act 1 build (2026-11-08):

> "The game, when you finish the last made chapter (currently ch4), when
> it says "to be continued" does not let you save so any playtester will
> need to replay that chapter again. Kinda annoying."

**Repro.** New Game, play to Chapter 4 (or load a save on the Act 1 world
map before Veyne Hall), travel to Veyne Hall, win. The results, then the
scenes `ch04_victory` and `ch04_tbc`, then "To be continued"
(`ToBeContinuedScreen`, `crates/ui/src/screens/game_over.rs` ~:164–212);
Confirm goes to the title. Load Game: the newest save is the one made on
the world map before the battle, so Veyne Hall must be fought again.
**Expected:** after the victory scenes the game asks "Save your progress?"
and a save written there loads on the world map with Veyne Hall cleared.

**Why** (`crates/ui/src/flow.rs`): a won battle's victory scenes end in
`next_scene` (~:588–600). Off the world map, `Then::NextChapter` shows
`Stage::SavePrompt` (0802's "Save your progress?", ~:597), and after it
`next_chapter` (~:791) goes on, or to `the_end` when there is no next
chapter. On the world map, `world_won` (~:515–544) clears the place
(`campaign.clear_node`) and then goes back to the map, starts the next
chapter, or, with no next chapter, calls `the_end` (~:741) directly: no
prompt, and the cleared state is never written. In Chapter 4 the only
saves are the world map menu's Save before the battle and the mid-battle
suspend.

Nick decided there is **no "Save your progress?" after a world-map battle**
when the map comes back (`docs/design/world-structure.md` ~:194, Nick:
"Straight to the map"): the player saves from the map's menu. That rule
stays. This ticket covers only the wins after which the map does **not**
come back.

A save made on the world map is `SaveFile::world_map` (`SavePoint::WorldMap`,
`crates/core/src/save.rs` ~:91), picked with
`SlotPickerScreen::save_world` (`crates/ui/src/screens/save.rs` ~:154); a
chapter save is `SaveFile::chapter_cleared`, which loads by starting the
campaign's chapter (`begin_loaded`), not the map.

## Nick input

None. *The agent's starting rule* (written in `world-structure.md`
beside Nick's rule): when a story battle on a world map is won and the map
won't come back after its scenes (no chapter follows, or the next one has
no world map), the game asks "Save your progress?" first, as at the end of
a chapter off the map, and a save made there is a world-map save of the
cleared state. Verified by the flow tests below; on the Act 1 playtest's
checklist (1064).

## Scope

**In:**
- In `world_won`, when the next step is `the_end` or a chapter with no
  world map: show `Stage::SavePrompt`; Yes opens
  `SlotPickerScreen::save_world` with the campaign as `clear_node` left it;
  saved, or No, goes on to where it was going ("To be continued", or the
  next chapter). Back from the slot picker asks again, as today
  (`advance`, ~:885–895). The flow must know which way to go on after the
  prompt (e.g. a field set before showing it, or a `Stage` variant).
- Loading that save: `SavePoint::WorldMap` → `begin_world` (~:887): the
  world map, the army at Veyne Hall, Veyne Hall cleared, the places it
  opened open.
- `world-structure.md` (*Saving and camp* or *The world map screen*): the
  rule above.

**Out (do not do):**
- A prompt when the map comes back (Nick's rule).
- What Act 1's end is (1065: the closing scene, "Act 1 ends", the army
  carried into Act 2). 1065 builds on this prompt; it is blocked by this
  ticket (a line to add to it).
- An autosave, or the title's Continue.
- What the player can do on the map after Chapter 4 (nothing new opens
  until 1071 and 1083).

## Implementation steps

1. Write the failing test first, on the test world map
   (`assets-test/worldmaps/test_world.ron`): with `test_wm1`'s `next` set
   to `None` (as `a_win_whose_next_chapter_has_no_map_starts_it` sets it,
   `crates/ui/tests/it/world_map.rs` ~:236), win Stone Mill: the top
   screen is `save_prompt`, not `to_be_continued`.
2. Route `world_won`'s two leaving cases through the prompt with a
   world-map save.
3. Update `ch04_winnable.rs`'s
   `chapter_4_plays_from_new_game_to_to_be_continued` (~:71): its loop
   presses Confirm on everything, so it would say Yes; have it answer the
   prompt (No, or save and check the file) before "To be continued".
4. Update `a_win_whose_next_chapter_has_no_map_starts_it` for the prompt.

## Acceptance criteria

- [ ] Winning the test map's last story battle with no next chapter shows
      `save_prompt`; Yes → slot picker → saved → `to_be_continued`
      (Harness test in `crates/ui/tests/it/world_map.rs`).
- [ ] The file written has `point == SavePoint::WorldMap`, the place in
      `cleared`, and the army's roster, stock and gold (test).
- [ ] Loading it from the title shows the world map with the place cleared
      (test).
- [ ] No → `to_be_continued`; nothing written (test).
- [ ] A win whose map comes back still goes straight to the map, no prompt
      (`win_a_story_battle_then_the_acts_final_one` passes unchanged).
- [ ] Chapter 4 from New Game reaches the prompt and then "To be
      continued" (`ch04_winnable.rs`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the Harness flow tests above (`world_map.rs`,
  `ch04_winnable.rs`, and `save.rs` if its world-map save tests need a
  case).

## Completion notes

*(Filled in by the session that completes the ticket.)*
