---
id: "0083"
title: "Decide: getting off and back on a horse"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-11-17
---

# 0083 — Decide: getting off and back on a horse

## Context

Nick played the Act 1 build (Chapters 1–4) on 2026-11-08:

> "We should allow manual dismount and remount for mounted class to
> avoid effectiveness (strategy) -- however, treat like fire emblem does,
> after you make an action you can't take a free action of dismounting or
> remounting."

**Today there is no dismounting.** A mounted class has the `Mounted` tag
and the `mounted` movement type (`assets/data/classes.ron`, e.g. Rider
~:85: Mov 7, Sword E–C, Spear D–C, Charge 1, Lance Rush). Mounted units
pay more in forest (3) and mountains (5) and can't enter rivers
(`docs/design/terrain.md` ~:102–106); they ride indoors (floor costs 1,
`terrain.ron` ~:33). Spears are ×2 might against `Mounted`
(`items.ron:50`), the Unhorse art ×3 (`combat-arts.md` ~:220).

In Act 1 the only playable rider is **Tamsin** (Rider). Enemy riders:
Hound riders in Chapters 2–4 (Chapter 4's turn-7 reinforcements).
Mounted classes later: Lancer, Outrider, Iron Rider (also Armored), High
Lancer, Juggernaut (also Armored), Windrunner.

**What it would do, worked** (Tamsin at her Chapter 2 start: HP 21,
Def 5 + Leather Vest 2 = 7):

| Attacker (Ch 2, Lv 5) | On horseback | On foot |
|---|---|---|
| Hound guard, Iron Spear (Str 8 + 7 × 2) | **15** a hit, two hits kill her | 8 |
| Hound rider, Iron Spear (Str 7 + 7 × 2) | 14 | 7 |

**What Fire Emblem did** (Fire Emblem Wiki, *Dismount*; check again
before showing):
- **Mystery of the Emblem and Thracia 776:** mounted units can get off
  and fight on foot. On foot most use **swords only** (Thracia keeps a
  separate, usually lower, sword rank on foot), their stats drop (Mystery
  of the Emblem's Paladin loses about 2 Str, 3 Skl, 3 Spd, 2 Def, 6 Res
  and 4 Mov) and they must get off to go indoors.
- **Genealogy of the Holy War:** only Seliph, after promotion, and it
  does nothing in play.
- **Three Houses:** Mount/Dismount is a **free action, before or after
  moving but not both**, and **not after the unit's main action**: this
  is the rule Nick describes. On foot the unit stops being cavalry (no
  more weakness to anti-cavalry), moves less, can use gauntlets, and a
  dismounted cavalier is about 2 Spd faster (a dismounted pegasus knight
  slower). Indoors never forces it.

Nick has fixed: it's manual, both ways, free, and **never after the
unit has acted**. The rest is open.

## Nick input

**Decision** (`ask-nick`), at most three questions per message, each
with Tamsin's numbers on and off the horse. Show `frame-png` mockups of a
rider on foot on the battle map, in both the glyph look and the picture
look, and of the action menu with the new line.

1. **What a rider on foot is.** For Tamsin:
   - Mov 7 → 5 (a foot soldier's), or 7 → 6?
   - Moves like a foot soldier (forest 2, mountains 3, **wades rivers**
     at 5), or keeps the horse's costs?
   - Weapons: both her swords and spears (simplest), swords only
     (Thracia), or spears only?
   - Stats: unchanged (simplest), a small trade like Three Houses (a
     little faster on foot), or weaker on foot like Mystery of the
     Emblem?
   - No `Mounted` tag on foot, so spears lose their ×2: that is the
     point; confirm.
2. **When.** Nick's rule: never after acting. Before moving only, or
   before or after moving but not both (Three Houses)? Does getting off
   change how far she can still move that turn (for example, mounted
   Mov 7, moves 3, gets off: 2 left on foot, or 4)?
3. **Who and where.**
   - Every mounted class, Armored riders keeping `Armored` on foot? And
     fliers (Three Houses lets them land), or horses only?
   - Indoors: free (Three Houses, and the game today) or riders must get
     off (Mystery of the Emblem, Thracia)?
   - Enemy riders: never get off (they keep their weakness, simplest), or
     get off when it helps them?
   - Each battle starts mounted, or as the unit ended the last one?
4. **The look.** On the map a rider on foot must look different. Glyph
   look: the same two letters with a mark, or other letters? Picture
   look: Tamsin's sprite and combat picture are the Paladin, on a horse
   (`look-and-feel.md` ~:930); on foot she needs a foot picture: a pick
   from the bought art (show candidates as game screenshots, never the
   raw files) or a purchase.

The control is a line in the unit's action menu (`Dismount` / `Mount`),
so it needs no new key and works on the Playdate as it is. Ask only
whether he also wants a shortcut (then it is a rebindable action, the
`keyboard-input` skill).

## Scope

**In:** the question, the mockups, Nick's answer and words in
`terrain.md` (mounted movement), `weapons-and-items.md` (*Unit tags*)
and `progression.md` (the mounted classes) as each part needs, and
`look-and-feel.md` for the look; the `docs/design/README.md` row; 0334's
text updated with what to build.

**Out (do not do):**
- Building it (rules, menu, AI, map skins, the bots): ticket **0334**.
- Buying a foot picture for Tamsin: if Nick wants one, a purchase ticket
  (`write-ticket`) that 0334 is blocked by.
- Tamsin's own stats (0864). Either order: use her stats as they are
  when asked.
- Chapter 2's houses (0082). Either order: if this is answered first and
  a rider on foot can wade, tell 0082.

## Implementation steps

1. Read `terrain.md`, `weapons-and-items.md` (*Unit tags*,
   *Effectiveness*), `progression.md` (mounted classes),
   `look-and-feel.md` (riders' pictures), `docs/design/controls.md`,
   `combat.rs` (effectiveness ~:583–595).
2. Check the Fire Emblem facts on the Fire Emblem Wiki
   (<https://www.fireemblemwiki.org/wiki/Dismount>).
3. Make the mockups (throwaway, never committed): a rider on foot in
   both looks with `cargo shuyi frame-png`, and the action menu.
4. Ask Nick; iterate; record his words and the rule set.
5. Update 0334's text.

## Acceptance criteria

- [x] Nick's answer and words are in the design files named above;
      `docs/design/README.md` has the row.
- [x] 0334's text says exactly what to build (and names a purchase
      ticket if one is needed).
- [x] All gates in the `run-gates` skill pass.

## Tests required

None (a decision); `cargo xtask ticket-lint`.

## Completion notes

Decided 2026-11-17 over four rounds (the AskUserQuestion pop-up), with
`frame-png` mockups in the scratchpad (a throwaway `zz_dismount` battle
on the Salt Road map and an env-var mock in the glyph skin and the action
menu, reverted; nothing committed).

- **Rules** (round 1, with Tamsin's numbers): "Three Houses trade" (on
  foot: foot movement, Mov 7 → 5, Spd +2, no `Mounted`, same weapons,
  stats and skills); "Before or after moving" (Three Houses: free, not
  both in one turn, never after acting); "Simplest set" (every horse
  class, Iron Rider and Juggernaut walk as Armored, ride indoors as now,
  enemies never get off, each battle starts mounted). Recorded in
  `terrain.md`, *Getting off and back on a horse*, with pointers in
  `weapons-and-items.md` (*Unit tags*) and `progression.md`.
- **Control:** "menu line, with optional, not bound by default key".
- **Look:** in the picture look, mounted units' map sprites today are the
  Tiny Tales knight **on foot** (MattWalkden's riders, picked in 0040,
  were never imported). After seeing the store samples Nick said yes to
  MattWalkden's riders for mounted units: ticket **0499** (a free
  download Nick makes first). The glyph mark: Nick rejected seven
  mockups (corner dot, bar over, horse legs, horseshoe in the corner,
  capitals, horseshoe behind the letters, small horseshoe under them)
  and said "defer glyphs. dont need to block mechanic on it." Ticket
  **0089** holds the glyph mark and the combat picture on foot; 0334
  doesn't wait for either.
- The Fire Emblem facts were checked on the Fire Emblem Wiki's
  *Dismount* page (in the browser pane; automated fetches are refused).
- **0334** now says exactly what to build: no longer blocked on a
  decision, skins paint nothing new, and it has an estimate (1,700–2,000)
  with a split rule. **0082** now says that a rider on foot can wade.
  **1064** waits for 0499 too (riders on and off the horse should look
  different in the playtest); 0089 isn't on it, because Nick deferred it.

**The agent's starting rules** (gameplay rules where Nick's answer was
silent):
- Mov −2 on foot for every mounted class (Tamsin's 7 → 5 was shown; the
  others follow: Lancer 8 → 6, Iron Rider 6 → 4).
- Spd +2 on foot for every mounted class (Three Houses' cavalry value,
  shown for Tamsin).
- A rider can't get back on while standing where a horse can't go (in a
  river).
- Getting off before moving and then back on after moving in one turn is
  not allowed (one change a turn, the Three Houses rule as Nick picked
  it).
