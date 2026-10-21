---
id: "1073"
title: "Act 1's supports (C, B, A) and camp events"
type: content
milestone: Post–Chapter 1
model: fable-5.1
effort: high
status: todo
blocked_by: ["1003", "0743", "0750"]
nick_input: none
completed:
---

# 1073 — Act 1's supports and camp events

## Context

Supports are FE GBA-style: a pair builds points by fighting side by side
and unlocks conversations at C, B and A, viewed at camp; "which pairs
exist, and what their conversations say, is decided by the story
pipeline" (`docs/design/supports.md`, *Pairs*). Camp events are short,
optional, scripted scenes that appear once the story reaches a set
point (*Rules: camp events*). 1003 builds both screens; this ticket
writes Act 1's.

Personal arcs live in the main script and the supports, never in camp
events alone (Nick: "Supports everywhere, camp for extras"). The
supports must fit what has happened, so this ticket comes after the
last Act 1 script (0750) as well as the new cast's sheets (0743).

## Nick input

None. Claude writes the conversations (story decisions only go to
Nick, with `ask-nick`: a support that would change a beat). Nick reads
them in the Act 1 playtest (1064).

## Scope

**In:**
- **The pairs**: which Act 1 pairs have supports (the lead with each
  companion, and companion pairs whose wants collide: the character
  sheets' suggestions), with thresholds overrides only where a pair
  needs one (`supports.md`).
- **The conversations**: C, B and A for each pair, in the support data
  format 1003 uses, by the `story-writing` skill (0736's craft rules,
  the critique pass); lead pairs follow the lead rules
  (`setting-and-tone.md`, reply choices).
- **Camp events** for Act 1: when each appears (after which story
  battle, who must be alive), group scenes and small world details.
- Names, the ledger (what a support reveals is canon from then on).

**Out (do not do):**
- Support thresholds, points or bonuses (`supports.md`; a tuning ticket
  after the playtest if the bots or Nick find them off).
- Act 2 pairs; endings (`supports.md`, *Open sub-questions*).

## Implementation steps

1. Read `supports.md`, the `story-writing` skill, every Act 1 sheet,
   the outline, the ledger and the scripts 0737, 0744–0750.
2. Pick the pairs; outline each pair's C → B → A arc in a few lines.
3. Write the conversations and the camp events; critique pass.
4. Data, content checks, ledger.

## Acceptance criteria

- [ ] Every listed pair has C, B and A conversations that load and pass
      `cargo xtask check-script`.
- [ ] Camp events appear at their points (harness test from saves at
      two points of the story).
- [ ] The critique pass is ticked in the PR body; the ledger is updated.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Content checks for the support and camp data.
- Integration: a pair reaching C at camp shows its conversation; a camp
  event appears after its story battle.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
