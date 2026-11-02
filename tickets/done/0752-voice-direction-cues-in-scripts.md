---
id: "0752"
title: "Voice-direction cues in .dlg scripts: how a line is said, for voice synthesis, not shown in game"
type: feature
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
nick_input: none
completed: 2026-11-02
---

# 0752 — Voice-direction cues in `.dlg` scripts

## Context

Nick, 2026-10-23, at the end of 0736 (the research on dialogue craft):

> "when writing up the script, I think our dialogue format should have
> cues for voice acting synthesis that do not render in the game dialogue
> viewer (i.e. Wenna talks calmly, Wenna shouts loudly) this way it can be
> mass fed into AI voice synthesis like Suno or Elevenlabs. Probably needs
> its own follow up tickets don't need to derail this ticket anymore."

Today a line's delivery lives only in its punctuation and its portrait
expression (0736: "Punctuation is the voice acting"). A voice tool gets
the words and the speaker's voice and has to guess the rest. Generated
voices are planned (`docs/design/voices-languages-and-script.md`; the
tool is decided in **0043**, built in **0721**, Chapter 1 voiced in
**0722**), and a hired cast may replace them later (ADR-0046 rule 10), so
the direction belongs in the script, where a writer sets it once and both
a synthesis tool and a human actor read it.

Builds on: the `.dlg` format (`assets/dialogue/README.md`, parser in
`crates/content/src/dialogue.rs`), line ids (ADR-0045: from speaker and
words), voice clips by line id (ADR-0046) and `spoken_text` (0238).

## Nick input

None. He asked for the feature and gave the examples; the syntax and
the plumbing are technical. Verified by tests.

## Scope

**In:**
- A way to give one speech line a delivery direction in plain words
  ("calmly", "shouting", "whispering, close to tears"), and a way to set
  a default for a stretch of a scene. Pick the syntax (an ADR if it is a
  new kind of cue); a direction must read naturally to a human writer and
  never be confused with an expression (`[angry]` already sets the
  portrait). Free text, not a fixed list: ElevenLabs-style tools take
  free-form tags, and a human actor reads plain English.
- The parser keeps the direction on the line; **the dialogue screen never
  shows it**, and the dialogue log neither.
- **A direction is not part of the line id.** Changing it must not mark
  translations stale (ADR-0045). It **does** mark that line's generated
  voice clip stale (it is part of what the voice tool was given, like the
  speaker's voice in 0721's plan): record it in the voice manifest's
  inputs.
- `cargo xtask lines` shows the direction in its own column, and 0721's
  recording script (`voice script --character`) prints it for a human
  actor.
- `check-script` accepts the cue and flags an empty one.
- Docs: `assets/dialogue/README.md` (the reference), the writer's guide
  (`docs/story/writers-guide.md`, a section and a checked `dlg` example),
  and the `story-writing` skill (write a direction where the punctuation
  and the expression don't already make the delivery obvious; never on
  every line).

**Out (do not do):**
- Calling a voice tool (0721), generating clips (0722), choosing the tool
  or the voices (0043).
- Adding directions to Chapter 1's lines (0737 writes the script; see
  below).
- Narration voicing is decided (0043: narration is voiced); nothing here changes it. Neither chosen reader takes a direction in words today (ADR-0107), so the cue is mainly for 0721's dry run, for a later tool and for a human cast.

## Implementation steps

1. Read ADR-0045, ADR-0046, 0238's completion notes and 0721's plan.
   Decide the syntax and write it up (ADR if it adds a cue kind).
   Expected size: about 150 production lines (parser field, validator
   rule, `lines` column, manifest input), judged by 0238's `spoken_text`
   plumbing. Write the engine-friction estimate first (ADR-0092). If the
   dialogue parser has moved into Shuyi Engine by then, follow the
   `engine-change` skill.
2. Parse it onto the speech line; keep it out of the line id and out of
   everything the dialogue screen and the log draw.
3. Add it to the voice manifest's inputs so a changed direction marks the
   clip stale; extend 0238's staleness tests.
4. `lines` column; `check-script` rule; docs and skill as in Scope.
5. **Either order with 0721:** if 0721 is done, pass the direction to its
   `Synth` trait and its recording script here; if not, add a line to
   0721's steps saying to pass it.
6. **Either order with 0737:** if 0737 is done, file a small content
   ticket to add directions to Chapter 1 where they help; if not, add a
   line to 0737 saying to write them.

## Acceptance criteria

- [x] A test script with directions parses; the dialogue screen snapshot
      of such a line shows no direction text.
- [x] Changing only a direction keeps the line id and marks the voice
      clip stale (tests).
- [x] `cargo xtask lines <scene>` prints the direction column.
- [x] `check-script` flags an empty direction.
- [x] The README, the writer's guide (with a checked `dlg` example) and
      the story-writing skill describe it.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: parsing (a line with a direction, a scene default, an empty one),
  line-id independence, voice staleness.
- Snapshot: the dialogue screen on a line with a direction (no direction
  shown).

## Completion notes

Done. A script line can say how it is spoken, and it was heard working on the
real voice tools.

**The format:** `speaker (direction): text`, `speaker[expression] (direction):
text`, `> (direction): text`, and `@delivery <direction>` / `@delivery clear`
(`assets/dialogue/README.md`, "Directions"; ADR-0109). The parser and the manifest
are the engine's, so the format landed first as engine#3114,
`Zafnok/shuyi-engine#238`, and this PR moves the pin to that merge. It passes 15
more engine PRs (`docs/engine/pins.md`).

**Tested with Nick, beyond the ticket:** he asked to prove that a direction
changes the tone and isn't read aloud. Across all the takes a speech-to-text check
heard no direction word, and neither did Nick. Two rounds by ear, on Harl, the
lead (woman), Hollis and Vix, on the listening page published as a private
artifact: `claude.ai/artifact/CK8BrWLt8WNXZMA9CbKpSw`. His words and picks are in
`docs/design/voices-languages-and-script.md`. The recipe is ADR-0109. It amends
ADR-0107, which expected directions to have little effect. The lab's scripts are
`D:\voice-lab\scripts\dir*.py` and `r2_*.py`, with the takes in `D:\voice-lab\out\`.
Only 0721 turns them into a tool.

**Deviations:**
- A changed direction doesn't make the clip stale in the sense the game uses,
  where a stale clip goes silent. The clip stays playable, because it still says
  the words on screen, and a new `needs_remake` tells the voice tool to redo it.
  The acceptance criterion holds for the tool, which is who it was for.
- `lines` puts the direction before the text, so the long column stays last.

**Follow-ups:** 0758 writes directions into Chapter 1, and 0722 now waits for it.
0757 writes them into the later chapters first. 0721's steps now say how to pass a
direction, retry bad takes and keep a whisper quiet. 1210 comes from the pin move.

*The agent's starting rules*, for the Act 1 playtest (1064):
- A directed clip keeps its distance from its character's level, within limits,
  so a whisper stays quiet. The limits are 0721's to set.
- Writers direct a minority of lines. Two lines per character were heard, so
  whether directions hold over a whole scene is still to judge by ear.
- A character's undirected lines keep their chosen reader, and directed lines take
  ADR-0109's recipe. Whether the two sound like one person across a scene hasn't
  been heard. If not, a whole scene may need to go one way.

Engine friction: estimate 150 production lines (ticket, judged by 0238), measured
155 (145 engine, 10 game), 1.0×.
