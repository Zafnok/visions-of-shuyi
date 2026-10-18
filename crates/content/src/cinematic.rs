//! Cinematic files (`assets/cinematics/*.ron`, ticket 0817, ADR-0060): a
//! list of shots and when each starts, played by `trpg_ui::cinematic`. The
//! format is described in `assets/cinematics/README.md`; the loader checks
//! every rule listed there.
//!
//! The file's frame (its id, length and the shots' times) is the engine's
//! (`shuyi_assets::cinematic`, ticket 1113); the shot kinds, their checks
//! and the `actions` they share are this game's ([`Shot`]).
use crate::prelude::*;

use alloc::collections::btree_map::Entry;
use alloc::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use shuyi_assets::cinematic::{self as frame, CinematicFile, ShotCheck};
use trpg_core::lead::{self, LEAD_ID, PORTRAIT_FEMALE, PORTRAIT_MALE, Part};
use trpg_core::{BattleDef, CharacterId, ClassTable, TerrainTable};

use crate::audio::AudioManifest;
use crate::battle::bare;
use crate::bundle;
use crate::character::CharacterTable;
use crate::dialogue::{
    BOX_LINES, BOX_WIDTH, DialogueTable, Present, STANDARD_EXPRESSIONS, Scene, Side, Step,
};
use crate::error::ContentError;
use crate::lang::text_width;
use crate::names::{Names, name_token};
use crate::portrait::PortraitTable;

mod action;
mod combat;
mod opening;

pub use action::{Act, ActionDef, Heading, ShotAction};
use action::{RawAction, RawActionRef, Stage};
pub use combat::{CombatShot, Fight, check_fights, fight};
pub use opening::{CastMember, OPENINGS_DIR, OpeningDef, OpeningRefs, load_all as load_openings};
pub use shuyi_assets::cinematic::MAX_SECONDS;
use shuyi_assets::cinematic::{millis, seconds};

/// Directory of the cinematic files inside the asset bundle.
pub const CINEMATICS_DIR: &str = "cinematics";

/// Extension of a cinematic file.
const EXTENSION: &str = ".ron";

/// The biggest zoom of a shot: whole steps 1× to 4×
/// (`docs/design/title-screen.md`, *Intro cinematic*).
pub const MAX_ZOOM: u8 = 4;

/// How many parts of a tile a [`TilePoint`] counts in.
const TILE_PARTS: f32 = 1000.0;

/// One cinematic: its shots, in order, over a length of time
/// (`shuyi_assets::cinematic`, with this game's [`Shot`]s).
pub type CinematicDef = frame::CinematicDef<Shot>;

/// A shot and when it starts.
pub type ShotDef = frame::ShotDef<Shot>;

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
    /// Text boxes of a dialogue scene, playing by themselves in the
    /// dialogue screen's look (ticket 0818).
    Talk {
        /// The scene, by id (`assets/dialogue/`).
        scene: String,
        /// The first text box shown, counted from 0 ([`opening_boxes`]).
        first: usize,
        /// How many text boxes are shown; at least 1.
        count: usize,
    },
    /// Characters' faces crossing the screen one after another, each with
    /// its name under it (ticket 0818).
    Characters {
        /// Who, in the order they cross.
        who: Vec<CharacterId>,
        /// The expression every face wears.
        expression: String,
        /// The side of the screen the faces come in from.
        from: Side,
    },
    /// One exchange of the battle's own combat scene (ticket 0832): the
    /// two fighters' pictures over the backdrop of the defender's ground,
    /// as a battle shows an attack with combat animations on. The rules
    /// work the combat out ([`fight`]); the file only says who attacks
    /// whom, from where and with which luck.
    Combat(CombatShot),
}

/// The text boxes a [`Shot::Talk`] counts in `scene`: its speech and
/// narration lines as they play with everyone there
/// ([`Present::Everyone`]), up to its first reply choice. The second value
/// is whether a reply choice ends them (and not the scene's end).
pub fn opening_boxes(scene: &Scene) -> (Vec<String>, bool) {
    let played = scene.resolved(&Present::Everyone);
    let mut boxes = Vec::new();
    for step in &played.steps {
        if matches!(step, Step::Choice { .. }) {
            return (boxes, true);
        }
        boxes.extend(step.text().map(str::to_owned));
    }
    (boxes, false)
}

/// What a cinematic's shots are checked against.
#[derive(Debug, Clone, Copy)]
pub struct Refs<'a> {
    /// The battles a map pan may name.
    pub battles: &'a BTreeMap<String, BattleDef>,
    /// The music a length may name.
    pub audio: &'a AudioManifest,
    /// The scenes a talk shot may name.
    pub scenes: &'a DialogueTable,
    /// The characters a characters shot may name.
    pub characters: &'a CharacterTable,
    /// Their portraits.
    pub portraits: &'a PortraitTable,
    /// The names table: how long a name token can get.
    pub names: &'a Names,
    /// The classes: what a walking unit's movement type is (0827).
    pub classes: &'a ClassTable,
    /// The terrain rules: which tiles a movement type can enter.
    pub terrain: &'a TerrainTable,
}

impl<'a> Refs<'a> {
    /// The tables of `content`.
    pub fn of(content: &'a crate::Content) -> Self {
        Self {
            battles: &content.battles,
            audio: &content.audio,
            scenes: &content.dialogue,
            characters: &content.characters,
            portraits: &content.portraits,
            names: &content.names,
            classes: &content.classes,
            terrain: &content.terrain.rules,
        }
    }
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
        let parts = |v: f32| libm::roundf(v * TILE_PARTS) as i32;
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

/// A cinematic file as written: the engine's frame over this game's shots
/// and their action lists by name.
type RawCinematic = CinematicFile<RawShot, Vec<RawAction>>;

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
    Talk {
        scene: String,
        first: usize,
        count: usize,
    },
    Characters {
        who: Vec<String>,
        expression: String,
        #[serde(default)]
        from: RawSide,
    },
    Combat {
        battle: String,
        attacker: String,
        target: String,
        /// Written bare: `from: (7, 3)`.
        #[serde(default, deserialize_with = "bare")]
        from: Option<(i32, i32)>,
        /// Written bare: `seed: 4`.
        #[serde(default, deserialize_with = "bare")]
        seed: Option<u64>,
    },
}

/// The side faces come in from; the right unless the file says otherwise
/// (the title's storyboard: right to left).
#[derive(Deserialize, Clone, Copy, Default)]
enum RawSide {
    Left,
    #[default]
    Right,
}

/// Loads every cinematic file in the bundle, keyed by id (the file stem),
/// checking its references against `refs`. Reports every error of every
/// file.
pub fn load_all(refs: Refs<'_>) -> Result<BTreeMap<String, CinematicDef>, Vec<ContentError>> {
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
/// - a talk shot whose scene doesn't exist, that shows no box, whose
///   boxes aren't all among the scene's [`opening_boxes`] (the scene has
///   fewer, or a reply choice comes first), or one of whose boxes may
///   not fit one page of the text box;
/// - a characters shot with nobody, with someone who isn't a character,
///   or with an expression one of their portraits lacks (for a character
///   without a portrait: one that isn't a standard expression);
/// - a combat shot whose battle doesn't exist, whose attacker or target
///   isn't on the battle's map at its start (named as an action names
///   them), that attacks itself, or whose `from` isn't on the map, is a
///   tile the attacker can't stand on or where another unit starts
///   (whether the rules allow the attack: [`check_fights`], once all the
///   content is loaded).
pub fn from_source(
    file: &str,
    stem: &str,
    source: &str,
    refs: Refs<'_>,
) -> Result<CinematicDef, Vec<ContentError>> {
    let raw: RawCinematic = frame::parse(file, source).map_err(|e| vec![e])?;
    let lists = ActionLists {
        table: &raw.actions,
        refs,
        named: BTreeMap::new(),
        used: BTreeSet::new(),
    };
    frame::check(file, stem, &raw, refs.audio, lists)
}

/// The action lists of one cinematic file, as its shots use them.
struct ActionLists<'a, 'r> {
    /// The file's `actions`: lists by name.
    table: &'a BTreeMap<String, Vec<RawAction>>,
    refs: Refs<'r>,
    /// Each named list as checked against each battle it is used on, by
    /// (name, battle).
    named: BTreeMap<(&'a str, String), Vec<ActionDef>>,
    /// The named lists a shot has used.
    used: BTreeSet<&'a str>,
}

impl ActionLists<'_, '_> {
    /// The action of `shot` (shot `n` of the file, counted from 1), if it
    /// is a map pan with one, adding what is wrong with it to `problems`.
    /// A named list's own problems are added the first time a shot uses it
    /// on a battle; nothing is checked on a battle that doesn't exist.
    fn of_shot(
        &mut self,
        n: usize,
        shot: &RawShot,
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
                let stage = Stage::battle(def, refs);
                let (actions, wrong) = action::check(list, &stage, refs.terrain);
                problems.extend(wrong.iter().map(|w| format!("shot {n}: {w}")));
                actions
            }
            (RawActionRef::Named(name), def) => {
                let Some((name, list)) = self.table.get_key_value(name) else {
                    problems.push(format!("shot {n}: no actions named \"{name}\""));
                    return None;
                };
                self.used.insert(name);
                let key = (name.as_str(), battle.clone());
                if let Entry::Vacant(unchecked) = self.named.entry(key.clone()) {
                    let stage = Stage::battle(def?, refs);
                    let (actions, wrong) = action::check(list, &stage, refs.terrain);
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

/// The game's checks of each shot: its own problems, then its action's.
impl ShotCheck<RawShot> for ActionLists<'_, '_> {
    type Shot = Shot;

    fn check(&mut self, n: usize, shot: &RawShot, problems: &mut Vec<String>) -> Shot {
        let refs = self.refs;
        for problem in shot_problems(shot, refs) {
            problems.push(format!("shot {n}: {problem}"));
        }
        let mut checked = shot.checked(refs);
        if let Shot::MapPan { action, .. } = &mut checked {
            *action = self.of_shot(n, shot, problems);
        }
        checked
    }

    fn finish(&mut self, problems: &mut Vec<String>) {
        problems.extend(self.unused());
    }
}

/// Every problem with `shot` on its own, in plain words.
fn shot_problems(shot: &RawShot, refs: Refs<'_>) -> Vec<String> {
    match shot {
        RawShot::MapPan {
            battle,
            from,
            to,
            zoom,
            ..
        } => pan_problems(refs.battles, battle, [("from", *from), ("to", *to)], *zoom),
        RawShot::Logo => Vec::new(),
        RawShot::Talk {
            scene,
            first,
            count,
        } => talk_problems(refs, scene, *first, *count),
        RawShot::Characters {
            who, expression, ..
        } => characters_problems(refs, who, expression),
        RawShot::Combat {
            battle,
            attacker,
            target,
            from,
            seed,
        } => combat::check(refs, (battle, attacker, target), *from, *seed).1,
    }
}

/// Every problem with a map pan.
fn pan_problems(
    battles: &BTreeMap<String, BattleDef>,
    battle: &str,
    points: [(&str, (f32, f32)); 2],
    zoom: u8,
) -> Vec<String> {
    let mut problems = Vec::new();
    match battles.get(battle) {
        None => problems.push(format!("no battle \"{battle}\"")),
        Some(def) => {
            let (w, h) = (def.map.tiles.width(), def.map.tiles.height());
            for (name, point) in points {
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
    if !(1..=MAX_ZOOM).contains(&zoom) {
        problems.push(format!("zoom {zoom}; a zoom is 1 to {MAX_ZOOM}"));
    }
    problems
}

/// Every problem with a talk shot of `count` boxes of `scene` from box
/// `first`.
fn talk_problems(refs: Refs<'_>, scene: &str, first: usize, count: usize) -> Vec<String> {
    let Some(def) = refs.scenes.get(scene) else {
        return vec![format!("no dialogue scene \"{scene}\"")];
    };
    if count == 0 {
        return vec!["count 0; a talk shot shows at least one text box".to_owned()];
    }
    let (boxes, choice) = opening_boxes(def);
    let end = first.saturating_add(count);
    if end > boxes.len() {
        let have = boxes.len();
        let why = if choice {
            "before its first reply choice, and a cinematic never shows a choice"
        } else {
            "in all"
        };
        return vec![format!(
            "text boxes {first} to {} of scene \"{scene}\": it has {have} {why} (the first is 0)",
            end - 1
        )];
    }
    let longest_name = refs.names.longest();
    let numbered = (first..end).zip(&boxes[first..end]);
    numbered
        .filter(|(_, text)| page_lines(text, longest_name) > BOX_LINES)
        .map(|(n, _)| {
            format!(
                "text box {n} of scene \"{scene}\" may not fit one page \
                 ({BOX_LINES} lines of {BOX_WIDTH}); a cinematic turns no pages"
            )
        })
        .collect()
}

/// How many lines of the text box `text` can take: wrapped at
/// [`BOX_WIDTH`] as the dialogue screen wraps it, with every token at its
/// longest (the lead's as [`lead::longest`], a name token as
/// `longest_name`).
fn page_lines(text: &str, longest_name: usize) -> usize {
    let mut filled = String::new();
    for part in lead::split_tokens(text) {
        match part {
            Part::Text(t) | Part::Unclosed(t) => filled.push_str(t),
            Part::Token(t) => {
                let len = match name_token(t) {
                    Some(_) => longest_name,
                    None => lead::longest(t).unwrap_or(text_width(t) + 2),
                };
                filled.push_str(&"x".repeat(len));
            }
        }
    }
    let (mut lines, mut used) = (0, 0);
    for word in filled.split_whitespace() {
        let len = word.chars().count();
        if used > 0 && used + 1 + len <= BOX_WIDTH {
            used += 1 + len;
        } else {
            // A word longer than a line is cut into lines; the last piece
            // starts the line the next word may join.
            let pieces = len.div_ceil(BOX_WIDTH);
            lines += pieces;
            used = len - (pieces - 1) * BOX_WIDTH;
        }
    }
    lines
}

/// Every problem with a characters shot of `who` wearing `expression`.
fn characters_problems(refs: Refs<'_>, who: &[String], expression: &str) -> Vec<String> {
    if who.is_empty() {
        return vec!["who: a characters shot needs at least one character".to_owned()];
    }
    let mut problems = Vec::new();
    for id in who {
        if !refs.characters.can_speak(&CharacterId(id.clone())) {
            problems.push(format!("no character \"{id}\""));
            continue;
        }
        // The lead has a portrait per look, as in a script.
        let ids: &[&str] = if id == LEAD_ID {
            &[PORTRAIT_MALE, PORTRAIT_FEMALE]
        } else {
            &[id.as_str()]
        };
        let portraits: Vec<_> = ids.iter().filter_map(|p| refs.portraits.get(*p)).collect();
        for portrait in &portraits {
            if portrait.expression(expression).is_none() {
                problems.push(format!(
                    "\"{}\"'s portrait has no expression \"{expression}\"",
                    portrait.character
                ));
            }
        }
        if portraits.is_empty() && !STANDARD_EXPRESSIONS.contains(&expression) {
            problems.push(format!(
                "\"{id}\" has no portrait, and \"{expression}\" is not a standard expression"
            ));
        }
    }
    problems
}

impl RawShot {
    /// The shot as the game keeps it, without its action (a combat
    /// shot's units 0 where a name is wrong: then there are problems).
    fn checked(&self, refs: Refs<'_>) -> Shot {
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
            RawShot::Talk {
                scene,
                first,
                count,
            } => Shot::Talk {
                scene: scene.clone(),
                first: *first,
                count: *count,
            },
            RawShot::Characters {
                who,
                expression,
                from,
            } => Shot::Characters {
                who: who.iter().cloned().map(CharacterId).collect(),
                expression: expression.clone(),
                from: match from {
                    RawSide::Left => Side::Left,
                    RawSide::Right => Side::Right,
                },
            },
            RawShot::Combat {
                battle,
                attacker,
                target,
                from,
                seed,
            } => Shot::Combat(combat::check(refs, (battle, attacker, target), *from, *seed).0),
        }
    }
}

#[cfg(test)]
mod tests;
