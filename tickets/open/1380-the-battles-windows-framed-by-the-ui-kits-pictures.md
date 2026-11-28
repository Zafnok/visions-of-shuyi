---
id: "1380"
title: "The battle's windows framed by the UI kit's pictures in the picture look"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1379", "1238"]
nick_input: none
completed:
---

# 1380 — The battle's windows framed by the UI kit's pictures

## Context

1379 picks the art for the picture look's windows (Nick, 2026-11-26:
"rework UI elements using assets from itch … from scratch like frames
and such … keep the glyph look as an option but I want it to be more
professional"). Its answers are in `docs/design/look-and-feel.md`,
*Windows (the picture look)*. This ticket builds the first part: the
picture frames themselves, and the battle's windows drawn with them.
1381 does every other screen, 1382 the cursor, gauges and icons.

**What 1379 picked** (2026-11-27; the full table and Nick's words in
*Windows (the picture look)*): Barely Games' *2D Pixel Quest Vol.3*
panels, in the private repository's
`library/barely-games/ui/pixel-quest-vol3/Panels/Panels/` (licence
`library/barely-games/licences/barely-apes__eula.txt`):

| Battle window | Picture | Slices (left, top, right, bottom) | Fill |
| ------------- | ------- | --------------------------------- | ---- |
| Side panel, forecast, unit info, the progress, guides, rewind and shop boxes, the fight box | `F-UI_Panel_H.png` | 22, 33, 22, 32 | its plum |
| Unit menu and its sub-menus | `F-UI_Panel_E.png` | 22, 22, 22, 22 | its plum |
| Map menu (Cancel on the map) | `F-UI_Panel_H.png` | 22, 33, 22, 32 | **royal blue**: the panel's cool pixels recoloured, the gold kept (1379's mock: hue 218 degrees) |
| Title plates (`Forecast`) | Post-Baroque `library/barely-games/ui/post-baroque/Titles/UI_TitleB1.png`: 16-pixel end caps, a plain body (column 60, rows 8 to 24) stretched to the title's width | | its dark |

The slices are cut where each edge is plain, so a corner ornament shows
once and an edge never repeats a pattern. Fills are **solid** (Nick:
"Solid"). At the default window scale (2) a frame is drawn at the font's
pixel; a frame whose border would cover the text is drawn a little
outside the window instead (1379's mock: 10 to 16 screen pixels).

**The kits are in the private repository from commit `b68863cc` on**
(its PR #42, `library/barely-games/`, `library/tiopalada/`,
`library/opengameart/`, `library/untied/`, `library/hypnobius/`). When
1379 merged, `assets-private.rev` was older (moving it then would have
carried other tickets' unmerged content), and `cargo shuyi private-assets
--library` checks out the pinned commit. So first check that the pin is at
or past `b68863cc`; if not, check out the private repository's `main` in
`assets-private/` and move the pin (`cargo shuyi private-assets --pin`) in
this ticket's PR, once no open public PR still needs an older pin for its
private content.

Today every box is a `GlyphBuffer::panel` with the engine's
`PanelLook` (`style: BoxStyle`, border, title, text and fill
colours). The engine's `crates/ui/src/frame/draw.rs` holds it, and the
lines are drawn from the font. 1238 (blocked_by) moves the last
hand-built boxes onto `panel`, so a frame drawn from a picture lands in
one place. The buffer already carries sprite items
(`GlyphBuffer::add_sprite`, `Sprite`) drawn at pixel positions over the
cells (ADR-0038: a picture is a sprite item from an asset file, never
per-pixel rectangles).

**The engine's picture frame is in the pin** (done by the pin move of
2026-11-27, `10cbb98b → 230080db`): engine PR #635 (engine ticket 3249,
engine ADR-0163), made for this ticket. `shuyi_assets::nine_slice`
(`NineSlice::new(picture, borders)` with `with_` methods, `Borders`,
`EdgeFill`, `CentreFill`), `shuyi_ui::frame::slice`
(`nine_slice_sprites`, `GlyphBuffer::nine_slice`), and in
`shuyi_ui::frame::draw` `PanelFrame { slice, outset }`, `TitlePlate {
slice, pad, text_top }`, `PanelLook::with_frame` and
`PanelLook::with_plate`: `panel` draws both, with no lines and no sprite
over the inside cells; with neither it draws as before. The menu widget
paints its rows with `glyph::paint_with(.., &look)` under a frame, with
no `├` `┤` joins. Run `cargo shuyi docs nine_slice` before using them.

## Nick input

None: 1379 decides the look. Pictures of each window in the PR come
from `frame-png` with `private-assets`, kept local; only a cropped
screenshot of the running game may go in the PR (ADR-0040, ADR-0118:
no real map). What is left to judge by playing goes on the Act 1
playtest's checklist (1064).

## Scope

**In:**
- ~~**Engine** (its own ticket and PR, `engine-change`): a frame drawn
  from a picture's nine slices~~ (done by the pin move of 2026-11-27:
  engine PR #635, above).
- **Game:** the chosen pack's pieces cut into the private repository's
  `game/ui/` by an importer (`cargo xtask ui-import`, its source list in
  `assets-src/ui/`, like `tileset-import`), and a public data file
  (`assets/ui/frames.ron`, its format in `assets/ui/README.md`) naming
  each frame (`panel`, `menu`, `plate`, `forecast` or as 1379 names
  them) with its picture's path and slice sizes. Missing pictures mean
  glyph frames: a public build looks as today.
- The **switch** as 1379 decided (Nick: "Two separate entries"):
  Options keeps *Map look: Pictures / Glyphs* and gains *Window look:
  Pictures / Glyphs*, in every language pack; it starts on *Pictures*
  when *Map look* is *Pictures* (starting rule), and a build without the
  bought art draws the glyph windows whatever it says.
- The battle's windows on the frames: the side panel, the unit menu
  and its sub-menus (Arts, Skill, Item, Equip), the forecast, the map
  menu, unit info (`glyph/info.rs`), the progress, guides, rewind and
  shop boxes, and the fight box (`glyph/fight_box.rs`).
- Credits: Barely Games' *Shipped items* row in `THIRD_PARTY_ASSETS.md`
  (1379 recorded the purchases) names the pieces used, where, and the
  recoloured map-menu fill as a modification; the credits screen's data
  (`assets/data/credits.ron`, "Barely Apes") with the new files' path
  under `private` (ADR-0051).

**Out (do not do):**
- Screens outside the battle and the dialogue box (1381).
- The menu cursor, gauges and icons (1382); the font (its own ticket if
  1379 picked one).
- The Playdate (its one-bit skins stay; 1184, 1294).
- Any change to a window's contents, size or place: only its frame and
  fill change.

## Implementation steps

1. Read *Windows (the picture look)* in `docs/design/look-and-feel.md`.
   Estimate the lines (ADR-0092): about 900 game lines (the engine's
   frame and its tests are in the pin) (importer, data file and its
   loading, the switch, the battle skins' looks), measured with `cargo
   xtask code-lines`.
2. ~~Engine ticket and PR (`engine-change`), then move the pin in this
   branch (`engine-change` §7).~~ Done by the pin move of 2026-11-27
   (engine PR #635).
3. `assets-src/ui/` source list and `cargo xtask ui-import`; run it,
   push the pictures to the private repository, `cargo shuyi
   private-assets --pin` (the art only, so no `content-change`).
4. `assets/ui/frames.ron` + its loader in `trpg-content` (validated:
   every name the skins ask for exists, slice sizes fit the picture).
5. One helper in `crates/ui` that turns a frame name into a
   `PanelLook` (picture when the look option says pictures and the
   picture loaded, else today's look), and every battle box listed
   above on it.
6. `frame-png` pictures of each window, both looks, checked by eye.

## Lesser targets

Playdate: unchanged (one-bit skins). The `reduced` and `minimal` tiers
(ADR-0131): the picture frame if it costs no more than the box lines
(it is a handful of sprites); otherwise the glyph frame, said in the
completion notes.

## Acceptance criteria

- [x] The engine's picture frame merged, the pin moved past it (the pin
      move of 2026-11-27, engine PR #635).
- [ ] `cargo xtask ui-import` writes the pieces; the private repository
      holds them and `assets-private.rev` names that commit.
- [ ] `assets/ui/frames.ron` loads and validates; a content test fails
      on a missing frame name.
- [ ] With the bought art and the picture look, every battle window in
      *In* draws the picture frame (a test reads the buffer's sprite
      items per window); with the glyph look or without the art, the
      snapshots are unchanged.
- [ ] *Window look* switches the windows and *Map look* the map, each
      on its own, in every language pack.
- [ ] `THIRD_PARTY_ASSETS.md` names the pieces used.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the frame name → `PanelLook` helper (picture, glyph look,
  picture missing).
- Snapshot / integration: the battle screens' existing snapshots stay
  as they are in the glyph look; a Harness test per window checks
  the picture frame's sprite items in the picture look (by name and
  rect, not pixels).
- Content: `frames.ron` validation.

## Completion notes

*(Filled in by the session that completes the ticket.)*
