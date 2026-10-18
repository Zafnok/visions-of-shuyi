//! A tileset's terrain looks (ADR-0052) as its file writes them: for each
//! look, a tile for every terrain and the layers painted over those tiles,
//! checked against this game's terrains and the looks its maps may name.
//! What a look is (its tiles, its corner and tile layers, the mix rule) is
//! the engine's (`shuyi_assets::tileset`, ticket 1115).
use crate::prelude::*;

use alloc::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use trpg_core::TerrainId;

pub use shuyi_assets::tileset::{
    CornerLayer, Layer, Look, MIXES, TileLayer, mix, mix_name, parse_mix,
};
use shuyi_assets::tileset::{MIX_IN as IN, MIX_OUT as OUT};

use super::{Checker, ImageRect, TilesetRefs, grid_rect};
use crate::map::{DEFAULT_TILES, TILE_LOOKS};

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
