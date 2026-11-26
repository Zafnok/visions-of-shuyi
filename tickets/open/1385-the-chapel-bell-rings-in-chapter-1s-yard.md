---
id: "1385"
title: "The chapel bell rings in Chapter 1's yard scene"
type: feature
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["1383", "1335"]
blocks: ["1064"]
nick_input: decision
completed:
---

# 1385 — The chapel bell rings in Chapter 1's yard scene

## Context

In Chapter 1's first scene (`ch01_intro`, the yard at dawn) the chapel
bell starts an hour early, and that is the turn of the scene. Asked
whether a story moment should change the sound, Nick (1324, 2026-11-26):
"sure, we can have the bell ring. but dont let it block implementing the
rest of the ambience now." The example he answered was the dawn chorus
stopping when the bell rings; 1324 records both (the private
repository's `story/design/audio.md`, *Chapter 1's ambience*).

The game has no directive for a sound effect at a point of a scene, and
`@ambience none` is 1383's. A scripted bell is not a passing call (1344:
random sounds at random gaps); it plays once, at one line.

## Nick input

**Decision** (`ask-nick`, one short round): which bell, on a listening
page over the yard's frame and its bed (1335), with a slider. Start from
the two heard in 1322: TollBell.wav (daytripper,
[73678](https://freesound.org/s/73678/), CC0, one toll, heard softened)
and Bells_distant_in_town.wav (ivolipa,
[266525](https://freesound.org/s/266525/), CC0, "too loud and long for a
call", which may suit a story moment), plus one or two new CC0 bells,
each checked for hum or buzz first. Also ask: one toll or a few, and
whether the dawn chorus stops with it (recorded as yes; confirm).

## Scope

**In:**
- A directive that plays a sound effect once at that point of a scene
  (`@sound <cue>`, a cue of `audio.ron`'s sound group), checked like
  `@music`.
- `ch01_intro`: the bell at the line where it starts, and `@ambience
  none` (1383) there if confirmed. Directives only, no words changed
  (allowed during the dialogue pause, the `story-writing` skill).
- The bell imported, levelled and credited.

**Out (do not do):**
- Passing calls (1344); other scenes' sounds; any script line.

## Implementation steps

1. Ask (above); record the answer under 1324's section in the private
   `story/design/audio.md`.
2. Add the directive like `@music` (parse, check, play once through the
   sound-effect volume).
3. Import the bell; add the directives to `ch01_intro` (the
   `content-change` skill).

Expected size: about 60–100 production lines for the directive, judged
by `@music`'s.

## Acceptance criteria

- [ ] Nick's bell pick recorded with his words.
- [ ] Playing `ch01_intro` to the bell's line plays the bell once (1332's
      audio fake) and, if confirmed, stops the bed.
- [ ] An unknown cue in the directive is a content error with file and
      line.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the directive's parse and check.
- Integration: the scene plays the bell at its line.

## Lesser targets

The Playdate plays sound effects; the bell plays there too.

## Completion notes
