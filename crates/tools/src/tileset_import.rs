//! `cargo xtask tileset-import [--list]`: turns the bought Tiled tilesets
//! into the game's tileset, `assets-private/game/tilesets/tiny_tales.png`
//! and `.ron` (ticket 0437, ADR-0052).
//!
//! The bought sets are Tiled files: a 16×16-tile image and a `.tsx` that
//! names, for a "terrain" of the set, the tile for each mix of its four
//! corners. Which bought terrain stands for which of ours is the mapping
//! file [`MAPPING_PATH`]: ours, with names and numbers and no art. The
//! importer reads both, copies each tile the mapping uses once into one
//! packed image (a recoloured copy where the mapping asks for a tint) and
//! writes the tileset file: the terrain looks, then the unit pictures of
//! `map-sprite-import`'s table.
//!
//! A tile the looks use that the `.tsx` animates (Tiled's `<animation>`)
//! moves in the game too (ticket 1315): its frames are packed and written
//! as the tileset's `frames`, each still on a lesser graphics tier. A loop
//! whose frames are all one picture doesn't move and is left out: Tiny
//! Tales' coasts, hills and woods are such loops (0096).
//!
//! A pack with no Tiled files (ELV Games' *Rogue Adventure*, ticket 1318)
//! is read as plain sheets: a tile by its place, a `Patch` layer from the
//! usual three-by-three layout, and a sheet's moving tiles by `moving`
//! (each frame a step along the row, or whole rows down the column).
//!
//! A `Quarters` layer (ticket 1359) builds each corner picture from
//! quarters of a kit's cells, RPG Maker style, so inside corners are round
//! too; `recolour` changes a sheet's colours as it is read (Lonesome
//! Forest's stones in a sandy brown): the recoloured art is never kept.
//! With a `face` (ticket 1361) its pictures are half a tile taller, the
//! cliff's face hanging on the tile below its bottom edge, and a `Shadow`
//! layer is their silhouettes, moved: the packed image then holds pictures
//! taller than a tile, each over the cell below its own.
//!
//! It is the only reader of Tiled files: the game reads our RON and PNG.
//! The XML it needs is three kinds of tag and their attributes, read by
//! [`tags`]; no XML crate.
//!
//! The output is bought art (ADR-0040): it goes to the private assets
//! checkout, never into this repository.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use trpg_content::TerrainDef;
use trpg_content::map::{DEFAULT_TILES, TILE_LOOKS};
use trpg_content::tileset::parse_mix;

use crate::map_sprite_import::{self, GAME_DIR, MapSprite, TILESET_ID};
use crate::tileset_ron::{
    self, At, FramesText, LayerText, LookText, Place, TreeKindText, TreesText,
};
use shuyi_cli::prelude::{decode_png, encode_png};

/// Shown for `--help` and after a bad argument.
pub const USAGE: &str = "usage: cargo xtask tileset-import [--list]\n\n\
Packs the bought tiles named by assets-src/tilesets/tiny_tales.ron, from\n\
assets-private/library/ (Tiny Tales' Tiled sets, other packs' plain sheets), into\n\
assets-private/game/tilesets/tiny_tales.png, and writes tiny_tales.ron beside it:\n\
the terrain looks, and the unit pictures of `map-sprite-import`'s table.\n\
--list  print which bought tiles each terrain and layer uses, and write nothing\n\n\
Run `cargo shuyi private-assets --library` first. Afterwards commit and push in\n\
assets-private/, then run `cargo xtask assets-pin --add`.";

/// The sorted bought packs, relative to the repo root: a plain sheet
/// (`….png`) of the mapping is named by its path under it.
pub const LIBRARIES_DIR: &str = "assets-private/library";
/// The Tiny Tales tilesets, under [`LIBRARIES_DIR`]: a Tiled sheet of the
/// mapping is named by its path under it, without `.tsx`.
pub const TILED_DIR: &str = "tiny-tales/tilesets";
/// The sorted bought tilesets, relative to the repo root.
pub const LIBRARY_DIR: &str = "assets-private/library/tiny-tales/tilesets";
/// The mapping file, relative to the repo root.
pub const MAPPING_PATH: &str = "assets-src/tilesets/tiny_tales.ron";
/// A tile's side in the packed image, in pixels: the bought tiles'.
pub const TILE: u32 = 16;
/// The height of a picture of a `Quarters` layer with a face (ticket
/// 1361): the rim's tile, and the lower half of the face of the tile below
/// its point.
pub const FACED: u32 = TILE + TILE / 2;
/// Tiles per row of the packed image.
pub const COLUMNS: u32 = 16;
/// What a moving tile still needs for the `Minimal` tier (a GBA one day:
/// `TODO(gba)`), listed by the content's stills check (ADR-0131).
pub const MINIMAL_TODO: &str = "a one-bit tile";
/// What a tree still needs for the `Minimal` tier (ticket 1356).
pub const MINIMAL_TREE_TODO: &str = "a one-bit tree";

/// A tile of a bought sheet: the sheet and the tile's id in it, counting
/// its 16×16 tiles row by row from 0. A sheet is a Tiled file (its `.tsx`
/// under [`TILED_DIR`], without the extension) or a plain sheet (a `.png`
/// under [`LIBRARIES_DIR`], with it): a pack with no Tiled files.
pub type TileRef = (String, u32);

/// The mapping file.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    /// Each of our terrains' own tile, unless a look has its own.
    pub terrain: BTreeMap<String, TileRef>,
    /// The layers, by a name the looks list them by.
    pub layers: BTreeMap<String, LayerMap>,
    /// The looks, by the name a map file gives.
    pub looks: BTreeMap<String, LookMap>,
    /// The plain sheets whose tiles move, by sheet: every packed tile of
    /// one loops through its frames ([`Strip`]).
    #[serde(default)]
    pub moving: BTreeMap<String, Strip>,
    /// Colours changed in a sheet as it is read, by sheet: each pixel of
    /// a pair's first colour takes its second, its alpha kept (ticket
    /// 1359: Lonesome Forest's stones in a sandy brown).
    #[serde(default)]
    pub recolour: BTreeMap<String, Vec<(Rgb, Rgb)>>,
}

/// A colour: red, green and blue.
pub type Rgb = (u8, u8, u8);

/// How a plain sheet draws its moving tiles: frame `k` of a tile is the
/// tile `k × step` places on, counted as ids are, each shown for `ms`: a
/// step shorter than a row runs along the row, a whole number of rows
/// down the column (ticket 1384: Rogue Adventure's fire pit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Strip {
    /// How many frames, the tile itself the first.
    pub frames: u32,
    /// Tiles from one frame to the next: along the row, or a whole
    /// number of rows (the sheet's columns times that) down the column.
    pub step: u32,
    /// How long each frame shows, in ms.
    pub ms: u32,
}

/// A layer of the mapping.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LayerMap {
    /// Pictures between tiles: a Tiled terrain's tile for each mix of
    /// corners.
    Corners {
        /// Our terrains in the layer.
        of: Vec<String>,
        /// The bought sheet.
        sheet: String,
        /// The Tiled terrain's name in it.
        terrain: String,
        /// Where the sheet has several tiles for a mix: which (0 = the
        /// first; the last if there are fewer).
        #[serde(default)]
        pick: usize,
        /// Whether to take the tile of all four corners in too. Not
        /// needed when the layer's terrains have it as their own tile.
        #[serde(default = "yes")]
        full: bool,
        /// A palette colour to recolour the tiles in, each pixel as
        /// bright as it was; empty for none.
        #[serde(default)]
        tint: String,
    },
    /// A picture on each tile.
    Tiles {
        /// Our terrains that get it.
        of: Vec<String>,
        /// The picture; left out, nothing (only `sides` and `near` draw).
        #[serde(default, deserialize_with = "some")]
        tile: Option<TileRef>,
        /// Our terrains a neighbour is looked at for.
        #[serde(default)]
        beside: Vec<String>,
        /// The picture for a mix of sides (up, right, down, left being
        /// one of `beside`), written as four of `#` and `.`.
        #[serde(default)]
        sides: BTreeMap<String, TileRef>,
        /// The picture where any side is one of `beside` and `sides` has
        /// none for the mix (ticket 1360: the keep's stone, a fort in a
        /// floor).
        #[serde(default, deserialize_with = "some")]
        near: Option<TileRef>,
    },
    /// The top of a wall seen from above, on each tile of the layer, by
    /// its neighbours in it (ticket 1360): `across` where one is beside it
    /// and none above or below, `side`'s left half and that half mirrored
    /// where one is above and one below and none beside, else `pillar`
    /// (a corner, an end, a wall alone).
    Walls {
        /// Our terrains in the layer.
        of: Vec<String>,
        /// The sheet.
        sheet: String,
        /// A wall running across.
        across: u32,
        /// The cell whose left half is a wall running up and down.
        side: u32,
        /// A pillar's cap.
        pillar: u32,
    },
    /// The face of a wall seen from the south, on the tile below each tile
    /// of the layer (ticket 1360): `pillar` where the tile above that one
    /// is in the layer too (the foot of a wall running down), else
    /// `across`. Drawn as pictures between tiles, from the point at the
    /// wall tile's top-left, so it never lands on the terrains `skip`
    /// names (another wall, a gate).
    Faces {
        /// Our terrains in the layer.
        of: Vec<String>,
        /// The sheet.
        sheet: String,
        /// The face under a wall running across.
        across: u32,
        /// The face under a pillar.
        pillar: u32,
        /// Our terrains whose tiles get no face.
        #[serde(default)]
        skip: Vec<String>,
    },
    /// A picture two tiles tall standing on each tile of the layer, its
    /// top on the tile above (ticket 1360: Harrowby's tower). Drawn as
    /// pictures between tiles, from the point at the tile's top-left, so
    /// it reaches out of its tile; `skip` names terrains its top never
    /// lands on.
    Standing {
        /// Our terrains in the layer.
        of: Vec<String>,
        /// The sheet.
        sheet: String,
        /// Its cells, the top first.
        cells: [u32; 2],
        /// Our terrains whose tiles get no part of it.
        #[serde(default)]
        skip: Vec<String>,
    },
    /// A picture on each tile, joined from a block of a sheet: nine tiles,
    /// three by three, of a patch of the terrain with its edge on every
    /// side (ticket 0452). Each quarter of a tile is the same quarter of
    /// the block's tile for whether the tile's neighbours that way are in
    /// the layer: a tile with no neighbour of the layer above has the top
    /// edge, one alone has the edge all round. So the patch's edge, inside
    /// its tiles, is drawn on the tiles where the terrain ends.
    Block {
        /// Our terrains that get it.
        of: Vec<String>,
        /// The bought sheet.
        sheet: String,
        /// The block's top-left tile.
        block: u32,
        /// A palette colour to recolour it in, as for `Corners`.
        #[serde(default)]
        tint: String,
    },
    /// Pictures between tiles, as `Corners`, from a sheet laid out the
    /// usual way rather than a Tiled terrain: a patch of nine tiles,
    /// three by three, whose eight outer tiles are its edge and outside
    /// corners and its middle the full tile, and nine more whose four
    /// corner tiles are the inside corners. A mix of two diagonal corners
    /// is two of the patch's outside corners, one over the other.
    Patch {
        /// Our terrains in the layer.
        of: Vec<String>,
        /// The sheet.
        sheet: String,
        /// The patch's top-left tile.
        patch: u32,
        /// The top-left tile of the nine with the inside corners.
        inside: u32,
        /// Whether to take the full tile in too, as for `Corners`.
        #[serde(default = "yes")]
        full: bool,
    },
    /// Pictures between tiles, as `Corners`, each joined from quarters of
    /// a kit's cells the way RPG Maker joins them (ticket 1359). Each
    /// quarter of a tile in the layer is the same quarter of one cell,
    /// chosen by the tile's two neighbours that way and the diagonal
    /// between them: neither in the layer, an outside corner; only the
    /// one across, an edge along the top or the bottom; only the one up
    /// or down, an edge down a side; both but not the diagonal, an inside
    /// corner; all three, nothing (the tile's own picture shows). So
    /// every corner, outside and inside, is the kit's round one.
    Quarters {
        /// Our terrains in the layer.
        of: Vec<String>,
        /// The kit's sheet.
        sheet: String,
        /// Which of its cells each kind of quarter comes from.
        kit: Kit,
        /// The cliff's face below the layer's bottom edge, if it has one
        /// (ticket 1361): each picture is then [`FACED`] pixels tall.
        #[serde(default, deserialize_with = "some")]
        face: Option<Face>,
        /// Our terrains whose tiles get no part of its pictures (a wall a
        /// face never hangs on).
        #[serde(default)]
        skip: Vec<String>,
    },
    /// The silhouettes of another layer's pictures, a `Quarters` layer
    /// with a face, in one see-through colour, moved by `offset` (ticket
    /// 1361: the soft shadow under a cliff). It skips what that layer
    /// skips.
    Shadow {
        /// The layer whose pictures it is the shadow of.
        of: String,
        /// The shadow's colour and alpha.
        colour: (u8, u8, u8, u8),
        /// Pixels across and down it is moved by.
        offset: (i32, i32),
    },
}

/// A field that may be left out: `Some` of what is written.
fn some<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}

/// The cells of a cliff's face (ticket 1361), hung on the tile below a
/// tile of the layer whose tile below isn't in it: its left half the
/// `left` cell's where the layer ends to the left of the tile above,
/// else `middle`'s; its right half the `right` cell's where it ends to
/// the right, else `middle`'s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Face {
    /// The face's left end.
    pub left: u32,
    /// The face where the layer goes on both ways.
    pub middle: u32,
    /// The face's right end.
    pub right: u32,
}

impl Face {
    /// The cells of the face on the down-left and the down-right tile of a
    /// point whose tiles (up-left, up-right, down-left, down-right) are
    /// `places` in the layer: of the down-left tile its right half, of the
    /// down-right its left half, the halves nearest the point; `None`
    /// where no face hangs.
    pub fn under(&self, places: [bool; 4]) -> [Option<u32>; 2] {
        let [up_left, up_right, down_left, down_right] = places;
        let end = |goes_on: bool, end: u32| if goes_on { self.middle } else { end };
        [
            (up_left && !down_left).then(|| end(up_right, self.right)),
            (up_right && !down_right).then(|| end(up_left, self.left)),
        ]
    }
}

/// What a wall's top is ([`LayerMap::Walls`], ticket 1360), by its
/// neighbours in the layer, as 0096's mockups drew Harl's keep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallTop {
    /// A wall running across: one beside it, none above or below.
    Across,
    /// A wall running up and down: one above and one below, none beside.
    UpDown,
    /// A pillar: a corner, an end, a wall alone.
    Pillar,
}

impl WallTop {
    /// The top of a wall whose neighbours up, right, down and left are
    /// `sides` in the layer.
    pub const fn of(sides: [bool; 4]) -> Self {
        match sides {
            [false, true, false, _] | [false, _, false, true] => Self::Across,
            [true, false, true, false] => Self::UpDown,
            _ => Self::Pillar,
        }
    }
}

/// The cell of the face ([`LayerMap::Faces`], ticket 1360) hung from the
/// point whose tiles (up-left, up-right, down-left, down-right) are
/// `places` in the layer, on the tile below its down-right one: `pillar`
/// where the up-right one is in too (the foot of a wall running down),
/// else `across`; `None` where the down-right one isn't in.
///
/// A point sees no further than its four tiles, so a wall alone (whose
/// top is a pillar) hangs the face of a wall running across.
pub const fn face(places: [bool; 4], across: u32, pillar: u32) -> Option<u32> {
    match places {
        [_, true, _, true] => Some(pillar),
        [.., true] => Some(across),
        _ => None,
    }
}

/// Half of `px` pixels, as an offset.
fn half(px: u32) -> i32 {
    i32::try_from(px / 2).unwrap_or(0)
}

/// A layer of a picture by each tile of `of` ([`LayerMap::Faces`],
/// [`LayerMap::Standing`]), drawn as pictures between tiles from the
/// point at the tile's top-left so that it may reach out of it: for each
/// mix of corners, `cut`'s picture of `sheet`, or none. `place` moves it
/// from the point to where it goes.
fn by_tile(
    packer: &mut Packer,
    of: &[String],
    sheet: &str,
    cut: impl Fn([bool; 4]) -> Option<Cut>,
    place: Place,
) -> Result<LayerText, String> {
    let mut tiles = Vec::new();
    for mix in 1..16u8 {
        if let Some(cut) = cut([8, 4, 2, 1].map(|bit| mix & bit != 0)) {
            tiles.push((mix, packer.packed(sheet, cut, "")?));
        }
    }
    Ok(LayerText::Corners {
        of: of.to_vec(),
        tiles,
        place,
    })
}

/// The cells of a [`LayerMap::Quarters`]: which cell each kind of quarter
/// is taken from, counted as a sheet's tiles are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kit {
    /// The outside corners, by quarter: up-left, up-right, down-left,
    /// down-right.
    pub outer: [u32; 4],
    /// The edges along the top and along the bottom.
    pub across: [u32; 2],
    /// The edges down the left and down the right.
    pub down: [u32; 2],
    /// The inside corners, by quarter.
    pub inside: [u32; 4],
}

impl Kit {
    /// The four quarters of the picture of a point whose tiles (up-left,
    /// up-right, down-left, down-right) are `places` in the layer, in
    /// that order: the cell each tile's quarter nearest the point comes
    /// from, or `None` where nothing is drawn.
    pub fn quarters(&self, places: [bool; 4]) -> [Option<u32>; 4] {
        // A tile's neighbour across is the place beside it (index ^ 1),
        // the one up or down the place above or below it (^ 2), the
        // diagonal the opposite place (^ 3); its quarter nearest the point
        // is the opposite quarter (^ 3), counted the same way.
        let mut out = [None; 4];
        for (place, cell) in out.iter_mut().enumerate() {
            let quarter = place ^ 3;
            let (across, upright, diagonal) =
                (places[place ^ 1], places[place ^ 2], places[place ^ 3]);
            *cell = match (places[place], across, upright) {
                (false, ..) => None,
                (true, false, false) => Some(self.outer[quarter]),
                (true, true, false) => Some(self.across[quarter >> 1]),
                (true, false, true) => Some(self.down[quarter & 1]),
                (true, true, true) => (!diagonal).then_some(self.inside[quarter]),
            };
        }
        out
    }
}

/// A mix of corners (up-left, up-right, down-left, down-right in the
/// layer), whether its picture is in the inside corners' nine of a
/// [`LayerMap::Patch`], and its tiles' (column, row) there, the second
/// drawn over the first.
type PatchPlace = ([bool; 4], bool, &'static [(u32, u32)]);

/// Where each mix of corners is in a [`LayerMap::Patch`].
const PATCH: [PatchPlace; 15] = {
    const fn m(ul: u8, ur: u8, dl: u8, dr: u8) -> [bool; 4] {
        [ul == 1, ur == 1, dl == 1, dr == 1]
    }
    [
        (m(0, 0, 0, 1), false, &[(0, 0)]),
        (m(0, 0, 1, 1), false, &[(1, 0)]),
        (m(0, 0, 1, 0), false, &[(2, 0)]),
        (m(0, 1, 0, 1), false, &[(0, 1)]),
        (m(1, 1, 1, 1), false, &[(1, 1)]),
        (m(1, 0, 1, 0), false, &[(2, 1)]),
        (m(0, 1, 0, 0), false, &[(0, 2)]),
        (m(1, 1, 0, 0), false, &[(1, 2)]),
        (m(1, 0, 0, 0), false, &[(2, 2)]),
        (m(1, 1, 1, 0), true, &[(0, 0)]),
        (m(1, 1, 0, 1), true, &[(2, 0)]),
        (m(1, 0, 1, 1), true, &[(0, 2)]),
        (m(0, 1, 1, 1), true, &[(2, 2)]),
        (m(1, 0, 0, 1), false, &[(2, 2), (0, 0)]),
        (m(0, 1, 1, 0), false, &[(0, 2), (2, 0)]),
    ]
};

const fn yes() -> bool {
    true
}

/// A look of the mapping.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LookMap {
    /// Own tiles that differ from the mapping's.
    #[serde(default)]
    pub terrain: BTreeMap<String, TileRef>,
    /// Its layers by name, the first undermost.
    pub layers: Vec<String>,
    /// The trees standing on its forest (ticket 1356), if it has any.
    #[serde(default, deserialize_with = "some")]
    pub trees: Option<TreesMap>,
}

/// A look's trees in the mapping (ticket 1356): plain sheets of frames,
/// each a kind of big tree, and each also halved for a half-size one.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreesMap {
    /// Our terrains they stand on.
    pub of: Vec<String>,
    /// The sheets, by path under [`LIBRARIES_DIR`]: each one kind, its
    /// frames row by row from the top-left.
    pub sheets: Vec<String>,
    /// A frame's size in pixels (even, so it halves).
    pub frame: (u32, u32),
    /// How many frames a sheet has.
    pub frames: u32,
    /// How long each frame shows, in ms.
    pub ms: u32,
    /// The pixel of a frame on the middle of its trunk tile's bottom edge.
    pub foot: (u32, u32),
}

/// Every tag of `xml` that opens an element, with its attributes. Enough
/// XML for a Tiled file: attribute values are in double quotes and hold no
/// `>`.
pub fn tags(xml: &str) -> Vec<(String, BTreeMap<String, String>)> {
    let mut out = Vec::new();
    for chunk in xml.split('<').skip(1) {
        let body = chunk.split('>').next().unwrap_or_default();
        // A closing tag, the declaration, a comment.
        if body.starts_with(['/', '?', '!']) {
            continue;
        }
        let body = body.trim_end_matches('/');
        let (name, mut rest) = body.split_once(char::is_whitespace).unwrap_or((body, ""));
        let mut attributes = BTreeMap::new();
        while let Some((key, after)) = rest.split_once("=\"") {
            let Some((value, tail)) = after.split_once('"') else {
                break;
            };
            attributes.insert(key.trim().to_owned(), unescape(value));
            rest = tail;
        }
        out.push((name.trim().to_owned(), attributes));
    }
    out
}

/// `value` with XML's five named characters written out.
fn unescape(value: &str) -> String {
    let named = [
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&apos;", "'"),
    ];
    let mut out = value.to_owned();
    for (from, to) in named {
        out = out.replace(from, to);
    }
    // Last, so `&amp;lt;` stays the text `&lt;`.
    out.replace("&amp;", "&")
}

/// What the importer reads of a Tiled tileset file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tsx {
    /// Its image, relative to the file.
    pub image: String,
    /// The colour that stands for nothing (`trans`), if it names one.
    pub clear: Option<[u8; 3]>,
    /// A tile's size, across × down.
    pub tile: (u32, u32),
    /// Tiles per row of the image.
    pub columns: u32,
    /// Its terrains' names, in order: a tile names them by position.
    pub terrains: Vec<String>,
    /// Each tile with corner terrains: its id and the terrain at its
    /// top-left, top-right, bottom-left and bottom-right corner (`None`:
    /// none).
    pub corners: Vec<(u32, [Option<usize>; 4])>,
    /// Each animated tile's frames, by its id: each frame's tile and its
    /// time in ms, in order.
    pub animations: BTreeMap<u32, Vec<(u32, u32)>>,
}

/// A colour written as six hex digits.
fn hex(text: &str) -> Option<[u8; 3]> {
    let digits = text.trim_start_matches('#');
    if digits.len() != 6 || !digits.is_ascii() {
        return None;
    }
    let channel = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok();
    Some([channel(0)?, channel(2)?, channel(4)?])
}

/// The corner terrains of a tile, as its `terrain` attribute writes them:
/// four places split by commas, each a terrain's number or empty.
fn corner_terrains(text: &str) -> Option<[Option<usize>; 4]> {
    let mut corners = [None; 4];
    let mut parts = text.split(',');
    for corner in &mut corners {
        let part = parts.next()?.trim();
        if !part.is_empty() {
            *corner = Some(part.parse().ok()?);
        }
    }
    parts.next().is_none().then_some(corners)
}

/// Parses a Tiled tileset file.
pub fn parse_tsx(xml: &str) -> Result<Tsx, String> {
    let tags = tags(xml);
    let first = |name: &str| tags.iter().find(|(n, _)| n == name).map(|(_, a)| a);
    let tileset = first("tileset").ok_or("no <tileset>")?;
    let number = |key: &str| {
        let value = tileset.get(key).and_then(|v| v.parse::<u32>().ok());
        value
            .filter(|&n| n > 0)
            .ok_or_else(|| format!("<tileset> has no {key}"))
    };
    let image = first("image").ok_or("no <image>")?;
    let source = image.get("source").ok_or("<image> has no source")?;
    let clear = match image.get("trans") {
        Some(text) => Some(hex(text).ok_or_else(|| format!("trans \"{text}\" is not a colour"))?),
        None => None,
    };
    let named = |name: &'static str| tags.iter().filter(move |(n, _)| n == name);
    let terrains = named("terrain")
        .map(|(_, a)| a.get("name").cloned().unwrap_or_default())
        .collect();
    let mut corners = Vec::new();
    for (_, attributes) in named("tile") {
        let Some(text) = attributes.get("terrain") else {
            continue;
        };
        let id = attributes.get("id").and_then(|v| v.parse::<u32>().ok());
        let tile = id.zip(corner_terrains(text));
        corners.push(tile.ok_or_else(|| format!("a <tile> with terrain \"{text}\" is not read"))?);
    }
    // A <frame> belongs to the <tile> before it: frames are only in a
    // tile's <animation>.
    let mut animations: BTreeMap<u32, Vec<(u32, u32)>> = BTreeMap::new();
    let mut tile = None;
    for (name, attributes) in &tags {
        let number = |key: &str| attributes.get(key).and_then(|v| v.parse::<u32>().ok());
        match name.as_str() {
            "tile" => tile = number("id"),
            "frame" => {
                let at = tile.ok_or("a <frame> outside a <tile>")?;
                let frame = number("tileid").zip(number("duration"));
                let frame = frame.ok_or_else(|| format!("tile {at}: a <frame> is not read"))?;
                animations.entry(at).or_default().push(frame);
            }
            _ => {}
        }
    }
    Ok(Tsx {
        image: source.clone(),
        clear,
        tile: (number("tilewidth")?, number("tileheight")?),
        columns: number("columns")?,
        terrains,
        corners,
        animations,
    })
}

/// A bought sheet: its Tiled file and its image's pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sheet {
    /// The Tiled file.
    pub tsx: Tsx,
    /// The image's width and height.
    pub size: (u32, u32),
    /// Its RGBA8 pixels, row by row.
    pub rgba: Vec<u8>,
}

impl Sheet {
    /// Reads sheet `name` of the packs under `libraries`: a plain sheet
    /// (`….png`) there, or a Tiled one (its `.tsx` without the extension)
    /// under [`TILED_DIR`].
    pub fn read(libraries: &Path, name: &str) -> Result<Self, String> {
        if is_plain(name) {
            return Self::plain(&libraries.join(name));
        }
        let path = libraries.join(TILED_DIR).join(format!("{name}.tsx"));
        let xml =
            fs::read_to_string(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
        let tsx = parse_tsx(&xml).map_err(|e| format!("{}: {e}", path.display()))?;
        if tsx.tile != (TILE, TILE) {
            let (w, h) = tsx.tile;
            return Err(format!(
                "{}: its tiles are {w}×{h} px, not {TILE}×{TILE}",
                path.display()
            ));
        }
        let image = path.with_file_name(&tsx.image);
        let bytes = fs::read(&image).map_err(|e| format!("reading {}: {e}", image.display()))?;
        let (width, height, rgba) =
            decode_png(&bytes).map_err(|e| format!("{}: {e}", image.display()))?;
        Ok(Self {
            tsx,
            size: (width, height),
            rgba,
        })
    }

    /// Reads the plain sheet at `path`: an image of 16×16 tiles and
    /// nothing else (no clear colour, no terrains, nothing moving).
    fn plain(path: &Path) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
        let (width, height, rgba) =
            decode_png(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        if width < TILE || height < TILE {
            return Err(format!(
                "{}: a {width}×{height} px image holds no {TILE}×{TILE} tile",
                path.display()
            ));
        }
        let image = path.file_name().unwrap_or_default().to_string_lossy();
        let tsx = Tsx {
            image: image.into_owned(),
            clear: None,
            tile: (TILE, TILE),
            columns: width / TILE,
            terrains: Vec::new(),
            corners: Vec::new(),
            animations: BTreeMap::new(),
        };
        Ok(Self {
            tsx,
            size: (width, height),
            rgba,
        })
    }

    /// The RGBA8 pixels of tile `id`, row by row, the Tiled file's clear
    /// colour as nothing.
    pub fn tile(&self, id: u32) -> Result<Vec<u8>, String> {
        let (left, top) = (
            (id % self.tsx.columns) * TILE,
            (id / self.tsx.columns) * TILE,
        );
        let (width, height) = self.size;
        if left + TILE > width || top + TILE > height {
            return Err(format!(
                "tile {id} is outside the {width}×{height} px image"
            ));
        }
        let mut out = Vec::new();
        for (x, y) in (top..top + TILE).flat_map(|y| (left..left + TILE).map(move |x| (x, y))) {
            let at = usize::try_from((y * width + x) * 4).unwrap_or(usize::MAX);
            let nothing = [0; 4];
            let pixel = self.rgba.get(at..at + 4).unwrap_or(&nothing);
            let clear = self.tsx.clear.is_some_and(|c| pixel[..3] == c) || pixel[3] == 0;
            out.extend_from_slice(if clear { &nothing } else { pixel });
        }
        Ok(out)
    }

    /// The tile of each mix of corners of the Tiled terrain `terrain`:
    /// of the tiles whose corners are that terrain or none, the `pick`th
    /// listed for the mix (the last, if there are fewer).
    pub fn corners(&self, terrain: &str, pick: usize) -> Result<[Option<u32>; 16], String> {
        let index = self.tsx.terrains.iter().position(|name| name == terrain);
        let index = index.ok_or_else(|| {
            let known = self.tsx.terrains.join(", ");
            format!("no terrain \"{terrain}\" (it has: {known})")
        })?;
        let mut listed: [Vec<u32>; 16] = Default::default();
        for (id, corners) in &self.tsx.corners {
            if corners.iter().flatten().all(|&t| t == index) {
                let mix = trpg_content::tileset::mix(corners.map(|c| c.is_some()));
                listed[usize::from(mix)].push(*id);
            }
        }
        let picked = |ids: &Vec<u32>| ids.get(pick).or(ids.last()).copied();
        let mut tiles = [None; 16];
        // Mix 0 is no corner at all: never a picture.
        for mix in 1..16 {
            tiles[mix] = picked(&listed[mix]);
        }
        Ok(tiles)
    }
}

/// Whether sheet `name` is a plain sheet (`….png`), not a Tiled one.
fn is_plain(name: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("png"))
}

/// How bright a colour is, 0 to 255 000: the usual weights of red, green
/// and blue.
fn brightness(rgb: [u8; 3]) -> u32 {
    299 * u32::from(rgb[0]) + 587 * u32::from(rgb[1]) + 114 * u32::from(rgb[2])
}

/// Recolours the RGBA8 `pixels` in `color`: each pixel that isn't clear
/// takes the colour, as bright as the pixel was (so shading stays).
pub fn tint(pixels: &mut [u8], color: [u8; 3]) {
    let full = brightness(color).max(1);
    let (pixels, _) = pixels.as_chunks_mut::<4>();
    for pixel in pixels.iter_mut().filter(|p| p[3] != 0) {
        let own = brightness([pixel[0], pixel[1], pixel[2]]);
        for (channel, c) in pixel.iter_mut().zip(color) {
            let scaled = u32::from(c) * own / full;
            *channel = u8::try_from(scaled).unwrap_or(u8::MAX);
        }
    }
}

/// Which picture of a sheet a cell of the packed image is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cut {
    /// One tile, by its id.
    Tile(u32),
    /// A tile joined from a block's quarters ([`LayerMap::Block`]): the
    /// block's top-left tile, and the mix of sides (up, right, down,
    /// left) where the terrain ends.
    Joined { block: u32, ends: u8 },
    /// Tile `over` drawn over tile `under` ([`LayerMap::Patch`]'s
    /// diagonal mixes).
    Pair { under: u32, over: u32 },
    /// A picture joined from four quarters ([`LayerMap::Quarters`]): for
    /// its up-left, up-right, down-left and down-right quarter, the cell
    /// whose opposite quarter goes there, or nothing.
    Quarters([Option<u32>; 4]),
    /// A [`Cut::Quarters`] [`FACED`] pixels tall, the half of a face cell
    /// nearest the point (see [`Face::under`]) hung below each of its
    /// lower two quarters that has one; with a `shadow`, its silhouette
    /// in that colour (RGBA).
    Faced {
        quarters: [Option<u32>; 4],
        face: [Option<u32>; 2],
        shadow: Option<[u8; 4]>,
    },
    /// A tile's left half, and that half mirrored on its right
    /// ([`LayerMap::Walls`]: a wall running up and down).
    Mirrored(u32),
    /// Two tiles, the first over the second: a picture two tiles tall
    /// ([`LayerMap::Standing`]).
    Stacked([u32; 2]),
    /// Nothing: a tile all see-through, of no sheet ([`LayerMap::Tiles`]
    /// without a `tile`).
    Blank,
}

/// `cells` each moved `places` along.
const fn moved<const N: usize>(cells: [Option<u32>; N], places: u32) -> [Option<u32>; N] {
    let mut moved = cells;
    let mut i = 0;
    while i < N {
        if let Some(cell) = moved[i] {
            moved[i] = Some(cell + places);
        }
        i += 1;
    }
    moved
}

impl Cut {
    /// The column of its tile furthest right, in a sheet `columns` tiles
    /// across: a block's is two past its top-left tile's.
    fn right_column(self, columns: u32) -> u32 {
        match self {
            Self::Tile(tile) | Self::Mirrored(tile) => tile % columns,
            Self::Joined { block, .. } => block % columns + 2,
            Self::Pair { under, over } => (under % columns).max(over % columns),
            Self::Quarters(cells) => cells
                .iter()
                .flatten()
                .map(|c| c % columns)
                .max()
                .unwrap_or(0),
            Self::Faced { quarters, face, .. } => quarters
                .iter()
                .chain(&face)
                .flatten()
                .map(|c| c % columns)
                .max()
                .unwrap_or(0),
            Self::Stacked([top, bottom]) => (top % columns).max(bottom % columns),
            Self::Blank => 0,
        }
    }

    /// How many cells of the packed image it takes, down: two for a
    /// picture taller than a tile.
    const fn down(self) -> u32 {
        if matches!(self, Self::Faced { .. } | Self::Stacked(_)) {
            2
        } else {
            1
        }
    }

    /// The same picture `places` tiles to the right in its sheet: its
    /// frame of a [`Strip`].
    const fn along(self, places: u32) -> Self {
        match self {
            Self::Tile(tile) => Self::Tile(tile + places),
            Self::Joined { block, ends } => Self::Joined {
                block: block + places,
                ends,
            },
            Self::Pair { under, over } => Self::Pair {
                under: under + places,
                over: over + places,
            },
            Self::Quarters(cells) => Self::Quarters(moved(cells, places)),
            Self::Faced {
                quarters,
                face,
                shadow,
            } => Self::Faced {
                quarters: moved(quarters, places),
                face: moved(face, places),
                shadow,
            },
            Self::Mirrored(tile) => Self::Mirrored(tile + places),
            Self::Stacked([top, bottom]) => Self::Stacked([top + places, bottom + places]),
            Self::Blank => Self::Blank,
        }
    }
}

/// [`Cut::Mirrored`]: tile `tile` of `sheet`, its right half its left
/// half mirrored. RGBA8, row by row.
fn mirrored(sheet: &Sheet, tile: u32) -> Result<Vec<u8>, String> {
    let mut out = sheet.tile(tile)?;
    let side = usize::try_from(TILE).unwrap_or(0);
    let (pixels, _) = out.as_chunks_mut::<4>();
    for row in pixels.chunks_exact_mut(side) {
        for x in 0..side / 2 {
            row[side - 1 - x] = row[x];
        }
    }
    Ok(out)
}

/// [`Cut::Blank`]: a tile all see-through. RGBA8, row by row.
fn blank() -> Vec<u8> {
    let side = usize::try_from(TILE).unwrap_or(0);
    vec![0; side * side * 4]
}

/// [`Cut::Stacked`]: tile `top` of `sheet` over tile `bottom`, a picture
/// two tiles tall. RGBA8, row by row.
fn stacked(sheet: &Sheet, [top, bottom]: [u32; 2]) -> Result<Vec<u8>, String> {
    let mut out = sheet.tile(top)?;
    out.extend(sheet.tile(bottom)?);
    Ok(out)
}

/// Tile `over` of `sheet` drawn over tile `under`: RGBA8, row by row.
fn paired(sheet: &Sheet, under: u32, over: u32) -> Result<Vec<u8>, String> {
    let mut out = sheet.tile(under)?;
    let top = sheet.tile(over)?;
    let (out_pixels, _) = out.as_chunks_mut::<4>();
    let (top_pixels, _) = top.as_chunks::<4>();
    for (pixel, drawn) in out_pixels.iter_mut().zip(top_pixels) {
        if drawn[3] == u8::MAX || pixel[3] == 0 {
            *pixel = *drawn;
        } else if drawn[3] != 0 {
            // A part see-through pixel over a solid one: mixed by its
            // alpha (the bought art has none, but a sheet might).
            let a = u32::from(drawn[3]);
            for (channel, top) in pixel.iter_mut().zip(drawn).take(3) {
                let mixed = (u32::from(*top) * a + u32::from(*channel) * (255 - a)) / 255;
                *channel = u8::try_from(mixed).unwrap_or(u8::MAX);
            }
            pixel[3] = pixel[3].max(drawn[3]);
        }
    }
    Ok(out)
}

/// A picture joined from quarters of `sheet`'s cells: for each of its
/// quarters (up-left, up-right, down-left, down-right), the opposite
/// quarter of the cell `cells` names there, or nothing. RGBA8, row by row.
fn quartered(sheet: &Sheet, cells: [Option<u32>; 4]) -> Result<Vec<u8>, String> {
    let half = usize::try_from(TILE / 2).unwrap_or(0);
    let line = usize::try_from(TILE * 4).unwrap_or(0);
    let mut out = vec![0; line * half * 2];
    for (to, cell) in cells.into_iter().enumerate() {
        let Some(cell) = cell else {
            continue;
        };
        let tile = sheet.tile(cell)?;
        let from = to ^ 3;
        let corner = |q: usize| (q >> 1) * half * line + (q & 1) * half * 4;
        for y in 0..half {
            let (src, dst) = (corner(from) + y * line, corner(to) + y * line);
            out[dst..dst + half * 4].copy_from_slice(&tile[src..src + half * 4]);
        }
    }
    Ok(out)
}

/// A [`Cut::Faced`] picture of `sheet`: the quarters `quarters` and, on
/// the lower two tiles, the halves of the face cells `face` nearest the
/// point, from the picture's middle down; its silhouette in `shadow` if
/// one is given. RGBA8, row by row, [`FACED`] rows.
fn faced(
    sheet: &Sheet,
    quarters: [Option<u32>; 4],
    face: [Option<u32>; 2],
    shadow: Option<[u8; 4]>,
) -> Result<Vec<u8>, String> {
    let half = usize::try_from(TILE / 2).unwrap_or(0);
    let line = usize::try_from(TILE * 4).unwrap_or(0);
    let mut out = quartered(sheet, quarters)?;
    out.resize(line * usize::try_from(FACED).unwrap_or(0), 0);
    for (side, cell) in face.into_iter().enumerate() {
        let Some(cell) = cell else {
            continue;
        };
        let tile = sheet.tile(cell)?;
        // The down-left tile's right half, the down-right's left half.
        let (from, to) = ((side ^ 1) * half * 4, side * half * 4);
        for (y, row) in tile.chunks_exact(line).enumerate() {
            let at = (half + y) * line + to;
            out[at..at + half * 4].copy_from_slice(&row[from..from + half * 4]);
        }
    }
    if let Some(colour) = shadow {
        let (pixels, _) = out.as_chunks_mut::<4>();
        for pixel in pixels.iter_mut().filter(|p| p[3] != 0) {
            *pixel = colour;
        }
    }
    Ok(out)
}

/// Changes each pixel of RGBA8 `pixels` whose colour is the first of one
/// of `pairs` to the second, its alpha kept.
pub fn recolour(pixels: &mut [u8], pairs: &[(Rgb, Rgb)]) {
    let (pixels, _) = pixels.as_chunks_mut::<4>();
    for pixel in pixels.iter_mut().filter(|p| p[3] != 0) {
        let own = (pixel[0], pixel[1], pixel[2]);
        if let Some(&(_, (r, g, b))) = pairs.iter().find(|(from, _)| *from == own) {
            pixel[..3].copy_from_slice(&[r, g, b]);
        }
    }
}

/// A tile joined from the quarters of the block of `sheet` whose top-left
/// tile is `block`, for a tile whose terrain ends on the mix of sides
/// `ends` (up, right, down, left): RGBA8, row by row.
fn joined(sheet: &Sheet, block: u32, ends: u8) -> Result<Vec<u8>, String> {
    if block % sheet.tsx.columns + 3 > sheet.tsx.columns {
        return Err(format!(
            "the block from tile {block} runs past the image's right edge"
        ));
    }
    let [up, right, down, left] = [8, 4, 2, 1].map(|bit| ends & bit != 0);
    // The block's row for the tile's top and bottom half, and its column
    // for the left and right half.
    let rows = [u32::from(!up), if down { 2 } else { 1 }];
    let columns = [u32::from(!left), if right { 2 } else { 1 }];
    let half = usize::try_from(TILE / 2).unwrap_or(0);
    let line = usize::try_from(TILE * 4).unwrap_or(0);
    let mut out = vec![0; line * half * 2];
    for (qy, row) in rows.into_iter().enumerate() {
        for (qx, column) in columns.into_iter().enumerate() {
            let tile = sheet.tile(block + row * sheet.tsx.columns + column)?;
            for y in qy * half..(qy + 1) * half {
                let at = y * line + qx * half * 4;
                out[at..at + half * 4].copy_from_slice(&tile[at..at + half * 4]);
            }
        }
    }
    Ok(out)
}

/// The packed image being made: each bought tile the mapping uses, once.
struct Packer<'a> {
    /// The packs, [`LIBRARIES_DIR`].
    dir: &'a Path,
    colors: &'a dyn Fn(&str) -> Option<[u8; 3]>,
    /// The plain sheets whose tiles move.
    strips: &'a BTreeMap<String, Strip>,
    /// The colours changed in each sheet as it is read.
    recolours: &'a BTreeMap<String, Vec<(Rgb, Rgb)>>,
    sheets: BTreeMap<String, Sheet>,
    /// The sheet, picture and tint of each picture, in the order packed.
    cells: Vec<(String, Cut, String)>,
    /// The pictures' pixels, in the same order.
    pixels: Vec<Vec<u8>>,
    /// The place in the image of each picture's top-left cell, counting
    /// cells row by row, in the same order.
    slots: Vec<u32>,
    /// Every cell of the image a picture takes.
    taken: BTreeSet<u32>,
}

impl Packer<'_> {
    /// Sheet `name`, read once.
    fn sheet(&mut self, name: &str) -> Result<&Sheet, String> {
        if !self.sheets.contains_key(name) {
            let mut sheet = Sheet::read(self.dir, name)?;
            if let Some(pairs) = self.recolours.get(name) {
                recolour(&mut sheet.rgba, pairs);
            }
            self.sheets.insert(name.to_owned(), sheet);
        }
        self.sheets.get(name).ok_or_else(|| name.to_owned())
    }

    /// Where tile `tile` of sheet `sheet`, recoloured in the palette's
    /// `tint_name` (empty: as it is), is in the packed image; packed now
    /// if it wasn't.
    fn cell(&mut self, sheet: &str, tile: u32, tint_name: &str) -> Result<At, String> {
        self.packed(sheet, Cut::Tile(tile), tint_name)
    }

    /// Where picture `cut` of sheet `sheet`, recoloured in the palette's
    /// `tint_name` (empty: as it is), is in the packed image; packed now
    /// if it wasn't.
    fn packed(&mut self, sheet: &str, cut: Cut, tint_name: &str) -> Result<At, String> {
        let key = (sheet.to_owned(), cut, tint_name.to_owned());
        let packed = self.cells.iter().position(|cell| *cell == key);
        let index = if let Some(index) = packed {
            index
        } else {
            // A blank is of no sheet: none is read for it.
            let pixels = if cut == Cut::Blank {
                Ok(blank())
            } else {
                let source = self.sheet(sheet)?;
                match cut {
                    Cut::Tile(tile) => source.tile(tile),
                    Cut::Joined { block, ends } => joined(source, block, ends),
                    Cut::Pair { under, over } => paired(source, under, over),
                    Cut::Quarters(cells) => quartered(source, cells),
                    Cut::Faced {
                        quarters,
                        face,
                        shadow,
                    } => faced(source, quarters, face, shadow),
                    Cut::Mirrored(tile) => mirrored(source, tile),
                    Cut::Stacked(cells) => stacked(source, cells),
                    Cut::Blank => Ok(blank()),
                }
            };
            let mut pixels = pixels.map_err(|e| format!("{sheet}: {e}"))?;
            if !tint_name.is_empty() {
                let color = (self.colors)(tint_name)
                    .ok_or_else(|| format!("no palette colour \"{tint_name}\""))?;
                tint(&mut pixels, color);
            }
            let slot = self.free(cut.down());
            self.cells.push(key);
            self.pixels.push(pixels);
            self.slots.push(slot);
            self.cells.len() - 1
        };
        Ok(self.at(index))
    }

    /// The first place, counting cells row by row, with `down` free cells
    /// from it down; taken now.
    fn free(&mut self, down: u32) -> u32 {
        let cells = |slot: u32| (0..down).map(move |k| slot + k * COLUMNS);
        let free = |slot: &u32| cells(*slot).all(|cell| !self.taken.contains(&cell));
        // Every cell past the last taken one is free, and those below it.
        let past = self.taken.last().map_or(0, |cell| cell + 1);
        let slot = (0..past).find(free).unwrap_or(past);
        self.taken.extend(cells(slot));
        slot
    }

    /// Where the `index`th picture packed is: its top-left cell.
    fn at(&self, index: usize) -> At {
        let slot = self.slots.get(index).copied().unwrap_or(u32::MAX);
        (slot % COLUMNS, slot / COLUMNS)
    }

    /// The frames of each packed tile its sheet animates, packed now: a
    /// cell for every frame, in the same tint, the loop turned to start on
    /// the tile itself (what stands still). A loop of one picture, or one
    /// that never shows the tile itself, is left out.
    fn moving(&mut self) -> Result<Vec<FramesText>, String> {
        let mut out = Vec::new();
        for index in 0..self.cells.len() {
            let (sheet, cut, tint_name) = self.cells[index].clone();
            if let Some(&strip) = self.strips.get(&sheet) {
                out.extend(self.stripped(index, &sheet, cut, &tint_name, strip)?);
                continue;
            }
            let Cut::Tile(tile) = cut else {
                continue;
            };
            let source = self.sheet(&sheet)?;
            let Some(mut frames) = source.tsx.animations.get(&tile).cloned() else {
                continue;
            };
            let Some(first) = frames.iter().position(|&(id, _)| id == tile) else {
                continue;
            };
            frames.rotate_left(first);
            let mut pictures = Vec::new();
            for &(id, _) in &frames {
                pictures.push(source.tile(id).map_err(|e| format!("{sheet}: {e}"))?);
            }
            if pictures.iter().all(|p| *p == pictures[0]) {
                continue;
            }
            let mut packed = Vec::new();
            for (id, ms) in frames {
                packed.push((self.packed(&sheet, Cut::Tile(id), &tint_name)?, ms.max(1)));
            }
            out.push(FramesText {
                at: self.at(index),
                frames: packed,
                minimal_todo: Some(MINIMAL_TODO.to_owned()),
            });
        }
        Ok(out)
    }

    /// The frames of packed cell `index`, picture `cut` of plain sheet
    /// `sheet` moving by `strip`, packed now; `None` if every frame is
    /// the same picture.
    fn stripped(
        &mut self,
        index: usize,
        sheet: &str,
        cut: Cut,
        tint_name: &str,
        strip: Strip,
    ) -> Result<Option<FramesText>, String> {
        let columns = self.sheet(sheet)?.tsx.columns;
        let last = (strip.frames.max(1) - 1) * strip.step;
        // Down the column, a frame past the bottom is outside the image
        // when it is packed; along the row, one past the right edge would
        // wrap onto the next row, so it is caught here.
        let across = !strip.step.is_multiple_of(columns);
        if across && cut.right_column(columns) + last >= columns {
            return Err(format!(
                "{sheet}: the frames of {cut:?} run past the image's right edge"
            ));
        }
        let mut packed = Vec::new();
        for k in 0..strip.frames {
            let frame = cut.along(k * strip.step);
            let at = self.packed(sheet, frame, tint_name);
            let at = at.map_err(|e| format!("{e} (frame {k} of a moving tile)"))?;
            packed.push((at, strip.ms.max(1)));
        }
        let pixels = |at: At| {
            let i = (0..self.cells.len()).find(|&i| self.at(i) == at);
            i.and_then(|i| self.pixels.get(i))
        };
        if packed
            .iter()
            .all(|&(at, _)| pixels(at) == pixels(packed[0].0))
        {
            return Ok(None);
        }
        Ok(Some(FramesText {
            at: self.at(index),
            frames: packed,
            minimal_todo: Some(MINIMAL_TODO.to_owned()),
        }))
    }

    /// The packed image: width, height and RGBA8 pixels.
    fn image(&self) -> (u32, u32, Vec<u8>) {
        let rows = self.taken.last().map_or(0, |cell| cell / COLUMNS + 1);
        let (width, height) = (COLUMNS * TILE, rows.max(1) * TILE);
        let row = usize::try_from(width * 4).unwrap_or(0);
        let mut rgba = vec![0; row * usize::try_from(height).unwrap_or(0)];
        let tile = usize::try_from(TILE).unwrap_or(0);
        for (index, pixels) in self.pixels.iter().enumerate() {
            let (column, top) = self.at(index);
            let column = usize::try_from(column).unwrap_or(0);
            let top = usize::try_from(top).unwrap_or(0);
            let (left, top) = (column * tile * 4, top * tile);
            for (y, line) in pixels.chunks_exact(tile * 4).enumerate() {
                let at = (top + y) * row + left;
                rgba[at..at + line.len()].copy_from_slice(line);
            }
        }
        (width, height, rgba)
    }
}

/// What an import makes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imported {
    /// The packed image: width, height and RGBA8 pixels.
    pub image: (u32, u32, Vec<u8>),
    /// The look of a map that names none: the tileset's own.
    pub own: LookText,
    /// The other looks, by name.
    pub others: Vec<(String, LookText)>,
    /// The packed tiles that move, and their frames.
    pub moving: Vec<FramesText>,
}

/// One layer of the mapping as the tileset file has it, its tiles packed.
/// `terrains` are ours, by id; `layers` the mapping's, for a shadow's.
fn layer(
    packer: &mut Packer,
    layer: &LayerMap,
    layers: &BTreeMap<String, LayerMap>,
    terrains: &[String],
) -> Result<LayerText, String> {
    match layer {
        LayerMap::Corners {
            of,
            sheet,
            terrain,
            pick,
            full,
            tint,
        } => {
            let table = packer
                .sheet(sheet)?
                .corners(terrain, *pick)
                .map_err(|e| format!("{sheet}: {e}"))?;
            let mut tiles = Vec::new();
            let last = if *full { 15 } else { 14 };
            for mix in 1..=last {
                if let Some(tile) = table[usize::from(mix)] {
                    tiles.push((mix, packer.cell(sheet, tile, tint)?));
                }
            }
            if tiles.is_empty() {
                return Err(format!("{sheet}: no tile has corners of \"{terrain}\""));
            }
            Ok(LayerText::Corners {
                of: of.clone(),
                tiles,
                place: Place::default(),
            })
        }
        LayerMap::Tiles { .. } => tiles_layer(packer, layer),
        LayerMap::Patch {
            of,
            sheet,
            patch,
            inside,
            full,
        } => patch_layer(packer, of, sheet, (*patch, *inside), *full),
        LayerMap::Quarters { .. } => {
            let quarters = Quarters::of(layer).ok_or("not a quarters layer")?;
            quarters_layer(packer, &quarters, None)
        }
        LayerMap::Walls { .. } => walls_layer(packer, layer),
        LayerMap::Faces { .. } | LayerMap::Standing { .. } => from_points(packer, layer),
        LayerMap::Shadow { of, colour, offset } => {
            let quarters = layers.get(of).and_then(Quarters::of);
            let quarters = quarters.filter(|q| q.face.is_some());
            let quarters = quarters.ok_or_else(|| shadow_problem(of))?;
            shadow_layer(packer, &quarters, *colour, *offset)
        }
        LayerMap::Block {
            of,
            sheet,
            block,
            tint,
        } => {
            let mut join = |ends| {
                packer.packed(
                    sheet,
                    Cut::Joined {
                        block: *block,
                        ends,
                    },
                    tint,
                )
            };
            // Where the terrain ends on no side: the block's middle.
            let at = join(0)?;
            let mut sides = Vec::new();
            for ends in 1..16 {
                sides.push((ends, join(ends)?));
            }
            // The terrain ends on a side whose neighbour is any other.
            let beside = terrains.iter().filter(|id| !of.contains(id)).cloned();
            Ok(LayerText::Tiles {
                of: of.clone(),
                at,
                beside: beside.collect(),
                sides,
            })
        }
    }
}

/// A [`LayerMap::Tiles`] as the tileset file has it, its pictures packed:
/// a see-through cell where it has no `tile`, and `near` for every mix of
/// sides with one of `beside` that `sides` doesn't name.
fn tiles_layer(packer: &mut Packer, layer: &LayerMap) -> Result<LayerText, String> {
    let LayerMap::Tiles {
        of,
        tile,
        beside,
        sides,
        near,
    } = layer
    else {
        return Err("not a tiles layer".to_owned());
    };
    let at = match tile {
        Some((sheet, tile)) => packer.cell(sheet, *tile, "")?,
        None => packer.packed("", Cut::Blank, "")?,
    };
    let mut pictures = Vec::new();
    for (text, (sheet, tile)) in sides {
        let mix = parse_mix(text)
            .ok_or_else(|| format!("sides \"{text}\" must be four of `#` and `.`"))?;
        pictures.push((mix, packer.cell(sheet, *tile, "")?));
    }
    if let Some((sheet, tile)) = near {
        let near = packer.cell(sheet, *tile, "")?;
        for mix in 1..16 {
            if !pictures.iter().any(|&(m, _)| m == mix) {
                pictures.push((mix, near));
            }
        }
    }
    pictures.sort_unstable();
    Ok(LayerText::Tiles {
        of: of.clone(),
        at,
        beside: beside.clone(),
        sides: pictures,
    })
}

/// A [`LayerMap::Walls`] as the tileset file has it: a tile layer looking
/// at its own terrains beside it, a pillar's cap unless [`WallTop::of`]
/// says across or up and down.
fn walls_layer(packer: &mut Packer, layer: &LayerMap) -> Result<LayerText, String> {
    let LayerMap::Walls {
        of,
        sheet,
        across,
        side,
        pillar,
    } = layer
    else {
        return Err("not a walls layer".to_owned());
    };
    let mut sides = Vec::new();
    for mix in 0..16u8 {
        let cut = match WallTop::of([8, 4, 2, 1].map(|bit| mix & bit != 0)) {
            WallTop::Across => Cut::Tile(*across),
            WallTop::UpDown => Cut::Mirrored(*side),
            WallTop::Pillar => continue,
        };
        sides.push((mix, packer.packed(sheet, cut, "")?));
    }
    Ok(LayerText::Tiles {
        of: of.clone(),
        at: packer.cell(sheet, *pillar, "")?,
        beside: of.clone(),
        sides,
    })
}

/// A [`LayerMap::Faces`] or [`LayerMap::Standing`] as the tileset file
/// has it: a picture by each of its tiles, drawn from the point at the
/// tile's top-left ([`by_tile`]): a face moved a tile down onto the tile
/// below, a standing picture two tiles tall moved up onto the tile above.
fn from_points(packer: &mut Packer, layer: &LayerMap) -> Result<LayerText, String> {
    match layer {
        LayerMap::Faces {
            of,
            sheet,
            across,
            pillar,
            skip,
        } => {
            let cut = |places| face(places, *across, *pillar).map(Cut::Tile);
            // From the wall tile's top-left point to the tile below it.
            let offset = (half(TILE), half(3 * TILE));
            let place = Place {
                size: None,
                offset,
                skip: skip.clone(),
            };
            by_tile(packer, of, sheet, cut, place)
        }
        LayerMap::Standing {
            of,
            sheet,
            cells,
            skip,
        } => {
            let cut = |[.., down_right]: [bool; 4]| down_right.then_some(Cut::Stacked(*cells));
            // From the tile's top-left point to the tile above it.
            let offset = (half(TILE), -half(TILE));
            let place = Place {
                size: Some((TILE, 2 * TILE)),
                offset,
                skip: skip.clone(),
            };
            by_tile(packer, of, sheet, cut, place)
        }
        _ => Err("not a layer of pictures by tiles".to_owned()),
    }
}

/// A [`LayerMap::Quarters`], borrowed.
struct Quarters<'a> {
    of: &'a [String],
    sheet: &'a str,
    kit: &'a Kit,
    face: Option<Face>,
    skip: &'a [String],
}

impl<'a> Quarters<'a> {
    /// `layer`, if it is a [`LayerMap::Quarters`].
    fn of(layer: &'a LayerMap) -> Option<Self> {
        match layer {
            LayerMap::Quarters {
                of,
                sheet,
                kit,
                face,
                skip,
            } => Some(Self {
                of,
                sheet,
                kit,
                face: *face,
                skip,
            }),
            _ => None,
        }
    }
}

/// A [`LayerMap::Shadow`] of `layer` as the tileset file has it: its
/// pictures packed as silhouettes in `(r, g, b, a)`, moved by `offset`.
fn shadow_layer(
    packer: &mut Packer,
    layer: &Quarters,
    (r, g, b, a): (u8, u8, u8, u8),
    offset: (i32, i32),
) -> Result<LayerText, String> {
    let mut text = quarters_layer(packer, layer, Some([r, g, b, a]))?;
    if let LayerText::Corners { place, .. } = &mut text {
        place.offset = offset;
    }
    Ok(text)
}

/// What is wrong with a shadow of layer `of`: it isn't a `Quarters`
/// layer with a face.
fn shadow_problem(of: &str) -> String {
    format!("a shadow is of a `Quarters` layer with a face, and \"{of}\" isn't one")
}

/// A [`LayerMap::Quarters`] as the tileset file has it, its pictures
/// packed: one for each mix of corners that draws anything, with the face
/// below if it has one, as silhouettes in `shadow` if one is given.
fn quarters_layer(
    packer: &mut Packer,
    layer: &Quarters,
    shadow: Option<[u8; 4]>,
) -> Result<LayerText, String> {
    let mut tiles = Vec::new();
    for mix in 1..16u8 {
        let places = [8, 4, 2, 1].map(|bit| mix & bit != 0);
        let quarters = layer.kit.quarters(places);
        let face = layer.face.map_or([None; 2], |f| f.under(places));
        if quarters.iter().chain(&face).all(Option::is_none) {
            continue;
        }
        let cut = if layer.face.is_some() {
            Cut::Faced {
                quarters,
                face,
                shadow,
            }
        } else {
            Cut::Quarters(quarters)
        };
        tiles.push((mix, packer.packed(layer.sheet, cut, "")?));
    }
    let place = Place {
        size: layer.face.map(|_| (TILE, FACED)),
        offset: (0, 0),
        skip: layer.skip.to_vec(),
    };
    Ok(LayerText::Corners {
        of: layer.of.to_vec(),
        tiles,
        place,
    })
}

/// A [`LayerMap::Patch`] as the tileset file has it, its pictures packed:
/// the patch and the inside corners from their top-left tiles.
fn patch_layer(
    packer: &mut Packer,
    of: &[String],
    sheet: &str,
    (patch, inside): (u32, u32),
    full: bool,
) -> Result<LayerText, String> {
    let columns = packer.sheet(sheet)?.tsx.columns;
    for (what, block) in [("patch", patch), ("inside corners", inside)] {
        if block % columns + 3 > columns {
            return Err(format!(
                "{sheet}: the {what} from tile {block} runs past the image's right edge"
            ));
        }
    }
    let at = |block: u32, (column, row): (u32, u32)| block + row * columns + column;
    let mut tiles = Vec::new();
    for (corners, in_inside, places) in PATCH {
        let mix = trpg_content::tileset::mix(corners);
        if mix == 15 && !full {
            continue;
        }
        let block = if in_inside { inside } else { patch };
        let cut = match places {
            [one] => Cut::Tile(at(block, *one)),
            [under, over] => Cut::Pair {
                under: at(block, *under),
                over: at(block, *over),
            },
            _ => continue,
        };
        tiles.push((mix, packer.packed(sheet, cut, "")?));
    }
    tiles.sort_unstable();
    Ok(LayerText::Corners {
        of: of.to_vec(),
        tiles,
        place: Place::default(),
    })
}

/// An error naming (as `what`) the first of `names` that isn't one of
/// `terrains`.
fn known<'a>(
    what: &str,
    names: impl IntoIterator<Item = &'a String>,
    terrains: &[String],
) -> Result<(), String> {
    let unknown = names.into_iter().find(|name| !terrains.contains(name));
    unknown.map_or(Ok(()), |name| {
        Err(format!("{what}: terrain \"{name}\" is not in terrain.ron"))
    })
}

/// What is wrong with `mapping` that shows without the bought files, the
/// first problem found: a terrain that isn't one of `terrains` (ours, by
/// id), a mix of sides that isn't one, a look a map can't name, no look
/// for a map that names none, a look without a tile for one of our
/// terrains, or a look listing a layer the mapping lacks.
pub fn check(mapping: &Mapping, terrains: &[String]) -> Result<(), String> {
    known("terrain", mapping.terrain.keys(), terrains)?;
    for (name, layer) in &mapping.layers {
        let what = format!("layer \"{name}\"");
        match layer {
            LayerMap::Corners { of, .. }
            | LayerMap::Block { of, .. }
            | LayerMap::Patch { of, .. }
            | LayerMap::Walls { of, .. } => {
                known(&what, of, terrains)?;
            }
            LayerMap::Quarters { of, skip, .. }
            | LayerMap::Faces { of, skip, .. }
            | LayerMap::Standing { of, skip, .. } => {
                known(&what, of.iter().chain(skip), terrains)?;
            }
            LayerMap::Shadow { of, .. } => {
                let quarters = mapping.layers.get(of).and_then(Quarters::of);
                if quarters.is_none_or(|q| q.face.is_none()) {
                    return Err(format!("{what}: {}", shadow_problem(of)));
                }
            }
            LayerMap::Tiles {
                of, beside, sides, ..
            } => {
                known(&what, of.iter().chain(beside), terrains)?;
                if let Some(text) = sides.keys().find(|text| parse_mix(text).is_none()) {
                    return Err(format!(
                        "{what}: sides \"{text}\" must be four of `#` and `.`"
                    ));
                }
            }
        }
    }
    for (sheet, strip) in &mapping.moving {
        if !is_plain(sheet) {
            return Err(format!(
                "moving \"{sheet}\": only a plain sheet (….png) moves this way"
            ));
        }
        if strip.frames < 2 || strip.step == 0 || strip.ms == 0 {
            return Err(format!(
                "moving \"{sheet}\": it needs 2 frames or more, a step and a time"
            ));
        }
    }
    if !mapping.looks.contains_key(DEFAULT_TILES) {
        return Err(format!("the mapping has no look \"{DEFAULT_TILES}\""));
    }
    for (name, look) in &mapping.looks {
        let what = format!("look \"{name}\"");
        if !TILE_LOOKS.contains(&name.as_str()) {
            return Err(format!("{what} is not a look a map can name"));
        }
        known(&what, look.terrain.keys(), terrains)?;
        let has =
            |id: &&String| look.terrain.contains_key(*id) || mapping.terrain.contains_key(*id);
        if let Some(id) = terrains.iter().find(|id| !has(id)) {
            return Err(format!("{what}: terrain \"{id}\" has no tile"));
        }
        let lacking = look
            .layers
            .iter()
            .find(|l| !mapping.layers.contains_key(*l));
        if let Some(layer) = lacking {
            return Err(format!("{what}: no layer \"{layer}\""));
        }
        if let Some(trees) = &look.trees {
            known(&format!("{what}: trees"), &trees.of, terrains)?;
            if name == DEFAULT_TILES {
                return Err(format!("{what}: only a look a map names has trees"));
            }
            if trees.sheets.is_empty() || trees.frames < 2 || trees.ms == 0 {
                return Err(format!(
                    "{what}: trees need a sheet, 2 frames or more and a time"
                ));
            }
        }
    }
    Ok(())
}

/// Imports the looks of `mapping` from the bought packs under `dir`
/// ([`LIBRARIES_DIR`]):
/// packs the tiles they use and writes them out as tileset looks, each
/// with a tile for every one of `terrains` (ours, in `terrain.ron`'s
/// order). `colors` gives a palette colour by name, for tints.
pub fn import(
    dir: &Path,
    mapping: &Mapping,
    terrains: &[String],
    colors: &dyn Fn(&str) -> Option<[u8; 3]>,
) -> Result<Imported, String> {
    check(mapping, terrains)?;
    let mut packer = Packer {
        dir,
        colors,
        strips: &mapping.moving,
        recolours: &mapping.recolour,
        sheets: BTreeMap::new(),
        cells: Vec::new(),
        pixels: Vec::new(),
        slots: Vec::new(),
        taken: BTreeSet::new(),
    };
    let mut looks = Vec::new();
    // The look of a map that names none first: its tiles lead the image.
    let own_first = |(name, _): &(&String, &LookMap)| *name != DEFAULT_TILES;
    let mut ordered: Vec<(&String, &LookMap)> = mapping.looks.iter().collect();
    ordered.sort_by_key(own_first);
    for (name, look) in ordered {
        let what = format!("look \"{name}\"");
        let mut text = LookText::default();
        for id in terrains {
            let tile = look.terrain.get(id).or_else(|| mapping.terrain.get(id));
            let (sheet, tile) =
                tile.ok_or_else(|| format!("{what}: terrain \"{id}\" has no tile"))?;
            let at = packer.cell(sheet, *tile, "");
            text.terrain
                .push((id.clone(), at.map_err(|e| format!("{what}: {e}"))?));
        }
        for layer_name in &look.layers {
            let map = mapping.layers.get(layer_name);
            let map = map.ok_or_else(|| format!("{what}: no layer \"{layer_name}\""))?;
            let made = layer(&mut packer, map, &mapping.layers, terrains);
            text.layers
                .push(made.map_err(|e| format!("layer \"{layer_name}\": {e}"))?);
        }
        looks.push((name.clone(), text));
    }
    let moving = packer.moving()?;
    let mut image = packer.image();
    let mut trees = TreePacker::below(&mut image);
    for (name, text) in &mut looks {
        if let Some(map) = mapping.looks.get(name).and_then(|l| l.trees.as_ref()) {
            let made = trees.look(dir, map);
            text.trees = Some(made.map_err(|e| format!("look \"{name}\": trees: {e}"))?);
        }
    }
    let mut looks = looks.into_iter();
    let own = looks.next().filter(|(name, _)| name == DEFAULT_TILES);
    let (_, own) = own.ok_or_else(|| format!("the mapping has no look \"{DEFAULT_TILES}\""))?;
    Ok(Imported {
        image,
        own,
        others: looks.collect(),
        moving,
    })
}

/// The trees' frames (ticket 1356), packed below the tiles of an image
/// [`COLUMNS`] cells wide: each frame from the top-left of the next free
/// cells along a shelf as tall as its tallest frame.
struct TreePacker<'a> {
    image: &'a mut (u32, u32, Vec<u8>),
    /// The shelf's top row and the next free column, in cells.
    at: (u32, u32),
    /// The shelf's height in cells.
    shelf: u32,
}

impl<'a> TreePacker<'a> {
    /// A packer adding to `image`, below what it holds.
    fn below(image: &'a mut (u32, u32, Vec<u8>)) -> Self {
        let rows = image.1 / TILE;
        Self {
            image,
            at: (rows, 0),
            shelf: 0,
        }
    }

    /// Packs `pixels` (RGBA8, `size` px) and returns its top-left cell.
    fn pack(&mut self, size: (u32, u32), pixels: &[u8]) -> At {
        let cells = |px: u32| px.div_ceil(TILE);
        let (across, down) = (cells(size.0), cells(size.1));
        if self.at.1 + across > COLUMNS {
            self.at = (self.at.0 + self.shelf, 0);
            self.shelf = 0;
        }
        let (row, column) = self.at;
        self.at.1 += across;
        self.shelf = self.shelf.max(down);
        let (width, height, rgba) = &mut *self.image;
        // The image grows down to the frame's bottom, if it isn't there.
        *height = (*height).max((row + down) * TILE);
        rgba.resize(usize::try_from(*height * *width * 4).unwrap_or(0), 0);
        let line = usize::try_from(size.0 * 4).unwrap_or(0);
        for (y, src) in (0..).zip(pixels.chunks_exact(line.max(1))) {
            let x = column * TILE;
            let at = usize::try_from(((row * TILE + y) * *width + x) * 4).unwrap_or(0);
            rgba[at..at + line].copy_from_slice(src);
        }
        (column, row)
    }

    /// Packs the trees of `map` from the sheets under `dir`: each sheet's
    /// frames as a kind of big tree, and each halved as a half-size one.
    fn look(&mut self, dir: &Path, map: &TreesMap) -> Result<TreesText, String> {
        let (fw, fh) = map.frame;
        if fw % 2 != 0 || fh % 2 != 0 || fw == 0 || fh == 0 || fw > COLUMNS * TILE {
            return Err(format!(
                "a frame of {fw}×{fh} px doesn't halve or doesn't fit"
            ));
        }
        let mut sheets = Vec::new();
        for name in &map.sheets {
            let sheet = Sheet::read(dir, name)?;
            let frames = tree_frames(&sheet, map).map_err(|e| format!("{name}: {e}"))?;
            sheets.push(frames);
        }
        let ms = map.ms.max(1);
        let mut kind = |size, foot, frames: &[Vec<u8>]| TreeKindText {
            size,
            foot,
            frames: frames.iter().map(|f| (self.pack(size, f), ms)).collect(),
        };
        let big: Vec<TreeKindText> = sheets
            .iter()
            .map(|frames| kind(map.frame, map.foot, frames))
            .collect();
        let half_foot = (map.foot.0 / 2, map.foot.1 / 2);
        let half = sheets
            .iter()
            .map(|frames| {
                let halved: Vec<Vec<u8>> = frames.iter().map(|f| halved(f, map.frame)).collect();
                kind((fw / 2, fh / 2), half_foot, &halved)
            })
            .collect();
        Ok(TreesText {
            of: map.of.clone(),
            big,
            half,
            minimal_todo: Some(MINIMAL_TREE_TODO.to_owned()),
        })
    }
}

/// The frames of a tree `sheet` as `map` lays them out: `map.frames` of
/// `map.frame` px, row by row from the top-left, as RGBA8 pixels.
fn tree_frames(sheet: &Sheet, map: &TreesMap) -> Result<Vec<Vec<u8>>, String> {
    let (fw, fh) = map.frame;
    let (width, height) = sheet.size;
    let across = width / fw;
    // `check` has made sure there are two frames or more.
    if across == 0 || map.frames.div_ceil(across) * fh > height {
        return Err(format!(
            "{} frames of {fw}×{fh} px don't fit the {width}×{height} px image",
            map.frames
        ));
    }
    let line = usize::try_from(fw * 4).unwrap_or(0);
    let frame = |k: u32| {
        let (left, top) = ((k % across) * fw, (k / across) * fh);
        let mut out = Vec::new();
        for y in top..top + fh {
            let at = usize::try_from((y * width + left) * 4).unwrap_or(0);
            out.extend_from_slice(&sheet.rgba[at..at + line]);
        }
        out
    };
    Ok((0..map.frames).map(frame).collect())
}

/// `pixels` (RGBA8, `w` × `h` px, both even) at half the size, each two by
/// two block one pixel (ticket 1356: ELV's trees, halved). A block of two
/// or more solid pixels is solid, their colours averaged, so the
/// picture's edge stays crisp; any other is its pixels' colour as clear as
/// the block is on the whole (the shadow's soft edge).
pub fn halved(pixels: &[u8], (w, h): (u32, u32)) -> Vec<u8> {
    let at = |x: u32, y: u32| {
        let i = usize::try_from((y * w + x) * 4).unwrap_or(0);
        pixels
            .get(i..i + 4)
            .map_or([0; 4], |p| [p[0], p[1], p[2], p[3]])
    };
    let mut out = Vec::new();
    for y in (0..h).step_by(2) {
        for x in (0..w).step_by(2) {
            let block = [at(x, y), at(x + 1, y), at(x, y + 1), at(x + 1, y + 1)];
            let solid: Vec<[u8; 4]> = block.iter().copied().filter(|p| p[3] == 255).collect();
            let (picked, alpha) = if solid.len() >= 2 {
                (solid, 255)
            } else {
                let seen: Vec<[u8; 4]> = block.iter().copied().filter(|p| p[3] > 0).collect();
                let sum: u32 = block.iter().map(|p| u32::from(p[3])).sum();
                (seen, u8::try_from(sum / 4).unwrap_or(u8::MAX))
            };
            // A block with no solid pixel and nearly clear is clear.
            if alpha == 0 {
                out.extend_from_slice(&[0; 4]);
                continue;
            }
            let n = u32::try_from(picked.len()).unwrap_or(1);
            let mean = |c: usize| {
                let sum: u32 = picked.iter().map(|p| u32::from(p[c])).sum();
                u8::try_from(sum / n).unwrap_or(u8::MAX)
            };
            out.extend_from_slice(&[mean(0), mean(1), mean(2), alpha]);
        }
    }
    out
}

/// The bundle path of the packed image.
fn image_path() -> String {
    format!("tilesets/{TILESET_ID}.png")
}

/// The tileset file: `imported`'s looks, then `sprites`' unit pictures.
pub fn tileset(imported: &Imported, sprites: &[MapSprite]) -> String {
    let mut out = format!(
        "// Written by `cargo xtask tileset-import` (ticket 0437). Do not edit: change\n\
         // {MAPPING_PATH} (the terrain) or the table in\n\
         // crates/xtask/src/map_sprite_import.rs (the units) and run it again.\n\
         (\n    id: \"{TILESET_ID}\",\n    image: \"{}\",\n    tile_px: ({TILE}, {TILE}),\n",
        image_path()
    );
    out.push_str(&tileset_ron::look(&imported.own, "    "));
    out.push_str(&tileset_ron::looks(&imported.others));
    out.push_str(&tileset_ron::frames(&imported.moving));
    out.push_str(&map_sprite_import::units(sprites));
    out.push_str(")\n");
    out
}

/// Which bought tiles each look's terrains and layers use, one per line.
pub fn list(mapping: &Mapping) -> String {
    let tile = |(sheet, tile): &TileRef| format!("{sheet} tile {tile}");
    let mut out = String::new();
    for (name, look) in &mapping.looks {
        // Writing to a `String` can't fail.
        let _ = writeln!(out, "look {name}");
        let mut own = mapping.terrain.clone();
        own.extend(look.terrain.clone());
        for (id, at) in &own {
            let _ = writeln!(out, "  {id:<16} {}", tile(at));
        }
        for layer_name in &look.layers {
            let described = match mapping.layers.get(layer_name) {
                Some(LayerMap::Corners {
                    of,
                    sheet,
                    terrain,
                    tint,
                    ..
                }) => {
                    let tinted = if tint.is_empty() {
                        String::new()
                    } else {
                        format!(", recoloured {tint}")
                    };
                    format!(
                        "between tiles of {}: {sheet} \"{terrain}\"{tinted}",
                        of.join(", ")
                    )
                }
                Some(LayerMap::Tiles {
                    of, tile: at, near, ..
                }) => {
                    let at = at.as_ref().map_or_else(|| "nothing".to_owned(), tile);
                    let near = near
                        .as_ref()
                        .map_or_default(|n| format!(", by them {}", tile(n)));
                    format!("on {}: {at}{near}", of.join(", "))
                }
                Some(LayerMap::Walls { of, sheet, .. }) => {
                    format!("wall tops on {}: {sheet}", of.join(", "))
                }
                Some(LayerMap::Faces { of, sheet, .. }) => {
                    format!("wall faces below {}: {sheet}", of.join(", "))
                }
                Some(LayerMap::Standing {
                    of, sheet, cells, ..
                }) => {
                    let [top, bottom] = cells;
                    format!(
                        "standing on {}: {sheet} tiles {top} over {bottom}",
                        of.join(", ")
                    )
                }
                Some(LayerMap::Block {
                    of, sheet, block, ..
                }) => format!("on {}: {sheet} block from tile {block}", of.join(", ")),
                Some(LayerMap::Patch {
                    of, sheet, patch, ..
                }) => format!(
                    "between tiles of {}: {sheet} patch from tile {patch}",
                    of.join(", ")
                ),
                Some(LayerMap::Quarters {
                    of, sheet, face, ..
                }) => {
                    let recoloured = if mapping.recolour.contains_key(sheet) {
                        ", recoloured"
                    } else {
                        ""
                    };
                    let face = face.map_or_default(|f| {
                        let Face {
                            left,
                            middle,
                            right,
                        } = f;
                        format!(", a face below from tiles {left}, {middle}, {right}")
                    });
                    format!(
                        "between tiles of {}: {sheet} quarters{recoloured}{face}",
                        of.join(", ")
                    )
                }
                Some(LayerMap::Shadow { of, offset, .. }) => {
                    let (x, y) = offset;
                    format!("the shadow of layer {of}, moved ({x}, {y})")
                }
                None => "(no such layer)".to_owned(),
            };
            let _ = writeln!(out, "  layer {layer_name:<10} {described}");
        }
    }
    out
}

/// Reads the mapping file under repo root `root`.
pub fn read_mapping(root: &Path) -> Result<Mapping, String> {
    let path = root.join(MAPPING_PATH);
    let text = fs::read_to_string(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    ron::from_str(&text).map_err(|e| format!("{MAPPING_PATH}: {e}"))
}

/// Runs the command under repo root `root`: imports the mapping's looks
/// and writes the packed image and the tileset file naming them and
/// `sprites`. Nothing is written unless everything was read.
pub fn run(root: &Path, sprites: &[MapSprite]) -> Result<String, String> {
    let library = root.join(LIBRARY_DIR);
    let game = root.join(GAME_DIR);
    if !library.is_dir() || !game.is_dir() {
        return Err(format!(
            "{LIBRARY_DIR}/ or {GAME_DIR}/ is missing: run `cargo shuyi private-assets --library` first"
        ));
    }
    let mapping = read_mapping(root)?;
    let joined = |errors: Vec<trpg_content::ContentError>| {
        let lines: Vec<String> = errors.iter().map(ToString::to_string).collect();
        lines.join("\n")
    };
    let palette = trpg_content::palette::load().map_err(joined)?;
    let terrain = TerrainDef::load(Some(&palette)).map_err(joined)?;
    let terrains: Vec<String> = terrain
        .display
        .terrains
        .iter()
        .map(|t| t.id.clone())
        .collect();
    let packs = root.join(LIBRARIES_DIR);
    let imported = import(&packs, &mapping, &terrains, &|name| palette.get(name))?;
    let (width, height, rgba) = &imported.image;
    let png = encode_png(*width, *height, rgba)?;
    let ron_path = format!("tilesets/{TILESET_ID}.ron");
    let files = [
        (image_path(), png),
        (ron_path.clone(), tileset(&imported, sprites).into_bytes()),
    ];
    for (bundle_path, bytes) in files {
        let path = game.join(&bundle_path);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
        }
        fs::write(&path, bytes).map_err(|e| format!("writing {}: {e}", path.display()))?;
    }
    Ok(format!(
        "tileset-import: wrote {GAME_DIR}/{} ({width}×{height}) and {GAME_DIR}/{ron_path} \
         ({} moving tiles)",
        image_path(),
        imported.moving.len()
    ))
}

#[cfg(test)]
mod tests;
