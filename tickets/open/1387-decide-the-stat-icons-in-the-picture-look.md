---
id: "1387"
title: "Decide: the stat icons beside the stat names in the picture look"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["1379"]
nick_input: decision
completed:
---

# 1387 — Decide: the stat icons in the picture look

## Context

1379 (question 5, `docs/design/look-and-feel.md`, *Windows (the picture
look)*) decided icons before weapon and item names, from ELV Games'
Rogue Adventure item icons. It mocked stat icons too, ByteWright's
*Quest UI Megapack* stat icons beside Str, Mag, Dex, Spd, Def and Res
(Mov had no match; Res used a water drop). Nick, 2026-11-27: *"yes stat
icons but not these. new ticket."*

The stats (`docs/design/stats-and-combat.md` and the side panel): HP, Str, Mag, Dex,
Spd, Def, Res, Mov, and the weapon lines Atk, Hit, Crit, Avo, AS. Each
needs an icon that reads at 16×16 beside the font, in the style of the
map's art (ELV Rogue Adventure, Tiny Tales) and the chosen windows
(Barely Games' Pixel Quest panels).

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

**Decision**, with the `ask-nick` skill over mockups of the side panel
(and the stat screen if one exists) in the picture look, as 1379 drew
them (its local compositor; mockups from bought art stay local,
ADR-0040). Questions:
1. Which icon set (or mix), one icon per stat?
2. Which stats get one: the seven core stats, HP, the weapon lines too?

## Scope

**In:**
- Owned packs first (the audit table in 1379's completion notes: Pixel
  Quest's *Menu Icons* and *Objects Icons Inventory*, Post-Baroque's *UI
  Icons*, ELV's icons, Royal Crown's icons, ByteWright's Pixel Loot),
  then itch.io (hand-made only; no AI-assisted packs), for a full set
  covering every stat.
- The rounds until both questions are answered; the answer and Nick's
  words in `look-and-feel.md`, *Windows (the picture look)*, question 5.
- Any purchase sorted into the private repository's `library/` with its
  licence and recorded in `THIRD_PARTY_ASSETS.md` (price, date, artist).
- The build: if 1382 is still open, add the stat icons to its scope and
  acceptance criteria; if it is done, file a build ticket (`write-ticket`).

**Out (do not do):**
- Building the icons (1382 or the filed ticket).
- Weapon and item icons (decided by 1379).

## Acceptance criteria

- [ ] Both questions answered by Nick, quoted in `look-and-feel.md`.
- [ ] 1382 updated, or a build ticket filed.
- [ ] Any purchase recorded and sorted.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

None (a decision).

## Completion notes

*(Filled in by the session that completes the ticket.)*
