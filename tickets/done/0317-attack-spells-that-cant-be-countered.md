---
id: "0317"
title: Attack spells the enemy can't counter (Cinder Oath)
type: feature
milestone: M2 Core rules
model: sonnet-5
effort: medium
status: done
blocked_by: ["0056"]
nick_input: sign-off
completed: 2026-10-15
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

- [x] A core test: an attack spell with `no_counter` against an adjacent
      defender with a range-1 weapon gives a forecast with no defender side
      and a combat with no counter strike.
- [x] A core test: the same defender attacking a unit with that spell
      equipped is still countered by it.
- [x] A content test: `no_counter` parses and defaults to `false`.
- [x] `cinder_oath` has `no_counter: true`; the forecast in Chapter 1 shows
      `no counter` for it (a UI test or snapshot).
- [x] Sim result in the Completion notes; replay tests pass.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit (core): the two tests above, next to the Guard Break tests in
  `crates/core/src/battle/tests/`.
- Unit (content): parsing.
- Integration / snapshot (ui): the forecast shows `no counter`.

## Completion notes

- `SpellDef::no_counter` (core), parsed from `spells.ron` as `no_counter`
  (default `false`) and set on `cinder_oath`. `battle.rs` clears the
  defender's side of the forecast when the attack is such a spell, next to
  the art's check (one helper, `uncounterable`), in the `Act`'s planning
  and in the AI's cheap `plain_forecast` alike, so the enemy AI also knows
  Rue can't be struck back. The `crate::spell` module docs list the rule.
- Tests: core (`a_no_counter_spell_leaves_the_defender_no_counter`,
  `a_no_counter_spell_still_counters_when_attacked`, with a test spell
  `oath` in the battle tests' table), content
  (`no_counter_parses_and_defaults_to_false`, and Cinder Oath in
  `spells_match_the_design`), UI (`cinder_oath_shows_no_counter_in_chapter_1`:
  the shipped Chapter 1 with a brigand moved beside Rue; the forecast says
  `no counter`). The core tests sit in `battle/tests/spell.rs` with the
  other spell-combat tests rather than beside Guard Break's art tests.
- 0453 isn't done, so the spell list's note is left to it; its scope now
  names `no_counter`.
- Chapter 1 sim (`cargo xtask playtest ch01 --runs 200`, baseline bot):
  won 49 of 200 (25%) with the rule, 60 (30%) with the same build and the
  flag off; turns median 8 and fallen mean 3.5 both ways; Rue fell in 49
  tries vs 53. The win gap is within what reshuffled dice give at 200 tries
  (one fewer counter roll changes every roll after it), so no balance
  change is read from it. The winning replay (`ch01_winnable`) passes
  unchanged.
- No new *Claude's starting rule*: the one this ticket implements (only
  the attacker's spell counts) was already written in `magic.md`.
