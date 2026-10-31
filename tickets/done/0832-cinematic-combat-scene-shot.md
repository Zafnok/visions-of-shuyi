---
id: "0832"
title: A cinematic shot that plays the combat scene
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
nick_input: sign-off
completed: 2026-10-14
---

# 0832 — A cinematic shot that plays the combat scene

## Context

Nick, Chapter 1 playtest round 2 (0804, 2026-10-14), on the title
cinematic: "the combat scene (2 battlers) is never shown, only some
in-battle-map fighting which also looks a bit odd."

The cinematic format (`crates/content/src/cinematic.rs`, `Shot`; ticket
0817, ADR in the cinematic tickets 0817/0818/0827) has `MapPan` (with
`Walk` / `Strike` actions on the map, 0827), `Talk`, `Characters` and
`Logo`. There is no shot of the **combat scene**: the two battlers with
their full-body pictures over a backdrop (tickets 0413, 0035; built in
`crates/ui/src/screens/battle/stage.rs`, `stage::view(ctx, &Playback, ground,
light) -> Option<StageView>`, painted by `stage/glyph.rs`). In the game the
combat scene plays every attack when combat animations are on.

0833 uses this shot in the title cinematic.

## Nick input

**Sign-off:** watch the title cinematic once 0833 uses it: the fight looks
like the game's own combat scene.

## Scope

**In:**
- A new `Shot` kind (e.g. `Combat { battle, attacker, target, strikes }`),
  named and documented like the others, read from `.ron`, checked by the
  content checks (the units exist in the battle; at least one strike).
- What it shows: the game's own combat scene for one exchange between two
  units of the battle, built by the same code the battle uses
  (`stage::view` from a `Playback` of a combat worked out by `core`, so the
  numbers are real), with the backdrop of the defender's terrain. Its
  outcome comes from the rules, not from the cinematic file; the shot may
  name the RNG seed or the result to show (`hit`/`miss`, HP left) if the
  rules allow a fixed result (pick the simplest honest way and say so in
  the Completion notes).
- How long it lasts: the combat's own length at normal speed; the cinematic
  cuts to the next shot at its `at` as for any shot.
- The cinematic's tests and a snapshot of a frame of the shot.

**Out (do not do):**
- Changing the title cinematic's shots (0833).
- Changing the combat scene itself.

## Implementation steps

1. Read `cinematic.rs` (format), `crates/ui/src/cinematic/` (the player:
   `talk.rs`, `faces.rs`, `action.rs`) and `stage.rs` / `stage/view.rs`.
2. Add the shot kind and its checks in `trpg-content`.
3. Play it in the cinematic player: build the combat from the battle's
   start state with `core` (a `Command` → `Event`s, ADR-0004), make the
   `Playback`, show `stage::view` at the shot's local time.
4. Tests: content parsing and checks; the player shows the stage view at a
   time inside the shot (read the view: both battlers, the striker); a
   snapshot frame.

## Acceptance criteria

- [x] A `.ron` with the new shot loads; a shot naming a unit not in the
      battle fails the content check (test).
- [x] Test: half a second into the shot, the view is the combat stage with
      the two named units.
- [x] Snapshot of a frame (public placeholder art).
- [x] The title cinematic plays unchanged (its tests pass).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: parsing and checks.
- Snapshot / integration: the player's view in the shot; one snapshot.

## Completion notes

**What was done**

- New shot kind `Combat(battle, attacker, target, from, seed)`
  (`trpg_content::cinematic::CombatShot`), documented in
  `assets/cinematics/README.md`, recorded in ADR-0062.
- The fight comes from the rules (`trpg_content::cinematic::fight`): the
  battle is set up as it starts, the attacker is put on `from`, the phases
  end until it is the attacker's, and it attacks the target through
  `Command::Act` with the first weapon (or equipped spell) the rules
  accept. The numbers, hits, misses and HP are the game's.
- The player (`trpg_ui::cinematic::combat::Duel`) makes the battle's own
  `Playback` of those events with the battle's timings and shows
  `stage::view_for` at the shot's local time, painted by the combat
  scene's own skin. The backdrop is the defender's ground in the
  battle's light. `CinematicPlayer::stage(ctx, t)` gives the view;
  `Frame::Combat { seconds }` says how far into the fight.
- Checks: when loading, the battle and both units exist, they are two,
  and `from` is on the map, a tile the attacker can stand on and nobody
  else's start. Once all the content is loaded, every combat shot's
  fight is run (`check_fights`); one the rules refuse is an error of the
  file (`shot N: the rules refuse the attack: …`).
- Test file: `assets-test/cinematics/combat.ron` (the lead attacking the
  test brigand from beside it). Tests in `crates/content/src/cinematic/
  tests/combat.rs` and `crates/ui/src/cinematic/combat_tests.rs`;
  snapshot `combat_first_strike`.

**Deviations**

- No `strikes` field (the ticket's example). The fight has as many
  strikes as the rules give; "at least one strike" is checked by running
  the fight (it must resolve a combat with a strike).
- The fixed result is chosen the simplest honest way: an optional `seed`
  for the fight's luck (the battle's own if left out). The author tries
  seeds until the fight shows what the storyboard wants; nothing forces a
  result.
- `from` places the attacker before the battle starts (staging), rather
  than moving it with the rules, so a pair far apart at the battle's
  start can still be shown. The target always fights from its own tile.
- The combat shot has its own test cinematic instead of a fifth shot in
  `test.ron`: adding one would have moved every time in that file's many
  tests and snapshots. So the debug menu's "Play test cinematic" doesn't
  show it; the title will once 0833 uses it.
- `stage::view` gained a sibling `stage::view_for(ctx, lead, …)` so the
  shot shows the cinematic's lead (the latest save's) by picture as well
  as by name. The combat scene itself is unchanged.

**Things to know (not game rules, no Nick decision needed)**

- No sound during the shot (a cinematic plays its music alone, as the
  map shots' strikes do), and the scenes an attack can set off in a
  battle aren't shown.
- The fight plays at normal battle speed from the shot's start (about
  2½ to 4 seconds); a longer shot holds the last moment, a shorter one
  cuts it off.
- A rebalance can change how a cinematic fight goes; if it makes one
  impossible, the content check fails with the reason.

No gameplay rules were decided. No follow-up tickets.
