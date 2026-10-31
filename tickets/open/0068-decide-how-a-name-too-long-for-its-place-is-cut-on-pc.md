---
id: "0068"
title: "Decide how a name too long for its place is cut on the PC screens: cut short, or cut with a … mark"
type: design-decision
milestone: M3 Battle UI
model: implementation
effort: low
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0068 — How a too-long name is cut on PC

## Context

Found by 1165. When a name or a line is longer than its place, the PC
screens (the side panel, the attack forecast, the fight box, the level-up
pages) stop it at the last cell that fits, with no mark. The class
name "Frost Elemental" shows in the forecast as **`Frost Element`**,
which reads as a different, complete name.

The Playdate screens, and the engine's own screens (options, Key
bindings), already end a cut line with `…`: the same place would show
`Frost Element` with its last letter replaced by `…`.

Nothing in `docs/design/look-and-feel.md` says which the PC screens do.
1165 kept the PC look as it is (no change the player sees was in its
scope).

## Nick input

Decision, with the `ask-nick` skill. Options to show, each rendered on the
real forecast with "Frost Elemental":

1. **Cut with `…`**, as on the Playdate and in most PC software menus
   (a file name too long for its column in Windows Explorer).
2. **Cut short, no mark** (`Frost Element`), as today.
3. **Never cut a name**: names are kept short enough to fit every place
   (a content rule, checked by a test), as the GBA Fire Emblem games'
   names fit their fixed name fields; only other text is cut.
4. Describe your own.

## Scope

**In:**
- The decision, recorded in `docs/design/look-and-feel.md`.
- If it changes the PC look: a follow-up ticket that moves the PC glyph
  skins' `print_clipped` (`screens/battle/glyph.rs`) onto the engine's
  `print_cut`, and the other hard cuts in the glyph skins with it.

**Out (do not do):**
- The change itself (the follow-up ticket's).

## Acceptance criteria

- [ ] `look-and-feel.md` says how a too-long name or line is cut on PC.
- [ ] A follow-up ticket exists if the PC look changes.

## Completion notes

*(Filled in by the session that completes the ticket.)*
