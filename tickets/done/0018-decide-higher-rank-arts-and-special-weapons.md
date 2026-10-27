---
id: "0018"
title: "Decide: Combat Arts for ranks C–S and the first special weapons"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: done
blocked_by: ["0014"]
nick_input: decision
completed: 2026-10-26
---

# 0018 — Decide: higher-rank arts and special weapons

## Context

Ticket 0014 (`docs/design/combat-arts.md`) decided how Combat Arts work and
the Chapter 1 list: two arts per weapon kind, at ranks E and D. Tier-1
classes reach rank C, and tier-2 classes reach A, so later chapters need arts
for ranks **C, B, A and S**. Nick also chose **special weapons** that carry
their own art (his example: a unique blade, like Dark Souls' Moonlight
Greatsword, whose art makes the combat hit Res). None exist yet, since Chapter
1 has no loot. Until this is decided, reaching rank C teaches nothing new.
Run with the `ask-nick` skill now. It waited for the Chapter 1 playtest
(0804) so the starter arts were felt first and question 3 could be
answered; Nick has played three rounds of it, and Act 1's side quests
give unique rewards (1072), so it is asked before them (re-gated
2026-10-21, 0129).

## Nick input

**Decision.**

## Questions to ask

1. **Arts for ranks C–S**, per weapon kind, with exact effects, costs and
   ranges. Draw them from Three Houses, Fortune's Weave and Engage, keep our
   own names, and avoid repeating class actives' functions (Nick: overlaps
   "should make sense and be rare"). Show a forecast mockup.
2. **Special weapons:** the first few (for Chapters 2–3), their stats and
   their own arts, including Nick's Res-hitting blade.
3. **Playtest follow-ups:** after 0804's rounds 1–3, does any tier 1–2 active need a new
   flavour so it overlaps arts less (Heavy Blow, Lance Rush, Dive are "might
   +N")?

## What to record

Extend `docs/design/combat-arts.md` (Later ranks, special weapons). Create
the implementation ticket(s) with the `write-ticket` skill (data in
`arts.ron`/`items.ron` on top of 0312; any new effect kinds in `core`).
Update `docs/design/README.md`.

## Acceptance criteria

- [x] Nick answered 1–3.
- [x] `combat-arts.md` updated with exact numbers.
- [x] Implementation tickets created; design README updated.
- [x] Ticket archived.

## Completion notes

Asked 2026-10-26 with a page of the 20 proposed arts, three weapons and
forecast mockups (<https://claude.ai/artifact/U13aJ3WFR1FWALZTiZonUn>),
then one round of follow-ups. Recorded in `combat-arts.md` (*Ranks C–S*,
*Special weapons*, *Class actives after the playtest*), with his words.

- **Arts C–S:** sword, spear and bow as offered (Break Shot renamed
  Rending Arrow; Overwatch made an action of its own, no attack, still
  costs durability). Axe: Wide Swing became a 270° cleave with no normal
  combat; Haft Break and Dread Swing were replaced by Hurl (B) and Skull
  Splitter (A). Gauntlet: Deflect also strikes back.
- **Special weapons:** Houndmaster's Axe and Fordwarden's Spear for
  Chapter 2 approved. The Moonblade moved to the end of a long optional
  questline (0075): a passive hits Res with Str, its art Moonwave reaches
  1–2 with the higher of Str and Mag. Chapter 3 gets no special weapon.
- **Actives:** Dive reflavoured (strike from up to 3 tiles away in a
  line, might +4); Lance Rush stays might +5 (Nick: it must not double up
  with Charge); Heavy Blow stays.

*Claude's starting rules* (Nick may veto): every "Exact rules" bullet of
*Ranks C–S* not quoted from him (Riposte only when it can counter; Disarm
cancels the rest of this combat's counters;
Unbroken Stride is a full move and action; Comet Lance in rows and columns
only, passing allies; Overwatch triggers once, only on a unit that moved;
Arrow Rain's four neighbours; Chain Strike's allies use no durability and
get no EXP), the name Rending Arrow, the selling question's wording, the
Houndmaster's Axe is given when Chapter 2 is won, and every cost and stat
(*tunable*).

Tickets created: 0325, 0326, 0327, 0328, 0329 (building it), 0075 (the
Moonblade's questline). 0848 and 1072 now wait for 0329; 1064 waits for
0325–0329.

After the PR was opened, Nick changed two starting rules (2026-10-26):
"cascade shouldnt count missed strikes I think" (only hits count now) and
"special weapons can be sold but should warn the player" (selling asks
first). Both are in `combat-arts.md`, 0325 and 0329.
