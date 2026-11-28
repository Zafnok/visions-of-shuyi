---
id: "1380"
title: "The battle's windows framed by the UI kit's pictures in the picture look"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1379", "1238"]
nick_input: none
completed: 2026-11-27
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
- [x] `cargo xtask ui-import` writes the pieces; the private repository
      holds them and `assets-private.rev` names that commit.
- [x] `assets/ui/frames.ron` loads and validates; a content test fails
      on a missing frame name.
- [x] With the bought art and the picture look, every battle window in
      *In* draws the picture frame (a test reads the buffer's sprite
      items per window); with the glyph look or without the art, the
      snapshots are unchanged.
- [x] *Window look* switches the windows and *Map look* the map, each
      on its own, in every language pack.
- [x] `THIRD_PARTY_ASSETS.md` names the pieces used.
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

- **Engine** (engine#3249, carved from engine#2338 with Nick's OK,
  2026-11-27): `shuyi_assets::nine_slice` (`NineSlice`, `Borders`,
  `EdgeFill`, `CentreFill`), `frame::slice::nine_slice_sprites` and
  `GlyphBuffer::nine_slice`, and on `PanelLook` a `PanelFrame` (a nine-slice
  and an outset, drawn by `panel_border` over the border cells, never over
  the inside, never past the screen's edge) and a `TitlePlate` behind the
  title (`GlyphBuffer::title_plate` for a title a skin prints itself); the
  menu widget's `glyph::look` and `glyph::paint_with` take a caller's look.
  ADR-0163 (engine): only a panel's border becomes a picture. engine#2338
  keeps the rest (tiled pictures, every engine panel asking the game for
  its frame), engine#2235 the file format.
- **Game:** `cargo xtask ui-import` (`crates/tools/src/ui_import.rs`, its
  list `assets-src/ui/pictures.ron`) cuts the four pictures into the private
  repository's `game/ui/`: `panel.png` (Pixel Quest `F-UI_Panel_H`),
  `menu.png` (`F-UI_Panel_E`), `map_menu.png` (H, its cool pixels moved to
  hue 218 at saturation ×1.6 and brightness ×1.5, 1379's mockup's numbers;
  its fill `#263857`) and `plate.png` (Post-Baroque `UI_TitleB1` cut to its
  16-pixel caps and column 60). `assets/ui/frames.ron` (format in
  `assets/ui/README.md`) names the frames `panel`, `menu`, `map_menu` and the
  plate `title`, loaded and checked by `trpg_content::frames` (every name
  the windows ask for and no other, fills are colours, borders and caps
  leave a middle; a missing picture leaves its frame out). `crates/ui/src/frames.rs`
  turns a name into a `PanelLook` (`framed`, `bg`, `plate_behind`,
  `centred_title`, `title_at`) when *Window look* says pictures, the build
  has the picture and the skins are the PC's.
- **Windows framed:** the side panel, the unit menu and its lists (Arts,
  Skill, Item, Equip), the unit list, the map menu (royal blue), the
  forecast, unit info, the EXP, level-up and class boxes, the guides, the
  rewind list, the shop and its sale box, and the fight box; and, as the
  battle's other windows (not named in *In*, *the agent's starting rule*),
  the questions, the objective page, the notes box, the phase and
  objective banners and the warning box. `BattleMenuView` gained
  `map_menu` so the skin knows the map menu.
- **Credits:** two *Shipped items* rows (one per kit, each with its own
  link) and two entries in `credits.ron` (`barely_pixel_quest`, its recolor
  said; `barely_post_baroque`, its plate cut down), author "Barely Apes".
- **The private pictures:** the private repository's PR #43, merged;
  `assets-private.rev` names its merge (`80e6e856`), which also carries
  1384's content (its game PR merged first) and not the later #44 (1378's
  lamps, whose game PR moves the pin past it).
- Friction: game production lines about 880 added against an estimate of
  900 (under 2×). PR size about 1,560 lines of this ticket's own (the
  branch also carries 1238's, under it).
- Tests: the helper (picture, glyph look, picture missing, the Playdate),
  `frames.ron`'s checks, a picture-look test per window reading the sprite
  items by picture and rect (`frames::tests::assert_framed`,
  `assert_plated`), the Options rows, the saved setting, and the
  private-assets test that every frame resolves from the bought pictures.
  Every glyph-look snapshot is unchanged.

*The agent's starting rules* (for the Act 1 playtest, 1064):

- Every text a skin sets into a window's top border sits on the title
  plate: the forecast's title, the guides', the shop's name and its gold,
  the fight box's banner, a list's header (the Arts weapon, the pack's
  size), a question's or the notes' title. Text on a bottom border (the
  forecast's swap keys) has no plate.
- The frames are drawn 5 console pixels (10 screen pixels at scale 2)
  outside their window, as 1379's mockup; at the screen's edge they stay
  on screen (the side panel, unit info). The forecast's plate on the top
  row has its top cut by the screen: pulled down whole, its rim crossed
  the title.
- Boxes inside a window (unit info's and the level-up's portrait boxes,
  the guides' divider, the forecast's rules) keep their glyph lines: 1381
  frames the portraits.
- The shop's box and the guides are painted by one painter each, so the
  town's and the Preparations' shops wear the frame too.
- The `reduced` and `minimal` tiers (ADR-0131) draw the picture frames: a
  window is 8 to 20 sprites, about the cost of its line glyphs.
- A save from before this ticket starts *Window look* on its *Map look*;
  a new player starts on *Pictures*.

What to judge by playing (1064): the frames' sizes and the plate on the
top row; the menus' focus bar is still today's (1382 brings Pixel Quest's
bar and the hand).
