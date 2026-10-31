---
id: "0044"
title: "Decide: the Preparations shop's basic items and how its tiers unlock"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-21
---

# 0044 — Decide: the Preparations shop's basic items and tiers

## Context

While building the Preparations screen (ticket 0408, 2026-10-01) Nick said
the screen should also have a shop, and how it differs from other shops
(`docs/design/weapons-and-items.md`, *Battle pack*):

> "The preparation screen, when it comes, which could be ch2, should also
> have a basic shop option -- some shops might sell unique items, this shop
> won't. It will only sell up to the tier of items you have unlocked as a
> basic. So for the beginning it might be Potion (heal 20) for the mid it
> might be a consumable to heal 40, etc."

and that from then on the player brings only their own consumables ("from
i.e. shops or battles or collecting or opening chests"). `world-structure.md`
already says Preparations has a shop from Chapter 2 and that towns'
inventories grow with the story; shop rules (gold, buying, selling) are in
`weapons-and-items.md`, *Money and shops*, and built in core by 0308.

What is not decided: which items are "basics", what the tiers are, and what
unlocks a tier. Ticket 0443 (the shop tab) waits for this.

Chapter 1 has no Preparations and no shop (`chapter-1.md`), so it waited
for the Chapter 1 playtest (0804). Chapter 2 brings Preparations and
its shop (`docs/story/outline.md`), so it is asked now, in the Act 1
design queue (re-gated 2026-10-21, 0129).

## Nick input

**Decision**, with the `ask-nick` skill. At most three questions per
message:

1. **What unlocks a tier?** Story progress (a chapter number, as FE's
   between-chapter armouries grow), the world map's towns (the best town
   shop visited), the army's level, or something bought or found. Say what
   each means for a player who falls behind.
2. **What does the basic shop sell?** Consumables only (his examples are
   healing items), or also basic weapons, armour and accessories (Iron,
   then Steel)? If weapons are sold here, does it repair too, or is that
   only the Blacksmith?
3. **The healing items per tier.** His examples are "Potion (heal 20)" and
   "a consumable to heal 40"; today's Potion heals 10 and the Elixir heals
   everything (`weapons-and-items.md`, *Chapter 1 consumables*, tunable,
   and all numbers rescale with 0013). Propose a tier table (name, heal,
   price) and ask whether lower tiers stay on sale.

Also ask (it came up in the same answer, as a "maybe"): should **green
units carry their own consumables**, not drawn from the player's pack?

## Scope

**In:**
- The decision, recorded in `docs/design/weapons-and-items.md` (his words,
  the rules, numbers marked Nick's or *tunable*) and the
  `docs/design/README.md` table.
- Updating ticket 0443's steps to match.
- `write-ticket` for anything else he chooses (new consumables in
  `items.ron`, green units' items).

**Out (do not do):**
- Any code or data.

## Implementation steps

1. Read `weapons-and-items.md` (*Battle pack*, *Money and shops*),
   `world-structure.md` (towns) and `chapter-1.md`.
2. Ask with `ask-nick`; check each option against `assets/data/items.ron`.
3. Record the answer; update 0443.

## Acceptance criteria

- [x] `weapons-and-items.md` says which items the Preparations shop sells at each tier and what unlocks a tier, with Nick's words.
- [x] `docs/design/README.md` row updated.
- [x] Ticket 0443 matches the decision.

## Tests required

- None (documentation).

## Completion notes

Asked Nick in two rounds (2026-10-21) and recorded it in
`weapons-and-items.md` (*Nick's words*, *Battle pack*, *Consumables*, the
new *The Preparations shop* section, *Durability*, *Open sub-questions*)
and the `docs/design/README.md` row.

- **Unlocks:** by chapter ("90% of the upgrades"); now and then a special
  unlock such as support rank A (open, chosen with the supports' content).
- **Stock:** consumables, Iron then Steel weapons, basic armour, and
  repairs. Chapter 2: Potion, Iron weapons, Leather Vest, Chain Mail,
  Warded Robe, repairs. Chapter 5: Steel weapons, Iron Plate. Chapter 8:
  Hi-Potion. Act 2's final battle or Act 3's start: Elixir. Lower tiers
  stay on sale.
- **Heals:** Potion 20 (was 10), Hi-Potion 40 (900 gold), no Mega Potion;
  the Elixir stays rare (a handful in Act 1, a few handfuls in Act 2).
- **Green units** carry their own consumables, set per battle.

*the agent's starting rules* (Nick may veto): the three basic armours at
Chapter 2 and Iron Plate with Steel; accessories never basics; the shop
buys at half price like every shop; town shops don't sell the Elixir in
Act 1 either; green units use items like player units (self or adjacent),
can't trade them, lose the unused ones after the battle, and (0513) use
one on themselves below half HP when they can't kill.

Follow-ups: 0443 rewritten (chapter-based stock data, repairs, Potion 20
and the Hi-Potion in `items.ron`); new ticket **0513** green units carry
their own consumables (in 1064's `blocked_by` and the roadmap).

Review on PR #385 (Nick, 2026-10-21): the basic shop buys at half price,
and **specialty stores** (an Alchemist for consumables, a Tailor for cloth
and leather armour) buy their own goods at full price; recorded in *Money
and shops*, built by new ticket **0320**. Nick also proposed **four armour
weights** by material (cloth, leather, chain, plate); that changes which
classes wear what, so it is its own decision, new ticket **0073**, and the
basic shop's armour rows follow it.
