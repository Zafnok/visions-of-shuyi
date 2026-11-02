---
id: "0754"
title: The voice says another word for the player's name
type: feature
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0043", "0238"]
nick_input: none
completed:
---

# 0754 — The voice says another word for the player's name

## Context

0043 (`docs/design/voices-languages-and-script.md`, *Voices*): a line that says
the player's name shows the name while the voice says another word. Today
`spoken_text` (`crates/content/src/voice.rs`) returns nothing for a line with
`{lead}`, so it can have no clip. Across Chapters 1–8 that is 62 of the 271
narration boxes and 4 spoken lines. ADR-0107 rule 4 amends ADR-0046.

## Nick input

None. The stand-in word is an agent's starting rule: "the exile" for now, and a
word that fits the lead's situation later. What is left to judge by playing
goes on the Act 1 playtest's checklist (1064).

## Scope

**In:** a way for a script line to say what the voice says in place of `{lead}`;
`spoken_text` using it; the stand-in written on every such line in `ch01`–`ch08`;
the docs.

**Out (do not do):** rewording a line otherwise; keys in Vix's lines (0475);
making clips (0721, 0722).

## Implementation steps

1. Choose the syntax so a human writer can read it, for example
   `{lead|the exile}`: the text box shows the name, and `spoken_text` gives
   "the exile". A capitalized stand-in follows `{Lead}`-style capitals if the
   scripts use them. If the token parser is in the engine, follow the
   `engine-change` skill, or keep the stand-in in the game's own pass over the
   line before it reaches the parser.
2. `spoken_text` returns the spoken words for such a line. A plain `{lead}` with
   no stand-in still returns nothing.
3. A stand-in is not part of the line id, so it doesn't mark translations stale
   (ADR-0045). It does mark the voice clip stale, because `spoken` changes.
4. Add the stand-ins in `assets/dialogue/ch01.dlg`–`ch08.dlg`. Read each line in
   its scene and check the grammar and `{they}` pronouns around it. Add a content
   check that every voiced line with `{lead}` has a stand-in.
5. Update `docs/voice.md` (it says a `{lead}` line can't have a clip),
   `assets/dialogue/README.md` and the `story-writing` skill's script rules.
6. If Japanese and Chinese packs exist, their lines take the stand-in in their
   language; otherwise the translation tickets (0718, 0725) inherit the rule.
   Add a line to each.

## Acceptance criteria

- [ ] `spoken_text` gives the stand-in for a `{lead|...}` line and the name in the box stays the player's (tests).
- [ ] Changing a stand-in leaves the line id the same and makes the clip stale (test).
- [ ] The content check fails on a voiced `{lead}` line with no stand-in (test), and passes on `assets/`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `spoken_text`, the line id, the content check.

## Completion notes

