---
id: "1309"
title: "The script preview stops a scene's music when Cancel leaves it"
type: bug
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-20
---

# 1309 — The script preview stops a scene's music when Cancel leaves it

## Context

Nick, 2026-11-20, on the live script preview after 1308 (PR #786): "the
music issue persists. Ensure it also stops when they back out via cancel
(D)".

The Story kit ran from `main` at the 1308 merge (a60bf1f) three minutes
after it merged, so the page Nick tried had 1308. 1308 stops the music
only when a scene ends. Cancel in a scene doesn't end it: the engine's
preview (`DialoguePreview::play_frame`) shows its tabs over the scene,
which stays loaded and paused (`playing()` is still the dialogue screen),
and Cancel on the tabs resumes it from the same text box. So backing out
with Cancel left the scene's music playing over the tabs, and it was
never stopped once a writer went on to the scene list.

Resuming keeps the same dialogue screen, which plays a `@music` line only
when it reaches one, so it would not ask for its music again by itself.

This comes before the writer trial's first packets go out (0766), on its
own PR so the Story kit can push it to the preview page from `main`.

## Nick input

None.

## Scope

**In:**
- The music stops whenever the scene leaves the screen: it ends, or
  Cancel brings the tabs over it (`StoryPreview::update`).
- A paused scene shown again (Cancel on the tabs) plays the music it had
  set; a scene played again from the scenes plays its own.

**Out:**
- The game's own screens.
- The engine's preview (no engine change: the game reads what shows).

## Implementation steps

1. Expected size: about 20 production lines.
2. `DialogueScreen::music`: the music the scene set last.
3. In `StoryPreview::update`, note whether a scene is on screen (one
   plays and the tabs aren't open) before and after the engine's preview
   updates: stop the music when it leaves, play the scene's music again
   when a paused one comes back.
4. Harness test: Cancel mid-scene stops the music once, Cancel again
   plays it again, playing it again from the scenes plays it again, and
   playing it to its end stops it.
5. Run the `run-gates` skill.

## Acceptance criteria

- [x] Cancel mid-scene stops its music, once (test).
- [x] Resuming the scene, or playing it again, plays its music again
      (test).
- [x] A scene played to its end still stops its music (test).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Harness integration test (`crates/ui/tests/it/story_preview.rs`).

## Completion notes

What Cancel did: in a scene, Cancel brings the engine's tabs over it
(`DialoguePreview::play_frame` sets `open`) and leaves the scene loaded,
so `playing()` stays the dialogue screen and 1308's check never fired;
Cancel on the tabs resumes the same screen, which plays its music only at
a `@music` line, so it would not have asked for it again either.

The fix (18 production lines): `StoryPreview::update` compares whether a
scene is on screen (one plays and the tabs aren't open) before and after
the engine's preview updates. Leaving it stops the music; a paused scene
coming back plays the music it had set (`DialogueScreen::music`, new); a
scene started afresh plays its own. No engine change.

`leaving_a_scene_stops_its_music` now: Cancel mid-scene stops `village`
once (and not again on the next frame), Cancel resumes it and plays it
again, played again from the scenes it plays again, and played to its end
it stops.

Checked on the web too (Nick, 2026-11-20: "1308 also took no effect"): a
local debug web build of the `story-trial` edition, driven in a browser
with the trial's `ch01.dlg` and a one-line scene, with the page's Web
Audio sources traced. Cancel disconnected the looping track, Cancel again
started it again, and each scene played to its end disconnected it. The
headless Harness with the real `ch01.dlg` sends the platform a `Stop` for
`village_home` and `dungeon_tense` at each scene's end, so 1308's path
worked on the web as well; what still played was the Cancel path. The
Story kit pushed 1308 (a60bf1f) to itch.io at 00:26 UTC on 2026-11-21
(19:26 Eastern on the 20th): a page opened before then keeps the old build
until it is reloaded.
