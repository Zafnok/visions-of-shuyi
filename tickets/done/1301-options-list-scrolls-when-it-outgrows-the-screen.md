---
id: "1301"
title: "The PC's Options list scrolls when its rows outgrow the screen, so its message always shows"
type: bug
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0923"]
nick_input: none
completed: 2026-11-19
---

# 1301 — The PC's Options list scrolls when its rows outgrow the screen

## Context

Found by 0923 (2026-11-18). The PC's Options screen is drawn by the
engine's glyph skin (`shuyi_ui::screens::options::glyph`): its panel
grows with the rows and rises toward the top so the message under it
("Tips will show again", "The game mode can be changed at Preparations,
before a battle", …) stays above the help line. 0923 added the Range
look row, and the engine let the panel rise one row higher (to row 0,
`Game 0923` engine PR) to keep the message in the build without
pictures. **With the bought pictures** the game shows two more rows (Map
look and Map zoom), so the panel there runs into the help line, and the
message has no row left at all: a player who resets the tips or tries
the game mode in a battle sees no message. This was already so before
0923 (the picture build had one row fewer then, still too many).

The Playdate's Options skin already scrolls its list.

## Nick input

None: no rows or text change, only that the list fits (*the agent's
starting rule*: the PC's list scrolls as the Playdate's does, keeping
the focused row in view).

## Scope

**In:** the PC's Options list keeps the panel above the message and the
help line whatever the number of rows, scrolling to keep the focused row
in view, with a mark that more rows lie above or below; likely an engine
change to its glyph options skin (`engine-change` skill; ask Nick before
an engine ticket or PR).

**Out (do not do):** removing, merging or regrouping rows (a design
question for Nick); the Playdate's skin.

## Implementation steps

1. ~~Engine: the options glyph skin scrolls its rows when they don't fit
   between `TOP_MIN` and the message row; the engine's tests.~~ Done by
   engine #510 (ticket 3225).
2. Game: ~~move the pin~~ (done by the pin move of 2026-11-19, engine
   #510, engine ticket 3225); the options integration tests with the bought
   pictures' rows (Map look, Map zoom) still show the message.

## Acceptance criteria

- [x] With every PC row shown (pictures on), the message shows under the
      panel and the help line is clear.
- [x] The focused row is always on screen as the cursor moves.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit (engine skin): more rows than fit scroll, the focused row in view,
  the message row free.
- Integration: Options with the picture rows, the mode message shown.

## Completion notes

- The engine's half (the glyph skin scrolls its rows above the message,
  with ▲ and ▼ marks, and its unit tests) landed as engine #510 (engine
  ticket 3225), and the game's pin already passes it (#770). This PR adds
  only the game's integration tests, in `crates/ui/tests/it/options.rs`,
  on a launch with the pictures (Map look and Map zoom) in a battle
  (Game mode and Tutorial too): `with_pictures_the_focused_row_is_always_on_screen`
  walks every row and round again, the focused row always on screen,
  the help line whole, and the list scrolls at least once;
  `with_pictures_the_messages_show_above_the_help_line` shows the game
  mode's message and Reset tips' with the help line clear.
- Found: a focused value's `◄` covers the last letter of the longest
  label ("Voice on hover in the lo◄ On ►"), with or without pictures.
  Filed as 1302; the walk matches each label's first 20 letters until
  then.
- Gates: run locally were fmt, clippy, the workspace's tests, ticket-lint,
  check-mentions, check-leaks, pr-size and typos; the target builds,
  check-keys/check-text, doc, deny and machete can't see a test-only
  change and are left to CI.
- No gameplay rules decided.
