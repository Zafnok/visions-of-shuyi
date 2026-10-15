//! Combat pictures (ADR-0056): the still, full-body picture a fighter is
//! shown as in the combat scene, and the backdrops behind the two of them.
//!
//! A battler is one sidecar file, `assets/battlers/<id>.ron`, naming one
//! PNG, which way the picture faces and how big it is drawn. Its id is a
//! class id, a character id, or [`DEFAULT_BATTLER`]. The backdrops are one
//! file, `assets/battlebacks/battlebacks.ron`. `content` reads only each
//! image's size, from the image table; `app` decodes and draws it. The
//! formats are documented in `assets/battlers/README.md`.
use crate::prelude::*;

use alloc::collections::BTreeMap;

use serde::Deserialize;

use crate::bundle;
use crate::error::ContentError;
use crate::image::{ImageId, ImageInfo, ImageTable};
use crate::lighting::Lighting;
use crate::ron_loader::parse_ron;
use crate::terrain::TerrainDisplayTable;

/// Directory of battler files inside the asset bundle. A sidecar's image
/// path is relative to it.
pub const BATTLERS_DIR: &str = "battlers";
/// File extension of a battler's sidecar file.
pub const SIDECAR_EXTENSION: &str = ".ron";
/// The battler of a fighter whose character and class have none.
pub const DEFAULT_BATTLER: &str = "default";
/// The biggest a picture may be once scaled, in its own pixels, width ×
/// height: what the combat scene has room for.
pub const MAX_SCALED_PX: (u32, u32) = (240, 144);
/// Directory of the backdrops inside the asset bundle.
pub const BACKDROPS_DIR: &str = "battlebacks";
/// The backdrops' file inside the asset bundle.
pub const BACKDROPS_PATH: &str = "battlebacks/battlebacks.ron";
/// The size every backdrop has, in pixels, width × height.
pub const BACKDROP_PX: (u32, u32) = (336, 248);

/// Which way a picture's fighter looks, as the file has it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Facing {
    /// Toward the left edge.
    Left,
    /// Toward the right edge.
    Right,
}

/// A validated battler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Battler {
    /// The class or character it is the picture of (also the file stem).
    pub id: String,
    /// The picture.
    pub image: ImageId,
    /// The picture's size in pixels.
    pub size: ImageInfo,
    /// Which way it faces as drawn in the file.
    pub faces: Facing,
    /// How big it is drawn next to a picture at `(1, 1)`: numerator and
    /// denominator, from 1 to 2 times.
    pub scale: (u32, u32),
}

impl Battler {
    /// The picture's size at its own scale times `zoom`, in pixels (whole,
    /// rounded down).
    pub fn scaled(&self, zoom: u32) -> (u32, u32) {
        let (num, den) = self.scale;
        let scaled = |v: u32| v * num * zoom / den.max(1);
        (scaled(self.size.width), scaled(self.size.height))
    }
}

/// Every battler, by id.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BattlerTable {
    /// The battlers by id (file stem).
    pub battlers: BTreeMap<String, Battler>,
    /// The backdrops.
    pub backdrops: Backdrops,
}

impl BattlerTable {
    /// The first battler `ids` names, else [`DEFAULT_BATTLER`]'s.
    pub fn pick<'a>(&self, ids: impl IntoIterator<Item = &'a str>) -> Option<&Battler> {
        ids.into_iter()
            .find_map(|id| self.battlers.get(id))
            .or_else(|| self.battlers.get(DEFAULT_BATTLER))
    }
}

/// The pictures behind the fighters.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Backdrops {
    /// The backdrop of a terrain that names none.
    pub default: Option<ImageId>,
    /// Backdrops by terrain string id.
    pub terrain: BTreeMap<String, ImageId>,
    /// The pictures drawn for another light than noon's, by the light's
    /// name (ticket 0438). A light with none here shows the ones above.
    pub lights: BTreeMap<String, BackdropSet>,
}

/// The backdrops drawn for one light.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BackdropSet {
    /// The backdrop of a terrain that names none.
    pub default: Option<ImageId>,
    /// Backdrops by terrain string id.
    pub terrain: BTreeMap<String, ImageId>,
}

impl BackdropSet {
    /// The backdrop of a fight on terrain `terrain` (the default's if
    /// `None`).
    fn of(&self, terrain: Option<&str>) -> Option<ImageId> {
        let own = terrain.and_then(|t| self.terrain.get(t));
        own.copied().or(self.default)
    }
}

impl Backdrops {
    /// The backdrop of a fight on terrain `terrain`.
    pub fn of(&self, terrain: &str) -> Option<ImageId> {
        self.terrain.get(terrain).copied().or(self.default)
    }

    /// The backdrop of a fight on terrain `terrain` (the default's if
    /// `None`) in the light called `light`, and whether it is a picture
    /// drawn for that light. If it isn't (the file has no pictures for
    /// the light), it is noon's picture, still to be shaded.
    pub fn in_light(&self, terrain: Option<&str>, light: &str) -> Option<(ImageId, bool)> {
        let drawn = self.lights.get(light).and_then(|set| set.of(terrain));
        let noon = || terrain.map_or(self.default, |t| self.of(t));
        drawn
            .map(|id| (id, true))
            .or_else(|| Some((noon()?, false)))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Sidecar {
    id: String,
    image: String,
    faces: Facing,
    #[serde(default = "own_size")]
    scale: (u32, u32),
}

/// A picture drawn at its own size.
fn own_size() -> (u32, u32) {
    (1, 1)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BackdropFile {
    default: String,
    #[serde(default)]
    terrain: BTreeMap<String, String>,
    #[serde(default)]
    lights: BTreeMap<String, RawSet>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSet {
    default: String,
    #[serde(default)]
    terrain: BTreeMap<String, String>,
}

/// Loads every sidecar in the bundle's battlers directory and the
/// backdrops' file, checking each image against `images`, each
/// backdrop's terrain against `terrain` and each light against
/// `lighting` (each if it loaded). Reports every error of every file.
pub fn load_all(
    images: &ImageTable,
    terrain: Option<&TerrainDisplayTable>,
    lighting: Option<&Lighting>,
) -> Result<BattlerTable, Vec<ContentError>> {
    let mut battlers = BTreeMap::new();
    let mut errors = Vec::new();
    for path in bundle::files_in(BATTLERS_DIR) {
        let Some(stem) = path
            .strip_prefix(BATTLERS_DIR)
            .and_then(|p| p.strip_prefix('/'))
            .and_then(|p| p.strip_suffix(SIDECAR_EXTENSION))
        else {
            continue;
        };
        let file = bundle::display_path(path);
        let result = bundle::file(path)
            .ok_or_else(|| vec![ContentError::new(&file, "file is not valid UTF-8")])
            .and_then(|source| parse_battler(&file, stem, source, images));
        match result {
            Ok(b) => {
                battlers.insert(stem.to_owned(), b);
            }
            Err(e) => errors.extend(e),
        }
    }
    if let Some(e) = missing_default(&battlers) {
        errors.push(e);
    }
    let file = bundle::display_path(BACKDROPS_PATH);
    let backdrops = bundle::file(BACKDROPS_PATH)
        .ok_or_else(|| {
            vec![ContentError::new(
                &file,
                "file is missing or not valid UTF-8",
            )]
        })
        .and_then(|source| parse_backdrops(&file, source, images, terrain, lighting));
    assemble(battlers, errors, backdrops)
}

/// The table of `battlers` and `backdrops`, or every error: the
/// battlers' `errors`, then the backdrops'.
fn assemble(
    battlers: BTreeMap<String, Battler>,
    mut errors: Vec<ContentError>,
    backdrops: Result<Backdrops, Vec<ContentError>>,
) -> Result<BattlerTable, Vec<ContentError>> {
    let backdrops = backdrops.unwrap_or_else(|e| {
        errors.extend(e);
        Backdrops::default()
    });
    if errors.is_empty() {
        Ok(BattlerTable {
            battlers,
            backdrops,
        })
    } else {
        Err(errors)
    }
}

/// The error of a set of battlers without [`DEFAULT_BATTLER`].
fn missing_default(battlers: &BTreeMap<String, Battler>) -> Option<ContentError> {
    let file = format!("{DEFAULT_BATTLER}{SIDECAR_EXTENSION}");
    let path = bundle::display_path(&format!("{BATTLERS_DIR}/{file}"));
    (!battlers.contains_key(DEFAULT_BATTLER)).then(|| {
        ContentError::new(
            path,
            "missing: the picture of a fighter whose class and character have none",
        )
    })
}

/// Parses sidecar `source` of the file with stem `stem` (errors attributed
/// to `file`), checking its image against `images`. Reports every problem.
pub fn parse_battler(
    file: &str,
    stem: &str,
    source: &str,
    images: &ImageTable,
) -> Result<Battler, Vec<ContentError>> {
    let sidecar: Sidecar = parse_ron(file, source).map_err(|e| vec![e])?;
    let mut errors = Vec::new();
    if sidecar.id != stem {
        errors.push(ContentError::new(
            file,
            format!(
                "id \"{}\" doesn't match the file name; name the file {}{SIDECAR_EXTENSION}",
                sidecar.id, sidecar.id
            ),
        ));
    }
    let (num, den) = sidecar.scale;
    let scale_ok = den > 0 && num >= den && num <= 2 * den;
    if !scale_ok {
        errors.push(ContentError::new(
            file,
            format!("scale ({num}, {den}) must be from 1 to 2 times: (1, 1) to (2, 1)"),
        ));
    }
    let path = format!("{BATTLERS_DIR}/{}", sidecar.image);
    let shown = bundle::display_path(&path);
    let found = images.id(&path).and_then(|id| Some((id, images.info(id)?)));
    if found.is_none() {
        errors.push(ContentError::new(
            file,
            format!("image: {shown} is not a PNG image in the assets"),
        ));
    }
    let Some((image, size)) = found.filter(|_| errors.is_empty()) else {
        return Err(errors);
    };
    let battler = Battler {
        id: sidecar.id,
        image,
        size,
        faces: sidecar.faces,
        scale: sidecar.scale,
    };
    let (w, h) = battler.scaled(1);
    let (max_w, max_h) = MAX_SCALED_PX;
    if w > max_w || h > max_h {
        return Err(vec![ContentError::new(
            file,
            format!(
                "image: {shown} is {w}×{h} px at scale ({num}, {den}); a combat picture must fit {max_w}×{max_h} px"
            ),
        )]);
    }
    Ok(battler)
}

/// Parses the backdrops' file `source` (errors attributed to `file`),
/// checking each image against `images`, each terrain id against
/// `terrain` and each light's name against `lighting` (each if given).
/// Reports every problem.
pub fn parse_backdrops(
    file: &str,
    source: &str,
    images: &ImageTable,
    terrain: Option<&TerrainDisplayTable>,
    lighting: Option<&Lighting>,
) -> Result<Backdrops, Vec<ContentError>> {
    let raw: BackdropFile = parse_ron(file, source).map_err(|e| vec![e])?;
    let mut errors = Vec::new();
    let noon = parse_set("", &raw.default, &raw.terrain, images, terrain);
    let noon = noon.unwrap_or_else(|problems| {
        errors.extend(problems);
        BackdropSet::default()
    });
    let mut lights = BTreeMap::new();
    for (light, set) in &raw.lights {
        let of = format!("light \"{light}\": ");
        if lighting.is_some_and(|known| known.get(light).is_none()) {
            errors.push(format!("{of}no such light"));
        }
        match parse_set(&of, &set.default, &set.terrain, images, terrain) {
            Ok(set) => {
                lights.insert(light.clone(), set);
            }
            Err(problems) => errors.extend(problems),
        }
    }
    if errors.is_empty() {
        Ok(Backdrops {
            default: noon.default,
            terrain: noon.terrain,
            lights,
        })
    } else {
        Err(errors
            .into_iter()
            .map(|message| ContentError::new(file, message))
            .collect())
    }
}

/// One light's backdrops, `default` and `by_terrain` as the file has
/// them, or every problem with them in plain words, each starting with
/// `of` (which light, if not noon).
fn parse_set(
    of: &str,
    default: &str,
    by_terrain: &BTreeMap<String, String>,
    images: &ImageTable,
    terrain: Option<&TerrainDisplayTable>,
) -> Result<BackdropSet, Vec<String>> {
    let mut errors = Vec::new();
    let mut check = |what: &str, relative: &str| {
        let result = check_backdrop(relative, images);
        if let Err(problem) = &result {
            errors.push(format!("{of}{what}: {problem}"));
        }
        result.ok()
    };
    let default = check("default", default);
    let mut pictures = BTreeMap::new();
    for (id, relative) in by_terrain {
        if let Some(image) = check(&format!("terrain \"{id}\""), relative) {
            pictures.insert(id.clone(), image);
        }
    }
    if let Some(known) = terrain {
        for id in by_terrain.keys().filter(|id| known.id_of(id).is_none()) {
            errors.push(format!("{of}terrain \"{id}\": no such terrain"));
        }
    }
    if errors.is_empty() {
        Ok(BackdropSet {
            default,
            terrain: pictures,
        })
    } else {
        Err(errors)
    }
}

/// The backdrop at `relative` (to the backdrops directory), or what is
/// wrong with it in plain words: it must be in `images` and be
/// [`BACKDROP_PX`] big.
fn check_backdrop(relative: &str, images: &ImageTable) -> Result<ImageId, String> {
    let path = format!("{BACKDROPS_DIR}/{relative}");
    let shown = bundle::display_path(&path);
    let found = images.id(&path).and_then(|id| Some((id, images.info(id)?)));
    let (id, size) = found.ok_or_else(|| format!("{shown} is not a PNG image in the assets"))?;
    let (w, h) = BACKDROP_PX;
    if (size.width, size.height) != (w, h) {
        return Err(format!(
            "{shown} is {}×{} px; a backdrop is {w}×{h} px",
            size.width, size.height
        ));
    }
    Ok(id)
}

#[cfg(test)]
mod tests;
