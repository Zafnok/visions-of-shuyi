---
id: "1384"
title: "A fire in Harrowby's yard and a candle in Veyne's study"
type: content
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
blocks: ["1064"]
nick_input: none
completed:
---

# 1384 — A fire in Harrowby's yard and a candle in Veyne's study

## Context

1324 (2026-11-26) gave two of Chapter 1's scene places a fire's crackle
in their beds, but neither map draws a fire: the yard behind the rented
house (`harrowby_yard`, where the script puts Hollis "by the fire") and
Veyne Hall's study (`veyne_study`). Nick wants the picture to match the
sound: "probably a fire in the yard makes sense and the study should be
more like some light source like a candle or something. but dont let it
block the rest of the ambience." His full words are in the private
repository's `story/design/audio.md`, *Chapter 1's ambience*.

He prefers drawn animation from the bought packs over faked motion
(0096, `look-and-feel.md`): a flame that moves comes from a pack's own
frames.

## Nick input

None: what to add is decided (1324). Verified by `cargo shuyi frame-png`
pictures (kept out of the repository: bought art, ADR-0040); what's left
to judge goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- `harrowby_yard`: a small open fire (a fire pit or campfire) near where
  Hollis sits, from the bought packs, animated if a pack has the frames.
- `veyne_study`: a candle or other small light on or near the desk, not a
  hearth, from the bought packs, animated if a pack has the frames.
- Both through the `content-change` skill (the maps are private content)
  and the ADR-0038 rules (sprite items from asset files, the map scene).

**Out (do not do):**
- Sound (the beds are 1335).
- Light on the map (glow, flicker of the room's light): not decided.
- Any other map; script lines.

## Implementation steps

1. Find a fire and a candle in the bought packs (`assets-private/game/`,
   `cargo shuyi private-assets --library` for the sorted packs), and
   credit any new file (ADR-0051).
2. Add them to the two maps; render each scene with `frame-png`
   (`--scene ch01_intro`, `--scene ch01_tbc`, picture look).
3. Content pull request first, then this one moves `assets-private.rev`.

## Acceptance criteria

- [ ] `frame-png` of `ch01_intro` shows a fire in the yard; of `ch01_tbc`
      a candle or small light in the study.
- [ ] Content tests pass (every map loads; every bought file credited).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Content: the two maps load with their new items.

## Lesser targets

The Playdate shows the still frame (1314).

## Completion notes
