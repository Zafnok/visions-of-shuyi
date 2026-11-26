---
id: "1362"
title: "Decide: the picture look's windows, a UI kit's frames, cursor, gauges, icons and font"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 1362 — Decide: the picture look's windows (frames, cursor, gauges, icons, font)

## Context

Nick (2026-11-26): "I want to rework UI elements using assets from itch.
We could use ones we already bought or search out new assets. We need to
go from scratch like frames and such. We can again keep the glyph look as
an option but I want it to be more professional as we're ramping up the
look and feel."

The battle map has been pictures since 0038 (the Tiny Tales tiles and
sprites, `docs/design/look-and-feel.md`). Every window is still drawn
with font cells in **both** looks. The side panel, the unit menu, the
forecast, the dialogue box and its portrait frames, and every menu
screen use thin single or double box-drawing lines and the 8×16 console
font, with a flat dark fill and a gold bar for the focused row.
Mapped out on 2026-11-26 with `cargo shuyi frame-png --battle ch01`
and `--scene ch01_prebattle` (built with `private-assets`).

This ticket picks the art for the windows in the **picture look**.
The build tickets are 1363 (the frames on the battle's windows),
1364 (every other screen and the dialogue box) and 1365 (cursor,
gauges, icons). They wait on it.

### What is already owned

- **Tiny Tales Code Ark UI** (in the Mega Tiles bundle,
  `assets-private/library/tiny-tales/ui/code-ark-ui/`). It has 9-slice
  frames in three sizes and two shapes (square and chamfered corners),
  `mono` versions made to be recoloured, window textures, three gauge
  styles, cursors, icons in four colours, and keyboard and gamepad
  button pictures. Its look is **sci-fi** (neon cyan/pink edges); the
  store page calls it "futuristic". Recoloured to gold or bronze, its
  plain frames could pass as fantasy.
- **Tiny Tales Heroes 1 RPG Maker window skin**
  (`library/tiny-tales/ui/heroes-1-rpgmaker-system/Window_1x.png`,
  64×64). A pale blue/white bordered window in RPG Maker's layout
  (frame, fill, cursor, arrows, text colours). Small and plain.
- **MattWalkden Fantasy Battle Pack** (`library/mattwalkden/UI Elements/`):
  map pieces only (a selection cursor, range tiles, movement arrows, a
  grid). No windows.
- **ELV Games Game Assets Mega Bundle**
  (<https://itch.io/s/79053/game-assets-mega-bundle>, bought 2026-11-24;
  its terms: personal and commercial use, edits allowed, "Credits to
  ElvGames", no resale, no generative-AI use, no crypto/NFT; its pages
  say no generative AI was used). **Its UI packs are the first
  candidates**, being owned and fantasy (read from the pages
  2026-11-26). Only the Neatpixels fonts are downloaded so far
  (`library/elv-games/fonts/`); Nick downloads the rest from his itch
  library (**Setup**):
  - **GUI KIT Pixelart Asset Pack**
    (<https://elvgames.itch.io/inventory-and-ui-buttons>): wood and
    parchment frames, inventory grids and boxes, buttons with and
    without text, "separated sprites to use with 9slices", and 335
    icons (armor, swords, equipment, UI symbols) on dark navy tiles in
    4 styles. No bars, cursor or font listed.
  - **Item Icons 16x16**, *Fantasy Dreamland* (288: swords, staffs,
    axes, bows, spears, armor, rings…) and *Rogue Adventure* (over 1,000,
    darker); **Enchanted Sword/Staff/Spear/Bow Icons 16x16**; 32×32
    weapon and armor icon packs.
  - **Neatpixels** fonts (5×5 to 7×7 pixels, downloaded): smaller than
    the 8×16 cell, so mostly for numbers or titles if at all.
  - **Keyboard UI** and **Controller Buttons** 16×16 (for hints: 0255,
    out of scope here).
  - Free Inventory for Farming RPG, Free Inventory Sci-Fi: farm and
    sci-fi inventories, a poor fit.
  The bundle's ELV art already draws Chapter 1's map (`rogue_adventure`
  look), so its UI may sit with the map better than a third artist's.
- Mega Tiles sells **no fantasy UI pack**; Code Ark is its only one (a
  commenter asked for one for the fantasy set; no answer, 2026-11-26).

### Itch candidates to buy, if the owned packs fall short (read from the store pages 2026-11-26; nothing downloaded)

The ELV GUI kit has no cursor or bars; a free or cheap pack below can
fill those gaps.

Prices and terms as stated on each page. **Read the licence inside the
download before using a pack** (ADR-0013: commercial use, no copyleft,
no non-commercial; a "no redistribution of the files" clause is fine for
private art, ADR-0040).

| # | Pack | Price | Terms on the page | Scale | Holds |
| - | ---- | ----- | ----------------- | ----- | ----- |
| 1 | Stonebase UI Pack, itchabop — <https://itchabop.itch.io/stonebase-ui-pack> | $10 | commercial OK, no redistribution, no AI | 16 px | 9-slice frames in stone, wood, metal and parchment; headers, stone tablets, arrow buttons, a health bar, vials, item/equipment/spell icons, a dialogue box, a portrait frame. No cursor |
| 2 | RPG UI pack, Franuka — <https://franuka.itch.io/rpg-ui-pack> | $3.99 | commercial OK, credit optional, no AI | 16/32/48 px | 9-tile menu boxes, 12 cursors, 32 mini icons, 3 title banners, dialogue boxes, HP/MP orbs (no bars), 3 fonts |
| 3 | Tiny RPG – Dragon Regalia GUI, tiopalada — <https://tiopalada.itch.io/tiny-rpg-dragon-regalia-gui> | free / pay what you want | CC0 (asks: no AI training, no crypto), no AI | not stated | 5 9-slice frames, stretchable name plates, a portrait frame, 5 animated cursors, HP/MP/EXP bars, arrow buttons |
| 4 | Medieval RPG GUI, 3blackpixels — <https://3blackpixels.itch.io/medieval-rpg-gui> | $3 | commercial OK, credit optional, no AI | made for 16 px icons | 9-slice panels, 2 header plates, buttons, 3 health bars, a portrait frame, cursors; 5 icons |
| 5 | Royal RPG Pixel Art GUI, untiedgames — <https://untiedgames.itch.io/royal-rpg-pixel-art-gui> | $2.50 | commercial OK, **credit required** | small | cute, colourful; 9-slice windows in 3 colours, 158 icons, arrows, meters. No portrait frame |
| 6 | Complete UI Essential Pack, Crusenho — <https://crusenho.itch.io/complete-ui-essential-pack> | $3.51 | CC BY 4.0 (a "no resale" line in the comments is unresolved) | 32 px | 13 themes (wood, stone, paper, runewood…), animated selection, bars, banners, a profile frame |
| 7 | Kenney Pixel UI Pack / Fantasy UI Borders — <https://kenney.nl/assets/pixel-ui-pack>, <https://kenney.nl/assets/fantasy-ui-borders> | free | CC0 | not stated | generic; a safe fallback |
| 8 | 2D Pixel Quest Vol.3 – UI/GUI, Barely Games — <https://barely-games.itch.io/2d-pixel-quest-the-uigui> | $8.99 | separate EULA (unread) | 32 px | 26 panels, 5 title bars, 19 bars, 200 icons, 2 cursors; in development |
| 9 | Raven Fantasy Icons, Clockwork Raven — <https://clockworkraven.itch.io/raven-fantasy-icons> | $35 (commercial tier) | commercial at that tier, no AI | 16/32/64 px | icons only, 8,000+ |
| 10 | 16x16 RPG Icon Pack, mochamiruku — <https://mochamiruku.itch.io/16x16-rpg-icon-pack> | $2.25 | commercial OK; AI use not stated | 16 px | icons only, 258 |

**Set aside:** Pixel UI Kit (gegx) and Pixel UI Fantasy Free
(heyheythere): marked AI-assisted (memory: AI art only sparingly).
Veyroa's "Fantasy RPG UI Pixel Pack": probably a re-upload of other
artists' packs. RPGui HUD, JRPG UI (LetheDiana), Hyperknife Pixel GUI,
Free Basic Pixel UI: no licence on the page. ToffeeCraft's free tier:
personal use only.

**Not searched yet:** Japanese free sites and BOOTH (Nick's art
candidates have come from there before: Shironejiya, Pixel Flag), for
example RPG Maker window skins by Japanese artists. Search them in
round 1, as the `ask-nick` skill's art candidates call for.

## Nick input

**Decision**, asked with the `ask-nick` skill over rendered mockups of
the picture look (memory: picture look only, the cursor shown). Render
the same four screens in each candidate: the battle with the unit
menu open, the forecast, a dialogue line with both portraits, and one
menu screen (Options or Preparations). Use `frame-png` with a
throwaway env-var skin variant (memory: staging fights for mockups).
Mockups made from bought or downloaded art stay local, never committed
(ADR-0040). Expect several rounds; offer mixes (one pack's frames with
another's cursor and bars).

The questions, in order:

1. **The windows' frames and plates:** which pack, and which of its
   materials (stone, wood, parchment, dark blue…), for the panels, the
   menus, the title and name plates. One material everywhere, or one
   for the battle and another for the menus and dialogue?
2. **Inside a window:** a flat fill, the pack's texture, or
   see-through over the map (Fire Emblem's menus are see-through blue).
3. **The menu cursor and focused row:** the pack's pointer (a hand or
   arrow beside the row), a highlighted bar like today's, or both.
4. **Gauges:** the HP bar on the side panel, the forecast and the
   fight, and the EXP bar. Pictures from the pack, or today's flat bars.
5. **Icons:** whether lists show an icon beside a weapon, item or spell
   (weapon type), and the stat names. This is new UI: ask, never
   assume (memory: no invented UI). If yes, from which pack.
6. **The font in the picture look:** today's 8×16 font, or a pixel
   font that fits the same 8×16 cell (one of a pack's fonts, or an
   OFL one). Any font that changes the cell size is an engine-scale
   change (1320's *More cells*); say so if he leans that way.
7. **The option:** today's Options entry is *Map look: Pictures /
   Glyphs*. Starting rule: **one entry switches the map and the
   windows together**, renamed *Look* (in every language pack).
   He may want them separate.
8. **The glyph look** keeps its box-drawing windows (Nick: keep it as
   an option). Ask only whether it should borrow anything (for
   example the new cursor's shape drawn in the font).

Downloading the ELV UI packs from his itch library, and buying any new
pack, is Nick's (**Setup**): ask for the ELV downloads at the start of
round 1. Each purchase is recorded with the artist and link (memory:
credit everyone).

## Scope

**In:**
- Round 1: the ELV bundle's GUI kit and icons first (owned), the Code
  Ark kit recoloured, then the bought-pack candidates and the Japanese
  sites, rendered on the four screens.
- The `ask-nick` rounds until each question has an answer.
- `docs/design/look-and-feel.md`: a new section *Windows (the picture
  look)* with each answer, Nick's words, and Claude's starting rules
  marked as such.
- After the purchase: the pack in the private repository's
  `library/<artist>/` (`INDEX.md`, licence in `licences/`), and its
  entry in `THIRD_PARTY_ASSETS.md` (private, with the price and date).
- Update 1363–1365 with the answers (file names, pack, materials). If
  Nick picks a new font, file its ticket (`write-ticket`), blocked by
  1363, and name it in 1365.

**Out (do not do):**
- Any code or skin change (1363–1365).
- The Playdate's windows (its one-bit look, 1184 and 1294).
- The world map's place icons (1082), class icons (1006), the
  controller button pictures in hints (0255).
- Committing a mockup or any picture cut from the packs.

## Lesser targets

Playdate: unchanged (its one-bit skins, 1184). The `reduced` and
`minimal` graphics tiers (ADR-0131): ask nothing; the build tickets
draw the glyph windows there unless a picture costs nothing.

## Acceptance criteria

- [ ] Each of the 8 questions answered by Nick, his words quoted in
      `docs/design/look-and-feel.md`, *Windows (the picture look)*.
- [ ] Any pack chosen is bought or downloaded by Nick, sorted into the
      private repository and credited in `THIRD_PARTY_ASSETS.md`.
- [ ] 1363, 1364 and 1365 updated with the answers; a font ticket filed
      if a new font was picked.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

None (a decision). `cargo xtask ticket-lint` and `cargo xtask
check-leaks` on the PR.

## Completion notes

*(Filled in by the session that completes the ticket.)*
