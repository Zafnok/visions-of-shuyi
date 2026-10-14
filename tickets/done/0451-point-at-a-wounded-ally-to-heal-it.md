---
id: "0451"
title: Point at a wounded ally to heal it, like pointing at an enemy
type: feature
milestone: M3 Battle UI
model: opus-5.5
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-14
---

# 0451 — Point at a wounded ally to heal it

## Context

Nick, Chapter 1 playtest round 2 (0804, 2026-10-14): "healers should have
the same hover over friendly unit, click, means heal, like units have for
enemy attacks..."

Today, with a unit selected, moving the cursor onto an enemy it can attack
aims the path at a tile it can attack from, and Confirm walks there and
opens the forecast (`docs/design/turn-structure.md`, *Pointing at an
enemy*; tickets 0428, 0430; code in `crates/ui/src/screens/battle/mode.rs`
and its `attack_tests.rs` / `magic_tests.rs`). Allies are healed only
from the action menu: move, then `Magic` → a heal spell → the ally
(`docs/design/magic.md`: "Tiles and allies are still cast on from
`Magic`"). Maud's Heal and Vigil Light (`assets/data/spells.ron`) are the
heals in Chapter 1.

## Nick input

**Decision (small, before building), with the `ask-nick` skill:** show the
starting rules below as player situations with an example (Maud and a
hurt Hollis in Chapter 1) and ask Nick to agree or change each; record the
answers in `turn-structure.md` (a *Pointing at an ally* part next to
*Pointing at an enemy*) and `magic.md`:

1. Which tile: the same rule as an enemy (keep the drawn arrow if it ends
   on a tile the heal reaches from, else the cheapest such tile, same
   tie-break).
2. What opens after Confirm: the heal's preview on that ally at once
   (`Heal on Hollis: HP 10 → 25`, what `Magic` shows today) with the
   equipped heal, or the first heal the unit knows; left / right swap the
   heal (Heal ↔ Vigil Light); Confirm casts, Cancel goes back.
3. Only an ally **who is hurt** is pointed at. Over an ally at full HP the
   cursor does what it does today (it is a tile the unit can't stop on).
4. A healing item (a potion) is **not** used this way; only heal spells
   (a unit that only carries a potion behaves as today).
5. A unit that can both attack and heal: enemies are attacked, allies
   healed; nothing else changes.

Then **sign-off:** in Chapter 1, select Maud and move the cursor onto a hurt
ally: she walks next to them and the heal opens.

## Scope

**In:**
- The decision above, recorded.
- Pointing at a hurt ally with a unit that knows a heal spell with uses
  left: the path aims at a tile the heal reaches from; Confirm walks there
  and opens the heal's preview; Confirm heals (the same `Command` the
  `Magic` menu sends).
- The help bar names the keys through the keymap (`keyboard-input` skill).

**Out (do not do):**
- Healing items, Trade, Talk or any other ally action by pointing.
- Changing how pointing at an enemy works.
- New heal spells or numbers.

## Implementation steps

1. Ask Nick (see *Nick input*); record the answers. Don't build before the
   answer.
2. Read how pointing at an enemy is built in `mode.rs` (find the aim and
   the target check used by 0428/0430) and the heal path of the `Magic`
   menu (`magic.rs`, `SpellChoice`, the heal preview). Reuse both; the
   tile choice should be one function shared with the enemy case, taking
   the reach of the heal instead of the attack.
3. Add the ally case: target check (ally, hurt, a heal reaches), aim, walk,
   open the heal preview, swap heals with left / right.
4. Help bar text for the new state, from the keymap.

## Acceptance criteria

- [x] `turn-structure.md` and `magic.md` record Nick's answers.
- [x] Harness (Chapter 1 or the Quick Battle): a healer selected, cursor on
      a hurt ally two tiles away → the path ends next to them; Confirm →
      the heal preview on that ally (through the heal list, Nick's answer); Confirm → the ally's HP went up and the
      healer has acted (read the state).
- [x] Harness: an ally at full HP is not a target (the cursor behaves as
      before).
- [x] Harness: ~~left / right swaps~~ the heal list offers Heal and Vigil
      Light for Maud (Nick chose a list instead of swapping).
- [x] Pointing at an enemy still works (existing tests pass unchanged).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the shared tile choice for a heal reach.
- Snapshot / integration: the Harness tests above; one snapshot of the
  heal preview opened by pointing.

## Completion notes

Nick's answers (2026-10-14), recorded in `turn-structure.md` (*Pointing at
an ally*), `magic.md` and the design README:

- **Which tile:** the closest tile **any** heal reaches the ally from, the
  same rule as an enemy (not "beside him for Heal"). So Maud two tiles from
  a hurt ally stays put and only Vigil Light can be used; draw the arrow
  next to the ally to use Heal.
- **What opens:** a **list of the unit's heals** first (not the preview
  straight away with left / right swapping). This changed the acceptance
  criterion about left / right: the list now offers both heals.
- **Up / down** change the ally, as in the enemy forecast.
- Hurt allies only; no potions; enemies attacked, allies healed: as
  proposed.

Built:

- `attack::aim_tile` is the one tile choice for both cases; `attack_tile`
  and the new `magic::heal_tile` call it with what reaches from a tile.
- `magic::heal_choices` builds the heal list for the ally pointed at (each
  heal's targets start at that ally); `can_heal` asks the core.
- `mode.rs`: the cursor aims at a hurt ally when no attack applies;
  Confirm walks there and opens the heal list (`Mode::SpellMenu`), which
  leads to the usual heal preview (`Mode::CastTarget`) and the same
  `Command::Act` cast the `Magic` menu sends.
- Help bar: `battle.help.aim_heal` (`arrows move · f heal · d cancel`, keys
  from the keymap), in English and the `test` pack.
- Tests: `screens/battle/heal_aim_tests.rs` (tile choice, targets, the
  heal list, harness runs, one snapshot of the preview opened by pointing).

*Claude's starting rules* (Nick can veto):

- In the list, a heal that can't reach the ally pointed at from that tile
  is dimmed, and the list opens on the first one that can.
- The list opens even with only one heal listed.
- Cancel: preview → list → action menu (on `Magic`) → path.
- The help bar says `heal` over a hurt ally the unit can heal.
- A green-side (Other) ally counts as an ally, as for `Magic`.

**Sign-off for Nick:** in Chapter 1, select Maud and move the cursor onto a
hurt ally: she walks to the closest tile a heal reaches from and the list
of her heals opens.
