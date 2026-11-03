---
id: "0754"
title: The voice says another word for the player's name
type: feature
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0043", "0238"]
nick_input: none
completed: 2026-11-03
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

- [x] `spoken_text` gives the stand-in for a `{lead|...}` line and the name in the box stays the player's (tests).
- [x] Changing a stand-in leaves the line id the same and makes the clip stale (test).
- [x] The content check fails on a voiced `{lead}` line with no stand-in (test), and passes on `assets/`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `spoken_text`, the line id, the content check.

## Completion notes

- **Syntax:** `{lead|the exile}`. The text box shows the name; the voice
  says the words after the bar. A bar with no words, or on any other token
  (`{they|she}`), stays an unknown-token error.
- **Where it lives:** in the game, not the engine. As a script is read
  (`trpg_content::dialogue`, new `dialogue/stand_in.rs`), each
  `{lead|…}` becomes `{lead}` before the engine checks the scene, and the
  line ids are given again from what is left. So the table, the line ids,
  the translations' English source and the text box all see a plain
  `{lead}`, and nothing in the engine changed. The lines as written are
  kept apart by line id (`Scripts::stand_ins`, `Content::stand_ins`);
  `dialogue::written` puts them back for the voice, and the game checks
  clips against that (`screen.rs`), so `voice::spoken_text` reads the
  stand-in. Doing it in the engine would have needed a new field on its
  lines and a change to its voice and language APIs (the engine's
  translation check compares the line's text, so keeping the stand-in in
  the text would have made translations stale).
- **Content check:** a speech or narration line with a bare `{lead}` is an
  error at its line (`dialogue::missing_stand_ins`, run with the place and
  presence checks when the content loads, so `cargo xtask check-script`
  reports it too). Replies are silent and need none. Unit tests that build
  scripts with `from_sources` alone aren't held to it.
- **Scripts:** all 66 lines in `ch01`–`ch08` (62 narration, 4 spoken) and
  the 9 in the test scripts have stand-ins. The writers' guide example
  and `assets/dialogue/README.md` example too.
- **Docs:** `docs/voice.md`, `assets/dialogue/README.md` ("What the voice
  says for the name"), `docs/story/writers-guide.md`, the `story-writing`
  skill. 0718 and 0725 got a line: a voiced translated line takes the
  stand-in in its language, and the pack's token check must allow it.

The agent's starting rules (for the Act 1 playtest, 1064):

- Narration says "the exile" for the name ("The exile" at the start of a
  sentence).
- The four spoken lines take a word that fits the speaker, never a name
  (names can be renamed in `names.ron`): the sergeant's dying line says
  "...No. Friend. There." (ch02); the heretic says "the exile's wrist"
  (ch02); the retainer says "this one and the boy" (ch04); the keeper says
  "Sleep, you. Please." (ch08).
- Engine friction: estimate 110 production lines, measured 200 (1.8×),
  under 2×. PR size: estimate 650, measured 726. The private-assets test
  wasn't run locally (no `assets-private/` checkout in this worktree); CI
  runs it.
