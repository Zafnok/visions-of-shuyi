---
id: "0460"
title: "Import the thatched cottages, the throne and the moving fire Nick picked"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
blocks: ["1064"]
nick_input: setup
completed: 2026-11-27
---

# 0460 — Import the thatched cottages, the throne and the moving fire

## Context

Ticket 0471 (2026-11-22) chose, with Nick, art to replace three of the
picture look's stand-ins (`docs/design/look-and-feel.md`, *Art bought
beside Tiny Tales for the stand-ins above*):

- **House:** edermunizz, *Castle and Village*
  (<https://edermunizz.itch.io/castle-and-village-pixel-art-pack>, $9),
  its small **brown thatched cottages**, one 16×16 tile each.
- **Throne:** ArMM1998, *Tile Fantasy: Castle*
  (<https://armm1998.itch.io/tilefantasy-castle>, $5), its
  red-and-gold throne, one tile.
- **Burning:** Krishna Palacio, *Minifantasy: Raided Village*
  (<https://krishna-palacio.itch.io/minifantasy-raided-village>, $4.99):
  its **animated** flames over the burnt trees, in place of today's
  forest tinted orange. Nick picked it after seeing it move.

The gate keeps the Fortress archway (nothing better turned up).

**Status: blocked until Nick buys the three packs** (2026-11-22: "Not
yet, record it"). Ask him for the zips at the start; nothing is bought
by the agent.

No terrain tile moves today: every tile of `assets-src/tilesets/tiny_tales.ron`
is one still picture (`crates/content/src/tileset.rs`; water uses one
still frame). The house and throne are a swap of tiles; the fire needs a
terrain layer with frames, which is why this is its own ticket and not
part of 0471.

## Nick input

**Setup:** Nick buys the three packs on itch.io and says where the zips
are (e.g. his Downloads folder). Nothing else: no check after merge
(`tickets/README.md`); the moving fire goes on the Act 1 playtest's
checklist (1064) to judge by playing.

## Scope

**In:**
- Sort the three zips into the private repository's `library/` (one
  folder per pack, its licence beside it), the files the game uses into
  `game/tilesets/` (ADR-0040; `CLAUDE.md`, *Bought art*).
- Swap the house and throne in `assets-src/tilesets/tiny_tales.ron` to
  the cottage and the throne, then `cargo xtask tileset-import`.
  Pick the brown cottage closest to the 0471 mockups (a small gabled
  cottage, not the long hall). If the cottage is built from autotiles
  in the pack, take the one-tile cottage only; if none is one tile,
  stop and ask Nick (with a picture) before going further.
- The Burning terrain: the burnt trees (Trees 3, as for Burnt) with the
  Minifantasy flames over them, a few per tile, **moving**, at their
  own pixel size (the pack is drawn on an 8-pixel grid; do not scale
  the flames). Frames: the pack's own; their timing from its files.
  This needs frames for a terrain layer: extend the tileset format
  (a layer's tile may name an animation strip and a frame time) and
  let the picture skin pick the frame from the clock. Follow ADR-0038
  and ADR-0052: the look only, nothing in `core`, the scene or play
  records; the glyph and Playdate skins are untouched.
- Respect the Options' idle-motion switch (0093, `look-and-feel.md`):
  with idle motion off the flames stand still on their first frame, like
  the sprites.
- Credits (`assets/data/credits.ron`, ADR-0051): a row per pack with its
  `private` paths and its store link. Minifantasy's licence requires
  credit naming **Krishna Palacio** and asks that he be sent a link to
  the finished game: add that to the release checklist ticket's text
  (0901, or the release ticket that lists store-page tasks), not a
  message now. edermunizz (CC BY 4.0 with added terms) and ArMM1998 ask
  no credit for the paid packs; credit them anyway, as for every asset.
- `THIRD_PARTY_ASSETS.md`: the three packs, as the other bought art.
- `cargo shuyi private-assets --pin`; commit `assets-private.rev`.
- `look-and-feel.md`: the table's House, Throne and Burning rows say
  what is drawn now (no longer stand-ins).

**Out (do not do):**
- Houses on fire: not a terrain; that is 0094's question.
- A new gate.
- The packs' other art (Minifantasy's wrecked houses, corpses and
  torches; edermunizz's trees, roads and villagers; ArMM's walls): a
  later ticket if Nick wants them.
- Any change to the glyph look or the Playdate.

## Implementation steps

1. Ask Nick for the zips; unpack each into its own empty folder under
   `assets-private/library/` (they are untrusted downloads: read the
   licence files, run nothing from them).
2. Find the cottage and the throne in the sheets; cut nothing by hand
   in code: name them in `tiny_tales.ron` as `("<sheet>", <id>)` the
   way the other picks are, adding the sheets to the importer's list of
   sheets if it only reads Tiny Tales' `.tsx` files (`crates/tools/src/tileset_import.rs`).
   A sheet with no `.tsx` gets a small one in `library/` with its grid
   size, or the importer learns a plain PNG grid: choose the smaller change.
3. Swap `"house"` and `"throne"` in `tiny_tales.ron`; re-import; run the
   tileset tests (`crates/content/src/tileset.rs`).
4. Animated layer: add an optional animation to a tileset layer (frames
   in a strip, a frame time in ms) in `crates/content/src/tileset.rs`
   and the importer; the picture map skin picks `frame = (time /
   frame_ms) % frames`, each tile starting at a phase from its position
   so neighbouring tiles don't flicker in step. Expected size: about
   150–250 production lines (the format, the importer, the skin's frame
   pick), judged against the walking frames (`WALK_FRAMES`,
   `walk_frame`) that already do this for sprites.
5. `"burning"`: Burnt's trees (Trees 3) plus the flame layer, replacing
   the `tint: "fire"` Block. Two or three flames per tile, whole flames
   inside the tile (0452's rule: nothing spills onto the next tile).
6. Credits, `THIRD_PARTY_ASSETS.md`, pin, `look-and-feel.md` as in Scope.
7. `cargo shuyi frame-png` pictures for the PR (cropped to the map; no
   real map, ADR-0118): a village with the cottages, a throne room, a
   burning wood at two moments of its animation.

## Acceptance criteria

- [ ] The house and throne tiles are drawn from the bought packs;
      `tiny_tales.ron` names them; the tileset tests pass.
- [ ] Burning tiles show the Minifantasy flames moving over burnt trees,
      still when idle motion is off; a test of the skin reads two frames
      at two times and finds them different, and the same with idle
      motion off.
- [ ] Credits list the three packs (Krishna Palacio named);
      `THIRD_PARTY_ASSETS.md` lists them; `assets-private.rev` is pinned.
- [ ] `look-and-feel.md`'s table no longer calls House, Throne or Burning
      a stand-in.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the tileset format's animated layer (parse, frame pick by time,
  phase by position, idle motion off = frame 0).
- Snapshot / integration: the existing tileset and map-skin tests still
  pass with the swapped tiles; `frame-png` pictures in the PR.

## Completion notes

**Done (2026-11-27).** Nick bought the three packs that day and gave the
zips.

- **The art:** the house is edermunizz's small gabled thatched cottage,
  `PNG/City_farm_tileset.png` tile 63 (one tile); the throne ArMM1998's,
  `castle_outdoors.png`; the burning wood Trees 3's bare trees (as Burnt)
  with Minifantasy's `8x8Fire1.png` flames, three to a tile at (1, 7),
  (7, 1) and (8, 8), starting on frames 0, 3 and 6, 8 frames of 100 ms
  (the pack's `Info.txt`). The ground under a burning wood stays grass,
  as before. Sorted in `library/edermunizz/`, `library/armm1998/` and
  `library/krishna-palacio/`, each with its licence and a README.
- **The throne is two tiles tall** (Nick's answer, asked with a picture
  of three ways): the real file is 22×32 px, not one tile as 0471's
  mockups had it. Drawn whole: its seat on the throne's tile, its back on
  the tile above, the 1–3 px of its back's edge and arms' tips on the
  tiles beside (Nick: "A: whole, 2 tall"). Three `Standing` layers; the
  new `shift` moves the side ones a tile across.
- **The moving fire needed no new runtime code:** tickets 1315 and 1384
  had already made a tileset's cells move (`frames`, the skin's
  `frame_skin`, idle motion off and the Playdate's tier standing them
  still). The importer learned a `Sprites` layer instead: each frame of
  the tile is packed with every flame drawn on it at its own size, and
  the first moves through them like any moving cell. Nothing in `core`,
  the scene or play records; the glyph look and the Playdate untouched.
- **Deviation, phase by position:** the ticket asked each tile to start
  at a phase from its position. The skin swaps a moving cell's picture
  for the whole map at once (engine#2359: the sprite skin knows no frames
  per tile), so every burning tile shows the same frame. Instead each
  flame inside a tile starts on its own frame, so a tile's three flames
  never flicker in step; neighbouring tiles do move together. Left to
  judge in the Act 1 playtest (1064).
- **Two PRs move the tileset:** 1378's (game #875, lamps) is open and its
  art already merged in the private repository; this branch's art
  (Zafnok/visions-of-shuyi-assets#46, built on the game's pin) leaves it
  out. Whichever game PR merges second merges `main`, re-imports and
  re-pins (`shuyi-private-files` §5). Zafnok/visions-of-shuyi-assets#45
  was opened first from the private `main`, which carries 1380's UI frames
  that no game credit covers yet; #46 supersedes it.
- **Checked:** `frame-png` of two staged maps (never committed): a
  village with cottages beside a burning wood at two moments, and a throne
  room (`docs/screenshots/0460-*.png`, the game's screenshots).
- **Tests:** the importer's `Sprites` layer (each flame on its own frame,
  the frames moving, the checks that keep a flame inside its tile and its
  strip) and `Standing`'s `shift`; the real tileset's burning wood moves in
  every look, stands still at time 0 (idle motion off) and on the
  Playdate's tier, and the throne stands in three pieces
  (`private_assets.rs`). The skin's reading of a moving cell at two times,
  and its stillness with idle motion off, are 1315's tests
  (`map_view/sprite/moving.rs`), which the flames go through unchanged.
- **Credits and records:** three `credits.ron` rows (Krishna Palacio named,
  as his licence requires), `THIRD_PARTY_ASSETS.md`'s purchase record and
  rows, `look-and-feel.md`'s and `terrain.md`'s tables; ticket 0901 now
  says Nick sends Krishna Palacio a link to the game when its page goes
  public; 0488 notes the throne's new size.
- **Seen, not fixed:** with the art checked out,
  `private_content::without_the_content_new_game_starts_the_test_chapter`
  and `a_file_in_the_content_replaces_the_assets_file_at_its_path` fail
  (`tiny_tales.ron`'s unit `defector` is not a character without the
  content): the units part of the file, which this import writes as it
  was.

- **Friction:** estimate 130 production lines (by 1384's `moving`
  strip and 1360's `Standing`); measured 301 added (2.3×), accepted: 103
  only moved, out of three functions past clippy's 100-line limit (198
  net, 1.5×). Row in `docs/engine/friction.md`.

**No gameplay rule decided.** Left to judge in play (1064): the flames'
look and motion on a burning wood, all tiles in step; the throne's height
over the tile above it.
