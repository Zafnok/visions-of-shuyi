---
id: "1198"
title: "World map shortcuts on the Key bindings screen"
type: feature
milestone: Post–Chapter 1
model: implementation
effort: low
status: todo
blocked_by: ["1007", "engine#2395"]
nick_input: none
completed:
---

# 1198 — World map shortcuts on the Key bindings screen

## Context

Ticket 1007 added two world-map actions, `WorldArmy` (the cursor to the
army) and `WorldFind` (the Find list), in a key group of their own
(`KeyGroup::WorldMap`, `crates/content/src/keymap.rs`), with Nick's keys
(`c`/`x`, mirrored `,`/`.`) and buttons (North/West). Nick asked for them
"remappable, optional", but the PC Key bindings screen had no room: the
engine's glyph skin drew every row with no scrolling, and the panel was
full, so their rows ran over its border and pushed Restore defaults off.

Engine ticket 2395 (`Zafnok/shuyi-engine#180`) makes that skin scroll.
Until the game's pin is past it, 1007 leaves the two rows off the screen:
`crates/ui/src/input.rs` keeps them in `WORLD_MAP_ROWS` and
`NOT_LISTED_YET`, marked `// engine-friction: engine#2395`. Their keys
work; only rebinding waits.

## Nick input

None. Verified by tests and a `cargo shuyi frame-png` picture of the Key
bindings screen scrolled to its bottom in the PR.

## Scope

**In:**
- ~~The pin at or past engine#2395's merge: the open `engine-pin` PR, or
  the first step here moves it (`engine-change` §7).~~ (done by the pin move
  of 2026-10-30, `50ae33db` → `1eaa1fa0`)
- `WORLD_MAP_ROWS` joins `KEY_ROWS` (after the dialogue log's row, under
  `WORLD_MAP_HEADING`, "On the world map"); `NOT_LISTED_YET` and the
  workaround comments go.
- The Playdate keeps leaving them out (`NOT_ON_THE_PLAYDATE`): it has no
  world map yet.

**Out (do not do):**
- New actions or default keys (`docs/design/controls.md` holds Nick's).
- The Playdate's rows.

## Implementation steps

1. ~~Check engine#2395 is merged and the pin is past it
   (`work-ticket` §1). If not, move the pin (`engine-change` §7, with the
   `engine-pin` skill's review), or wait for the open `engine-pin` PR.~~
   (done by the pin move of 2026-10-30, `50ae33db` → `1eaa1fa0`)
2. `crates/ui/src/input.rs`: append `WORLD_MAP_ROWS` to `KEY_ROWS`
   (`KeyRow; 20`), delete `NOT_LISTED_YET` and its comment;
   `PLAYDATE_KEY_ROWS` stays as it is.
3. `crates/ui/src/screens/key_bindings.rs` tests: drop the
   `NOT_LISTED_YET` filter; the headings are the four groups, the last
   "On the world map" with `Your army` and `Find`.
4. Update the Key bindings snapshots (`crates/ui/tests/it/snapshots/
   it__key_bindings_screen__*`, `it__rebind_buttons__*`): the list now
   scrolls; check `Restore defaults` is reachable (a test that presses
   Up from the top lands on it and sees it drawn).
5. `grep -rn "engine#2395" crates` finds nothing left.

Expected size: about 10 production lines (the rows move into the table).

## Acceptance criteria

- [ ] The pin is at or past engine#2395.
- [ ] The PC Key bindings screen lists `Your army` and `Find` under "On
      the world map"; rebinding `Find` to `v` makes `v` open Find on the
      world map (a Harness test).
- [ ] `Restore defaults` is drawn when focused (the screen scrolls).
- [ ] No `engine-friction: engine#2395` comment is left.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the rows list every rebindable action once, required first.
- Integration (Harness): rebind `WorldFind`, then use it on the test
  world map (debug menu "Test world map").
- Snapshot: the Key bindings screen, scrolled.
