# ADR-0072: Options changes are previewed, then kept or put back on leaving

- **Status:** Accepted
- **Date:** 2026-10-15
- **Related tickets:** 0839 (this), 0058, 0836
- **Amends:** [ADR-0050](0050-player-settings.md) (saving at once) and
  [ADR-0069](0069-player-profiles-a-prefix-per-profile-in-one-storage.md)
  (its *Saving at once is gone* note, which waited for this ticket)

## Context

ADR-0050 had `Ctx::change_settings` save the settings on every change.
Nick (PR #264, `docs/design/options.md`, *Keeping changed options is the
player's choice*) wants overwriting options to be deliberate: a change on
the Options screen shows at once, but is kept only when the player says so
on leaving (Keep / Discard / Back, cursor on Back). Settings changed
elsewhere (the Zoom map key, the battle's Auto-end key, the window's
fullscreen shortcut, the layout picker) keep saving at once.

## Decision

- `Ctx` keeps two copies of the settings: `settings` (in use: what every
  screen reads, what `FrameOutput` takes the volumes and fullscreen from)
  and `stored_settings` (what the storage holds: loaded, or last written).
- `preview_settings(change)` changes the settings in use and applies what
  changed (map skin, zoom, palette) without saving.
  `change_settings(change)` is `preview_settings` then a write: callers
  outside Options are unchanged.
- `keep_settings()` writes the settings in use if they differ from
  `stored_settings`. `restore_settings(&to)` puts the settings in use back
  to `to` (the layout aside) and writes only if the storage then holds
  something else, which happens only if another change was saved meanwhile.
- `choose_layout` writes `stored_settings` with the new layout, so picking
  a layout from Options doesn't save the changes not kept yet.
- The **Options screen** takes a copy of the settings on its first frame.
  Cancel with anything different from that copy (the layout aside) opens
  the question; Keep calls `keep_settings` and closes, Discard calls
  `restore_settings(copy)` and closes, Back stays. `Restore defaults` is a
  previewed change like the rows'. A failed Keep stays on the screen and
  says it couldn't save.
- The question is part of the screen's view (`OptionsView::leaving`, a
  `LeaveView` of `ChoiceView`s) and painted by the glyph skin (ADR-0054).

## Consequences

- A change not kept is lost if the game closes while Options is open (the
  same as Discard).
- Every caller that should save at once keeps calling `change_settings`;
  only Options previews. A future screen that edits the profile (the
  Profile screen's name box, `options.md`) can use the same two calls.
- Tests that change settings in Options and relaunch must leave with Keep
  first.

## Alternatives considered

- **The screen holds the working copy and `Ctx` only saves on Keep** —
  every screen and `FrameOutput` read `ctx.settings()`, so a change made
  only on the screen's copy wouldn't show or sound at once.
- **Snapshot in the screen, Discard writes the snapshot** — writes on every
  Discard even when nothing was saved, and the ticket asks that Discard
  saves nothing.
