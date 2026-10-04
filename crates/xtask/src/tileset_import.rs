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
//! It is the only reader of Tiled files: the game reads our RON and PNG.
//! The XML it needs is three kinds of tag and their attributes, read by
//! [`tags`]; no XML crate.
//!
//! The output is bought art (ADR-0040): it goes to the private assets
//! checkout, never into this repository.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use trpg_content::map::{DEFAULT_TILES, TILE_LOOKS};
use trpg_content::tileset::parse_mix;
use trpg_content::{PaletteDef, TerrainDef};

use crate::font_atlas::{decode_png, encode_png};
use crate::map_sprite_import::{self, GAME_DIR, MapSprite, TILESET_ID};
use crate::tileset_ron::{self, At, LayerText, LookText};

/// Shown for `--help` and after a bad argument.
pub const USAGE: &str = "usage: cargo xtask tileset-import [--list]\n\n\
Packs the bought tiles named by assets-src/tilesets/tiny_tales.ron, from\n\
assets-private/library/tiny-tales/tilesets/, into\n\
assets-private/game/tilesets/tiny_tales.png, and writes tiny_tales.ron beside it:\n\
the terrain looks, and the unit pictures of `map-sprite-import`'s table.\n\
--list  print which bought tiles each terrain and layer uses, and write nothing\n\n\
Run `cargo xtask private-assets --library` first. Afterwards commit and push in\n\
assets-private/, then run `cargo xtask private-assets --pin`.";

/// The sorted bought tilesets, relative to the repo root.
pub const LIBRARY_DIR: &str = "assets-private/library/tiny-tales/tilesets";
/// The mapping file, relative to the repo root.
pub const MAPPING_PATH: &str = "assets-src/tilesets/tiny_tales.ron";
/// A tile's side in the packed image, in pixels: the bought tiles'.
pub const TILE: u32 = 16;
/// Tiles per row of the packed image.
pub const COLUMNS: u32 = 16;

/// A tile of a bought sheet: the sheet (its `.tsx` under [`LIBRARY_DIR`],
/// without the extension) and the tile's id in it.
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
        /// The picture.
        tile: TileRef,
        /// Our terrains a neighbour is looked at for.
        #[serde(default)]
        beside: Vec<String>,
        /// The picture for a mix of sides (up, right, down, left being
        /// one of `beside`), written as four of `#` and `.`.
        #[serde(default)]
        sides: BTreeMap<String, TileRef>,
    },
}

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
    Ok(Tsx {
        image: source.clone(),
        clear,
        tile: (number("tilewidth")?, number("tileheight")?),
        columns: number("columns")?,
        terrains,
        corners,
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
    /// Reads sheet `name` (its `.tsx` without the extension) under `dir`.
    pub fn read(dir: &Path, name: &str) -> Result<Self, String> {
        let path = dir.join(format!("{name}.tsx"));
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

/// The packed image being made: each bought tile the mapping uses, once.
struct Packer<'a> {
    dir: &'a Path,
    colors: &'a dyn Fn(&str) -> Option<[u8; 3]>,
    sheets: BTreeMap<String, Sheet>,
    /// The sheet, tile and tint of each cell, in the image's order.
    cells: Vec<(String, u32, String)>,
    /// The cells' pixels, in the same order.
    pixels: Vec<Vec<u8>>,
}

impl Packer<'_> {
    /// Sheet `name`, read once.
    fn sheet(&mut self, name: &str) -> Result<&Sheet, String> {
        if !self.sheets.contains_key(name) {
            let sheet = Sheet::read(self.dir, name)?;
            self.sheets.insert(name.to_owned(), sheet);
        }
        self.sheets.get(name).ok_or_else(|| name.to_owned())
    }

    /// Where tile `tile` of sheet `sheet`, recoloured in the palette's
    /// `tint_name` (empty: as it is), is in the packed image; packed now
    /// if it wasn't.
    fn cell(&mut self, sheet: &str, tile: u32, tint_name: &str) -> Result<At, String> {
        let key = (sheet.to_owned(), tile, tint_name.to_owned());
        let packed = self.cells.iter().position(|cell| *cell == key);
        let index = if let Some(index) = packed {
            index
        } else {
            let cut = self.sheet(sheet)?.tile(tile);
            let mut pixels = cut.map_err(|e| format!("{sheet}: {e}"))?;
            if !tint_name.is_empty() {
                let color = (self.colors)(tint_name)
                    .ok_or_else(|| format!("no palette colour \"{tint_name}\""))?;
                tint(&mut pixels, color);
            }
            self.cells.push(key);
            self.pixels.push(pixels);
            self.cells.len() - 1
        };
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        Ok((index % COLUMNS, index / COLUMNS))
    }

    /// The packed image: width, height and RGBA8 pixels.
    fn image(&self) -> (u32, u32, Vec<u8>) {
        let count = u32::try_from(self.cells.len()).unwrap_or(u32::MAX);
        let (width, height) = (COLUMNS * TILE, count.div_ceil(COLUMNS).max(1) * TILE);
        let row = usize::try_from(width * 4).unwrap_or(0);
        let mut rgba = vec![0; row * usize::try_from(height).unwrap_or(0)];
        let tile = usize::try_from(TILE).unwrap_or(0);
        let columns = usize::try_from(COLUMNS).unwrap_or(1);
        for (index, pixels) in self.pixels.iter().enumerate() {
            let (left, top) = (index % columns * tile * 4, index / columns * tile);
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
}

/// One layer of the mapping as the tileset file has it, its tiles packed.
fn layer(packer: &mut Packer, layer: &LayerMap) -> Result<LayerText, String> {
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
            })
        }
        LayerMap::Tiles {
            of,
            tile: (sheet, tile),
            beside,
            sides,
        } => {
            let at = packer.cell(sheet, *tile, "")?;
            let mut pictures = Vec::new();
            for (text, (sheet, tile)) in sides {
                let mix = parse_mix(text)
                    .ok_or_else(|| format!("sides \"{text}\" must be four of `#` and `.`"))?;
                pictures.push((mix, packer.cell(sheet, *tile, "")?));
            }
            pictures.sort_unstable();
            Ok(LayerText::Tiles {
                of: of.clone(),
                at,
                beside: beside.clone(),
                sides: pictures,
            })
        }
    }
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
            LayerMap::Corners { of, .. } => known(&what, of, terrains)?,
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
    }
    Ok(())
}

/// Imports the looks of `mapping` from the Tiled sheets under `dir`:
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
        sheets: BTreeMap::new(),
        cells: Vec::new(),
        pixels: Vec::new(),
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
            let made = layer(&mut packer, map);
            text.layers
                .push(made.map_err(|e| format!("layer \"{layer_name}\": {e}"))?);
        }
        looks.push((name.clone(), text));
    }
    let mut looks = looks.into_iter();
    let own = looks.next().filter(|(name, _)| name == DEFAULT_TILES);
    let (_, own) = own.ok_or_else(|| format!("the mapping has no look \"{DEFAULT_TILES}\""))?;
    Ok(Imported {
        image: packer.image(),
        own,
        others: looks.collect(),
    })
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
                Some(LayerMap::Tiles { of, tile: at, .. }) => {
                    format!("on {}: {}", of.join(", "), tile(at))
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
            "{LIBRARY_DIR}/ or {GAME_DIR}/ is missing: run `cargo xtask private-assets --library` first"
        ));
    }
    let mapping = read_mapping(root)?;
    let joined = |errors: Vec<trpg_content::ContentError>| {
        let lines: Vec<String> = errors.iter().map(ToString::to_string).collect();
        lines.join("\n")
    };
    let palette = PaletteDef::load().map_err(joined)?;
    let terrain = TerrainDef::load(Some(&palette)).map_err(joined)?;
    let terrains: Vec<String> = terrain
        .display
        .terrains
        .iter()
        .map(|t| t.id.clone())
        .collect();
    let imported = import(&library, &mapping, &terrains, &|name| palette.get(name))?;
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
        "tileset-import: wrote {GAME_DIR}/{} ({width}×{height}) and {GAME_DIR}/{ron_path}",
        image_path()
    ))
}

#[cfg(test)]
mod tests;
