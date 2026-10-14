---
id: "0731"
title: Name the guide and write the tips and lessons in their voice
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: medium
status: todo
blocked_by: ["0457", "0458"]
nick_input: decision
completed:
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
rewrites every tip and lesson in it. Claude writes all the lines; Nick
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

- [ ] The tip and lesson boxes show the guide's name as the title.
- [ ] Every tip and lesson is in the guide's voice, still ≤ the length
      limits, with the same `{Action}` placeholders.
- [ ] `docs/story/characters/guide.md` has the name and voice notes, and
      Nick's words choosing the name.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None new beyond the content and names checks (text only); update any
  snapshot that shows a tip's words.

## Completion notes

*(Filled in by the session that completes the ticket.)*
