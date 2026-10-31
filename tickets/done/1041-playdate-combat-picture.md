---
id: "1041"
title: "Playdate: the simple one-bit combat picture"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1039", "1040", "1061", "0062"]
nick_input: none
completed: 2026-10-23
---

# 1041 — The Playdate combat picture

## Context

Nick kept "a simple one-bit combat picture" on the Playdate
(`docs/design/playdate.md`, Q5). 0062 decides what "simple" looks like
(read its answer in the design doc's *One-bit pictures* section first;
if it changed this ticket's scope, 0062's session updated it).

The combat scene is already a view painted by a skin
(`crates/ui/src/screens/battle/stage.rs`, `stage/view.rs`, `stage/glyph*`;
ADR-0056, `docs/design/look-and-feel.md`, *Combat screen*): each fighter
is one still, full-body picture moved by the game, over a battle
background. On the Playdate it is a new paint of the same stage view,
with one-bit pictures from 1040. 1039 painted the fight box and the rest
of the fight's windows.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Nick watches a fight in the Simulator or on his Playdate
> (e.g. the lead against the first brigand) and comments.

## Scope

**In:**
- A Playdate paint of the stage view, chosen by `SkinSet` (1037), at
  400 × 240: the two fighters and background as 0062 decided
  (`playdate.md`, *One-bit pictures*: fighters in style A on plain white
  ground with a 2-pixel white rim, the dithered backdrop only above a
  horizon line; the portrait style setting doesn't change them), the HP
  and strike numbers the PC combat screen shows, the hit, miss and
  crit moments the PC shows, at the PC's timing.
- The pictures it needs converted by 1040's tool (battlers and
  battlebacks of Chapter 1), with the method 0062 chose.
- The Playdate bundle (1036) lists them.
- Look tests with the skin; behaviour tests read the stage view.
- **Measure on the device:** frame time during a fight (the motion
  redraws every frame); the PR lists it.

**Out (do not do):**
- New combat animations or effects the PC doesn't have.
- Changing the PC combat scene.

**Carve:** no carve ticket touches the stage; follow where the frame
went (1108).

## Implementation steps

1. Read 0062's answer, `stage.rs`, `stage/view.rs`, `stage/glyph.rs`.
2. The Playdate paint and its tests.
3. Convert and bundle Chapter 1's battlers and battlebacks.
4. Simulator, then ask Nick to sideload for the frame time.

## Measured on Nick's Playdate

Frame time during a fight (min/avg/max) and the stack high-water mark,
from the Simulator console while the device runs a fight.

Nick's Playdate (Rev B), release build with the bought art, 2026-10-23,
Chapter 1's first battle with a few fights (the front end's minute
report; a frame is drawn only when it changes, so the render's numbers
are mostly the fights' frames):

| A minute on the battle | Frames a second | `Game::frame` min / avg / max | Render and copy min / avg / max (frames drawn) |
| ---------------------- | --------------- | ----------------------------- | ---------------------------------------------- |
| 1 | 24.3 | 3.0 / 19.8 / 477.5 ms | 6.9 / 136.4 / 409.3 ms (21) |
| 2 | 14.5 | 27.9 / 54.3 / 253.1 ms | 6.1 / 169.3 / 637.9 ms (57) |
| 3 | 17.7 | 28.3 / 48.1 / 250.1 ms | 6.1 / 136.9 / 600.2 ms (29) |

Game stack used: 23,640 of 65,536 bytes. Nick: "moving the cursor after
selecting a unit, the unit moving after confirming, and in-battle fight
scene is the choppiest. It drops a lot of frames and causes audio
hitching." **The fight stutters**: its frames take 136–169 ms to render
on average against 33. Follow-up 1078 (with engine#3056) says what to
cut; the cursor and the walk are 1057's.

## Acceptance criteria

- [x] A fight in Chapter 1 shows the one-bit combat picture in the
      Simulator, at the PC's timing.
- [x] Device frame time is in the PR; the fight doesn't stutter (or a
      follow-up ticket says what to cut). *It stutters: 1078 and
      engine#3056 say what to cut.*
- [x] No PC snapshot changes.
- [x] Starting rules and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with the skin): one frame of a strike.
- Integration (harness, Playdate skin set): a fight plays to its end;
  reads the stage view.

## Completion notes

**What was done.**

- **The Playdate's paint of the stage view**
  (`crates/ui/src/screens/battle/playdate/stage.rs`), chosen by the
  battle's Playdate skin: while a fight plays it covers the whole
  screen, and the map is no longer painted under it (the PC's paint
  still covers its map as before). The same `StageView` as the PC's, so
  the fight's beats, HP drain and timing are the PC's; the motions reuse
  the PC's `offset_px` with the same pixel sizes (a fighter is as many
  pixels big on the Playdate as on the PC's console).
  - The backdrop sprite is the whole picture at its converted size,
    clipped to the 110 pixels above the horizon, so each dot of its
    dither lands on one screen pixel; a black line a pixel high is the
    horizon; plain white ground below.
  - The fighters at twice their sidecar's scale, rounded as the
    converter rounds (a 1.25× fighter's picture is enlarged smoothly by
    the converter, per 0062), middles 176 pixels apart, feet 4 pixels
    above the plates, facing each other, the striker drawn last.
  - Plates: two boxes three rows tall across the bottom, the striker's
    double, name and HP in the top of the frame, numbers inside. The
    result over the target and an art's name at the top, each in a
    one-row white box with a black edge a pixel wide.
  - A hit fighter is a black shape; a critical flips the picture (two
    new names in the Playdate palette, `stage_flash` and `stage_veil`).
- **The pictures** (`crates/content/src/playdate_images.rs`, used by
  `build.rs` and `xtask playdate-images` through one `make`):
  - **The fighters' white rim (2 pixels) is baked into their one-bit
    pictures** when they are converted (`with_rim`: the mask grown by a
    round brush, the new pixels white), so the device draws one sprite
    per fighter, not a rim of copies. Every bought battler has at least
    one see-through pixel above its head, so the rim always fits there;
    two (the exile and the heretic) touch their picture's right edge,
    where their rim is cut.
  - **Battle backdrops are lightened as in the mockup Nick chose**
    (`BATTLE_LIFT`, `0.6 b + 0.47`, no stretch). 1040's starting
    conversion (curve 0.8 and the 3% stretch, still the title's) made
    the tree line solid black behind the fighters' heads.
  - Chapter 1's battlers and battlebacks were already in the Playdate's
    bundle (1036, 1040, 1061): nothing to add to `playdate-bundle.txt`.
- **Tests:** look tests with the skin (`battle/playdate/stage/tests.rs`):
  the layout, the backdrop's and the fighters' sprites at the sizes the
  real converter makes, the motions in pixels, **one frame of a strike**
  (sprites, the result box, the plates, and in pixels through the
  one-bit renderer: the backdrop, the horizon, the white ground, the
  rim two pixels above a head, the struck fighter black), the critical's
  flip, a banner and "no counter", a long name and big numbers cut to
  the plate. The rim and the lift have their own tests
  (`playdate_images`). Integration (`tests/it/playdate.rs`): the Quick
  Battle on the Playdate's skins, a fight read from the stage view frame
  by frame until it ends (the player's fighter on the left, the
  attacker's result first, over the brigand, its HP drained), then the
  EXP bar.

**Simulator** (`cargo xtask playdate --release --private-assets`,
driven by window messages to the Simulator's own window): New Game,
Chapter 1's first battle, Tamsin moved beside a brigand, Attack: the
forecast, then the fight on the one-bit picture over the bought
backdrop, `HIT -11` over the Brigand and his HP draining in his plate,
his counter `HIT -7` over Tamsin, who flashed black, then her EXP bar,
about 1.8 seconds in all (the Simulator runs at the PC's speed). Two
screenshots of the running game are in the PR
(`docs/design/playdate/1041-simulator-*.png`), and the same fight with
the public placeholders from `cargo shuyi frame-png --playdate`
(`1041-strike-placeholders.png`).

**On the device** (above): the fight stutters. The render, not the
game: the one-bit renderer draws a sprite a pixel at a time with two
64-bit multiply-divides per pixel, and the stage has about 95,000
sprite pixels a frame. Engine ticket **engine#3056**
([Zafnok/shuyi-engine#94](https://github.com/Zafnok/shuyi-engine/pull/94))
gives sprites a fast path; game ticket **1078** moves the pin, measures
again with the bench build, and lists what the game can cut if that is
not enough. Nick's cursor and walk choppiness, and `Game::frame`'s
27.9–54.3 ms, went into **1057**'s Context.

**Engine friction:** estimate 200 production lines (one paint of an
existing view, judged by the PC's paint of the same view,
`stage/glyph.rs`, about 250 lines with the backdrop's mirrored edges the
Playdate doesn't need), measured 317 (1.6×), under 2× by lines; the lift and the rim are steps before and after the
engine's converter, which could own both one day (`Method` with a floor,
a rim option) if another kind of picture needs them. The device run
then missed the performance budget (a signal): outcome 2, the
renderer's sprite path, shipped with follow-ups engine#3056 and 1078.
Row in `docs/engine/friction.md`.

**Follow-up tickets:** 1078 (the fight at 30 frames a second, blocked
by engine#3056); engine#3056 (one-bit sprites drawn fast). 1057 got the
day's battle numbers.

***the agent's starting rules*** (in `docs/design/playdate.md`, *The combat
picture*; for 1064's checklist, Nick can veto):
- The backdrop lightened as in 0062's mockup (`0.6 b + 0.47`), showing
  the mockup's rows of the picture.
- Plates three rows tall, name and HP in the top of the frame, the
  numbers inside (the mockup had both lines inside a thinner box, which
  the screen's 16-pixel text rows can't make without a third more
  height); the name is the bar while that fighter is about to strike.
- The result in a one-row box with a thin black edge on the row over
  the target's head (the top row, or the second under an art's name).
- A hit fighter shows as a black shape (the PC's white one would vanish
  on the white ground); a critical flips the picture above the plates.
- Fighters 176 pixels apart, feet 4 pixels above the plates; the PC's
  motions pixel for pixel. A fighter taller than the 192-pixel scene
  (the guard, 192 pixels with its 40 see-through rows on top) is cut at
  the screen's top, where its picture is empty.

**Left to judge by playing (1064):** whether the fight reads on the
device's small screen (the dither of the fighters, the 2-pixel rim,
the thin result box); whether the black hit flash and the inverted
critical feel like the PC's flashes; whether the name and HP in the
plate's frame read as well as the mockup's two lines.
