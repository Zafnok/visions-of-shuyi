# ADR-0131: Each target has a graphics tier, and every moving look names its stills

- **Status:** Accepted
- **Date:** 2026-11-23
- **Related tickets:** 1314 (engine 3347); 1315, 1316 and 1317 build on it

## Context

Nick (2026-11-22), with the tileset overhaul (0096): the moving looks
coming next (flowing water, swaying trees, drifting petals: 1315, 1316)
are for the PC and the web, and the Playdate (and a GBA one day) can't
afford them. He wants the switch built first, so every moving look is
written with its lesser-target version in place and a port later is not
a big job; the lesser versions may be scaffolded with a `TODO` for the
missing picture.

What existed:

- **Where the game runs** is the engine's `PlatformFacts` (engine
  ADR-0060), read from the target's profile (`targets/*.ron`) and kept on
  `Ctx::platform`. It had nothing about graphics.
- **Which skins paint** is `SkinSet::{Glyph, Playdate}` (ADR-0083), set
  by a front end; the player picks a `MapLook` (pictures or glyphs).
  Neither says how much motion a target can afford: the Playdate's skins
  are one-bit, and a moving look on them is a separate question.
- **`ctx.idle_motion`** (On or Off) stops looping motion in tests, so a
  frame is the same a moment later (0093). It answers "may the screen
  move now", not "can this target afford it".
- The engine's parked tickets that will draw moving looks (2359 animated
  tiles, 2360 a tile-layer painter, 2103 particles, 2370 weather) are all
  blocked, so none of them could carry the switch.

## Decision

### 1. The engine decides the tier; each target's profile says it

The tier is a fact of the target, so it is the engine's: engine ticket
3347 adds `GraphicsTier::{Minimal, Reduced, Full}` (ordered,
`Full` the default) to each target profile (`graphics:`) and to
`PlatformFacts::graphics`. Its names are generic: what each tier allows
is a guide the game keeps to, not a limit the engine enforces.

| Target | Tier | What the game shows there |
| --- | --- | --- |
| desktop, web | `Full` | every moving look |
| playdate | `Reduced` | stills, and at most a few cheap moving tiles a later ticket picks; no particles, no effects over the screen |
| (a GBA) | `Minimal` | stills only; `TODO(gba)`: no target exists (engine 3022) |

The desktop's and the web's tiers are the engine's profiles' (`Full`, by
default). The Playdate's is in both of the game's copies of its profile,
`targets/playdate.ron` and `crates/ui/src/playdate_target.ron`
(`graphics: Reduced`), which a test keeps equal to the engine's. A GBA
gets no profile and no row in `shuyi.ron` until it is a target; the
`Minimal` stills are written now all the same (§3).

### 2. A skin asks `ctx.platform.graphics`

A skin that draws a moving look reads the tier from the context it
paints with, `ctx.platform.graphics`, and never from `SkinSet`, a `cfg!`
or the target's id. `core`, the bots and play records never read it
(ADR-0038): a battle plays the same at every tier.

The tier and `idle_motion` are separate and both apply: below a look's
lowest tier the skin draws its still; at or above it, with `idle_motion`
off, the skin draws the look at time 0 as today.

### 3. Every moving look names its stills, as data

A look that moves by itself carries a `Motion` (`trpg_content::motion`)
beside its frames, in its own data file (a tileset row in 1315, a map
effect in 1316):

```ron
motion: (
    lowest: Full,             // the default: moves at Full only
    stills: {
        Reduced: (show: FirstFrame),
        Minimal: (show: FirstFrame, todo: "a one-bit tile"),
    },
),
```

- `lowest` is the lowest tier it moves at (`Full` unless said; a cheap
  tile the Playdate may move says `Reduced`).
- `stills` holds one `Still` for **every** tier below `lowest`: `show`
  is `FirstFrame` (frame 0, held) or `Nothing` (left out: an effect).
  `todo` says what is still to make for that tier (a lighter picture);
  until then the `show` stands in.
- `Motion::drawn(tier)` is what the skin draws: `Moving`, or
  `Still(show)`. A still left out is drawn as the first frame, so a
  mistake is never a crash.

### 4. A test runs the check over the game's content

`motion::check` reads every moving look (`motion::moving_looks(&content)`)
and reports:

- a **problem** for a tier below `lowest` with no still, and for a still
  on a tier where the look moves;
- a **to-do** line for each `todo`, listed and never failed.

The content crate's integration test `every_moving_look_names_its_stills`
(`crates/content/tests/it/all_assets_load.rs`) fails on a problem and
prints each to-do (`--nocapture`). `moving_looks` finds none today; 1315
adds the tilesets' moving rows to it and 1316 the map effects
(`TODO(1315)`, `TODO(1316)` in the code).

### 5. A lesser tier is previewed on the PC

- `cargo shuyi frame-png --graphics reduced` (the game's option) draws a
  picture at that tier with the PC's skins and screen.
- `SHUYI_GRAPHICS=reduced` before a debug build of the game
  (`cargo run -p trpg-app`) shows that tier in the window: the engine's
  `shuyi-app` reads it in a build with the debug tools, never in one
  that ships.

The tier changes what moves, not the colours: the Playdate's one-bit
look stays `--playdate`'s.

## Consequences

- 1315 and 1316 write each moving look's stills from the start, and the
  check holds them to it; a Playdate or GBA port later finds them there.
- The Playdate's look does not change: nothing moves yet, and its skins
  are its own. Every picture is the same at each tier today (a test
  checks it).
- One more fact to keep in step: the game's copies of the Playdate's
  profile must say `graphics: Reduced` as the engine's does (the existing
  test fails otherwise), until 1227 drops the copies.
- The tiers are coarse: three steps, no particle count. A finer budget
  (how many particles at `Reduced`) is a field of the look's own data
  when a ticket needs it, not a fourth tier.

## Alternatives considered

- **The game's own tier, chosen by target id** (a `match` on
  `ctx.platform.target`) — a tier is a fact of the target, like its
  colours, and every game on the engine with a weak target needs it; a
  match on names is the `cfg!` that ADR-0060 replaced.
- **Fold it into `SkinSet`** — the skin set is the Playdate's one-bit
  look, not its speed; a fast one-bit target or a slow colour one would
  need both answers.
- **Capabilities per feature** (`animated_tiles: bool`, `particles: u16`,
  `screen_effects: bool` in each profile) — more to keep in step on every
  target for looks that don't exist yet; the engine's parked
  capabilities ticket (3017) says what hardware can do, and an ordered
  tier says how much a game spends, which a look compares with one `>=`.
- **A still missing at run time is an error** — a crash on a device for
  a data slip the check catches before shipping; drawing frame 0 is
  always safe.
- **A debug-menu row to switch the tier** — needs a menu row, its text
  and its keys; the variable and the `frame-png` option cover the PC and
  the pictures with no screen work.
