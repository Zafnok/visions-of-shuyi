# Third-party assets

Every non-crate third-party item shipped with the game (fonts, vendored
JavaScript, images, audio, SDK files) must be listed here, per
[ADR-0013](docs/adr/0013-licensing-and-third-party-policy.md). Rust crates are
checked automatically by `cargo-deny` and listed in the generated
`THIRD_PARTY_LICENSES.html` in release packages.

**The web build's notices come from `shuyi.ron`'s `notices`**: `cargo shuyi
web` puts each file it names in `NOTICES.txt` whole, the license texts and
[`assets/NOTICES-game.txt`](assets/NOTICES-game.txt), which lists the font
and every music track and sound. A new row here that the web build ships
needs an entry there; a test in `xtask`
(`the_notices_name_the_games_works_and_license_texts`) fails without it.

Only licenses allowed by ADR-0013 (amended by ADR-0027 and ADR-0032) are
permitted. **Bought art** (ADR-0032) is listed here too, with the seller, the
licence text as quoted on the store page, the date bought, whether it's
AI-assisted, and *private*: its files and licence text live in the private
assets repository (ADR-0040), never in this public repo. For every other
item the license text must be committed next to the item. The music and sounds are
also listed, with tags, in [`assets/audio/audio.ron`](assets/audio/audio.ron);
the game's credits screen (0808) reads them from there.

**Every row needs a credit with the same source link**: a music or sound
credit in `assets/audio/audio.ron`, or an entry in
[`assets/data/credits.ron`](assets/data/credits.ron) for anything else. The
credits screen shows them all, except entries marked `hidden: true`
(software the player never sees). A test in `trpg-content`
(`every_third_party_asset_has_a_credit`) fails when a row has no credit or
a credit has no row.

**Bought art is credited by file** (ADR-0051). Its credit in
`credits.ron` lists, under `private`, the files of `assets-private/game/`
it covers. `cargo shuyi private-assets --pin` refuses a file there that no
credit covers, and so does the private test the Pages build runs
(`every_bought_file_has_a_credit`). So a ticket that puts new bought files
in the game (0706, 0413, 0437) can't finish without touching the credit:
it adds the files' path to `private`, their pack to the row below, and
their artist to the credit's `author`. The credit shows in a build that
has the files, and not in one without them.

## Bought art (private): purchase record

What was bought is recorded here as a list. What of it is in the game has
a row in the table below, and a credit: **one row and one credit for a
bundle bought as one**, naming each of its packs the game uses. A pack
from another seller gets its own row and credit.

**A bought row** in *Shipped items* is filled in like this (ADR-0032
§5, ADR-0040):

- *Item*: the bundle's name, each pack of it in the game and what is
  used, then **(private)**.
- *Source URL*: the seller's store page.
- *Version*: the version in each download's name, and the date bought.
- *License*: `Custom (<seller>)`, the licence text quoted in full from the
  store page or the pack's licence file, and whether the art is AI-assisted.
- *License file*: the path of the licence text **inside the private
  repository**, written as plain text, not a link (it isn't in this one):
  `private: library/tiny-tales/licences/<pack>__License.txt`.
- *Used for*: where the game shows it, and the folder of
  `assets-private/game/` its files are in.

- **Mega Tiles, "2025 Bundle Sale"** (all 37 Mega Tiles products),
  <https://megatiles.itch.io/>. Bought by Nick on **2026-10-02** for
  **$99.99**. *Private*: the files are never in this repository. They are in
  the private repository `Zafnok/visions-of-shuyi-assets` (ADR-0040):
  sorted in `library/tiny-tales/`, the 37 downloads untouched in
  `originals/tiny-tales/`, every pack's licence text and the generator's
  EULA in `library/tiny-tales/licences/`.
  - **Licence**, the same text in every pack's `License.txt`: "You cannot
    claim ownership of the assets (copyright/IP). Assets can be used both
    in free and commercial games. Assets can be modified freely to fit the
    needs of your game. Redistribution and reselling of the asset files or
    derivatives as is without permission is strictly forbidden."
  - **Character Generator EX 1.2** has its own EULA (Mechwolf Productions):
    the pictures it makes "are owned by the End User and may be used for
    personal, commercial, or non-commercial purposes", but may not be
    distributed or sold "as is" outside a larger project, and the program
    itself may not be redistributed.
  - **Credits named in the packs:** produced and published by Megatiles.
    Portraits and static battlers: Rayane Félix, Lunatic Red. Side-view
    battlers and map sprites: Kodots Games Studio. *Wild Beasts* and other
    battler packs: Lunatic Red, Inazuma. *Epic Monsters* and *Gods and
    Gallants*: Aekashics. Generator software: Mechwolf Productions.
  - **AI-assisted:** no (the store pages say no generative AI was used,
    read 2026-09-30).
  - **What we plan to use:** *Heroes: A New Beginning*, *Heroes 2:
    Rebellious Souls* (faces, still battle pictures, map sprites); still
    battler packs Vol.1–5 with their map sprite packs; *Gods and
    Gallants*, *Epic Monsters*; the Human, Knights, Nobility, Orcs, Dark
    Elves and Beastmen map sprite packs and the Mega Sprite Pack; the World
    Map, Overworld, Dungeons 1 and 2 and Tower tilesets; *Battlebacks
    Vol.1*; the Character Generator. Not planned: the Code Ark sci-fi packs
    and UI kit, *Terrains Plus* and the two *Trees* packs (48-pixel tiles
    in another style).
- **Electric Lemon (Electric Lemon Games), "Pixel Grit" series**: the
  bundle *Fantasy RPG Characters & Tile Set Bundle (Pixel Grit Series)*,
  <https://itch.io/s/148198/fantasy-rpg-characters-tile-set-bundle-pixel-grit-series>
  (Lonesome Forest with its extra trees, Grand Forests, Old Town Exteriors
  and Interiors, Natural Interiors, Dungeons, World Map; Citizens, Guards
  and Warriors; Warriors, Rogues and Mages; Heavy Knight; Creatures
  Extended; High Dragon; the Vampire Lord), and *Fantasy RPG Tileset:
  Stronghold* with its *Ballista and Catapult*,
  <https://electriclemon.itch.io/fantasy-rpg-tileset-fortress> ($4.49).
  Bought by Nick on **2026-11-24** for ticket 0096. *Private*: sorted in
  `library/electric-lemon/`, the downloads untouched in
  `originals/electric-lemon/`, each pack's licence in
  `library/electric-lemon/licences/`.
  - **Licence**, in every pack's licence file: "You may not publicly
    distribute the contents of this zip file in whole or in part, unless
    as part of a digital game or application product." Attribution "is
    NOT necessary" (some packs: "appreciated but NOT necessary"); "Feel
    free to use in any for profit / not for profit product". We credit
    Electric Lemon anyway.
  - **AI-assisted:** no (the store pages say no generative AI was used).
  - **In the game** (row below): Lonesome Forest's cliff kit, recoloured,
    the rim round Chapter 1's hills and peaks (ticket 1359); Dungeons'
    candle in a dish, the light on the desk in Veyne Hall's study (ticket
    1384); the rest are
    candidates for the battle map's tileset (0096).
- **ByteWright, "Everything So Far :)"** (all 33 ByteWright packs),
  <https://itch.io/s/216178/everything-so-far->, <https://bytewrightstudio.itch.io/>.
  Bought by Nick on **2026-11-24** for **$8.59** for ticket 0096.
  *Private*: every download as bought in the private repository's release
  `originals-bytewright` (too big for git); the packs the game can use,
  lean, in `library/bytewright/`; every pack's licence in
  `library/bytewright/licences/`.
  - **Licence**, in every pack: use in any number of games, commercial or
    not; edit, recolour, cut up and combine; "Credit is appreciated but not
    required"; never resell, share or re-upload the assets themselves; no
    NFTs; the newer wording also forbids training AI on them. We credit
    ByteWright anyway.
  - **AI-assisted:** no ("No generative AI was used" on every pack; the
    licence: "hand-crafted pixel art and sound made by us, never produced
    by an AI tool").
  - **In the game:** nothing yet; candidates: Weather Overlays for the
    drifting effects (0096), Ambient Critters (1323), the VFX & SFX mega
    pack and the sound effects pack (1322, 1323).

- **ELV Games, "Game Assets Mega Bundle"** (293 products),
  <https://itch.io/s/79053/game-assets-mega-bundle>, store
  <https://elvgames.itch.io/>. Bought by Nick on **2026-11-24** for
  **$59.99**, for ticket 0096 (its *Rogue Adventure* tilesets were one of
  his two favourite looks for the battle map). *Private*: the files are
  never in this repository. Every download as bought (464 files) is in the
  private repository's release `originals-elv-games`; the packs the game
  may use are sorted in `library/elv-games/`, with every pack's licence in
  `library/elv-games/licences/`.
  - **Licence**, the store's terms (<https://elvgames.itch.io/terms>, last
    updated 2022-07-20), one for every pack: "The Assets may be used for
    commercial or non-commercial purposes." "The Assets can be freely
    edited and modified, make sure it's clearly stated they have been
    modified." "The Assets may not be resold, published, or distributed in
    any way except as an integral part of the project for which they were
    used; this also applies to those that have been edited or modified."
    "The Assets may not be used as a standalone on any kind of physical
    content, including but not limited to printed materials, merchandise,
    or tangible products." "The Assets cannot be used on any kind of Crypto
    or NFT related project." The packs' own `License.txt` adds: "You
    cannot: Use on Training/Generative AI", and asks for "Credits to
    ElvGames" (the sound packs: "Credits to ElvGames & Pegonthetrack").
    **When we edit one of these pictures or sounds, the credits say it was
    modified.**
  - **Credits named in the packs:** art by ElvGames; the sound effect packs
    by ElvGames and Pegonthetrack (<https://linktr.ee/pegonthetrack>).
  - **AI-assisted:** no. 290 of the 292 product pages with downloads say
    "No generative AI was used" (read 2026-11-24); the other two, the
    *Sprite Creator Tool* and the *Retro Character Creator Tool*
    (programs, not used), give no answer.
  - **What we may use** (sorted in the library; nothing chosen yet): the
    *Rogue Adventure* tilesets (every area, the Overworld, the animated
    trees and the winter/autumn jungle), its heroes, monsters, bosses and
    item icons; the 32- and 16-pixel weapon and armor icons; the fire,
    ice, lightning and buff/debuff effects; *Game Sound Effects 1–2*,
    *Magic & Spells 1–2*, *User Interface 1–2*, *Footsteps 1* and
    *Ambient Weather Loops 1*; *Turn Based RPG Monsters* 1–10 and 20–22;
    *Pixel Battlers* 1–11; the *Neatpixels* fonts. In the release only
    (another style or another kind of game): the *Fantasy Dreamland* and
    *Farming Game* lines, the platformer tilesets, *Retro Monsters*,
    *EvoMonsters*, the tower defense, card and domino packs, the UI kits,
    the creator tools.
  - **In the game** (row below): *Rogue Adventure World*'s ground, river,
    jungle, overworld and ruins sheets and two of its animated trees,
    Chapter 1's map (tickets 1318, 1359, 1360, 1356).

- **Barely Games (Barely Apes), two UI kits**: *2D Pixel Quest Vol.3 –
  The UI/GUI*, <https://barely-games.itch.io/2d-pixel-quest-the-uigui>
  (**$9.99**), and *Post-Baroque UI/GUI Kit*,
  <https://barely-games.itch.io/post-baroque-uigui-kit> (**$8.99**, 10%
  off). Bought by Nick on **2026-11-26** for ticket 1379 (the picture
  look's windows). *Private*: the downloads as received in the private
  repository's release `originals-ui-kits-1379`; lean copies and the
  licence in `library/barely-games/`.
  - **Licence**, the store's EULA (<https://barely-games.itch.io/end-user-license-agreement-eula>,
    version 1.0, February 2026): edit and modify; use in personal,
    educational or commercial projects and in finished works, free or
    paid. Not allowed: reselling, sublicensing, sharing or redistributing
    the assets, original or edited; NFT, blockchain or crypto projects; AI
    (training, datasets, generative AI systems). The packs ask, optionally,
    for a mention of "Barely Apes"; we credit them anyway.
  - **AI-assisted:** no ("No generative AI was used" on both pages).
  - **Chosen by 1379** (`docs/design/look-and-feel.md`, *Windows (the
    picture look)*): Pixel Quest's panels, focus bar and HP gauge;
    Post-Baroque's name and title plate. Built by 1380–1382.
  - **In the game** (rows below): Pixel Quest's panels H and E as the
    battle's window frames, H recolored royal blue for the map menu, and
    Post-Baroque's title plate cut to its caps (ticket 1380).
- **unTied Games (Will Tice), *Super Pixel Fantasy UI: Royal Crown***,
  <https://untiedgames.itch.io/super-pixel-fantasy-ui-royal-crown>.
  Bought by Nick on **2026-11-26** for **$3.49** (50% off) for ticket
  1379. *Private*: as above, `library/untied/`.
  - **Licence** (<http://untiedgames.com/files/license.txt>, kept in
    `library/untied/licences/`): commercial and non-commercial use; no
    distribution of the files but inside a game; **attribution to Will
    Tice / unTied Games required**; not to be bundled with a game-making
    tool or engine; no AI training.
  - **AI-assisted:** no ("No generative AI was used").
  - **Not chosen** (round 3's "Mix 2"); a candidate.
- **Free packs fetched for ticket 1379** (2026-11-26; downloaded without
  paying; *private*, with the bought ones, though their licences would
  allow the public repository):
  - tiopalada (Gabriel Lima), *Tiny RPG – Dragon Regalia GUI* v1.0,
    <https://tiopalada.itch.io/tiny-rpg-dragon-regalia-gui>: **CC0 1.0**
    (the page asks, without making it a condition, that it not feed AI
    models or crypto). Not AI-assisted (a 2024 jam entry). **Chosen** for
    the dialogue portraits' frames. `library/tiopalada/`.
  - andreyin, *Hand cursor* (2018-10-21), <https://opengameart.org/content/hand-cursor>:
    **CC0**. **Chosen** for the menu hand. `library/opengameart/`.
  - Hypnobius, *Dark Ages UI* v1.0, <https://hypnobius.itch.io/dark-ages-ui>:
    commercial and personal use, edits allowed, credit optional; no
    reselling, redistributing or packing into other packs; no NFTs,
    blockchain games or AI training datasets. "No generative AI was
    used". Not chosen. `library/hypnobius/`.

## Shipped items

| Item | Source URL | Version | License | License file | Used for | Added by ticket |
| ---- | ---------- | ------- | ------- | ------------ | -------- | --------------- |
| Terminus Font (`ter-u16n`, 8×16, and its bold face `ter-u16b` for key caps), converted to `assets/fonts/atlas.png` and its one-bit copy `assets/fonts/atlas_1bit.bin` (the Playdate) | https://terminus-font.sourceforge.net/ | 4.49.1 | OFL-1.1 | [`assets/fonts/Terminus-LICENSE.txt`](assets/fonts/Terminus-LICENSE.txt) | The game's font for every narrow glyph (Latin, numbers, boxes, symbols); source BDF in `assets-src/fonts/`, unmodified. The atlas adds four glyphs of our own (`✕ ◯ □ △`, `assets-src/fonts/pad-shapes.bdf`, ticket 0220), which the OFL permits for a modified version not named after the font | 0203, 1032, 0254 |
| Terminus Font glyphs, stamped into `assets-test/tilesets/test.png`, `assets-test/tilesets/test_auto.png` and `assets-test/tilesets/test_units/*.png` by `cargo xtask test-tileset` | https://terminus-font.sourceforge.net/ | 4.49.1 (from the font atlas) | OFL-1.1 | [`assets/fonts/Terminus-LICENSE.txt`](assets/fonts/Terminus-LICENSE.txt) | The sprite map skins' generated test tilesets (debug menu only): each terrain's two glyphs on its tile, each class's first two letters on its picture or its unit sheet. The rest of each image is our own data (terrain and palette colours, a drawn figure, lines round terrains) | 0433, 0436, 0437 |
| Cinzel (Natanael Gama, the Cinzel Project Authors), its Bold (700) and Medium (500) instances in `assets-src/fonts/`, baked by `cargo xtask lettering` into `assets/title/chapter_name.png` and `assets/title/chapter_line.png` | https://github.com/NDISCOVER/Cinzel | Google Fonts' `Cinzel[wght].ttf` (https://github.com/google/fonts/tree/main/ofl/cinzel), 2026-11-17 | OFL-1.1 | [`assets/fonts/Cinzel-LICENSE.txt`](assets/fonts/Cinzel-LICENSE.txt) | The chapter title card's lettering (ADR-0128): the chapter's name and the `Chapter 1` line. The static instances were made with fontTools' instancer, which the OFL permits for a modified version (the font declares no Reserved Font Name) | 0812 |
| Fusion Pixel Font 12px (TakWolf; it combines Ark Pixel, Galmuri and Cubic 11), the monospaced Simplified Chinese BDF `fusion-pixel-12px-monospaced-zh_hans.bdf`, its glyphs converted into the wide set `zhhans` of `assets/fonts/atlas.png` and its one-bit copy `assets/fonts/atlas_1bit.bin` | https://github.com/TakWolf/fusion-pixel-font | 2026.11.10 | OFL-1.1 (Cubic 11's notice also carries the M+ and JF Dot fonts' "unlimited permission", commercial use included) | [`assets/fonts/FusionPixel-LICENSE.txt`](assets/fonts/FusionPixel-LICENSE.txt) | Chinese text, each character over two cells (ADR-0045 §4; `docs/design/voices-languages-and-script.md`, *Chinese (Simplified)*). Source BDF in `assets-src/fonts/` with its glyphs unmodified; its `FONTBOUNDINGBOX` is widened to the atlas's 16×16 wide cell (`assets/fonts/README.md`), which the OFL permits for a modified version (the font declares no Reserved Font Name) | 0236 |
| Tiny Tales, from Mega Tiles' "2025 Bundle Sale": the map sprites of *Heroes: A New Beginning* (fighters, witch, archer, thief), *Faith and Evil NPC* (church knight, church cleric), *Elemental NPC* (fire and ice elemental), *Human NPC Knights* (knight) and *Human NPC Advanced* (warrior, fighter, adventurer); the portraits of six heroes of *Heroes: A New Beginning* (male and female fighter, archer, witch, thief; the samurai as a test portrait); the Fox Miko's portrait of *Heroes 2: Rebellious Souls*, the guide of the tips; nine faces made with *Character Generator EX* (its base parts); the tiles of the *World Map* and *Dungeons* tilesets that the battle map's terrain is drawn with, and of the *Overworld* tileset that Chapter 1's places are drawn with (the yard's house, hedge, well and rack; the study's furniture, rug and hanging) **(private)** | https://megatiles.itch.io/ | *Heroes: A New Beginning* 2.0, *Heroes 2: Rebellious Souls* 1.0, *Character Generator EX* 1.2, the four NPC packs 1.0, *World Map* 1.0, *Dungeons* 2.0, *Overworld* 2.0.1; bought 2026-10-02 | Custom (Megatiles): "You cannot claim ownership of the assets (copyright/IP). Assets can be used both in free and commercial games. Assets can be modified freely to fit the needs of your game. Redistribution and reselling of the asset files or derivatives as is without permission is strictly forbidden." Not AI-assisted | private: `library/tiny-tales/licences/<pack>__License.txt`, one per pack, the same text in each | Units on the battle map (`assets-private/game/units/`, named by `tilesets/tiny_tales.ron`); the test portraits of the debug dialogue (`assets-private/game/portraits/test_lord`, `test_knight`); the Chapter 1 cast's dialogue portraits (`assets-private/game/portraits/`: `lead_m`, `lead_f` with her top recoloured blue, `poacher`, `heretic` from the heroes; a Chapter 2 character's from the Thief hero (0858); `retainer`, `sergeant`, `keeper`, `red_captain`, `rival`, `messenger`, `soldier` exported from the generator; two Chapter 3 characters' (0861), their hair, brows, eyes or stubble recoloured, whose EULA, `library/tiny-tales/licences/`, gives the user the generated assets for commercial games); the guide's face in the tip box (`assets-private/game/portraits/guide`, the Fox Miko of *Heroes 2: Rebellious Souls*); the battle map's terrain (`assets-private/game/tilesets/tiny_tales.png`, packed from the three tilesets by `cargo xtask tileset-import`, the burning forest's tiles recoloured; with the places' tiles, `assets/maps/harrowby_yard.map` and `veyne_study.map`); the combat scene's pictures (`assets-private/game/battlers/`: `exile` and `lead_f` the fighters, the second with her top recoloured blue, `poacher` the archer and `archer` the same recoloured for the enemy's, `heretic` the witch, from *Heroes: A New Beginning*; `guard` and `retainer` the church knight recoloured red and blue, `cleric` the church cleric and `mage` the church wizard from *Battlers 5: Faith and Evil*; `fire_elemental` and `frost_elemental` from *Battlers 2: Elemental Forces*) and its backdrops (`assets-private/game/battlebacks/`, two pictures of *Battlebacks Vol.1*), copied by `cargo xtask battler-import`; the title screen's backdrops (`assets-private/game/title/`, seven pictures of *Battlebacks Vol.1*: Fields A at dusk and at night, Fields B by day, at dusk and at night, Meadows at dusk and at night), copied by `cargo xtask title-art --bought`. Credited as Megatiles, with the artists the packs name: Rayane Félix, Lunatic Red, Kodots Games Studio | 0711, 0436, 0437, 0706, 0413, 0811, 0455, 0733 (credited by 0829), 0858, 0861 |
| Shironejiya (白螺子屋) battle pictures: `m_s_bandanna05`, `m_s_bandanna05_b`, `m_s_fighter04_a`, `m_s_hero07` and `m_s_mage_dark_m01` of its *monster_s* set (64×64, hand-made, free), their flat background colour made clear, `m_s_hero07` recoloured (brown hair, the red cloth black), `m_s_mage_dark_m01` too (red hair white) **(private)** | http://hi79.web.fc2.com/ (terms: http://hi79.web.fc2.com/material/contents.html) | The files on the site on 2026-10-11 (`m_s_hero07` on 2026-11-23, `m_s_mage_dark_m01` on 2026-11-24) | Custom (Shironejiya): free; paid games allowed (not adult games), any tool, edits allowed; no reposting or redistribution of the files (screenshots of the game are fine); the game must carry a text with the site's name, its address and "the materials in this game may not be reused"; no use with generative-AI tools (we feed the art to none). Not AI-assisted | private: `library/shironejiya/terms.txt`, the terms page as read on 2026-10-11 | The combat scene's pictures of the Brigand, the Raider, the red captain (`assets-private/game/battlers/brigand`, `raider`, `red_captain`) a Chapter 2 character (0858) and a Chapter 3 one (0861), at 1.25×. The credits screen's entry is the required text; a release's bundled text must carry it too (ticket 0901) | 0413, 0858, 0861 |
| Pixel Flag (ぴくせるふらっぐ商店), *ドット絵TRPGのコマ「クラスとモブ」全64種*: 64 hand-made pixel-art pieces, 64×64 on a clear background; the game uses the Paladin (no. 24), recoloured **(private)** | https://booth.pm/ja/items/7851483 | Version 1 (2026-01-09); bought 2026-11-16, ¥500 | Custom (Pixel Flag): commercial and non-commercial use; edits allowed (「加工、改変は自由にご利用いただけます。」); no redistribution or sale of the material itself, edited copies included, and care that third parties can't reuse it; no credit required. Not AI-assisted ("手打ちドット100％") | private: `library/pixel-flag/terms.txt`, the item page's terms as read on 2026-11-16; `library/pixel-flag/INDEX.md` lists all 64 pieces | The combat scene's riders (`assets-private/game/battlers/rider`, the enemy's in red, and `sergeant`, Tamsin's), at 1.5×, copied by `cargo xtask battler-import` | 0446 |
| MattWalkden, *Fantasy Battle Pack* (version 06-07-25): hand-made pixel-art units on 32×32 cells (lance, sword and bow riders; knights, fighters, an archer, a thief, a wizard; each with idle, move, attack and death in three directions), 16×16 tiles and battle UI pieces; the game uses the red lance and bow riders' move frames, cut to its walking sheet (mirrored for the fourth direction, the drop shadow left out) **(private)** | https://mattwalkden.itch.io/fantasy-battle-pack | 06-07-25; downloaded 2026-11-17, pay what you want | Custom (MattWalkden): "Royalty free commercial and personal license"; the files may be distributed only as a component part of an artistic or commercial work, never alone or in an asset collection; edits allowed for personal and commercial use, never distributed on their own; no credit required. Not AI-assisted ("No generative AI was used") | private: `library/mattwalkden/License and Information.rtf` (the download's licence) and `library/mattwalkden/terms.txt` (its text); `library/mattwalkden/INDEX.md` lists everything in the pack | The mounted classes' map sprites (`assets-private/game/units/lance_cavalier.png` for the Rider, Lancer, High Lancer, Iron Rider and Juggernaut; `mounted_archer.png` for the Outrider and Windrunner), cut by `cargo xtask map-sprite-import` | 0499 |
| ELV Games, *Rogue Adventure World* (the tilesets of the *Game Assets Mega Bundle*): from `RA_Ground_Tiles.png` the grass, the forest floor joined at its corners and the road's dirt; from `RA_Animated_Water.png` the river and its banks, joined at their corners, in its 8 frames; from `RA_Jungle.png` a plank bridge; from `RA_Overworld_Ground.png` the peaks' dark rock, joined at its corners (1359); from `RA_Ruins.png` Harl's keep, its wall tops, pillar caps, faces with arrow slits and paving, a carved stone on its fort tile, and a pillar standing as Harrowby's tower (1360); from `animated-trees/` the green trees `tree01_s_01` and `tree02_s_01`, their 12 frames each, as the forest's swaying trees (1356); from `RA_Village_Animations.png` the fire pit in Harrowby's yard, in its 4 frames (1384). **Modified:** a wall running up and down is the left half of a wall cell and that half mirrored (1360); the half-size trees are those trees' frames at half the size, each two by two pixels made one by `cargo xtask tileset-import` (1356); otherwise as drawn (two corner tiles of a sheet drawn one over the other where a river or a wood meets itself at a point, a pillar's cap over its face) **(private)** | https://elvgames.itch.io/rogue-adventure-world | The bundle's newest *Rogue Adventure World* download; bought 2026-11-24 | Custom (ElvGames): commercial use; edits allowed if the credits say so; no resale or redistribution of the files, edited or not, but inside the game; no merchandise, crypto or NFT use; no generative-AI training; "Credits to ElvGames" (the purchase record above). Not AI-assisted ("No generative AI was used") | private: `library/elv-games/licences/rogue-adventure-world__License.txt`; the store terms in the purchase record | Chapter 1's battle map (its look `rogue_adventure`, `assets-private/game/tilesets/tiny_tales.png`, packed by `cargo xtask tileset-import`); the yard behind the rented house at Harrowby (`harrowby_yard`) | 1318, 1359, 1360, 1356, 1384 |
| Electric Lemon Games, *Lonesome Forest* (the *Pixel Grit* bundle): `Lonesome_Forest_WALLS_and_CLIFF_EDGES.png`, the cliff kit's rim, its outside corners, edges and inside corners joined from quarters of its cells round Chapter 1's hills and peaks. **Modified:** its four stone colours recoloured to a sandy brown by the importer (`recolour` in `assets-src/tilesets/tiny_tales.ron`), and its cells cut into quarters and joined **(private)** | https://itch.io/s/148198/fantasy-rpg-characters-tile-set-bundle-pixel-grit-series | Lonesome Forest 1.2.1, *Summer Tileset*; bought 2026-11-24 | Custom (Electric Lemon): use in any game, for profit or not; edits allowed; no distribution of the files, edited or not, but inside the game; attribution not required (we credit it, and say it was modified). Not AI-assisted | private: `library/electric-lemon/licences/Lonesome_Forest_1.2.1__License.txt` | Chapter 1's battle map (its look `rogue_adventure`, `assets-private/game/tilesets/tiny_tales.png`, packed by `cargo xtask tileset-import`) | 1359 |
| Electric Lemon Games, *Dungeons* (the *Pixel Grit* bundle): `classic_dungeons_animated_candles.png`, a candle in a dish, its first 4 frames of 5; otherwise as drawn **(private)** | https://itch.io/s/148198/fantasy-rpg-characters-tile-set-bundle-pixel-grit-series | The bundle's *Dungeons* download; bought 2026-11-24 | Custom (Electric Lemon): use in any game, for profit or not; no distribution of the files but inside the game; attribution not required (we credit it). Not AI-assisted | private: `library/electric-lemon/licences/Dungeons__License.txt` | The desk in Veyne Hall's study (`veyne_study`, `assets-private/game/tilesets/tiny_tales.png`, packed by `cargo xtask tileset-import`) | 1384 |
| Idylwild's *Chromatic Campfires*: the orange campfire with smoke, its four frames cut from `campfires_PNG.png` into `assets/title/campfire.png` (hand-pixeled, no AI tools, per its page) | https://opengameart.org/content/idylwilds-chromatic-campfires | idylwilds_chromatic_campfires.zip (2025) | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | The title screen's campfire | 0922 |
| Barely Games (Barely Apes), *2D Pixel Quest Vol.3 – The UI/GUI*: the panels `F-UI_Panel_H.png` and `F-UI_Panel_E.png`, drawn in nine slices around the battle's windows. **Modified:** `F-UI_Panel_H.png` copied a second time with its cool pixels recolored royal blue (hue 218°, its gold kept) for the map menu, by `cargo xtask ui-import` (`assets-src/ui/pictures.ron`) **(private)** | https://barely-games.itch.io/2d-pixel-quest-the-uigui | The download of 2026-11-26 | Custom (Barely Games' EULA, version 1.0): use and edits in commercial games; no resale, sharing or redistribution of the files, edited or not, but inside the game; no NFT, crypto or AI use; a mention of "Barely Apes" asked for, optionally (the purchase record above). Not AI-assisted | private: `library/barely-games/licences/barely-apes__eula.txt` | The battle's window frames in the picture look (`assets-private/game/ui/panel.png` the side panel, the forecast, unit info and the battle's other boxes; `menu.png` the unit menu and its sub-menus; `map_menu.png`, recolored, the map menu), named by `assets/ui/frames.ron` | 1380 |
| Barely Games (Barely Apes), *Post-Baroque UI/GUI Kit*: the title plate `Titles/UI_TitleB1.png`. **Modified:** cut to its two 16-pixel end caps and one plain column of its body (column 60), which the game stretches to a title's width, by `cargo xtask ui-import` **(private)** | https://barely-games.itch.io/post-baroque-uigui-kit | The download of 2026-11-26 | Custom (Barely Games' EULA, version 1.0), as the row above. Not AI-assisted | private: `library/barely-games/licences/barely-apes__eula.txt` | The plate behind a window's title in the picture look (`assets-private/game/ui/plate.png`: the forecast's) | 1380 |
| Shuyi Engine's `crates/cli/web/mq_js_bundle.js` (macroquad's JS/WASM loader; copied into the web build by `cargo shuyi web`) | https://github.com/not-fl3/macroquad/blob/5e9b5ca912ac65962c05c0da842a4a70eaae34b9/js/mq_js_bundle.js | 0.4.16 (commit `5e9b5ca9`; no matching git tag, see [its README](https://github.com/Zafnok/shuyi-engine/blob/main/crates/cli/web/README.md)) | MIT | [`mq_js_bundle-LICENSE-MIT.txt`](https://github.com/Zafnok/shuyi-engine/blob/main/crates/cli/web/mq_js_bundle-LICENSE-MIT.txt), in the engine's repository | Loads and runs the WASM binary in the browser build | 0206 |
| Shuyi Engine's `crates/cli/web/sapp_jsutils.js` (JS↔Rust marshalling of strings and bytes the web shell's scripts need; copied into the web build by `cargo shuyi web`) | https://github.com/not-fl3/sapp-jsutils, shipped inside the `sapp-jsutils` crate's `js/sapp_jsutils.js` | 0.1.7 | MIT (no upstream `LICENSE` file; standard MIT text reconstructed from the crate's declared license, see [its README](https://github.com/Zafnok/shuyi-engine/blob/main/crates/cli/web/README.md)) | [`sapp_jsutils-LICENSE-MIT.txt`](https://github.com/Zafnok/shuyi-engine/blob/main/crates/cli/web/sapp_jsutils-LICENSE-MIT.txt), in the engine's repository | Hands strings and bytes to the web shell's scripts (saves, player files, the pack) | 0207 |
| SDL_GameControllerDB (`gamecontrollerdb.txt`), compiled into native builds by the `gilrs` crate | https://github.com/mdqinc/SDL_GameControllerDB, shipped inside the `gilrs` crate (`SDL_GameControllerDB/`) | The copy in `gilrs` 0.11.2 | Zlib | [`crates/app/SDL_GameControllerDB-LICENSE.txt`](crates/app/SDL_GameControllerDB-LICENSE.txt), copied from the crate's `SDL_GameControllerDB/LICENSE` (the `gilrs` crate itself, MIT / Apache-2.0, is covered by `cargo-deny` and `THIRD_PARTY_LICENSES.html`) | Maps PlayStation, Switch and generic controllers to standard button positions (ADR-0034) | 0219 |
| Playdate SDK build support: `C_API/buildsupport/setup.c`, compiled and linked into the Playdate game (`pdex.elf`, `pdex.dll`) by `cargo xtask playdate`; never copied into this repository | https://play.date/dev/ (inside the SDK download) | The copy in Playdate SDK 3.1.2 | 0BSD | [`crates/playdate/setup.c-LICENSE.md`](crates/playdate/setup.c-LICENSE.md), copied from the SDK's `C_API/buildsupport/License.md` | The Playdate game's start-up: hands the SDK's allocator to the C library and calls our `eventHandler` (ADR-0070 §1) | 1031 |
| Music: "New Sunrise" by nene | https://opengameart.org/content/new-sunrise | V1 (New Sunrise.wav) for the title screen; V2 (new_sunrise_V2_0.wav) for a first city visit | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `title`, `city_first_visit` cues | 0214 |
| Music: "Squirrel Village" by SoManyWhales | https://opengameart.org/content/squirrel-village | SV_loop.mp3 (the loop file) from squirrelvillage.zip | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `village_home` cue | 0214 |
| Music: "Aria" by Kistol | https://opengameart.org/content/aria | Aria_2.ogg | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `village` cue | 0214 |
| Music: "Dark Forest Theme" by cynicmusic | https://opengameart.org/content/dark-forest-theme | GameMusic_ForestTheme_24_0.mp3 | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `dungeon_tense` cue | 0214 |
| Music: "Dark Quest" by Alexandr Zhelanov | https://opengameart.org/content/dark-quest | Dark Quest.ogg | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `graveyard_desert` cue | 0214 |
| Music: "Father's Scabbard" by SoManyWhales | https://opengameart.org/content/fathers-scabbard | Father's Scabbard - Loop.wav (the loop file) from fathers_scabbard.zip | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `side_quest` cue | 0214 |
| Music: "Field - Orchestra" by migfus20 | https://opengameart.org/content/field-orchestra | asdf_2.mp3 | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `talk_calm` cue | 0214 |
| Music: "Classical Murder" by Alexandr Zhelanov | https://opengameart.org/content/classical-murder | Classical Murder.ogg | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `talk_antagonist` cue | 0214 |
| Music: "Peractorum (from Emotional Orchestral Music)" by César da Rocha | https://opengameart.org/content/emotional-orchestral-music | Peractorum.wav from EmotionalOrchestra_24bit.zip | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `scene_sad` cue | 0214 |
| Music: "Hesitation (Orchestral Version)" by Maarten Schellekens | https://freemusicarchive.org/music/maarten-schellekens/free-music-made-for-screen/hesitation-orchestral-version/ | The orchestral version (the MP3 the page streams; the download button needs a login) | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `scene_tragic` cue | 0214 |
| Music: "Epic Endgame Cinematic" by cynicmusic | https://opengameart.org/content/epic-endgame-cinematic | ChoirChordsBassMaster.wav, the only version (it has vocals; no instrumental exists) | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `mythic_moment` cue | 0214 |
| Music: "Hope (Orchestral battle music)" by MintoDog | https://opengameart.org/content/hopeorchestral-battle-music | hope_orchestral_battle_music_bpm165.flac (made to loop) | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `battle_bright` cue | 0214 |
| Music: "Determined Pursuit (Epic Orchestra Loop)" by Emma_MA | https://opengameart.org/content/determined-pursuit-epic-orchestra-loop | determined_pursuit_loop.wav (the file is the loop) | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `battle_desperate` cue | 0848 |
| Music: "Battle" by mla | https://opengameart.org/content/battle-0 | battle.flac, whole file (the author's loop point, sample 378000, isn't used) | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `battle_bittersweet` cue | 0214 |
| Music: "Sigil" by Kistol | https://opengameart.org/content/sigil | Sigil_3.ogg | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `battle_easy` cue | 0214 |
| Music: "Ending Scene (orchestral version)" by nene | https://opengameart.org/content/ending-scene | ending_scene_orchestral.wav | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `battle_new_area` cue | 0214 |
| Music: "Battle Theme A" by cynicmusic | https://opengameart.org/content/battle-theme-a | battleThemeA.mp3 | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `battle_epic_a` cue | 0214 |
| Music: "Battle Theme B for RPG" by cynicmusic | https://opengameart.org/content/battle-theme-b-for-rpg | battleThemeB.mp3 | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `battle_epic_b` cue | 0214 |
| Music: "Fantasy Choir 2 and 3" by César da Rocha | https://opengameart.org/content/fantasy-choir-3-orchestral-pieces | Fantasy Choir 2.wav and Fantasy Choir 3.wav from FantasyChoir24bit.zip | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `battle_church_2`, `battle_church_3` cues | 0214 |
| Music: "Battle Themes 1, 2, 3 and 5" by Alexandr Zhelanov | https://opengameart.org/content/battle-themes | Battle Theme 1, 2, 3 and 5.mp3 from Battle Themes.zip (not 4) | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `skirmish` pool | 0214 |
| Music: "RPG - Battle Theme" by jocolloman | https://opengameart.org/content/rpg-battle-theme-0 | battle_theme_2.wav (the uncompressed file, made to loop); copyright 2021 Joseph Collard | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `skirmish` pool | 0214 |
| Music: "Win Jingle" by Fupi | https://opengameart.org/content/win-jingle | 13 of the 17 .wav versions in winjingle.zip (not Saw, Square, Banjo or Harpsichord) | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `victory` pool | 0809 |
| Music: "The Fallen" by Jonathan Shaw (www.jshaw.co.uk) | https://opengameart.org/content/the-fallen-rpg-orchestral-essentials-defeated-music | The Fallen MP3.mp3 from The Fallen MP3_0.zip (RPG Orchestral Essentials), whole, with 12 s of silence added at the end | CC-BY-3.0 | [`assets/audio/licenses/CC-BY-3.0.txt`](assets/audio/licenses/CC-BY-3.0.txt) | `defeat` cue | 0809 |
| Music: "Medieval: The Old Tower Inn" by RandomMind | https://opengameart.org/content/medieval-the-old-tower-inn | The_Old_Tower_Inn.mp3 (the full version, not the loop) | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `camp` pool | 1003 |
| Music: "Woodland Fantasy" by Matthew Pablo | https://opengameart.org/content/woodland-fantasy | Woodland Fantasy_0.mp3 | CC-BY-3.0 | [`assets/audio/licenses/CC-BY-3.0.txt`](assets/audio/licenses/CC-BY-3.0.txt) | `camp` pool | 1003 |
| Music: "A Slave To No One (RPG Orchestral Essentials)" by Jonathan Shaw (www.jshaw.co.uk) | https://opengameart.org/content/a-slave-to-no-one-rpg-orchestral-essentials-boss-music | A Slave To No One MP3.mp3 from A Slave To No One MP3.zip (RPG Orchestral Essentials), its first 4:30, faded out | CC-BY-3.0 | [`assets/audio/licenses/CC-BY-3.0.txt`](assets/audio/licenses/CC-BY-3.0.txt) | `battle_rival` cue | 0850 |
| Music: "Battle Music" by Peter Eastman (peastman) | https://opengameart.org/content/battle-music-0 | Battle_1.ogg, the source's loop (0:04.6 to 1:28.6) | CC-BY-3.0 | [`assets/audio/licenses/CC-BY-3.0.txt`](assets/audio/licenses/CC-BY-3.0.txt) | `battle_night_siege` cue | 0851 |
| Music: "Soliloquy" by Matthew Pablo | https://opengameart.org/content/soliloquy | Soliloquy_1.mp3 | CC-BY-3.0 | [`assets/audio/licenses/CC-BY-3.0.txt`](assets/audio/licenses/CC-BY-3.0.txt) | `scene_confide` cue | 0851 |
| Music: "The World At Your Door (RPG Orchestral Essentials)" by Jonathan Shaw (InspectorJ) | https://opengameart.org/node/108343 | The World At Your Door MP3.mp3 from The World At Your Door MP3.zip | CC-BY-3.0 | [`assets/audio/licenses/CC-BY-3.0.txt`](assets/audio/licenses/CC-BY-3.0.txt) | `world_map` cue | 1007 |
| Music: "Adventures in Adventureland" by Kevin MacLeod | https://incompetech.com/music/royalty-free/index.html?isrc=USUAN2100029 | Adventures in Adventureland.mp3 | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `capital` cue | 1007 |
| Music: "Medieval: The Bard's Tale" by RandomMind | https://opengameart.org/node/80558 | The_Bards_Tale.mp3 (the full version, not the loop) | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `camp` pool | 1003 |
| Sound: "Sword sound 1" by Merrick079 | https://freesound.org/s/568170/ | HQ preview | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `hit_sword` cue | 0214 |
| Sound: "Knife Stab" by Mixedupmoviestuff | https://freesound.org/s/179222/ | HQ preview | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `hit_spear`, `hit_axe` cues | 0214 |
| Sound: "Arrow Impact" by Twisted_Euphoria | https://freesound.org/s/205938/ | HQ preview | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `hit_bow` cue | 0214 |
| Sound: "Punch" by EminYILDIRIM | https://freesound.org/s/544680/ | HQ preview | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `hit_gauntlet` cue | 0214 |
| Sound: "Deep Cut / Slash / Gash" by SypherZent | https://freesound.org/s/420674/ | HQ preview | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `crit_physical` cue | 0214 |
| Sound: "Combat Punch Metal Armor" by EminYILDIRIM | https://freesound.org/s/545021/ | HQ preview | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `block` cue | 0214 |
| Sound: "Short-Fireball-Woosh" by wjl | https://freesound.org/s/267887/ | HQ preview | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `cast_fire` cue | 0214 |
| Sound: "Hard Glass Impact" by deleted freesound user 3656686 | https://freesound.org/s/418194/ | HQ preview; the uploader's account was deleted | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `cast_ice` cue | 0214 |
| Sound: "Magic Earth Spell Impact & Punch" by EminYILDIRIM | https://freesound.org/s/541477/ | HQ preview | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `hit_magic` cue | 0214 |
| Sound: "Fireball Impact" by EminYILDIRIM | https://freesound.org/s/577450/ | HQ preview, untrimmed | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `crit_fire` cue | 0214 |
| Sound: "Magic Ice Impact Skill Spell" by EminYILDIRIM | https://freesound.org/s/550267/ | HQ preview, untrimmed | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `crit_ice` cue | 0214 |
| Sound: "Footstep_Dirt_00" by LittleRobotSoundFactory | https://freesound.org/s/270415/ | HQ preview | CC-BY-4.0 | [`assets/audio/licenses/CC-BY-4.0.txt`](assets/audio/licenses/CC-BY-4.0.txt) | `step_foot` cue | 0214 |
| Sound: "dirt/gravel footstep 4" by Yoyodaman234 | https://freesound.org/s/223153/ | HQ preview | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `step_foot` cue | 0214 |
| Sound: "Footstep_Grass_5" by GiocoSound | https://freesound.org/s/421135/ | HQ preview | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `step_foot` cue | 0214 |
| Sound: "Knight Right Footstep on Gravel 5 (With Chainmail)" by Ali_6868 | https://freesound.org/s/384890/ | HQ preview | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `step_armored` cue | 0214 |
| Sound: "Maxheadroom's Galloping Horse" by Podsburgh | https://freesound.org/s/274898/ | HQ preview; one hoof-beat pair cut from 0.895-1.035 s | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `step_mounted` cue | 0214 |
| Sound: "Campfire 02" by HECKFRICKER | https://freesound.org/s/729396/ | HQ preview; 40 s from 1:00, compressed and cut to loop (`assets-src/audio/import.py`) | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `title_campfire` cue | 0922 |
| Sound: "Summer Meadow" by baryy | https://freesound.org/s/409143/ | HQ preview; 40 s from 0:20, mono, levelled like the music and cut to loop (`assets-src/audio/import.py`) | CC0-1.0 | [`assets/audio/licenses/CC0-1.0.txt`](assets/audio/licenses/CC0-1.0.txt) | `bed_summer_meadow` cue: the day bed of the places' ambience | 1332 |
| Qwen3-TTS (code `022e286`; weights: Base `fd4b25438912`, VoiceDesign `5ecdb67327fd`, Tokenizer `7dd38ad4`), its output only: **AI-generated** voice clips **(private)** | https://github.com/QwenLM/Qwen3-TTS, https://huggingface.co/Qwen/Qwen3-TTS-12Hz-1.7B-Base | code `022e286` | Apache-2.0 (code and weights): "perpetual, worldwide, non-exclusive, no-charge, royalty-free"; claims nothing over generated output | [`assets-src/audio/voice-refs/Qwen3-TTS-LICENSE.txt`](assets-src/audio/voice-refs/Qwen3-TTS-LICENSE.txt) | Designed every voice (the reference clips in `assets-src/audio/voice-refs/`) and reads most characters' lines: the voice clips of the private repository's `voice/` (Chapter 1), shipped beside the game, never embedded (ADR-0046). The tool itself doesn't ship. Credited only in a build that has clips | 0043, 0721, 0722 |
| Chatterbox Multilingual V3 (code `5de7a54`; weights `5bb1f6ee58e5`), its output only: **AI-generated** voice clips **(private)** | https://github.com/resemble-ai/chatterbox, https://huggingface.co/ResembleAI/chatterbox | code `5de7a54` | MIT (code and weights): "to deal in the Software without restriction"; claims nothing over generated output | [`assets-src/audio/voice-refs/Chatterbox-LICENSE.txt`](assets-src/audio/voice-refs/Chatterbox-LICENSE.txt) | Reads the retainer and the poacher, and converts every directed line to its voice (ADR-0109): voice clips of the private repository's `voice/`, as above. Its clips carry an inaudible watermark, which the license doesn't require and we don't remove. The tool itself doesn't ship. Credited only in a build that has clips | 0043, 0721, 0722 |

## AI voice tools

Chosen in ticket 0043 ([ADR-0107](docs/adr/0107-free-local-voice-tools-with-designed-voices-and-a-reader-per-character.md)).
They run on a developer's machine, and neither tool ships; what they generate
does, from ticket 0722 (Chapter 1), so each has a row in the table above and a
credit, shown only when the game has clips. The reference clips in
`assets-src/audio/voice-refs/` are development inputs. No voice is cloned from a
real person (ADR-0046 rule 8). Terms read 2026-11-01.

## Platform SDKs (not shipped)

SDKs a build is made with but that are not in the game or in this
repository. They need no row in the table above and no credit: nothing of
them ships. Their licenses still bind how the game is built and sold
(ADR-0013, *Platform SDKs*).

- **Playdate SDK** (Panic Inc.), 3.1.2, https://play.date/dev/, under
  Panic's SDK license (`SDK_LICENSE.md` in the SDK): free and
  royalty-free, games made with it are ours to sell anywhere; the SDK may
  not be redistributed, decompiled or used for other platforms, and
  "Playdate" or "Panic" may not be in the game's name without permission
  (ADR-0070). Installed by hand on a developer's machine and downloaded
  from Panic by CI's `playdate` job, never committed. The game calls the
  Playdate's system through its headers (`crates/playdate/shim/shim.c`);
  the only SDK file linked into the game is `setup.c` (0BSD, in the table
  above). Added by ticket 1031.
