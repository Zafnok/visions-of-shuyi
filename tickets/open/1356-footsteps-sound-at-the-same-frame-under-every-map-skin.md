---
id: "1356"
title: "Footsteps sound at the same frame under every map skin"
type: bug
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1356 — Footsteps sound at the same frame under every map skin

## Context

Found in 1332. `crates/ui/tests/it/map_skin.rs`
(`the_skin_never_changes_the_game`) plays one scripted Quick Battle under
the glyph skin and the sprite skins and compares the sounds asked for.
Until 1332 it compared only which sounds played, in order. 1332's sound bed
fades in by setting its volume every frame, which marks the frames in that
list. It showed the first `step_foot` of the lord's walk asked for about
five frames later under the `sprite` skin (1×) than under `glyph`. The
battle states and the cursor are the same after every step, so the rules
agree; the footstep's timing doesn't. ADR-0038: graphics are a skin, and a
skin must not change when anything is heard.

1332 left the test comparing sounds without the bed's per-frame volume
steps (what it compared before), so the difference isn't checked yet.

## Nick input

None.

## Scope

**In:**
- Find why a walk's footstep falls on another frame under a sprite skin
  (the walk's pace, a frame the skin's layout takes, or the step's
  trigger reading the skin), and make it the same under every skin.
- `the_skin_never_changes_the_game` compares the frame of every sound
  (with `Harness::audio_log`), not only their order.

**Out (do not do):** the bed's fades (1332); the footstep sounds
themselves.

## Implementation steps

1. Log `h.audio_log()` frames of `step_foot` under `glyph` and `sprite`
   in the test's script; find the first frame they part.
2. Fix the cause where it is (the battle's walk or the skin), never by
   loosening the test.
3. Compare frames in the test.

## Acceptance criteria

- [ ] A test: every sound of the scripted battle is asked for on the same
      frame under every map skin, zoom and light.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: `map_skin.rs` compares each sound's frame.

## Lesser targets

None: the Playdate has one map skin.

## Completion notes
