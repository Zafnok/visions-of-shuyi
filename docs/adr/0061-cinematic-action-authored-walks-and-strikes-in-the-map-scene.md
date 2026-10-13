# ADR-0061: Cinematic action: authored walks and strikes, put in the map scene from the action clock alone

- **Status:** Accepted
- **Date:** 2026-10-13
- **Related tickets:** 0827, 0817, 0036, 0820
- **Extends:** ADR-0060 (cinematics: timed shots drawn from the time alone)
- **Follows:** ADR-0004 (crate boundaries), ADR-0005 (content is data,
  validated), ADR-0038 (the map is painted by a skin from a map scene)

## Context

- The storyboard Nick picked for the title's intro (0036,
  `docs/design/title-screen.md`, *Intro cinematic*) shows units walking
  across the battlefield and fighting: a march, close-ups where blows are
  traded and an enemy falls, the whole field with the party advancing.
  ADR-0060's `MapPan` only shows a battle's units standing where they
  start.
- ADR-0060's rule: a frame is a pure function of the time. The title
  follows the music's clock, which lags, jumps and wraps.
- One fight is shown across several shots (a cut to a close-up and back).
- Stats, weapons and the combat rules will keep being rebalanced. An
  intro that played a real battle would change, or break, each time.
- The battle map is whatever the map skin paints from a `MapScene`
  (ADR-0038). There are two skins and more may come.
- `Content` is compared with `Eq`: no floats in it.

## Decision

### 1. The action is authored, and never runs the rules

A `MapPan` shot may carry an **action**: a list of timed entries, each a
`Walk` or a `Strike` (format: `assets/cinematics/README.md`).

- `Walk(unit, path, secs)`: the unit goes along the path at an even
  speed and stays at its end.
- `Strike(unit, target, hp_left)`: the unit leans at the target; half way
  the target flashes and has `hp_left` percent of its full HP; at 0 it
  fades out and is gone.

What a strike does is what the file says. **`core` is not involved**: no
`BattleState`, no `Command`, no forecast, no RNG, no combat scene, no
EXP, no sound. The walk's pace is the file's `secs`, not the unit's Mov.
A rebalance can never change the cinematic.

The loader only checks that a list **can be drawn** (`trpg_content::
cinematic::action`), reporting every problem at once:

- the units exist on the battle's map at its start, named as the battle
  file names them (a slot's character, an enemy's `id`, an enemy
  character);
- a walk starts where its unit is then, after its walk before has ended,
  stays on the map in runs along a row or a column, and crosses only
  tiles its class's movement type can enter (read from the terrain
  table: the one place rule *data* is looked at, so nobody walks on
  water);
- a unit doesn't strike mid-walk or itself; `hp_left` is 0 to 100 and
  never rises; a unit left at 0 is not named again; entries are in order
  of time.

Loaded, names are unit ids, times whole milliseconds, and a path is every
tile of the walk (`ShotAction`, `ActionDef`, `Act`). An enemy's `id` isn't
in `BattleDef` (core has no use for it): `battle::unit_names` reads
it from the battle's file, as `battle::load_lights` reads their light.

### 2. The action clock

An action has its own clock. A shot starts it at its `action_from` and it
then runs with the cinematic's time:

```
clock = action_from + (t − the shot's start)
```

Several shots naming one list (the file's `actions` table, so the list is
written once) with different `action_from` show the same fight further
on, from another camera. Nothing links the shots but that number: a shot
can be moved, cut or dropped without touching the others.

### 3. The state is a pure function of the clock

`trpg_ui::cinematic::action_state(start, actions, clock_ms) -> Vec<UnitView>`
takes the battle's units as they start and returns them as they are at
that moment: each along its walk (reusing the battle screen's
`walk::gait`, so a walker turns and steps the same way), leaning in a
lunge, with the HP its last landed strike left it, flashing, fading, or
not there any more. It reads every entry that has started; it keeps
nothing. So ADR-0060 §2 holds for action shots as it does for pans: the
same `t` is the same frame, and a jump or a loop needs no care.

The strike's timings (lunge 0.4 s, flash 0.2 s, fall 0.5 s, lean a quarter
of a tile) are constants beside it, marked tunable. They are a look.

### 4. It only changes what is in the scene

`action_state` returns the map scene's own unit entries.
`CinematicPlayer::scene(t)` is the battle's `MapScene` with those units,
and `draw` hands it to the map skin (at the art's own size, ADR-0060 §3).
The cinematic draws no unit, bar or flash itself.

- A walk uses what the scene already had for a battle's walk: the tile,
  `offset`, `facing`, `frame`. A lunge is a small `offset`. A fall is
  `fade`, as in a battle.
- One field is new: `UnitView::flash` (0 to 1). The glyph skin lights the
  unit's tile toward the text colour; the sprite skins draw the unit's
  shape in that colour over its picture. A new skin must paint it too:
  the sprite skin's "every scene feature is painted" test lists it.
- Tests of what happens read `scene(t)`; only the skins' tests and the
  snapshots read cells and sprites.

A shot with no action is painted once when the player is made, as
before. A shot with an action is painted **each frame** (the scene
changes), with the skin `ctx` has then.

## Consequences

- 0820 writes the title's march and fights as data on the Chapter 1 map.
  Moving a unit's starting tile in `ch01.ron`, or changing the map under a
  path, fails the content check with the entry's number, not at run time.
- A fight in the intro can show something the rules would never do (a
  recruit felling the boss in two blows). That is the author's to judge;
  the storyboard is Nick's.
- An action shot repaints the whole map every frame. A Chapter 1 map is a
  few hundred tiles; the battle screen paints as much each frame.
- An action shot follows a change of map skin only as far as its units
  go: the pan's points were worked out for the skin the player was made
  with. Whoever changes the skin makes the player again (as ADR-0060
  already asks).
- The glyph skin's units stand in cells: a walk steps from tile to tile
  and a lunge's lean of a few pixels doesn't show (the flash and the HP
  bar do). The sprite skin shows both.
- Units not named in the list stand still, without the battle's
  stepping-on-the-spot. If the intro wants that, it is a look to add to
  `action_state`.
- Reinforcements, terrain changes, spells and items can't be shown. Each
  would be a new kind of entry with its own checks.
- The enemies' `id`s are read from the battle files a second time when
  the cinematics load.

## Alternatives considered

- **Replaying a recorded battle** (a play record through `core`). The
  real rules, for free. But every rebalance changes or breaks it, the
  state at time `t` needs a replay from the start (or snapshots), and it
  can't show a fight the storyboard wants if the dice disagree.
- **A scripted battle through `core` with fixed outcomes.** Needs a
  second way to resolve combat in `core` that only the intro uses.
- **A player that advances the action each frame** (`update(dt)`).
  Simpler to write, but it can't jump or loop with the music, and two
  shots of one fight would have to share a running state.
- **One long shot with camera moves inside it.** Nick chose hard cuts
  and one size per shot (0036). `action_from` gives the cuts without it.
- **Positions in pixels** (as the ticket sketched for `action_state`).
  They would mean another place under each skin; the scene's tile and
  offset in tiles is what both skins already paint.
- **The enemies' names in `BattleDef`.** A field in `core` that no rule
  reads, in every place a battle is built.
- **Drawing the flash and the lunge in the cinematic player.** It would
  have to know each skin's tile size and pictures: the look would live
  outside the skins (ADR-0038).
