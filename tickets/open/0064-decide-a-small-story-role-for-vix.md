---
id: "0064"
title: "Decide: a small role for Vix in the story (map hints on unusual maps)"
type: design-decision
milestone: Design decisions
model: reasoning
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0064 — Decide: a small role for Vix in the story

## Context

Vix, the tutorial guide (0057, 0455, 0731), is outside the story for now:
Nick, in 0057, "maybe tied in much later, 'Angel from borderlands 2'"
(`docs/design/tutorial.md`; `docs/story/characters/guide.md`: she never
speaks of the world, the plot or herself).

Chapter 1 playtest round 3 (0804, 2026-10-16):

> "I think she turned out great actually... she really ties ch1 together
> as an intro to a quality game... idk about "around more" since too many
> tutorials would be annoying to anyone but maybe if there is a way to
> slot her into a minor role in the story it would be good. For example
> in fortune's weave, Fortuna is a clippy, but she just advises on unique
> maps like "hey over here is an explosive barrel" or something."

Check the game he names before using it as a reference (what Fortuna
does, when she speaks); say so honestly if it can't be checked.

Not needed before the next playtest, so not in the Act 1 playtest's `blocked_by` (1064).

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

**Decision** (`ask-nick`), with options from real games and how each fits
Vix's voice. For example:

1. **What she does after the tutorial.** A. Map hints only: one line when
   a map has something unusual (a breakable wall, a burning field, a
   special tile), as he described. B. A. plus a one-line remark at key
   story moments, never during scenes. C. A. plus an in-world identity
   revealed later (Angel in Borderlands 2, Navi in Ocarina of Time). D.
   Your own.
2. **Can players turn her map hints off** (separately from the tutorial
   level), or does the tutorial level cover it?
3. **Who she is, if C:** a story question; the story-writing pipeline
   (ADR-0011) needs Nick's beat first.

## Scope

**In:** the questions; recording answers in `tutorial.md` and, for story
parts, `docs/story/beats.md` (Nick's words only); writing the build
ticket(s).

**Out (do not do):** writing her story lines; building map hints.

## Acceptance criteria

- [ ] Nick answered; recorded.
- [ ] Build ticket(s) written with `blocked_by`; `cargo xtask
      ticket-lint` passes.

## Tests required

- None (decision).

## Completion notes

*(Filled in by the session that completes the ticket.)*
