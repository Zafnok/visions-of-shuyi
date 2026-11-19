---
id: "1301"
title: "The PC's Options list scrolls when its rows outgrow the screen, so its message always shows"
type: bug
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0923"]
nick_input: none
completed:
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

- [ ] With every PC row shown (pictures on), the message shows under the
      panel and the help line is clear.
- [ ] The focused row is always on screen as the cursor moves.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (engine skin): more rows than fit scroll, the focused row in view,
  the message row free.
- Integration: Options with the picture rows, the mode message shown.

## Completion notes

*(Filled in by the session that completes the ticket.)*
