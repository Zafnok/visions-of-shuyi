//! Openings (`assets/openings/*.ron`, ticket 0734, ADR-0076): a few
//! characters placed on a place's map, and the walks they make there
//! before a conversation's next line, which a script asks for with
//! `@opening <id>`. The walks are a cinematic action's (ADR-0061), checked
//! the same way on the place's map: authored pictures, never the rules.
//! The format is described in `assets/openings/README.md`.
use crate::prelude::*;

use alloc::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use trpg_core::{CharacterId, ClassTable, Pos, TerrainTable, UnitId};

use super::action::{self, Actor, Heading, RawAction, Stage};
use super::{Act, ActionDef};
use crate::bundle;
use crate::character::CharacterTable;
use crate::error::ContentError;
use crate::map::MapDef;
use crate::ron_loader::parse_ron;

/// Directory of the opening files inside the asset bundle.
pub const OPENINGS_DIR: &str = "openings";

/// Extension of an opening file.
const EXTENSION: &str = ".ron";

/// The movement type someone without a class walks by (a speaker who
/// never fights, like a messenger): on foot.
const ON_FOOT: &str = "foot";

/// One opening: who stands where on which place, and how they walk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpeningDef {
    /// The opening's id (its file stem).
    pub id: String,
    /// The place it plays on: a map of `assets/maps/`, by id.
    pub map: String,
    /// Who is there, in the order they are written; never empty.
    pub cast: Vec<CastMember>,
    /// What they do, in order of time; never empty. Only walks.
    pub actions: Vec<ActionDef>,
    /// How long it plays, in milliseconds: until its last walk ends.
    pub length_ms: u32,
}

/// Someone an opening places on its map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CastMember {
    /// The unit id its walks name it by: its place in the cast, from 1.
    pub unit: UnitId,
    /// The character (`assets/data/characters.ron`).
    pub character: CharacterId,
    /// The tile they stand on when it starts.
    pub at: Pos,
    /// The way they face while they stand.
    pub facing: Heading,
}

/// What openings are checked against.
#[derive(Debug, Clone, Copy)]
pub struct OpeningRefs<'a> {
    /// The places an opening may play on.
    pub maps: &'a BTreeMap<String, MapDef>,
    /// Who may be in one.
    pub characters: &'a CharacterTable,
    /// Their classes: how each walks.
    pub classes: &'a ClassTable,
    /// Which tiles a movement type can enter.
    pub terrain: &'a TerrainTable,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawOpening {
    id: String,
    map: String,
    cast: Vec<RawCast>,
    action: Vec<RawAction>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCast {
    who: String,
    at: (i32, i32),
    #[serde(default)]
    facing: Heading,
}

/// Loads every opening file in the bundle, keyed by id (the file stem),
/// checking it against `refs`. Reports every error of every file.
pub fn load_all(refs: OpeningRefs<'_>) -> Result<BTreeMap<String, OpeningDef>, Vec<ContentError>> {
    let mut openings = BTreeMap::new();
    let mut errors = Vec::new();
    for path in bundle::files_in(OPENINGS_DIR) {
        let Some(stem) = path
            .strip_prefix(OPENINGS_DIR)
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
                openings.insert(stem.to_owned(), def);
            }
            Err(e) => errors.extend(e),
        }
    }
    if errors.is_empty() {
        Ok(openings)
    } else {
        Err(errors)
    }
}

/// Parses and checks opening `source` (errors attributed to `file`, whose
/// name without its extension is `stem`). Reports every problem:
///
/// - the id isn't the file name;
/// - the map isn't one of `assets/maps/`;
/// - nobody in the cast; someone who isn't a character, or is in it
///   twice; a tile that isn't on the map, that someone can't stand on (by
///   their class's movement type, on foot without a class), or where
///   someone else stands;
/// - every problem of the walks, as a cinematic's action has them
///   (`action::check`), on the map with the cast where it puts them; and
///   a strike: an opening has walks only.
pub fn from_source(
    file: &str,
    stem: &str,
    source: &str,
    refs: OpeningRefs<'_>,
) -> Result<OpeningDef, Vec<ContentError>> {
    let raw: RawOpening = parse_ron(file, source).map_err(|e| vec![e])?;
    let mut problems = Vec::new();
    if raw.id != stem {
        problems.push(format!(
            "id \"{}\" must match the file name \"{stem}\"",
            raw.id
        ));
    }
    let map = refs.maps.get(&raw.map);
    if map.is_none() {
        problems.push(format!(
            "map: no map \"{}\"; an opening plays on a map of assets/maps/ (its file name \
             without .map)",
            raw.map
        ));
    }
    if raw.cast.is_empty() {
        problems.push("cast: an opening needs someone in it".to_owned());
    }
    let cast = cast(&raw.cast, refs, &mut problems);
    let mut actions = Vec::new();
    if let Some(map) = map {
        let stage = Stage {
            map: &map.map,
            cast: on_map(&raw.cast, &cast, (&raw.map, map), refs, &mut problems),
            nobody: "in the opening's cast".to_owned(),
            strikes: false,
        };
        let (checked, wrong) = action::check(&raw.action, &stage, refs.terrain);
        actions = checked;
        problems.extend(wrong);
    }
    if !problems.is_empty() {
        return Err(problems
            .into_iter()
            .map(|p| ContentError::new(file, p))
            .collect());
    }
    let length_ms = actions.iter().map(end_ms).max().unwrap_or(0);
    Ok(OpeningDef {
        id: raw.id,
        map: raw.map,
        cast,
        actions,
        length_ms,
    })
}

/// When `action` ends, in milliseconds on the opening's clock.
fn end_ms(action: &ActionDef) -> u32 {
    match &action.act {
        Act::Walk { ms, .. } => action.at_ms.saturating_add(*ms),
        Act::Strike { .. } => action.at_ms,
    }
}

/// The cast as the game keeps it, each numbered from 1 in the order
/// written, adding a problem for each who isn't a character or is in it
/// twice.
fn cast(raw: &[RawCast], refs: OpeningRefs<'_>, problems: &mut Vec<String>) -> Vec<CastMember> {
    let mut seen = BTreeSet::new();
    let mut cast = Vec::new();
    for (unit, (n, member)) in (1..).zip(raw.iter().enumerate()) {
        let n = n + 1;
        let character = CharacterId(member.who.clone());
        if !refs.characters.can_speak(&character) {
            problems.push(format!("cast {n}: no character \"{}\"", member.who));
        }
        if !seen.insert(member.who.as_str()) {
            problems.push(format!("cast {n}: \"{}\" is in the cast twice", member.who));
        }
        cast.push(CastMember {
            unit: UnitId(unit),
            character,
            at: Pos::new(member.at.0, member.at.1),
            facing: member.facing,
        });
    }
    cast
}

/// The cast on `map`, by the names the walks call them (their character
/// ids), each walking by their class's movement type (on foot without a
/// class), adding a problem for each who stands off the map, where they
/// can't stand, or where someone else stands.
fn on_map<'a>(
    raw: &[RawCast],
    cast: &[CastMember],
    (id, map): (&str, &MapDef),
    refs: OpeningRefs<'a>,
    problems: &mut Vec<String>,
) -> BTreeMap<String, Actor<'a>> {
    let tiles = &map.map.tiles;
    let (width, height) = (tiles.width(), tiles.height());
    let mut stage = BTreeMap::new();
    let mut taken: BTreeMap<Pos, &str> = BTreeMap::new();
    for (n, (member, written)) in cast.iter().zip(raw).enumerate() {
        let n = n + 1;
        let def = refs.characters.characters.get(&member.character);
        let class = def.and_then(|d| refs.classes.get(&d.class));
        let mut actor = Actor::of_class(member.unit, member.at, class);
        if actor.walks.is_none() {
            let foot = refs.terrain.movement_type(ON_FOOT);
            actor.walks = foot.map(|movement| (movement, "person on foot".to_owned()));
        }
        let (x, y) = (member.at.x, member.at.y);
        match tiles.get(member.at) {
            None => problems.push(format!(
                "cast {n}: ({x}, {y}) is not on map \"{id}\" ({width}×{height} tiles)"
            )),
            Some(&terrain) => {
                let blocked = actor
                    .walks
                    .as_ref()
                    .filter(|(movement, _)| refs.terrain.move_cost(terrain, *movement).is_none());
                if let Some((_, what)) = blocked {
                    problems.push(format!("cast {n}: a {what} can't stand on ({x}, {y})"));
                }
            }
        }
        match taken.get(&member.at) {
            Some(other) => {
                problems.push(format!("cast {n}: ({x}, {y}) is where \"{other}\" stands"));
            }
            None => {
                taken.insert(member.at, &written.who);
            }
        }
        // Someone in it twice walks from where they are first written.
        stage.entry(written.who.clone()).or_insert(actor);
    }
    stage
}
