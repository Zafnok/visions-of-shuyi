---
id: "0317"
title: Attack spells the enemy can't counter (Cinder Oath)
type: feature
milestone: M2 Core rules
model: sonnet-5
effort: medium
status: todo
blocked_by: ["0056"]
nick_input: sign-off
completed:
---

# 0317 — Attack spells the enemy can't counter (Cinder Oath)

## Context

Ticket 0056 (2026-10-15): Nick kept Rue's Cinder Oath at range 1 "because of
flavor", with the enemy unable to strike back at it (`docs/design/magic.md`
→ *Personal signature spells* → *Cinder Oath: range 1, no counter*;
`docs/story/characters/heretic.md`). Spells have no such rule today. Arts
do: Guard Break's `ArtEffect::no_counter` (`crates/core/src/art.rs`), applied
in `crates/core/src/battle.rs` (search `effect.no_counter`) by setting
`forecast.defender = None`, which the forecast panel and the combat
picture already show as `—  no counter`.

## Nick input

**Sign-off:** in Chapter 1, have Rue attack an enemy next to her with
Cinder Oath. The forecast's enemy side says `no counter`, and the enemy
doesn't strike back.

## Scope

**In:**
- A `no_counter: bool` field on `SpellDef` (`crates/core/src/spell.rs`),
  written in `spells.ron` as `no_counter: true` and defaulting to `false`
  (`#[serde(default)]` on `RawSpell` in `crates/content/src/spell.rs`).
- In the attack planning in `battle.rs`, when the attacker's attack is a
  spell with `no_counter`, set `forecast.defender = None`, exactly as the
  art does (the result must be the same whether the no-counter comes from
  an art or a spell).
- `no_counter: true` on `cinder_oath` in `assets/data/spells.ron`; replace
  the comment above it ("once the engine has the rule (0317)") with the
  plain rule.
- Update the `crate::spell` module docs (*Rules*) with the new rule.
- Only the attacker's spell counts: if Rue has Cinder Oath equipped and is
  attacked from the next tile, she still counters with it (`magic.md`,
  Claude's starting rule).

**Out (do not do):**
- Kept Promise and what triggers it (open, Nick: after the Chapter 1
  playtest).
- Any change to Cinder Oath's numbers or range.
- A note for the spell list's description row (0453 builds that from the
  spell's data; if 0453 is done first, add the `no counter` note there in
  this ticket; if not, 0453 picks the field up).

## Implementation steps

1. Add `pub no_counter: bool` to `SpellDef` with a doc comment; fix every
   `SpellDef { .. }` literal (content tests, core tests, fixtures).
2. Parse it in `crates/content/src/spell.rs` (`RawSpell`, default `false`).
3. In `battle.rs`, next to the art's `no_counter` check, also clear
   `forecast.defender` when the attack is a spell whose def has
   `no_counter` (find the spell from the attacker's `with`/loadout the same
   way the surrounding code finds the attack).
4. Set it on `cinder_oath` in `spells.ron`.
5. Run the Chapter 1 sim (`cargo xtask playtest ch01 --runs 200`) and the
   winning-replay tests; if the replay breaks, re-record it as
   `crates/ui/tests/it/ch01_winnable.rs` says. Write the sim result in the
   Completion notes.

## Acceptance criteria

- [ ] A core test: an attack spell with `no_counter` against an adjacent
      defender with a range-1 weapon gives a forecast with no defender side
      and a combat with no counter strike.
- [ ] A core test: the same defender attacking a unit with that spell
      equipped is still countered by it.
- [ ] A content test: `no_counter` parses and defaults to `false`.
- [ ] `cinder_oath` has `no_counter: true`; the forecast in Chapter 1 shows
      `no counter` for it (a UI test or snapshot).
- [ ] Sim result in the Completion notes; replay tests pass.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (core): the two tests above, next to the Guard Break tests in
  `crates/core/src/battle/tests/`.
- Unit (content): parsing.
- Integration / snapshot (ui): the forecast shows `no counter`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
