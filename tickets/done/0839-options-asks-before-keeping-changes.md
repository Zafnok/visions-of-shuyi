---
id: "0839"
title: Leaving Options asks whether to keep the changes
type: feature
milestone: M7 Chapter 1 & game flow
model: reasoning
effort: medium
status: done
blocked_by: []
nick_input: sign-off
completed: 2026-10-15
---

# 0839 — Leaving Options asks whether to keep the changes

## Context

Nick on PR #264 (ticket 0058), 2026-10-14: "ensure it's deliberate when a
player overwrites their options ... if they don't want to overwrite, let
them make that choice." Recorded in `docs/design/options.md`, *Where
options are kept* (*Keeping changed options is the player's choice*).

Today `Ctx::change_settings` (`crates/ui/src/screen.rs`, ADR-0050) saves
the settings the moment a row changes. From now on a change shows at once
but is written only when the player keeps it.

## Nick input

**Sign-off:** after merge, change a speed in Options, back out, pick
Discard, and see the old speed back; do it again and pick Keep, reload the
page, and see the new speed.

## Scope

**In:**
- The Options screen keeps a copy of the settings as they were when it
  opened. A change still applies at once (the player sees and hears it).
- Leaving with anything changed asks `Keep these changes?`: **Keep**
  (save), **Discard** (put back the copy, including volumes and
  fullscreen), **Back** (stay). Cursor on Back. Leaving with nothing
  changed asks nothing. Every place Options opens: the title, the map menu,
  the Preparations tab (leaving the tab counts as leaving).
- `Restore defaults` still asks its own yes/no; afterwards the defaults
  are a change like any other, kept or discarded on leaving.
- `Reset tips` keeps working as it does (it asks first already).
- Settings changed outside the Options screen keep saving at once: the
  Zoom map key (`options.md`, *the agent's starting rule*) and the hand-layout
  picker.
- Words in the language files (American English; `test` pack too); keys
  named through the keymap (`keyboard-input` skill).
- An ADR amending ADR-0050's save-at-once rule (`write-adr` skill), or
  folded into 0836's ADR if that lands first.

**Out (do not do):**
- Profiles (0836, 0837).

## Implementation steps

1. A way to change settings without saving and to save or restore later
   (e.g. `Ctx` keeps an unsaved working copy; the Options screen asks it
   to keep or restore).
2. The question on leaving (view + glyph skin, ADR-0054), with Back
   focused.
3. Tests.

## Acceptance criteria

- [x] A change shows at once; leaving with changes asks, cursor on Back
      (tests).
- [x] Keep saves; Discard puts every value back, volumes and fullscreen
      included, and saves nothing; Back stays (tests).
- [x] Leaving with nothing changed asks nothing (test).
- [x] The same from the title, the map menu and the Preparations tab
      (tests).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the working copy keeps/restores; `FrameOutput` volumes and
  fullscreen follow a Discard.
- Snapshot / integration: the question's view; scripted change → leave →
  each answer, checking `MemoryStorage` was or wasn't written.

## Completion notes

- `Ctx` keeps the settings in use and the settings as stored apart:
  `preview_settings` changes without saving, `keep_settings` saves,
  `restore_settings` puts a copy back (writing only if something else was
  saved meanwhile). `change_settings` is preview + save, so the Zoom map
  key, the Auto-end key and the window's fullscreen shortcut save at once
  as before; `choose_layout` saves only the layout. ADR-0072 records it
  (amends ADR-0050 and ADR-0069's note that waited for this ticket).
- The Options screen copies the settings on its first frame. Cancel with
  anything changed opens `Keep these changes?` (Keep / Discard / Back,
  cursor on Back), as part of the screen's view (`OptionsView::leaving`)
  painted by the glyph skin. The same screen opens from the title, the
  map menu and the Preparations tab, and each is tested.
- `Restore defaults` is now a change like any other; `Reset tips` is
  unchanged. Existing tests that relaunched after a change now keep it
  first.

**the agent's starting rules** (the design docs were silent):
- Cancel on the question is Back (stay).
- The three answers sit on one line, Keep, Discard, Back; the cursor's
  keys move between them and stop at the ends.
- If Keep can't save, the screen stays open and says it couldn't save.
- Closing the game with Options open and changes not kept loses them
  (as Discard would).
- Discard puts back everything as it was when Options opened, including
  a fullscreen change made with the window's shortcut while it was open.

