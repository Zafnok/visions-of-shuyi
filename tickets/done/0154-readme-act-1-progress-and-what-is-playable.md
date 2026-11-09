---
id: "0154"
title: "README: Act 1's progress, what is playable now, and the plan up to the Act 1 playtest"
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-09
---

# 0154 — README: Act 1's progress, what is playable now, and the plan up to the Act 1 playtest

## Context

The root `README.md` was last refreshed by 0125 on 2026-10-15, when the
goal was Chapter 1 and its next playtest round. About 245 tickets have
landed since. The critical path moved to Act 1 (0129): Chapters 2–5 are
in the game (0848–0851), with the world map, towns, shops, camp, random
skirmishes and the first side quests (1003, 1007, 1008, 0409, 1070,
1072); the tutorial with Vix (0455–0458, 0731, 0476), player profiles
(0836, 0837), conversation places (0732–0734), Chapter 1's voices (0722)
and the arts of ranks C–S (0325–0329) are in. The engine carve finished
and moved to its own repository (1131). Nick paused agent-written
dialogue until a human writer is found (0766, 2026-11-08). The README
still describes the Chapter 1 playtest rounds, lists the world map,
shops, the tutorial and profiles as missing or planned, and its progress
numbers are from 2026-10-15.

Nick asked (2026-11-09): "our README could use an update / overhaul.
Update any progress numbers and trim out parts that aren't relevant and
add new parts that might be relevant without being redundant".

## Nick input

None.

## Scope

**In:**
- `README.md`: what is playable today, what is not in yet, progress
  (an Act 1 table and the ticket counts), the roadmap up to the Act 1
  playtest, the Playdate and Shuyi Engine sections brought up to date,
  the plans after Act 1 without what is already built, and the playtest
  bots in *How it's built*.

**Out (do not do):**
- No new screenshots: the gallery's Chapter 1 pictures still match the
  game. A world-map or later-chapter picture needs a capture route the
  debug tools don't offer yet (the debug menu opens only the test world
  map).
- No change to the License section or the SonarCloud badges: 0153 (the
  repository goes private) changes them, and `LICENSE` is Nick's.
- No story spoilers past the chapters' places; no game code.

## Acceptance criteria

- [x] The README's progress numbers match `tickets/` as of 2026-11-09
      (this ticket counted as done).
- [x] Everything listed as playable is in the game on `main`; everything
      listed as not in yet or planned has a ticket or a design doc.
- [x] No key is presented as fixed.
- [x] `cargo xtask ticket-lint` and `typos` pass.

## Tests required

- None (docs only).

## Completion notes

- **Progress counts** (2026-11-09, this ticket counted): 473 of 718. Rows
  by block as before, except: a ticket whose title starts with
  "Playdate" counts in *Playdate version* whatever its block; `11xx` and
  `12xx` together are *Shuyi Engine and tooling*; `10xx` without the
  Playdate is *World map and later plans* (it was *After Chapter 1*);
  `08xx` is *Chapters and game flow*.
- **Act 1 table:** scripts drafted 8/8 (0737, 0744–0750), battles 5/8
  (`assets/chapters/ch01`–`ch05`), side quests 2/5 (1072; 1083 and 1084
  left), fixed skirmishes 0/8 (1071), chapters voiced 1/8 (0722; 0757).
- **Corrected:** the default layout is the arrow keys and the home row
  (`docs/design/controls.md`), not vim-style keys.
- **Removed:** the Chapter 1 playtest rounds as the current step, the
  world map, supports, shops and villages under *Planned* (built), the
  engine carve's history, voices and transitions as *Planned* (voices
  are in for Chapter 1; transitions and title cards are listed as not in
  yet).
- **Added:** the world map and its towns, camp, skirmishes and side
  quests; the tutorial, profiles and voices; Act 1's remaining work, the
  two Act 1 playtests' tickets and the search for a writer; the
  playtest bots; `docs/targets.md`.
