---
id: "0080"
title: "Decide: how the army gets through armour"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0080 — Decide: how the army gets through armour

## Context

Nick played the Act 1 build (Chapters 1–4) on 2026-11-08:

> "Aske the archer is too weak to even get XP so she snowballs inversely
> and becomes useless almost immediately. Her STR + wep are not enough to
> go thru the heavily armored units. Some things that might make this
> better are more thorough bots automated balancing (i.e. tweaking not
> just the enemy but also the player classes and units and levelups) OR
> introducing piercing mechanic with a guaranteed min hit i.e. 2 or 3 min
> hit for piercing damage"

> "Mages (i.e. Rue) are too necessary (and thus get used too often) bc
> most enemies are far too armored, i.e. armored knights are literally
> impossible to kill without mages so she gets too much combat and thus
> snowballs"

> "Probably, we can introuce piercing"

**Where it comes from.** `docs/design/weapons-and-items.md` ~:174:
armoured units have no special weapon rule; armoured classes have high
Def and low Res (Nick: "encouraged to use magic users on those
enemies"). So mages beating armour is Nick's own design. What he reports
is that it went too far: only Rue can. Treat the answer as a nudge, one
notch, not a reversal.

**The damage rule** (`crates/core/src/combat.rs`, module doc ~:8–60,
`fn side` ~:569): `damage = max(0, power − mitigation)`, power = Str (or
Mag) + weapon might, mitigation = Def (or Res) + the tile's Def. Then
axes raise it to at least 5 (`AxeMinDamage(5)`, `assets/data/items.ron:51`).
There is **no general minimum**. A `pierce` modifier exists (it lowers the
target's Def or Res, never the tile's), but its only source is the
Lancer's tier-2 active Piercing Lance (pierce 5, `skills.ron` ~:222).

**The only Armored enemy class in Chapters 2–4 is the Guard** (base
20/7/0/4/2/9/0, Def growth 55 %, Iron Spear, Chain Mail +4 Def). Generic
stats are fixed averages (`progression.md`, *Generic units*):

| Where | Guard | HP | Def | on a gate (+3) |
|---|---|---|---|---|
| Ch 2, (8,6) and (10,6) by Hendry | Lv 5 | 23 | 15 | — |
| Ch 3, gate (15,8), courtyard (18,7), stair top (18,4) | Lv 6 | 24 | 15 | 18 |
| Ch 4, gate (13,12), (13,6), (7,3) | Lv 7 | 24 | 16 | 19 |

**What the army does to a Chapter 2 guard today** (base stats, Def 15,
Res 1):

| Unit | Weapon | Power | Damage a hit |
|---|---|---|---|
| Lead | Iron Sword / Steel Sword | 12 / 15 | 0 / 0 |
| Hollis | Iron Spear / Steel Spear | 15 / 18 | 0 / **3** |
| Tamsin | Iron Spear | 14 | 0 |
| Aske | Iron Bow | 11 | 0 |
| Maud | Iron Gauntlets | 4 | 0 |
| Joss (from Ch 3) | Iron Sword | 11 | 0 |
| Rue | Fire (vs Res) | 12 | **11** (Cinder Oath: 15) |

By the bots' recorded army (`docs/playtesting.md` ~:310–330) Aske is
Lv 5 at Chapter 3 and Lv 6 at Chapter 4, Rue Lv 8 and Lv 12. Aske's
expected Str by then is about 6–7 (growth 45 %): Iron Bow 12–13, Steel
Bow 15–16, still **0** against Def 15–16. Rue's Mag is about 12 at Lv 8
(growth 80 %): Fire about 16 a hit, two hits, so one attack kills a
24-HP guard.

**Why Aske falls behind** (0081 has the EXP side): a combat that does no
damage gives **2 EXP** (`progression.rs` ~:100). Aske Lv 5 hitting a
Lv 6 guard for even 1 would get **20**; killing it, **66**. Each guard
Rue kills at Lv 8 gives her **46**.

**What already exists that bites armour:**
- Axes: every hit does at least 5, and **Armor Cleave** (axe rank D,
  `arts.ron` ~:122) doubles might against Armored. Nobody in the party
  can wield an axe yet (0079).
- **Rending Arrow** (bow rank C, `arts.ron` ~:151): the first hit lowers
  the target's Def by 5 until the end of its next phase (Three Houses'
  Break Shot, renamed). Aske starts at Bow D; C needs 70 weapon EXP
  (`items.ron` `rank_exp`), and she earns little.
- Magic: Rue (Fire, Cinder Oath). Maud has only Heal and Vigil Light.
  The next magic user joins in Chapter 6 (the Sorcerer, `battlemage.md`).

Nick paused AI story writing on 2026-11-08 until a human writer is hired
(ticket 0766). Any option that needs new dialogue or a new named
character says so: its words wait for the writer.

## Nick input

**Decision** (`ask-nick`), at most three questions per message, each
option as a player situation with the numbers above. Several rounds are
likely. Check every game claim before showing it; claim only what the
game really does.

**1. How does a non-mage hurt a guard?**

- **A. Piercing with a minimum hit (Nick's idea).** A "piercing" weapon's
  hit never does less than **2** or **3** (Nick's numbers), like this
  game's axes already never do less than 5. Example, min 3 on bows: Aske
  (Lv 2) shoots a Chapter 2 guard: 3, and she strikes twice (her attack
  speed 3 against the guard's −4), so **6 a round**: four rounds to kill
  it alone, and **22 EXP** for the damage instead of 2. With min 2: 4 a
  round, six rounds. Enemy archers pierce too: Hollis takes 0 from a
  Chapter 2 archer today and would take 3. Sub-questions: which kinds
  pierce (bows only, bows and spears, or chosen weapons), and 2 or 3?
- **B. Piercing as "ignore some Def"**, like this game's Piercing Lance
  (ignore 5), Fire Emblem's Luna skill (on a chance it ignores the
  target's Def: all of it in Path of Radiance, half from Awakening on) or
  XCOM 2's armour piercing (AP Rounds ignore armour points). Example,
  bows ignore 5: Aske's Iron Bow does `11 − 10 = 1` at base, the Steel
  Bow 4. Weak at low Str, strong later. Can combine with A ("ignore 3,
  never below 3").
- **C. A minimum for every hit** (say 1 or 2), so no unit is ever immune.
  Mainline Fire Emblem has no such rule (a hit can do 0 in the GBA games
  and Three Houses); Final Fantasy Tactics avoids the problem differently:
  armour adds HP there, not a damage cut. Downside: it also makes Hollis
  and every armoured unit easier to chip.
- **D. Softer guards.** Enemy guards wear the Leather Vest (+2) instead
  of Chain Mail (+4), or the Guard class's Def drops by 2: Def 13 in
  Chapters 2–3. Then the Lead's Steel Sword does 2 (4 a round, he
  doubles), Hollis's Steel Spear 5, Tamsin 1, Aske and Joss still 0.
- **E. Armour-slaying weapons in the Preparations shop from Chapter 2**,
  like Fire Emblem's Armorslayer (a sword) and Hammer (an axe), effective
  against armoured units. In this game "effective" doubles might (spears
  against Mounted). Example, a sword with Mt 6, ×2 against Armored: the
  Lead does `7 + 12 − 15 =` 4, twice. No Fire Emblem bow does this that
  I can vouch for; a bow version is our own idea.
- **F. More magic, earlier.** For example Maud gets a weak attack vow,
  as Three Houses' white-magic users also learn an attack spell
  (Nosferatu). A new mage recruit before Chapter 6 is a story decision
  and waits for the writer.
- **G. Aske's own numbers.** Honest: +1 Str or +1 bow might doesn't get
  her through Def 15; she needs about +4 power. So this only fits with
  another option (her growth, her bow, Tor's Bow, or Rending Arrow
  sooner).
- **H. Combinations, or describe your own.** For example A (bows, min 2)
  with 0079's axe holder as the heavy answer.

**2. Nick's other idea: should the bots tune the player's side too?**
Today autobalancing changes only the enemies (`docs/design/
playtest-bots.md`, *Autobalancing*: "only the battle changes"; knobs in
`crates/bots/src/balance.rs` ~:112–150), by Nick's own rule. The player's
classes, units, growths and level-ups are his design, so the honest shape
is: **the bots try player-side changes and report which ones put each
chapter in its band, and Nick picks** (as 0864 asks him about Tamsin).
Ask: report only (A), report and apply small changes inside limits he
sets (B), or neither (C)? Ticket **0527** builds that report.

**3. Only after 1:** how fast should it bite? For example "a guard falls
to three non-mage attacks" as the target the bots check.

## Scope

**In:**
- The tables above checked against the real army the bots arrive with
  (`trpg_bots::arrival`) for Chapters 2, 3 and 4, and redone after each
  option.
- The question, the answer and Nick's words in `weapons-and-items.md`
  (the weapon kinds' table and *Armoured units*), `stats-and-combat.md`
  (*Attack and damage*) if the damage rule changes, `playtest-bots.md`
  (*Autobalancing*) for question 2; the `docs/design/README.md` row.
- 0333's text (and 0527's for question 2) updated with what to build.

**Out (do not do):**
- Building it: ticket **0333**; the bots' player-side report: **0527**.
- EXP for a unit above its enemy's level (0081). Either order: if 0081
  is answered first, use its formula in the EXP numbers here; if not,
  use today's and note it in 0081.
- Who wields the Houndmaster's Axe (0079). Either order: show its pick
  here if answered, else list "an axe user" as part of option H.
- Tamsin's buff (0864) and the lead's (0846).

## Implementation steps

1. Read `weapons-and-items.md`, `stats-and-combat.md`, `combat-arts.md`
   (Rending Arrow, Armor Cleave), `magic.md`, `playtest-bots.md`,
   `combat.rs` (module doc), `classes.ron`, `characters.ron`,
   `items.ron`, `docs/playtesting.md`.
2. Recompute the tables for the bots' arrival army in Chapters 2–4
   (expected stats; say they are averages) and for each option.
3. Optionally run the bots on Chapters 2–4 with the leading option as a
   throwaway data change (never committed) to show the win rates and how
   often each unit fights.
4. Ask Nick; iterate; record his words and the rule set (numbers marked
   his or *tunable*).
5. Update 0333's and 0527's text.

## Acceptance criteria

- [ ] Nick's answer, his words and the numbers are in the design files
      named above; `docs/design/README.md` has the row.
- [ ] 0333 (and 0527 if question 2 changed it) say exactly what to build.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

None (a decision); `cargo xtask ticket-lint`.

## Completion notes

*(Filled in by the session that completes the ticket.)*
