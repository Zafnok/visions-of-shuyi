---
id: "0752"
title: "Voice-direction cues in .dlg scripts: how a line is said, for voice synthesis, not shown in game"
type: feature
milestone: M6 Story & dialogue
model: opus-5.5
effort: high
status: todo
blocked_by: []
nick_input: none
completed:
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
- Narration voicing, if 0043 hasn't decided narration is voiced.

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

- [ ] A test script with directions parses; the dialogue screen snapshot
      of such a line shows no direction text.
- [ ] Changing only a direction keeps the line id and marks the voice
      clip stale (tests).
- [ ] `cargo xtask lines <scene>` prints the direction column.
- [ ] `check-script` flags an empty direction.
- [ ] The README, the writer's guide (with a checked `dlg` example) and
      the story-writing skill describe it.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: parsing (a line with a direction, a scene default, an empty one),
  line-id independence, voice staleness.
- Snapshot: the dialogue screen on a line with a direction (no direction
  shown).

## Completion notes

*(Filled in by the session that completes the ticket.)*
