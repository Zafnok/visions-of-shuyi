//! Camp events (`assets/data/camp.ron`, ticket 1003): the optional scenes
//! listed at camp, and when each appears. The rules are in
//! `trpg_core::camp`; which events exist is the story pipeline's
//! (`docs/design/supports.md`, *Rules: camp events*).
//!
//! Loading checks that no id is listed twice, that each title isn't
//! empty, and that each event's scene, battles and characters exist.
use crate::prelude::*;

use alloc::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use trpg_core::{BattleDef, CampEvent, CharacterId};

use crate::bundle;
use crate::character::CharacterTable;
use crate::dialogue::DialogueTable;
use crate::error::ContentError;
use crate::ron_loader::parse_ron;
use crate::terrain::line_of;

/// Path of the camp file inside the asset bundle.
pub const CAMP_PATH: &str = "data/camp.ron";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    events: Vec<RawEvent>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEvent {
    id: String,
    title: String,
    scene: String,
    #[serde(default)]
    flags: Vec<String>,
    #[serde(default)]
    battles: Vec<String>,
    #[serde(default)]
    units: Vec<String>,
}

/// What camp events refer to, each `None` (not checked) if its files
/// failed to load.
#[derive(Debug, Clone, Copy, Default)]
pub struct CampRefs<'a> {
    /// The characters.
    pub characters: Option<&'a CharacterTable>,
    /// The dialogue scenes.
    pub dialogue: Option<&'a DialogueTable>,
    /// The battles, by id.
    pub battles: Option<&'a BTreeMap<String, BattleDef>>,
}

/// Loads and validates the embedded camp file, with the test files'
/// events after its own when the build has them (ADR-0057).
pub fn load(refs: CampRefs) -> Result<Vec<CampEvent>, Vec<ContentError>> {
    let display = bundle::display_path(CAMP_PATH);
    let source = bundle::file(CAMP_PATH).ok_or_else(|| {
        vec![ContentError::new(
            &display,
            "file not found in asset bundle",
        )]
    })?;
    let mut files = vec![(display, source)];
    files.extend(bundle::test_part(CAMP_PATH));
    from_sources(&files, refs)
}

/// Parses and validates camp files, `(file name, source)` in order: their
/// events, one list. Reports every problem found.
pub fn from_sources<F: AsRef<str>>(
    files: &[(F, &str)],
    refs: CampRefs,
) -> Result<Vec<CampEvent>, Vec<ContentError>> {
    let mut errors = Vec::new();
    let mut seen = BTreeSet::new();
    let mut events = Vec::new();
    for (file, source) in files {
        let file = file.as_ref();
        let raw: RawFile = match parse_ron(file, source) {
            Ok(raw) => raw,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        for e in raw.events {
            let line = line_of(source, &format!("id: \"{}\"", e.id));
            let what = format!("camp event \"{}\"", e.id);
            let mut at = |message: String| {
                let err = ContentError::new(file, format!("{what}: {message}"));
                errors.push(match line {
                    Some(l) => err.at(l, None),
                    None => err,
                });
            };
            if !seen.insert(e.id.clone()) {
                at("the id is listed twice".into());
            }
            if e.title.trim().is_empty() {
                at("the title is empty".into());
            }
            if refs.dialogue.is_some_and(|d| d.get(&e.scene).is_none()) {
                at(format!("unknown dialogue scene \"{}\"", e.scene));
            }
            if let Some(battles) = refs.battles.filter(|b| !b.is_empty()) {
                for b in e.battles.iter().filter(|b| !battles.contains_key(*b)) {
                    at(format!("unknown battle \"{b}\""));
                }
            }
            let units: Vec<CharacterId> = e.units.into_iter().map(CharacterId).collect();
            if let Some(characters) = refs.characters {
                for u in units.iter().filter(|u| !characters.characters.contains_key(*u)) {
                    at(format!("unknown character \"{}\"", u.0));
                }
            }
            events.push(CampEvent {
                id: e.id,
                title: e.title,
                scene: e.scene,
                flags: e.flags,
                battles: e.battles,
                units,
            });
        }
    }
    if errors.is_empty() {
        Ok(events)
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests;
