//! A tileset's terrain looks (ADR-0052): for each look, a tile for every
//! terrain and the layers painted over those tiles. A layer's picture is
//! chosen from the tiles around it, so a terrain grid alone says what the
//! map looks like: a map file only ever names terrain.
//!
//! Two kinds of layer:
//!
//! - **Corners**: pictures drawn *between* tiles, each centred on the
//!   point where four tiles meet, one for each mix of those four being in
//!   the layer or not. A shore, a road's edge and a forest's outline are
//!   these: they end where their tiles end.
//! - **Tiles**: one picture on each tile of the layer (a fort, a bridge),
//!   which may depend on which of its four neighbours are certain terrains
//!   (a bridge turns to cross the water).

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use trpg_core::TerrainId;

use super::{Checker, ImageRect, TilesetRefs, grid_rect};
use crate::map::{DEFAULT_TILES, TILE_LOOKS};

/// How many mixes four yes-or-no places have: a layer has a picture for
/// each (or none).
pub const MIXES: usize = 16;

/// The character of a place that is in, in a mix as written (`"##.."`).
const IN: char = '#';

/// The character of a place that is out.
const OUT: char = '.';

/// The number of a mix of four places, the first the highest bit: for a
/// corners layer the tiles at the top-left, top-right, bottom-left and
/// bottom-right of a corner point; for a tile's sides its neighbours up,
/// right, down and left.
pub fn mix(places: [bool; 4]) -> u8 {
    places.iter().fold(0, |m, &p| (m << 1) | u8::from(p))
}

/// The mix written as four of `#` (in) and `.` (out), or `None` if `text`
/// isn't that.
pub fn parse_mix(text: &str) -> Option<u8> {
    let mut places = [false; 4];
    let mut chars = text.chars();
    for place in &mut places {
        *place = match chars.next()? {
            IN => true,
            OUT => false,
            _ => return None,
        };
    }
    chars.next().is_none().then(|| mix(places))
}

/// Mix `m` as written: four of `#` and `.`.
pub fn mix_name(m: u8) -> String {
    [8, 4, 2, 1]
        .map(|bit| if m & bit == 0 { OUT } else { IN })
        .iter()
        .collect()
}

/// A terrain look: how one kind of place (outdoors, indoors) is drawn.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Look {
    /// Each terrain's own tile, painted first. Every terrain has one.
    pub tiles: BTreeMap<TerrainId, ImageRect>,
    /// What is painted over the tiles, the first undermost.
    pub layers: Vec<Layer>,
}

/// One layer of a [`Look`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Layer {
    /// Pictures between tiles.
    Corners(CornerLayer),
    /// A picture on each tile.
    Tiles(TileLayer),
}

/// Pictures drawn between tiles: each is one tile big and centred on a
/// point where four tiles meet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CornerLayer {
    /// The terrains that are in the layer.
    pub of: BTreeSet<TerrainId>,
    /// The picture for each [`mix`] of the four tiles round a point being
    /// in the layer. A mix with none draws nothing; mix 0 never has one.
    pub tiles: [Option<ImageRect>; MIXES],
}

/// A picture on each tile of some terrains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileLayer {
    /// The terrains that get the picture.
    pub of: BTreeSet<TerrainId>,
    /// The picture, unless [`sides`](Self::sides) has one.
    pub tile: ImageRect,
    /// The terrains a neighbour is looked at for.
    pub beside: BTreeSet<TerrainId>,
    /// The picture for a [`mix`] of the tile's four neighbours being one
    /// of [`beside`](Self::beside), where it differs from
    /// [`tile`](Self::tile).
    pub sides: [Option<ImageRect>; MIXES],
}

impl TileLayer {
    /// The picture of a tile whose neighbours mix as `sides`.
    pub fn picture(&self, sides: u8) -> ImageRect {
        let at = usize::from(sides);
        self.sides.get(at).copied().flatten().unwrap_or(self.tile)
    }
}

/// A look as written: in a tileset's `looks`, or (as `terrain` and
/// `layers`) at its top.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LookFile {
    pub(super) terrain: BTreeMap<String, (u32, u32)>,
    #[serde(default)]
    pub(super) layers: Vec<LayerFile>,
}

/// A layer as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum LayerFile {
    Corners {
        of: Vec<String>,
        tiles: Vec<CornersAt>,
    },
    Tiles {
        of: Vec<String>,
        at: (u32, u32),
        #[serde(default)]
        beside: Vec<String>,
        #[serde(default)]
        sides: Vec<SidesAt>,
    },
}

/// The picture of one mix of corners, as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CornersAt {
    corners: String,
    at: (u32, u32),
}

/// The picture of one mix of sides, as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SidesAt {
    sides: String,
    at: (u32, u32),
}

/// What a named look is called in a problem: nothing for the tileset's
/// own.
fn prefix(name: Option<&str>) -> String {
    name.map(|n| format!("look \"{n}\": ")).unwrap_or_default()
}

/// What is wrong with calling a look `name` in `looks`, if anything: only
/// the looks a map can name, and not the one the tileset's own `terrain`
/// and `layers` are.
pub(super) fn name_problem(name: &str) -> Option<String> {
    if name == DEFAULT_TILES {
        return Some(format!(
            "look \"{name}\" is the tileset's own `terrain` and `layers`: write it there"
        ));
    }
    (!TILE_LOOKS.contains(&name)).then(|| {
        let known: Vec<String> = TILE_LOOKS.iter().map(|l| format!("\"{l}\"")).collect();
        format!(
            "look \"{name}\" is not a look a map can name ({})",
            known.join(", ")
        )
    })
}

/// A look being checked.
pub(super) struct Looking<'a, 'c> {
    pub(super) check: &'a mut Checker<'c>,
    /// Names that don't exist, reported after every rectangle.
    pub(super) unknown: &'a mut Vec<String>,
    pub(super) refs: &'a TilesetRefs<'c>,
    /// A tile's size in the image.
    pub(super) tile_px: (u32, u32),
    /// Its name in `looks`; `None` for the tileset's own.
    pub(super) name: Option<&'a str>,
}

impl Looking<'_, '_> {
    /// The rectangle of tile `at`, reported (as `what`) if it lies outside
    /// the tileset's image.
    fn rect(&mut self, what: &str, at: (u32, u32)) -> ImageRect {
        let rect = grid_rect(at, self.tile_px, (0, 0));
        if let Some(sheet) = self.check.sheet {
            self.check.inside(what, sheet, rect);
        }
        rect
    }

    /// The terrains called `names`, reporting (as `what`) each that
    /// doesn't exist.
    fn terrains(&mut self, what: &str, names: &[String]) -> BTreeSet<TerrainId> {
        let mut ids = BTreeSet::new();
        for name in names {
            match self.refs.terrain.id_of(name) {
                Some(id) => {
                    ids.insert(id);
                }
                None => self
                    .unknown
                    .push(format!("{what}: terrain \"{name}\" is not in terrain.ron")),
            }
        }
        ids
    }

    /// The pictures of the mixes `entries` (each a mix as written, called
    /// a `kind`, and its tile), by mix; every problem reported as `what`.
    fn mixes<'e>(
        &mut self,
        what: &str,
        kind: &str,
        entries: impl Iterator<Item = (&'e str, (u32, u32))>,
    ) -> [Option<ImageRect>; MIXES] {
        let mut tiles = [None; MIXES];
        for (text, at) in entries {
            let Some(m) = parse_mix(text) else {
                self.check.problems.push(format!(
                    "{what}: {kind} \"{text}\" must be four of `{IN}` (in) and `{OUT}` (out)"
                ));
                continue;
            };
            let rect = self.rect(&format!("{what}, {kind} \"{text}\""), at);
            if tiles[usize::from(m)].replace(rect).is_some() {
                let problem = format!("{what}: {kind} \"{text}\" is listed twice");
                self.check.problems.push(problem);
            }
        }
        tiles
    }

    /// Layer `file`, the `number`th of the look, checked.
    fn layer(&mut self, number: usize, file: &LayerFile) -> Layer {
        let what = format!("{}layer {number}", prefix(self.name));
        let (LayerFile::Corners { of, .. } | LayerFile::Tiles { of, .. }) = file;
        if of.is_empty() {
            let problem = format!("{what}: `of` names no terrain");
            self.check.problems.push(problem);
        }
        let of = self.terrains(&what, of);
        match file {
            LayerFile::Corners { tiles, .. } => {
                let entries = tiles.iter().map(|t| (t.corners.as_str(), t.at));
                let tiles = self.mixes(&what, "corners", entries);
                if tiles[0].is_some() {
                    self.check.problems.push(format!(
                        "{what}: corners \"{}\" has no corner in the layer, so there is nothing \
                         to draw",
                        mix_name(0)
                    ));
                }
                if tiles.iter().all(Option::is_none) {
                    let problem = format!("{what}: it has no pictures");
                    self.check.problems.push(problem);
                }
                Layer::Corners(CornerLayer { of, tiles })
            }
            LayerFile::Tiles {
                at, beside, sides, ..
            } => {
                let tile = self.rect(&what, *at);
                let entries = sides.iter().map(|s| (s.sides.as_str(), s.at));
                let sides = self.mixes(&what, "sides", entries);
                let beside = self.terrains(&what, beside);
                Layer::Tiles(TileLayer {
                    of,
                    tile,
                    beside,
                    sides,
                })
            }
        }
    }

    /// The look of `terrain` (each terrain's tile, by name) and `layers`,
    /// checked. Terrains without a tile are [`missing`]'s to report.
    pub(super) fn look(
        &mut self,
        terrain: &BTreeMap<String, (u32, u32)>,
        layers: &[LayerFile],
    ) -> Look {
        let own = prefix(self.name);
        let mut tiles = BTreeMap::new();
        for (name, &at) in terrain {
            let rect = self.rect(&format!("{own}terrain \"{name}\""), at);
            match self.refs.terrain.id_of(name) {
                Some(id) => {
                    tiles.insert(id, rect);
                }
                None => self
                    .unknown
                    .push(format!("{own}terrain \"{name}\" is not in terrain.ron")),
            }
        }
        let layers = (1..)
            .zip(layers)
            .map(|(number, layer)| self.layer(number, layer))
            .collect();
        Look { tiles, layers }
    }
}

/// A problem for each terrain of `refs` that the look called `name`
/// (`None`: the tileset's own) has no tile for in `terrain`.
pub(super) fn missing(
    name: Option<&str>,
    terrain: &BTreeMap<String, (u32, u32)>,
    refs: &TilesetRefs,
) -> Vec<String> {
    let own = prefix(name);
    let lacks = |id: &&str| !terrain.contains_key(*id);
    let ids = refs.terrain.terrains.iter().map(|t| t.id.as_str());
    ids.filter(lacks)
        .map(|id| format!("{own}terrain \"{id}\" has no tile"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mix_is_four_places_the_first_the_highest_bit() {
        assert_eq!(mix([false; 4]), 0);
        assert_eq!(mix([true, false, false, false]), 8);
        assert_eq!(mix([false, true, false, false]), 4);
        assert_eq!(mix([false, false, true, false]), 2);
        assert_eq!(mix([false, false, false, true]), 1);
        assert_eq!(mix([true; 4]), 15);
        assert_eq!(mix([true, false, true, true]), 11);
    }

    #[test]
    fn a_mix_is_written_as_four_of_hash_and_dot() {
        for m in 0..16 {
            assert_eq!(parse_mix(&mix_name(m)), Some(m), "{m}");
        }
        assert_eq!(mix_name(0), "....");
        assert_eq!(mix_name(8), "#...");
        assert_eq!(mix_name(5), ".#.#");
        assert_eq!(mix_name(15), "####");
        assert_eq!(parse_mix("#..#"), Some(9));
        for bad in ["", "#", "###", "#####", "##.x", "x#..", "##. "] {
            assert_eq!(parse_mix(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn a_tile_layer_shows_its_sides_picture_where_it_has_one() {
        let rect = |x| ImageRect {
            x,
            y: 0,
            w: 16,
            h: 16,
        };
        let mut sides = [None; MIXES];
        sides[5] = Some(rect(32));
        let layer = TileLayer {
            of: BTreeSet::new(),
            tile: rect(16),
            beside: BTreeSet::new(),
            sides,
        };
        assert_eq!(layer.picture(5), rect(32));
        assert_eq!(layer.picture(0), rect(16));
        assert_eq!(layer.picture(15), rect(16));
        // No such mix: the plain picture.
        assert_eq!(layer.picture(16), rect(16));
        assert_eq!(layer.picture(u8::MAX), rect(16));
    }

    #[test]
    fn a_look_is_named_only_as_a_map_can_name_one() {
        assert_eq!(name_problem("indoor"), None);
        assert_eq!(
            name_problem("outdoor").as_deref(),
            Some("look \"outdoor\" is the tileset's own `terrain` and `layers`: write it there")
        );
        assert_eq!(
            name_problem("cave").as_deref(),
            Some("look \"cave\" is not a look a map can name (\"outdoor\", \"indoor\")")
        );
        assert_eq!(prefix(None), "");
        assert_eq!(prefix(Some("indoor")), "look \"indoor\": ");
    }
}
