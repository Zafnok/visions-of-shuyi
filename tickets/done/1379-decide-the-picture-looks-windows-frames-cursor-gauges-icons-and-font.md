---
id: "1379"
title: "Decide: the picture look's windows, a UI kit's frames, cursor, gauges, icons and font"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
nick_input: decision
completed: 2026-11-27
---

# 1379 — Decide: the picture look's windows (frames, cursor, gauges, icons, font)

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
The build tickets are 1380 (the frames on the battle's windows),
1381 (every other screen and the dialogue box) and 1382 (cursor,
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
- **First, before any search for new packs: check every bought asset
  for UI candidates** (Nick, 2026-11-26: "we should ensure all bought
  assets are checked for ui candidates before ruling them out and
  scouring for more"). Go through **every** purchase, by its store
  page's full product list, not only what is downloaded into the
  private repository. That means every private row of
  `THIRD_PARTY_ASSETS.md` and every folder of the private repository's
  `library/` and `originals/`. Today those are the Tiny Tales bundle
  (all 37 products), the ELV Game Assets Mega Bundle (all of it), the
  Electric Lemon *Pixel Grit* bundle, MattWalkden's *Fantasy Battle
  Pack*, Pixel Flag, Shironejiya and Bytewright. For each pack, write
  down what UI it holds (frames, plates, buttons, cursor, bars, icons,
  fonts, dialogue box, portrait frame) or "none", in a table in the
  completion notes. Nick downloads the packs not yet downloaded
  (**Setup**).
- Only for a part no bought pack covers (or that Nick rules out after
  seeing it) look further: the itch candidates above, then the
  Japanese sites. The completion notes name the gap each new pack
  fills.
- Round 1 renders: the bought candidates first (the ELV GUI kit and
  icons, the Code Ark kit recoloured, whatever the check finds), then
  new packs only for the gaps, on the four screens.
- The `ask-nick` rounds until each question has an answer.
- `docs/design/look-and-feel.md`: a new section *Windows (the picture
  look)* with each answer, Nick's words, and Claude's starting rules
  marked as such.
- After the purchase: the pack in the private repository's
  `library/<artist>/` (`INDEX.md`, licence in `licences/`), and its
  entry in `THIRD_PARTY_ASSETS.md` (private, with the price and date).
- Update 1380–1382 with the answers (file names, pack, materials). If
  Nick picks a new font, file its ticket (`write-ticket`), blocked by
  1380, and name it in 1382.

**Out (do not do):**
- Any code or skin change (1380–1382).
- The Playdate's windows (its one-bit look, 1184 and 1294).
- The world map's place icons (1082), class icons (1006), the
  controller button pictures in hints (0255).
- Committing a mockup or any picture cut from the packs.

## Lesser targets

Playdate: unchanged (its one-bit skins, 1184). The `reduced` and
`minimal` graphics tiers (ADR-0131): ask nothing; the build tickets
draw the glyph windows there unless a picture costs nothing.

## Acceptance criteria

- [ ] The completion notes hold the table of every bought pack and the
      UI it holds (or "none"), made before any new pack was searched.
- [ ] Each of the 8 questions answered by Nick, his words quoted in
      `docs/design/look-and-feel.md`, *Windows (the picture look)*.
- [ ] Any pack chosen is bought or downloaded by Nick, sorted into the
      private repository and credited in `THIRD_PARTY_ASSETS.md`.
- [ ] 1380, 1381 and 1382 updated with the answers; a font ticket filed
      if a new font was picked.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

None (a decision). `cargo xtask ticket-lint` and `cargo xtask
check-leaks` on the PR.

## Completion notes

Decided 2026-11-26/27 over eleven rounds with Nick; the answers, his
words and the starting rules are in `docs/design/look-and-feel.md`,
*Windows (the picture look)*, and in the table of `docs/design/README.md`.

### Every bought pack's UI, checked before any new search

Read from each pack's files: the private repository's `library/` and
`originals/`, and the releases `originals-elv-games` and
`originals-bytewright` (every download as bought). The ELV and ByteWright
downloads were already in those releases, so the **Setup** step (Nick
downloading the ELV UI packs) was not needed.

| Seller, pack | UI it holds |
| ------------ | ----------- |
| Tiny Tales (Mega Tiles bundle, 37 products) — *Code Ark UI Assets* | Sci-fi kit: 9-slice frames in 3 sizes, square and chamfered, `mono` versions to recolour, window textures, 3 gauge styles, cursors (a cyan-tinted glove and arrows), icons in 4 colours, keyboard and gamepad buttons |
| Tiny Tales — *Heroes 1: A New Beginning* | RPG Maker window skin (`Window_1x/2x/3x`: frame, fill, cursor, arrows, text colours) |
| Tiny Tales — *Heroes 2*, *WorldMap* | Battle weapon animations only; the world map's place icons (1082) |
| Tiny Tales — the other 33 products (battlers, NPC sprites, tilesets, battlebacks, generators) | none |
| ELV Games bundle — *GUI KIT Pixelart* | 4 styles (orange neon, cyan neon, steel/navy, light wood): 9-slice inventory frames, slots, buttons, health bars (styles 1–2), 335 icons in 4 styles; no cursor, font or plates |
| ELV — *Free Inventory*, *Retro Inventory*, *Free SciFi Inventory* | Farm/retro inventory frames, hearts, health bars; sci-fi boxes |
| ELV — *Keyboard UI*, *Controller Buttons* | Hint keys and buttons (0255) |
| ELV — *Item Icons* (Rogue Adventure ~1000, Fantasy Dreamland extras, enchanted 16 px, 32 px weapons and armour) | Icons |
| ELV — *Neatpixels* fonts (4) | 5–7 px fonts |
| ELV — the other ~280 products (tilesets, characters, monsters, effects, sounds, tools) | none |
| ByteWright bundle (33 packs) — *Quest UI Megapack* (= *Parchment*, *Tavern*, *Dungeon*, *Royal*, *Arcane*, *Neon UI*) | 24 colourways: 9-slice panels, inset, plate, ribbon, portrait frame, divider, dialogue box, choices (focus bar and arrow), mouse cursors (hand, arrow), bars (health, mana, XP, stamina…), buttons, slots, controls, key and pad pictures, HUD pieces, banners, title fonts, 4 text fonts, 72 icons (10 stat icons, 62 items) |
| ByteWright — *Pixel Loot Megapack* | 858 item icons, 16/32/64 px |
| ByteWright — *Fantasy RPG Fonts*, *Pixel Font Megapack*, *Horror & Dungeon*, *Arcade*, *Sci-Fi*, *Coin-Op*, *Slime & Cute* fonts | Fonts (Kingsrune and Crypt fit 8×16, capitals only) |
| ByteWright — *Pixel TCG Card Frames* | Card frames |
| ByteWright — the other packs (tilesets, creatures, effects, sounds, weather, hero maker) | none |
| Electric Lemon *Pixel Grit* (18 downloads) | none (tilesets, characters, creatures) |
| MattWalkden *Fantasy Battle Pack* | Map pieces only: selection cursor, range tiles, movement arrows, grid; no windows |
| Pixel Flag (64 TRPG pieces), Shironejiya (*monster_s*) | none |

### The rounds

1. Eleven owned kits (ByteWright Royal, Royal Obsidian, Stone, Oak,
   Ebony, Parchment, Crystal; ELV GUI kit steel and wood; Code Ark in
   gold; the Heroes window skin) on the four screens: *"I like royal,
   parchment, elv4, and crystal the best, but I don't wanna settle"*.
2. BOOTH and Japanese free sites (nothing fit: streaming overlays and
   modern sets); itch.io browsed in Nick's Chrome (the built-in browser
   was stopped by itch's bot check): 15 kits' store previews. He picked
   A, B, D, M, O; bought A, D, O (skipped B); M is free. Asked about
   unTied's sale bundle ($49.99 for 75 packs): worth it only for its
   effects and skill icons, not for the windows.
3. The bought kits alone and two mixes. *"do a better job of stretching
   the hp bars … it has the frames copied twice"* and *"the dragon
   regalia gold pointer looks out of place"*: the mockup now cuts each
   bar at its plain middle and draws no pointer; Mix 1 led.
4–5. A pointing glove (Code Ark's, its tint removed: *"looks ugly"*), the
   map menu and Options in three frames and then four blues, seven
   hand-cursor packs.
6–7. The CC0 hand with a nudge (GIF); Options fills; five dialogue and
   portrait frames; the hand at two sizes.
8–10. Ornate portrait frames: Pixel Quest's knot corners, Post-Baroque's
   scrollwork, Royal Crown's gem corners, Dragon Regalia larger and
   mirrored, Hypnobius's *Dark Ages UI* (free); a mockup bug fixed on
   Nick's catch (*"why are there black pixels where there should be
   transparent?"*: the mock had painted dark where a frame doesn't
   reach); names kept inside their plates and frames.
11. The set; solid vs see-through; weapon and stat icons; the two owned
   fonts that fit the cell; the option; the glyph look.

### New packs and the gap each fills

All bought or downloaded by Nick's choice, recorded in
`THIRD_PARTY_ASSETS.md` (purchase record) and sorted into the private
repository by its PR #42 (`library/barely-games/`, `library/untied/`,
`library/tiopalada/`, `library/hypnobius/`, `library/opengameart/`; the
downloads in the release `originals-ui-kits-1379`):

- Barely Games' *2D Pixel Quest Vol.3* ($9.99): the frames, focus bar and
  HP gauge — none of the owned kits was settled on (round 1).
- Barely Games' *Post-Baroque UI/GUI Kit* ($8.99): the name and title
  plates.
- tiopalada's *Dragon Regalia* (CC0, free): the ornate portrait frames.
- andreyin's *Hand cursor* (OpenGameArt, CC0): the menu hand — the owned
  hands were mouse pointers or Code Ark's (rejected).
- unTied Games' *Royal Crown* ($3.49) and Hypnobius's *Dark Ages UI*
  (free): looked at, not chosen.
- Set aside as AI-assisted (memory: AI art only sparingly): Harning's
  *Classic RPG UI Kit* (the hand Nick liked most), heyheythere's kits,
  marcomyly's portrait frames, SiriusForge's *Brass Parley* and *Gilded
  Saga*, Pixel Bruja's *Mystic*, zql01's *Eclipse Atelier*, Catau Games'
  *Fantasy UI Pack*, Gridwright's ornate UI. Lumixy's *Handy Handz* hand:
  its page licence says non-commercial (the artist's comment says
  otherwise); not used.

### Follow-up tickets filed

- **1386** — Decide: a text font for the picture look (today's font the
  frontrunner).
- **1387** — Decide: the stat icons.
- **1388** — Decide: the EXP bar and the volume sliders.

1380, 1381 and 1382 carry the answers (files, slices, fills, sizes, the
two Options entries); 1382 leaves the EXP bar, sliders and stat icons to
1387 and 1388, which add them to it if it is still open.

### The agent's starting rules (Nick can veto; listed in the doc)

- HP colours: green above half, yellow from half to a quarter, red at a
  quarter or less.
- The hand's nudge: 0, 1, 2, 3, 2, 1 screen pixels, about 0.6 s a cycle,
  still while a key is held.
- Sizes at other window scales round to whole screen pixels; at scale 1
  the hand and portrait frames use the font's pixel.
- *Window look* starts on *Pictures* when *Map look* does; a build
  without the bought art shows glyph windows.
- The recoloured map-menu and Options fills are credited as edits.

### Notes

- The mockups were composited over real `frame-png` pictures of a staged
  fight (a throwaway `zz_mock` battle in the private checkout, deleted,
  never committed) by a local script (`target/ui-audit/tools/mock.py` in
  this worktree), never committed (ADR-0040). The script can be reused by
  1386–1388 while the worktree exists.
- Barely Games' EULA forbids use in projects "related to … generative AI
  systems". The game's AI-voiced lines (Qwen3-TTS, Chatterbox) are not
  such a project (the clause is aimed at AI products and training); noted
  for the record.
- No code or skin changed. The private repository's PR #42 only adds to
  `library/`, which no build reads, so `assets-private.rev` stays.
