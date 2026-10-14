# ADR-0062: Cinematic combat shot: a fight the rules work out, played back on the battle's own stage

- **Status:** Accepted
- **Date:** 2026-10-14
- **Related tickets:** 0832, 0833, 0413, 0827
- **Extends:** ADR-0060 (cinematics: timed shots drawn from the time alone)
- **Follows:** ADR-0004 (crate boundaries), ADR-0005 (content is data,
  validated), ADR-0054 (a view a skin paints), ADR-0056 (the combat stage)

## Context

- Nick, Chapter 1 playtest round 2 (0804): the title cinematic never
  shows the **combat scene** (two battlers over a backdrop, 0413), only
  fights on the map, "which also looks a bit odd". Ticket 0832 asks for a
  shot that is the game's own combat scene, built by the code the battle
  uses, **with real numbers**: the outcome comes from the rules, not the
  file.
- ADR-0061 kept the map shots' action away from `core` on purpose: the
  file says what each strike does, so a rebalance never changes the
  intro. This shot is the opposite on purpose.
- The battle's combat scene is `stage::view(ctx, &Playback, ground,
  light) -> StageView`, painted by `stage::glyph::paint`. A `Playback`
  is built from one command's `Event`s and has a clock (`tick`).
- ADR-0060 §2: a frame is a pure function of the time.
- The rules' luck is the battle's `SimRng`, seeded by `BattleSetup::seed`.
- Content is loaded before a `Content` exists, but setting a battle up
  for a campaign (`battle_campaign`, `battle_setup`) needs the whole
  `Content`.

## Decision

### 1. The shot names who, from where and with which luck

`Shot::Combat(CombatShot { battle, attacker, target, from, seed })`
(format: `assets/cinematics/README.md`). Units are named as an action
names them (`action::cast`). `from` is the tile the attacker strikes from
(where it starts if left out); `seed` the luck (the battle's own if left
out). There is no field for the result: to show a given result, the
author picks a seed that rolls it. That is the "simplest honest way" the
ticket asks for: the dice are real, only which roll is shown is chosen.

### 2. `trpg_content::cinematic::fight` runs the attack through `core`

1. The battle is set up as it starts (`battle_campaign` in Classic mode
   with the given lead, `battle_setup`), with the shot's seed.
2. The attacker is placed on `from` **in the setup**, before
   `BattleState::new`. It is staging, not a move: a unit can be put
   beside a target it couldn't reach on turn 1. The loader checks the
   tile (on the map, enterable by the attacker's movement type, nobody
   else's start), because the rules never check a unit's own tile.
3. `Command::EndPhase` until it is the attacker's phase (at most a round).
4. `Command::Act` at its tile with the first attack the rules accept
   (`BattleState::check`): its equipped weapon or attack spell, then each
   other weapon, first slot first. No art, active or item.
5. The events must hold a `CombatResolved` with a strike.

It returns the units before, the events and the state after (`Fight`).
State changes only through commands (ADR-0004); no I/O.

### 3. The content check runs every fight

`cinematic::check_fights(Content)` runs after `assemble` in `load()`,
with the default lead. A fight the rules refuse is an error of the
cinematic file: `shot N: the rules refuse the attack: <CommandError>`.
So a rebalance or a map change that breaks a combat shot fails the
all-assets test, not the title.

### 4. The player plays it back as the battle does

`trpg_ui::cinematic::combat::Duel` (built once in `CinematicPlayer::new`)
holds the `Playback` of the fight's events at its start, made with the
battle's `TIMINGS`, without the `SceneTriggered` events (a playback waits
at a scene; a cinematic plays only its own shots). It also keeps the
defender's ground and the map's light, as the battle reads them.

- `Frame::Combat { seconds }`: seconds into the fight, at normal speed
  from the shot's start, held at the fight's end.
- `CinematicPlayer::stage(ctx, t) -> Option<StageView>`: a **copy** of
  the playback ticked to `seconds`, given to `stage::view_for`. Nothing
  is carried between frames (ADR-0060 §2).
- `draw` paints it with `stage::glyph::paint`, the battle's own skin.
- `stage::view_for(ctx, lead, …)` is `stage::view` with the lead's
  picture from a given `LeadProfile` instead of `ctx.lead`: the shot shows
  the cinematic's lead (the latest save's), by name (from the campaign
  built with it) and by picture, like its talk and characters shots.

The test file is `assets-test/cinematics/combat.ron` (a cinematic of the
one shot), not a shot added to `test.ron`: every time in the test
cinematic's many tests and snapshots would move.

## Consequences

- The title's fights on the stage are the game's: if the brigand's HP or
  the lead's sword change, the intro's numbers change with them, and a
  hit may become a miss. Re-picking a seed is a data change.
- A fight the rules stop allowing is caught by the content check. If one
  slips through anyway (content changed after loading, in a test), the
  shot is blank, never a crash.
- The shot only shows an attack at a battle's start. A fight later in a
  battle (after moves, levels, items) can't be shown.
- A shot shorter than its fight cuts it off; nothing checks that, since
  the fight's length is the `ui`'s timings. The README gives the rough
  length; the player exposes it (`Duel::seconds`).
- No sound and no triggered scenes, as for every cinematic shot.
- The content load runs `core` once per combat shot (a battle setup and
  one attack): negligible.

## Alternatives considered

- **The file states the result** (hit/miss and HP per strike), as the
  map action does. Not the rules' numbers, which is what the ticket and
  Nick's note ask for.
- **A fixed result forced through the rules** (a scripted RNG in the
  battle state). `core` would need a second source of luck only the intro
  uses, against ADR-0019's single `SimRng`.
- **Moving the attacker with the `Act` command** instead of placing it.
  Honest about Mov, but most pairings the storyboard wants aren't within
  one turn's move at the battle's start.
- **Checking the fights in the `ui` crate's tests only.** The error would
  not point at the file, and loading the content (the game's start, the
  all-assets test, `cargo xtask check-script`) would not report it.
- **Building the stage view without a `Playback`.** A second copy of the
  combat scene's timing; the shot would drift from the battle's look.
