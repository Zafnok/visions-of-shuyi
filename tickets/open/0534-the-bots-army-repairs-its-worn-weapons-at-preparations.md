---
id: "0534"
title: "The bots' army repairs its worn weapons at Preparations"
type: feature
milestone: M4 Enemy AI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0534 — The bots' army repairs its worn weapons at Preparations

## Context

Found by 0080 (how the army gets through armour), which printed the army
the bots bring to Chapters 2–4 (`trpg_bots::arrival`, Classic, at
`assets-private.rev` e6606a53): nobody ever repairs a weapon, so by
Chapter 4

- Tamsin's only weapon, her Iron Spear, is at **0/20** (broken: half
  might, hit −20, no arts; `weapons-and-items.md`, *Durability*);
- Aske's only weapon, her Iron Bow, is at **2/20**, and the lead's Iron
  Sword at 2/20.

A player would repair them: the Preparations shop repairs from Chapter 2
(`weapons-and-items.md`, *The Preparations shop*, Nick's option C; a Steel
Bow at 10/25 costs 216 gold to repair). `arrival::prepare`
(`crates/bots/src/arrival.rs` ~:251) fills only the pack: "Weapons,
armour and loadouts stay as they are" (*the agent's starting rule*,
`docs/playtesting.md`, *The army*). So the bands of Chapters 3 to 5 are
measured with an army weaker than any player's, and every balance ticket
of Lane I (0333 above all: it changes what bows and spears do) reads
them.

## Nick input

None. Verified by tests and the bots' tables in `docs/playtesting.md`.

## Scope

**In:**

- `prepare` repairs before it fills the pack, when the battle's shop
  repairs (`ShopKind::repairs`): every weapon the fielded units carry
  whose durability is under **half** its maximum, the lowest share first,
  while the gold pays for it (`shop::repair`, `crates/core/src/shop.rs`,
  as `Preparations::repair` does, `crates/core/src/prep.rs` ~:664; the
  cost is `shop::repair_cost`). Then the pack as today, with
  the gold left. (*The agent's starting rule*, tunable: a player repairs
  what is nearly worn out, not every chip.)
- A broken weapon (0 left) is repaired first among equal shares.
- `docs/playtesting.md`, *The army*: the sentence on Preparations says
  the army repairs its worn weapons, then fills the pack.
- Re-run the bots on Chapters 3, 4 and 5 (both seed ranges, `--bot all`)
  and record the tables. If a chapter leaves its band, say so in the PR
  and the tables; autobalancing it back is **0876**'s or the next balance
  ticket's job, not this one's.
- If a recorded playthrough no longer wins with the repaired army (a
  stronger hit kills earlier and a later command finds no target),
  re-record it and say so. Since 0531 the bots' army comes from their
  own 24 lines of recorded wins (`crates/bots/wins/`, ADR-0130): the
  playtest names the battle and line that stopped winning, and `cargo
  xtask bot-wins <battle>` records it and every battle after it.
  `trpg-ui`'s `chNN_winnable.rs` fixtures don't move the bots' army.

**Out (do not do):**
- Buying new weapons or armour for the army, or changing loadouts.
- The Steel weapons (sold from Chapter 5) or any shop stock.
- Balancing a chapter back into its band (0876, the Lane I tickets).

## Implementation steps

1. Read `arrival.rs` (`prepare`, `setup`), `prep.rs`
   (`Preparations::repair`, `WeaponAt`) and `shop.rs` (`repair`,
   `repair_cost`, `ShopKind::repairs`).
2. Add the repair pass at the start of `prepare`, with the threshold as a
   named constant and a comment saying it is the agent's starting rule.
3. Unit tests in `crates/bots/src/arrival/tests.rs`.
4. Run the bots on Chapters 3–5 and update `docs/playtesting.md`.

**Expected size** (ADR-0101): about 40 production lines, 80 of tests,
plus the playtesting tables.

## Acceptance criteria

- [ ] A test: the army arriving at Chapter 4 has no weapon under half
      its durability (Tamsin's Iron Spear is no longer broken), when the
      gold pays for it.
- [ ] A test: with too little gold, the lowest share is repaired first and
      the pack gets what is left.
- [ ] A test: a battle whose shop doesn't repair (or that has no shop)
      changes no weapon.
- [ ] The bots' tables for Chapters 3–5 after the change are in
      `docs/playtesting.md` and the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the repair pass (threshold, order, gold, no-repair shop).
- Integration: the arrival tests of Chapters 3–5; the bots' runs.

## Completion notes

*(Filled in by the session that completes the ticket.)*
