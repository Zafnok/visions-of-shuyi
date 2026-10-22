# Terrain

Decided: 2026-09-26; building tiles, capturing and healing tiles 2026-10-22
Source: ticket 0301 (Nick revisited the player-facing choices of that ticket);
ticket 0037 (house, gate, throne; capturing; healing tiles)

## Nick's words

> "Idk I didn't even mention terrain defense etc so that's not really already
> mine. The four movement types I'm ok with though."
>
> **Q1. How much rough terrain slows units** (options: heavy like FE GBA /
> light like Three Houses / type-driven like Advance Wars): "1A with tuning. I
> think horses shouldn't be better in every scenario compared to foot units.
> Flyers I guess they'll be late game? Like tier 3+. We can adjust the class
> progression to make that work. But for terrain cost maybe mountains are only
> 3 on foot and maybe horses can climb mountains but it costs like 5. Rest of
> numbers seem ok."
>
> **Q2. Can foot units wade rivers** (wadeable but slow like FE7 / impassable
> / shallow ford like FFT): "2A"
>
> **Q3. What fliers can pass over** (everything but walls like FE GBA /
> slowed by some terrain like Three Houses / everything like Advance Wars):
> "3B but no weather for now just peaks"
>
> **Q4. How good terrain helps in a fight** (avoid and a bit of defence like
> FE GBA / defence only like Advance Wars / minimal like FFT): "1A"
>
> **Q5. Healing tiles** (forts, gates, thrones heal like FE GBA / only your own
> buildings like Advance Wars / none): "2 I think we can defer healing for
> now, since the idea of capturing sounds interesting, but I also don't think
> I ever explicitly signed off on a fort, gate, or throne tile type either."
>
> **Q6. Do fliers get terrain bonuses** (no like FE GBA / yes): "3A"
>
> **Q7. Building tiles** (the FE set: fort, gate, throne, village / buildings
> only later as a capturable layer like Advance Wars / just one "fort" tile
> for now): "we can go w C for now. make a note that we might allow capturing
> later and the other building type tiles might come around then or even
> without capturing. but let's keep it simple for the first playtest."

Ticket 0037 (2026-10-22, after the Chapter 1 playtest; asked with the
question pop-up, mockups of each tile next to the fort in both looks):

> **Which building tiles now** (house, gate, throne, with FE GBA's numbers):
> "House, Gate, Throne" (all three, as recommended)
>
> **Capturing:** "for act 1 I only envision seize, but capture might also
> be another option later. We can leave this part open. However, no
> benefits come from capture, it is just another way to change control
> that might be for a challenging map"
>
> **Healing tiles:** "potentially dedicated heal tiles outside of thrones
> gates and forts. Thrones gates and forts should not heal."
>
> **The looks** (first round: two-letter glyphs such as `⌂⌂`, `ΠΠ`, `ΨΨ`,
> and the bought pictures): "I prefer one tile but the current glyph you
> have for it is 2 glyphs per tile which is a bit visually confusing. Also
> imo the tileset look for the house is too modern/nice looking, these
> should be poor peasant people getting raided they need shittier houses.
> Probably need to buy additional assets for this so we can have some
> ticket scope for good tilesets for this that might bundle a few other
> gaps in our current design." … [gate:] "the archway looks better just bc
> it reminds me of double door but I don't like either that much. The
> tileset here is serviceable, it's not great but not as bad as house." …
> [throne:] "Both suck a lot... is there any other way to get a cleaner
> look or can we graft 2 fonts together into the glyph style?? Also we def
> need a new tileset to complement our existing one if all we got is a
> chair bruh"
>
> [Second round: each building drawn as one picture across the tile's two
> cells, our own glyphs:] "B: lean-to shack", "B: portcullis", "B: crowned
> throne"

## Rules

### Movement types

`foot`, `mounted`, `armored`, `flying` (Nick; the class list in
`progression.md` uses these).

### Movement costs

Move points to enter one tile; `–` = can't enter. Heavy, Fire Emblem GBA
style: terrain shapes routes (Nick, Q1). Numbers Nick set are **bold**; the
rest are FE GBA values Nick accepted ("rest of numbers seem ok"). All
*tunable*.

| Terrain | Foot | Mounted | Armored | Flying |
| ------- | ---- | ------- | ------- | ------ |
| plain, road, bridge, floor | 1 | 1 | 1 | 1 |
| forest | 2 | 3 | 2 | 1 |
| mountain | **3** | **5** | – | 1 |
| peak | – | – | – | **3** |
| fort | 2 | 2 | 2 | 1 |
| house, gate, throne | 1 | 1 | 1 | 1 |
| water (river, shallow) | 5 | – | – | 1 |
| sea | – | – | – | 1 |
| thicket, wall, door | – | – | – | – |

- **Horses are not better everywhere** (Nick): mounted units are fastest on
  open ground but pay more than foot units in forest (3 vs 2) and mountains
  (5 vs 3), and can't enter rivers.
- **Rivers** (Nick, Q2): foot units can wade at 5, which usually uses a whole
  turn; mounted and armored units can't enter. Bridges are the normal crossing.
- **Fliers** (Nick, Q3): cost 1 almost everywhere but are slowed by **peaks**
  (3, *Claude's starting value* inside Nick's "slowed by peaks"). No weather
  for now. Walls, doors and thickets still block them (*Claude's starting
  rule*, unchanged by Nick's answers).

### Terrain combat bonuses

Fire Emblem GBA style (Nick, Q4): good terrain makes a unit harder to hit
(avoid) and adds a little defence. The bonus applies to the unit standing on
the tile, whether it attacks or defends; `defense` adds to both Def and Res.
Numbers are FE GBA values, *tunable*:

| Terrain | Def | Avoid |
| ------- | --- | ----- |
| plain, road, bridge, floor | 0 | 0 |
| forest | +1 | +20 |
| mountain | +2 | +30 |
| peak | +2 | +40 |
| water, sea | 0 | +10 |
| fort | +2 | +20 |
| house | 0 | +10 |
| gate | +3 | +20 |
| throne | +3 | +30 |
| thicket, wall, door | — | — |

"—" = impassable, so it never matters in combat.

- **Fliers get no terrain Def/Avoid bonus** (Nick, Q6): they fly above the
  trees. This balances their mobility.

### Building tiles

Four building tiles (Nick, 0301 Q7 and 0037). Costs and bonuses are in
the tables above: Fire Emblem GBA's values, which Nick picked, *tunable*.

| Tile | What it is for | Glyph look | Picture look |
| ---- | -------------- | ---------- | ------------ |
| `fort` | A defensive building for bosses and chokepoints | `╦╦` battlements in `fort` gold (`look-and-feel.md`) | The grey stone tower |
| `house` | A village house to save before raiders burn it (`turn-structure.md`, *Save the village*) | **Lean-to shack** (Nick): a sagging slanted roof, a small window and a door, in `wood` on `wood_bg` | **None good yet**: the bought houses are "too modern/nice" (Nick); a stand-in until ticket 0471 finds poorer ones |
| `gate` | A castle or abbey gate in a wall; often the boss's tile, and a Seize point | **Portcullis** (Nick): a barred grate in an arch, in `fort` gold on `stone_bg` | The dungeon set's archway, "serviceable" (Nick); 0471 may find a better one |
| `throne` | The boss's seat at the end of a hall; the classic Seize point | **Crowned throne** (Nick): a wide back topped with a crown, a cushion, broad legs, in `fort` gold on `floor_bg` | **None**: the bundle has no throne (only a chair); a stand-in until 0471 |

- **One picture per tile in the glyph look** (Nick: two of the same
  letter per tile is "visually confusing"). The house, gate and throne
  are each one 16×16 drawing split over the tile's two cells: our own
  glyphs, added to the font like the gamepad's button shapes
  (`assets-src/fonts/pad-shapes.bdf`). The drawings Nick picked are in
  ticket 0322. They never look like the cursor (`look-and-feel.md`).
- **A house is one tile.** Its tile is where a unit stands to `Evacuate`
  it and what a raider burns (`turn-structure.md`, *Save the village*);
  in that rule, "the house's door" means this tile. Houses appear only on
  maps whose story is about a village (0069).
- **Gate and throne** are where bosses stand and where a `Seize`
  objective points (Chapter 3's scriptorium may be a throne or another
  tile; the chapter ticket picks). Standing on one has no other effect.

### Capturing

**Not in Act 1** (Nick): Act 1 uses only Fire Emblem's **Seize** (a
`Seize((x, y))` objective on a tile, usually a gate or a throne). Capturing may come later, as an option for "a challenging map",
and then **it brings no benefits**: no income, no healing, no supplies.
It only changes who controls the building (Nick: "just another way to
change control"). How it works is open until a map needs it.

### Healing tiles

**Forts, gates and thrones do not heal** (Nick), unlike Fire Emblem GBA.
No tile heals in Act 1: `heal_percent` stays 0 on every terrain.

Nick may want **dedicated healing tiles** later (Nick: "potentially
dedicated heal tiles outside of thrones gates and forts"), a tile whose
only job is to heal. Open until a map needs one.

## Open sub-questions

- **Fliers as late-game classes** (Nick: "tier 3+"): decided in ticket 0017.
  The flying line now starts at tier 3 (`progression.md`).
- **Building tiles, capturing and healing tiles:** decided in ticket 0037
  (above). Still open, for a later map that needs them: how capturing
  works (it gives no benefits), and dedicated healing tiles.
- **Bought pictures for the house and the throne** (and a better gate):
  ticket 0471 finds a tileset with poorer houses and a throne, with other
  gaps of the picture look.
- Weather (Nick: "no weather for now").
