---
id: "0439"
title: "A key and a button toggle the battle map's zoom (1× / 2×), and the game remembers it"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0433"]
nick_input: decision
completed: 2026-10-12
---

# 0439 — Map zoom toggle (1× / 2×), remembered

## Context

Nick, 2026-10-02 (ticket 0038), choosing between the bought tiles at
their own size (spike render C: a 16-pixel tile, about 34×28 tiles on
screen) and at double size (render C2: a 32-pixel tile, about 17×14 tiles,
close to Fire Emblem on the GBA): "A key/button to toggle zoom (1x / 2x)
with memory (i.e. going to next battle, keep the setting)".

Rule: `docs/design/look-and-feel.md`, *Battle map: bought tiles and unit
sprites*, *Zoom*.

ADR-0038: "Tile size belongs to the skin. The camera and viewport ask the
skin how many tiles fit." 0432 and 0433 build that; this ticket makes the
sprite skin's tile size switchable while a battle is running.

**From Nick's playtest (ticket 0804, 2026-10-12):**

> "I think we can scale the battle map bigger. tile size / overworld unit
> size. rn it's quite small idk if the original assets are this small or
> we shrunk them to glyph size but we can make the 2D size bigger"

The art is not shrunk: the bought tiles and unit sprites are 16 pixels and
the map shows them at their own size (1×). 2× is this ticket. What his
note adds is a question nothing has decided: **which zoom a new player
starts at.** Ask it with the others below, with both frames of Chapter 1's
first turn side by side. *(the agent's suggestion to put to him, not a
decision: start at 2×, since 1× is what he found too small.)* Tell him
what 2× costs: Chapter 1's map is 24×16 and 2× shows about 17×14 of it, so
the map scrolls.

## Nick input

**Also asked at the start (added 2026-10-12):** the zoom a new player
starts at, 1× or 2×. Record it in `look-and-feel.md`, *Zoom*.

**Decision, asked at the start of this ticket** (`ask-nick`, before any
code): the default key in both layouts, the controller button, and
whether the action is optional. Default keys are Nick's (`keyboard-input`
skill, rule 4); don't pick them yourself. Asked once on 2026-10-02, Nick
put it off to this ticket: "I think we can ask this as we build the zoom
feature in another ticket? I will decide it later".

What was offered then, to offer again with anything else that is free in
`docs/design/controls.md`: `Q` in the right-handed layout and `P` in the
left-handed one (finger mirrors, beside the other keys); the right
trigger (`RT` / `R2` / `ZR`) on a controller (the left trigger is
Rewind); optional, like Unit info and Rewind (may be left without a key).
Record his answer in `controls.md`: the default-keys table, the
controller table and the required/optional lists.

**Sign-off:** Nick tries it in the Pages build after merge (the toggle on
the test tileset from the debug menu if the bought-art skin isn't in yet).

## Scope

**In:**
- A new input action, Zoom map, per the `keyboard-input` skill.
- The sprite skin draws at 1× or 2×: tiles, unit sprites, HP bars, marks,
  cursor and path all scale together.
- The camera keeps the cursor on screen when the zoom changes, and
  scrolls as it does today.
- The choice is saved with the player's settings and used by the next
  battle and the next time the game starts.

**Out (do not do):**
- Other zoom levels, smooth zooming, a mouse wheel.
- Zooming the glyph skin: its tile is two font cells. With the glyph skin
  the action does nothing and the help bar doesn't list it. (0228's
  cinematic camera zooms a glyph scene for cutscenes; it isn't reused
  here.)
- An Options-menu row, unless 0805 is already done: then add "Map zoom:
  1× / 2×" there; if not, add a line to 0805 to do it.
- Zooming the side panel, menus or text.

## Implementation steps

1. **Action** (`keyboard-input` skill, *Adding an action*): `Action::ZoomMap`
   in `crates/content/src/keymap.rs`, in `Action::ALL`, required or
   optional as Nick answered; defaults in `assets/data/keymap.ron` for
   both layouts and the `pad` table exactly as recorded in `controls.md`;
   a row in the Key bindings screen (`ROWS`).
2. **Skin**: `SpriteSkin` gets a `zoom: u8` (1 or 2). `tile_px` and
   `unit_px` are multiplied by it; every `dest` is in whole console
   pixels, so art pixels stay square. `view_tiles(area)` already follows
   `tile_px` (0433). HP bar thickness, the side mark, the cursor's corner
   arms and the path's width scale by the zoom too.
3. **Setting**: `map_zoom` in the player's `Settings` (the same storage as
   `cursor_style`; if 0805's `Settings` struct isn't there yet, keep it in
   `Ctx` and persist it under its own storage key, and add a line to 0805
   to move it). Default 1×. An unreadable or out-of-range saved value
   falls back to 1× without an error.
4. **Battle screen**: on `Action::ZoomMap`, flip the setting, save it,
   rebuild the skin's zoom, and re-centre the camera so the cursor's tile
   stays visible (clamped to the map as scrolling is today). Works while
   browsing, with a unit selected, while targeting, and in the enemy
   phase; does nothing in menus that cover the map. Not a `Command`: the
   zoom is not game state (ADR-0004, ADR-0038), so rewind, bots and play
   records never see it.
5. **Help**: name the action in the map menu's or help bar's key list via
   `ctx.help_keys()` (rule 2), only when the current skin can zoom. Check
   `the_longest_help_bar_fits_on_every_pad` still passes. If the hint
   goes on the toggles line (with the danger zone and auto-end): ticket
   0445 decides where that line's hints sit. Either order works: if 0445
   is done, place the zoom hint by its rule; if not, put it where the
   line is today and 0445 moves it.
6. Render Quick Battle on the test tileset at both zooms with
   `cargo xtask frame-png` (0232 if done) and **look at them**: nothing
   cut at the view's edge, a menu opened beside a unit still sits beside
   it at 2×.

## Acceptance criteria

- [x] Nick's key, button and required-or-optional answer is in
      `controls.md` with his words, and `keymap.ron` matches it (the
      keymap-matches-design test).
- [x] Pressing the Zoom map key on the sprite skin switches between 1×
      and 2×; the cursor's tile is inside the view after each switch (a
      scene assertion, for cursor positions at each map corner).
- [x] The zoom survives leaving the battle and starting another, and a
      restart (a storage test).
- [x] With the glyph skin the action changes nothing, and the help text
      doesn't list it.
- [x] `the_skin_never_changes_the_game` (0433) passes with the script run
      at 2×: same `BattleState`, same cursor tiles, same sounds.
- [x] Rebinding the action works, and it shows `! not mapped` when
      unbound (if optional).
- [x] Snapshots of Quick Battle on the test tileset at 2×.
- [x] `cargo xtask check-keys` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `view_tiles` and every `dest` at both zooms; the camera
  re-centring; the setting's load, save and fallback.
- Property: at either zoom, every painted rectangle lies inside the map
  area and tiles don't overlap.
- Snapshot / integration: as listed, plus one Harness test using the
  default key and one with the action rebound.

## Completion notes

**Nick's answers (2026-10-12, asked before the feature was written, with
Chapter 1's first turn rendered at both zooms):** a new player starts at
**2×**; the key is **`Q`** right-handed and **`P`** left-handed; the
controller button is the **right trigger** (he asked about a bumper; both
are Previous / Next ready unit, so he took the trigger); the action is
**optional**. Recorded in `controls.md` (*Zoom map*, both tables, the
required/optional lists), `look-and-feel.md` (*Zoom*) and `options.md`
(*Map zoom*).

**What was done**

- `Action::ZoomMap` (optional), its defaults in `keymap.ron`, its row on
  the Key bindings screen.
- `MapZoom` (1× / 2×) in `Settings`, saved as its number; a number the
  game doesn't have falls back to the default. `Ctx::set_map_skin` draws
  whatever skin is put in at the player's zoom, so the zoom is kept from
  one skin to the next and from one launch to the next.
- The sprite skin draws at the zoom: `Grid` carries the scale, and every
  mark on a tile (unit, outline, HP bar, effect arrow, cursor arms, path)
  is worked out at the art's own size and placed with `Grid::zoomed`, so
  a 2× frame is the 1× frame with every pixel doubled (a test checks
  exactly that). `MapSkin::zoom` / `at_zoom`: the glyph skin, and unit
  pictures on glyph terrain, say they can't zoom.
- The battle screen flips the setting on the key. The cameras were
  already refitted when the view's size changes (0433), so the cursor's
  tile is centred after a switch. Not a `Command`.
- The Options screen has a `Map zoom` row (0805 is done, so it is here).
- Screenshots: `docs/screenshots/0439-zoom-1x.png`, `0439-zoom-2x.png`
  (Chapter 1's first turn, the bought art, as shown to Nick).

**the agent's starting rules (Nick can veto; `controls.md`, *Zoom map*, and
`options.md`):**

- The key works whenever the map is what you are looking at: browsing,
  with a unit selected or walking, while picking a target, and during the
  enemy's turn. It does nothing while a menu or a yes/no question is
  open, during a fight's playback, or on the rewind screen.
- After a switch the cursor's tile is in the middle of the view (as near
  as the map's edges allow).
- The hint is first on the toggles line: `q zoom: 2× · w danger zone: OFF
  · Shift+Space auto-end: OFF`. No sound, no message.
- `Zoom map` is the row before `Map menu` on the Key bindings screen.
- The Options row `Map zoom` sits under `Map look` and shows only when
  the game has pictures to zoom.
- "Pick your layout" lights `Q` / `P` on its keyboards as keys in use, but
  its list of keys is unchanged (the panel has no free line).

**Deviations**

- The ticket said "Default 1×"; Nick picked 2×. Tests that relied on the
  test tileset showing as many tiles as before now say which zoom they
  mean (`map_skin.rs`), or expect the 2× numbers.
- Tests that used the right trigger or `q` as "a button / key with no
  job" now use the right stick's press and `z`.
- No new help-bar hint on the key-help row: it is already at its longest
  on a PlayStation pad, so the hint went on the toggles line. A line in
  0445 says so.
- No ADR: the zoom follows ADR-0038 as written ("Tile size belongs to the
  skin"); `crates/ui/README.md` documents `zoom` / `at_zoom` /
  `set_map_skin`.

**For Nick, when playing on Pages:** the Pages build has no bought art, so
the map is glyphs and the key does nothing there. To try it: debug menu →
`Map skin` until it says `test tileset`, start Quick Battle, press `Q`.
The real look is `cargo run -p trpg-app --features private-assets`.
