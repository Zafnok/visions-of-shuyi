//! Cinematic files (`assets/cinematics/*.ron`, ticket 0817, ADR-0060): a
//! list of shots and when each starts, played by `trpg_ui::cinematic`. The
//! format is described in `assets/cinematics/README.md`; the loader checks
//! every rule listed there.

use std::collections::BTreeMap;

use serde::Deserialize;
use trpg_core::BattleDef;

use crate::audio::AudioManifest;
use crate::bundle;
use crate::error::ContentError;
use crate::ron_loader::parse_ron;

/// Directory of the cinematic files inside the asset bundle.
pub const CINEMATICS_DIR: &str = "cinematics";

/// Extension of a cinematic file.
const EXTENSION: &str = ".ron";

/// The biggest zoom of a shot: whole steps 1× to 4×
/// (`docs/design/title-screen.md`, *Intro cinematic*).
pub const MAX_ZOOM: u8 = 4;

/// The longest a cinematic that states its own length may be, in seconds.
pub const MAX_SECONDS: f32 = 3600.0;

/// How many parts of a tile a [`TilePoint`] counts in.
const TILE_PARTS: f32 = 1000.0;

/// One cinematic: its shots, in order, over a length of time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CinematicDef {
    /// The cinematic's id (its file stem).
    pub id: String,
    /// The music cue whose length is the cinematic's, if it follows one.
    pub music: Option<String>,
    /// How long it is, in milliseconds (never 0): the music cue's
    /// `length_ms`, or the length the file states.
    pub length_ms: u32,
    /// The shots, by start time; never empty, the first at 0. A shot lasts
    /// until the next one starts, the last until the end.
    pub shots: Vec<ShotDef>,
}

/// A shot and when it starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShotDef {
    /// Milliseconds from the cinematic's start.
    pub at_ms: u32,
    /// What is shown.
    pub shot: Shot,
}

/// What a shot shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shot {
    /// A battle's map, with its units where they start, the middle of the
    /// screen moving from one point to another.
    MapPan {
        /// The battle, by id (`assets/battles/`).
        battle: String,
        /// The point in the middle of the screen when the shot starts.
        from: TilePoint,
        /// The point in the middle of the screen when it ends.
        to: TilePoint,
        /// Screen pixels per map pixel: 1 to [`MAX_ZOOM`], the same for
        /// the whole shot.
        zoom: u8,
    },
    /// The title's logo over its backdrop.
    Logo,
}

/// A point on a map, in tiles: a whole number is the **middle** of that
/// tile, so `(4, 9)` is the middle of the tile at column 4, row 9, and
/// `(4.5, 9)` the edge between it and the tile on its right. Kept in
/// thousandths of a tile (content is compared with `Eq`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TilePoint {
    x: i32,
    y: i32,
}

impl TilePoint {
    /// The point `(x, y)`, in tiles, to the nearest thousandth. A
    /// coordinate that isn't a number counts as 0.
    pub fn new(x: f32, y: f32) -> Self {
        #[allow(clippy::cast_possible_truncation)] // saturating, far off any map
        let parts = |v: f32| (v * TILE_PARTS).round() as i32;
        Self {
            x: parts(x),
            y: parts(y),
        }
    }

    /// The point in tiles, across and down.
    #[allow(clippy::cast_precision_loss)] // thousandths of a map's few tiles
    pub fn tiles(self) -> (f32, f32) {
        (self.x as f32 / TILE_PARTS, self.y as f32 / TILE_PARTS)
    }

    /// Whether the point is on a map `width` × `height` tiles big: from
    /// the outer edge of its first tile to the outer edge of its last,
    /// each way.
    pub fn is_on(self, width: u16, height: u16) -> bool {
        let on = |v: i32, tiles: u16| (-500..=i32::from(tiles) * 1000 - 500).contains(&v);
        on(self.x, width) && on(self.y, height)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCinematic {
    id: String,
    length: RawLength,
    shots: Vec<RawShotAt>,
}

#[derive(Deserialize)]
enum RawLength {
    Music(String),
    Seconds(f32),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawShotAt {
    at: f32,
    shot: RawShot,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
enum RawShot {
    MapPan {
        battle: String,
        from: (f32, f32),
        to: (f32, f32),
        zoom: u8,
    },
    Logo,
}

/// Loads every cinematic file in the bundle, keyed by id (the file stem),
/// checking its references against `battles` and `audio`. Reports every
/// error of every file.
pub fn load_all(
    battles: &BTreeMap<String, BattleDef>,
    audio: &AudioManifest,
) -> Result<BTreeMap<String, CinematicDef>, Vec<ContentError>> {
    let mut cinematics = BTreeMap::new();
    let mut errors = Vec::new();
    for path in bundle::files_in(CINEMATICS_DIR) {
        let Some(stem) = path
            .strip_prefix(CINEMATICS_DIR)
            .and_then(|p| p.strip_prefix('/'))
            .and_then(|p| p.strip_suffix(EXTENSION))
        else {
            continue;
        };
        let file = bundle::display_path(path);
        let Some(source) = bundle::file(path) else {
            errors.push(ContentError::new(&file, "file is not valid UTF-8"));
            continue;
        };
        match from_source(&file, stem, source, battles, audio) {
            Ok(def) => {
                cinematics.insert(stem.to_owned(), def);
            }
            Err(e) => errors.extend(e),
        }
    }
    if errors.is_empty() {
        Ok(cinematics)
    } else {
        Err(errors)
    }
}

/// Parses and checks cinematic `source` (errors attributed to `file`,
/// whose name without its extension is `stem`). Reports every problem:
///
/// - the id isn't the file name;
/// - the length: a music cue that doesn't exist, or seconds that aren't
///   above 0 and at most [`MAX_SECONDS`];
/// - no shots; a first shot not at 0; a shot that doesn't start after the
///   one before it, or not before the end;
/// - a map pan whose battle doesn't exist, whose `from` or `to` isn't on
///   the battle's map, or whose zoom isn't 1 to [`MAX_ZOOM`].
pub fn from_source(
    file: &str,
    stem: &str,
    source: &str,
    battles: &BTreeMap<String, BattleDef>,
    audio: &AudioManifest,
) -> Result<CinematicDef, Vec<ContentError>> {
    let raw: RawCinematic = parse_ron(file, source).map_err(|e| vec![e])?;
    let mut errors = Vec::new();
    let mut err = |m: String| errors.push(ContentError::new(file, m));
    if raw.id != stem {
        err(format!(
            "id \"{}\" must match the file name \"{stem}\"",
            raw.id
        ));
    }
    let (music, length_ms) = match &raw.length {
        RawLength::Music(cue) => {
            let length = audio.music.get(cue).map(|m| m.length_ms);
            if length.is_none() {
                err(format!("length: no music cue \"{cue}\""));
            }
            (Some(cue.clone()), length)
        }
        RawLength::Seconds(seconds) => {
            let length = millis(*seconds).filter(|&ms| ms > 0 && *seconds <= MAX_SECONDS);
            if length.is_none() {
                err(format!(
                    "length: {seconds} seconds; it must be above 0 and at most {MAX_SECONDS}"
                ));
            }
            (None, length)
        }
    };
    if raw.shots.is_empty() {
        err("shots: a cinematic needs at least one shot".to_owned());
    }
    let mut shots = Vec::new();
    for (i, RawShotAt { at, shot }) in raw.shots.iter().enumerate() {
        let n = i + 1;
        let at_ms = millis(*at);
        match (at_ms, shots.last().map(|s: &ShotDef| s.at_ms)) {
            (None, _) => err(format!("shot {n}: at {at} is not a time")),
            (Some(ms), None) if ms != 0 => {
                err(format!("shot {n}: the first shot must be at 0.0, not {at}"));
            }
            (Some(ms), Some(before)) if ms <= before => err(format!(
                "shot {n}: at {at} must be after the shot before it; shots are in order of time"
            )),
            _ => {}
        }
        if let (Some(ms), Some(length)) = (at_ms, length_ms)
            && ms >= length
        {
            err(format!(
                "shot {n}: at {at} is not before the end ({} seconds)",
                seconds(length)
            ));
        }
        for problem in shot_problems(shot, battles) {
            err(format!("shot {n}: {problem}"));
        }
        shots.push(ShotDef {
            at_ms: at_ms.unwrap_or(0),
            shot: shot.checked(),
        });
    }
    match length_ms {
        Some(length_ms) if errors.is_empty() => Ok(CinematicDef {
            id: raw.id,
            music,
            length_ms,
            shots,
        }),
        _ => Err(errors),
    }
}

/// `seconds` in whole milliseconds, or `None` if it isn't a time: not a
/// number, or below 0.
fn millis(seconds: f32) -> Option<u32> {
    let ms = (f64::from(seconds) * 1000.0).round();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // checked
    (ms.is_finite() && (0.0..=f64::from(u32::MAX)).contains(&ms)).then_some(ms as u32)
}

/// `ms` in seconds, for a message.
fn seconds(ms: u32) -> f64 {
    f64::from(ms) / 1000.0
}

/// Every problem with `shot` on its own, in plain words.
fn shot_problems(shot: &RawShot, battles: &BTreeMap<String, BattleDef>) -> Vec<String> {
    let RawShot::MapPan {
        battle,
        from,
        to,
        zoom,
    } = shot
    else {
        return Vec::new();
    };
    let mut problems = Vec::new();
    match battles.get(battle) {
        None => problems.push(format!("no battle \"{battle}\"")),
        Some(def) => {
            let (w, h) = (def.map.tiles.width(), def.map.tiles.height());
            for (name, point) in [("from", from), ("to", to)] {
                let finite = point.0.is_finite() && point.1.is_finite();
                if !finite || !TilePoint::new(point.0, point.1).is_on(w, h) {
                    problems.push(format!(
                        "{name} ({}, {}) is not on the map of battle \"{battle}\" ({w}×{h} tiles)",
                        point.0, point.1
                    ));
                }
            }
        }
    }
    if !(1..=MAX_ZOOM).contains(zoom) {
        problems.push(format!("zoom {zoom}; a zoom is 1 to {MAX_ZOOM}"));
    }
    problems
}

impl RawShot {
    /// The shot as the game keeps it.
    fn checked(&self) -> Shot {
        match self {
            RawShot::MapPan {
                battle,
                from,
                to,
                zoom,
            } => Shot::MapPan {
                battle: battle.clone(),
                from: TilePoint::new(from.0, from.1),
                to: TilePoint::new(to.0, to.1),
                zoom: *zoom,
            },
            RawShot::Logo => Shot::Logo,
        }
    }
}

#[cfg(test)]
mod tests;
