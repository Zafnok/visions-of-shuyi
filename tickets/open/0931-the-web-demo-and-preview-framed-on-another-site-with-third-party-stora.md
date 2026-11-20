---
id: "0931"
title: "The web demo and the script preview, framed on another site with third-party storage blocked, start, play and say when a save fails"
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0931 — The web demo and the script preview, framed on another site with third-party storage blocked, start, play and say when a save fails

## Context

Filed from the writer trial's test audit (0766, 2026-11-19). The trial
sends applicants the Chapter 1 web demo (0914, on a password-protected
itch.io page, 0915) and the web script preview (0767, 0771). Both are
played in itch.io's page frame: "The game itself is served from an
`html-classic.itch.zone` address in the page's frame"
(`docs/releasing.md`, *The demo's itch.io pages*). So the game runs in a
frame from **another site**. Browsers limit storage there:
blocking third-party cookies is a setting in Chrome, Edge and Firefox,
and Chrome's Incognito windows block them by default. A blocked
frame's `localStorage` is expected to be refused (it throws) or
partitioned. Nobody has seen which, or what the game does then.

1266 (PR #671) checked a browser that refuses storage, but only by
simulation. In the desktop app's browser pane, a page script made
`window.localStorage` throw a `SecurityError`. The game was on its own
address, not in a frame, and the frames were driven by a `MessageChannel`
loop because the pane was hidden. It found every save the player asks
for shows its error and the game goes on, and a long error once
ran into the battle's toggles row. 0445 (PR #742) has since moved that
error into a warning box at the map's bottom left
(`MessageView::warning`, `crates/ui/src/screens/battle/view.rs`). No
browser has shown that box.

What a real block may do differently from 1266's script is unchecked:
- the engine's `storage.js` (engine PR #386, ticket 3033);
- the crash reports kept in `localStorage` (`docs/session-records.md`);
- the pack streamed in at start (1298).

The trial brief also tells applicants to Suspend a battle to keep it
(`docs/writer-trial/brief.md` on the 0766 branch). Whether that can work
in their browser is what this ticket finds out.

## Nick input

None. What the game should *tell* a player whose browser keeps nothing
is Nick's. It is the decision ticket filed with this one (0932, "What
the web game shows when the browser keeps nothing"). Either order
works: this ticket adds no player text, and its notes are input for
that question.

Either order with 0774 (the web demo and preview keep their saved
settings apart from the playtest page). Today the demo, the preview
and the playtest page all save under the storage name `tactical-rpg`
(`crates/app/src/app_info.rs`). If 0774 is done, the demo has a
storage name of its own: the walk checks the `localStorage` keys under
that name (read it from `app_info.rs` or 0774's completion notes), and
the notes say which name was checked.

## Scope

**In:**
- A local stand-in for itch.io's page: a small HTML page served on one
  site that frames the web build served from another. `http://localhost:<a>`
  and `http://127.0.0.1:<b>` are different sites. Use a 1280×720 frame
  with fullscreen allowed, as `docs/releasing.md` sets the itch.io embed.
  Keep the page and its server script in the scratchpad, not the
  repository.
- The two builds, made as `docs/releasing.md` says: the demo edition for
  the web, and the `story-trial` preview for the web. Keep MSYS2's
  `mingw64/bin` off `PATH` while making a local release web build (a
  local wasm-opt stalls for an hour).
- These browsers, each with a **throwaway profile** in the scratchpad.
  Never the user's own profile or settings.
  - Chrome (or Edge if Chrome isn't installed), started with
    `--user-data-dir=<scratch>`: once with `--incognito` and once in a
    normal window with third-party cookies blocked in that profile.
  - Firefox, if installed, with `-profile <scratch>` and
    `network.cookie.cookieBehavior` set to `1` in that profile's
    `user.js`.
  - Drive them over the browser's remote-debugging port, from a script
    kept in the scratchpad, with the window visible.
- In each browser, first record what the frame's `localStorage` does
  (under the demo's storage name: see the paragraph on 0774 under
  *Nick input*):
  - throws (and with what error);
  - works but is partitioned (kept across a reload or not);
  - works.
- The demo walk, as in 1266:
  - first launch, the layout picker, New Game, game mode, "How familiar
    are you?", voices, lead, the opening scenes, the first battle;
  - Suspend from the map menu (the warning box shows inside the map,
    clear of the toggles row, and the battle goes on);
  - change an Option (`Couldn't save: the change lasts until you quit`);
  - reload the page and note what came back.
- The preview walk: it starts to its first screen. If the harness can
  deliver a dropped file (a DevTools drag event), it plays
  `ch01.dlg` from a trial packet (`cargo xtask trial-packet --for test
  ...`). If it can't, say so in the notes.
- Fix any place that stops the game, hangs it, leaves a blank page or
  throws an uncaught error. The fix shows the game's existing error and
  goes on. A fix in the engine goes through the `engine-change` skill.
- The completion notes: per browser, its name, version and how
  third-party storage was blocked, what `localStorage` did, and what was
  seen at each step of both walks. Add any console errors.

**Out (do not do):**
- Any new player text, notice or screen, and skipping the layout picker:
  that is the decision ticket 0932 ("What the web game shows when the
  browser keeps nothing").
- The real itch.io pages: they sit behind a password the agent doesn't
  type. The local stand-in takes their place.
- Safari (it doesn't run on Windows), phones and tablets (the brief
  tells applicants to use a computer).
- The desktop's and the Playdate's storage.
- Changing `docs/writer-trial/` (0766's).

## Implementation steps

1. Read 1266's completion notes (`tickets/done/1266-...md`). Then
   read the test `a_storage_that_refuses_everything_still_plays_and_says_why_it_cant_save`
   (`crates/ui/tests/it/save.rs`), and the engine's
   `crates/app/src/storage/local_storage.rs` (in Shuyi Engine at the pin
   `shuyi-engine.rev`, not in this repository; it answers `BLOCKED = -2`
   when the browser refuses storage).
2. Make the two web builds, the framing page and two static servers (one
   per site).
3. For each browser above, start it with its throwaway profile and
   record what `localStorage` does in the frame. Then run the demo walk
   and the preview walk, taking screenshots at the warning box, the
   Options error and after the reload.
4. For each failure found, write a failing test first. Use a `Storage`
   whose calls answer `StorageError::Backend`, as 1266's test does, or
   whatever the browser threw. Then fix it.
5. Write the completion notes. If a browser kept storage (partitioned)
   and the demo worked as normal, say so: it tells the decision ticket
   which applicants are affected.

## Acceptance criteria

- [ ] The completion notes name every browser run (name, version, how
      third-party storage was blocked). For each they say whether the
      framed game's `localStorage` threw, was partitioned or worked.
- [ ] In every browser run, the framed demo reaches the first battle.
      Suspend shows its error in the warning box and the battle goes on.
      Changing an Option shows `Couldn't save: the change lasts until
      you quit` when storage is refused. No uncaught error appears in the
      console.
- [ ] In every browser run, the framed script preview starts. The notes
      say whether a dropped `ch01.dlg` played, or why it couldn't be
      dropped.
- [ ] Every place fixed has a test that failed before the fix. If
      nothing needed fixing, the notes say so.
- [ ] Nothing from the stand-in page, its servers or the throwaway
      profiles is committed.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit / integration: one test per place fixed, over a `Storage` that
  answers `StorageError::Backend` (or the error the browser gave), in
  `crates/ui/tests/it/save.rs` beside 1266's test or next to the fixed
  code.
- Property / snapshot: none, unless a fix changes a look. Then that
  screen's snapshot is updated and read.
- If no place needs a fix, no new test: the browser runs are this
  ticket's evidence (as in 1266). The existing refusing-storage test
  must still pass.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
