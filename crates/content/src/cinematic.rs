//! Cinematic files (`assets/cinematics/*.ron`, ticket 0817, ADR-0060): a
//! list of shots and when each starts, played by `trpg_ui::cinematic`. The
//! format is described in `assets/cinematics/README.md`; the loader checks
//! every rule listed there.

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use trpg_core::{BattleDef, ClassTable, TerrainTable, UnitId};

use crate::audio::AudioManifest;
use crate::battle::bare;
use crate::bundle;
use crate::character::CharacterTable;
use crate::error::ContentError;
use crate::ron_loader::parse_ron;

mod action;

pub use action::{Act, ActionDef, ShotAction};
use action::{RawAction, RawActionRef};

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
        /// What its units act out (ticket 0827), if anything: without it
        /// they stand where they start.
        action: Option<ShotAction>,
    },
    /// The title's logo over its backdrop.
    Logo,
}

/// What a cinematic file is checked against.
#[derive(Debug, Clone, Copy)]
pub struct CinematicRefs<'a> {
    /// The battles, by id.
    pub battles: &'a BTreeMap<String, BattleDef>,
    /// The music cues.
    pub audio: &'a AudioManifest,
    /// Each battle's enemies that its file gives an `id`, by battle id
    /// ([`crate::battle::load_unit_names`]).
    pub units: &'a BTreeMap<String, BTreeMap<String, UnitId>>,
    /// The classes: what a unit's movement type is.
    pub classes: &'a ClassTable,
    /// The terrain rules: which tiles a movement type can enter.
    pub terrain: &'a TerrainTable,
    /// The characters: which class a player slot's unit has.
    pub characters: &'a CharacterTable,
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
    /// Action lists by name, for shots to share.
    #[serde(default)]
    actions: BTreeMap<String, Vec<RawAction>>,
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
        /// Written bare: `action_from: 2.5`.
        #[serde(default, deserialize_with = "bare")]
        action_from: Option<f32>,
        /// Written bare: `action: [ … ]` or `action: "name"`.
        #[serde(default, deserialize_with = "bare")]
        action: Option<RawActionRef>,
    },
    Logo,
}

/// Loads every cinematic file in the bundle, keyed by id (the file stem),
/// checking its references against `refs`. Reports every error of every
/// file.
pub fn load_all(
    refs: &CinematicRefs<'_>,
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
        match from_source(&file, stem, source, refs) {
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
///   the battle's map, or whose zoom isn't 1 to [`MAX_ZOOM`];
/// - a map pan's action: an `action_from` that isn't a time, or with no
///   `action`; an `action` naming no list of the file's `actions`; a
///   named list no shot uses; and every problem of the list itself
///   (`action::check`), a named one's reported once for each battle it is
///   used on.
pub fn from_source(
    file: &str,
    stem: &str,
    source: &str,
    refs: &CinematicRefs<'_>,
) -> Result<CinematicDef, Vec<ContentError>> {
    let (battles, audio) = (refs.battles, refs.audio);
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
    let mut lists = ActionLists {
        table: &raw.actions,
        refs,
        named: BTreeMap::new(),
        used: BTreeSet::new(),
    };
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
        let mut checked = shot.checked();
        if let Shot::MapPan { action, .. } = &mut checked {
            let mut problems = Vec::new();
            *action = lists.of_shot(n, shot, &mut problems);
            problems.into_iter().for_each(&mut err);
        }
        shots.push(ShotDef {
            at_ms: at_ms.unwrap_or(0),
            shot: checked,
        });
    }
    lists.unused().into_iter().for_each(&mut err);
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

/// The action lists of one cinematic file, as its shots use them.
struct ActionLists<'a, 'r> {
    /// The file's `actions`: lists by name.
    table: &'a BTreeMap<String, Vec<RawAction>>,
    refs: &'a CinematicRefs<'r>,
    /// Each named list as checked against each battle it is used on, by
    /// (name, battle).
    named: BTreeMap<(&'a str, &'a str), Vec<ActionDef>>,
    /// The named lists a shot has used.
    used: BTreeSet<&'a str>,
}

impl<'a> ActionLists<'a, '_> {
    /// The action of `shot` (shot `n` of the file, counted from 1), if it
    /// is a map pan with one, adding what is wrong with it to `problems`.
    /// A named list's own problems are added the first time a shot uses it
    /// on a battle; nothing is checked on a battle that doesn't exist.
    fn of_shot(
        &mut self,
        n: usize,
        shot: &'a RawShot,
        problems: &mut Vec<String>,
    ) -> Option<ShotAction> {
        let RawShot::MapPan {
            battle,
            action_from,
            action,
            ..
        } = shot
        else {
            return None;
        };
        let from_ms = action_from.map(millis);
        if let Some((from, None)) = action_from.zip(from_ms) {
            problems.push(format!("shot {n}: action_from {from} is not a time"));
        }
        if action.is_none() && action_from.is_some() {
            problems.push(format!("shot {n}: action_from without an action"));
        }
        let refs = self.refs;
        let def = refs.battles.get(battle);
        let actions = match (action.as_ref()?, def) {
            (RawActionRef::List(list), Some(def)) => {
                let (actions, wrong) = action::check(list, def, refs);
                problems.extend(wrong.iter().map(|w| format!("shot {n}: {w}")));
                actions
            }
            (RawActionRef::Named(name), def) => {
                let Some((name, list)) = self.table.get_key_value(name) else {
                    problems.push(format!("shot {n}: no actions named \"{name}\""));
                    return None;
                };
                self.used.insert(name);
                let key = (name.as_str(), battle.as_str());
                if let Entry::Vacant(unchecked) = self.named.entry(key) {
                    let (actions, wrong) = action::check(list, def?, refs);
                    let on = format!("actions \"{name}\" (on battle \"{battle}\")");
                    problems.extend(wrong.iter().map(|w| format!("{on}: {w}")));
                    unchecked.insert(actions);
                }
                self.named.get(&key)?.clone()
            }
            (RawActionRef::List(_), None) => return None,
        };
        Some(ShotAction {
            from_ms: from_ms.flatten().unwrap_or(0),
            actions,
        })
    }

    /// A problem for each named list no shot has used (so never checked).
    fn unused(&self) -> Vec<String> {
        let unused = self
            .table
            .keys()
            .filter(|name| !self.used.contains(name.as_str()));
        unused
            .map(|name| format!("actions \"{name}\": no shot uses it"))
            .collect()
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
        ..
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
    /// The shot as the game keeps it, without its action.
    fn checked(&self) -> Shot {
        match self {
            RawShot::MapPan {
                battle,
                from,
                to,
                zoom,
                ..
            } => Shot::MapPan {
                battle: battle.clone(),
                from: TilePoint::new(from.0, from.1),
                to: TilePoint::new(to.0, to.1),
                zoom: *zoom,
                action: None,
            },
            RawShot::Logo => Shot::Logo,
        }
    }
}

#[cfg(test)]
mod tests;
