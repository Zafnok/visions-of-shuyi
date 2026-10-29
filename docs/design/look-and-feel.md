# Look and feel

Decided: 2026-09-25
Source: ticket 0011 (attack forecast: 0404; bought portraits and battle art:
0021; bought tiles and sprites on the battle map: 0038; walking sprites,
effect marks, the glyph look as an option, busts in dialogue: 0039; which
bought tiles each terrain is: 0437; threat arrows: 0055; the title
screen: 0811; descriptions in lists: 0429; places behind conversations:
0029; arrows at the map's edges: 0449; Act 1's new cast: 0070)

Nick judged real renders, not descriptions. Every mockup was drawn with the
game's own font atlas at in-game size. The final ones are in
`docs/screenshots/0011-*.png`:

| Screenshot | Shows |
| ---------- | ----- |
| `0011-battle-browse.png` | Battle screen, palette D, browsing: initials, HP bars (its bracket cursor was replaced by corner marks in 0416) |
| `0011-battle-selected.png` | A unit selected: move/attack ranges, path line with arrowhead |
| `0011-conversation.png` | Conversation screen with 32×32 shaded portraits (the portrait style was replaced by bought art in 0021; the layout stands) |
| `0011-portrait-expressions.png` | Old portrait style sample: confident, happy, angry, sad (replaced by bought art, 0021) |
| `0011-theme-c/d/e/g.png` | The four palettes offered as colour themes |
| `0404-forecast.png` | The attack forecast (decided 2026-09-27, ticket 0404) |

Names, stats, weapons and the sample face in the screenshots are placeholders.

## Nick's words

> **Colour mood** (first round A–C, then D–G): "A and C look better than B, but
> all three aren't ideal to me. Can we get some more options?" … "how about we
> just offer the choice of C, D, E, and G for the color settings. But for now
> if we want to nail the look and feel for one before doing scope creep D is
> the standout for me."
>
> **Units on the map:** "I like C [name initials] the best but I am wondering if
> we can get some more glyphs to represent rather than simply letters" … "for
> the unit symbol options I am still not impressed. […] if I had to pick the name
> initials themselves are still the best. Maybe if the whole UI is rendered and
> it shows more info when you hover a unit somewhere else on the screen then it
> would be more acceptable." … "I think backing is maybe where we can show the
> HP without hovering. Like a green/yellow/red bar showing % hp by fill." … "I
> think the thin bar works but I don't really care about faction backing."
>
> **Cursor:** "I like B [blinking brackets] the best" … "for the cursor I think B
> in general looks good and then maybe on selection it changes to E [arrows] to
> make it clear when you're in selected mode"
>
> [2026-09-27, ticket 0416, after playtesting the Quick Battle:] "the cursor
> overlaps units' names when adjacent. Like if Ar is on tile 1x1 and cursor
> [ ] is on tile 1x2 then it just says A [ ] the r in Ar is cut off. […] Make
> the cursor skinnier or something. I think z-indexing won't help here it
> would just clutter it" [shown thin brackets, corner marks and a tile glow:]
> "corner marks do look good but I wonder if we can make them more square than
> they are now -- they are more wide than tall" [shown 2×2, 3×3 and 4×4
> corners:] "3x3 default with 4x4 and the full tile glow as accessibility
> options"
>
> **Path:** "I would expect a path trace when you start to move from the
> selected unit, but I do like the move range and attack range colors" … "the
> start of path should either be Z-indexed under the character, or just start
> on the next adjacent tile since right now it obscures the unit name.
> Additionally, while the arrows work for selecting while ON a unit, once you
> start moving away it seems it makes it unclear what tile you end up at. So
> maybe the end of the traced path should be a rendered arrow (singular)."
>
> **Portraits:** "I think the line art isn't good." [On the shaded-blocks
> sample:] "He should be much more expressive and human looking" [on the
> redraw:] "this looks a lot better all around… if there is a way to give
> subtle shading to give more volume to the face it would be good" … "this new
> shaded style seems OK it does make this particular character look a bit old
> (i.e. like he has wrinkles) but the general theme is ok. And we can probably
> iterate on a character by character basis."
>
> **Where portraits appear:** "it might be nice to have portraits appear in
> hover/selection in battles. But combat screen probably we would have more like
> a full body rendering of the 2 battling units. But if we can have them display
> in hover/selection we have to make sure they won't clutter the UI" … [after
> seeing the 16×16 panel mini-portrait:] "that one looks like a memeface so we
> can just forego the battle portrait"
>
> **Bought character art** (2026-09-28 to 2026-09-30, ticket 0021): [on
> Claude's portraits and CaptainSkolot's bundle:] "I like the style better
> than your style" … "it's fantasy not modern day which fits the game" …
> "buying is OK as long as it's not super expensive" … "I'll buy it later
> when we finish more stuff" … [on the combat screen:] "I guess we need an
> itch artist who has a pack with portraits and battle sprites" … [shown
> mockups of five artist options:] "I like A2 the best but want to ensure you
> know how to route the animations correctly" … "AI assisted art is ok as
> long as it is not shipped without a human touch at all... the captain guy
> said he touched it up, that's alright" … [asked for a price ceiling:] "stop
> asking, I will decide each time if I am comfortable" … [told that the big
> Tiny Tales fighters are still images and only the small ones animate:] "1A
> seems best" [the big still images, moved by the game] … [characters no
> face fits:] "probably A or D" [the Character Generator, or Claude's small
> edits] … [classes with no animated fighter:] "since I am ok with 1A then 3A
> seems fine?" [still images as stand-ins] … [named characters without
> their own combat picture use their class's picture, recoloured:] "sure" …
> [Rue: the Witch's face and combat picture, or a generated face and a class
> picture:] "I think we can rewrite Rue's physical description as needed once
> we buy the art... I would rather have visual consistency at this time
> between portrait and battler" … [on commissions:] "this is where I veto --
> this would be expensive for just a single asset" … [Harl:] "let's just have
> a follow up for this Harl character I can shop around and find more
> contenders"
>
> **Bought sprites on the battle map** (2026-10-02, ticket 0038; Nick had
> bought the whole Mega Tiles bundle and judged spike renders of the battle
> screen: A today's glyphs, B glyph terrain with bought unit sprites, C the
> bought tileset with bought unit sprites, C2 the same at double size, C3
> the same darkened toward palette D, G1a/G1b a combat picture with a bought
> background in the box over the map and full screen): "I'm really digging
> these sprites... I think we can go ahead and move forward with replacing
> our tile rendering and battler rendering with these bought sprites without
> regret..." … "as for how the tilesets look, I'm between C, C2, and C3. But
> I think this actually has an easy answer. 1. Per-map lighting effects
> (i.e. if it takes place during what would be a low light time, it can be
> shaded like with C3. Or if it's noon, it can be like C/C2. 2. A key/button
> to toggle zoom (1x / 2x) with memory (i.e. going to next battle, keep the
> setting)" … "for G1a and G1b I think both are a little off. The main thing
> that's off though is that you put the enemy facing backwards... let's
> always mirror the image to face the player when battling. Other than
> that, I worry G1b won't be able to convey all the info, but maybe it can
> work with some additional in-battle overlays or pop up messages" … "for
> the cast, I guess you need to help me locate some more itch bundles to
> fill in with a similar style for rest of cast... or if these are
> non-battle units, I think the generator can work" … "I paid $100 for all
> these assets" … [asked how a sprite unit shows its side and that it has
> acted: A a coloured outline, B a tinted tile, C a corner mark, D the HP
> bar only:] "A, and potentially A with C" … [asked which key and button
> toggle the zoom:] "I think we can ask this as we build the zoom feature
> in another ticket? I will decide it later" … [asked whether the Chapter 1
> playtest waits for the bought-art map: A yes, B no:] "A"
>
> **Sprite units and dialogue busts** (2026-10-02, ticket 0039; Nick judged
> animated mockups J1–J3 of sprites standing still, walking only when
> moved, and stepping on the spot as well; K, four ways to mark a unit
> under an effect; H2/H3, faces and busts in the dialogue screen): [do
> sprites walk:] "1C - but make sure the hp bar is not overlapping the
> sprite... right now it is" … [the effect mark, shown as a purple square:]
> "what does under an effect mean here? like poison or something? can you
> show what a poison mark might look like then rather than a square" …
> [faces or busts:] "can you see if bust can be zoomed in more, it seems
> like there's empty space on top and sides makes me think bust 4x might be
> possible..." … [shown J4/J5 with the feet on the HP bar, K2 with an up
> arrow for a bonus and a down arrow for a penalty in the tile's corner or
> over the head, H4/H5 with busts at 4×:] "1A [the arrow in the tile's
> corner] but I want the mark raised one more pixel up... maybe 2" … "2C"
> [busts at 4×] … [may players pick the glyph look:] "if both exist I guess
> we can add the option and we already have the decoupled art/engine logic
> afaik" … "for walking with HP bar, can you raise the sprite 1-2 more
> pixels?" … [shown sheet L, the sprite raised 0, 1 or 2 more pixels and
> the arrow raised 1 or 2:] "let me see sprite up 1 mark up 2 vs sprite up
> 0 mark up 1" … [shown the two side by side, L2:] "sprite 0 mark 1 looks
> best to me" … [does the Chapter 1 playtest wait for the walking: A no, B
> yes:] "B" … [told that an acted unit standing still was Claude's
> addition:] "sure, unit can stand still if exhausted" … [on the arrows:]
> "I was thinking the mark itself could be animated a bit too, like
> bouncing up and down for up/down arrows, and maybe changing between
> colors for poison, etc" … "for the combined up/down arrow though, I
> don't know how I feel about it, maybe it should be a swirl? or alternate
> between the marks" … [shown K4, animated: both arrows stacked, the two
> taking turns, a turning swirl:] "I think B [taking turns] looks best...
> I am still seeing quite a bit of clipping with the unit when they are
> vertically adjacent and in the walking animation with the glow it clips
> heavily into the other units' health bar and feet... can we have a clip
> mask to shave their head so it doesn't clip like that?" … [shown M and
> M2: A shaved only under another unit, B always kept inside its own
> tile:] "A is best" … "can we also adjust hp bar to be 1 less pixel left
> and right? this would be always active" … [asked whether that goes for
> the glyph look too:] "I think glyph look should also have 14px hp bar
> otherwise rest is good"

Arrows at the map's edges (ticket 0449, 2026-10-21; Nick had asked on
2026-10-12: "when the map overflows the viewport, there should be little
arrows that show you can move the map, and I guess it would just happen
when you move the cursor past the viewport it moves"; shown Chapter 1's
first turn at 2× and the view scrolled to the fort, with A a small arrow
in the middle of each edge the map goes on past, A2 a bigger one, B the
arrow fading in and out, C a thin bright line along the edge):

> [which marks:] "can we try flashing big?" … [shown B2, the big arrow
> fading smoothly, and B3, blinking fully on and off:] "B2 smooth fade"
> … [when they show:] "Only while browsing" … [whether the view already
> scrolling once the cursor is 3 tiles from the edge is what he meant:]
> "we might revisit the threshold for when it moves but that should be a
> followup / 00 ticket rather than in this ticket" … [the glyph look's
> arrow:] "Cursor yellow"

Descriptions in lists (ticket 0429, 2026-10-14; shown four renders of the
knight's `Skill` menu and the lord's `Arts` menu: A a strip inside the menu
box, B a window under the list with the name in its border, C the help bar,
D the side panel):

> "A but with the more pronounced coloration of B. I don't like repeating
> the skill name but white is easier to read" … [asked whether the cursor
> should stop on greyed-out lines so they can be read:] "Stop on them"

## Rules

### Colour

- **The game's palette is mood D, "Earthy painterly":** browns, olives and
  brick reds on a near-black base, and **every terrain has its own background
  tint** (olive grass, dark-green forest, deep-teal water, brown rock). The
  values are in `assets/data/palette.ron` *(tunable)*.
- Terrain draws its glyphs in `<terrain>` and its background in
  `<terrain>_bg` (e.g. `forest` / `forest_bg`).
- **Fort glyph is `╦╦` (battlements)**, in `fort` gold (decided 2026-09-28,
  ticket 0422). Nick: "not a fan of the fort glyph it looks too similar to a
  cursor" … "I like C for the fort". The old `[]` read as the cursor's
  brackets; terrain glyphs should never look like the cursor.
- **House, gate and throne are each one picture across the tile's two
  cells** (decided 2026-10-22, ticket 0037): our own glyphs (a lean-to
  shack, a portcullis, a crowned throne), not two of the same letter,
  which Nick found "visually confusing". Details in `terrain.md`,
  *Building tiles*.
- Move/attack/heal/danger overlays blend their colour over the terrain
  background at about **75%** *(tunable)*, so the ground stays visible
  underneath. Nick liked the move and attack range colours as shown.
  **The same 75% on the bought tiles** (decided 2026-10-04, ticket 0437:
  Nick was shown a selected unit's ranges over the picture tiles at 75%,
  50% and 35%: "75% looks ok").
- **Colour themes** (ticket 0806): Options has a **Colors** row with four
  palettes, **C Rich & painterly, D Earthy painterly (default), E War-table
  parchment (light), G Moonlit**. D is `assets/data/palette.ron`, the others
  `assets/data/palettes/{painterly,parchment,moonlit}.ron`. A theme recolours
  glyphs and the UI, never a picture. Their mockup values are recorded below;
  the colours the mockups didn't list were derived to match each theme's feel
  *(tunable)*. In E, `black` is the light console background and `white` is
  the strongest ink.

### Battle map: bought tiles and unit sprites

Decided 2026-10-02, ticket 0038. The renders Nick judged used bought files,
so they aren't in this repository (ADR-0032).

- **The battle map is drawn with the bought art**: terrain from the Tiny
  Tales 16×16-pixel tilesets, and each unit as its Tiny Tales **map sprite**
  (16 pixels wide, 20 tall, so a head overlaps the tile above). The art is
  the same size as our map tile (16×16 pixels), so nothing is scaled.
  Tickets 0436 (units) and 0437 (terrain).
- **The glyph look below stays in the game** (ADR-0038): it is what a copy
  of the public repository shows, since the bought files aren't in it.
- **Players may pick the look** (decided 2026-10-02, ticket 0039): an
  Options entry, **"Map look: Pictures / Glyphs"**, like Dwarf Fortress on
  Steam (new art or classic ASCII). Nick: "if both exist I guess we can
  add the option". Pictures is the default. *(Claude's starting rules: the
  entry switches terrain and units together; it changes the map at once
  and is remembered; a build without the bought files has only glyphs and
  doesn't show the entry.)* Ticket 0824.
- **Lighting is set per map.** A battle at noon shows the tiles as bought
  (render C). A battle in low light (dusk, night, indoors by torchlight)
  shows them darkened toward our earthy palette, as in render C3. Unit
  sprites keep their own colours in every light, as they did in C3
  *(Claude's starting rule: that is how C3 was drawn; Nick didn't comment
  on it)*. Built in ticket 0438: the lights are **noon** (the tiles as
  bought), **dusk** (render C3's numbers) and **night** *(Claude's
  starting values: darker and bluer than dusk)*, each a line of
  `assets/data/lighting.ron` *(all tunable; more lights are more lines)*.
  A map file names its light and a battle file may name another, so one
  map can be fought at two times of day. *Claude's starting rules:* the
  range colours, the path, the cursor and the HP bars aren't shaded
  either; the glyph map looks the same in every light. Which map is in
  which light: every map is at noon until Nick says otherwise (Chapter 1's
  battle is in the morning; the story doesn't call it low light).
- **Zoom:** a key and a controller button switch the map between **1×**
  (a 16-pixel tile, 35×30 tiles on screen, render C) and **2×** (a
  32-pixel tile, 17×15 tiles, render C2). Tiles, unit sprites, HP bars,
  effect arrows, the cursor's marks and the path all grow together; the
  side panel, menus and text don't. The game **remembers** the choice:
  the next battle, and the next launch, open at the same zoom. Built by
  ticket 0439.
  - **A new player starts at 2×** (Nick, 2026-10-12, shown Chapter 1's
    first turn at both, [`0439-zoom-1x.png`](../screenshots/0439-zoom-1x.png)
    and [`0439-zoom-2x.png`](../screenshots/0439-zoom-2x.png): "2x"). In
    the 0804 playtest he had found 1× too small: "I think we can scale
    the battle map bigger. tile size / overworld unit size. rn it's quite
    small". The cost he was told: Chapter 1's map is 24×16, so at 2× it
    scrolls (the fort is off screen at the start).
  - **The key and the button** (Nick, 2026-10-12): `Q` right-handed, `P`
    left-handed, the right trigger on a controller; optional. His words
    and when the key works: `controls.md`, *Zoom map*. Also a row of the
    Options screen, `Map zoom` (`options.md`).
  - **Only pictures zoom.** The glyph look's tile is two font cells: with
    it the key does nothing and the help doesn't list it.
- **Arrows at the map's edges** (decided 2026-10-21, ticket 0449; render
  [`0449-map-edge-arrows.png`](../screenshots/0449-map-edge-arrows.png)):
  where the map goes on past the view, a **big arrow** (a triangle 6 px
  deep and 11 px across, in the cursor's colour with a 1 px dark edge;
  twice that at 2×) on the **middle tile of that edge**, pointing out,
  1 px in from the edge and above the HP bar. It **fades smoothly** from
  full to 35% and back once a second *(both tunable)*.
  - **Only while browsing:** shown while the cursor moves freely (browsing,
    a unit selected, choosing where to move after an attack); hidden during
    the enemy's turn, a unit's walk, menus, target picking and a fight.
  - It sits over the units and under the cursor: a unit on that tile is
    partly covered (Nick was told).
  - A map that fits the view has none. The view still scrolls once the
    cursor comes within 3 tiles of its edge; when it starts moving may be
    revisited (ticket 0072).
- **A sprite unit's side** (decided 2026-10-02): a **1-pixel outline in
  its side's colour** (player blue, enemy red, ally green, neutral yellow)
  around the sprite, as in renders B, C, C2 and C3. **The outline alone,
  no corner mark** (decided 2026-10-03, ticket 0436: Nick was shown the
  game's own frames with the outline alone and with a 3×3 corner mark in
  the tile's top-left, and picked "Outline alone").
  **The player's outline is a deeper blue** than the side's blue used
  elsewhere (`player_outline`, `#2a60ff`; decided 2026-10-13, ticket 0447).
  Nick, playing Chapter 1: "the blue outline is hard to see for
  friendlies, the red is very clear." Shown the game's frames with a
  deeper blue, cyan, a 2-pixel outline, a dark edge and mixes of them, he
  picked the deeper blue at 1 pixel: "A1 looks good". The other sides'
  outlines, the width, and the blue of names and of the glyph look are
  unchanged. It is the same in every color theme, as the tiles are.
- **An acted sprite unit** is drawn grey and darker, as in the renders;
  its outline dims with it. A change of brightness, not of hue, like the
  glyph look's rule. Each pixel goes 75% of the way to its own grey and is
  then three quarters as bright (*tunable*). The renders used 0.6 as
  bright; seeing it in the game on the dark glyph ground, Nick chose
  "Lighten a bit now" (2026-10-03, ticket 0436).
- **Which sprite each unit is** (ticket 0436; Claude's starting picks from
  the spike, *for Nick to change*: say which and they are swapped in
  `cargo xtask map-sprite-import`'s table). All are Tiny Tales map
  sprites:

  | Unit | Sprite |
  | ---- | ------ |
  | The lead | Heroes: Fighter (male) or Fighter (female), by the gender picked |
  | Exile (the placeholder lord's class) | Heroes: Fighter (male) |
  | Mage | Heroes: Witch |
  | Archer (ours and the enemy's) | Heroes: Archer |
  | Guard | *Faith and Evil*: Church Knight |
  | Cleric | *Faith and Evil*: Church Cleric |
  | Rider | Human Knights: Knight M1, **on foot**: nothing mounted exists in the bundle (ticket 0040 looks for mounted art) |
  | Brigand | Human NPC Advanced: Warrior M1 |
  | Raider | Human NPC Advanced: Fighter M1 |
  | Fire Elemental, Frost Elemental | *Elemental Forces*: Fire Elemental, Ice Elemental |
  | Any other class | Human NPC Advanced: Adventurer M1 |

  A sprite hangs off a class or a named character, not a side: the
  outline says whose a unit is. So an enemy Archer looks like ours (the
  spike gave enemy archers a Rogue sprite; that needs a sprite per side,
  which the tileset file doesn't have).
- **Which bought tiles each terrain is** (ticket 0437). The sets are
  Nick's decision of 2026-10-02; the picks within them are Claude's
  starting picks, *for Nick to change*: say which and they are swapped in
  `assets-src/tilesets/tiny_tales.ron`. A map outdoors is the World Map
  set in its Standard colours:

  | Terrain | Drawn as |
  | ------- | -------- |
  | Plain | The green grass ("Grass") |
  | Road | The tan ground ("Terrains"), with its own edge against the grass |
  | Forest | The small green trees, as a mass (Trees 1) |
  | Thicket | The dark grey-blue pines, as a mass (Trees 6) |
  | Mountain | The brown mountains ("Grasslands Mountain") |
  | Peak | The grey-purple mountains ("Ruin Mountains") |
  | Water | Shallow water, with the set's shore: sand, a wet band and foam |
  | Sea | Deep water; next to land it has a strip of shallows |
  | Bridge | The wooden bridge, turned to cross the water |
  | Fort | The grey stone tower: the only stronghold in the set that is one tile big |
  | Ice | The snow ground ("Snow"), laid on the water |
  | Burnt | Bare grey trees (Trees 3) on dark ground ("Grounds") |
  | Burning | **A stand-in.** Nothing in the bundle burns: the forest's trees, recoloured in our `fire` colour (each pixel as bright as it was) |
  | Floor, Wall, Door | The Dungeons 1 "Fortress" set: blue-grey slabs, tan brick, a wooden door |
  | House | **A stand-in** (ticket 0322, until 0471): the World Map set's small single-tile house, on the grass; the bundle's houses are "too modern/nice" (Nick) |
  | Gate | **A stand-in** (0322, until 0471): the Fortress set's open archway, on the grass ("serviceable", Nick) |
  | Throne | **A stand-in** (0322, until 0471): the bundle has no throne, so a cushioned chair (Overworld Interiors), on the floor's slabs |

  A map **indoors** (its file says so: `look: (tiles: "indoor")`) is the
  Fortress set: slabs for the floor, brick for the walls, the door, and
  water as a dark pool with a stone rim. Terrains that set lacks (grass,
  trees, mountains) keep their outdoor pictures.
- **Tiles are joined from the terrain alone** (ticket 0437, ADR-0052): a
  shore, a road's edge and the outline of a wood are worked out from which
  tiles are which, and end exactly where their tiles end, so what is seen
  is what the rules count.
- **A wood stops inside its own tiles** (decided 2026-10-14, ticket 0452,
  after Chapter 1 playtest round 2). Nick saw the trees spill half a
  tile onto the road and the grass, and chose from six renders (today's,
  one tree per tile, one tree per tile on a dark floor, two other tree
  masses, and the trees cut off at the forest's tiles). His words: "C and
  D look better but the thing is D is clipped too aggressively, it would
  be better if the forest, rather than A and C overflowing, it
  underflowed, so just take the half trees out of D and it will still
  occupy the tiles it needs to without visually confusing anybody."
  So the trees are drawn only on the forest's tiles, and the wood's edge
  is a row of whole trees a few pixels inside them: no tree is cut in
  half and none stands on the road or the grass beside it. A forest tile
  alone is a small clump of whole trees in the middle of its tile.
  Thicket, burnt and burning trees are drawn the same way (*Claude's
  starting rule*: the same problem, the same art layout; say if they
  should differ). Where a wood turns a corner inward (forest above and
  beside a tile but not on the diagonal), the trees run on to the corner
  of the tile (*Claude's starting rule*).
- **What the bought-art maps don't have** (each its own ticket if Nick
  wants it): the bundle's castles, towns and houses bigger than one tile;
  moving water and waterfalls (one still frame is used); hand-placed
  decoration (rocks in the sea, single trees).
- **The Chapter 1 playtest waits for the bought-art map** (decided
  2026-10-02): units (0436) and terrain (0437) are in before Nick plays,
  and so is the walking below (0440; Nick, ticket 0039: "B").
- **Sprites step on the spot and walk** (decided 2026-10-02, ticket 0039,
  "1C"; like Fire Emblem on the GBA). Each map sprite has three walking
  frames in four directions:
  - A unit that **can still act steps on the spot**, facing the camera
    (about four steps a second in the mockup, *tunable*).
  - A unit that **has acted stands still** (and is grey). So "still" also
    says "done" (Nick: "sure, unit can stand still if exhausted").
  - A **moving unit walks along its path**, gliding from tile to tile with
    its legs going, and **turns to face the way it walks** (about a fifth
    of a second per tile in the mockup, *tunable*; the speed settings
    still apply). When it arrives it faces the camera again.
  Built in ticket 0440. The values (*tunable*, all in
  `crates/ui/src/screens/battle/walk.rs`): a step on the spot every
  250 ms; a walking sprite changes frame every 100 ms.
  - **How fast a sprite walks** (Nick, 2026-10-04, after seeing 5, 8, 10
    and 12 tiles a second side by side; 5 was "still too slow"):
    **6** tiles a second normally; **8** with the fast setting in
    Options (ticket 0805); **12** while the speed-up button is held.
    The two don't stack: "it's just 12 max". *(Claude's reading: the
    speed-up button is holding Confirm, which speeds up the enemy's
    phase, `controls.md`; holding Confirm during your own unit's walk
    still skips it, as before.)*
  - **The glyph look keeps its own walking speed**, 12 tiles a second
    (Nick, 2026-10-04: "original glyph version I think should have its
    original walking speed"). Glyph units jump from tile to tile.
  - **The clip mask always holds** (Nick, 2026-10-04; decided in 0039 and
    0436): no sprite or outline is drawn inside a tile another unit
    stands on above it, walking or standing, not even the 1-pixel edge
    of an outline beside it. A unit below a walker passing over is cut
    for as long as the walker is on the tile above it.
  - On the glyph ground (the retro look with sprite units), a walking
    unit walks over the ground's glyphs: the tile it leaves shows its
    ground at once (Nick, 2026-10-04).
  - *(Claude's starting rule: a walking unit passing over a tile an ally
    stands on is drawn in front of the ally.)*
- **The HP bar never overlaps the sprite's feet** (Nick: "make sure the hp
  bar is not overlapping the sprite"). The sprite is drawn higher on its
  tile, so its feet stand **directly on top of** the 2-pixel HP bar (the
  16×20 sprite ends 2 pixels above the tile's bottom edge, and its head
  reaches 6 pixels into the tile above). Nick saw it raised 1 and 2
  pixels more and chose this ("sprite 0 … looks best to me"). HP bars are
  drawn over every sprite, so HP is never hidden.
- **The HP bar is 14 pixels wide**, one pixel in from each side of its
  tile, always and **in both looks** (Nick: "1 less pixel left and right
  … always active", "glyph look should also have 14px hp bar"), so the
  bars of units standing side by side don't run together. Sprite units
  get it in 0436, the glyph look in 0441.
- **A sprite never covers the unit above it** (decided 2026-10-02, ticket
  0039, "A is best"): a sprite and its coloured outline are **not drawn
  inside a tile another unit stands on**, so a head or hat is shaved off
  at the tile's edge instead of covering that unit's feet and HP bar. A
  unit with nobody above it keeps its whole head, and heads still overlap
  empty ground, trees and buildings. A walking unit is shaved the same
  way while it passes under someone. *(Claude's starting rule: a unit
  walking through a tile an ally stands on is drawn in front of the ally
  and not shaved there, or it would vanish.)*
- **A sprite unit under an effect** (decided 2026-10-02, ticket 0039,
  "1A"): a small **arrow in the top-right corner of its tile**: an **up
  arrow for a bonus** (a stance such as Sidestep's avoid +20) and a **down
  arrow for a penalty** (Pinning Shot's Mov −3, Pressure Point's Spd −3).
  The arrow is 5 pixels wide with a dark edge and sits **1 pixel higher
  than in the first mockup** (Nick: "mark 1"): with its edge it reaches 3
  pixels above the tile's top edge and 1 pixel past its right edge. The
  direction carries the meaning, not only the colour (light blue up,
  purple down in the mockup, *tunable*). On an acted unit the arrow dims
  with the sprite.
  - **The marks move** (Nick: "bouncing up and down for up/down arrows"):
    the up arrow bounces 1 pixel up and the down arrow 1 pixel down (every
    375 ms in the mockup, *tunable*).
  - **A unit with both a bonus and a penalty** shows one arrow at a time:
    **the two take turns** (every 750 ms in the mockup, *tunable*). Nick
    saw them stacked and as a turning swirl too: "B looks best".
  - **When another unit stands in the tile above**, the mark sits 4 pixels
    lower, inside its own tile, so it doesn't touch that unit
    *(Claude's starting rule, shown in mockup M)*. The same on the top row
    of the map view, where the arrow would otherwise be cut by the view's
    edge *(Claude's starting rule, ticket 0436)*.
  - **Which arrow**: an effect that lowers any of the unit's numbers is a
    penalty; any other is a bonus *(Claude's starting rule, ticket 0436:
    the rules don't label effects, and none today both raises and
    lowers)*.
  - *(Claude's starting rule: marks on an acted unit keep moving, dimmed,
    as in mockup K4.)*
  - The game has no poison or other lasting ailment today. If one is
    added it gets its own small picture in the same corner, **changing
    between two colours** (Nick: "changing between colors for poison";
    he was shown a green drop as an example).
  The glyph look keeps its tinted background.
- Cursor, path line, HP bar and range tints keep their rules below.

### Units on the map (the glyph look)

- A unit is drawn as **two letters: an initial pair from its name** (`Al`,
  `Be`), in its **faction colour** (player blue, enemy red, ally green, neutral
  yellow) on the terrain background. **No faction backing.**
  - Generic enemies use the first two letters of their class (`Br` Brigand,
    `Ra` Raider, `Ar` Archer); named enemies and bosses use their name.
  - Content can override the pair (e.g. two party members named "Al…"). A
    duplicate pair within one side on a map is a content validation error.
- **Acted** units are **dimmed** toward the background; their initials keep
  their case (`Al` stays `Al`). The dimming is a brightness change, not a hue
  change, so "has acted" doesn't rely on hue alone. *(Changed 2026-09-28 from
  "lowercase and dimmed". Nick, after playing 0404: "seems after moving the
  initials for my units goes from i.e. Lo to lo I think it should just stay Lo
  but be shaded different (it already has this aspect so keep it like that)".
  ADR-0029.)*
- **HP bar:** a thin bar (2 px of the 16 px tile height) along the bottom of the
  unit's tile, **14 px wide, 1 px in from each side** (changed 2026-10-02,
  ticket 0039, from the full 16; built in 0441), filling left to right by
  current HP %. It's `hp_high` above 2/3,
  `hp_mid` above 1/3 and `hp_low` at or below 1/3 *(thresholds tunable)*. The
  unfilled part is dark. The bar's length carries the information, not only its
  colour.
- **Arrows at the map's edges** (ticket 0449, as for the bought tiles
  above): an arrow glyph (`→ ← ↓ ↑`) in the **cursor's yellow** in one cell
  of the middle tile of each edge the map goes on past (the tile's outer
  cell on the right, its left cell elsewhere), fading into the cell's
  background and back once a second; the terrain glyph or unit letter in
  that cell gives way to it. Only while browsing. The glyph look's view is
  35 × 30 tiles, so no map the game has today shows them.
- The **side panel shows the unit under the cursor** (name, class, level, HP,
  stats, weapon, terrain). That's where class and full numbers are read.
- Class symbols (`†`, `»`, `}`, …) were tried and rejected: the font's symbols
  are too small and generic. Custom-drawn class icons may be explored later
  (ticket 1006); until then, initials are the rule.

### Cursor and selection

- **Browsing:** thin **corner marks** around the tile, in `cursor` colour,
  **pulsing between bright and about half brightness** (period about 1 s,
  *tunable*). Never fully off. Each corner is two 1-pixel arms, **3 px** along
  and 3 px down (decided 2026-09-27, ticket 0416; the bracket cursor in the
  0011 screenshots is replaced). The marks sit in the gaps the font leaves
  around letters, so **they never cover a unit's initials, including a
  neighbour's**.
- **Accessibility options** (picked in the Options menu, ticket 0805):
  - **Large corners:** the same marks with **4 px** arms.
  - **Tile glow:** no marks; the tile's background is tinted towards the
    `cursor` colour (up to about 30% at full brightness, *tunable*), pulsing
    the same way.
- **On picture tiles** (the bought tiles; decided 2026-10-13, ticket 0444,
  from rendered options). Nick, 2026-10-12, after playing Chapter 1: "the
  cursor is also a bit hard to see". Shown thick marks, marks with a dark
  edge, both together and a frame round the tile, he picked: "can we use C
  style [thick marks with a dark edge] for 1x and B style [thin marks with
  a dark edge] for 2x?"
  - **Map at 1× zoom:** corner marks **2 px thick and 4 px long**, with a
    1 px near-black edge round them.
  - **Map at 2× zoom:** marks 1 px of the art thick and 3 long, with an
    edge of 1 px of the art (so 2 px thick, 6 long and a 2 px edge on
    screen).
  - The marks and their edge stay inside the tile. The marks pulse as
    above; the edge doesn't, so the cursor never fades into the ground.
  - **Large corners:** the same, one pixel longer.
  - **Tile glow** is not drawn on picture tiles. Nick: "for tile glow imo
    we deprecate it for the 2D style it's simply unsustainable to keep it
    around for this style but for glyph we might consider keeping it".
    And of the Options menu: "just hide it on options menu, it should not
    be selectable when graphics are 2D/picture mode". So while the map is
    pictures the `Cursor` row offers only `Corners` and `Large corners`.
    A player who had picked the glow sees the corner marks, and the row
    says `Corners`, until they pick again or go back to glyphs (*Claude's
    starting rule*). The glyph look keeps its glow.
- **Unit selected, cursor still on it:** arrows `►Al◄` replace the corner
  marks. *(Open, see below: as whole glyphs they would cover a neighbour's
  initial, as the brackets did.)*
- **Moving the cursor away from a selected unit:** a **path line** runs through
  tile centres from the **edge of the unit's tile** (never over its letters) to
  the destination tile, and **ends in a single arrowhead** on that tile. It's
  drawn under glyphs, in `path` colour. The arrowhead marks the destination;
  there's no separate cursor frame there.

### Threat arrows

Decided 2026-10-12, ticket 0055, on mockups drawn over the Chapter 1 map
with the bought art (five looks: curved lines, straight arrows, a mark
over each enemy, a red frame around each enemy, lines and marks). The rule
(which enemies, when) is in `turn-structure.md`, *Threat arrows*.

> [which look:] "B + C" [straight arrows, and a mark over each enemy] …
> [shown the two together:] "pls shift the exclamation mark a few more
> pixels up, rn it covers the enemy's head and looks weird" … [an enemy
> off the screen: A the arrow comes in from the screen's edge:] "A"

- **A slim straight arrow from each enemy to the tile**, ending in an
  arrowhead just short of the tile's middle. In the mockup: a 1-pixel red
  line with a 1-pixel dark edge each side, starting at the middle of the
  enemy's sprite; the arrowhead's two barbs are 4 pixels long (colours
  and sizes *tunable*).
- **A mark over each of those enemies:** a yellow `!` with a dark edge,
  **above the sprite's head, not on it** (Nick: the first mockup had it 4
  pixels lower, "it covers the enemy's head and looks weird"). In the
  mockup the mark is 3 pixels wide and 9 tall (a 6-pixel bar, a 1-pixel
  gap, a 2-pixel dot), centred on the tile, its lowest pixel 4 pixels
  above the tile's top edge (*tunable*, but it must clear the head).
- **An enemy off the screen:** its arrow **comes in from the edge of the
  map view**, along the same straight line. (Its mark is off the screen
  with it.) Chapter 1's map fits on the screen whole, so this is for
  bigger maps and the zoom (0439).
- Arrows and marks are drawn over units and terrain, so they are never
  hidden.
- *Claude's starting rules (Nick can veto):*
  - **The glyph look** gets the same arrows and marks, drawn over the
    glyphs. Nick saw only the bought-art look. One difference (ticket
    0448): there an arrow starts at the **edge of the enemy's tile**, not
    its middle, so it never covers the enemy's two letters (the path line
    starts at its unit's edge for the same reason).
  - The arrows are in the palette's `enemy` colour and the marks in
    `text_highlight`, both edged in `black`, so they follow the colour
    theme (ticket 0448).
  - The mark is drawn in front of a unit standing in the tile above, and
    on the map view's top row it is moved down to stay inside the view.
  - Neither moves or blinks.

### Attack forecast

Decided 2026-09-27, ticket 0404, over three rounds of renders (the ticket's
two-column `DMG 7+8 ×2` panel, a stacked panel, a wide box over the map,
then three strike-list layouts). Screenshot: `0404-forecast.png`.

> "from what you shown me, none are perfect but here's what I like from each
> / I like the bar representing HP in the forecast. / I like not having all
> upper case for the statistics / as for multi strike, I am leaning towards
> having them on multiple lines somehow... idk how in practice but like
> Strike 1 Strike 2 etc? maybe showing a total? and number of strikes? / we
> should also forecast with a glyph showing it will kill on which strike
> (assuming all hit)" … "strikes should be in order they happen like E / the
> death glyph shouldn't be a cross it should be a skull or an X or something"
> … "ok skull is fine.. seems good"

- While a target is chosen, the forecast **replaces the side panel** (double
  border, title `Forecast`): the attacker in the left column, the target in
  the right, names in faction colour, weapon names dimmed under them.
- **Mixed-case labels:** `HP`, `Hit`, `Crit`. A side that can't counter shows
  `--` for Hit and Crit.
- **HP bars:** `HP 19` and a short bar. The part of the bar the unit would
  lose **if every strike hit** (no crits) is shaded in `hp_low`.
- **Strikes in the order they happen**, one per row, numbered down the
  middle: the attacker's on the left (`8 dmg →`), the target's counters on
  the right (`← 9 dmg`). `no counter` in the right column when it can't.
  After a separator, a **total** per side: `17 ×2` (damage of its strikes and
  how many).
- **Kill mark:** a small **skull**, pixel-drawn in `hp_low`, on the strike
  that makes a unit fall if every strike before it hit. Strikes after it are
  dimmed: they happen only if something misses.
- Effective weapons put `!` after `dmg` in the highlight colour; a broken
  weapon shows `(broken)` in `hp_low` under its name *(Claude's starting
  rule, not rendered for Nick)*.
- The Combat Arts list and art line (0414) go above this forecast; that
  ticket works out the combined box.

### Portraits and battle art (bought)

Decided 2026-09-30, ticket 0021 (it replaces the 2026-09-25 rule that
portraits are 32×32 pixel art drawn by Claude with Nick). Nick judged
mockups made from store previews in our dialogue screen and a stand-in combat
scene: one artist with both kinds of art (SolaarNoble, Tiny Tales, Time
Fantasy), and CaptainSkolot's portraits next to another artist's battle
sprites.

- **Artist: Mega Tiles, "Tiny Tales" packs** (<https://megatiles.itch.io/>),
  hand-drawn pixel art, medieval fantasy. The core packs are *Tiny Tales 2D
  Heroes: A New Beginning* and *Heroes 2: Rebellious Souls* ($24.99 each on
  2026-09-30). Each hero comes with a face set, a large portrait, a big still
  battle image, a small animated battle sprite and a map sprite. The still
  battler packs (e.g. *Vol.5 Faith and Evil*, *Vol.1 Monstrous Uprising*)
  add classes with a still battle image and a map sprite but **no face**.
- **Bought 2026-10-02:** Nick bought Mega Tiles' whole "2025 Bundle Sale"
  (37 products, $99.99; his words: "I paid $100 for all these assets").
  It holds both Heroes packs, the Character Generator EX, the still
  battler packs Vol.1–5, *Gods and Gallants*, *Epic Monsters*, the map
  sprite packs, five 16-pixel tilesets (World Map, Overworld, Dungeons 1
  and 2, Tower), *Battlebacks Vol.1* (24 battle backgrounds), and some
  packs we don't use (a sci-fi set, a sci-fi UI kit, three 48-pixel
  terrain and tree packs in another style). The files are in the
  private assets repository, never in this one (ADR-0032, ADR-0040).
  The purchase record is in `THIRD_PARTY_ASSETS.md`.
- **What the real files are** (checked 2026-10-02; earlier notes came
  from store previews): a face is 48×48 pixels and a **bust is 80×80**,
  and **both come in the same 8 expressions** (neutral, smile, stern,
  sad, surprise, thinking, sly, unique), as single files and as a sheet;
  a map sprite is a 48×80 sheet of 16×20 frames (3 walking frames, 4
  directions); every tile is 16×16; a battle background is 336×248. Each
  pack ships its art at 1×, 2× and 3×: we use the 1× files.
- **No Claude-drawn character art.** Portraits and battle art are bought.
  Claude may only make **small edits** to bought art: recolours, a scar,
  spectacles, a missing expression made from an existing face. No new hair,
  clothes or bodies (that's redrawing).
- **Paid art is fine; Nick decides each purchase himself.** He doesn't want a
  price ceiling asked. Music and sound stay free (`audio.md`).
- **AI-assisted art is acceptable if a human has worked on it**, e.g.
  CaptainSkolot's "local generative tool for rough concepts, then refined by
  hand". Art that is AI output with no human touch is not. The Tiny Tales
  packs say no generative AI was used. If AI-assisted art ships, the Steam
  page discloses it (ticket 0903).
- **Every bought work is credited** on the credits screen (0808), even when
  its licence doesn't ask for credit (`audio.md` rule 3).

#### Dialogue portraits

- **The busts, at 4×, filling the frame** (decided 2026-10-02, ticket
  0039, "2C"; it replaces 0021's rule of the 48×48 faces at 5×, whose
  reason, that the large portrait has only one expression, turned out
  wrong on the bought files). Each Tiny Tales hero has an **80×80 bust in
  8 expressions**. The bust is cut to its **middle 64 columns and bottom
  64 rows** and drawn with square pixels at **4×**, which is exactly the
  256×256 pixels of the existing 32×16-cell frame, so the dialogue layout
  doesn't change. Against the faces: the whole hat and the shoulders show,
  and a face is about four fifths as big. **What the cut loses:** the tips
  of very tall ears or horns (the Fox Miko's ears) and the outer edge of
  the widest shoulders. A bust that sits off-centre may be cut a few
  columns to one side instead *(Claude's starting rule)*. Nick first saw
  the bust at 3× with empty space around it and asked for it bigger ("makes
  me think bust 4x might be possible"). Ticket 0711.
- **Expressions:** the five the dialogue needs (`neutral`, `happy`, `angry`,
  `sad`, `surprised`) are mapped to the closest of the 8 per character
  (0706). A missing one may be made by a small edit.
- **The Chapter 1 cast's faces** (decided 2026-10-04 and 2026-10-11,
  ticket 0706; each character's sheet in `docs/story/characters/` has the
  face and its expressions):
  - **Bought heroes:** the lead is the Male and the Female Fighter, Aske
    the Archer, Rue the Witch. The female lead's top is recoloured from
    pink to his coat's blue (Nick: "for lead yes recolor").
  - **Generator faces**, picked by Nick from three mockups each ("for
    Hollis A, for Tamsin B, for Maud C, for Dace A, for messenger A, for
    soldier B", "for harl I actually like c") and made by him in the
    Character Generator: Hollis, Tamsin, Maud, Harl, Dace, the Vigil
    `messenger` and the one `soldier` face of unnamed enemies. Harl's hair
    and beard are brown ("there isn't a good red brown color in the
    generator for the hair"); Dace has no ears drawn ("he didn't look good
    w/ ears").
  - **What the generator couldn't do:** hoods, headscarves, caps, wrinkles.
    So Maud's hood is down, Harl has no headscarf in dialogue, and Hollis
    and Harl look younger than they are.
  - **Maud's eyes are softened by Claude's small edit** (decided
    2026-10-15, ticket 0730). Nick, after playing Chapter 1 (0804):
    "Maud's eyes look too scary like they are cat-like like too skinny,
    too focused, they stare into my soul... maybe she needs different
    eyes". The generator's eyes gave her 2-pixel irises with a lot of
    white and a hard flick at the outer lid. He was shown all 8
    expressions beside Hollis and Rue, at the game's size and 2×, in four
    versions: two other generator eye shapes (droopy, angled), Claude's
    edit of her own eyes, and the Dancer hero's eyes. He picked **the
    edit**: each iris one pixel wider, a white catch-light in the near
    eye, and the outer flick of the near lid turned to the lid's brown
    shade. Her map sprite and combat picture are the Church Cleric's, not
    this face, so they didn't change.
- **Characters who fight and have no fitting bought art**: Nick wanted
  more packs in a similar style (2026-10-02: "help me locate some more
  itch bundles to fill in with a similar style for rest of cast"). What he
  chose is in *Fighters with no Tiny Tales art* below (ticket 0040).
  **Characters who never fight** may get a face from the Character
  Generator ("if these are non-battle units, I think the generator can
  work").
- **Characters no bought face fits** (in Chapter 1 likely Hollis, Harl,
  Maud and Crane): first Mega Tiles' **Character Generator EX** (bought
  with the bundle; version 1.2), a Windows program that makes new
  characters in the same style: a face with 8 expressions, a small animated
  battle sprite and a map sprite. Its licence (the same text as the packs)
  allows the characters it makes in a sold game. It makes **no big still
  battle image**, so in combat a generated character uses its class's
  still image. Version 1.2 has six kinds of outfit (soldier, rogue,
  brawler, commoner, traveller, simple dress), one wizard hat, beards and
  moustaches, and no hoods, helmets or glasses, so some faces aren't
  possible yet. Otherwise, or on
  top, Claude's small edits, or rewriting the character's written look to
  fit a bought face (see *Combat screen* below; Nick allows it, done in
  0706 after purchase). **No commissions**: Nick vetoed them as too
  expensive for single assets.
- **Where portraits appear** (unchanged):
  - **Conversations:** two full portraits, speaker on the left at full
    brightness with a double-line frame, listener dimmed on the right, name
    plates underneath, 3-line text box below.
    Narration (no face, no name) sits in the same box from the same
    column as speech, on the first text row, in the dim narration colour
    (ticket 0740; it was centred before).
  - **Not** in the battle side panel or hover. That panel shows stats only
    (Nick dropped the mini portrait).

#### Combat screen

- Nick expects **full-body art of the two fighting units** there, not
  portraits.
- **The art is the big Tiny Tales still battle images, moved by the game**
  (like the enemies in Final Fantasy VI or Dragon Quest): each fighter is one
  picture; the game makes it lunge to strike, flash on a hit, shake, and fade
  when defeated. The packs' small animated battle sprites are **not** used
  there. Scale, the exact motions and the rest of the scene are ticket 0413.
- **The two fighters always face each other** (decided 2026-10-02, ticket
  0038): a picture that faces the wrong way is mirrored. In the mockups the
  player's fighter stood on the left and the enemy on the right; the enemy
  was drawn as bought and so faced away, which Nick called the main thing
  wrong with them.
- **The fight fills the screen above the help bar** (decided 2026-10-11,
  ticket 0413).
  Nick first saw two mockups (0038), one in the box over the map and one
  filling the screen, and found both "a little off"; his worry about the
  big one was that it "won't be able to convey all the info", though
  "maybe it can work with some additional in-battle overlays or pop up
  messages". In 0413 he saw four layouts with the facing fixed: the box
  with and without a background, and the map pane as a stage with and
  without one. "1A or 1B not the 2 option but torn between A and B", then,
  after seeing both with the run-up and Harl: **"1B"**. Seeing it run in
  the game, he dropped the side panel too: "the side panel is somewhat
  useless in the fight scene... it has the same information... we can
  just stretch the whole fight scene over" it. So:
  - While a fight plays, the map **and the side panel** make way for the
    two fighters at **double size** in front of a bought **backdrop**;
    the help bar stays. There is always a backdrop. (The backdrop is
    narrower than the screen at that size: its two edges are drawn again,
    mirrored, to fill the width.) Nick allowed keeping the side panel for
    the glyph look; the stage is drawn the same whichever map look is on,
    so it never shows it *(Claude's reading; say if the glyph look should
    get the old box back)*.
  - **A plate per fighter** under the picture: name, HP with a bar, and
    its damage, hit and crit for this fight (or "no counter"). The name
    lights up when that fighter is about to strike.
  - **The result** (`HIT -8`, `MISS`, `CRITICAL! -27`) pops up over the
    target's head. An art's or an active's name shows at the top.
  - **The player's fighter stands on the left**; if neither or both are
    the player's, the attacker does *(Claude's starting rule; the mockups
    Nick chose from were drawn that way)*.
  - With **combat animations off** (Options) fights aren't shown at all,
    as before.
- **The motions** (decided 2026-10-11, ticket 0413, on GIFs of both): the
  striker **runs up to its target and back**, like Fire Emblem GBA, not
  the short hop of Final Fantasy VI (Nick: "2B"). The rest was in every
  GIF he saw and is *tunable* in `stage/motion.rs`:
  - a target that is **hit** flashes white and shakes;
  - a target that is **missed** steps back out of the way and returns;
  - a **critical** leans back first, then dashes; the whole stage flashes
    white and the target shakes twice as hard;
  - a fighter that **falls** fades and sinks.
  - *Claude's starting rules:* a fighter striking from **two tiles or
    more away** (a bow, a spell) only steps forward a little instead of
    running up; a strike that **heals** its target (Absorb) moves nobody;
    a line spoken inside a fight (a boss's, last words) plays **over the
    stage**, not over the map.
- **Backdrops** (ticket 0413): Tiny Tales' *Battlebacks Vol.1* has
  desert, oasis, two fields, meadows, two mountains and wasteland, each
  at day, dusk and night; nothing indoors, no forest interior, no town,
  no bridge or water. *Claude's starting rule:* a fight on a **mountain
  or a peak** is in front of the mountains, **every other terrain** in
  front of the fields with the forest edge behind (so a forest, a fort, a
  bridge and a village all fall back to the fields). Which one is the
  terrain the **defender** stands on. The picture follows the map's
  light (ticket 0438): the bundle's own dusk and night pictures of the
  same place; a light the bundle has no picture for *(none today)* shows
  the day picture shaded like the map's ground *(Claude's starting
  rule)*.
- **Classes with no hero art use a still image** (updated 2026-10-04,
  ticket 0040; the full list is in *Fighters with no Tiny Tales art*
  below):
  - **Cleric:** the church cleric (*Faith and Evil*); she is also Maud's
    picture.
  - **Guard:** the church knight (*Faith and Evil*).
  - **Brigand:** Shironejiya's bearded axe bandit in its second colours;
    **Raider:** its armoured axe fighter in red (decided 2026-10-11,
    ticket 0413; it was the orc axe fighter). Both classes fight with
    axes and the picture's weapon must match, and of the headscarf
    bandits only the bearded one holds an axe (the others are bare-handed
    or hold knives). So Brigands look like Harl in other clothes. Nick:
    "2A but open to change as we progress in development".
  - **Rider:** Pixel Flag's Paladin (it was an on-foot lance fighter).
  0413 picks the exact images. None of these have faces; they're generic
  enemies or get a face from the Character Generator.
- **Named characters use their class's picture, recoloured to their own
  colours** when they have no picture of their own (like Fire Emblem GBA,
  where most named units share their class's battle animation in their own
  palette). Characters with their own hero art use it for both face and
  combat picture.
- **Who is recoloured in Chapter 1** (decided 2026-10-11, ticket 0413, on
  renders of each pair; Nick: "1A", "2A", "3A"):
  - **The Guard:** the Church Knight's gold trim, tabard and shield are
    **red for the Red Company's guard** and **faded Veyne blue for
    Hollis** (both recoloured, so neither is the bought white and gold).
  - **The Archer:** **Aske is the Archer hero as bought**; the **Red
    Company's archers** are the same picture with dark hair and red in
    their clothes, so they aren't her twins.
  - **The female lead's** top is blue, as in her portrait (0706).
  - A **class's picture is what the enemy's soldiers of that class look
    like**; a named character of the player's who shares it gets their own
    recolour *(Claude's rule for how the files are set up; in Chapter 1
    the only soldiers without a name are the Red Company's)*.
- **Sizes** (decided 2026-10-11, ticket 0413): the **Church Cleric (Maud)
  and the Church Wizard (the Mage class) are drawn at 1.25×**, like
  Shironejiya's pictures, so they stand as tall as a hero; the Church
  Knight is already that tall and stays at 1×. The heroes are 1×.
- **Face and combat picture should match** (Nick prefers visual
  consistency). Where a character's written look doesn't fit the art we
  can buy, the written look is rewritten to fit the art after purchase
  (0706), not the other way round. First case: **Rue** uses the Tiny Tales
  **Witch** for both, hair recoloured as needed.
- **Harl** (Chapter 1 boss) has no fitting picture in the Mega Tiles
  catalogue: the axe fighters are orcs, beasts or a minotaur, and a human
  knight would change his weapon (weapon type is a rule, so the picture must
  match it). The orc, the Dragon Knight and the Magitek dark knight were
  shown and not chosen. **He uses Shironejiya's bearded axe bandit**
  (ticket 0040, below), and his written look was rewritten to fit it
  (ticket 0035, `red_captain.md`): a headscarf and a bare-armed vest, not
  a kettle helm and mail.

#### Fighters with no Tiny Tales art

Decided 2026-10-02 to 2026-10-04, ticket 0040, on mockups of each
candidate beside the lead in a combat scene. Four search rounds covered
itch.io, the Unity Asset Store, GameDev Market, CraftPix, OpenGameArt,
BOOTH, DLsite, Nico Commons, SRPG Studio and RPG Maker material, the
long-running Japanese free-material sites, stock sites and AI-made packs.
Nick's words on the first shortlist: "I hate all of them... these all look
like shit". What the search was held to after that:

- **A battle picture is drawn side-on, so the two fighters face each
  other** ("I need them to be facing each other ideally"). A picture that
  faces the camera is out.
- **Sizes between 1× and 2× are allowed** ("maybe 1.5x or 1.25x"), so a
  picture from another artist can stand as tall as a Tiny Tales hero.
  Tiny Tales' own class pictures are drawn smaller than its heroes (the
  Church Cleric is 42 pixels tall, the lead 62; Nick: "the scale is
  slightly different than the hero character"), so they may be enlarged
  the same way. 0413 sets each picture's size on the real screen.
- **AI-assisted art only sparingly**: "not a huge fan of them unless we
  use them sparingly like for bosses or rare enemies". None is used today.

**Who uses what:**

| Fighter | Combat picture | Size | Map sprite | Source |
| ------- | -------------- | ---- | ---------- | ------ |
| Mounted classes (Rider, Lancer, Outrider, Iron Rider; Tamsin) | The **Paladin** (armoured lancer on a white horse), recoloured per character | 1.5× | MattWalkden's riders | Pixel Flag; MattWalkden |
| Harl | The **bearded axe bandit** (`bandanna05`) | 1.25× | a Tiny Tales human | Shironejiya |
| Brigand | The **bearded axe bandit in its second colours** (`bandanna05_b`) (0413) | 1.25× | a Tiny Tales human | Shironejiya |
| Raider | The **armoured axe fighter**, red (`fighter04_a`) (0413) | 1.25× | a Tiny Tales human | Shironejiya |
| Other human bandits (none in Chapter 1; they must not fight with axes) | The **headscarf bandits** (`bandanna01`–`04`) | 1.25× | a Tiny Tales human | Shironejiya |
| Aske | The **Archer** hero, as she is | 1× | the Archer's | Tiny Tales (owned) |
| Maud (the keeper; Cleric) | The **Church Cleric** | 1.25× (0413) | the Church Cleric's | Tiny Tales (owned) |
| Crane | The **old hooded man with a staff** (`mage_old_m01`), with a spectacle rim added by Claude (a 21-pixel edit) | 1.25× | a Tiny Tales human | Shironejiya |
| Brawler | The **fist fighter** (`martial03`) | 1.25× | a Tiny Tales human | Shironejiya |
| Dace | The **dark knight** (`darkknight`): black horned armour, closed helm, red cape | 1.25× | a Tiny Tales human | Shironejiya |

- **JAPANweb's art can't be used.** The art Nick liked best was
  JAPANweb's *SRPG Studio ICON オリジナル版* (riders, pegasus, griffon and
  wyvern riders, fighters and a monk, side-on, with map sprites): "would
  prefer if we can use japanweb". Its terms forbid use outside SRPG
  Studio, so Nick emailed the artist on 2026-10-03. He answered by
  2026-10-04; Nick: "apparently the reason he said SRPG Studio only is
  because he modified the SRPG Studio assets... so we really can't use
  them". SRPG Studio's own terms lock its bundled art, and every edit of
  it, to games made with SRPG Studio, so his permission would not be
  enough. The table above is therefore the answer, not a stand-in.
- **Aske and the keeper were rewritten to fit the art** (Nick: "I don't
  have a major attachment to any story beats... whatever is lowest lift in
  your POV that makes sense"). Aske is a Brennish girl of 17 drawn as the
  Archer; her pointed ears are never mentioned (the world has no elves).
  The keeper, a man called Piers, is a woman called **Maud** drawn as the
  Church Cleric ("Piers reads as a man's name so I would like the name
  changed") *(the name Maud is Claude's; it can be changed in
  `names.ron`)*.
- **Dace is not the Samurai hero**: "this samurai is clearly female".
  The dark knight's helm hides his face, so his dialogue face comes from
  the Character Generator and need not match (an exception to *face and
  combat picture should match*).
- **Shironejiya's bald monk** (`martial01`) is kept for one of the
  Brawler's promoted classes ("I like both assets... maybe we can use one
  for a promoted class"); which one is that class's ticket's call.
- **Holder's axe fighter** (free animated battlers) is "ok but not for
  harl or brigands": only for a party member it happens to fit. Use only
  Holder's own designs, not the ones modelled on RPG Maker's characters.
- **Fallback already judged**, if art is needed for a boss or a rare
  enemy and nothing above fits: from cogabushi's AI-assisted side-view set
  Nick "doesn't mind much" the red-armoured captain with a battle-axe and
  the crescent-axe fighter.
- **The sources and their terms** (read on 2026-10-03 and 2026-10-04;
  each gets a row in `THIRD_PARTY_ASSETS.md` when its files come in):
  - **Shironejiya (白螺子屋)**, <http://hi79.web.fc2.com/>, free,
    hand-made. 「シェアウェア等への商用利用可」「使用するゲーム作成ツール
    は特に制限無し」「素材の加工、修正可」: paid games, any tool and edits
    are allowed. The game must carry a text with the site's name, its
    address and "the materials in this game may not be reused"; the
    credits screen and a bundled text file do that. No redistribution.
    Adult games may not use it commercially. The terms also say 「生成AIと
    呼ばれるツールの使用は不可です」 (tools called generative AI may not be
    used), which in context is about feeding the art to image generators:
    we never do that.
  - **Pixel Flag (ぴくせるふらっぐ商店)**, *ドット絵TRPGのコマ「クラスと
    モブ」全64種*, <https://booth.pm/ja/items/7851483>, ¥500. Commercial
    use and edits allowed, no resale or redistribution, no credit needed,
    "AI生成不使用の手打ちドット100％". Nick likes only its Paladin. **Not
    bought yet.**
  - **MattWalkden**, *Fantasy Battle Pack*,
    <https://mattwalkden.itch.io/fantasy-battle-pack>, pay what you want.
    "Royalty free commercial and personal license"; "No generative AI was
    used". Idle, move, attack and death in 4 directions ("as long as they
    are animated same as rest of our units MattWalkden looks good"). Read
    the licence file in the download for edits before the first use.
- **Not usable, so nobody rechecks:** SRPG Studio's bundled art and every
  edit of it (its terms allow it only in games made with SRPG Studio);
  RPG Maker stock-character lookalikes; Fire Emblem fan sprites (reworks
  of Nintendo's art, no commercial grant); JAPANweb's sets (built on SRPG
  Studio's art, as he told Nick; the others are fan art
  of licensed series, or barred from money-making use).
- **Seen and not chosen:** the *SRPG Maker Asset Pack* on Steam
  (<https://store.steampowered.com/app/2819000/>, $17.99; 1998 graphics
  with mounted knights, dragon riders, map units and faces; "Makerシリーズ
  以外でも利用可・商用利用可"), in a 1990s semi-realistic style. Nick has
  the link; nothing is decided about it.

#### Act 1's new cast

Decided 2026-10-26 to 2026-10-28, ticket 0070 (Hedda and the Hound
lieutenant left to 0076). Nick judged mockups
in the scratchpad (never committed, ADR-0040): each candidate's bust cut
as the dialogue frame shows it, and their combat picture at double size
on the fields backdrop facing Harl or the lead. Faces that don't exist
yet were rough mockups put together from the Character Generator's part
images, as in 0706; Nick makes the real ones in the generator. Two
searches beyond the bundle (itch.io, BOOTH, GameDev Market, CraftPix,
the Japanese free-art sites, RPG Maker resources with open terms) found
Shironejiya the only close match in style, so more of its pictures are
used; its own faces are an older 48×48 style that doesn't fit our
busts, so they aren't.

| Who | Face | Combat picture | Status |
| --- | ---- | -------------- | ------ |
| Joss Pellam (`defector`) | The **Thief** hero (Heroes 1), with its map sprite | Shironejiya's **sword-and-shield hero** (`m_s_hero07`) at 1.25×, recoloured by Claude: hair brown and headband dark like the Thief's, and its red cloth turned Hound black | Decided ("F Hound black") |
| Oriel Mast (`battlemage`) | A **generator** face: greying hair, stubble, a grey-and-gold coat | Shironejiya's **court mage** (`m_s_mage_middle_m01`) at 1.25×, casting with an arm out, his blue hair recoloured grey | Decided ("D Court mage") |
| Crane (`vowmaster`) | A **generator** face: long white hair and a full white beard, with a spectacle rim added by Claude | Unchanged (0040): Shironejiya's old hooded man | Decided ("F Long hair"); his sheet becomes bearded |
| Ragna Holt (`envoy`) | A **generator** face, no combat picture (she never fights): dark-blonde hair in a bun, a traveller's coat | none | Claude's pick of Nick's two (below) |
| Dace (`rival`) in person | Unchanged (0706): his generator face | Unchanged (0040): the dark knight | Already decided; his map picture is 0735's |
| Gil Parrow (`prizefighter`) | Generator face **B**: slicked-back black hair, a scar over one eye, a goatee | Sagak Art's **"8-Bit Heroes 2: Fighter"** (<https://sagak-art-pururu.itch.io/8bitheroes-fighter>, $3, not bought yet): bare-chested, gold fist wraps, red sash, about 45 pixels tall, drawn at **1.75×** | Decided |
| Hedda Ravn (`shieldbearer`) | A **generator** face to match the picture: long red hair in a ponytail, about 32, weathered; not the pack's own painted portraits (0076) | elie automata's **Battle Axemaiden** (<https://elie-automata.itch.io/battle-axemaiden-animated-character-story-art>, free with credit, AI-assisted): red ponytail, dark plate with a red cape, a long axe; her three-quarter pose | Decided (0076: "the axemaiden seems fine for hedda") |
| The Hound lieutenant, Silas Hendry (Chapter 2 boss, axe) | He speaks (Chapter 2's scenes), so a **generator** face to match the picture: shaggy red hair and a full red beard | Stupid Creations' **TTMini: Bandit Brigand** (<https://stupidcreations.itch.io/ttmini-bandit-brigand>, $2.99, not bought yet, AI-assisted): red beard, leather and mail, red cloak, hand axe and round shield; his three-quarter pose | Decided (0076: "Bandit Brigand") |
| Greywater's boss: **the prior** (Chapter 3) | Only if he speaks | Shironejiya's **dark-robed priest** (`m_s_mage_dark_m01`) at 1.25× | Decided: the prior is the boss, not the Hound captain |
| Crane's champion (Chapter 7 boss) | Only if she speaks | Shironejiya's **blue-armoured knight** with sword and shield (`m_s_val`) at 1.25×; the champion is a woman | Decided |

**Nick's words:**

> [Round 1, Joss:] "mostly like the thief hero but if its a swordsman
> doesnt really work... the other battlers dont work for me at all. would
> suggest looking at other packs." [Gil:] "B face looks best but the
> battlers look very bizarre for this character." [Hedda:] "both
> portraits are ok but is there any in between rather than scantily clad
> (she almost looks naked) and totally full body armor...?" [During the
> round:] "u can also look at other bundles from itch and wherever else if
> our bundle doesn't suit the new chars..."
>
> [Round 2, the Thief's knife lengthened into a sword by an edit:] "well
> his blade is tilted down which is bizarre and the aliasing on the sword
> looks off so no" [Oriel:] "only C looks any good, I think zealot using a
> sword is a bit weird, and the wizard in A is black while his portrait is
> white what.... search some other packs" [Ragna:] "A or C I guess"
> [Crane:] "make him bearded, better to have bearded sheet than
> de-bearding a sprite. but A looks weird is he supposed to be bald? idk I
> picture him with hair"
>
> [Round 3, Crane:] "F Long hair" [Ragna:] "idrc whichever fits her sheet
> better" [Bosses' faces:] "Only if they speak"
>
> [Round 4, Shironejiya's pictures:] Joss "F Hound black", Gil "None",
> Hedda "keep looking.", Oriel "D Court mage"
>
> [On the bosses' pictures:] "please give me more insights into who each
> character is before asking me to pick each char. I frankly have not
> kept pace with the story"
>
> [Round 5, after a summary of each boss:] lieutenant "Search more";
> Greywater "the prior with the Shironejiya mage sprite, since the cleric
> sprite is a female and priors can't be female afaik" (then, asked
> which: "B Dark priest"); the champion "the valkyrie / blue knight looks
> best but is female. is it ok for the story?" (yes: nothing in the
> story fixes the champion's sex); bosses' faces "we can iterate on the
> face together using generator components, and you can tell me what
> components are necessary, and then I will use the generator program.
> Since your simulated generator does not work as well as you think it
> does."
>
> [Round 6, Gil with the Sagak fighter at 1.5× and 2×:] "1.75X with
> sharpening might look ok? I like it but it's blurry." (The blur was
> the store preview, pasted and scaled; the bought file is sharp pixel
> art at its own size.) Hedda and the lieutenant: "idc keep looking."

- **Generator faces are made by Nick from a list of parts.** Claude's
  mockups pasted from the generator's part images are rough; for every
  generator face still to make (Gil, Oriel, Crane, Ragna, and a boss who
  speaks) the chapter ticket gives Nick the parts (body, hair, eyes,
  facial hair, outfit, colours) and they iterate together; Nick builds it
  in the generator.
- **Gil's picture is bought by Nick** (paid art is fine, Nick decides
  each purchase). It is credited (`assets/data/credits.ron`) and listed
  in `THIRD_PARTY_ASSETS.md` when Chapter 4 imports it; its licence:
  commercial use and edits allowed, no redistribution or resale.

- **Ragna** *(Claude's pick)*: her sheet isn't written yet (0743), so
  the bun and traveller's coat, which suit an envoy on the road, over the
  long red hair and dress. 0743 describes her to match.
- **Crane's sheet** changes from clean-shaven to a full white beard and
  long white hair (Nick's call; it matches his combat picture).
- **Lesson for these questions:** say who a character is in the story
  before asking Nick to pick their look.

**Hedda and the Hound lieutenant** (decided 2026-10-28, ticket 0076).
New searches (Mega Tiles, Sagak Art and Shironejiya had nothing new;
itch.io, GameDev Market by hand, CraftPix, OpenGameArt, Unity, BOOTH,
DLsite and the Japanese free-art sites) were opened to any style once
Nick said so; candidates were shown as the stores show them, since
Claude's cut-outs of store previews left artifacts. Both picks are
AI-assisted, which Nick accepted for these two, so the Steam page
discloses them (0903); both are shown in their three-quarter pose, which
faces the other fighter and still shows the face (a pure side pose hides
it behind the weapon).

> [Round 1, our own art recoloured:] "Hedda still looks bad. I don't
> care about "style" just find ANY candidates at all." [Lieutenant,
> cogabushi's Captain:] "C looks ok but your recolor looks like shit
> ... just present without recolor"; "the pack seems ok..... I might get
> it, wanna see other alternatives since it is a style clash but if we
> have no other candidates then we can use it"
>
> [Hedda:] "stop suggesting to park her ... You need to either - adjust
> her to find something that actually fits (character sheet) OR work
> harder"
>
> [Round 3:] "battle axemaiden looks good enough to me from elie
> automata. If she has a good one for fitting Silas as well it would be
> good to check" [elie automata's pack characters:] "do you have their
> side portraits? since that's what we'd use for the battles right";
> "the axemaiden seems fine for hedda but im not convinced by the pack
> chars for silas"
>
> [Hedda's face:] "generator, but I wonder how much her being 44 is
> integral to the story, or if her being younger matters much, since the
> battler still looks young." [Her age:] "About 32" [Silas, after one
> more search:] "Bandit Brigand"

- **Hedda** is 32 now, Ragna's younger foster-sister, and her sheet's
  look follows the Axemaiden (`shieldbearer.md`). Her face is a generator
  face, not the pack's painted portraits (they clash with the pixel busts
  and look about 25).
- **Silas** speaks in Chapter 2's scenes, so he gets a generator face;
  his sheet's look follows the Brigand (`hound_lieutenant.md`).
- **Fallback already judged** for an axe boss: cogabushi's red-armoured
  Captain from *Pixel art-like style Enemies Pack 300+*
  (<https://cogabushi.itch.io/pixel-art-like-style-enemies-pack-300>,
  $28.80), "ok" as sold, not recoloured.

### Title screen

Decided 2026-10-12, ticket 0811, over four rounds of renders at the game's
size. Screenshot of the built screen:
[`0811-title.png`](../screenshots/0811-title.png).

Round 1 (A big block lettering on black; B thin letters from box-line
glyphs; C the game's font at 4×; D the lettering over a glyph scene with a
castle; E the lettering over a bought battle backdrop, at night or at
dusk; F the lettering with embers drifting up; a line under the title: A
none, B "an ASCII tactics game", C his own; the music: A at once, B faded
in):

> 1. probably E1 or E2 with maybe F as well. But I was thinking probably
> have a distinction between Visions and Of. The Shuyi looks good but Of
> should not have same importance as Visions. We probably need to nail
> down the backdrop.
> 2. A
> 3. A

Round 2 (lettering L1 "VISIONS of" on one line, L2 "of" in the text font
between hairlines, L3 a small "OF" between two bars; nine backdrops side
by side; the embers over a backdrop: A as shown, B fewer or slower, C
none):

> 1. I like L1 and L3. Can I see lowercase of with the lines?
> 2. rotate between them on launch imo. Other than 7 and 8 probably.
> 3. A and we might tune it a bit later but it looks more dynamic than a
> static background this way

Round 3 (L4, a lowercase "of" between the bars; the rotation's rules):

> 1. can we make the of the same color as visions
> 2. yes sounds good

Shown `of` pale with the bars gold, and with the bars pale too:

> I like pale of, gold bars btw

- **The logo is block lettering in three rows:** `VISIONS` in pale
  letters, a small lowercase `of` between two short bars, and `SHUYI` big
  below, shaded from pale gold at the top to brown at the bottom. Every
  part has a dark edge down and to the right, so it reads on any backdrop.
  Each block is 8 screen pixels. `of` is the same pale colour as
  `VISIONS`, and the two bars are gold.
- **No line under the title** (Nick: "A"). "an ASCII tactics game" is
  gone.
- **Behind the logo is a bought battle backdrop** (Tiny Tales *Battlebacks
  Vol.1*), filling the screen above the help line, its edges mirrored to
  reach the sides as in fights.
- **The backdrop changes with each launch** (Nick: "rotate between them on
  launch"). The pool is seven pictures: Fields A at dusk and at night,
  Fields B by day, at dusk and at night, Meadows at dusk and at night.
  Mountain B at night and Wasteland at night are out (their mirrored edges
  looked wrong). *Claude's starting rules, which Nick agreed to ("yes
  sounds good"):*
  - One backdrop per launch, picked at random, never the same one twice
    in a row. Example: the game opens on Meadows at dusk; quit and start
    again, and it is any of the other six.
  - It stays for the whole launch: back from a battle or Game Over, the
    title shows the same picture.
  - The lettering is the same on every backdrop.
- **Sparks drift up the screen** (Nick: "it looks more dynamic than a
  static background this way"; "we might tune it a bit later"): 46 small
  dots and stars in warm colours, each rising 9 or 18 rows a second and
  swaying a cell or two (*all tunable*, `title/glyph.rs`). They pass
  behind the logo and the menu, and show from the first frame, before the
  first press too *(Claude's starting rule)*.
- **The title music starts at once** on the first press, as before (Nick:
  "A"): the song's own opening is quiet and rising.
- **Layout:** the logo at the top; under it the menu, or `Press any key or
  button` with its two pictures on a dark band so it reads on a bright
  backdrop; a notice under the menu on a band too; the help line on the
  bottom row, which the backdrop doesn't cover.
- **The logo follows `title.md`'s precautions**: plain block letters, no
  sword or tree built into it. (The ticket's "crossed swords" idea was not
  offered for that reason.)
- **A copy of the public repository** shows the logo over a flat
  placeholder: the bought pictures aren't in it (ADR-0040).
- The intro cinematic's last shot (`title-screen.md`, *Intro cinematic*)
  and the Game Over animation (ticket 0830) use this logo.

### Game Over screen

Decided 2026-10-13, ticket 0830, over two rounds of renders made with the
game's own renderer: five moving ones, then two stills.

Round 1 (what moves: A the battle fades to black, `GAME OVER` fades in, the
logo small and gray under it, as Fire Emblem's plain fade; B the battle
stays, dark, with a black band and red words, as Dark Souls' `YOU DIED`; C
the title's gold logo drains to gray and its sparks die; D the logo
crumbles away. Behind it: A black, B the lost battle, darkened. How long:
A two to three seconds and any press skips, B the same with no skip, C
about five seconds and any press skips):

> 1. A but align the GAME OVER with the logo better...
> 2. A
> 3. A

Round 2 (the logo at its title size, so the words and the logo are the
same width: A1 the words on top and the logo under them, A2 the logo on
top and the words under it):

> A2

- **The lost battle fades to black**, then **the logo fades in** where the
  title screen has it, at the same size, **drained to gray**; then the
  words **`GAME OVER` fade in under it in gold**, in block lettering as
  wide as the logo, their edges in line with its edges. Then the menu
  (`Rewind` while a charge is left, `Retry Battle`, `Title`) and the help
  line are there.
- **Black behind it** (Nick: "A"): nothing of the battle is left once it
  has faded. No backdrop and no sparks.
- **It takes two seconds** (Nick: "A", the short one): the battle is gone
  after 0.7 s, the logo is there from 0.6 s to 1.4 s, the words from 1.1 s
  to 1.9 s (*the split is tunable*, `game_over/glyph.rs`).
- **Any press skips it** (Nick: "A"): a key, bound or not, or a controller
  button jumps to the finished screen. That press does nothing else.
  Example: the cursor will be on `Rewind`; pressing Confirm during the
  fade shows the menu and does not rewind; the next Confirm does.
- **The menu can't be used before the end**: it isn't shown until then.
- The `defeat` music plays on through it, as before (`audio.md`); the
  animation is in time with nothing.
- *Claude's starting rules:*
  - Backing out of the rewind screen returns to Game Over as it was left,
    without the animation playing again.
  - The skip makes no sound.
  - The menu appears at once at the end; it doesn't fade in.
  - `GAME OVER` is a picture, like the logo
    (`assets-src/title/game_over.txt`), so it reads the same in every
    language until a language brings its own (ticket 0831).

### Descriptions in lists

Decided 2026-10-14 (ticket 0429). When you move the cursor over a line of
a list where you choose what a unit does, the menu box tells you what that
line does, on its own row under the list:

```
┌─ Iron Sword 20/20 ─────────────┐
│ Flowing Cut     −2 dur  E      │
│ Guard Break     −4 dur  D      │   ← the cursor is here
├────────────────────────────────┤
│ +10 hit; no counter            │
└────────────────────────────────┘
```

- The row is inside the same box, under a rule, and changes as the cursor
  moves. The name isn't repeated there: only what it does, in the normal
  (white) text colour.
- The box is as wide as its longest line or description, so it doesn't
  change size as the cursor moves.
- **Greyed-out lines can be read too:** the cursor stops on a line you
  can't use right now (`no uses left`, `out of range`, `broken`), shown as
  a dimmer bar, and its description shows. Pressing confirm on it does
  nothing. Example: the knight has spent all 3 Braces; the cursor can
  still go to `Brace  0/3  no uses left` and read `self: Def +5 Res +5`.
- The text is built from the game's own numbers (`+10 hit; no counter`,
  `self: Def +5 Res +5`), so it always matches the rules.
- Where it applies: the `Skill` menu and the `Arts` menu (0429). Later
  lists follow the same rule: the spell list, the preparations lists and
  the promotion choice.

### Places behind conversations

Decided 2026-10-15 (ticket 0029). Nick, after playing Chapter 1 (0804):

> "probably the conversations should have some kind of background imagery,
> similar to a cutscene... like even just Dace pacing a castle or
> whatever... and then flip to the conversation portraits with the
> background opacity lessened or otherwise darkened... it would set the
> mood better than portraits against black background"

He was shown real frames of the Chapter 1 conversations with the bought
portraits over: black with the place line (today); the place drawn in the
map's glyphs; the bought side-on painted backdrops (*Battlebacks*); a
top-down place laid out from the bought tilesets; the battle map itself;
and two moving openings (the lead and Hollis sparring in the yard, then the
talk over the darkened yard; Dace pacing a stone hall). His answers:

> [the look:] "probably E [a short moving scene, then the talk over the
> place, darkened], and C [a still backdrop] where E is infeasible. Glyphs
> if the player chooses it (or on playdate, I guess?) though with E, it
> seems it takes too long to get to the conversation, like conversation
> with Hollis and lead could probably start after 1/2 the current time it
> takes"
>
> [what picture:] "if the conversation is outside of battle, like we're
> using an omniscient view, then use the tileset that is appropriate. if
> it's taking place before or during or after the battle, then the battle
> tileset... this should be obvious"
>
> [when the moving part plays:] "Only where the script asks"
>
> [talk during a battle:] "The battle map, darkened"

- **Every conversation is drawn over its place, darkened.** The portraits,
  name plates, text box and place line are drawn as today; the black
  around them shows the place instead. The place is a top-down map, seen
  from above as the battle map is (the "omniscient view").
- **Which place:**
  - **Talk away from a battle:** a place laid out from the bought tilesets
    for that scene (a yard, a hall, a road), the same art and scale as the
    battle map.
  - **Talk just before, during or just after a battle:** the battle map
    itself, at the spot where the talk happens. During a battle (the
    boss's lines, death quotes, a turn's line) it is the map being played,
    darkened, with its units as they stand.
- **A short moving opening, only where the script asks.** Before the first
  line, the characters move in the place at full brightness (the lead and
  Hollis trading blows; Dace pacing alone), then the place darkens and the
  talk comes up over it. It is short: Nick found the mockup's (3 s of
  movement and 0.5 s of darkening) twice too long. Scenes the script
  doesn't ask for one open straight on the darkened place.
- **The look follows the map look.** With *Map look: Glyphs* (Options), and
  on the Playdate, whose look is the glyphs (`playdate.md`), the place is
  drawn in glyphs, as the battle map is; with *Pictures*, in the bought
  tiles. It is the same place either way.
- **No bought painted backdrops behind talk.** The *Battlebacks* stay the
  combat screen's backdrop (*Combat screen* above).
- **The place line stays** (`@caption`, today's top line: "Harrowby, the
  Thornmarch").
- **Chapter 1's places** (`docs/story/chapters/ch01.md`):

  | Scene | Place | Moving opening |
  | ----- | ----- | -------------- |
  | `ch01_intro` (the letter) | Harrowby, the yard behind the lead's rented house, dawn: a new tileset place | the lead and Hollis spar |
  | `ch01_prebattle` (the Red Company) | the battle map, at Harrowby by the chapel | none |
  | the battle's scenes | the battle map, as it stands | none |
  | `ch01_victory` (after the fight) | the battle map, at the toll-fort | none |
  | `ch01_tbc` (Veyne Hall) | Veyne Hall, the lead's father's old study, night: a new tileset place | Dace paces |

*Claude's starting rules (Nick can veto):*

- The place behind the talk is at **about 40% of its brightness**, a
  little darker than the dimmed listener's portrait, so the faces stand out
  (tunable).
- A moving opening lasts **about 1.5 seconds** of movement, then the place
  darkens over **0.5 seconds** and the first line starts (tunable). A press
  of Confirm during it skips to the first line.
- The opening's characters are their map sprites (their glyphs in the glyph
  look), walking at the battle map's walking speed.
- The intro and victory scenes of a chapter use the battle map only when
  the script names it: a scene like the letter, at dawn away from the
  field, gets its own place. In Chapter 1 that is the table above.
- A place is lit like its time of day (dawn, noon, night) with the battle
  map's lighting (ADR-0058) where the pictures have it; the glyph look has
  no lighting.
- A scene with no place named keeps today's black.
- The four scene music picks (`audio.md`, *Chapter 1's cues*) are asked
  about again once these scenes are in the game (Nick: "I want to see how
  the cutscenes would play out with this ambience").

### Dialogue log

Decided 2026-10-14 (ticket 0729). Nick, playing Chapter 1 (0804):

> "we should have a chat log, accessible via key shortcut, optional
> binding. when a conversation plays, press chat log button and able to
> scroll thru messages, hover to see the portraits and that dialogue how it
> was, click to skip back (with option to skip forward again to latest seen
> point). Also if we add VA then it will play the voice again."

He was shown three looks rendered over the Chapter 1 opening (the scene
behind showing the highlighted line with the log under it; a full-screen
list of names and text, like Fire Emblem: Three Houses' Log; a full-screen
list with each line's face) and picked **"C: full-screen with faces"**.
The other answers: replies passed on the way back are **"Asked again"**;
the log holds **"This scene"**; jumping forward: **"Both"** (pick a dim
line, or a *Back to latest* row); voices: **"also on hover with option to
disable this behavior"**; the title cinematic: **"No log there"**. The key
is in `controls.md`, *Dialogue log*.

- **The log fills the screen**, over the scene (and over the battle map
  when a scene plays during a battle). Each line the player has seen is a
  row: the speaker's **face as it was at that line** (the portrait's
  picture at 1×, with that line's expression), their name, and the text.
  Narration has no face and no name, in the dim narration colour. A reply
  the lead picked shows as the lead's line, starting `> `, with their face.
- **The cursor** starts on the line on screen and moves with the cursor
  keys; the list scrolls to keep it in view. The help under the log names
  the keys: move, jump to this line, close.
- **Confirm jumps to the highlighted line:** the log closes and the scene
  is back at that line, with the portraits, expressions and caption it had
  then, and plays on from there.
- **Jumping forward:** after a jump back, the lines seen past it stay in
  the log, dimmer. Confirm on one of them jumps forward to it. The last
  row, **Back to latest**, jumps to the furthest line seen. Example: Mia
  reads ten lines, opens the log, jumps back to line 7, reads on to
  line 8, opens the log again: lines 9 and 10 are dim, and *Back to
  latest* takes her to line 10.
- **A reply is asked again.** Jumping back to a line before a reply drops
  the reply and every line after it from the log; jumping forward goes
  only as far as the line the reply answers. Reading on, the replies come
  up again and any one can be picked. (Replies never change the story,
  `docs/story/outline.md`, so this is always safe.)
- **This scene only.** The log starts empty when a scene starts.
- **Cancel closes the log** and the scene goes on where it was. Nothing
  else closes it.
- **Voices** (when the game has them, ticket 0720): jumping to a line plays
  it as the scene always does, voice and all. **Moving the cursor onto a
  line plays its voice too**; an Options setting turns that off
  (`options.md`, *Voices in the dialogue log*).
- **No log in the title screen's cinematic**: it plays by itself.

*Claude's starting rules (Nick can veto):*

- A text box too long for one page is one row of the log, in full. The
  line still typing out when the log opens is in the log in full.
- Lines passed with *Skip scene?* aren't in the log: they were never read.
- Jumping to a line starts its text from the beginning, typing out as
  usual. The music is the music that was playing at that line.
- *Back to latest* is only there after a jump back (when the line on
  screen isn't the furthest one seen).
- The log sounds like a menu: opening it, moving the cursor and closing it
  make the menu sounds.
- While a scene plays, the text box shows the log's key next to the
  Confirm key (`b log   f ▼`). With no key, it shows nothing.

### The world map

Decided 2026-10-29, ticket 1007, over five rounds of rendered mockups
(the real font and palette; the picture look over the bought world-map
tiles, kept local). Nick's words:

> Markers: "for glyph look, symbol plates look best. make sure we also
> consider the tilemap / 2D sprite look tho. the game is not all glyph
> based anymore, it's an option. also, on glyph look, the roads/connections
> are hard to parse as-is"
>
> Army: "initials on node with flag"
>
> Roads (glyph): "mix of band + double lines, so it stands out most once
> we get to playdate I think" → "looks ok but how about having lines
> outlining the shaded road instead" → "the only problem is the roads are
> narrow when vertical and wide when horizontal" → (the same width both
> ways) "Yes, build it"
>
> Picture look: roads "Dotted"; "flag is prob unnecessary now, and pls
> render the cursor as well" → "for pixel look, flag is unnecessary, and
> it looks good"; the glyph look keeps its flag ("keep flag").

**Glyph look** (`crates/ui/src/screens/world_map/glyph.rs`):

- **Places are symbol plates:** a 16 × 16 pixel plate in the place's
  colour, centred on its cell, with its symbol on it in the panel's dark
  ink: story battle `‼` on `text_highlight` gold, side quest `♥` on
  `ally` green, town `⌂` on `wood`, capital `⌂` on `fort` gold, a won
  battle `·` on `panel_border`. A place to pass through is a plain `●` in
  `path` colour, no plate. 1008's skirmishes take the mockup's marks:
  fixed skirmish `!` on a brick red plate, random skirmish `?` on a grey
  one (*colours tunable*).
- **The level marker** `Lv7` sits on the row above a battle not yet won,
  centred, in its kind's colour (gold, green for a side quest). A won
  battle has none.
- **A place not open yet**, one road from an open one (`world-structure.md`,
  *The world map screen*): its plate in `text_dim` grey with its kind's
  symbol, no level marker; its road a band of the ground's road colour
  with a `text_dim` edge (*Claude's look, tunable*).
- **Roads** are a shaded band 12 px wide (*tunable*), the same width
  across and down, outlined by 1 px lines in `path` colour; roads that
  meet join. The ground's glyphs give way to the band. A road runs
  straight across or down between its turns (the map file gives them).
- **The army** is the lead's two letters on a `player` blue plate where
  it stands, with a small `▼` flag above it in `player` blue.
- **The cursor** is the battle map's corner marks, pulsing the same way,
  round the plate under it (round the army's when it is on the army).
- **The ground** is terrain glyphs on each terrain's background tint,
  sparser than a battle map's (forests mostly `♣`, mountains `^` here and
  there, the sea `≈` now and then), from the map file's ground.
- **Layout:** the map on the left (70 × 30 cells, centred if smaller),
  the panel on the right with the place under the cursor (its name in the
  title, the act and chapter, its kind, its level or `Won`, a town's
  shops, `Your army is here`), a list over the map when one is open
  (menu, Find, a town's shops), the help line at the bottom.

**Picture look** (`picture.rs`), for a map with a picture when the
player's map look is pictures:

- The map's own picture under everything (made from the bought World Map
  tiles, ticket 1070). Towns and castles are the art's own pictures.
- **Roads are dotted:** pale `path` dots with a dark edge, one every
  8 px; a road to a place not open yet in `text_dim` dots, the place's
  plate grey.
- **The same plates and level markers** as the glyph look, with a dark
  edge (1082 may swap the plates for an icon set).
- **The army is the lead's map sprite** (its first frame, facing down,
  at twice its size) standing on its place, **no flag**.
- **The cursor** is the battle map's picture cursor: thick marks with a
  dark edge round the plate.

### Screen layout

As in ADR-0018 and the screenshots: map on the left (single-line frame), side
panel on the right (single-line; **double-line** when a unit is selected), a
status line on top (chapter, objective, phase and turn, rewinds), and a 2-row
message and key-help bar at the bottom.

## Theme palettes for ticket 0806 (*mockup values, tunable*)

Base colours, as `bg / text / dim / highlight / panel_bg / border / focus /
player / enemy`:

| Theme | Base colours |
| ----- | ------------ |
| C Rich & painterly | `#0b1416 #b8ccc6 #5e7a78 #e8b848 #0f2024 #2e7c86 #e8b848 #6ec8f0 #e8582a` |
| D Earthy painterly | see `assets/data/palette.ron` |
| E War-table parchment | `#e8dcc0 #2a2218 #7a6a50 #8a2a10 #d8c8a4 #6a5438 #8a2a10 #1a4a9a #b01e10` |
| G Moonlit | `#06080e #f0e0c0 #7a8090 #ffb040 #0c1018 #3a4a60 #ffb040 #60b0ff #ff5030` |

Terrain glyph colour / background:

| Theme | grass | forest | water | mountain |
| ----- | ----- | ------ | ----- | -------- |
| C | `#8aa050/#1a2412` | `#3ea83a/#0e2410` | `#58c0d8/#0c3440` | `#c09868/#2c2016` |
| E | `#b0a078/#e8dcc0` | `#3a6a2a/#dcd8b0` | `#2a5a8a/#c8d4d0` | `#6a5030/#d8c8a8` |
| G | `#3a5a5a/#0a1216` | `#2e7a64/#081410` | `#4a78b0/#0a1a30` | `#6a7a8a/#10141c` |

Overlays and cursor, as `move / attack / cursor / hp_high / hp_low`:

| Theme | Values |
| ----- | ------ |
| C | `#1a5a78 #7a2c14 #f0c850 #8cd050 #e8582a` |
| E | `#a8c0e0 #e8a898 #2a2218 #3a7a2a #b01e10` |
| G | `#1c3a6a #5a1a1a #ffc860 #70d080 #ff5030` |

E is a light theme: "acted" dimming must fade toward the background, not toward
black.

## Open sub-questions

- Title screen: the sparks may be tuned after Nick has seen them in the
  game (ticket 0811).
- Selection arrows `►Al◄` next to another unit: whole-glyph arrows would hide
  one of its letters (the 0416 bracket problem). Ask Nick before 0403 draws them.
- Combat screen (ticket 0413 decided where it is shown, the backdrop, the
  motions, the sizes and the Chapter 1 recolours): which backdrop each
  terrain gets (today mountains or fields, Claude's starting rule); the
  riders, who show a plain figure until Pixel Flag's Paladin is bought
  (ticket 0446).
- Sprite units on the map: the outline alone or with a corner mark
  (0436).
- On the bought tiles (seen in ticket 0437, not decided): the picks in
  the terrain table above, the burning stand-in first.
- The battle screen's bottom two rows: the danger-zone hint on the left
  and the auto-end hint in a column with `end turn` (Nick, 2026-10-04);
  where a message goes, and what auto-end lines up with when the key help
  has no `end turn`, are for him to pick from rendered options (ticket
  0445).
