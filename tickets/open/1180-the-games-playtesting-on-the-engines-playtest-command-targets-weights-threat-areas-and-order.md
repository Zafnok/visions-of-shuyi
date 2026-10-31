---
id: "1180"
title: "The game's playtesting on the engine's: cargo shuyi playtest, targets by tier, weighted features, threat areas and the candidate order; the xtask command and the workflow's loop go"
type: feature
milestone: "Engine: carve"
model: implementation
effort: high
status: blocked
blocked_by: ["0506", "engine#2129", "engine#2130", "engine#2131", "engine#2132", "engine#2532", "engine#2533", "engine#2021"]
nick_input: none
completed:
---

# 1180 — The game's playtesting on the engine's

## Context

Ticket 0506 built the persona playtest bots (Casual, Normal, Hardcore)
on the engine's rollout bot (engine#2125), playtest runner (engine#2126)
and turn planner (engine#2128); ADR-0095 is their design and
`docs/design/playtest-bots.md` the rules Nick decided (a bot knows the
danger zone and the forecasts, never the enemy's mind; Casual's
bands). It measured 1.8× (estimate 600, measured 1,060), under 2×, but
the game wrote a lot that isn't the game's: Nick (2026-10-22) asked
that it move into the engine, so other games get playtest personas and
a playtest command out of the box. The sweep of the playtest area that
followed (`docs/engine/friction.md`, rows "0506 (sweep)" and the
section *0506, 2026-10-22*) filed seven engine tickets. This ticket
moves the game onto them and deletes its own copies.

What each engine ticket replaces (game repository paths):

| Engine ticket | What it gives | What goes in the game |
| ------------- | ------------- | --------------------- |
| engine#2532 | `shuyi playtest` plays a game's playtest plan itself: stages, `--entrant` with groups, `--threads`, the game's defaults, time per try, `--report <path>` | `crates/xtask/src/playtest.rs`'s options, `Bot`, `play_on_threads`, `Record`/`Played`, `run`'s JSON file; the `playtest` arm of `crates/xtask/src/main.rs` |
| engine#2533 | the history (`was …`, previous date and commit), summary items over named measures, `--brief` | `render`, `Lines`, `median`, `was`, `history_file`, `last_record`, `append`, `git_commit`, `utc_stamp` |
| engine#2129 | `Targets<T>`: success rules, bands per tier, floors, medians; `checks(tier, entrants)` | `crates/bots/src/playtest.rs`'s `Targets`, `Faster` |
| engine#2130 | `Weights<N>`, `Weighted`, `Persona<X>` | `crates/bots/src/persona.rs`'s `Weights`, `Persona`, `Tally`'s fields and `Tally::score` |
| engine#2131 | `strike_area`, `Threats`, `Threats::danger`, `GoalDistance` | `persona.rs`'s `threats`, `Threat`, `Goals`, the sums in `danger` |
| engine#2132 | `ranked_order`, `TacticsOrder` with `OrderHooks` | `persona.rs`'s `order`, `advance`, `hold`, `attack_key`, `once_per_action` |
| engine#2021 | a reusable playtest workflow | the body of `.github/workflows/playtest.yml` |

What stays the game's: the counting (`Tally::between`: what a fall, an
HP lost, progress and approach mean), the forecast hook
(`BattleState::threat_forecast`), the lord rules in `danger`, the
classifier of commands (a strike's forecast, a heal's need, the enemy
AI's pick), the talk rule and `PersonaBot`, `BattleMeasures` and
`BattleMeasure` (the game's events and modes: accepted by the sweep),
`BattleEntrant`, and the data files `crates/bots/personas.ron` and
`crates/bots/targets.ron` (their shape may change; their values may
not).

## Nick input

None. Nothing a player sees changes, and the personas must play exactly
as before: verified by the tests and the bands below, not by Nick. What
is left to judge by playing is already on the Act 1 playtest's
checklist (1064).

## Scope

**In:**
- ~~Move the pin past the seven engine tickets (`engine-change` skill,
  step 3.6 and §7).~~ (done by the pin move of 2026-10-25, `3f2256d4` → `b4591a81`)
- `cargo shuyi playtest` is the game's playtest command: `Trpg` in
  `crates/xtask/src/game_tools.rs` implements `playtest_stages` (the
  battle ids of `assets/battles/`), `playtest_plan` and
  `playtest_defaults` (`seed: 1`, `turns: 60`, `tries: 100`, as
  `cargo xtask playtest` has today), with the game options `--mode
  classic|casual` (the baseline's mode).
- The plan: entrants `baseline`, `casual`, `normal`, `hardcore` with
  their mode as the label; the group `all` = the three personas; the
  default entrant `baseline`; checks from `Targets::checks` (engine#2129)
  for the battle's `difficulty`; summary items per entrant that print
  today's lines (won, deaths or retreats per try by mode, most fallen
  unit, items median and max, turns median).
- The bots on the engine's pieces (the table above).
- `.github/workflows/playtest.yml` calls engine#2021's workflow with the
  game's lever script as its `levers` input (`bash
  .github/playtest-levers.sh`, 0514), never a broad path list such as
  `assets/*` or `crates/core/*` (that would play on every dialogue,
  art and test change again, which 0514 stopped at Nick's request), with
  the concurrency engine#2021 documents (a push's run is never
  cancelled, 0515), and the macroquad packages. The script names
  `crates/xtask/src/playtest.rs`, which this ticket deletes: replace that
  line with the files the playtest plan now lives in.
- The enemy AI's walking distance (`flow` and `flow_field` in
  `crates/core/src/ai.rs`: a flow field per faction and movement type,
  the targets' tiles free to enter, the exit area in an escape) on the
  kit's `GoalDistance` too, so the bots and the AI share one, as
  `persona.rs`'s `Goals` does after step 4. The AI's tests and step 9's
  comparison stay unchanged.
- `trpg_core::threat_area` (`crates/core/src/movement.rs`) builds on the
  kit's `strike_area`, its tests unchanged.
- `docs/playtesting.md`, ADR-0095 (a line under *Decision* 6 and 3:
  where the runner, targets, weights and order now live), and
  `docs/engine/boundary.md` (the bots' row).
- The completion notes name the engine PRs.

**Out (do not do):**
- Changing any persona setting, weight, band, floor or success rule
  (ADR-0095 §5: only 0508's comparison with Nick's play changes them).
- Changing the measures or their names (0508 and 1021 read them).
- New playtest features (a campaign run is 0512's).

**Either order with 0450 and 0516:** 0450 gives one function for the
tiles a unit threatens, a stationary enemy's included; if it is done,
step 4's `Threats` takes stationary enemies from it rather than from
`persona.rs`'s own copy of the rule; if not, keep the rule as it is and
0450 moves it. 0516 fixes what `approach` and `enemy_fell` count in
`Tally::between`, which stays the game's here: either order, the fix
lives in `Tally::between`.

**Either order with 0507, 0508, 0509, 0512 and 1021:** each names
`cargo xtask playtest` or reads `crates/bots/personas.ron`. If one of
them is done first, its use of the command moves to `cargo shuyi
playtest` here (0509's autobalance and 1021's modkit report keep their
output; `playtest-compare` from 0508 keeps reading the personas file
through engine#2130's `Persona`). If this ticket is done first, each of
them is written against `cargo shuyi playtest`: update their text in
this ticket's PR.

## Implementation steps

1. **A baseline to compare with**, before anything changes: on the
   current pin (the pin move of 2026-10-25 changed no behaviour, so it
   stands for the old pin in step 9 and the acceptance criteria), run `cargo xtask playtest <Chapter 1's battle id> --bot
   all --runs 20 --json before.json` and the same without `--bot all`
   into `before-baseline.json` (the id is the one `assets/battles/`
   names Chapter 1). Keep both files outside the repo.
2. ~~**Pin.** Check each engine ticket is in the engine's `tickets/done/`,
   then `cargo xtask engine`, `git -C shuyi-engine switch main`,
   `git -C shuyi-engine pull --ff-only`, `cargo xtask engine --off`,
   `cargo xtask engine-pin`. Fix what the engine's breaking changes
   break (engine#2532 replaces `GameTools::playtest`).~~ (done by the pin move of 2026-10-25, `3f2256d4` → `b4591a81`)
3. **The scoring** (`crates/bots/src/persona.rs`): a `const FEATURES:
   [&str; 16]` with today's names (`won`, `lost`, `lord fell`, …; spaces
   or underscores, matching `personas.ron`'s keys); `Tally::between`
   returns `[i64; 16]` (or keep `Tally` as a thin newtype over it if
   tests read fields); `Scoring` becomes engine#2130's `Weighted`.
   `Persona` becomes `trpg_bots::Persona = shuyi_core::bots::weigh::Persona<PersonaGame>`
   with `PersonaGame { mode: GameMode }`; rewrite `personas.ron` into
   that shape with the same numbers (a test compares each persona's
   settings and weights with a table in the test).
4. **The danger** (`persona.rs` `danger`, `threats`): build engine#2131's
   `Threats<UnitId>` (stationary enemies with `strike_area_from`, the
   rest from their stop tiles and `attack_ranges` with `strike_area`),
   and sum with `Threats::danger` and a closure over
   `BattleState::threat_forecast` that returns engine `Strike`s. The
   caps (`exposed` at most the unit's HP, `counter` at most the enemy's
   HP), `at_risk` (`worst >= hp`) and the lord-alone rule stay here.
   `Goals` becomes `GoalDistance<u8>` keyed by the movement type.
5. **The order** (`persona.rs` `order`): an `OrderHooks` impl for
   `BattleState`: `prepare` builds the threats and goal distance;
   `classify` gives `Strike` (from `preview_attack`, value `(expected
   dealt, -expected taken)`, `way` = the action without its target and
   tile), `Extra { group: 0, value: missing HP }` for heals (a heal that
   moves ranks after one that doesn't, as today: fold it into `value`),
   `Step` for a `Wait`, `End` for `EndPhase`, `Drop` for `Equip` and
   `Talk`, `Other` for the rest; `pick` is `next_command` with the AI
   weights. `PersonaOrder` becomes `TacticsOrder<PersonaHooks>`.
6. **The targets** (`crates/bots/src/playtest.rs`): `Targets` becomes
   `shuyi_core::bots::playtest::Targets<Difficulty>` (derive `Ord` on
   `Difficulty` in `trpg-core`, a bot-only need: say so in its doc
   comment); `faster` becomes one `medians` entry in `targets.ron`; the
   game's `Targets::from_ron` stays as the reader. A test checks
   `unknown_entrants` is empty for the four entrant names.
7. **The command**: `Trpg::playtest_stages`, `playtest_plan`,
   `playtest_defaults` in `crates/xtask/src/game_tools.rs`, built from
   what `crates/xtask/src/playtest.rs::play` does today (the army of the
   battle's player slots, `BattleStage`, `BattleEntrant`). Then delete
   `crates/xtask/src/playtest.rs` and its tests and the `playtest` arm
   and usage line of `crates/xtask/src/main.rs` (its CLI tests move to
   `cargo shuyi playtest`, through `crates/xtask/tests/cli.rs`).
8. **The workflow**: `.github/workflows/playtest.yml` becomes a caller of
   engine#2021's workflow (its `levers` input set to the game's script,
   its concurrency as above), pinned to the engine commit in
   `shuyi-engine.rev` (the pin move also moves this line; add it to
   `engine-pin`'s list of files if that command keeps such a list).
9. **Compare**: run step 1's commands with `cargo shuyi playtest --stage
   <id> --entrant all --tries 20 --report after.json` (and the
   baseline). Every try's measures must equal `before.json`'s (same seeds:
   the personas' choices are unchanged). If one differs, a step above
   changed behaviour: find it, don't tune.
10. **Docs**: `docs/playtesting.md` (the command, its options, where the
    history is), ADR-0095, `docs/engine/boundary.md`.
11. **Measure** with `cargo xtask code-lines` (the friction row of this
    ticket; the estimate is below) and add the row.

**Expected size:** about 300 added production lines (the plan in
`game_tools.rs` about 80, the hooks and closures about 120, the rest
type changes) and about 1,000 removed. If the added lines pass 600,
split at step 7 into a second ticket (the command and the workflow)
and land steps 3–6 first.

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed the last
      of the seven engine tickets.
- [ ] `crates/xtask/src/playtest.rs` is gone; `cargo xtask playtest`
      prints "unknown command" (or the xtask usage).
- [ ] `cargo shuyi playtest --stage <Chapter 1's id> --entrant all
      --tries 20` gives, try by try, the same measures as `cargo xtask
      playtest <id> --bot all --runs 20` did on the old pin (step 9),
      and the same check verdicts.
- [ ] Its text shows, per entrant, the same numbers as before: won,
      deaths (Classic) or retreats (Casual) per try, most fallen unit,
      items median and max, turns median, seconds per try, and `was …`
      on a second run.
- [ ] `--threads 1` and the default give the same report.
- [ ] The Playtest workflow on this ticket's PR runs through the
      engine's reusable workflow and passes; its job summary has every
      battle's table.
- [ ] `crates/bots/personas.ron` and `targets.ron` hold the same numbers
      as before (a test compares them with a table).
- [ ] `cargo xtask code-lines` shows `trpg-bots` and `xtask` production
      lines down by at least 700 together.
- [ ] A row in `docs/engine/friction.md` for this ticket.
- [ ] `cargo xtask ticket-lint --check-engine` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the scoring (`Tally::between` counts on the existing fixtures;
  the old `score` tests now through `Weighted`), the danger (the
  existing `danger` tests, unchanged expectations), the order (the
  existing `order` tests, unchanged expectations), the targets.
- Integration: `crates/bots/tests/baseline.rs` unchanged;
  `crates/xtask/tests/cli.rs` runs `cargo shuyi playtest` on a test
  battle.
- Behaviour: step 9's comparison, kept as a test if it fits in a
  minute (20 tries of Casual on the test battle, measures equal to a
  committed fixture made on the old pin).

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
