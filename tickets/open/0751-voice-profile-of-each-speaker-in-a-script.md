---
id: "0751"
title: "A voice profile of each speaker in a dialogue script (cargo xtask voices)"
type: feature
milestone: M6 Story & dialogue
model: sonnet-5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0751 — A voice profile of each speaker in a script

## Context

Ticket 0736 researched why Nick found the Chapter 1 script bland
(`docs/story/craft.md`). One finding was measurable: every speaker had
the same rhythm. Over the whole script the median sentence was 4 words
for five of the six speakers and 5 for the sixth, and 51–63% of each
speaker's sentences were 4 words or fewer, although the character sheets
ask for very different voices. The critic who found it counted with a
throwaway script. Writers can't see this by reading; a number shows it in
seconds.

The `story-writing` skill's critique pass checks voices with a cold read
(a fresh subagent tells the speakers apart with names removed). This
ticket adds the number beside it: a report per speaker, for a scene or a
chapter, that a writer reads during the critique pass. It is a report,
never a gate: prose can't be gated.

Builds on `cargo xtask lines` (`crates/xtask/src/lines.rs`), which already
lists every line of a scene with its speaker.

## Nick input

None. A writers' tool, verified by its tests.

## Scope

**In:**
- `cargo xtask voices [scene-or-prefix]`: with `ch01_` every scene whose
  id starts with it, with a scene id that scene, with nothing every scene.
- For each speaker (narration as `>`, replies as `*`, the lead as `lead`),
  one row: boxes; median and mean characters per box; median words per
  sentence; share of sentences of 4 words or fewer; contractions per 100
  words; questions per box; the share of boxes that end on a sentence of
  4 words or fewer (the critic's "every box ends on a closer"); the five
  most frequent words that are not on a short list of common English
  words.
- Under the table, the pairs of speakers whose median sentence length and
  short-sentence share are both within a small margin (tunable
  constants), as "similar rhythm: <a> and <b>".
- A line in the story-writing skill's critique pass: run it on the
  chapter and look at the similar pairs.

**Out (do not do):**
- Failing a build or a test on a profile (it is advice for writers).
- Judging word choice or quality; any language model.
- Changing `check-script`, the dialogue format or any script.

## Implementation steps

1. Expected size: about 150 production lines, judged by `lines.rs` (30
   lines for the listing) plus the statistics and the table. Write the
   engine-friction estimate first (ADR-0092).
2. Add `crates/xtask/src/voices.rs` with `USAGE` and a pure
   `profile(dialogue: &DialogueTable, filter: Option<&str>) -> Result<Report, String>`
   built on `Scene::lines()`, and `Display` for the report. Sentence
   splitting: on `.`, `!`, `?` and `...`, ignoring name tokens (`{n:...}`,
   `{lead}`), which count as one word.
3. Dispatch `voices` in `crates/xtask/src/main.rs` next to `lines`, with
   `--help`.
4. Add the critique-pass line to `.claude/skills/story-writing/SKILL.md`
   and the command to `docs/story/writers-guide.md`, section 8.

## Acceptance criteria

- [ ] `cargo xtask voices ch01_` prints one row per Chapter 1 speaker and
      the similar pairs.
- [ ] Unit tests: sentence splitting with name tokens and `...`; medians
      on a small made-up script; a pair flagged as similar and a pair not.
- [ ] `cargo xtask voices nope` exits non-zero with a clear message.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the statistics on made-up scripts (`from_sources`, as
  `lines.rs`'s tests do).
- Integration: `dispatch(["voices", "ch01_"])` returns 0, beside the
  existing `lines` dispatch test in `main.rs`.

## Completion notes

*(Filled in by the session that completes the ticket.)*
