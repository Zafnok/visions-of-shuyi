//! How units move on the map (`docs/design/look-and-feel.md`, *Battle map:
//! bought tiles and unit sprites*; ticket 0440): a unit that can still act
//! steps on the spot, and a moving unit glides from tile to tile along its
//! path, its legs going, turned the way it walks. The timing is the tactics
//! kit's (`shuyi_ui::grid::walk`, ticket 1115); the speeds are this game's,
//! each map skin's setting ([`GLYPH_WALK`], [`SPRITE_WALK`]): glyph units
//! jump from tile to tile quickly, sprites take the time to walk.
//!
//! All of it is a look (ADR-0038): the rules, the camera and the sounds go
//! by the tile a walk is on, which changes at each tile's edge as it always
//! has.

use shuyi_ui::grid::skin::WalkSpeeds;

pub use shuyi_ui::grid::walk::{
    Gait, IDLE_FRAMES, IDLE_STEP_MS, WALK_FRAME_S, WALK_FRAMES, facing, gait, idle_frame, steps,
};

/// Walking speed under the glyph skin (and any skin that doesn't say), in
/// tiles per second. *Tunable.*
pub const WALK_TILES_PER_S: f32 = 12.0;

/// An AI unit's walking speed under the glyph skin while Confirm is held,
/// in tiles per second: four times as fast, as the rest of its action.
pub const HELD_WALK_TILES_PER_S: f32 = 4.0 * WALK_TILES_PER_S;

/// Walking speed under a sprite skin, in tiles per second (Nick,
/// 2026-10-04: 5 was too slow). *Tunable.*
pub const SPRITE_WALK_TILES_PER_S: f32 = 6.0;

/// Walking speed under a sprite skin with the Options screen's Fast
/// animations (Nick, 2026-10-04, `docs/design/look-and-feel.md`), in tiles
/// per second. *Tunable.*
pub const SPRITE_FAST_WALK_TILES_PER_S: f32 = 8.0;

/// An AI unit's walking speed under a sprite skin while Confirm is held,
/// in tiles per second: the fastest a sprite ever walks (Nick, 2026-10-04:
/// "it's just 12 max").
pub const SPRITE_HELD_WALK_TILES_PER_S: f32 = 12.0;

/// The glyph skin's walks: the same with Fast animations.
pub const GLYPH_WALK: WalkSpeeds = WalkSpeeds {
    normal: WALK_TILES_PER_S,
    fast: WALK_TILES_PER_S,
    held: HELD_WALK_TILES_PER_S,
};

/// A sprite skin's walks.
pub const SPRITE_WALK: WalkSpeeds = WalkSpeeds {
    normal: SPRITE_WALK_TILES_PER_S,
    fast: SPRITE_FAST_WALK_TILES_PER_S,
    held: SPRITE_HELD_WALK_TILES_PER_S,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sprite_walks_slower_than_a_glyph_and_12_tiles_a_second_at_most() {
        // A sprite 6 tiles a second, 8 with Fast animations, 12 at the
        // most; a glyph unit 12, and four times that with Confirm held.
        assert_eq!(
            SPRITE_WALK,
            WalkSpeeds {
                normal: 6.0,
                fast: 8.0,
                held: 12.0
            }
        );
        assert_eq!(
            GLYPH_WALK,
            WalkSpeeds {
                normal: 12.0,
                fast: 12.0,
                held: 48.0
            }
        );
        assert_eq!(GLYPH_WALK, WalkSpeeds::of(WALK_TILES_PER_S));
        assert!((HELD_WALK_TILES_PER_S - 48.0).abs() < 1e-6);
    }
}
