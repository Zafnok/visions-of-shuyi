---
id: "1358"
title: "Footsteps sound at the same frame under every map skin"
type: bug
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-25
---

# 1358 — Footsteps sound at the same frame under every map skin

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

- [x] A test: every sound of the scripted battle is asked for on the same
      frame under every map skin, zoom and light **that walks at the same
      speed**; across walking speeds, the same sounds in the same order
      (amended by Nick's answer, 2026-11-25: see the notes).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: `map_skin.rs` compares each sound's frame.

## Lesser targets

None: the Playdate has one map skin.

## Completion notes

- **Two causes.** (1) The walk's pace: each map skin walks at its own
  speed (glyphs 12 tiles a second, sprites 6, 8 with Fast animations;
  Nick, 2026-10-04), and a footstep plays as the unit enters a tile, so
  the first `step_foot` falls 5 frames later under sprites (1/6 s
  against 1/12 s). Asked again (ask-nick, 2026-11-25), Nick chose
  **"Keep the two speeds"**: the footsteps keep following the walk on
  screen, recorded in `docs/design/look-and-feel.md`. (2) A frame the
  skin's layout takes, found once the test compared frames: an enemy's
  action skipped its quarter-second camera pan when the camera didn't
  move, and whether it moves depends on how many tiles the skin shows
  (sprites at 1× against 2× parted by 15 frames in the enemy phase).
  Fixed in `ai_phase.rs`: the pan always lasts its time.
- **The test** (`map_skin.rs`, `the_skin_never_changes_the_game`) now
  reads `Harness::audio_log`. Runs that walk at the same speed (glyph at
  both zooms and at dusk; the three sprite skins at both zooms and
  `test_auto` at dusk) ask for every request on the same frame, the
  fades' volume steps included; every run asks for the same sounds in
  the same order. It checks that both speeds were played and each
  matched by another run.
- **The agent's starting rule** (pacing, ticket 0502's ~0.25 s pan): an
  enemy whose unit is already in view still waits the pan's quarter
  second, with the cursor on it from the start, before it walks. Before,
  it went straight to the 0.2 s mark. Each enemy now has the same beat
  whatever the look and zoom; the Act 1 playtest (1064) can judge the
  feel.
- **Size:** production estimate about 5 lines (one duration and the
  cursor's rule, judged by `AiAction::new`); measured 3 added (under
  2×). PR about 250 changed lines.
