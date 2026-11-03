---
id: "0759"
title: Dialogue round trip fails on a line whose text starts with parentheses
type: bug
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0759 — Dialogue round trip fails on a line whose text starts with parentheses

## Context

Found while working 0326: the property test
`trpg_content::dialogue::tests::print_parse_round_trip`
(`crates/content/src/dialogue/tests.rs`) failed once in a full test run
and proptest shrank it to a narration line whose **text** is `():A`.
Printing the scene and parsing it back gives

```
t.dlg:10: a line's ( ) has no direction in it
```

so the printer writes text that the parser reads as an empty voice
direction (`( )`, ticket 0752, `docs/voice.md`). The shrunk case, as
proptest saved it (the regression file was not committed with 0326):

```
cc 27a778cc767c69a249ead8e185a049ce995d5b628a6b360acf5706f37785b1f9 # shrinks to scenes = [Scene { id: "a", steps: [If { character: CharacterId("a"), then: [], otherwise: [If { character: CharacterId("a"), then: [], otherwise: [If { character: CharacterId("a"), then: [Choice { options: [ChoiceOption { tone: "a", text: "a", line: LineId("a_000e2ebc"), steps: [If { character: CharacterId("a"), then: [Narrate { text: "():A", direction: None, line: LineId("a_ac0a5c9f") }], otherwise: [] }] }] }], otherwise: [] }] }] }] }]
```

The parser that reports it is the engine's
(`shuyi-engine` `crates/assets/src/dialogue/parse.rs`); the printer and the
generator of the test may be the game's or the engine's. It fails only
when the generator happens to draw text starting with `(`, so CI is red
at random until it is fixed.

## Nick input

None. Verified by tests.

## Scope

**In:**
- Find which side is wrong: the printer (it should write text starting
  with `(` so it reads back as text, e.g. escaped), the parser, or the
  test's generator (if such text can never be written in a `.dlg`, the
  generator shouldn't make it and a content check should refuse it).
- Fix it there; if the fix is in the engine, follow the `engine-change`
  skill (engine PR first, then the pin move here).
- Commit the regression case above as
  `crates/content/proptest-regressions/dialogue/tests.txt` so it runs
  every time.

**Out (do not do):**
- Changing the direction syntax of the scripts.

## Implementation steps

1. Add the regression file above and reproduce:
   `cargo test -p trpg-content --lib dialogue::tests::print_parse_round_trip`.
2. Read how a line's text and its `( )` direction are printed and parsed
   (game `crates/content/src/dialogue*`, engine
   `crates/assets/src/dialogue/`), and decide which side is wrong.
3. Fix it; add a unit test with a narration and a spoken line whose text
   starts with `(`.

## Acceptance criteria

- [ ] The round-trip test passes with the regression case committed.
- [ ] A unit test covers text starting with `(` (narration and speech).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: text starting with `(` round-trips or is refused with a clear
  error, as decided.
- Property: the existing round trip, with the regression case.

## Completion notes

*(Filled in by the session that completes the ticket.)*
