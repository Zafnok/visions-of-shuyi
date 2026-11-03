---
id: "0759"
title: "A dialogue line whose text starts with ( prints and parses back unchanged"
type: bug
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0759 — A dialogue line whose text starts with ( prints and parses back unchanged

## Context

Found while working 0474: `print_parse_round_trip` in
`crates/content/src/dialogue/tests.rs` failed once on `main` (proptest,
random seed). Its shrunk case is a narration step whose text is `():0`. The
engine's `print_scene` (`shuyi-engine`, `crates/assets/src/dialogue.rs`)
writes it as is. Then the engine's parser (`crates/assets/src/dialogue/parse.rs`,
`direction`) reads the leading `( )` as the line's voice direction (0752)
and reports `a line's ( ) has no direction in it`. A non-empty `(...)` at
the start of a line's text would be read as a direction instead and
silently dropped from the text.

The shrunk case was:

```
cc 550d4aa78a16178e28897005f4406b8c09f9b9c4236ec55397aa2e1cb45aa89d # shrinks to scenes = [Scene { id: "a", steps: [If { character: CharacterId("a"), then: [], otherwise: [If { character: CharacterId("a"), then: [], otherwise: [Choice { options: [ChoiceOption { tone: "a", text: "a", line: LineId("a_000e2ebc"), steps: [If { character: CharacterId("a"), then: [], otherwise: [If { character: CharacterId("a"), then: [Narrate { text: "():0", direction: None, line: LineId("a_ac09d2fc") }], otherwise: [] }] }] }] }] }] }] }]
```

The engine's own round-trip test never puts `(` in text (its `arb_text` is
letters and `,.!?`), so only the game's wider `arb_text` (any printable
ASCII) can hit it.

## Nick input

None. Verified by the property test; nothing to judge by playing.

## Scope

**In:**
- Text that starts with `(` survives print → parse: either `print_scene`
  writes it so the parser reads it as text (an escape the `.dlg` format
  already has, or a new one documented in the format), or the parser
  rule is narrowed so that it can't be read as a direction. Pick
  whichever keeps written scripts unchanged.
- That regression case as a fixed test, in the engine and in
  `crates/content/proptest-regressions/dialogue/tests.txt`.

**Out (do not do):**
- Narrowing the game's `arb_text` to avoid `(`: that hides the bug.
- Changing what a voice direction is or how it is written (0752).

## Implementation steps

1. Reproduce: add the `cc` line above to
   `crates/content/proptest-regressions/dialogue/tests.txt` and run
   `cargo test -p trpg-content --lib print_parse_round_trip`.
2. The printer and parser are the engine's: follow the `engine-change`
   skill (an engine PR in `Zafnok/shuyi-engine` with a test in its
   `crates/assets/src/dialogue/tests.rs` for text starting with `(` and
   `()`, and the engine's `arb_text` widened to include `(` and `)`),
   then move the pin here.
3. Check no script in `assets/` or `assets-test/` changes meaning:
   `cargo test --workspace` (the scripts load) and grep the `.dlg` files
   for a line's text starting with `(`.

## Acceptance criteria

- [ ] `print_parse_round_trip` passes with the regression file holding the
      case above.
- [ ] The engine's round-trip property covers text with `(` and `)`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Property: the round-trip in the engine and in `trpg-content`, with the
  saved regression case.
- Unit: text starting with `()` and with `(soft) word` prints and parses
  back as text.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
