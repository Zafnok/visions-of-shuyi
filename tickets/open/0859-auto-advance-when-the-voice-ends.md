---
id: "0859"
title: Auto-advance when the voice ends
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0720", "0826"]
nick_input: none
completed:
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

- [ ] Off: nothing advances by itself (Harness test).
- [ ] On: a line advances after its clip plus the pause; a silent line after the reading time (Harness tests, with time stepped).
- [ ] On: a reply choice never advances by itself (test).
- [ ] The setting survives a restart and old settings load as Off (tests).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: settings migration.
- Integration: the Harness tests above.

## Completion notes

