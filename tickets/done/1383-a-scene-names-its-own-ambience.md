---
id: "1383"
title: "A scene names its own ambience: the ambience directive"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
blocks: []
nick_input: none
completed: 2026-11-26
---

# 1383 — A scene names its own ambience: the ambience directive

## Context

1324 (2026-11-26, `docs/design/audio.md`, *Chapter 1's beds*) decided
that **a scene over a battle's map can play a bed of its own**, not the
battle's: Chapter 1's Red Company and after-the-fight scenes play over
the battle map (`@place battle`) with beds of their own (no birds, and
quieter). 1332 built ambience by place: `assets/data/ambience.ron`'s
`places` picks a bed by the `@place` map's id, and `@place battle` always
plays the battle file's own `ambience` (`crates/content/src/ambience.rs`,
`assets/audio/README.md`, *Ambience*). Nothing lets a scene choose.

A directive is allowed while dialogue writing is paused: it changes no
words (the `story-writing` skill: "changes to a script that touch no
words (`@music`, `@place`, triggers)").

## Nick input

None: the rule is decided (1324). Verified by tests; what's left to judge
by ear goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- A dialogue directive `@ambience <id>` that plays that ambience (an id
  of `ambience.ron`'s `ambiences`) from that point of the scene until the
  next `@ambience` or `@place`, and `@ambience none` for silence (the
  music goes on). Without it, a scene plays its place's bed as now.
- The check: `@ambience` naming an unknown ambience is a content error
  with file and line, as `@music` naming an unknown cue is.
- `assets/dialogue/README.md`'s directive table and *Places* text, and
  `assets/audio/README.md`'s *Ambience*.

**Out (do not do):**
- Chapter 1's beds and its scripts' directives (1335).
- A sound effect at a line (the chapel bell, 1385).
- Any script line (AGENTS.md rule 3).

## Implementation steps

1. Read how `@music` is parsed, checked and played (the `.dlg` parser in
   `trpg-content`, the dialogue screen's audio in `trpg-app`/`trpg-ui`)
   and how 1332 starts a scene's bed from `@place`.
2. Add `@ambience <id>` / `@ambience none` the same way: a parsed
   directive, checked against the ambience table, and when played,
   switching the bed with the same fade 1332 uses between places.
   `@place` after it goes back to the place's bed.
3. Document it in both READMEs.

Expected size: about 80–120 production lines, judged by `@music`'s
parse, check and play.

## Acceptance criteria

- [x] A scene with `@place battle` then `@ambience x` plays bed `x`, not
      the battle's (test through 1332's audio fake).
- [x] `@ambience none` stops the bed and keeps the music.
- [x] An unknown id is a content error naming file and line.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the directive's parse and check.
- Integration: a test scene's bed switches and stops (1332's audio fake).

## Lesser targets

Same data everywhere; the Playdate plays no bed (1322), so the directive
does nothing there.

## Completion notes

- `@ambience <id>` and `@ambience none` are a game directive
  (`Stage::Ambience(AmbienceLine)` in `trpg-content`'s `dialogue.rs`), parsed beside
  `@place`, printed back as written, and of their own kind for blocks:
  every reply of a choice and both parts of an `@if` leave the same one.
- The check is with the `@place` checks (`dialogue/setting.rs`,
  `check_places`): an id that isn't one of `data/ambience.ron`'s
  `ambiences` is an error with file and line. The ambience table loads
  after the dialogue (the world maps it checks need the scenes), so the
  check reads the file's ids alone (`ambience::ids`); a file that doesn't
  parse leaves `@ambience` unchecked and is reported by the ambience
  load.
- Played by the dialogue player's setting (`trpg-ui`'s `dialogue.rs`):
  `@ambience` sets the scene's bed, `@place` clears it back to the
  place's. The dialogue screen asks 1332's `ambience::play` for it, so the
  switch fades as between places, and the log's jumps carry it with the
  rest of the scene's state. A scene over the battle with no place of its
  own that names an ambience plays it too.
- Tests: the parse, print and check in
  `crates/content/src/dialogue/tests/ambience.rs`; the screen's bed in
  `screens/dialogue/tests.rs`; and through the headless speaker in
  `crates/ui/tests/it/ambience.rs` (a test bed registered beside the
  meadow, since the public file has one ambience).
- No gameplay rule decided. To judge by ear on the Act 1 playtest
  (1064): nothing until 1335 writes Chapter 1's `@ambience` lines.
- Friction: estimate 100 production lines (the ticket's, checked against
  `@place`'s parse and `@opening`'s check), measured 91: under 2×.
