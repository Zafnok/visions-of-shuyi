//! Moving tiles (ticket 1315, ADR-0131): a cell of a tileset's image that
//! loops through frames, each shown for its own time. Every row of the
//! tileset that draws the cell (a terrain's own tile, a corner layer's
//! mix, a tile layer's picture, in any look) moves with it, so the rows
//! themselves stay as they are written.
//!
//! The frames are drawn from the time alone (the `shuyi-animate` skill):
//! the picture for a time is [`MovingTile::frame_at`], never stepped. Each
//! moving cell says how it shows on a lesser graphics tier
//! ([`Motion`]): the check in the content test reads them all
//! ([`moving_rows`]).
//!
//! As written, in a tileset beside `terrain` and `layers`:
//!
//! ```ron
//! frames: [
//!     // The cell, then its frames: each a cell and its time in ms. The
//!     // first frame is the cell itself, what stands still.
//!     (at: (3, 1), frames: [((3, 1), 600), ((4, 1), 200), ((5, 1), 400)],
//!      motion: (stills: {
//!          Reduced: (show: FirstFrame),
//!          Minimal: (show: FirstFrame, todo: "a one-bit tile"),
//!      })),
//! ],
//! ```
use crate::prelude::*;

use alloc::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use super::{Checker, ImageRect, Layer, Look, TerrainTiles, Tileset, grid_rect};
use crate::motion::{GraphicsTier, Motion, Show};

/// A moving cell as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FramesFile {
    at: (u32, u32),
    frames: Vec<((u32, u32), u32)>,
    motion: Motion,
}

/// A rectangle as a key: its left, top, width and height.
type Key = [u32; 4];

const fn key(r: ImageRect) -> Key {
    [r.x, r.y, r.w, r.h]
}

/// One moving cell: its frames and how it shows on each tier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovingTile {
    /// The cell, as written.
    pub at: (u32, u32),
    /// Its rectangle: what stands still, and its first frame.
    pub still: ImageRect,
    /// Each frame's rectangle and how long it shows, in ms (at least 1),
    /// the first the cell itself.
    pub frames: Vec<(ImageRect, u32)>,
    /// Its stills on the tiers below the lowest it moves at.
    pub motion: Motion,
}

impl MovingTile {
    /// The frame shown `t_ms` into the loop's first turn (the loop repeats
    /// for ever).
    pub fn frame_at(&self, t_ms: u64) -> ImageRect {
        let cycle: u64 = self.frames.iter().map(|&(_, ms)| u64::from(ms)).sum();
        let mut into = t_ms.checked_rem(cycle).unwrap_or(0);
        for &(rect, ms) in &self.frames {
            match into.checked_sub(u64::from(ms)) {
                Some(left) => into = left,
                None => return rect,
            }
        }
        self.still
    }
}

/// A tileset's moving cells, by the rectangle that stands still.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TileFrames(BTreeMap<Key, MovingTile>);

impl TileFrames {
    /// Whether nothing moves.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Every moving cell.
    pub fn iter(&self) -> impl Iterator<Item = &MovingTile> {
        self.0.values()
    }

    /// The picture drawn for `still` `t_ms` into the scene's clock on
    /// `tier`: its frame then if it moves there, else `still` itself (its
    /// first frame).
    pub fn picture(&self, still: ImageRect, tier: GraphicsTier, t_ms: u64) -> ImageRect {
        match self.0.get(&key(still)) {
            Some(tile) if tier.plays(tile.motion.lowest) => tile.frame_at(t_ms),
            _ => still,
        }
    }

    /// Whether a cell shows another frame than its first `t_ms` into the
    /// clock on `tier`.
    pub fn moves_at(&self, tier: GraphicsTier, t_ms: u64) -> bool {
        self.iter()
            .any(|tile| self.picture(tile.still, tier, t_ms) != tile.still)
    }

    /// `look` with each picture as [`picture`](Self::picture) says.
    pub fn look_at(&self, look: &Look, tier: GraphicsTier, t_ms: u64) -> Look {
        let at = |r: ImageRect| self.picture(r, tier, t_ms);
        let mut out = look.clone();
        for rect in out.tiles.values_mut() {
            *rect = at(*rect);
        }
        for layer in &mut out.layers {
            match layer {
                Layer::Corners(c) => c.tiles.iter_mut().flatten().for_each(|r| *r = at(*r)),
                Layer::Tiles(t) => {
                    t.tile = at(t.tile);
                    t.sides.iter_mut().flatten().for_each(|r| *r = at(*r));
                }
            }
        }
        out
    }
}

/// Every rectangle a look of `terrain` draws.
fn drawn(terrain: &TerrainTiles) -> BTreeSet<Key> {
    let mut out = BTreeSet::new();
    for look in core::iter::once(&terrain.look).chain(terrain.looks.values()) {
        out.extend(look.tiles.values().map(|&r| key(r)));
        for layer in &look.layers {
            match layer {
                Layer::Corners(c) => out.extend(c.tiles.iter().flatten().map(|&r| key(r))),
                Layer::Tiles(t) => {
                    out.insert(key(t.tile));
                    out.extend(t.sides.iter().flatten().map(|&r| key(r)));
                }
            }
        }
    }
    out
}

/// The moving cells `files` write, checked (problems into `check`)
/// against the terrain tiles they move; `tile_px` is a cell's size.
pub(super) fn read(
    check: &mut Checker,
    files: &[FramesFile],
    terrain: Option<&TerrainTiles>,
    tile_px: (u32, u32),
) -> TileFrames {
    let mut out = BTreeMap::new();
    let Some(terrain) = terrain else {
        if !files.is_empty() {
            check
                .problems
                .push("frames need `terrain`: they move its tiles".to_owned());
        }
        return TileFrames::default();
    };
    let drawn = drawn(terrain);
    for file in files {
        let (x, y) = file.at;
        let what = format!("frames of cell ({x}, {y})");
        let rect = |at| grid_rect(at, tile_px, (0, 0));
        let still = rect(file.at);
        if out.contains_key(&key(still)) {
            check
                .problems
                .push(format!("{what}: the cell is listed twice"));
            continue;
        }
        if !drawn.contains(&key(still)) {
            let problem = format!("{what}: no terrain or layer draws the cell");
            check.problems.push(problem);
        }
        if file.frames.len() < 2 {
            let problem = format!("{what}: a moving tile needs two frames or more");
            check.problems.push(problem);
        }
        if file.frames.first().is_some_and(|&(at, _)| at != file.at) {
            check.problems.push(format!(
                "{what}: the first frame must be the cell itself, ({x}, {y}): it is what stands still"
            ));
        }
        if file.frames.iter().any(|&(_, ms)| ms == 0) {
            check.problems.push(format!("{what}: a frame lasts 0 ms"));
        }
        if file.motion.stills.values().any(|s| s.show == Show::Nothing) {
            check.problems.push(format!(
                "{what}: a tile's still is its first frame (`show: FirstFrame`); `Nothing` would \
                 leave a hole in the map"
            ));
        }
        let frames: Vec<(ImageRect, u32)> =
            file.frames.iter().map(|&(at, ms)| (rect(at), ms)).collect();
        for &(r, _) in &frames {
            check.inside(&what, terrain.image, r);
        }
        let tile = MovingTile {
            at: file.at,
            still,
            frames,
            motion: file.motion.clone(),
        };
        out.insert(key(still), tile);
    }
    TileFrames(out)
}

/// Every moving cell of `tilesets`, each named as the check's messages show
/// it (`tileset "test_auto", cell (3, 1)`), and every look's trees
/// (`tileset "tiny_tales", look "rogue_adventure"'s trees`, ticket 1356),
/// with its motion: what [`motion::check`](crate::motion::check) reads.
pub fn moving_rows<'a>(
    tilesets: impl IntoIterator<Item = &'a Tileset>,
) -> Vec<(String, &'a Motion)> {
    tilesets
        .into_iter()
        .flat_map(|tileset| {
            let cells = tileset.frames.iter().map(move |tile| {
                let (x, y) = tile.at;
                let name = format!("tileset \"{}\", cell ({x}, {y})", tileset.id);
                (name, &tile.motion)
            });
            let trees = tileset.trees.iter().map(move |(look, trees)| {
                let name = format!("tileset \"{}\", look \"{look}\"'s trees", tileset.id);
                (name, &trees.motion)
            });
            cells.chain(trees)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::tests::{Fixture, file};
    use super::*;
    use crate::motion::GraphicsTier::{Full, Minimal, Reduced};

    const STILLS: &str =
        "motion: (stills: {Reduced: (show: FirstFrame), Minimal: (show: FirstFrame)})";

    /// Every terrain on (0, 0), the water on (1, 0), and `frames`.
    fn moving(f: &Fixture, frames: &str) -> String {
        let terrain =
            format!("{{ {} }}", f.terrain("")).replace("\"water\": (0, 0)", "\"water\": (1, 0)");
        file(f, &[("terrain", &terrain), ("frames", frames)])
    }

    fn cell(x: u32, y: u32) -> ImageRect {
        grid_rect((x, y), (16, 16), (0, 0))
    }

    #[test]
    fn a_cell_shows_each_frame_for_its_time_and_loops() {
        let f = Fixture::new();
        let source = moving(
            &f,
            &format!(
                "[(at: (1, 0), frames: [((1, 0), 600), ((2, 0), 200), ((3, 0), 400)], {STILLS})]"
            ),
        );
        let tileset = f.parse(&source).unwrap();
        let tile = tileset.frames.iter().next().unwrap();
        assert_eq!(tile.at, (1, 0));
        let at = |t| tile.frame_at(t);
        assert_eq!(
            [at(0), at(599), at(600), at(799), at(800), at(1199)],
            [
                cell(1, 0),
                cell(1, 0),
                cell(2, 0),
                cell(2, 0),
                cell(3, 0),
                cell(3, 0)
            ]
        );
        assert_eq!(
            [at(1200), at(1800), at(u64::MAX)],
            [cell(1, 0), cell(2, 0), at(u64::MAX % 1200)]
        );
        assert_eq!(tile.still, cell(1, 0));
    }

    #[test]
    fn a_lesser_tier_and_a_still_cell_show_the_first_frame() {
        let f = Fixture::new();
        let source = moving(
            &f,
            &format!("[(at: (1, 0), frames: [((1, 0), 100), ((2, 0), 100)], {STILLS})]"),
        );
        let frames = f.parse(&source).unwrap().frames;
        assert_eq!(frames.picture(cell(1, 0), Full, 150), cell(2, 0));
        for tier in [Reduced, Minimal] {
            assert_eq!(frames.picture(cell(1, 0), tier, 150), cell(1, 0), "{tier}");
        }
        // A cell with no frames stands still everywhere.
        assert_eq!(frames.picture(cell(0, 0), Full, 150), cell(0, 0));
        let cheap = moving(
            &f,
            "[(at: (1, 0), frames: [((1, 0), 100), ((2, 0), 100)], \
              motion: (lowest: Reduced, stills: {Minimal: (show: FirstFrame)}))]",
        );
        let frames = f.parse(&cheap).unwrap().frames;
        assert_eq!(frames.picture(cell(1, 0), Reduced, 150), cell(2, 0));
        assert_eq!(frames.picture(cell(1, 0), Minimal, 150), cell(1, 0));
    }

    /// Whether anything shows another frame than its first: what lets the
    /// skin paint as it is when nothing moves.
    #[test]
    fn a_cell_moves_only_on_its_tiers_and_off_its_first_frame() {
        let f = Fixture::new();
        let source = moving(
            &f,
            &format!("[(at: (1, 0), frames: [((1, 0), 100), ((2, 0), 100)], {STILLS})]"),
        );
        let frames = f.parse(&source).unwrap().frames;
        assert!(!frames.is_empty());
        assert!(frames.moves_at(Full, 150));
        // On its first frame, and on a tier where it stands still.
        assert!(!frames.moves_at(Full, 50));
        assert!(!frames.moves_at(Full, 200));
        assert!(!frames.moves_at(Reduced, 150));
        assert!(!frames.moves_at(Minimal, 150));
        let still = f.parse(&file(&f, &[])).unwrap().frames;
        assert!(still.is_empty() && !still.moves_at(Full, 150));
    }

    /// A tileset whose image is missing says so once: its moving cells
    /// aren't checked against tiles it doesn't have.
    #[test]
    fn a_missing_image_is_the_one_problem_of_its_moving_cells() {
        let f = Fixture::new();
        let frames = format!("[(at: (1, 0), frames: [((1, 0), 100), ((2, 0), 100)], {STILLS})]");
        let source = file(
            &f,
            &[
                ("image", "\"tilesets/gone.png\""),
                (
                    "units",
                    "(characters: {}, classes: {}, fallback: (image: \"units/a.png\", frame: (0, 0)))",
                ),
                ("frames", &frames),
            ],
        );
        assert_eq!(
            f.errors(&source),
            ["image \"tilesets/gone.png\" is not a PNG in the asset bundle"]
        );
    }

    #[test]
    fn a_look_at_a_time_moves_every_row_that_draws_the_cell() {
        let f = Fixture::new();
        let layers = "[Corners(of: [\"water\"], tiles: [(corners: \"...#\", at: (1, 0))]), \
                       Tiles(of: [\"bridge\"], at: (1, 0), beside: [\"water\"], \
                             sides: [(sides: \".#.#\", at: (1, 0))])]";
        let terrain =
            format!("{{ {} }}", f.terrain("")).replace("\"water\": (0, 0)", "\"water\": (1, 0)");
        let frames = format!("[(at: (1, 0), frames: [((1, 0), 100), ((2, 0), 100)], {STILLS})]");
        let source = file(
            &f,
            &[
                ("terrain", &terrain),
                ("layers", layers),
                ("frames", &frames),
            ],
        );
        let tileset = f.parse(&source).unwrap();
        let look = &tileset.terrain.as_ref().unwrap().look;
        let water = f.content.terrain.display.id_of("water").unwrap();
        let moved = tileset.frames.look_at(look, Full, 100);
        assert_eq!(moved.tiles[&water], cell(2, 0));
        let plain = f.content.terrain.display.id_of("plain").unwrap();
        assert_eq!(moved.tiles[&plain], cell(0, 0));
        let Layer::Corners(shore) = &moved.layers[0] else {
            panic!("{:?}", moved.layers[0]);
        };
        assert_eq!(shore.tiles[1], Some(cell(2, 0)));
        let Layer::Tiles(bridge) = &moved.layers[1] else {
            panic!("{:?}", moved.layers[1]);
        };
        assert_eq!(
            (bridge.tile, bridge.sides[0b0101]),
            (cell(2, 0), Some(cell(2, 0)))
        );
        // At time 0, and on a lesser tier, the look is as written.
        assert_eq!(&tileset.frames.look_at(look, Full, 0), look);
        assert_eq!(&tileset.frames.look_at(look, Reduced, 100), look);
    }

    #[test]
    fn the_rows_are_named_for_the_stills_check() {
        let f = Fixture::new();
        let source = moving(
            &f,
            "[(at: (1, 0), frames: [((1, 0), 100), ((2, 0), 100)], motion: ())]",
        );
        let tileset = f.parse(&source).unwrap();
        let rows = moving_rows([&tileset]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "tileset \"t\", cell (1, 0)");
        let report = crate::motion::check(rows);
        assert_eq!(
            report.problems,
            [
                "tileset \"t\", cell (1, 0): no still for Minimal, where it doesn't move (it moves from Full)",
                "tileset \"t\", cell (1, 0): no still for Reduced, where it doesn't move (it moves from Full)",
            ]
        );
        let still = f.parse(&file(&f, &[])).unwrap();
        assert!(still.frames.is_empty() && moving_rows([&still]).is_empty());
    }

    #[test]
    fn a_moving_cell_is_drawn_has_two_frames_from_itself_and_lies_inside() {
        let f = Fixture::new();
        let cases = [
            (
                "[(at: (3, 0), frames: [((3, 0), 100), ((2, 0), 100)], STILLS)]",
                "frames of cell (3, 0): no terrain or layer draws the cell",
            ),
            (
                "[(at: (1, 0), frames: [((1, 0), 100)], STILLS)]",
                "frames of cell (1, 0): a moving tile needs two frames or more",
            ),
            (
                "[(at: (1, 0), frames: [((2, 0), 100), ((1, 0), 100)], STILLS)]",
                "frames of cell (1, 0): the first frame must be the cell itself, (1, 0): it is what stands still",
            ),
            (
                "[(at: (1, 0), frames: [((1, 0), 100), ((2, 0), 0)], STILLS)]",
                "frames of cell (1, 0): a frame lasts 0 ms",
            ),
            (
                "[(at: (1, 0), frames: [((1, 0), 100), ((9, 0), 100)], STILLS)]",
                "frames of cell (1, 0): its 16×16 px at (144, 0) lies outside the 64×64 px image",
            ),
            (
                "[(at: (1, 0), frames: [((1, 0), 100), ((2, 0), 100)], STILLS), \
                  (at: (1, 0), frames: [((1, 0), 100), ((3, 0), 100)], STILLS)]",
                "frames of cell (1, 0): the cell is listed twice",
            ),
            (
                "[(at: (1, 0), frames: [((1, 0), 100), ((2, 0), 100)], \
                   motion: (stills: {Reduced: (show: Nothing), Minimal: (show: FirstFrame)}))]",
                "frames of cell (1, 0): a tile's still is its first frame (`show: FirstFrame`); \
                 `Nothing` would leave a hole in the map",
            ),
        ];
        for (frames, problem) in cases {
            let frames = frames.replace("STILLS", STILLS);
            assert_eq!(f.errors(&moving(&f, &frames)), [problem], "{frames}");
        }
        let units_only = file(
            &f,
            &[
                ("tile_px", ""),
                ("terrain", ""),
                (
                    "frames",
                    &format!("[(at: (1, 0), frames: [((1, 0), 100), ((2, 0), 100)], {STILLS})]"),
                ),
            ],
        );
        assert_eq!(
            f.errors(&units_only),
            ["frames need `terrain`: they move its tiles"]
        );
    }
}
