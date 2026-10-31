---
id: "0129"
title: "Move the backlog onto one Act 1 playtest"
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-21
---

# 0129 — Move the backlog onto one Act 1 playtest

## Context

Phase 0 of the Act 1 plan. Nick asked which game and engine tickets get
Act 1 (Chapters 1–8, `docs/story/outline.md`, including the world-map
chapters 4–8) playable on the web and on the Playdate. The planning
session found:

- Only Chapter 1 exists as content (`assets/chapters/ch01.ron`); the
  chapter flow already chains chapters with `next`.
- `core` has Rout, DefeatUnit, Seize and Survive, reinforcements,
  talk-recruit, battle notes, elementals, chests and shops. Missing:
  villages (0313), the shop screen (0409), the Preparations shop (0443),
  camp (1003), the world map (1007, 1008), escape and fleeing-boss
  rules, and Chapters 2–8's content.
- No engine ticket blocks Act 1. The game's own Playdate front end runs
  on the device, so engine 3010 is cleanup. engine#2304 is done but past
  the game's pin.
- 1118 (the carve's last ticket) is done; most Chapter 1 round 4
  tickets waited only for 1118 and 1050.

Nick, 2026-10-21: "any tickets that say 'check after merge' aren't
gonna happen -- I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself. I don't wanna spot
verify any gameplay tickets anymore."

This ticket moves the backlog off the Chapter 1 playtest gates (0804,
1050) onto a single Act 1 playtest, waives the post-merge sign-offs, and
files the tickets the Act 1 plan needs. Backlog and docs only, no code.

## Nick input

None. (Nick's rule above is the input; no design answer is invented
here: every undecided question went into a `00xx` ticket.)

## Scope

**In:**
- 1064, the Act 1 playtest, replacing 0804 and 1050 (both closed into
  it, carrying their open blockers over).
- Re-gating: 0804, 1050 and 1118 out of every open `blocked_by`.
- Waiving post-merge sign-offs on feature, bug, tuning and content
  tickets; the rule in `tickets/README.md`, the template and the
  `write-ticket` and `work-ticket` skills.
- The Act 1 tickets: design 0069, 0070, 0071; systems 1177, 1065, 0319;
  Playdate 1066–1069; content 0743, 0744–0750, 0848–0854, 1070–1073;
  proof 0512.
- `docs/ROADMAP.md`: the top note and *Act 1 critical path*.

**Out (do not do):**
- Any game or engine code; any design answer.

## Implementation steps

1. Close 0804 and 1050 into 1064.
2. Re-gate every open ticket; waive the sign-offs.
3. Write the new tickets; check `blocked_by` both ways.
4. Rewrite the roadmap's note and add the Act 1 path; update the README,
   template and skills.
5. `cargo xtask ticket-lint`, the gates, the PR.

## Acceptance criteria

- [x] No open ticket's `blocked_by` has 0804, 1050 or 1118.
- [x] 0804 and 1050 are in `tickets/done/` with a note that they were
      folded into 1064, which carries their open blockers.
- [x] No feature, bug, tuning or content ticket asks for a check after
      merge (`nick_input: sign-off` is left only on 1064 and 0906, a
      business call).
- [x] The new tickets are filed with consistent dependencies (no cycle,
      every id exists).
- [x] `docs/ROADMAP.md` opens with the Act 1 note and has *Act 1
      critical path*.
- [x] `cargo xtask ticket-lint` passes.

## Tests required

- `cargo xtask ticket-lint`; no code.

## Completion notes

**Closed into 1064:** 0804 (Chapter 1 playtest; round 4 never played)
and 1050 (Chapter 1 on the Playdate). Lint allows only `done` in
`tickets/done/`, so both are `status: done` with a Completion note that
says they were folded, as 0904 was closed without its check. Their open
blockers (round 3's tickets and the Playdate's 1039, 1041, 1043, 1044,
1046, 1058, 1060, 1062) are in 1064's `blocked_by`, with 0512 and the
Act 1 critical path.

**Re-gated** (0804, 1050 and 1118 removed): 71 open tickets. Where the
reason for the wait was "after Nick plays", the ticket now waits for
1064 instead: 0046, 0047, 0049, 0050, 0051, 0054, 0060, 0903, 1052 and
0508 (its recordings come from the Act 1 playtest). 0449 and 0740 wait
only for 1177 (the pin move picks up engine 2304 and 2305; 2305 closed
on 2026-10-21 too). 1051 is no longer conditional (todo, `setup` for
the device measurement): Act 1's battles are bigger. 0511 is no longer
conditional either: Act 1 has a boss per chapter. 0013, 0018, 0037 and
0044 are asked now, in the Act 1 design queue; 0013 is decided before
Act 1 is balanced (0848–0854 and 1070–1072 wait for it). 0063–0065
became ready (they waited for the carve and the Playdate port).

**Sign-offs waived:** 40 tickets went from `sign-off` to `none`, their
old question quoted in *Nick input* for 1064's checklist (1064 collects
them with `grep -l "What the old sign-off asked" tickets/done`). Where
the old sign-off was really a design question asked first, it became
`decision`: 0445, 0735, 0736 (a story-direction question), 0907 (the
store wording), 1001, 1007 (the glyph mockup stays; "Nick plays a test
map" is dropped). Where it was a device measurement, `setup`: 1051,
1057, 1059. 0449, 0605, 0606, 1010, 1054 and 1062 kept their
decision, setup or answer-first and lost only the post-merge check.
0906 (the title's trademark) keeps its sign-off: it is a business call,
not gameplay.

**Choices beyond the brief:**

- 0069 also asks Nick to confirm the outline's table of what each
  early chapter introduces, which the outline says "each chapter ticket
  confirms with Nick": asking once keeps the chapter tickets free of
  design questions.
- Each chapter's battle ticket (0848–0854) imports its chapter's new
  faces and combat pictures for the PC (0070 only decides; 1069 makes
  the one-bit versions), and waits for 0070, 0071 and 0013 as well as
  the brief's blockers. The world-map chapters (0850–0854) also wait for
  1070, whose locked story nodes they fill in.
- 1071 (fixed skirmishes) waits for the chapter scripts 0746–0749, which
  write the skirmishes' scenes and Gil's recruitment; 1073 (supports)
  waits for the last script, 0750, so the supports fit the ledger.
- 1072 asks Nick one story question first (does the runaway Candle of
  *The Other Cart* join), which the outline leaves to that ticket; if
  she joins outside battle, 1009 must be answered first.
- 0319 is `answer-first` (on 0069); 0512 is `setup` (the Playdate run
  needs a sideload).
- The `title` of 0069 names its sixth question.

**Follow-ups:** none beyond the tickets filed here.
