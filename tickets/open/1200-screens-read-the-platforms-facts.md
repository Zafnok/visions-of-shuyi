---
id: "1200"
title: "The title offers Quit only where the game can quit, from the platform's facts"
type: bug
milestone: M8 Release
model: sonnet-5
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1200 — The title offers Quit only where the game can quit

## Context

Engine PR #187 (ticket 3053), reached by the pin move of 2026-10-30, gives
every screen `ctx.platform`: the target's id, whether a game can quit,
whether its window can go full screen, and its keyboard, pointer and touch,
read from the target's profile. The web profile quits through the tab, and
the Playdate's through its system menu.

`Title::build` (`crates/ui/src/screens/title.rs`) pushes `Item::Quit` on every
target, so the web build and the Playdate list a Quit a player has no use for.
The Options screen's Fullscreen row and the window shortcut are the same
question, answered today by a list for the Playdate only
(`crates/ui/src/screens/options.rs`).

The Playdate and the tools also set the facts by hand: one field in
`crates/playdate/src/start.rs` and `crates/xtask/src/game_tools.rs`
(`ctx.platform.keyboard = Keyboard::Absent`), with the rest left at the
desktop's. So `can_quit()` is wrong there.

## Nick input

None. If `docs/design/playdate.md` or `docs/design/` says what the Playdate's
title lists, that wins.

## Scope

**In:**
- `crates/playdate/src/start.rs` and the tools' Playdate start set the whole
  of `ctx.platform` from the Playdate's profile (`PlatformFacts::from_profile`).
- `Title::build` adds `Item::Quit` only where `ctx.platform.can_quit()`.
- The Options screen's Fullscreen row, and the window's shortcut, only where
  the platform is windowed. The Playdate's row list keeps its other rows.

**Out (do not do):**
- Any new option, or a new way to leave the game.
- Settings menus of a Steam Deck build (0903).

## Implementation steps

1. Set the facts in both Playdate starts; `crates/playdate/src/start.rs`'s test
   checks `can_quit()` is false and `windowed` false.
2. The title's item, then Options' row. `Fullscreen` leaves the Playdate-only
   list, whose comment says why it was there.
3. Harness tests with `Harness::as_target` for `desktop`, `web` and `playdate`.

Expected size: about 15 production lines.

## Acceptance criteria

- [ ] As `web` and `playdate`, the title has no Quit and nothing on it quits.
- [ ] As `desktop`, the title and its snapshots are unchanged.
- [ ] As `playdate`, Options has no Fullscreen row, as before; as `web` and
      `desktop` it has one, as before.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration (Harness): the three targets above.
- Snapshot: only web's and the Playdate's title move, by the Quit row.
