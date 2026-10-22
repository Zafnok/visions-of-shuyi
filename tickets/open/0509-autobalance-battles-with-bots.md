---
id: "0509"
title: "cargo xtask autobalance: the battle's knobs on the engine's search, until the persona bots land in their bands"
type: feature
milestone: M4 Enemy AI
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0506", "engine#2127"]
nick_input: none
completed:
---

# 0509 — Autobalance battles with the persona bots

## Context

Nick (0033, `docs/design/playtest-bots.md`, *Autobalancing*): "the point of
automated balancing is I don't want to manually do this stuff". A missed
target "should never happen" because battles are autobalanced. 0506 gives
three persona bots and a report that checks each bot's success rate
against a band for the battle's tier. This ticket adds the tool that
changes a battle until all three bands are met.

**Engine first (re-gated 2026-10-21, 0130).** The search is generic and
is the engine's: engine#2127 adds `Knob` (a bounded integer step that
changes a stage definition) and `autobalance`. The search is
deterministic. It scores each setting by the bands of engine#2126's
playtest runner, checks it with a quick try count and confirms it at
the full one, and prunes to the smallest change set it found. It stops
when every rate check is inside its band or a budget of evaluations
runs out, and it never touches the bots. This ticket writes only the
battle's knobs and the command:

| Engine | This ticket (the game) |
| ------ | ---------------------- |
| engine#2127: `Knob`, `BalanceSettings`, `autobalance`, `Balanced` (the changes in the knobs' words, the reports before and after, why it stopped) | the battle `kind`; the "enemy numbers" knobs over a battle definition; `cargo xtask autobalance` as a thin wrapper (load, call `autobalance` with 0506's entrants and checks, print, `--apply` writes the file) |

What it may change depends on the battle kind (Nick, Q11):

| Kind | Knobs |
| ---- | ----- |
| Random skirmish | Everything: enemy levels, count, classes, gear, positions, reinforcements, terrain (terrain and layout knobs come with the generator, 0510) |
| Fixed skirmish, side quest, story battle | Enemy numbers only: level, count, gear, which classes appear. Map, named units (bosses, characters) and objectives stay as authored. |

**The bots are never tuned to pass** (0506 context): only the battle
changes. The engine's search has no way to reach the bots' settings.

**Either order with 1180:** if 1180 is done, the autobalance
search judges with the engine's `Targets` (engine#2129) and the game's
playtest plan; if not, with `trpg_bots::Targets`, and 1180 moves it.

**The command is the engine's when it can be** (2026-10-22, from the
retro of the last 50 PRs: 0506 wrote its own playtest command, threads
and history, and 1180 now moves them out). engine#2127 left out a
`shuyi autobalance` operation and threads; engine#2541 adds them (the
search on every core, a time limit, `--apply` through the game's
`write`). **Either order with engine#2541:** if it is done and pinned,
this ticket implements its plan hooks in `crates/xtask/src/game_tools.rs`
instead of a `cargo xtask autobalance` wrapper, and moves 1183 to
`done/` in its PR ("done by 0509"); if not, the wrapper stays as thin
as below, writes no thread pool, options parser or report format of its
own beyond what the table names, and marks its command module
`// engine-friction: engine#2541`, and 1183 moves it.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Nick plays one autobalanced battle on the Pages build and says
> whether it felt about as tight as the design asks ("a bit tight" in Casual).

## Scope

**In:**
- Moving the engine pin past engine#2127, if 0506 didn't already pin
  past it.
- `cargo xtask autobalance <battle-id> [--apply] [--runs 100] [--quick 30] [--budget 200] [--seed 1]`.
- The "enemy numbers" knob set (fixed skirmish, side quest, story
  battle).
- A battle `kind` field if 0801's battle files don't have one yet.

**Out (do not do):**
- The search itself, the distance to the bands, the pruning, or the
  report of changes. These are engine#2127's. If something generic is
  missing, use the `engine-friction` and `engine-change` skills.
- Terrain, positions and reinforcement knobs (0510 adds them for random
  skirmishes).
- Changing persona settings or the bands (`docs/design/playtest-bots.md`).
- Running autobalance in CI. It's a dev tool that Claude runs, and
  Claude commits its result; CI only runs 0506's check.

## Implementation steps

Expected size (ADR-0092): about 350 production lines:

- the kind, about 30;
- the knobs, about 200;
- the xtask wrapper and `--apply`, about 120.

Measure with `cargo xtask code-lines`, and at 3× run the
`engine-friction` skill.

1. **The pin.** If `shuyi-engine.rev` is before engine#2127's merge,
   run `cargo xtask engine-pin <engine main past it>` (the
   `engine-change` skill, §7).
2. **Kind.** If the battle file format (0801) has no kind, add
   `kind: Story | FixedSkirmish | SideQuest | RandomSkirmish` (default
   `Story`), validated by the content check. Document it where the
   battle format is documented.
3. **Knobs** (`crates/bots/src/balance.rs`, pure). Each is an engine
   `Knob` over a loaded battle definition, not a running `BattleState`.
   The order matters: the engine tries knobs in the order given.
   - **The level offset of every generic (unnamed) enemy:** first, with
     a stride of 4 and a range of -10 to +10, so the search moves it
     coarsely before the rest.
   - **A per-enemy level offset**, -5 to +5.
   - **Per generic enemy, its count:** -1 removes it, +1 adds a copy on
     a free tile within 2 tiles of the original, in the same group with
     the same AI. `apply` gives `None` when there is no free tile.
   - **Per generic enemy, its class:** a step picks among the classes
     already in that battle, in the battle's order.
   - **Per generic enemy, its weapon:** -1 or +1 moves it a tier down
     or up in the same weapon type.

   Each knob's `describe` says the change in plain words ("Brigand ×2 →
   lv 4", "removed Archer at 12,5"). Named units, the map and the
   objectives are never touched, and a unit test enforces it. The
   battle's kind picks the knob set. Only "enemy numbers" exists until
   0510.
4. **The command** (`crates/xtask/src/autobalance.rs`):
   - load the battle;
   - build 0506's three persona entrants and its checks for the
     battle's tier (the `baseline` entrant isn't one);
   - call the engine's `autobalance` with
     `BalanceSettings { tries: --runs from --seed, quick: --quick, evaluations: --budget }`;
   - print its `Balanced`: why it stopped, the changes, and the reports
     before and after. Show the reports with 0506's game lines.

   `--apply` writes the changed battle file, keeping its formatting and
   comments as far as the RON writer allows. If it can't keep them,
   document the loss.
5. **Docs.** Give `docs/playtesting.md` an *Autobalancing* section:
   what the knobs are, how to read the output, and what `--apply`
   does. Say that the search is the engine's (engine#2127).

## Acceptance criteria

- [ ] On a test battle made deliberately too hard (enemies +5 levels),
      `cargo xtask autobalance <it>` finds changes after which all three
      bots are inside their bands, and `--apply` writes them.
- [ ] Unit test `autobalance_never_touches_named_units_map_or_objectives`:
      every knob at every step of its range leaves them as authored.
- [ ] Same seed → same changes.
- [ ] No search, distance or prune code in the game (the PR says which
      engine pieces it uses).
- [ ] Chapter 1 (`ch01`) report before/after in Completion notes; if
      Chapter 1 already meets its bands, say so and don't change it.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each knob (its range, `apply`, `describe`, and `None` where it
  can't apply); the named-units guard.
- Property: any setting of the knobs within their ranges yields a
  battle that passes content validation.
- Snapshot / integration: the deliberately-too-hard battle is fixed.

## Completion notes
