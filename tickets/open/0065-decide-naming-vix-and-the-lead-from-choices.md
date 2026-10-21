---
id: "0065"
title: "Decide: the player names Vix (and the lead) from suggested names or their own"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0065 — Decide: naming Vix and the lead

## Context

The guide's name is Vix (0731; Fen and Kit were Nick's other OKs). The
lead is named by the player today on the lead-select screen (a free-text
box with a default name; 0247, `docs/story/names.md`: names are
variables). Chapter 1 playtest round 3 (0804, 2026-10-16), asked whether
to keep the name Vix:

> "Vix is fine but what we can also do is let players name her like they
> do the rival in pokemon with pre-filled choices or with free text. We
> can do the same with the lead character too."

In the Pokémon games (Red/Blue onward) the rival's name is picked from a
short list of suggestions or typed (check the details before citing).

Not needed before the next playtest, so not in the Act 1 playtest's `blocked_by` (1064).

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

**Decision** (`ask-nick`), on mockups of the real New Game screens.

## Questions

1. **When is Vix named?** A. At New Game, after the familiarity question
   (only when there are lessons, i.e. not at None). B. The first time she
   appears, in her box. C. Your own.
2. **The choices:** which suggested names (Vix, Fen, Kit, others?), plus
   "type your own".
3. **The lead:** add suggested names beside today's free text? Which?
4. **Where her name shows:** the box title only (today), or her lines
   too (she never says her own name, `guide.md`)?
5. **Per profile or per campaign?**

Then write the build ticket, blocked by this one.

## Scope

**In:** the questions; recording answers in `tutorial.md` and
`docs/story/names.md`; writing the build ticket.

**Out (do not do):** building it.

## Acceptance criteria

- [ ] Nick answered; recorded.
- [ ] Build ticket written; `cargo xtask ticket-lint` passes.

## Tests required

- None (decision).

## Completion notes

*(Filled in by the session that completes the ticket.)*
