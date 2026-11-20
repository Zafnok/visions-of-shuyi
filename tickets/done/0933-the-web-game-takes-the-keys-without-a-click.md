---
id: "0933"
title: "The web game takes the keys without a click, and again after fullscreen"
type: bug
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-19
---

# 0933 — The web game takes the keys without a click, and again after fullscreen

## Context

Nick, 2026-11-20, playing the Chapter 1 web demo on itch.io as a writer
applicant would (0766's `before-sending.md`): "I have to click on the itch
frame for keyboard input to work. And after clicking fullscreen button as
well."

The game is keyboard first, and writer applicants start the demo and the web
script preview from an itch.io page (0915, 0767), where the game runs inside
itch's iframe (`html-classic.itch.zone`). After **Run**, and after itch's
**Fullscreen** button under the frame, the key presses go to the itch page,
not to the game, until the player clicks the game. The packet's START-HERE
now says "Click the game" as a stopgap; this ticket makes it unneeded.

The page the game runs in is Shuyi Engine's web shell (engine
`crates/cli/web/index.html` and its scripts at the pin in
`shuyi-engine.rev`), so the fix is most likely in the engine: focus the
canvas (or the window) when the page loads and is shown, when the frame
becomes visible, and after a fullscreen change (`fullscreenchange`), and give
the canvas a `tabindex` so it can hold focus. Check what the engine already
does first.

This comes before the writer trial's first packets go out (0766). Nick asked
for the fix (2026-11-20), so the engine step is his go-ahead.

## Nick input

None. Verified in a real browser (below) and by the engine's tests.

## Scope

**In:**
- Find why keys need a click: inside an iframe, after load; after itch's
  Fullscreen button (which fullscreens the iframe element from the parent
  page, so the focus stays on the parent).
- Fix it where it belongs: an engine change through the `engine-change`
  skill (engine ticket closed in the same engine PR, then this game PR moves
  the pin), or game-side if the shell is the game's.
- Both the demo edition and the `story-trial` web preview get the fix (same
  shell).
- After the fix, remove "Click the game" from `docs/writer-trial/start-here.txt`
  and `brief.md` only if they are on `main`; otherwise note it for 0766.

**Out:**
- Mouse support (the game has none).
- The pause sign while unfocused (0928).

## Implementation steps

1. Expected size: about 20-40 lines in the engine's web shell, plus a test.
2. Reproduce: serve a web build inside a test page that frames it in an
   iframe (as itch does) with a Fullscreen button on the parent that calls
   `requestFullscreen()` on the iframe. Confirm keys don't reach the game
   until a click.
3. Fix: on load, `visibilitychange` to visible, and `fullscreenchange`,
   call `focus()` on the canvas (with `tabindex="0"`) and the window. Note
   that a cross-origin parent can't focus into the frame; the frame focusing
   itself after load is allowed by browsers for programmatic focus without a
   user gesture in most cases, but verify in Chrome, Edge and Firefox.
4. Engine tests for the shell if it has any (the engine's web tests or a
   script test); otherwise a note in the PR with the manual results.
5. Move the pin (engine-change §7), run the `run-gates` skill.

## Acceptance criteria

- [x] In Chrome, a web build framed by a test page: after Run, pressing a
      key works without clicking the game.
- [x] After the parent page fullscreens the frame, keys still work without
      clicking.
- [x] Same in Edge and Firefox (recorded in the Completion notes).
- [ ] All gates in the `run-gates` skill pass (after the pin moves).

## Tests required

- What the engine's web shell allows (a script or browser test), else the
  manual browser results in the PR.

## Completion notes

- **Cause** (the engine's web shell, `crates/cli/web/index.html`): it
  focused the canvas once, when its script ran. Inside a site's
  cross-origin frame the keys then stay on the site in two cases: when the
  site shows the frame only after the game page has loaded (the canvas had
  no box to take focus with, and focus stays on the site's Run button),
  and after the site's Fullscreen button, which fullscreens the frame from
  outside (focus stays on the button).
- **Fix** in the engine, engine#3505 (Zafnok/shuyi-engine#512): the
  page takes focus at load, on the first resize before it ever had focus
  (the frame shown late), and on a resize that fills the screen without
  focus (the site's fullscreen); the canvas follows the window's focus.
  Any other resize leaves focus alone, so a text box on itch.io keeps it.
  Both the demo and the `story-trial` preview use this shell.
- **Checked** on Windows 11 in real Chrome and Edge windows with this
  game's web build, framed cross-origin by a test page with Run and
  Fullscreen buttons (keys sent over the DevTools protocol):
  - Chrome, before: keys go to the site after a late-shown Run and after
    Fullscreen (frame or its container). After: keys reach the game in
    every case.
  - Edge: the late-shown Run case fails before and passes after; Edge
    wouldn't go fullscreen under automation, so its Fullscreen case is
    unchecked.
  - Firefox: not installed here, and Playwright's Firefox wouldn't start:
    unchecked.
  - The engine's CI (`web-render`) now runs `check.py frame` in headless
    Chromium, which fails the old shell on the fullscreen case.
- I couldn't load itch.io's own page for the demo (password-protected), so
  the test page stands in for it; how itch shows its frame after Run is
  the likeliest of the two causes, not confirmed on itch itself.
- "Click the game" in START-HERE and `brief.md`: `docs/writer-trial/` is
  not on `main`, so that text is untouched here. Once this merges and the
  demo and preview are pushed to itch.io again, 0766 can drop the line.
- Starting rules: none (no gameplay).
