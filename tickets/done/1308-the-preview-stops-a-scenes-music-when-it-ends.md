---
id: "1308"
title: "The script preview stops a scene's music when the scene ends"
type: bug
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-20
---

# 1308 — The script preview stops a scene's music when the scene ends

## Context

Nick, 2026-11-20, trying the web script preview as a writer applicant
would (0766's `before-sending.md`, section 2): "the music keeps playing
after a scene finishes which seems like a bug, like on the menu screen it
keeps playing music in the writer tryout".

A scene's `@music` line starts its music in the game's dialogue screen. In
the game the next screen plays its own music; in the preview (ADR-0125)
the next screen is the preview's tabs, which play none, so the scene's
music kept playing over the scene list.

This comes before the writer trial's first packets go out (0766), and on
its own PR so the Story kit can push it to the preview page from `main`.

## Nick input

None.

## Scope

**In:**
- When the scene playing in the preview ends and the preview is back on
  its tabs, the music stops (`StoryPreview::update`).

**Out:**
- Opening the tabs over a scene that is still playing (it keeps its
  music, as it is paused, not over).
- The game's own screens.

## Implementation steps

1. Expected size: about 10 production lines.
2. In `StoryPreview::update`, note whether a scene played before the
   engine's preview updates; if one did and none plays after, ask the
   audio to stop the music.
3. Harness test: a scene with `@music`, played to its end, asks for the
   music once and stops it once.
4. Run the `run-gates` skill.

## Acceptance criteria

- [x] A scene played to its end stops its music; nothing stops it while it
      plays (test).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Harness integration test (`crates/ui/tests/it/story_preview.rs`).

## Completion notes

`StoryPreview::update` stops the music when the scene that was playing
before the engine's preview updated is gone after it (7 production
lines). `leaving_a_scene_stops_its_music` plays a scene with `@music
village` to its end: the music is asked for once and stopped once, and not
stopped while the scene plays. Backing out of a scene with Cancel isn't
tested: Cancel doesn't leave a scene in the preview. Live on the preview
page after the next Story kit run from `main`.
