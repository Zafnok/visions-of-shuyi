//! Trees by footprint (ticket 1356, `docs/design/look-and-feel.md`,
//! *Woods: trees by footprint*): pictures that stand on the forest of a
//! look, each bigger than a tile, looping through drawn frames.
//!
//! Where they stand is a pure function of the map's terrain ([`place`]):
//! a **big** tree wherever a 3×3 of forest is free (its trunk on the
//! bottom row's middle tile, its crown over the tiles to its left and
//! right and the two above), then a **half-size** one on each forest tile
//! left (its crown over the tile above). Which picture a tree is and how
//! far into its loop it starts come from its place, never a random
//! number, so a forest doesn't sway as one ([`Trees::picture`]).
//!
//! As written, in a look of a tileset's `looks`:
//!
//! ```ron
//! trees: (
//!     of: ["forest"],
//!     // Each kind: a frame's size in pixels, the pixel of a frame that
//!     // stands on the middle of its trunk tile's bottom edge, and the
//!     // frames: the top-left cell of each and its time in ms.
//!     big: [(size: (64, 80), foot: (32, 74), frames: [((0, 40), 160), ((4, 40), 160)])],
//!     half: [(size: (32, 40), foot: (16, 37), frames: [((0, 60), 160), ((2, 60), 160)])],
//!     motion: (stills: {Reduced: (show: FirstFrame), Minimal: (show: FirstFrame)}),
//! ),
//! ```
use crate::prelude::*;

use alloc::collections::BTreeSet;

use serde::Deserialize;
use trpg_core::{Pos, TerrainId};

use super::ImageRect;
use super::look::Looking;
use crate::motion::{GraphicsTier, Motion, Show};

/// A look's trees as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TreesFile {
    of: Vec<String>,
    big: Vec<KindFile>,
    half: Vec<KindFile>,
    motion: Motion,
}

/// A kind of tree as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct KindFile {
    size: (u32, u32),
    foot: (u32, u32),
    frames: Vec<((u32, u32), u32)>,
}

/// A look's trees: the terrains they stand on, their pictures and how they
/// show on a lesser graphics tier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trees {
    /// The terrains a tree stands on: the forest.
    pub of: BTreeSet<TerrainId>,
    /// The kinds of big tree (3×3 tiles); each tree is one of them, by its
    /// place.
    pub big: Vec<TreeKind>,
    /// The kinds of half-size tree (one tile wide, two tall).
    pub half: Vec<TreeKind>,
    /// Its stills on the tiers below the lowest it moves at.
    pub motion: Motion,
}

/// One kind of tree: its frames and where each stands on its trunk tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeKind {
    /// The pixel of a frame that stands on the middle of the trunk tile's
    /// bottom edge.
    pub foot: (u32, u32),
    /// Each frame's rectangle and how long it shows, in ms (at least 1).
    pub frames: Vec<(ImageRect, u32)>,
}

impl TreeKind {
    /// The frame shown `t_ms` into the loop's first turn (the loop repeats
    /// for ever).
    pub fn frame_at(&self, t_ms: u64) -> ImageRect {
        let first = self.frames.first().map(|&(rect, _)| rect);
        let still = first.unwrap_or(ImageRect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        });
        let cycle = self.cycle_ms();
        let mut into = t_ms.checked_rem(cycle).unwrap_or(0);
        for &(rect, ms) in &self.frames {
            match into.checked_sub(u64::from(ms)) {
                Some(left) => into = left,
                None => return rect,
            }
        }
        still
    }

    /// How long its loop lasts, in ms.
    pub fn cycle_ms(&self) -> u64 {
        self.frames.iter().map(|&(_, ms)| u64::from(ms)).sum()
    }
}

/// How much of the forest a tree stands on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Footprint {
    /// 3×3 tiles: its trunk's, the ones to its left and right and the two
    /// rows above them.
    Big,
    /// One tile wide and two tall: its trunk's and the one above.
    Half,
}

/// A tree where it stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tree {
    /// The tile its trunk stands on.
    pub trunk: Pos,
    /// How much of the forest it stands on.
    pub footprint: Footprint,
}

/// Where the trees of a `width` × `height` map whose forest tiles are
/// those `forest` says stand, the same on every run: a big tree wherever a
/// 3×3 of forest is free, row by row from the top-left; then, from the
/// bottom row up, a half-size tree on each forest tile left, its crown
/// taking the tile above if that is forest left too. A lone tile (no
/// forest left above it) gets a half-size tree all the same, its crown
/// over the tile above. Listed from the top row down and left to right,
/// the order they are painted in (a lower tree over a higher one).
pub fn place(width: i32, height: i32, forest: impl Fn(Pos) -> bool) -> Vec<Tree> {
    let (w, h) = (width.max(0), height.max(0));
    let index = |p: Pos| usize::try_from(p.y * w + p.x).ok();
    let mut taken = vec![false; usize::try_from(w * h).unwrap_or(0)];
    let free = |taken: &[bool], p: Pos| {
        let on_map = (0..w).contains(&p.x) && (0..h).contains(&p.y);
        on_map && forest(p) && index(p).is_some_and(|i| !taken[i])
    };
    let mut trees = Vec::new();
    // A footprint off the map is never free, so every tile is tried.
    for y in 0..h {
        for x in 0..w {
            let footprint =
                (y - 2..=y).flat_map(|fy| (x - 1..=x + 1).map(move |fx| Pos::new(fx, fy)));
            if footprint.clone().all(|p| free(&taken, p)) {
                for i in footprint.filter_map(index) {
                    taken[i] = true;
                }
                let trunk = Pos::new(x, y);
                trees.push(Tree {
                    trunk,
                    footprint: Footprint::Big,
                });
            }
        }
    }
    for y in (0..h).rev() {
        for x in 0..w {
            let trunk = Pos::new(x, y);
            if !free(&taken, trunk) {
                continue;
            }
            let crown = Pos::new(x, y - 1);
            for p in [trunk, crown] {
                if let Some(i) = index(p).filter(|_| free(&taken, p)) {
                    taken[i] = true;
                }
            }
            trees.push(Tree {
                trunk,
                footprint: Footprint::Half,
            });
        }
    }
    trees.sort_by_key(|t| (t.trunk.y, t.trunk.x));
    trees
}

/// What a number from a tree's place picks: each a number of its own, so
/// a tree's kind doesn't follow from its phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pick {
    /// Which of its footprint's kinds it is.
    Kind = 1,
    /// How far into its loop it starts.
    Phase = 2,
}

/// A number from `trunk`'s place for `pick`, the same on every run: a
/// tree's look varies over a forest without a random number (nothing
/// secret: a scramble of the coordinates, not cryptography).
fn mixed(trunk: Pos, pick: Pick) -> u32 {
    #[allow(clippy::cast_sign_loss)]
    let (x, y) = (trunk.x as u32, trunk.y as u32);
    let mut h = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77) ^ pick as u32;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h
}

impl Trees {
    /// The kind tree `tree` is: one of its footprint's kinds, by its
    /// place. `None` if the look has no kind of that footprint.
    pub fn kind(&self, tree: Tree) -> Option<&TreeKind> {
        let kinds = match tree.footprint {
            Footprint::Big => &self.big,
            Footprint::Half => &self.half,
        };
        let len = u32::try_from(kinds.len()).ok().filter(|&n| n > 0)?;
        kinds.get(usize::try_from(mixed(tree.trunk, Pick::Kind) % len).ok()?)
    }

    /// How far into its loop tree `tree` of kind `kind` is when the
    /// scene's clock is at 0, in ms: from its place.
    pub fn phase_ms(tree: Tree, kind: &TreeKind) -> u64 {
        let cycle = kind.cycle_ms();
        u64::from(mixed(tree.trunk, Pick::Phase))
            .checked_rem(cycle)
            .unwrap_or(0)
    }

    /// The frame of `tree` drawn `t_ms` into the scene's clock on `tier`:
    /// its frame then, from its own phase, if it moves there; else (or
    /// with no clock, `None`: idle motion off) its first frame. `None` if
    /// the look has no kind of its footprint.
    pub fn picture(
        &self,
        tree: Tree,
        tier: GraphicsTier,
        t_ms: Option<u64>,
    ) -> Option<(&TreeKind, ImageRect)> {
        let kind = self.kind(tree)?;
        let frame = match t_ms {
            Some(t) if tier.plays(self.motion.lowest) => {
                kind.frame_at(t.saturating_add(Self::phase_ms(tree, kind)))
            }
            _ => kind.frame_at(0),
        };
        Some((kind, frame))
    }

    /// Where its trees stand on a `width` × `height` map whose terrain is
    /// `terrain` ([`place`]).
    pub fn place(
        &self,
        width: i32,
        height: i32,
        terrain: impl Fn(Pos) -> Option<TerrainId>,
    ) -> Vec<Tree> {
        place(width, height, |p| {
            terrain(p).is_some_and(|id| self.of.contains(&id))
        })
    }
}

impl Looking<'_, '_> {
    /// The trees `file` writes, checked.
    pub(super) fn trees(&mut self, file: &TreesFile) -> Trees {
        let what = format!("{}trees", super::look::prefix(self.name));
        if file.of.is_empty() {
            let problem = format!("{what}: `of` names no terrain");
            self.check.problems.push(problem);
        }
        let of = self.terrains(&what, &file.of);
        if file.motion.stills.values().any(|s| s.show == Show::Nothing) {
            self.check.problems.push(format!(
                "{what}: a tree's still is its first frame (`show: FirstFrame`); `Nothing` would \
                 leave the forest bare"
            ));
        }
        let mut kinds = |name: &str, files: &[KindFile]| {
            if files.is_empty() {
                let problem = format!("{what}: `{name}` has no kind of tree");
                self.check.problems.push(problem);
            }
            (1..)
                .zip(files)
                .map(|(number, file)| self.tree_kind(&format!("{what}, {name} {number}"), file))
                .collect()
        };
        let big = kinds("big", &file.big);
        let half = kinds("half", &file.half);
        Trees {
            of,
            big,
            half,
            motion: file.motion.clone(),
        }
    }

    /// The kind of tree `file` writes, checked; `what` names it.
    fn tree_kind(&mut self, what: &str, file: &KindFile) -> TreeKind {
        if file.size.0 == 0 || file.size.1 == 0 {
            let problem = format!("{what}: a frame of no pixels");
            self.check.problems.push(problem);
        }
        let (fx, fy) = file.foot;
        if fx > file.size.0 || fy > file.size.1 {
            let (w, h) = file.size;
            self.check.problems.push(format!(
                "{what}: its foot ({fx}, {fy}) lies outside its {w}×{h} px frame"
            ));
        }
        if file.frames.len() < 2 {
            let problem = format!("{what}: a tree needs two frames or more");
            self.check.problems.push(problem);
        }
        if file.frames.iter().any(|&(_, ms)| ms == 0) {
            self.check
                .problems
                .push(format!("{what}: a frame lasts 0 ms"));
        }
        let frames = (1..)
            .zip(&file.frames)
            .map(|(k, &(at, ms))| (self.sized(&format!("{what}, frame {k}"), at, file.size), ms))
            .collect();
        TreeKind {
            foot: file.foot,
            frames,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion::GraphicsTier::{Full, Minimal, Reduced};

    /// A map drawn as rows: `#` forest, anything else not.
    fn map(rows: &[&str]) -> (i32, i32, impl Fn(Pos) -> bool) {
        let rows: Vec<Vec<bool>> = rows
            .iter()
            .map(|r| r.chars().map(|c| c == '#').collect())
            .collect();
        let h = i32::try_from(rows.len()).unwrap();
        let w = i32::try_from(rows[0].len()).unwrap();
        let forest = move |p: Pos| {
            let row = usize::try_from(p.y).ok().and_then(|y| rows.get(y));
            let tile = row.and_then(|r| usize::try_from(p.x).ok().and_then(|x| r.get(x)));
            tile.copied().unwrap_or(false)
        };
        (w, h, forest)
    }

    fn placed(rows: &[&str]) -> Vec<(i32, i32, Footprint)> {
        let (w, h, forest) = map(rows);
        place(w, h, forest)
            .into_iter()
            .map(|t| (t.trunk.x, t.trunk.y, t.footprint))
            .collect()
    }

    use Footprint::{Big, Half};

    #[test]
    fn a_three_by_three_forest_gets_one_big_tree_on_its_bottom_middle_tile() {
        assert_eq!(
            placed(&[".....", ".###.", ".###.", ".###.", "....."]),
            [(2, 3, Big)]
        );
    }

    #[test]
    fn a_two_by_two_and_a_one_by_two_forest_get_half_size_trees() {
        // Two trunks on the bottom row, their crowns over the top row.
        assert_eq!(
            placed(&["....", ".##.", ".##.", "...."]),
            [(1, 2, Half), (2, 2, Half)]
        );
        // One wide, two tall: one tree.
        assert_eq!(placed(&["...", ".#.", ".#.", "..."]), [(1, 2, Half)]);
        // Two wide, one tall: a tree on each, the crowns over the grass.
        assert_eq!(
            placed(&["....", ".##.", "...."]),
            [(1, 1, Half), (2, 1, Half)]
        );
        // A lone tile gets one too.
        assert_eq!(placed(&["...", ".#.", "..."]), [(1, 1, Half)]);
        // One wide, three tall: the bottom tree's crown over the middle,
        // the top tile a tree of its own.
        assert_eq!(placed(&[".#.", ".#.", ".#."]), [(1, 0, Half), (1, 2, Half)]);
    }

    #[test]
    fn big_trees_come_first_and_half_size_ones_take_what_is_left() {
        // Five across, three down: a big tree on the left three, then the
        // two columns left get half-size trees.
        assert_eq!(
            placed(&["#####", "#####", "#####"]),
            [
                (3, 0, Half),
                (4, 0, Half),
                (1, 2, Big),
                (3, 2, Half),
                (4, 2, Half)
            ]
        );
        // Six across: two big trees side by side.
        assert_eq!(
            placed(&["######", "######", "######"]),
            [(1, 2, Big), (4, 2, Big)]
        );
    }

    #[test]
    fn no_tree_stands_on_a_tile_that_isnt_forest_and_every_forest_tile_is_under_one() {
        let rows = [
            "..##....#",
            ".####..##",
            "######.#.",
            ".###.###.",
            "..#..##..",
        ];
        let (w, h, forest) = map(&rows);
        let trees = place(w, h, &forest);
        // Each tile a big tree or a trunk stands on is under one tree only;
        // a half-size tree's crown may reach over another's or the grass.
        let mut covered = BTreeSet::new();
        for tree in &trees {
            assert!(forest(tree.trunk), "{tree:?}");
            match tree.footprint {
                Big => {
                    for dy in 0..=2 {
                        for dx in -1..=1 {
                            let p = Pos::new(tree.trunk.x + dx, tree.trunk.y - dy);
                            assert!(forest(p), "{tree:?} at {p:?}");
                            assert!(covered.insert(p), "{p:?} under two trees");
                        }
                    }
                }
                Half => {
                    assert!(covered.insert(tree.trunk), "{tree:?} under two trees");
                    covered.insert(Pos::new(tree.trunk.x, tree.trunk.y - 1));
                }
            }
        }
        for y in 0..h {
            for x in 0..w {
                let p = Pos::new(x, y);
                assert!(!forest(p) || covered.contains(&p), "{p:?} has no tree");
            }
        }
        // The same on every run, painted from the top row down.
        assert_eq!(place(w, h, &forest), trees);
        assert!(trees.is_sorted_by_key(|t| (t.trunk.y, t.trunk.x)));
    }

    fn kind(ms: &[u32], x: u32) -> TreeKind {
        let rect = |k: u32| ImageRect {
            x: x + 16 * k,
            y: 0,
            w: 16,
            h: 32,
        };
        TreeKind {
            foot: (8, 30),
            frames: (0..).zip(ms).map(|(k, &ms)| (rect(k), ms)).collect(),
        }
    }

    fn trees() -> Trees {
        let still = crate::motion::Still {
            show: Show::FirstFrame,
            todo: None,
        };
        let stills = [Reduced, Minimal].map(|tier| (tier, still.clone()));
        Trees {
            of: BTreeSet::new(),
            big: vec![kind(&[100; 12], 0), kind(&[100; 12], 512)],
            half: vec![kind(&[100; 12], 1024)],
            motion: Motion {
                lowest: Full,
                stills: stills.into_iter().collect(),
            },
        }
    }

    /// `mixed` of three places for the phase, and of one for the kind.
    const MIXED_PHASES: [u32; 3] = [1_479_801_789, 2_116_403_790, 3_682_769_428];
    const MIXED_KIND: u32 = 41_153_166;

    /// A tree's phase lies inside its loop and differs from tree to tree;
    /// it and its kind are fixed numbers of its place, the same on every
    /// build (a change would move every tree of every forest).
    #[test]
    fn a_trees_kind_and_phase_are_fixed_numbers_of_its_place() {
        let t = trees();
        let tree = |x, y| Tree {
            trunk: Pos::new(x, y),
            footprint: Footprint::Big,
        };
        let kind = &t.big[0];
        let phases: BTreeSet<u64> = (0..6)
            .flat_map(|x| (0..4).map(move |y| tree(x, y)))
            .map(|tr| Trees::phase_ms(tr, kind))
            .collect();
        assert!(phases.len() > 6, "{phases:?}");
        assert!(phases.iter().all(|&p| p < kind.cycle_ms()), "{phases:?}");
        assert_eq!(
            [(0, 0), (1, 2), (5, 3)].map(|(x, y)| mixed(Pos::new(x, y), Pick::Phase)),
            [MIXED_PHASES[0], MIXED_PHASES[1], MIXED_PHASES[2]]
        );
        assert_eq!(mixed(Pos::new(1, 2), Pick::Kind), MIXED_KIND);
    }

    #[test]
    fn a_frame_is_the_one_for_the_time_into_the_loop() {
        let k = kind(&[300, 100, 200], 0);
        let x = |t| k.frame_at(t).x;
        assert_eq!(
            [x(0), x(299), x(300), x(399), x(400), x(599), x(600)],
            [0, 0, 16, 16, 32, 32, 0]
        );
        assert_eq!(k.cycle_ms(), 600);
    }

    #[test]
    fn trees_of_one_forest_show_different_frames_and_each_the_same_on_every_run() {
        let t = trees();
        let (w, h, forest) = map(&["######", "######", "######", "##.###", "##.###"]);
        let placed = place(w, h, forest);
        assert!(placed.len() >= 4);
        for clock in [0, 250, 1_000, 7_777] {
            let frames: Vec<ImageRect> = placed
                .iter()
                .map(|&tree| t.picture(tree, Full, Some(clock)).unwrap().1)
                .collect();
            let distinct: BTreeSet<(u32, u32)> = frames.iter().map(|r| (r.x, r.y)).collect();
            assert!(distinct.len() > 1, "{clock} ms: every tree on one frame");
            // The two big trees side by side differ.
            assert_ne!(frames[0], frames[1], "{clock} ms");
            let again: Vec<ImageRect> = placed
                .iter()
                .map(|&tree| t.picture(tree, Full, Some(clock)).unwrap().1)
                .collect();
            assert_eq!(frames, again);
        }
    }

    #[test]
    fn a_lesser_tier_draws_every_trees_first_frame() {
        let t = trees();
        let (w, h, forest) = map(&["######", "######", "######", "#.#.##"]);
        for tier in [Reduced, Minimal] {
            for tree in place(w, h, &forest) {
                let kind = t.kind(tree).unwrap();
                for clock in [0, 150, 1_234] {
                    let (_, frame) = t.picture(tree, tier, Some(clock)).unwrap();
                    assert_eq!(frame, kind.frames[0].0, "{tier}, {clock} ms");
                }
            }
        }
    }

    use super::super::frames::moving_rows;
    use super::super::tests::{Fixture, file};

    /// A tileset of the fixture's 64 × 64 image whose look
    /// `rogue_adventure` has `trees`.
    fn with_trees(f: &Fixture, trees: &str) -> String {
        let looks = format!(
            "{{ \"rogue_adventure\": (terrain: {{ {} }}, trees: {trees}) }}",
            f.terrain("")
        );
        file(f, &[("looks", &looks)])
    }

    /// Trees as written: a big kind of 32 × 32 px, a half-size one of 16 ×
    /// 16, each two frames; `motion` its stills.
    fn written(of: &str, big: &str, motion: &str) -> String {
        format!(
            "(of: [{of}], big: [{big}], \
             half: [(size: (16, 16), foot: (8, 16), frames: [((0, 2), 100), ((1, 2), 100)])], \
             motion: (stills: {{{motion}}}))"
        )
    }

    const BIG: &str = "(size: (32, 32), foot: (16, 30), frames: [((0, 0), 160), ((2, 0), 160)])";
    const STILLS: &str = "Reduced: (show: FirstFrame), Minimal: (show: FirstFrame)";

    #[test]
    fn a_look_reads_its_trees() {
        let f = Fixture::new();
        let tileset = f
            .parse(&with_trees(&f, &written("\"forest\"", BIG, STILLS)))
            .unwrap();
        let trees = tileset.trees("rogue_adventure").expect("the look's trees");
        assert_eq!(tileset.trees("indoor"), None);
        let forest = f.content.terrain.display.id_of("forest").unwrap();
        assert_eq!(trees.of, [forest].into_iter().collect());
        let rect = |x, y, w, h| ImageRect { x, y, w, h };
        assert_eq!(
            trees.big,
            [TreeKind {
                foot: (16, 30),
                frames: vec![(rect(0, 0, 32, 32), 160), (rect(32, 0, 32, 32), 160)],
            }]
        );
        assert_eq!(trees.half[0].frames[1], (rect(16, 32, 16, 16), 100));
        // The stills check reads them.
        let rows = moving_rows([&tileset]);
        let names: Vec<&str> = rows.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["tileset \"t\", look \"rogue_adventure\"'s trees"]);
    }

    #[test]
    fn a_looks_trees_are_checked() {
        let f = Fixture::new();
        let errors = |trees: &str| f.errors(&with_trees(&f, trees));
        let what = "look \"rogue_adventure\": trees";
        assert_eq!(
            errors(&written("\"woods\"", BIG, STILLS)),
            [format!("{what}: terrain \"woods\" is not in terrain.ron")]
        );
        let outside = "(size: (32, 32), foot: (16, 30), frames: [((0, 0), 160), ((3, 0), 160)])";
        assert_eq!(
            errors(&written("\"forest\"", outside, STILLS)),
            [format!(
                "{what}, big 1, frame 2: its 32×32 px at (48, 0) lies outside the 64×64 px image"
            )]
        );
        let bad = "(size: (32, 32), foot: (40, 30), frames: [((0, 0), 0)])";
        assert_eq!(
            errors(&written("\"forest\"", bad, STILLS)),
            [
                format!("{what}, big 1: its foot (40, 30) lies outside its 32×32 px frame"),
                format!("{what}, big 1: a tree needs two frames or more"),
                format!("{what}, big 1: a frame lasts 0 ms"),
            ]
        );
        let bare = "Reduced: (show: Nothing), Minimal: (show: FirstFrame)";
        assert_eq!(
            errors(&written("\"forest\"", BIG, bare)),
            [format!(
                "{what}: a tree's still is its first frame (`show: FirstFrame`); `Nothing` would \
                 leave the forest bare"
            )]
        );
        let none = written("", "", STILLS);
        assert_eq!(
            errors(&none),
            [
                format!("{what}: `of` names no terrain"),
                format!("{what}: `big` has no kind of tree"),
            ]
        );
        // A frame with no width (or no height) has no pixels.
        let flat = "(size: (0, 32), foot: (0, 30), frames: [((0, 0), 160), ((2, 0), 160)])";
        assert_eq!(
            errors(&written("\"forest\"", flat, STILLS)),
            [format!("{what}, big 1: a frame of no pixels")]
        );
        // A foot on the frame's bottom-right corner is inside it.
        let corner = "(size: (32, 32), foot: (32, 32), frames: [((0, 0), 160), ((2, 0), 160)])";
        let corner = with_trees(&f, &written("\"forest\"", corner, STILLS));
        assert!(f.parse(&corner).is_ok());
    }
}
