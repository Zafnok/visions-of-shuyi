---
id: "0489"
title: "Vix explains weapon effectiveness, and the screens say what each weapon kind is good against"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-10
---

# 0489 — Vix explains weapon effectiveness, and the screens say what each kind is good against

## Context

Nick's playtest of the Act 1 build (2026-11-08):

> "Vix needs a tutorial on effectiveness of weapons, only I know that
> spears are good against horses"

The rules (`docs/design/weapons-and-items.md` ~:158–176, *Weapon types and
traits*; Nick's, from Fortune's Weave): there is **no weapon triangle**.
Each kind has one trait (`assets/data/items.ron` ~:48–54; core
`CombatRules::type_trait`, `crates/core/src/combat.rs` ~:261, and
`ItemTable::trait_of`, `crates/core/src/item.rs` ~:420):

| Kind | Trait | In data |
| ---- | ----- | ------- |
| Sword | follow-up strikes ×6/5 | `SwordFollowUp` |
| Spear | weapon might ×2 against **Mounted** | `Effective(Mounted, 2)` |
| Axe | a hit deals at least 5 | `AxeMinDamage(5)` |
| Bow | weapon might ×3 against **Flying**; range 2 only | `Effective(Flying, 3)` |
| Gauntlet | +15 avoid while equipped | `GauntletAvoid(15)` |

Where it shows today: only a `!` after an effective strike in the
forecast (`crates/ui/src/screens/battle/glyph/forecast.rs` ~:137 and
~:243; the Playdate's `screens/battle/playdate/forecast.rs`). That `!` also
shows for a spell against an element the target is Weak to
(`combat.rs` ~:438, `docs/design/magic.md` ~:345). No screen says a kind's
trait; no lesson or tip mentions it (`assets/data/lessons.ron`,
`assets/data/tips.ron`, `docs/design/tutorial.md`). An art's own
effectiveness already has words: `×{mult} vs {tag}`
(`battle.art.desc.effective`, `ui.ron` ~:611; `art_list.rs` ~:129), and
`battle.art.desc.followup`, `battle.art.desc.axe_min` too.

Where it first matters in Act 1: Chapter 1 has a `red_rider` (Rider, Iron
Spear) and spear-carrying guards against Tamsin (a Rider, Mounted), and
Hollis's spear against the rider (`assets/battles/ch01.ron`,
`assets/data/characters.ron` ~:202).

**Text.** Nick paused AI story writing on 2026-11-08. A tip is
instructional: write it in plain teaching words, accurate to the rules
above. The hired writer (0766) may revise Vix's wording later.

## Nick input

None. A **tip** at Full, like 0483's wading tip (*the agent's starting
rule*: one fact with nothing to tick off; tips show only at Full,
`tutorial.md` *Tips*). Players at Short and None learn it from the
forecast's note and the unit info screen below, which show at every
level. Both go on the Act 1 playtest's checklist (1064). Verified by the
tests below and `frame-png` pictures of the forecast with the note and
the info screen in the PR.

## Scope

**In:**
1. **A trigger** `FirstEffective` (`crates/content/src/tip.rs`,
   `TipTrigger`): the forecast is open (`Mode::Targeting`) and a strike of
   either side is effective **by a weapon kind's trait** (spear vs
   Mounted, bow vs Flying), not by an element. A doc comment in
   `TipTrigger` and the header of `tips.ron`. Fire it beside
   `FirstForecast` (`crates/ui/src/screens/battle/mod.rs` ~:919). Decide
   it with a small helper reading the attacker's and defender's weapons
   and class tags (e.g. in `screens/battle/forecast.rs` or `doable.rs`),
   not from the `!` alone.
2. **A tip** `weapon_kinds` in `tips.ron`: at most 3 lines of 60
   characters, naming no key, saying that a spear's might counts double
   against riders and a bow's triple against fliers, that it works for
   the enemy too, and that the `!` in the forecast means it applies. A
   draft (fits the limits; check every word against the table):

   ```
   A spear's might counts double against riders, a bow's
   triple against fliers, yours and theirs alike. A ! in
   the forecast means it's working; a unit's details say more.
   ```
3. **The forecast says why.** When a strike is effective by a kind's
   trait, the forecast's notes get one line, e.g. `Spear ×2 vs Mounted`
   (the kind's name `battle.kind.*` and the existing
   `battle.art.desc.effective`), in the glyph and Playdate forecasts
   (both paint `view.notes`).
4. **The unit info screen names each kind's trait** on its Weapon ranks
   rows (`screens/battle/info.rs` ~:158; the glyph skin and the
   Playdate's `playdate/info.rs`): `Spear  D  ×2 vs Mounted`,
   `Sword  D  follow-up ×6/5`, `Axe  E  min damage 5`, `Bow  D  ×3 vs
   Flying`, `Gauntlet  E  avoid +15`, reusing the art description keys
   where they exist (a new key for the gauntlet's avoid if none fits).
   If a row doesn't fit its column, the trait goes on a row under it.
5. `tutorial.md`: the tip in the *Tips* list with the starting rule;
   `weapons-and-items.md`: one sentence that the forecast and the info
   screen name the traits.

**Out (do not do):**
- Any rule change, or a weapon triangle.
- A checklist lesson about it.
- The shop's lines (0486).
- The forecast's layout (0480, asks Nick). Either order: if 0480 is done
  first, put the note where its layout keeps notes.
- Vix's voice clip: if 0476 (Vix speaks her tips) is done, add the line to
  its clip list and generate it (`docs/voice.md`); if not, 0476 voices
  every tip that exists then.
- The wading tip (0483): either order; both add a `TipTrigger`, the second
  rebases.

## Implementation steps

Expected size: about 120–180 production lines (a trigger and its helper,
a tip, a forecast note, the info rows in two skins), plus tests.

1. Write the failing tests first (below).
2. Trigger and helper; fire it; the tip.
3. The forecast note; the info screen rows, glyph and Playdate.
4. Docs; snapshots of the forecast and the info screen.

## Acceptance criteria

- [x] Opening the forecast of Hollis's spear on a Rider fires
      `FirstEffective` once and the `weapon_kinds` tip shows at Full
      (Harness test on a test battle with a rider).
- [x] Opening the forecast of Tamsin attacking a spear guard (the
      counter is effective) fires it too (test).
- [x] A Fire spell on a unit Weak to fire doesn't fire it (test).
- [x] At Short, no tip; the forecast note still shows (test).
- [x] The forecast's notes have `Spear ×2 vs Mounted` on an effective
      spear strike and none on a plain one (view test).
- [x] The info screen's Weapon ranks rows show each kind's trait (view
      test; snapshot).
- [x] The tip passes the tip-text checks and names no key.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the helper (spear vs Mounted, bow vs Flying, spear vs foot, a
  Weak spell).
- Integration: the Harness tests above (`screens/battle/tip_tests.rs`).
- Snapshot: the forecast and info screen snapshots (glyph and Playdate).

## Completion notes

- **Trigger** `FirstEffective` (`crates/content/src/tip.rs`), fired with
  `FirstForecast` while an attack's forecast (`Mode::Targeting`) has a
  strike effective by its weapon kind's trait. Decided by
  `forecast::kind_effects`, which reads the striker's weapon kind, the
  kind's trait from the item data and the target's class tags, for the
  attacker and for the target's counter if it has one; a spell is never
  a weapon kind, so a Weak target adds nothing.
- **Tip** `weapon_kinds` ("Weapon kinds", face `sly`) with the ticket's
  draft text word for word.
- **Forecast note** `Spear ×2 vs Mounted` (new key
  `battle.forecast.kind_trait`, `{kind} {trait}`, over the existing
  `battle.kind.*` and `battle.art.desc.effective`); both skins already
  paint `view.notes`. The same note from both sides shows once.
- **Info screen**: `InfoView::ranks` is now `InfoRankView` (kind, rank,
  trait). The trait words reuse `battle.art.desc.followup`,
  `.effective`, `.axe_min`, and `battle.mods.avoid` for the gauntlet
  (`+15 avo`, matching the panel's AVO), so no new key there.
  - PC: the trait follows the rank, dim, ending a cell before the middle
    column; the rank column moved one cell left (`Sword    D`) so every
    current trait fits on its row. A longer one goes on the row under.
  - **Playdate: deviation.** Page 2 was full to the frame, so the weapon
    ranks (one per row, with the trait) and the affinities moved to a new
    page 3; the skills and spells are page 4 (`◀ 1/4 ▶`).
    `docs/design/playdate.md` updated.
- Docs: `tutorial.md` (*Tips*), `weapons-and-items.md`, `playdate.md`.

Starting rules for the Act 1 playtest (1064), all *the agent's*:
- The tip shows the first time a forecast has an effective weapon-kind
  strike from either side, after the forecast tip if both are new.
- It shows only for attack forecasts, not spell forecasts (a spell has no
  weapon kind).
- The Playdate's unit info has a fourth page: weapon ranks with their
  traits, and the affinities.
- The gauntlet's trait reads `+15 avo`.

No follow-up tickets. Vix's voice clip: 0476 isn't done, so it voices
this tip with the rest.
