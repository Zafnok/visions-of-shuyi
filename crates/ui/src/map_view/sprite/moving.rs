// engine-friction: engine#2359, moves (game 1357 deletes it)
//! Moving tiles on the sprite skin (ticket 1315, ADR-0131): a tileset's
//! cells that loop through frames ([`trpg_content::tileset::TileFrames`])
//! drawn at the scene's clock, `clock_ms`, so a frame at a given time is
//! always the same picture.
//!
//! The kit's sprite skin paints one picture per cell and knows no frames
//! yet (engine#2359), so for each frame in which a cell moves this paints
//! with a copy of the skin whose look has each moving cell's picture
//! swapped for its frame then. Every row that draws the cell (a terrain's
//! tile, a corner piece, a tile layer's picture, a prop a map places such
//! as a lamp, ticket 1378, and the tint wash over it) shows the same
//! frame.
//!
//! A cell stands still, on its first frame, below the lowest graphics tier
//! it moves at (`ctx.platform.graphics`: the Playdate is `Reduced`) and
//! while idle motion is off (`ctx.idle_motion`: a test's frame is the same
//! a moment later).

use shuyi_ui::grid::glyph::GlyphSkin;
use shuyi_ui::grid::sprite::SpriteSkin;
use trpg_content::Tileset;
use trpg_content::tileset::TerrainTiles;

use crate::map_view::scene::MapScene;
use crate::screen::{Ctx, IdleMotion};
use crate::screens::battle::walk::SPRITE_WALK;

/// The kit's sprite skin for `scene`'s frame of `tileset`'s moving tiles,
/// with `glyph`'s colours and marks at `zoom`; `None` when no cell shows
/// another frame than its first now, and the skin as it is paints it.
pub fn frame_skin(
    ctx: &Ctx,
    scene: &MapScene,
    tileset: &Tileset,
    glyph: GlyphSkin,
    zoom: i32,
) -> Option<SpriteSkin> {
    let terrain = tileset.terrain.as_ref()?;
    let t_ms = match ctx.idle_motion {
        IdleMotion::On => scene.clock_ms,
        IdleMotion::Off => 0,
    };
    let tier = ctx.platform.graphics;
    if !tileset.frames.moves_at(tier, t_ms) {
        return None;
    }
    let look = tileset
        .frames
        .look_at(terrain.look(&scene.look.tiles), tier, t_ms);
    let tiles = TerrainTiles::new(terrain.image, terrain.tile_px, look);
    Some(SpriteSkin::new(Some(tiles), glyph, SPRITE_WALK).zoomed(zoom))
}

#[cfg(test)]
mod tests {
    use trpg_content::ImageRect;
    use trpg_content::motion::GraphicsTier;
    use trpg_content::tileset::MovingTile;
    use trpg_core::Pos;

    use crate::glyph_buffer::{GlyphBuffer, Item, PxRect, Rect};
    use crate::map_view::MapSkin;
    use crate::map_view::scene::MapScene;
    use crate::map_view::sprite::src_rect;
    use crate::map_view::sprite::tests::{blank, zoomed};
    use crate::screen::tests::ctx;
    use crate::screen::{Ctx, IdleMotion};
    use crate::settings::MapZoom;

    /// The test tileset with layers and looks, whose water flows.
    fn auto(c: &Ctx) -> MapSkin {
        MapSkin::sprite(c.game.content.tilesets["test_auto"].clone())
    }

    /// 2 × 1 tiles at the console's corner: water, then grass.
    fn scene(c: &Ctx, clock_ms: u64) -> MapScene {
        let id = |name| c.game.content.terrain.display.id_of(name);
        let mut scene = MapScene::new(Pos::new(0, 0), (2, 1));
        scene.set_terrain(|pos| if pos.x <= 0 { id("water") } else { id("plain") });
        scene.clock_ms = clock_ms;
        scene
    }

    /// The source rectangle of each picture `skin` paints on the tile
    /// whose pixels are `tile`, `clock_ms` into `c`'s clock, the map in
    /// [`AREA`] (at 1×; twice it across and down at 2×).
    fn drawn(c: &Ctx, skin: &MapSkin, clock_ms: u64, tile: PxRect) -> Vec<PxRect> {
        let mut buf: GlyphBuffer = blank();
        let zoom = skin.zoom().map_or(1, MapZoom::factor);
        let area = Rect::new(0, 0, AREA.w * zoom, AREA.h * zoom);
        skin.paint(c, &scene(c, clock_ms), area, &mut buf);
        buf.items()
            .iter()
            .filter_map(|item| match item {
                Item::Sprite(s) if s.dest == tile => Some(s.src),
                _ => None,
            })
            .collect()
    }

    /// The water's frames in the test tileset, as source rectangles.
    fn water_frames(c: &Ctx) -> Vec<PxRect> {
        let tileset = &c.game.content.tilesets["test_auto"];
        let tile = tileset.frames.iter().next().expect("the water moves");
        tile.frames.iter().map(|&(r, _)| src_rect(r)).collect()
    }

    fn moving_ctx(idle: IdleMotion, tier: GraphicsTier) -> Ctx {
        let mut c = ctx();
        c.idle_motion = idle;
        c.platform = c.platform.clone().with_graphics(tier);
        c
    }

    const WATER: PxRect = Rect::new(0, 0, 16, 16);

    /// The cells of 2 × 1 tiles of 16 px.
    const AREA: Rect = Rect::new(0, 0, 4, 1);

    #[test]
    fn the_water_shows_the_frame_for_the_scenes_clock() {
        let c = moving_ctx(IdleMotion::On, GraphicsTier::Full);
        let skin = auto(&c);
        let frames = water_frames(&c);
        assert_eq!(frames.len(), 4);
        // 600, 200, 400 and 200 ms (`FLOW_MS` of the generator).
        for (clock_ms, frame) in [(0, 0), (599, 0), (600, 1), (800, 2), (1200, 3), (1400, 0)] {
            let srcs = drawn(&c, &skin, clock_ms, WATER);
            assert_eq!(srcs.first(), Some(&frames[frame]), "{clock_ms} ms");
        }
        assert_ne!(drawn(&c, &skin, 0, WATER), drawn(&c, &skin, 600, WATER));
        // The grass beside it stands still.
        let grass = Rect::new(16, 0, 16, 16);
        assert_eq!(drawn(&c, &skin, 0, grass), drawn(&c, &skin, 600, grass));
    }

    #[test]
    fn with_idle_motion_off_the_water_shows_its_first_frame() {
        let c = moving_ctx(IdleMotion::Off, GraphicsTier::Full);
        let skin = auto(&c);
        let first = drawn(&c, &skin, 0, WATER);
        assert_eq!(first.first(), Some(&water_frames(&c)[0]));
        for clock_ms in [600, 800, 1200] {
            assert_eq!(drawn(&c, &skin, clock_ms, WATER), first, "{clock_ms} ms");
        }
    }

    /// The Playdate's tier (`Reduced`) and the lowest (`Minimal`, a GBA
    /// one day) draw every moving cell's first frame at every time: the
    /// whole frame is the one the tileset paints standing still.
    #[test]
    fn a_lesser_tier_draws_every_moving_cell_still() {
        for tier in [GraphicsTier::Reduced, GraphicsTier::Minimal] {
            let c = moving_ctx(IdleMotion::On, tier);
            let skin = auto(&c);
            let tileset = &c.game.content.tilesets["test_auto"];
            for tile in tileset.frames.iter() {
                assert_ne!(tile.motion.drawn(tier), trpg_content::motion::Drawn::Moving);
            }
            let paint = |clock_ms| {
                let mut buf = blank();
                skin.paint(&c, &scene(&c, clock_ms), AREA, &mut buf);
                buf
            };
            let still = paint(0);
            for clock_ms in [200, 600, 800, 1200] {
                assert_eq!(
                    paint(clock_ms).items(),
                    still.items(),
                    "{tier}, {clock_ms} ms"
                );
            }
        }
    }

    /// The lamp's frames: two tiles tall, from cells (4, 6), (5, 6) and
    /// (6, 6), 150 ms each (ticket 1378).
    fn lamp_frames() -> [ImageRect; 3] {
        [4, 5, 6].map(|x| ImageRect {
            x: x * 16,
            y: 96,
            w: 16,
            h: 32,
        })
    }

    /// The test tileset with a lamp a map may place, which burns.
    fn lit(c: &Ctx) -> MapSkin {
        let mut tileset = c.game.content.tilesets["test_auto"].clone();
        let terrain = tileset.terrain.as_mut().expect("test_auto has terrain");
        let [still, ..] = lamp_frames();
        terrain.look.props.insert("lamp".to_owned(), still);
        let water = tileset.frames.iter().next().expect("the water moves");
        let lamp = MovingTile {
            at: (4, 6),
            still,
            frames: lamp_frames().map(|r| (r, 150)).to_vec(),
            motion: water.motion.clone(),
        };
        tileset.frames = tileset.frames.iter().cloned().chain([lamp]).collect();
        MapSkin::sprite(tileset)
    }

    /// The source and the place of each lamp picture `skin` paints, a
    /// lamp placed on (1, 1) of 2 × 2 tiles of grass, `clock_ms` into the
    /// clock.
    fn lamps(c: &Ctx, skin: &MapSkin, clock_ms: u64) -> Vec<(PxRect, PxRect)> {
        let id = c.game.content.terrain.display.id_of("plain");
        let mut scene = MapScene::new(Pos::new(0, 0), (2, 2));
        scene.set_terrain(|_| id);
        scene.clock_ms = clock_ms;
        scene.look.props.insert((1, 1), "lamp".to_owned());
        let mut buf: GlyphBuffer = blank();
        skin.paint(c, &scene, Rect::new(0, 0, 4, 2), &mut buf);
        let frames = lamp_frames().map(src_rect);
        buf.items()
            .iter()
            .filter_map(|item| match item {
                Item::Sprite(s) if frames.contains(&s.src) => Some((s.src, s.dest)),
                _ => None,
            })
            .collect()
    }

    /// A lamp stands on its tile, its bottom on the tile's bottom, reaching
    /// a tile up, and burns through its frames by the scene's clock.
    #[test]
    fn a_placed_lamp_burns_through_its_frames_by_the_clock() {
        let c = moving_ctx(IdleMotion::On, GraphicsTier::Full);
        let skin = lit(&c);
        let frames = lamp_frames().map(src_rect);
        let standing = Rect::new(16, 0, 16, 32);
        for (clock_ms, frame) in [(0, 0), (149, 0), (150, 1), (300, 2), (450, 0)] {
            assert_eq!(
                lamps(&c, &skin, clock_ms),
                [(frames[frame], standing)],
                "{clock_ms} ms"
            );
        }
    }

    /// On the Playdate's tier, and with idle motion off, the lamp stands
    /// still on its first frame.
    #[test]
    fn a_placed_lamp_stands_still_on_a_lesser_tier() {
        let first = src_rect(lamp_frames()[0]);
        for (idle, tier) in [
            (IdleMotion::On, GraphicsTier::Reduced),
            (IdleMotion::On, GraphicsTier::Minimal),
            (IdleMotion::Off, GraphicsTier::Full),
        ] {
            let c = moving_ctx(idle, tier);
            let skin = lit(&c);
            for clock_ms in [0, 150, 300] {
                let drawn = lamps(&c, &skin, clock_ms);
                assert_eq!(drawn.len(), 1, "{tier}, {clock_ms} ms");
                assert_eq!(drawn[0].0, first, "{tier}, {clock_ms} ms");
            }
        }
    }

    #[test]
    fn the_frame_shows_at_every_zoom() {
        let c = moving_ctx(IdleMotion::On, GraphicsTier::Full);
        let skin = zoomed(auto(&c), MapZoom::X2);
        let frames = water_frames(&c);
        let water = Rect::new(0, 0, 32, 32);
        assert_eq!(drawn(&c, &skin, 600, water).first(), Some(&frames[1]));
        assert_eq!(drawn(&c, &skin, 0, water).first(), Some(&frames[0]));
    }
}
