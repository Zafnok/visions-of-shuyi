---
id: "1084"
title: "Act 1's side quests, part 3: Maud"
type: content
milestone: Post–Chapter 1
model: creative
effort: high
status: todo
blocked_by: ["1083", "1070", "0018", "0069", "0749", "0509", "0013", "0323", "0608", "0329"]
nick_input: none
completed:
---

# 1084 — Act 1's side quests, part 3: Maud

## Context

Split from 1072 by ticket 0135 (ADR-0101). 1072 holds the context, the scope
per quest and the steps: **read it first**, and build this one the way 1072
and 1083 built theirs.

| Quest | Whose | Unlocked | What it is |
| --- | --- | --- | --- |
| *Unwritten* | Maud | once Ossary is reached, Maud alive | she brings the register's names back to their families |

## Nick input

None; no check after merge (Nick, 2026-10-21). The reward is picked here,
inside 0069's kinds, and listed in the Completion notes for 1064's checklist.

## Scope

**In:** for this quest, everything 1072's scope lists per quest (beat sheet,
scenes, map and battle, unique reward, node in 1070's map with its unlock
condition, names, ledger, autobalance and playtest).

**Out (do not do):** the other four quests (1072, 1083); supports and camp
events (1073); fixed skirmishes (1071); changing what the main story says
happened.

## Implementation steps

1. Read 1072 and 1083 and what they built, the `story-writing` skill, the
   outline, the ledger, Maud's sheet, Chapter 7's script, 0069's and 0018's
   answers.
2. Beat sheet, script, critique pass, map, battle, reward, node.
3. Bots; `frame-png` pictures; ledger.

Expected size: about 900 changed lines (0135; `cargo xtask pr-size`).

## Acceptance criteria

- [ ] The quest unlocks under its condition and not otherwise (tests).
- [ ] It can be won (a winning replay) and gives its reward.
- [ ] The script passes `cargo xtask check-script`; the critique pass is
      ticked in the PR body.
- [ ] The bots land in their bands; the report is in the PR.
- [ ] Starting values and the reward are in the Completion notes
      (1064's checklist).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the unlock condition (Maud dead or not recruited → locked);
  a winning replay; the reward in the army's stock.
- Content checks for every new file.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
