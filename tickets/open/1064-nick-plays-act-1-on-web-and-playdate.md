---
id: "1064"
title: "Nick plays Act 1 on web and Playdate; feedback becomes tickets"
type: playtest
milestone: Post–Chapter 1
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0512", "0250", "0318", "0446", "0449", "0461", "0462", "0463", "0465", "0466", "0468", "0736", "0737", "0738", "0739", "0740", "0741", "0742", "0842", "0843", "0844", "0845", "0846", "0847", "1039", "1041", "1043", "1044", "1046", "1058", "1060", "1062", "0313", "0409", "0443", "0606", "1003", "1007", "1008", "1065", "0319", "0511", "0506", "0509", "0510", "1038", "1042", "1051", "1056", "1057", "1059", "1063", "1066", "1067", "1068", "1069", "0743", "0744", "0745", "0746", "0747", "0748", "0749", "0750", "0848", "0849", "0850", "0851", "0852", "0853", "0854", "1070", "1071", "1072", "1073", "1178", "0513", "0320", "0321", "0074", "1074", "0322", "0471", "0323", "0608", "0324"]
nick_input: sign-off
completed:
---

# 1064 — Nick plays Act 1 on web and Playdate

## Context

The one playtest left before release work: all of Act 1, Chapters 1–8
(`docs/story/outline.md`: Chapters 1–3 linear, 4–8 on the Act 1 world
map), on the web build and on Nick's Playdate. It replaces the Chapter 1
playtest's round 4 (0804) and the Chapter 1 play-through on the Playdate
(1050), both closed into this ticket on 2026-10-21 (ticket 0129).

Nick, 2026-10-21: "any tickets that say 'check after merge' aren't gonna
happen -- I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself. I don't wanna spot
verify any gameplay tickets anymore." So between now and this ticket,
gameplay tickets are verified by tests, `cargo shuyi frame-png` pictures
and the playtest bots (0506, 0509; the whole act by 0512), and whatever
was left to judge by playing collects here.

`blocked_by` holds: the bots' run of all of Act 1 (0512), which itself
waits for the content and the Playdate work; the round 3 tickets 0804
was waiting for (its open blockers, carried over); the Playdate tickets
1050 was waiting for; and the Act 1 critical path in `docs/ROADMAP.md`
(*Act 1 critical path*, Phases 2 and 3).

## Nick input

**Playtest:** Nick plays Act 1 from New Game to "Act 1 ends" on the
Pages build, and Chapter 1 at least (all of Act 1 if he wants) on his
Playdate (sideload: Simulator, *Device → Upload Game to Device*). He
answers the checklist and reports anything else in plain words: bugs,
feel, difficulty, story, looks, controls, sounds, pauses that feel long.

## Scope

**In:**
- Cut a release the way 0804 did (bump the version in its own PR from a
  branch whose name doesn't start with `t1064`, tag, wait for the release
  and Pages) and build the English `.pdx` (`cargo xtask playdate
  --release`).
- **The checklist**, published as a private claude.ai page that saves
  his answers (as 0804's round 3 did), with the links, the key
  cheat-sheet from `keymap.ron`, the Playdate's buttons and crank, and
  questions in play order. Collect them from:
  - the new Chapter 1 script (0737): ask about it first, as round 4
    would have;
  - every ticket in `tickets/done/` whose *Nick input* quotes a waived
    sign-off (`grep -l "What the old sign-off asked" tickets/done`), and
    the *Claude's starting rules* in their Completion notes;
  - 1050's questions: whether the enemy's pauses on the Playdate feel
    long, whether start-up feels slow (numbers from 0512's device run);
  - the new chapters' starting rules (0848–0854, 1070–1073) and the
    Act 1 battle rules (0069, 0319).
- Turn every item of his feedback into a ticket (`write-ticket` skill,
  *Turning Nick's playtest feedback into tickets*): bugs with repro
  steps, feel items as tuning with proposed values, design changes as a
  `00xx` question first. Ask at most one clarifying question per unclear
  item.
- Record his answers that settle a starting rule in the design doc that
  holds it.

**Out (do not do):**
- Fixing anything in this ticket.
- Playtesting Act 2 or anything past "Act 1 ends".

## Implementation steps

1. Check every `blocked_by` id is in `tickets/done/`, `main` is green and
   Pages is deployed.
2. Version bump PR, tag, release; build the `.pdx`.
3. Build the checklist page from the sources above (`artifact-design`
   and `artifact-capabilities` skills: the page keeps his answers).
4. Send Nick the links and the page.
5. Read his answers back; write the tickets; record settled rules.
6. Put a summary in the Completion notes: his words verbatim and the
   ticket each became (or "won't do" with the reason he agreed to), as
   0804 did.

## Acceptance criteria

- [ ] Nick played Act 1 on the web build and on his Playdate.
- [ ] Every feedback item maps to a ticket (or an explicit "won't do"
      Nick agreed to); the table is in the Completion notes.
- [ ] The release exists and the `.pdx` was handed over.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None (playtest).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
