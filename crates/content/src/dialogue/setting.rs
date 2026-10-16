//! The checks on `@place` lines that need the maps and the battles
//! ([`check_places`]): the map is one of `assets/maps/`, the tile is on it,
//! and `@place battle` is only in a scene that has a battle. And on
//! `@opening` lines, which need the openings: the opening is one of
//! `assets/openings/`, and plays on the place in force (ticket 0734).
use crate::prelude::*;

use alloc::collections::{BTreeMap, BTreeSet};

use trpg_core::{BattleDef, BattleMap, Pos, SupportTable};

use super::{Lines, ParsedScene};
use super::{PlaceLine, Stage, Step};
use crate::chapter::ChapterDef;
use crate::cinematic::OpeningDef;
use crate::error::ContentError;
use crate::map::MapDef;

/// What [`check_places`] checks a scene's `@place` lines against.
#[derive(Debug, Clone, Default)]
pub struct PlaceRefs<'a> {
    /// The maps of `assets/maps/`, by id.
    pub maps: BTreeMap<&'a str, &'a BattleMap>,
    /// By scene id, the battles a scene is played for and their maps: a
    /// chapter's intro and victory scenes are its battle's, a trigger's
    /// scene is its battle's. `None` when the battles or chapters aren't
    /// known: `@place battle` isn't checked then.
    pub battles: Option<BTreeMap<String, Vec<(&'a str, &'a BattleMap)>>>,
    /// The scenes also played away from any battle: support
    /// conversations.
    pub elsewhere: BTreeSet<String>,
    /// The map each opening of `assets/openings/` plays on, by opening id.
    /// `None` when the openings aren't known: `@opening` isn't checked
    /// then.
    pub openings: Option<BTreeMap<&'a str, &'a str>>,
}

impl<'a> PlaceRefs<'a> {
    /// The places a script can name with `maps`, these `battles` and
    /// `chapters`, and the `supports` (each `None` if it failed to load).
    pub fn new(
        maps: Option<&'a BTreeMap<String, MapDef>>,
        battles: Option<&'a BTreeMap<String, BattleDef>>,
        chapters: Option<&'a BTreeMap<String, ChapterDef>>,
        supports: Option<&'a SupportTable>,
    ) -> Self {
        let maps = maps
            .into_iter()
            .flatten()
            .map(|(id, def)| (id.as_str(), &def.map))
            .collect();
        // Both load empty, without an error, when a file they need failed:
        // they are no base to check on then.
        let battles = battles.filter(|b| !b.is_empty());
        let chapters = chapters.filter(|c| !c.is_empty());
        let battles = battles.zip(chapters).map(|(battles, chapters)| {
            let mut of: BTreeMap<String, Vec<(&str, &BattleMap)>> = BTreeMap::new();
            let mut add = |scene: &str, battle: &'a BattleDef| {
                of.entry(scene.to_owned())
                    .or_default()
                    .push((battle.id.as_str(), &battle.map));
            };
            for battle in battles.values() {
                for trigger in &battle.triggers {
                    add(&trigger.scene, battle);
                }
            }
            for chapter in chapters.values() {
                let Some(battle) = battles.get(&chapter.battle) else {
                    continue;
                };
                for scene in chapter.intro_scenes.iter().chain(&chapter.victory_scenes) {
                    add(scene, battle);
                }
            }
            of
        });
        let elsewhere = supports
            .into_iter()
            .flat_map(SupportTable::pairs)
            .flat_map(|def| {
                let c = &def.conversations;
                [c.c.clone(), c.b.clone(), c.a.clone()]
            })
            .collect();
        Self {
            maps,
            battles,
            elsewhere,
            openings: None,
        }
    }

    /// The same, with the `openings` an `@opening` line may name (`None`
    /// if they failed to load).
    #[must_use]
    pub fn with_openings(self, openings: Option<&'a BTreeMap<String, OpeningDef>>) -> Self {
        let openings = openings.map(|all| {
            let maps = all.iter().map(|(id, def)| (id.as_str(), def.map.as_str()));
            maps.collect()
        });
        Self { openings, ..self }
    }
}

/// Checks the `@place` lines of `parsed`: one error for each line that
/// names a map `refs` hasn't got, a tile off its map, or the battle in a
/// scene that isn't a battle's (or is also played away from one), or a
/// tile off that battle's map. And its `@opening` lines: one error for
/// each that names an opening `refs` hasn't got, or one that plays on
/// another map than the place in force. (Where an `@opening` may go:
/// `check_scene`.)
pub fn check_places(parsed: &ParsedScene, refs: &PlaceRefs) -> Vec<ContentError> {
    let mut places = Vec::new();
    settings(&parsed.scene.steps, &parsed.lines, &mut places);
    let mut errors = Vec::new();
    for (line, place) in places {
        let message = match place {
            PlaceLine::Map { map, at } => map_problem(refs, map, *at),
            PlaceLine::Battle { at } => battle_problem(refs, &parsed.scene.id, *at),
            PlaceLine::None => None,
        };
        if let Some(message) = message {
            errors.push(ContentError::new(&parsed.file, message).at(line, None));
        }
    }
    let Some(openings) = &refs.openings else {
        return errors;
    };
    for (line, id, place) in opening_lines(&parsed.scene.steps, &parsed.lines) {
        let message = match (openings.get(id), place) {
            (None, _) => Some(format!(
                "no opening \"{id}\"; an opening is a file of assets/openings/ (its file name \
                 without .ron)"
            )),
            (Some(&on), Some(PlaceLine::Map { map, .. })) if on != map => Some(format!(
                "opening \"{id}\" plays on map \"{on}\", but the place here is \"{map}\""
            )),
            _ => None,
        };
        if let Some(message) = message {
            errors.push(ContentError::new(&parsed.file, message).at(line, None));
        }
    }
    errors
}

/// An `@opening` line: where it is written, the opening's id and the
/// place in force there.
type OpeningLine<'s> = (u32, &'s str, Option<&'s PlaceLine>);

/// The `@opening` lines at the top level of `steps` (written at `lines`).
fn opening_lines<'s>(steps: &'s [Step], lines: &Lines) -> Vec<OpeningLine<'s>> {
    let mut place = None;
    let mut out = Vec::new();
    for (step, &line) in steps.iter().zip(&lines.steps) {
        match step {
            Step::Directive(Stage::Opening(id)) => out.push((line, id.as_str(), place)),
            _ => place = last_setting(step).or(place),
        }
    }
    out
}

/// The place `step` leaves in force, if it changes it: a `@place` line's,
/// or the last one in a block's first part (every part of a block leaves
/// the same: `check_scene`).
pub(super) fn last_setting(step: &Step) -> Option<&PlaceLine> {
    match step {
        Step::Directive(Stage::Setting(place)) => Some(place),
        Step::Choice { options } => last_in(&options.first()?.steps),
        Step::If { then, .. } => last_in(then),
        _ => None,
    }
}

/// The place the last step of `steps` that changes it leaves in force.
fn last_in(steps: &[Step]) -> Option<&PlaceLine> {
    steps.iter().rev().find_map(last_setting)
}

/// Adds every `@place` line of `steps` (written at `lines`) and of the
/// blocks among them to `out`, with its line.
fn settings<'s>(steps: &'s [Step], lines: &Lines, out: &mut Vec<(u32, &'s PlaceLine)>) {
    let mut blocks = lines.blocks.iter();
    for (step, &line) in steps.iter().zip(&lines.steps) {
        let parts: Vec<&[Step]> = match step {
            Step::Directive(Stage::Setting(place)) => {
                out.push((line, place));
                continue;
            }
            Step::Choice { options } => options.iter().map(|o| &o.steps[..]).collect(),
            Step::If {
                then, otherwise, ..
            } => vec![then, otherwise],
            _ => continue,
        };
        if let Some(block) = blocks.next() {
            for (part, at) in parts.into_iter().zip(block) {
                settings(part, &at.lines, out);
            }
        }
    }
}

/// What is wrong with `@place <map> <at>`, if anything.
fn map_problem(refs: &PlaceRefs, map: &str, at: Pos) -> Option<String> {
    let Some(tiles) = refs.maps.get(map) else {
        return Some(format!(
            "no map \"{map}\"; a place is a map of assets/maps/ (its file name without .map)"
        ));
    };
    off_map(tiles, at).map(|size| format!("tile {},{} is off map \"{map}\" ({size})", at.x, at.y))
}

/// What is wrong with `@place battle <at>` in scene `scene`, if anything.
fn battle_problem(refs: &PlaceRefs, scene: &str, at: Pos) -> Option<String> {
    let battles = refs.battles.as_ref()?;
    let only = "\"@place battle\" is only for a battle's scenes (a chapter's intro and victory \
                scenes, a battle trigger's scene)";
    let Some(of) = battles.get(scene) else {
        return Some(format!("{only}; no battle plays \"{scene}\""));
    };
    if refs.elsewhere.contains(scene) {
        return Some(format!(
            "{only}; \"{scene}\" is also a support conversation, played away from a battle"
        ));
    }
    of.iter().find_map(|(battle, map)| {
        off_map(map, at).map(|size| {
            format!(
                "tile {},{} is off the map of battle \"{battle}\" ({size})",
                at.x, at.y
            )
        })
    })
}

/// The size of `map`, in words, if `at` is off it.
fn off_map(map: &BattleMap, at: Pos) -> Option<String> {
    let tiles = &map.tiles;
    (!tiles.in_bounds(at)).then(|| {
        let (w, h) = (tiles.width(), tiles.height());
        format!(
            "{w} by {h} tiles, from 0,0 to {},{}",
            w.saturating_sub(1),
            h.saturating_sub(1)
        )
    })
}
