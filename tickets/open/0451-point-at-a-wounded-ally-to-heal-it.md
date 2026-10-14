---
id: "0451"
title: Point at a wounded ally to heal it, like pointing at an enemy
type: feature
milestone: M3 Battle UI
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
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

- [ ] `turn-structure.md` and `magic.md` record Nick's answers.
- [ ] Harness (Chapter 1 or the Quick Battle): a healer selected, cursor on
      a hurt ally two tiles away → the path ends next to them; Confirm →
      the heal preview on that ally; Confirm → the ally's HP went up and the
      healer has acted (read the state).
- [ ] Harness: an ally at full HP is not a target (the cursor behaves as
      before).
- [ ] Harness: left / right swaps Heal and Vigil Light for Maud.
- [ ] Pointing at an enemy still works (existing tests pass unchanged).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the shared tile choice for a heal reach.
- Snapshot / integration: the Harness tests above; one snapshot of the
  heal preview opened by pointing.

## Completion notes

*(Filled in by the session that completes the ticket.)*
