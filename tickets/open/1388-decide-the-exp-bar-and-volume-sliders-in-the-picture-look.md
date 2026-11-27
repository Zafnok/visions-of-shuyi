---
id: "1388"
title: "Decide: the EXP bar and the volume sliders in the picture look"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["1379"]
nick_input: decision
completed:
---

# 1388 — Decide: the EXP bar and the volume sliders in the picture look

## Context

1379 (question 4, `docs/design/look-and-feel.md`, *Windows (the picture
look)*) put every HP gauge on Barely Games' Pixel Quest
`F-UI_DynamicBar_A1` (made symmetric), filled green, yellow or red by the
HP left. Its mockups drew the EXP bar the same with a gold fill, and the
Options screen's volume sliders (music, sound, ambience, voices) on
Post-Baroque's `UI_ChargeFrameA_2` with a gold fill. Nick, 2026-11-27:
*"exp bar probably different?"* and *"volume slider I didnt like the pick
you had. probably new ticket for volume and exp bar."*

The EXP bar shows on the results and level-up screens; the sliders on
Options (every volume sound layer has one, memory: listening-page
sliders, and they are percentages, 0882).

**The kits are in the private repository from commit `b68863cc` on**
(its PR #42, `library/barely-games/`, `library/tiopalada/`,
`library/opengameart/`, `library/untied/`, `library/hypnobius/`). When
1379 merged, `assets-private.rev` was older (moving it then would have
carried other tickets' unmerged content), and `cargo shuyi private-assets
--library` checks out the pinned commit. So first check that the pin is at
or past `b68863cc`; if not, check out the private repository's `main` in
`assets-private/` and move the pin (`cargo shuyi private-assets --pin`) in
this ticket's PR, once no open public PR still needs an older pin for its
private content.

## Nick input

**Decision**, with the `ask-nick` skill over mockups in the picture look
(1379's local compositor; mockups from bought art stay local, ADR-0040):
the results/level-up screen with the EXP bar, and Options with its
sliders, each with several bars and fills from the owned kits (Pixel
Quest's *Dynamic Bars* and *Slide Bar*, Post-Baroque's *Health Bars* and
*Sliders & bars*, Royal Crown's meters, ByteWright's bars) and, if none
fits, store candidates (hand-made only). Questions:
1. The EXP bar: which bar and fill?
2. The volume sliders: which slider (track, fill, handle)?

## Scope

**In:**
- The rounds until both are answered; the answers and Nick's words in
  `look-and-feel.md`, *Windows (the picture look)*, question 4.
- Any purchase sorted into the private repository's `library/` with its
  licence and recorded in `THIRD_PARTY_ASSETS.md`.
- The build: if 1382 is still open, add the EXP bar and the sliders to
  its scope; if it is done, file a build ticket (`write-ticket`).

**Out (do not do):**
- The HP bars (decided by 1379).
- Building them (1382 or the filed ticket).

## Acceptance criteria

- [ ] Both questions answered by Nick, quoted in `look-and-feel.md`.
- [ ] 1382 updated, or a build ticket filed.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

None (a decision).

## Completion notes

*(Filled in by the session that completes the ticket.)*
