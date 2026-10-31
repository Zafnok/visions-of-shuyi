---
id: "0817"
title: "Cinematic file format and player: timed shots, map pans, the logo"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0036", "0227", "0228", "0801", "0811"]
nick_input: answer-first
completed: 2026-10-12
---

# 0817 — Cinematic file format and player

## Context

Nick wants an intro cinematic on the title screen (ticket 0036,
`docs/design/title-screen.md`, *Intro cinematic*): shots of a battlefield,
the overworld, characters and conversation snippets, as long as the title
song, ending on the logo. This ticket builds the machinery with the two
simplest shots (a map pan and the logo) and a test cinematic. Later
tickets add the other shots (0818), put it on the title in time with the
music (0819) and make the real one (0820).

Content is data (ADR-0005): the cinematic is a file listing shots and when
each starts, so 0820 and 1010 change it without code.

Builds on: 0227 (each music cue's length), 0228 (the pan and zoom window),
0801 (battle files: a map with its units), 0811 (the logo art and its
loader).

**Added 2026-10-12 (0811 is done):** the logo is a picture, a sprite
(`assets/title/logo.png`, ADR-0059), over one of the title's backdrops,
with sparks drifting up. `trpg_ui::screens::title::glyph::paint_logo(ctx,
backdrop, sparks, buf)` paints exactly the logo shot (backdrop, sparks,
logo, nothing else): call it; don't copy it. The backdrop is the one the
title picked for this launch (`TitleView::backdrop`), so the shot and the
title show the same picture.

## Nick input

**Answer first:** ticket 0036 (answered 2026-10-03,
`docs/design/title-screen.md`, *Intro cinematic*): zoom is a whole step
(2×, 3×, 4×) and a shot keeps one size; shots change with **hard cuts**,
no fades between shots.

## Scope

**In:**
- The file format `assets/cinematics/*.ron`, its loader and validator in
  `trpg_content`, documented in `assets/cinematics/README.md`.
- `trpg_ui::cinematic`: a timeline and a player that draws the shot for any
  time `t`.
- Shot kinds `MapPan` and `Logo`.
- Hard cuts between shots (0036). The player still takes a whole-frame
  `brightness` (0..1) so the title can fade the cinematic in under its
  menu (0819); the file has no fades.
- `assets/cinematics/test.ron` and a debug tool to watch it.
- An ADR (`write-adr`) for the format and the "pure function of time" rule.

**Out (do not do):**
- Character and conversation shots (0818), the overworld shot (1010).
- Anything on the title screen, or following the music (0819). The debug
  tool runs on its own frame clock.
- The real title cinematic (0820).
- Units moving or fighting inside a shot: ticket 0827 (0036 chose it).

## Implementation steps

1. **ADR and format.** Recommended:
   ```ron
   (
     id: "test",
     music: "title",          // the cue whose length is the cinematic's length (0227's length_ms)
     shots: [
       (at: 0.0,  shot: MapPan(battle: "test", from: (4.0, 9.0), to: (12.0, 3.0), zoom: 2)),
       (at: 6.0,  shot: Logo),
     ],
   )
   ```
   `at` is seconds from the start. A shot lasts until the next shot's `at`;
   the last one lasts until the track's end. `from`/`to` are the map tile
   (fractions allowed) at the centre of the screen at the shot's start and
   end. `zoom` is 1, 2, 3 or 4 (0036: whole steps, one size per shot).
   There is no fade setting: shots cut. The length is the music cue's
   `length_ms` (0227); the file doesn't repeat it.
2. **Loader and validator** (`crates/content/src/cinematic.rs`,
   `Content::cinematics`, the all-assets test): shots sorted by `at`, the
   first at `0.0`, every `at` before the end; the battle exists; `from` and
   `to` are on its map; `zoom` is allowed; the music cue exists. All errors
   at once, with the file name.
3. **Timeline** (`crates/ui/src/cinematic.rs`): `Timeline::at(t) ->
   ShotAt { index, progress /* 0..1 */ }`, with
   `t` wrapped at the length. **Every shot is drawn from `t` alone**: no
   state carried from frame to frame. That is what lets 0819 follow the
   music, jump and loop, and lets tests snapshot any moment.
4. **Player**: `CinematicPlayer::new(ctx, id)` builds what the shots need
   once (each battle's scene: draw the whole map and its units at their
   starting places into one `GlyphBuffer`; move `BattleScreen::draw_terrain`'s
   body into a function both can call, and use `units::draw_unit`). If the
   map skin (0432, ADR-0038) is done, build a `MapScene` of the whole map
   instead and paint it with `ctx.map_skin`, so the cinematic follows
   whatever skin the battle uses; if it isn't, do as written here (0432
   then moves this code behind the skin).
   `CinematicPlayer::draw(&self, ctx, t, buf)` draws the current shot:
   - `MapPan`: the scene as a 0228 backdrop over the whole console, centre
     moving from `from` to `to` with an ease-in-out on `progress`.
   - `Logo`: 0811's title art, drawn by the same function `TitleScreen`
     uses (make it shareable; don't copy it).
   - `draw` takes a `brightness` (0..1) for the whole frame, used by 0819
     when the title's menu fades into the cinematic: `buf.dim` and the
     backdrop's brightness. It must cover everything in the frame: cells,
     rectangles and sprite items (ADR-0038). `dim` only changes cells, so
     fade items too (rectangles by colour, sprites by `opacity`).
5. **Debug tool** (`crates/ui/src/debug.rs`, `TOOLS`): "Play test
   cinematic", a screen that advances `t` by `dt` and loops; Cancel closes
   it. Help line per the `keyboard-input` skill.
6. `assets/cinematics/test.ron` on the test battle from 0801
   (`assets-test/battles/test.ron`), short (about 10 s).

## Acceptance criteria

- [x] Content validation loads `test.ron`; each validator error has a test.
- [x] Unit: `Timeline::at` at a shot's start, middle and end and at the
      wrap. A snapshot at `brightness` 0.5 dims cells and items alike.
- [x] Snapshot: the test cinematic at three times (start of the pan, its
      middle, the logo). Drawing the same `t` twice gives the same buffer.
- [x] Harness: F2 → "Play test cinematic" plays, loops and closes on
      Cancel.
- [x] The logo is drawn by the same code as the title screen's.
- [x] The ADR is written and listed in `docs/adr/README.md`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: loader/validator, `Timeline`.
- Property: for any `t`, `Timeline::at` returns a valid shot with
  `progress` in `0..=1`; `at(t)` equals `at(t + length)`.
- Snapshot / integration: the snapshots and the Harness test above.

## Completion notes

**Done**

- **Format, loader, validator**: `crates/content/src/cinematic.rs`,
  `Content::cinematics`, documented in `assets/cinematics/README.md`.
  Every problem of every file is reported at once, with the file name;
  each rule has a test.
- **`trpg_ui::cinematic`**: `Timeline` (whole milliseconds, wrapped at the
  length), `CinematicPlayer` (`frame` says what is shown as plain data,
  `draw` paints it), `MapPicture` (a whole map painted by a map skin; the
  scene camera now uses it too) and `ease_in_out`.
- **Brightness**: `GlyphBuffer::fade` darkens cells, rectangles, sprites
  (by opacity) and the backdrop's scene alike.
- **Debug tool**: F2 → "Play test cinematic"
  (`crates/ui/src/debug/cinematic.rs`): plays, loops, Cancel closes.
- **ADR-0060**, listed in `docs/adr/README.md`.

**Deviations from the plan**

- **`length: Music("title")` or `length: Seconds(10.0)`** instead of a
  required `music:` field. The ticket wanted a test cinematic of about
  10 s whose length is a music cue's, and the shortest cue is 48 s. A
  cinematic on a track still takes the cue's length and never repeats it.
- **The test cinematic is `assets-test/cinematics/test.ron`**, not
  `assets/cinematics/test.ron`: it names the test battle, and a file that
  ships may not name a test file (ADR-0057, which is newer than this
  ticket). `assets/cinematics/` holds only its README until 0819.
- **Times in milliseconds, map points in thousandths of a tile** inside
  `Content` (it is compared with `Eq`, so it holds no floats). The file
  is still written in seconds and tiles.
- **A whole number in `from`/`to` is the middle of that tile** (the ticket
  didn't say whether `(4, 9)` is a tile's corner or its middle).
- **The map is painted through the map skin** (0432 is done), at the
  art's own size whatever the player's map zoom is, so a shot's `zoom` is
  the only zoom. `BattleScreen::draw_terrain` no longer exists to share.
- **`draw(ctx, t, brightness, buf)`** with `t` in seconds, as the music
  clock gives it. The backdrop behind the logo is given with
  `with_backdrop` (0819 passes the title's; the debug tool uses the first).
- **The backdrop has no brightness of its own**: `fade` swaps in a faded
  copy of its scene, so no renderer changed.
- "Play test cinematic" sits before "Scene camera" in the debug menu, so
  two tests that count key presses to a tool were updated.

**Follow-up tickets touched** (wording only, to match the above): 0818 and
0827 (the test file's path), 0819 (the `draw` call; its stand-in
`title.ron` must pan over the Chapter 1 battle, not the test one).

**Gameplay rules decided here**: none. The look (hard cuts, whole zoom
steps, ease in and out) is as 0036 and this ticket say.

**For Nick**: nothing to see in the game yet. In a debug build or on the
Pages build, F2 → "Play test cinematic" shows six seconds of the test map
sliding sideways, a cut to the logo, and round again. The title gets its
cinematic with 0819 and 0820.
