---
id: "1311"
title: "The low-HP tip shows whenever its moment comes"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1311 — The low-HP tip shows whenever its moment comes

## Context

Nick, after watching a new player play Chapter 1 (2026-11-22):

> "The tip for low hp (rewind / potion) is not consistently appearing
> via trigger"

The tip is `low_hp` in `assets/data/tips.ron` (around line 60), trigger
`FirstLowHp`. The check is in `crates/ui/src/screens/battle/triggers.rs`
(`BattleScreen::triggers_now`, around line 99): any player unit with
`hp > 0 && hp * 2 <= max hp`. It runs every frame from
`take_in_tips`, called at the start of `update_frame`
(`screens/battle/mod.rs`). Whether it shows is decided in
`crates/ui/src/screens/battle/guide.rs`:

- `Guide::due_tip` shows a tip only in the player phase, with no banner,
  nothing playing (combat, EXP, level up, a queued scene, notes) and no
  screen over the battle;
- `shown_tip` holds every tip back while a lesson is open **or is the
  next one**, and Chapter 1 has many lessons (1304);
- the queue is first in, first out;
- `take_in_mode` and `set_covered` drop queued tips through `go_stale`
  (`screens/battle/tips.rs`); the trigger fires again on the next frame
  only while the unit is still at half HP or lower;
- a tip is saved as seen in `TipsSeen` once drawn (0467), per profile.

Likely causes of "not consistently": a unit hurt in the enemy phase and
healed (or killed, or rewound past) before the player phase settles, so
the moment passes before any frame may show it; the lesson queue holding
it the whole battle; the tip going stale on a mode change in the same
frame the trigger stopped holding. 1292 ("a tip that misses its moment
waits for the next") fixed one such case. 1310 rebuilds the triggers as a
registry; this fix is independent of it (either order works: if 1310 is
done first, put the fix in its `low_hp` predicate row).

## Nick input

None. Verified by the tests below; the Act 1 playtest (1064) checks it in
play.

## Scope

**In:** find every case where a player unit reaches half HP or less in
Chapter 1 and the tip never shows; a failing test for each; the fix.
The rule to keep: the first time a player unit is at half HP or less,
the tip shows at the next moment a tip may show (the player phase,
settled), even if the unit has since healed, unless the unit is dead or
the battle ended.

**Out (do not do):** changing the tip's text (dialogue writing is
paused); new tips; the lesson order.

## Implementation steps

1. Write failing tests in `crates/ui/src/screens/battle/tip_tests.rs`
   for: hurt in the enemy phase, then healed before the player phase;
   hurt while a lesson is next; hurt during a mode change; hurt, then
   rewound to before the hit (decide: the tip still shows once, since the
   player saw the danger).
2. Make the trigger latch: once seen at half HP, the tip stays queued
   until shown (it does not go stale on mode changes or covers), like
   1292 did for others.
3. If lessons hold it for a whole battle, let `low_hp` jump the lesson
   queue at the next lesson gap (it teaches Rewind, which saves a run).
4. Replay a bot's Chapter 1 record where a unit drops to half HP
   (`cargo shuyi playtest`, the casual persona) and check the tip's
   `TipsSeen` entry is written.

## Acceptance criteria

- [ ] Each case in step 1 has a test that failed before the fix and
      passes after.
- [ ] `a_badly_hurt_unit_shows_the_low_hp_tip` and the other tip tests
      still pass.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit/integration: the battle screen through the headless Harness, in
  `tip_tests.rs`.

## Lesser targets

The Playdate uses the same guide; nothing extra.

## Completion notes

