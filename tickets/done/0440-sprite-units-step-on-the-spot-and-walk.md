---
id: "0440"
title: "Sprite units step on the spot while waiting and walk along their path"
type: feature
milestone: M3 Battle UI
model: opus-5.5
effort: medium
status: done
blocked_by: ["0436"]
nick_input: sign-off
completed: 2026-10-03
---

# 0440 — Sprite units step on the spot and walk along their path

## Context

Nick decided on 2026-10-02 (ticket 0039, `docs/design/look-and-feel.md`,
*Battle map: bought tiles and unit sprites*) that sprite units move like
Fire Emblem's on the GBA. He judged three animated mockups (J1–J3, then
J4 with the feet above the HP bar; in the bought-art folder's
`spike-renders/`) and picked "1C":

- A unit that **can still act steps on the spot**, facing the camera.
- A unit that **has acted stands still** (and is grey, 0436).
- A **moving unit walks along its path**: it glides from tile to tile
  with its legs going and turns to face the way it walks. When it arrives
  it faces the camera again.

He also asked that a sprite never cover the unit above it ("can we have a
clip mask to shave their head", then "A is best"): a sprite and its
outline aren't drawn inside a tile another unit stands on. 0436 does that
for standing units; this ticket does it for a walking one (mockup M2).

He also said his Chapter 1 playtest **waits for this** ("B"), so 0804 has
this ticket in `blocked_by` and it is on the critical path in
`docs/ROADMAP.md`.

0436 draws each unit as the standing, front-facing frame of its bought
map sprite. Every sheet is 48×80: 3 columns (walking frames; the middle
one is the standing pose) × 4 rows (facing down, left, right, up) of
16×20 frames. Today a moving unit jumps one tile per step
(`ActionPlayback::walker_pos` in `crates/ui/src/screens/battle/
ai_phase.rs`, and the player's own move).

ADR-0038: what is shown goes in the `MapScene`; each map skin paints it.
The walking look is the sprite skin's; the glyph skin keeps today's look.

## Nick input

**Sign-off:** Nick sees it in his Chapter 1 playtest (0804), and before
that in a short screen recording or a few frames of the Quick Battle sent
to him (never committed: they show bought art). What to comment on: the
stepping speed, the walking speed, whether the map feels too busy. It
doesn't block the PR.

## Scope

**In:**
- The `MapScene` says, per unit: which way it faces, which walking frame
  it shows, and how far between two tiles it is.
- The sprite skins (0433's and 0436's mixed skin) paint that.
- Timing values in one place, *tunable*: a step on the spot every 250 ms
  (the frames go left foot, standing, right foot, standing); a walking
  unit crosses a tile in 200 ms and changes frame every 100 ms. These are
  the mockup's values. The animation-speed setting that already speeds up
  movement applies to walking too.

**Out (do not do):**
- Any change to the glyph skin's look: initials don't turn or step, and
  they keep jumping tile by tile. Every glyph snapshot stays unchanged.
- Any change to `core`, the bots or play records (ADR-0038): facing and
  frames are look only, and a unit's facing is never a rule.
- Attack or hit animations on the map; the combat scene (0413).
- Animated terrain (0437).
- Zoom (0439): **either order works.** Everything here is in tile
  fractions, so it follows whatever `tile_px` is.

## Implementation steps

1. **Scene** (`crates/ui/src/map_view/`, the `UnitView` from 0432):
   add `facing: Facing` (`Down`, `Left`, `Right`, `Up`; `Down` when not
   walking), `frame: u8` (0, 1 or 2; 1 is standing) and
   `offset: (f32, f32)` (how far toward the next tile, in tiles, each in
   `-1.0..=1.0`; `(0.0, 0.0)` when not walking). Add them to the scene's
   snapshot line only when they aren't the defaults, so existing scene
   snapshots don't change for still units.
2. **Who steps:** in `BattleScreen::scene`, a unit that hasn't acted,
   isn't fading and isn't walking gets `frame` from the screen's
   animation clock (the one the cursor pulse uses): the cycle 0, 1, 2, 1,
   one entry per 250 ms. An acted unit keeps `frame: 1`. All stepping
   units step together (one clock), as in the mockup.
3. **Walking:** where a walk is shown (`ActionPlayback` for the enemy
   phase and the player's own move), keep the tile-by-tile `pos` the
   rules and the camera use, and add the fraction of the way to the next
   tile as `offset`, the direction of that step as `facing`, and a
   `frame` that changes every 100 ms. The step sounds (0424) stay on the
   tile boundaries, where they are today.
4. **Sprite skin:** the unit's source frame is `(frame, row of facing)`
   in its sheet instead of 0436's fixed `(1, 0)`; `dest` moves by
   `offset × tile_px`, rounded to whole pixels. The outline, the HP bar
   and the effect arrows move with it. **Shaving:** no part of a walking
   unit's sprite or outline is drawn inside a tile that another unit
   stands on and that lies above the row the walker's feet are in (while
   it glides sideways under someone, only the columns under that tile
   are cut, so the sprite may need two sprite items with different
   clips). A walker crossing a tile an ally stands on is drawn in front
   of the ally and isn't cut there (*Claude's starting rule*; say so in
   the PR). 0433's generated test tileset has
   one picture per unit: a tileset entry without walking frames ignores
   `facing` and `frame` and only glides. Say in the tileset format
   (`assets/tilesets/README.md`) how an entry declares that it has the
   3×4 layout.
5. **Glyph skin:** reads none of the three fields.
6. Render or record the Quick Battle with the bought sprites and **look
   at it**: the feet don't slide oddly, a unit turns at each corner of
   its path, nothing flickers when a walk ends. Send it to Nick.
7. Update `crates/ui/README.md` and `look-and-feel.md` if a value
   changed.

## Acceptance criteria

- [x] Scene test: after a move that goes right and then up, the walking
      unit's `facing` was `Right` and then `Up`, its `offset` went from 0
      toward 1 on each step, and it ends with `Down`, `frame: 1`,
      `offset: (0, 0)`.
- [x] Scene test: a unit that can act changes `frame` over time; a unit
      that has acted keeps `frame: 1`.
- [x] 0433's `the_skin_never_changes_the_game` and
      `every_scene_feature_is_painted` cover the three new fields under
      both sprite skins.
- [x] Test: a unit walking along the row below another unit has no
      sprite pixel inside that unit's tile at any step (checked on the
      sprite items' `dest` and `clip`), and is whole again once past.
- [x] Every glyph-skin snapshot is unchanged.
- [x] `git status` shows no bought file; the PR has no picture made from
      one.
- [x] Nick was sent the recording or frames.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the frame cycle against the clock; the source rectangle for each
  facing and frame in a 48×80 sheet; `dest` for an offset; an entry
  without walking frames.
- Property: for any path, `offset` stays within one tile of `pos` and the
  unit ends on the path's last tile with no offset.
- Snapshot / integration: a scripted move in the Quick Battle under the
  sprite skin on the public fixture, read from the `MapScene`
  (ADR-0038), not from sprites.

## Completion notes

Done 2026-10-03.

- **Scene:** `UnitView` has `facing`, `frame` and `offset`
  (`map_view/scene.rs`). `BattleScreen::scene` sets them: every unit that
  can still act and isn't falling steps on the spot by the scene's clock;
  the walking unit (the player's move, or an AI unit's) is turned the way
  it goes, between two tiles, its legs going.
- **Timings, in one place:** `crates/ui/src/screens/battle/walk.rs`
  (250 ms a step on the spot; 200 ms a tile; 100 ms a walking frame). An
  AI walk held with Confirm goes four times as fast, legs and all.
- **Sprite skins:** a unit's picture is the frame of its facing and step,
  drawn where its offset puts it, with its outline, HP bar and arrow. It
  is cut where it passes under a unit standing above it (two sprite items
  while half under).
- **Tileset format:** a unit picture with an image of its own says
  `walk: true` when the image has the 3 × 4 layout
  (`assets/tilesets/README.md`). The test unit sheets and the bought
  `tiny_tales` tileset have it (the private repository is pushed and
  `assets-private.rev` pinned); the 24 px test tileset doesn't, and only
  glides.
- **Glyph skin:** reads none of the three fields (a test checks it). No
  glyph snapshot changed.

**Deviations**

- **Walking is slower under every skin**: 5 tiles a second instead of 12.
  The ticket's 200 ms a tile is the game's walking speed, since a skin
  can't change the timing or the step sounds. Six tests that waited 0.5 s
  for a four-tile walk now wait longer; nothing they check changed.
- The step-on-the-spot cycle starts on the standing frame (standing, a
  foot, standing, the other foot): the same cycle as the ticket's, a
  quarter turn on, so a battle doesn't open mid-step.
- `MapScene::to_text` shows `facing`, `offset` and `frame` only for a unit
  on its walk. The frame of a unit stepping on the spot isn't in the text:
  it goes by the clock alone, which the text leaves out.
- One sprite snapshot changed
  (`quick_battle_with_sprite_units_after_two_actions`): the units that can
  still act are caught mid-step.
- The walk code is `AiAction` and `Mode::Moving` (the ticket's
  `ActionPlayback` doesn't exist).
- Acceptance test on shaving: the walker's **picture** is never inside the
  tile of the unit above, at any step, and its **outline** isn't while any
  of its tile is under that unit. Standing still on the tile diagonally
  below, a 1-pixel edge of the outline reaches into that tile, as it does
  for any unit standing there since 0436; that wasn't changed here.

**Claude's starting rules** (for Nick to agree or veto)

- A walking unit that passes over a tile an ally stands on is drawn in
  front of the ally, and isn't cut there (the ticket's rule).
- A unit standing below a walker that passes above it keeps its head: it
  isn't cut for that moment (it would flicker).

**Looked at** with the bought sprites (`cargo xtask frame-png`, private
assets): the unit glides, turns at the corner of its path, faces the
camera when it arrives; nothing flickers at the end. On the glyph ground
the tile a walker has just left stays blank until it reaches the next one
(the glyph terrain is cleared under a unit's tile); that goes with the
bought terrain (0437).

**For Nick:** a short animation of the Quick Battle was sent in the
session (not committed). Say if the stepping or the walking is too fast,
too slow, or the map too busy: each is one number.

No follow-up tickets.
