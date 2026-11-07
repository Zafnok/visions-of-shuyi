---
id: "0761"
title: Voice the short lines the transcript check can't hear
type: feature
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0722"]
nick_input: none
completed: 2026-11-06
---

# 0761 — Voice the short lines the transcript check can't hear

## Context

`cargo xtask voice generate` (0721, ADR-0113) keeps a take only when
faster-whisper's transcript of it is at least 85 % alike to the line's
words, character by character (`crates/xtask/src/voice/synth.rs`, `says`,
`SAME_WORDS`). On a line of one invented name that fails even when the
reader says it right: in 0722 the lead's line "Dace's."
(`ch01_prebattle_a4c1d405`) was read correctly by both lead voices, but
the transcriber spells the name "Dases" or "Daces", 0.67 to 0.8 alike, so
no take passed in three tries (12 seeds a voice) and the line is silent
in Chapter 1. Respelling it in `say.ron` ("Daces", "Dayces") passed some
takes in a probe but not in the tool's runs. Every later chapter has short
lines with names (0757), so this will come back.

## Nick input

None. Verified by tests; how the line sounds goes on the Act 1 playtest's
checklist (1064).

## Scope

**In:**
- A way for a short line that is read right to get a clip.
- Voicing `ch01_prebattle_a4c1d405` (both `.m` and `.f`) with it, in the
  private repository, and the pin (`assets-private.rev`).

**Out (do not do):**
- Rewording the line.
- Lowering `SAME_WORDS` for every line.
- Other chapters' voices (0757).

## Implementation steps

1. Pick one of these and record it in ADR-0113's follow-ups (or a new ADR
   if it changes the tool's rules):
   - the check compares a short line (a few words) by sound rather than
     spelling, e.g. a phonetic key of each word (Metaphone-like) before
     the character distance; or
   - `cargo xtask voice generate --keep <clip file>`: saves the first take
     of that clip without the transcript check, for a person or agent who
     listened to it, and marks it so `status` doesn't flag it.
2. Test it in `crates/xtask/src/voice/tests.rs` with the fake readers:
   "Dases" heard for "Dace's." passes (or `--keep` saves the clip), and a
   wrong word on a long line still fails.
3. Run it on `ch01_prebattle`; `cargo xtask voice status --scene
   ch01_prebattle` reports nothing to make. Commit in the private
   repository, `cargo shuyi private-assets --pin`, commit the pin.
4. Document it in `docs/voice.md`.

## Acceptance criteria

- [ ] `ch01_prebattle_a4c1d405` has both clips and `cargo xtask voice
  status` reports nothing missing for any `ch01` scene.
- [ ] A test shows a short line whose name is spelled differently by the
  transcriber gets a clip, and a long line with a wrong word still doesn't.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the check (or `--keep`) with the fake readers in
  `crates/xtask/src/voice/tests.rs`.

## Completion notes

- Picked the first option, the check by sound (ADR-0113's new
  *Follow-ups*): a line of at most four words (`SHORT_LINE_WORDS`,
  *tunable*) also passes when the transcript has as many words and each
  word's Metaphone-like key (`synth::sound`) is the same. Every longer line
  keeps the 85 % rule. `--keep` was rejected: it needs someone to listen
  to every such line of every later chapter (0757).
- Tests: "Dases", "Daces" and "Days" say "Dace's."; "Bases", "Dace is"
  and "Hold the boat" (for "Hold the gate") don't; five names spelled
  otherwise fail on a five-word line and pass on a four-word one.
- `cargo xtask voice generate --scene ch01_prebattle` made both clips on
  the first seed. `voice status` lists nothing to make for any `ch01`
  scene (the 873 left are later chapters, 0757). Private repository
  `0665d99`, pinned.
- Deviation: the tool wrote `sent.ron` with `\r\n` line ends on Windows
  (ron's default), turning the whole committed file over. One line in
  `files::sent_text` keeps it `\n`, with a test; needed to commit the
  clips cleanly, so done here rather than ticketed.
- For the Act 1 playtest (1064): how the lead says "Dace's." in both
  voices. The cost of the rule: a short line read as a sound-alike word
  would pass, so listen for it there.
- No gameplay rules decided.
- Friction: estimate 35 production lines, measured 55 (1.6×), under 2×.

