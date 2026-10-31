---
id: "0731"
title: Name the guide and write the tips and lessons in their voice
type: content
milestone: M6 Story & dialogue
model: fable-5.1 / gpt-6-astra
effort: medium
status: done
blocked_by: ["0457", "0458"]
nick_input: decision
completed: 2026-10-15
---

# 0731 — Name the guide and give them a voice

## Context

Ticket 0057 decided the tutorial guide (`docs/design/tutorial.md`, *The
guide*; `docs/story/characters/guide.md`): a new mascot with the **Fox
Miko** face, **outside the story for now** (it talks only to the player).
Nick may tie it into the story much later ("think - Angel from borderlands
2").

Until now the box title says `Guide` (text key `guide.name`, 0455) and the
tips (`assets/data/tips.ron`) and lessons (`assets/data/lessons.ron`, 0457)
are written plainly. This ticket gives the guide a name and a voice, and
rewrites every tip and lesson in it. The agent writes all the lines; Nick
picks the name (`ask-nick`).

## Nick input

**Decision** with the `ask-nick` skill: the guide's name. Offer several
names with the feel of each (deep option write-ups; the `story-writing`
skill; names must be renameable). Show a tip box with the name and two or
three rewritten lines for each, rendered with the bought art
(`cargo xtask frame-png --features private-assets`; not committed).
Don't propose a backstory: the guide stays outside the story (0057).
Ask about the voice only if Nick wants a say ("describe your own").

## Scope

**In:**
- The name: id `guide` added to `docs/story/names.md` and
  `assets/data/names.ron` (they must stay in step; a test checks), and the
  box title reads the name through the names table instead of
  `guide.name` (remove that key if nothing else uses it).
- `docs/story/characters/guide.md`: the name and the voice notes (what
  the guide sounds like, what they never say).
- Every tip and lesson rewritten in the guide's voice, same length limits
  and placeholders; the meaning and every key unchanged. Pick an
  expression (`face`) per tip and lesson where it helps.
- `cargo xtask check-text` and the content checks pass.

**Out (do not do):**
- Tying the guide into the story, giving them a scene or backstory.
- New tips, lessons, or changes to what any of them teaches.
- Translations (0719, 0726 pick the new text up).

## Implementation steps

1. Draft names and sample lines; render the samples; ask Nick.
2. Record the name (`names.md`, `names.ron`, the character sheet).
3. Rewrite `tips.ron` and `lessons.ron`; read every one in a rendered box.

## Acceptance criteria

- [x] The tip and lesson boxes show the guide's name as the title.
- [x] Every tip and lesson is in the guide's voice, still ≤ the length
      limits, with the same `{Action}` placeholders.
- [x] `docs/story/characters/guide.md` has the name and voice notes, and
      Nick's words choosing the name.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None new beyond the content and names checks (text only); update any
  snapshot that shows a tip's words.

## Completion notes

- **Name: Vix** (Nick's words in `docs/story/characters/guide.md`). Nick
  was shown four candidates (Kit, Rin, Sable, Pip), each with Chapter 1's
  first tip rendered in that voice with the bought art. He took **Sable's
  voice** (the dry insider, like Angel in *Borderlands 2*) but not the name
  ("sounds like a grandma"), and **Rin's face** (the calm `neutral`
  expression). Of a second round (Vix, Fen, Vesper, Kit) he said Vix, Fen
  and Kit were all fine and left the pick to the agent: Vix.
- `guide` is in `docs/story/names.md` (a new "Outside the story" table) and
  `assets/data/names.ron`. The tip and lesson boxes read their title from
  the names table (through the language pack, so a translation can rename
  her: the `test` pack has `names.guide`); the `guide.name` text key is
  gone.
- `docs/story/characters/guide.md`: the name, Nick's words, which face
  goes with what, the voice notes and what she never says. She is "she"
  (Nick: "rename her"). `docs/design/tutorial.md` points to it.
- Every tip text and every lesson text rewritten in her voice, each with a
  `face` (`neutral` mostly; `sly`, `thinking`, `stern`, one `smile` at a
  level up). Same placeholders, same meaning, same ids; tips within 3 × 60,
  lesson text within the 45 the story keeps to. **Not changed:** the
  titles (the Guides list's labels) and the checklist labels (instructions
  ticked off, kept plain).
- Read in rendered boxes: the snapshot tests show the start tip (keyboard,
  three pads, the `test` language), the first lesson and the Guides list;
  the start tip was also rendered with the bought art (`cargo run -p xtask
  --features private-assets -- frame-png <out> --tips ...`; not
  committed).
- Snapshots updated: the start tip's box is one column wider (its longest
  line grew by one).
- No gameplay rules decided. Nick can veto any line.
