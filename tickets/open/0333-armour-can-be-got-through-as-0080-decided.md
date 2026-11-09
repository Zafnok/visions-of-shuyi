---
id: "0333"
title: "Armour can be got through, as 0080 decided"
type: feature
milestone: M2 Core rules
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["0080"]
nick_input: answer-first
completed:
---

# 0333 — Armour can be got through, as 0080 decided

## Context

Nick played the Act 1 build (Chapters 1 to 4) on 2026-11-08:

> "Aske the archer is too weak to even get XP so she snowballs inversely
> and becomes useless almost immediately. Her STR + wep are not enough to
> go thru the heavily armored units. Some things that might make this
> better are more thorough bots automated balancing (i.e. tweaking not
> just the enemy but also the player classes and units and levelups) OR
> introducing piercing mechanic with a guaranteed min hit i.e. 2 or 3 min
> hit for piercing damage"

> "Mages (i.e. Rue) are too necessary ... armored knights are literally
> impossible to kill without mages so she gets too much combat and thus
> snowballs"

> "Probably, we can introuce piercing"

**The numbers today.** Damage (`crates/core/src/combat.rs`, rules in the
module doc ~:8–60, `forecast` ~:543, `side` ~:569–670):
`power = Str (or Mag) + might × eff_mult`;
`mitigation = max(0, Def (or Res) − pierce) + terrain Def`;
`damage = max(0, power − mitigation)`, halved for a Resist affinity, then
an axe's minimum (`AxeMinDamage(5)`, `assets/data/items.ron:51`). There
is no general minimum. `pierce` comes only from skill mods
(`CombatMods::pierce`, ~:323; ~:619), and its one source is Piercing
Lance (`assets/data/skills.ron:222`, pierce 5, a Lancer tier-2 active).
Armor Cleave (an Axe D art, ×2 vs Armored, `arts.ron` ~:122) is the only
armour answer a player could have, and nobody in Act 1 wields axes (0332).

The only Armored enemies in Chapters 2 to 4 are Guards (class base
20/7/0/4/2/9/0, Chain Mail +4 Def/+1 Res): Lv 5 HP 23 Def 15; Lv 6 Def 15;
Lv 7 Def 16, plus +3 on a gate. Against a Chapter 2 guard (Def 15) at
their starting stats: the lead's Iron Sword (power 12) and Steel Sword
(15) do 0, Aske's Iron Bow (11) 0, Joss (11) 0, Tamsin (14) 0; Hollis's
Steel Spear 3; Rue's Fire 11 (it hits Res 1). The design meant it
(`docs/design/weapons-and-items.md` ~:174: armoured classes have "high Def
and low Res", "use magic users on those enemies"), but with one mage
it makes Rue the only answer: in the recorded playthroughs Rue is Lv 8 at
Chapter 3 and Lv 12 at Chapter 4 while Aske is Lv 5 and 6
(`docs/playtesting.md` ~:310–330).

**0080** asks Nick how the army gets through armour; this ticket builds
the answer and re-balances Chapters 1 to 4 around it.

## Nick input

**Answer first:** ticket 0080. Nothing else is asked here. Verified by
property and unit tests, `cargo shuyi frame-png` pictures of the forecast
in the PR, and the bots; what is left to judge by playing goes on the
Act 1 playtest's checklist (1064).

## Scope

**In:** what 0080 decided. The likely shapes, and where each one's code
goes (the session builds the one Nick picked, or the combination):

- **A. A minimum damage on a hit** for every weapon, or for a kind
  (bows), or for weapons marked piercing: a field of `CombatRules`
  (`combat.rs` ~:200–255, beside `axe_min_damage`), applied in `side`
  after mitigation as the axe's is (and the order against Resist halving
  written down). If it is per weapon, a field on the weapon in
  `items.ron` (read by `trpg_content::item`) carried into
  `CombatantInput`'s weapon. Worked example to put in the design doc:
  Aske's Iron Bow (power 11) into a Chapter 2 guard (Def 15, 23 HP) does
  0 today; with a minimum of 3 she does 3 a hit, so 8 hits to kill it.
- **B. Piercing: a weapon or kind ignores some Def**, through the
  `pierce` that exists (`CombatMods::pierce`), from a weapon field or a
  kind's trait (`WeaponTrait` in `combat.rs` ~:130–140,
  `CombatRules::type_trait` ~:258–270, the kinds in `items.ron:48–54`).
  It may come with A ("a guaranteed min hit for piercing damage").
- **C. Weapons effective against Armored** in the shops (an
  Armorslayer-like item): data only, through the weapon's `effective`
  list that `side` already reads, placed in the Preparations shop or a
  town by `assets/data/` as 0080 says.
- **D. The player's side tuned** (classes, units, growths): data in
  `characters.ron` / `classes.ron`, the values 0080 gave. If 0080 instead
  makes the bots' autobalancer tune the player's side, that is a change
  to `docs/design/playtest-bots.md` *Autobalancing* and to
  `crates/bots/src/balance.rs` (`knobs`, ~:112, enemy numbers only today)
  big enough for its own ticket: write it (`write-ticket`) and do only A
  to C here.
- **E. Armour thinner:** the Guard class's Def or Chain Mail's bonus,
  data only.
- In every case:
  - The rules' doc comment in `combat.rs` and `docs/design/stats-and-combat.md`
    (*Attack and damage*, ~:110, and a worked example) and
    `weapons-and-items.md` (the armoured paragraph ~:174, the kinds'
    table) say the new rule.
  - **The forecast shows it.** The damage number already comes from
    `Forecast`; if 0080's answer needs a mark (like the `!` after an
    effective strike: `crates/ui/src/screens/battle/glyph/forecast.rs`
    ~:137, ~:243, and the Playdate's `screens/battle/playdate/forecast.rs`),
    both skins paint it from the view. If 0080 asks for no mark, the
    number alone is enough.
  - The AI and the persona bots read forecasts, so they follow the rule
    with no change; check one AI test that scores an attack on a guard.
  - **Re-balance Chapters 1 to 4.** Every damage changes, so: the
    winnable tests' recorded playthroughs (`crates/ui/tests/it/ch01_winnable.rs`
    to `ch04_winnable.rs`; each header says how to record on purpose),
    then the bots on each chapter, both seed ranges, inside their bands
    (`cargo xtask playtest chNN --bot all`; `cargo shuyi autobalance
    --stage chNN` where a chapter leaves its band), as 0524 (Chapters 1
    and 2) and 0855 (Chapter 1 after the armour weights) did. The tables
    go in `docs/playtesting.md`.

**Out (do not do):**
- The EXP cut for a level gap (0610), Joss's stats (0866) and who wields
  the Houndmaster's Axe (0332). **Either order with each:** whichever
  lands second re-records the playthroughs and re-runs the bands with the
  first in.
- A forecast redesign (0480, asks Nick). **Either order:** if 0480 is done
  first, a new mark goes where 0480 put the strike's notes; if not, add a
  line to 0480 naming the mark.
- New Combat Arts or skills unless 0080 picked one.

## Implementation steps

1. Read 0080's answer, `combat.rs` (`side`, `CombatRules`,
   `CombatMods`), its tests (`crates/core/src/combat/`), and the done
   tickets 0524 and 0855 for how a re-balance was run.
2. Build the rule (A, B) or the data (C, D, E). Keep every number a field
   of `CombatRules` or the data (ADR-0004: `core` stays pure).
3. Update the design docs and the doc comment.
4. Forecast mark, if any, in both skins, with snapshot tests.
5. Re-record the four playthroughs in order (1 → 4), reading each new
   fixture's last lines (who fell, how many turns).
6. Run the bots on Chapters 1 to 4; autobalance a chapter that leaves its
   band; record the tables.

## Acceptance criteria

- [ ] The rule is what 0080 decided, in `CombatRules` or the data, and
      the design docs give a worked example with this game's numbers.
- [ ] A unit test: Aske (Iron Bow) against a Lv 5 `hound_guard` on a
      plain gets the damage 0080's rule gives (not 0, if that is the
      answer).
- [ ] Property tests (below) pass.
- [ ] `frame-png` pictures of the forecast of that attack, glyph and
      Playdate skins, are in the PR.
- [ ] The four winnable tests pass with re-recorded playthroughs, and the
      bots are in their bands on Chapters 1 to 4 (tables in
      `docs/playtesting.md` and the PR).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the worked examples, in `crates/core/src/combat/` tests.
- Property (`proptest`, as the combat tests do): a minimum never lowers
  damage (new damage ≥ old damage for every input); a hit that lands
  under the rule deals at least the minimum; damage still never rises
  with the defender's Def; nothing changes for an input the rule doesn't
  cover (e.g. a weapon without the trait).
- Snapshot: the forecast with the new mark, if there is one.
- Integration: the winnable tests of Chapters 1 to 4; the bots' runs.

## Completion notes

*(Filled in by the session that completes the ticket.)*
