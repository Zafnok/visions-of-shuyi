---
id: "1050"
title: "Nick plays Chapter 1 on his Playdate; feedback becomes tickets"
type: playtest
milestone: Post–Chapter 1
model: opus-5.5
effort: medium
status: todo
blocked_by: ["1036", "1039", "1041", "1043", "1044", "1045", "1046", "1047"]
nick_input: sign-off
completed:
---

# 1050 — Nick plays Chapter 1 on his Playdate

## Context

Nick asked to rush Chapter 1 onto the Playdate (2026-10-15). Once the
English build has every screen of Chapter 1 in one bit, the controls,
the crank and sound (1027–1047), he plays it through on his own Playdate
(Rev B) before anything is sold. Voices (1048, waiting for 0722) and the
Japanese and Chinese builds (1049, waiting for the translations) come
later and don't hold this play-through up.

ADR-0070 §5 left two speed questions for this play-through: whether the
enemy's pauses (1026: ~0.1 s per enemy, up to ~0.25 s) feel long
(1051), and whether start-up feels slow (1036's numbers).

## Nick input

**Sign-off / playtest:** Nick sideloads the English `.pdx` (Simulator:
*Device → Upload Game to Device*) and plays Chapter 1 from the title to
the end. He tells Claude, in plain words, anything that feels wrong:
text he can't read, things he can't tell apart, buttons or the crank
doing something unexpected, pauses that feel long, sounds.

## Scope

**In:**
- Build the English `.pdx` from `main` (`cargo xtask playdate
  --release`), give Nick the file and the steps.
- Turn every item of his feedback into a ticket (`write-ticket` skill,
  *Turning Nick's playtest feedback into tickets*): bugs with repro
  steps, feel items as tuning with proposed values, design changes as a
  `00xx` question first.
- Record his answer about the enemy pauses: if they feel long, 1051
  goes ahead; if not, close 1051 as not needed (move it to `done/` with
  a note).
- Collect device numbers from the console while he plays (start-up, the
  slowest enemy phase, frame times) for the PR.

**Out (do not do):**
- Fixing anything in this ticket.

## Implementation steps

1. Build; send Nick the `.pdx` and the steps.
2. Wait for his play-through; ask one concrete question per unclear item.
3. Write the tickets; update 1051; add them to the ROADMAP's Playdate
   bullet, and to 1052's `blocked_by` the ones Nick wants fixed before it
   is sold.

## Acceptance criteria

- [ ] Nick has played Chapter 1 on his Playdate.
- [ ] Each feedback item is a ticket (numbers listed in the completion
      notes); 1051 is either open with Nick's words or closed as not
      needed.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None (playtest).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
