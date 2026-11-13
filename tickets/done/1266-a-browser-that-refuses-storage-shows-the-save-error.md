---
id: "1266"
title: A browser that refuses storage shows the save error and the web build keeps running
type: infra
milestone: Later
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-13
---

# 1266 — A browser that refuses storage shows the save error and the web build keeps running

## Context

Filed by the engine pin move of 2026-11-13 (`docs/engine/pins.md`,
engine PR #386, engine ticket 3033). The web build's saves now go
through the engine's own `storage.js` in place of `quad-storage.js`.
When `localStorage` throws (storage turned off, a sandboxed frame such
as an embed, the quota full) the engine answers
`StorageError::Backend` ("localStorage is turned off or blocked in this
browser", "localStorage is full", "localStorage failed") where it threw
through the frame before.

The game shows a storage failure as `save.error.storage` with the
error's text (`crates/ui/src/save.rs`, `SaveError::Storage`). Nobody
has seen that happen in a browser: the profiles' set-up at start
(`profiles::keep_set_up`), the settings and the saves each read or
write storage, and a failure at start may stop the game before any
screen shows the error.

## Nick input

None: the error's wording is `save.error.storage`'s, unchanged.

## Scope

**In:**
- In the browser pane, with a release web build (`cargo shuyi web
  --release`): block `localStorage` (a page script that makes it throw,
  or the browser's setting), start the game, start a new game, save,
  change a setting. Then fill the quota and save.
- Every failure shows the game's error and the game keeps running; any
  place that stops the game or shows nothing is fixed so it shows
  `save.error.storage` (or the setting's own error) and goes on.
- A test over `MemoryStorage`'s failing form (or a fake `Storage` that
  answers `StorageError::Backend`) for each place found.
- What was seen in the completion notes, with the browser and how the
  storage was blocked.

**Out (do not do):**
- New player text or a new screen: if a place needs words the game
  doesn't have, a ticket for Nick (`ask-nick`).
- The Playdate's and the desktop's storage.

## Implementation steps

1. Read the engine's `crates/cli/web/README.md` section on `storage.js`
   and `crates/app/src/storage/local_storage.rs`.
2. Run the checks above in the browser pane.
3. Fix and test each place that doesn't show the error.

## Acceptance criteria

- [x] With `localStorage` blocked and with it full, the web build starts,
      plays and shows the storage error on saving, in the browser pane.
- [x] Each place fixed has a test. (No place needed a fix; the walk has
      one test instead, below.)
- [x] All gates in the `run-gates` skill pass.

## Tests required

- A test per place fixed, with a storage that answers
  `StorageError::Backend`.

## Completion notes

**Nothing needed fixing.** Every place a player saves or keeps an option
shows its error and the game goes on, blocked or full.

**How it was checked.** `cargo shuyi web --release` at engine pin
`a434b973`, served from a copy of `dist/web` with one script added first
in `index.html`, in the Claude desktop app's browser pane (Chromium 156,
Windows 11). Frames were driven by a `MessageChannel` loop because the
pane was hidden; the toasts were read from the canvas (`toDataURL` right
after a frame).

- **Blocked:** the script made `window.localStorage` throw a
  `SecurityError` (as storage turned off or a sandboxed frame does), and
  the origin's storage was cleared first, so this was a first launch.
  The console warned that profiles, key bindings and settings couldn't
  be read. Seen: the title; the layout picker; New Game, game mode,
  "How familiar are you?", voices, lead; the opening scenes; the first
  battle. The map menu's Suspend showed `Save storage error: localStorage
  is turned off or blocked in this browser` and the battle went on.
  Options: Text speed changed and kept showed `Couldn't save: the change
  lasts until you quit`.
- **Full:** `localStorage` filled with junk keys until `setItem` threw
  `QuotaExceededError` (about 52 MB in this browser), then the page was
  reloaded, so the game started full. Same walk; the console warned
  `profiles: can't be saved: storage error: localStorage is full`.
  Suspend showed `Save storage error: localStorage is full` and the
  battle went on.
- No panic, no error in the console, and the game never stopped.

**Places that show nothing, on purpose.** None of these is a save the
player asked for, so they stay quiet, and each tries again later:
the engine's layout picker (`shuyi_ui::screens::layout_picker`: the
layout is used this session and asked again on the next launch), the
profile's set-up mark (`profiles::keep_set_up`, tried again each time
the title shows), the title's backdrop, the tips seen, the familiarity
answer (asked again each new game). With storage blocked, the layout
picker and "How familiar are you?" come back each launch: that is what a
game that can't remember does, not a bug.

**Test.** No place needed a fix, so there is no test per fix. Instead
`save::a_storage_that_refuses_everything_still_plays_and_says_why_it_cant_save`
(`crates/ui/tests/it/save.rs`) plays the browser walk over a `Storage`
whose every call answers the engine's `StorageError::Backend` text: the
layout picker, New Game to the battle, Suspend showing the error with
the battle unchanged, the win, and a slot save showing the error with
the slots still up. The Options error already had its test
(`screens/options/tests.rs`, `ReadOnly`).

**Deviation.** The slot save after a victory was checked by the test, not
in the browser: reaching it there means winning the first battle by key
presses in a hidden pane. It goes through the same `save::write` and
`SaveError::Storage` text as the suspend save seen in the browser.

**Follow-up.** A long message on the battle's message line runs into the
toggles: the blocked text was printed over `w danger zone: OFF ·`, its
last letter touching `Shift+Space auto-end: OFF`. Added to open ticket
0445 (which decides that row's layout with Nick) rather than a new
ticket.

**Starting rules:** none.
