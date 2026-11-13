---
id: "1266"
title: A browser that refuses storage shows the save error and the web build keeps running
type: infra
milestone: Later
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
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

- [ ] With `localStorage` blocked and with it full, the web build starts,
      plays and shows the storage error on saving, in the browser pane.
- [ ] Each place fixed has a test.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- A test per place fixed, with a storage that answers
  `StorageError::Backend`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
