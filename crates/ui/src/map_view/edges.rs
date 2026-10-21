//! The arrows at the edges of the map's view where the map goes on past it
//! ([`MapScene::more`], ticket 0449, `docs/design/look-and-feel.md`, *Map
//! edges*): one on each such edge, on its middle tile, pointing out, fading
//! slowly in and out. Every skin paints them over the units and under the
//! cursor (ADR-0038); this says where they go and how bright they are.

use trpg_core::Pos;

use super::scene::MapScene;

/// A side of the view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// More map to its right.
    Right,
    /// More map to its left.
    Left,
    /// More map below it.
    Down,
    /// More map above it.
    Up,
}

impl Side {
    /// Every side.
    pub const ALL: [Side; 4] = [Side::Right, Side::Left, Side::Down, Side::Up];

    /// The tile of the view of `size` tiles from `origin` its arrow is on:
    /// the middle one of the view's column or row along this side (the
    /// first of the two middle ones when there are two).
    pub fn tile(self, origin: Pos, size: (i32, i32)) -> Pos {
        let middle = Pos::new(origin.x + (size.0 - 1) / 2, origin.y + (size.1 - 1) / 2);
        match self {
            Side::Right => Pos::new(origin.x + size.0 - 1, middle.y),
            Side::Left => Pos::new(origin.x, middle.y),
            Side::Down => Pos::new(middle.x, origin.y + size.1 - 1),
            Side::Up => Pos::new(middle.x, origin.y),
        }
    }
}

/// The arrows `scene` shows: each side the map goes on past, with the
/// tile its arrow is on.
pub fn arrows(scene: &MapScene) -> impl Iterator<Item = (Side, Pos)> + '_ {
    let more = scene.more;
    let shown = [more.right(), more.left(), more.down(), more.up()];
    Side::ALL
        .into_iter()
        .zip(shown)
        .filter(|&(_, shown)| shown)
        .map(|(side, _)| (side, side.tile(scene.origin, scene.size)))
}

/// How long the arrows take to fade from bright to faint and back, in
/// milliseconds. *Tunable.*
pub const PULSE_MS: u16 = 1000;

/// How bright the arrows are at their faintest (`1` = as bright as at
/// the start). *Tunable.*
pub const FAINTEST: f32 = 0.35;

/// How bright the arrows are at `clock_ms` on the scene's clock: `1` at
/// the start of each pulse, down evenly to [`FAINTEST`] half way through
/// it and back up.
pub fn brightness(clock_ms: u64) -> f32 {
    let into = clock_ms % u64::from(PULSE_MS);
    let t = f32::from(u16::try_from(into).unwrap_or(0)) / f32::from(PULSE_MS);
    let dip = 1.0 - (2.0 * t - 1.0).abs();
    1.0 - (1.0 - FAINTEST) * dip
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use shuyi_ui::grid::{Ends, MapEdges};

    use super::*;

    fn p(x: i32, y: i32) -> Pos {
        Pos::new(x, y)
    }

    #[test]
    fn each_arrow_is_on_the_middle_tile_of_its_side() {
        let (origin, size) = (p(3, 2), (17, 15));
        assert_eq!(Side::Right.tile(origin, size), p(19, 9));
        assert_eq!(Side::Left.tile(origin, size), p(3, 9));
        assert_eq!(Side::Down.tile(origin, size), p(11, 16));
        assert_eq!(Side::Up.tile(origin, size), p(11, 2));
        // Two middle tiles: the first.
        assert_eq!(Side::Left.tile(p(0, 0), (10, 4)), p(0, 1));
        assert_eq!(Side::Up.tile(p(0, 0), (10, 4)), p(4, 0));
    }

    #[test]
    fn only_the_sides_the_map_goes_on_past_have_arrows() {
        let mut scene = MapScene::new(p(5, 0), (4, 3));
        assert_eq!(arrows(&scene).count(), 0);
        scene.more = MapEdges::of_view(scene.origin, scene.size, (10, 3));
        let sides = |s: &MapScene| arrows(s).map(|(side, _)| side).collect::<Vec<_>>();
        assert_eq!(sides(&scene), [Side::Right, Side::Left]);
        let both = Ends {
            before: true,
            after: true,
        };
        scene.more = MapEdges { x: both, y: both };
        assert_eq!(sides(&scene), Side::ALL);
        scene.more.x = Ends::default();
        let tiles: Vec<Pos> = arrows(&scene).map(|(_, tile)| tile).collect();
        assert_eq!(tiles, [p(6, 2), p(6, 0)]);
    }

    #[test]
    fn the_arrows_fade_to_faintest_half_way_and_back() {
        let near = |a: f32, b: f32| (a - b).abs() < 1e-6;
        assert!(near(brightness(0), 1.0));
        assert!(near(brightness(500), FAINTEST));
        assert!(near(brightness(250), f32::midpoint(1.0, FAINTEST)));
        assert!(near(brightness(750), brightness(250)));
        assert!(near(brightness(1000), 1.0));
        assert!(near(brightness(3500), brightness(500)));
    }

    proptest! {
        #[test]
        fn brightness_stays_between_faintest_and_full(clock_ms in any::<u64>()) {
            let b = brightness(clock_ms);
            prop_assert!((FAINTEST..=1.0).contains(&b), "{b}");
        }
    }
}
