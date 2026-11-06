---
id: "0859"
title: Auto-advance when the voice ends
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0720", "0826"]
nick_input: none
completed: 2026-11-06
---

# 0859 — Auto-advance when the voice ends

## Context

Nick chose "Same, plus auto-advance" in 0043
(`docs/design/voices-languages-and-script.md`, *Voices*): an Options setting
moves to the next line by itself when the clip ends, so a player can sit back and
listen. 0720 plays the clips and 0826 adds the voice settings.

## Nick input

None. Verified by tests. The pause lengths are the agent's starting values; what
is left to judge by playing goes on the Act 1 playtest's checklist (1064).

## Scope

**In:** an **Auto-advance: On / Off** row under the voice rows, starting Off; the
dialogue screen moving on after a clip ends.

**Out (do not do):** auto-advance on reply choices (they always wait); on lessons
and tips (they wait for the player); a speed setting.

## Implementation steps

Expected size: about 60 production lines, judged by the dialogue screen's skip.

1. Add the setting to `Settings`, migrate, and add the row.
2. In `crates/ui/src/screens/dialogue.rs`: when on, and the current line's clip
   has ended (its length is known, ADR-0046 *As built*), wait 0.5 s (*tunable*)
   and advance as a press would.
3. A silent line (no clip, or voices off) waits as long as reading takes: about
   1 s plus 0.05 s per character (*tunable*).
4. Any press still works as usual, and a choice always waits.

## Acceptance criteria

- [x] Off: nothing advances by itself (Harness test).
- [x] On: a line advances after its clip plus the pause; a silent line after the reading time (Harness tests, with time stepped).
- [x] On: a reply choice never advances by itself (test).
- [x] The setting survives a restart and old settings load as Off (tests).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: settings migration.
- Integration: the Harness tests above.

## Completion notes

- **Setting:** `ShuyiSettings::auto_advance` (`AutoAdvance::Off` / `On`,
  an enum because clippy allows a struct three bools), saved as
  `auto_advance: Some(true)` only when on, so every older save loads as
  Off with no version bump. The **Auto-advance** row sits under *Voice on
  hover in the log*; like the other voice rows it is PC-only until the
  Playdate plays voices (1048). `docs/design/options.md` has the rules.
- **Dialogue screen:** with the setting on, a revealed page with no key
  pressed, no reply choice and no "Skip scene?" open moves on as Confirm
  would. A voiced line's last page waits until its clip ends plus 0.5 s;
  every other page (silent lines, voices off, earlier pages of a long
  voiced line) waits 1 s + 0.05 s a character on the page. Openings and
  the open log hold it.
- **Deviation (engine):** the ticket assumed the screen knows the clip's
  length (ADR-0046 *As built*). Only the platform's voice player knows it;
  nothing reports it back to screens. Engine PR #285 has the same audio
  files open, so per `engine-change` this ships with a marked workaround:
  a clip is taken to last 0.08 s a character (`SPEECH_PER_CHAR_S`,
  `// engine-friction: engine#2255`). Engine ticket 2255 (Zafnok/shuyi-engine#290)
  has the platform report the clip sounding each frame; game ticket 0863
  then waits for the real end and removes the workaround.
- **The agent's starting rules** (for the Act 1 playtest, 1064): the 0.5 s
  pause, the reading time of 1 s + 0.05 s a character (from the ticket);
  reading time counted per page, not per line; a voiced line's earlier
  pages turn on reading time while the clip plays on; the clip estimate of
  0.08 s a character until 0863 (a fast clip waits a little longer than
  it needs, a very slow one could be cut short).
- **Engine layout fix:** the new row made the in-battle Options (with
  Game mode and Tutorial) reach the help line, so its message row (e.g.
  "The game mode can be changed at Preparations…") was painted over.
  Zafnok/shuyi-engine#294 lets the engine's options panel rise into the
  blank rows above it; this PR moves the pin past it (the snapshots show
  the panel one or two rows higher in battle).
- Friction: estimate 60, measured 84 (1.4×); signal: the workaround above,
  row in `docs/engine/friction.md`.
