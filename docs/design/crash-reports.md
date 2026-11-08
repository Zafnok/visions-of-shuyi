# Crash reports: what the player sees

Decided: 2026-11-06
Source: ticket 1245

When the game crashes, it saves a crash report on the player's machine
(nothing is ever sent anywhere; `docs/session-records.md`, *Crash
reports*). This is what the player sees at the next start.

## The question

Asked with mockups of the title screen, four options: nothing in game (the
report is only a file for bug reports, as most console games and Fire
Emblem do); a short one-time notice under the title's menu; the notice
with the report's path on screen (as Stardew Valley or Factorio do); the
notice plus a "Show report" menu item that opens the folder (as Minecraft's
launcher or Paradox games do).

### Nick's words

> Short notice

## Rules (Nick fixed)

1. **A short notice on the title, once.** At the first start after a
   crash, the title screen shows under its menu, where its other notices
   go (a dark band, warning color):

   ```
   The game closed unexpectedly last time. A report was saved.
   ```

2. **No path and no menu item.** Where the reports are kept is in the
   README-level docs (`docs/session-records.md`), not on screen.
3. **Once per crash.** The start after that says nothing, until the game
   crashes again.

## Claude's starting rules

- The notice goes at the next key press, like the title's other notices
  (*Continue* failing). Where "Press any key or button" shows first, the
  notice waits for the menu and goes at the key press after that.
- Only the title says it: a start straight onto a dialogue scene (the
  script writers' `--scene`) doesn't.
