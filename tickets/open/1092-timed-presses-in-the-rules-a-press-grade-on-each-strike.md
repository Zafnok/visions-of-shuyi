---
id: "1092"
title: "Timed presses in the rules: a press grade on each strike, recorded in the command, Good when auto-resolved"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0086", "1064"]
nick_input: answer-first
completed:
---

# 1092 — Timed presses in the rules

## Context

Nick's search for the game's storefront hook (2026-11-16) chose
legendary beasts fought as a raid, on top of three battle systems that
make the enemy's turn the exciting part. The second is **timed presses**,
as in Expedition 33: a press on attacks, follow-ups, parries and dodges.
**The hit and crit rolls stay as they are** (FE true hit,
`docs/design/stats-and-combat.md`, *RNG procedure*); the press grade
(miss, good, perfect) only changes what a landed or blocked blow does.
Auto-resolve and the Off setting count every press as Good. The chain is
0085–0088, 0772, 1091–1099 and 1275–1281; this ticket is the rules half,
1093 the timing ring and bar in the combat frame, 1094 Steal the Turn
(a perfect parry freezes the enemy phase).

**The rules come from 0086's answers** in `docs/design/timing.md`: what
each grade does on an attack, a follow-up, a dodge and a parry, which
blows prompt, and the numbers (all *tunable*). Where they differ from the
sketch here, the decision wins.

Because the rules are deterministic (ADR-0004) and rewind, replays and
the bots replay commands (`crates/core/src/history.rs`, ADR-0033), **a
press is recorded as part of the command**. This ticket decides how;
an ADR records it.

What exists today:
- `crates/core/src/combat.rs`: `resolve(rules, forecast, attacker,
  defender, rng)` plays a combat strike by strike (`strike_order`,
  `roll_hit`, the crit roll, a Deflect's `strike_back`) and returns
  `CombatOutcome { strikes: Vec<Strike>, .. }`; `Strike` records `by`,
  `hit`, `crit`, `damage`, `healed`, `reaction`. Its numbers are in
  `CombatRules` (built by `ItemTable::combat_rules`,
  `crates/core/src/item.rs`).
- `crates/core/src/battle.rs`: every combat goes through
  `BattleState::clash` (an attack's combat, an art's extra strikes, an
  Overwatch shot during an enemy's move). `Command` has `Act`, `Equip`,
  `MoveAfter`, `Talk`, `EndPhase`; about 200 places build a
  `Command::Act`, most of them tests.
- The AI (`next_command`, `crates/core/src/ai.rs`), the playtest bots
  (`crates/bots/src/`), the title cinematic's fight
  (`crates/content/src/cinematic/combat.rs`) and the battle screen
  (`crates/ui/src/screens/battle/mod.rs`, `apply`) all send commands.

## Nick input

**Answer first:** ticket 0086. Nothing after merge: verified by tests
and the bots.

## Scope

**In:**
- The press types, which strikes are due a press, what each grade does
  (0086's rules), and the grade each strike used recorded in its
  `Strike`.
- The presses carried by the command, missing ones counted as Good.
- A rehearsal the battle screen can call to learn what a command would
  do with the presses made so far (1093 builds the screen on it).
- An ADR: a timed press is part of the command.

**Out (do not do):**
- Anything on screen: the ring, the bar, the press key, the Timed
  presses option (1093).
- Steal the Turn (1094). Beast parts (1096, 1097).
- Bots that press anything but Good (a persona that presses is a
  ticket of its own if 0086 asks for one).

## Implementation steps

Expected size (ADR-0092): about **350 production lines** in `core`
(types, the press feed in `resolve`, the command, its checks, the
rehearsal) and an ADR. Comparable: 0326 (stances that react, in
`resolve` and `clash`). Tests about 500 lines.

1. Read `docs/design/timing.md`, `stats-and-combat.md`, ADR-0004,
   ADR-0020, ADR-0033 and `crates/core/src/combat.rs`'s module docs.
2. **Types** in `combat.rs`: `PressGrade { Miss, Good, Perfect }`,
   `PressKind` (the presses 0086 lists, for example `Strike`, `Dodge`,
   `Parry`) and `Press { kind, grade }`. `Strike` gets
   `#[serde(default)] press: Option<Press>`: the press the strike used
   (`None`: none was due). The numbers 0086 sets go in `CombatRules`
   with their defaults.
3. **Which strikes are due a press**: one function, `press_due` (who
   strikes whom, their factions, the strike's damage if it hits and the
   target's HP) `-> Option<PressKind>`, from 0086's rule (every blow at
   or by a player unit, or only big blows: one that would kill if it
   hit, one over a share of max HP, a boss's). Unless 0086 says
   otherwise it doesn't depend on whether the blow hits, so the ring
   never gives the roll away; a grade on a missed blow changes nothing.
   Strikes between two units that are not the player's are never due
   one. The screen shows the ring on exactly these strikes (1093).
4. **`resolve` with presses.** `resolve_pressed(rules, forecast,
   attacker, defender, rng, presses: &mut impl FnMut(PressKind) ->
   PressGrade)`; `resolve` stays, with every press Good. The hit and
   crit rolls are taken as today, before the grade is read, so a grade
   **never turns a miss into a hit** and changes no roll before its own
   strike. The grade then changes the damage, blocks the blow or adds
   what 0086 says. Update the module docs' rules block.
5. **The command.** Add `Command::Pressed { presses: Vec<Press>,
   command: Box<Command> }` rather than a field on `Act`, so the ~200
   existing `Act`s and every old play record stay as they are. Only an
   `Act` may be wrapped (`CommandError::PressesOnAct` otherwise, and
   for a `Pressed` in a `Pressed`). The battle reads the list in strike
   order across the whole command (every `clash`, Overwatch shots
   included); a strike due a press with no entry left counts as Good;
   entries left over are ignored. A press whose kind isn't the one the
   strike allows counts as Miss. Keep the feed as a transient,
   `#[serde(skip)]` field set for one `apply` (or pass it down `act` →
   `attack` → `fight` → `clash`; the PR says which).
6. **Rehearsal.** `pub fn rehearse(&self, cmd: &Command) ->
   Result<Vec<Event>, CommandError>`: the events `cmd` would give,
   from a copy. Document the promise 1093 builds on: the events up to
   and including strike *k* depend only on presses 1..*k*.
7. **Everyone else** sends what they send today: the AI, the bots, the
   cinematic and the battle screen send unwrapped commands, so every
   press is Good. If 0086 makes Good the same as today's numbers, the
   forecast (`forecast`, `AttackPreview`), the AI's scoring and the
   bots' bands don't change; if not, the forecast shows Good's numbers
   and the PR says how the bands moved.
8. **The ADR** (`write-adr`): presses in the command, Good by default,
   the prefix promise, why a wrapper and not a field.

## Acceptance criteria

- [ ] `a_press_never_turns_a_miss_into_a_hit` and
      `a_press_changes_nothing_before_its_own_strike` (property tests in
      `crates/core/src/combat/tests.rs`).
- [ ] Unit tests for each grade of each press kind 0086 defines (for
      example `a_perfect_parry_blocks_the_blow`,
      `a_good_dodge_halves_the_damage`), reading `Strike::press`.
- [ ] `missing_presses_count_as_good` and `an_unwrapped_act_fights_as_today`
      (the outcome equals today's `resolve` for the same seed).
- [ ] A `Pressed` around anything but an `Act` is refused.
- [ ] `rehearse` then `apply` with the same presses give equal events
      (property test over generated battles).
- [ ] Rewind replays a battle with presses to the same state
      (`crates/core/tests/it/replay.rs`).
- [ ] The playtest bots' results on Act 1 are unchanged (Good = today),
      or the PR lists the change.
- [ ] The ADR is in `docs/adr/` and its index.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each grade's effect; `press_due`; the command checks.
- Property: the two promises above; determinism of a replay with
  random presses.
- Integration: rewind and replay with presses; a play record from
  before this ticket loads and replays unchanged.
- Bots: Act 1's bands.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
