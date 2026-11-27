---
id: "1386"
title: "Decide: a text font for the picture look, today's font the frontrunner"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["1379"]
nick_input: decision
completed:
---

# 1386 — Decide: a text font for the picture look

## Context

1379 chose the picture look's windows (`docs/design/look-and-feel.md`,
*Windows (the picture look)*, question 6). No owned font both has lower
case and fits the game's 8×16 cell: ByteWright's Kingsrune and Crypt
(*Fantasy RPG Fonts*, *Pixel Font Megapack*) fit the cell but are
capitals only, and long dialogue lines read worse in them than in today's
Terminus. Nick, 2026-11-27: *"new ticket to search for new font, with
today's font as frontrunner."*

The cell is 8×16 font pixels; Chinese uses a second, 16×16 set (Fusion
Pixel, `assets/fonts/README.md`). Any font that changes the cell size is
an engine-scale change (1320's *More cells*): say so if Nick leans that
way. The title card's Cinzel (0812) shows that a font may differ in one
place (*"a titlecard is fine to break from our established one font"*).

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

**Decision**, with the `ask-nick` skill over rendered mockups of the
picture look (memory: picture look only): the unit menu, the side panel,
the forecast and a long dialogue line, today's font beside each
candidate, in the windows 1379 chose. 1379's local compositor
(`target/ui-audit/tools/mock.py` and `fontmock.py` in its worktree; if
gone, rebuild from its completion notes) shows how: lay the font in the
8×16 cells over a `frame-png`. Mockups made from bought art stay local
(ADR-0040).

The questions:
1. Keep today's font, or a candidate, for all window text?
2. If a candidate: also for the dialogue box, or only menus and panels?
3. A display font only for titles and name plates (Kingsrune, Crypt or a
   candidate), or none?

## Scope

**In:**
- Search, owned packs first (the audit in 1379's completion notes), then
  open-licensed pixel fonts (OFL, CC0, CC-BY; ADR-0013) and itch.io /
  BOOTH: lower and upper case, the Latin accents the language packs use,
  and a medieval or fantasy feel, drawn to fit 8×16 (or 8×16 with
  spacing), hand-made (no AI-assisted fonts).
- Mockups and the `ask-nick` rounds until the three questions are answered.
- `docs/design/look-and-feel.md`, *Windows (the picture look)*, question 6:
  the answer and Nick's words.
- If a new font is picked: its licence in `assets/fonts/`, its row in
  `THIRD_PARTY_ASSETS.md`, and a build ticket (`write-ticket`) that adds it
  to the font atlas (`cargo shuyi font-atlas`), blocked by 1380.

**Out (do not do):**
- Building the font into the game (the build ticket above).
- The Chinese set (Fusion Pixel stays).
- The glyph look's font (it keeps today's).

## Acceptance criteria

- [ ] The candidates and their licences listed in the completion notes.
- [ ] The three questions answered by Nick, his words quoted in
      `look-and-feel.md`.
- [ ] A build ticket filed if a new font was picked.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

None (a decision).

## Completion notes

*(Filled in by the session that completes the ticket.)*
