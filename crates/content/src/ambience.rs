//! A place's ambience (`assets/data/ambience.ron`, ticket 1332, ADR-0132):
//! the sound bed looped under its music, one per light where the place has
//! more than one (`docs/design/audio.md`, *Ambience*). Look data only: the
//! rules never hear it (ADR-0004).
//!
//! The file names each ambience's beds and which ambience a scene's
//! `@place` map and camp play. A battle file names its own (`ambience:
//! "meadow"`, [`battle::ambiences`](crate::battle::ambiences)), and a world
//! map names the map's and each town's ([`WorldMapDef::ambience`]).
//!
//! Loading checks that every bed is a looped sound cue of the
//! [`AMBIENCE`] group, every light one of the lights, that every ambience
//! has a bed for [`NOON`] (a light without its own plays it), and that
//! every name of an ambience is one.
use crate::prelude::*;

use alloc::collections::BTreeMap;

use serde::Deserialize;

use crate::audio::AudioManifest;
use crate::bundle;
use crate::error::ContentError;
use crate::lighting::{Lighting, NOON};
use crate::map::MapDef;
use crate::ron_loader::parse_ron;
use crate::terrain::line_of;
use crate::worldmap::WorldMapDef;

/// Path of the ambience file inside the asset bundle.
pub const AMBIENCE_PATH: &str = "data/ambience.ron";

/// The sound group every bed plays in, with its own volume on Options.
pub const AMBIENCE: &str = "ambience";

/// Every ambience and what plays which.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AmbienceTable {
    /// Each ambience's beds, by ambience id: a sound cue by light.
    pub ambiences: BTreeMap<String, BTreeMap<String, String>>,
    /// The ambience of each place a scene's `@place` shows, by map id.
    pub places: BTreeMap<String, String>,
    /// The ambience camp plays, if any.
    pub camp: Option<String>,
    /// The ambience each battle file names, by battle id.
    pub battles: BTreeMap<String, String>,
}

impl AmbienceTable {
    /// The bed ambience `id` plays in light `light`: its own for the light,
    /// else its [`NOON`] bed. `None` for an ambience there isn't.
    pub fn bed(&self, id: &str, light: &str) -> Option<&str> {
        let beds = self.ambiences.get(id)?;
        beds.get(light)
            .or_else(|| beds.get(NOON))
            .map(String::as_str)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    ambiences: BTreeMap<String, BTreeMap<String, String>>,
    #[serde(default)]
    places: BTreeMap<String, String>,
    #[serde(default)]
    camp: Option<String>,
}

/// What the ambience file and the names of an ambience are checked
/// against.
#[derive(Debug, Clone, Copy)]
pub struct AmbienceRefs<'a> {
    /// The sound cues.
    pub audio: &'a AudioManifest,
    /// The lights.
    pub lighting: &'a Lighting,
    /// The maps, by id.
    pub maps: &'a BTreeMap<String, MapDef>,
    /// The world maps, by id.
    pub worldmaps: &'a BTreeMap<String, WorldMapDef>,
    /// The ambience each battle file names, by battle id.
    pub battles: &'a BTreeMap<String, String>,
}

/// Loads and checks the embedded ambience file.
pub fn load(refs: AmbienceRefs<'_>) -> Result<AmbienceTable, Vec<ContentError>> {
    let display = bundle::display_path(AMBIENCE_PATH);
    let source = bundle::file(AMBIENCE_PATH).ok_or_else(|| {
        vec![ContentError::new(
            &display,
            "file not found in asset bundle",
        )]
    })?;
    from_source(&display, &source, refs)
}

/// Parses ambience file `source` (attributing errors to `file`) and checks
/// it and every name of an ambience in `refs`. Reports every problem.
pub fn from_source(
    file: &str,
    source: &str,
    refs: AmbienceRefs<'_>,
) -> Result<AmbienceTable, Vec<ContentError>> {
    let raw: RawFile = parse_ron(file, source).map_err(|e| vec![e])?;
    let mut errors = Vec::new();
    let mut at = |needle: &str, message: String| {
        let err = ContentError::new(file, message);
        errors.push(match line_of(source, needle) {
            Some(line) => err.at(line, None),
            None => err,
        });
    };
    for (id, beds) in &raw.ambiences {
        if !beds.contains_key(NOON) {
            at(
                &format!("\"{id}\""),
                format!("ambience \"{id}\": no \"{NOON}\" bed (a light without its own plays it)"),
            );
        }
        for (light, cue) in beds {
            if refs.lighting.get(light).is_none() {
                let lights = refs.lighting.names();
                at(
                    &format!("\"{light}\""),
                    format!(
                        "ambience \"{id}\": \"{light}\" is not a light; the lights are {lights}"
                    ),
                );
            }
            if let Some(problem) = cue_problem(refs.audio, cue) {
                at(
                    &format!("\"{cue}\""),
                    format!("ambience \"{id}\": sound cue \"{cue}\" {problem}"),
                );
            }
        }
    }
    let known = |id: &str| raw.ambiences.contains_key(id);
    for (map, id) in &raw.places {
        if !refs.maps.contains_key(map) {
            at(&format!("\"{map}\""), format!("places: no map \"{map}\""));
        }
        if !known(id) {
            at(
                &format!("\"{id}\""),
                format!("places: \"{map}\": no ambience \"{id}\""),
            );
        }
    }
    if let Some(id) = raw.camp.as_deref().filter(|id| !known(id)) {
        at("camp", format!("camp: no ambience \"{id}\""));
    }
    let names = elsewhere(refs).filter(|(_, id)| !known(id));
    errors.extend(names.map(|(file, id)| {
        ContentError::new(
            &file,
            format!("ambience: no ambience \"{id}\" in {AMBIENCE_PATH}"),
        )
    }));
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(AmbienceTable {
        ambiences: raw.ambiences,
        places: raw.places,
        camp: raw.camp,
        battles: refs.battles.clone(),
    })
}

/// What is wrong with sound cue `cue` as a bed, if anything.
fn cue_problem(audio: &AudioManifest, cue: &str) -> Option<&'static str> {
    let Some(sound) = audio.sounds.get(cue) else {
        return Some("is not a sound cue");
    };
    if !sound.looped {
        return Some("is not looped");
    }
    (sound.group.as_deref() != Some(AMBIENCE)).then_some("is not in the \"ambience\" group")
}

/// Every ambience named outside the ambience file, with the file that
/// names it: the battles' and the world maps' (each map's and its towns').
fn elsewhere(refs: AmbienceRefs<'_>) -> impl Iterator<Item = (String, &str)> {
    let battles = refs.battles.iter().map(|(battle, id)| {
        let file = format!("{}/{battle}.ron", crate::battle::BATTLES_DIR);
        (bundle::display_path(&file), id.as_str())
    });
    let worldmaps = refs.worldmaps.iter().flat_map(|(map, def)| {
        let file = bundle::display_path(&format!("{}/{map}.ron", crate::worldmap::WORLDMAPS_DIR));
        let own = def.ambience.iter();
        let towns = def.place_ambience.values();
        own.chain(towns).map(move |id| (file.clone(), id.as_str()))
    });
    battles.chain(worldmaps)
}

#[cfg(test)]
mod tests;
