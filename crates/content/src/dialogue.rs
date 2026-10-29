//! Dialogue scripts (`assets/dialogue/*.dlg`): scenes of portraits, speech
//! and narration. The format is documented in `assets/dialogue/README.md`
//! (ADR-0005) and is the engine's (`shuyi_assets::dialogue`, ticket 1105);
//! this module adds the game's four directives ([`Stage`]: `@place`,
//! `@opening`, `@dies` and `@joins`) and its rules ([`check_scene`]: the cast, portraits, the
//! lead and the music cues). Every line that shows text has a [`LineId`]
//! (ADR-0045). Lines in an `@if` block play only while a character is there
//! ([`Present`], ADR-0055); [`check_presence`] keeps those who may be gone
//! inside one.
use crate::prelude::*;

mod deaths;
mod openings;
mod presence;
mod rules;
mod setting;

use shuyi_assets::dialogue::{self as engine, Directive, Parsed};
use trpg_core::{CharacterId, Pos};

pub use deaths::{deaths, joined, joins};
pub use presence::{Cast, check_presence};
pub use rules::LeadTokens;
pub use setting::{PlaceRefs, check_places};
pub(crate) use shuyi_assets::dialogue::is_id;
pub use shuyi_assets::dialogue::{
    DialogueLimits, Line, LineId, Lines, MusicLine, NARRATION_SPEAKER, PartLines, Present,
    REPLY_SPEAKER, Side, caption_key, check_duplicates, print_scene,
};

use crate::audio::AudioManifest;
use crate::bundle;
use crate::character::CharacterTable;
use crate::error::ContentError;
use crate::formats;
use crate::names::Names;
use crate::portrait::PortraitTable;

/// Directory of dialogue files inside the asset bundle.
pub const DIALOGUE_DIR: &str = "dialogue";
/// File extension of dialogue files.
pub const DIALOGUE_EXTENSION: &str = ".dlg";
/// Longest text of one speech or narration line, in characters (two text
/// boxes of about 3 × 70; ADR-0011).
pub const MAX_TEXT_LEN: usize = 200;
/// Characters per line of the dialogue screen's text box.
pub const BOX_WIDTH: usize = 92;
/// Lines of text the text box shows at once: one page.
pub const BOX_LINES: usize = 3;
/// Longest text of a reply choice, in characters: it must fit the menu.
pub const MAX_OPTION_LEN: usize = 60;
/// Most text steps (speech or narration) in one reply's reaction, so the
/// scene rejoins quickly.
pub const MAX_REACTION_TEXTS: usize = 4;
/// Longest line the lead may speak outside reply choices
/// (`docs/design/setting-and-tone.md`, "Rules for writing the lead").
pub const MAX_LEAD_LINE_LEN: usize = 40;
/// The expressions every portrait has: the ones a script may use for a
/// character without a portrait. A character with one may use exactly its
/// portrait's expressions.
pub const STANDARD_EXPRESSIONS: [&str; 5] = ["neutral", "happy", "angry", "sad", "surprised"];

/// The game's dialogue box and reply menu: what scripts and the language
/// packs' lines are checked against.
pub const LIMITS: DialogueLimits = DialogueLimits {
    box_width: BOX_WIDTH,
    box_lines: BOX_LINES,
    max_text: MAX_TEXT_LEN,
    max_option: MAX_OPTION_LEN,
    min_options: 2,
    max_options: 3,
    max_reaction_texts: MAX_REACTION_TEXTS,
};

/// One step of a scene, in script order: the engine's, with the game's
/// [`Stage`] directives.
pub type Step = engine::Step<Stage>;
/// One reply the lead can pick: `* <tone>: <text>`, then its reaction.
pub type ChoiceOption = engine::ChoiceOption<Stage>;
/// One scene: `@scene <id>` … `@end`.
pub type Scene = engine::Scene<Stage>;
/// Every scene of every dialogue file, by id.
pub type DialogueTable = engine::DialogueTable<Stage>;
/// A scene as written in a file, with the source lines the checks point at.
pub type ParsedScene = engine::ParsedScene<Stage>;
/// Every scene of every dialogue file, with where each was written: what
/// the checks that need the battles and the roster ([`check_presence`])
/// run on once those are loaded.
pub type Scripts = engine::Scripts<Stage>;

/// The game's directives: what is staged around the talk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stage {
    /// `@place <map> <x>,<y>`, `@place battle <x>,<y>` or `@place none`:
    /// the place drawn behind the talk from this line on, until the next
    /// one (`docs/design/look-and-feel.md`, *Places behind conversations*).
    Setting(PlaceLine),
    /// `@opening <id>`: before the next text box, the opening of
    /// `assets/openings/<id>.ron` plays on the place (its characters move
    /// at full brightness, then the place darkens): ticket 0734.
    Opening(String),
    /// `@dies <character>`: the character leaves the army for good when
    /// the scene plays, in every mode (Hollis, ticket 0319): see
    /// [`deaths`].
    Dies(CharacterId),
    /// `@joins <character>`: the character joins the army when the scene
    /// plays (a recruit on a story beat, Joss, ticket 0848;
    /// `battle-scenes-and-recruitment.md`): see [`joins`].
    Joins(CharacterId),
}

/// What a `@place` line asks for: the place behind the talk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaceLine {
    /// `@place <map> <x>,<y>`: the map of `assets/maps/<map>.map`, centred
    /// on tile `at`.
    Map {
        /// The map's id (its file stem).
        map: String,
        /// The tile in the middle.
        at: Pos,
    },
    /// `@place battle <x>,<y>`: the map of the battle the scene belongs to
    /// (its chapter's, for a chapter's intro or victory scene; the one
    /// being played, for a battle's scene), centred on tile `at`.
    Battle {
        /// The tile in the middle.
        at: Pos,
    },
    /// `@place none`: no place, black again.
    None,
}

impl PlaceLine {
    /// What follows `@place` for [`PlaceLine::Battle`].
    pub const BATTLE: &'static str = "battle";
    /// What follows `@place` for [`PlaceLine::None`].
    pub const NONE: &'static str = "none";

    /// What follows `@place` in a script: `<map> <x>,<y>`,
    /// `battle <x>,<y>` or `none`.
    pub fn args(&self) -> String {
        match self {
            PlaceLine::Map { map, at } => format!("{map} {},{}", at.x, at.y),
            PlaceLine::Battle { at } => format!("{} {},{}", Self::BATTLE, at.x, at.y),
            PlaceLine::None => Self::NONE.to_owned(),
        }
    }
}

/// The tile `<x>,<y>` (two whole numbers from 0, no spaces), if `s` is one.
fn tile(s: &str) -> Option<Pos> {
    let (x, y) = s.split_once(',')?;
    let number = |n: &str| {
        let digits = !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit());
        digits.then(|| n.parse::<i32>().ok()).flatten()
    };
    Some(Pos::new(number(x)?, number(y)?))
}

/// `@place` with `args`: a map id and a tile (`<x>,<y>`), `battle` and a
/// tile, or `none`.
fn parse_place(args: &str) -> Parsed<Stage> {
    let args: Vec<&str> = args.split_whitespace().collect();
    let place = match args[..] {
        [PlaceLine::NONE] => PlaceLine::None,
        [map, at] if map != PlaceLine::NONE => {
            let Some(at) = tile(at) else {
                return Parsed::error(format!(
                    "\"{at}\" is not a tile; write it as <x>,<y>, e.g. 3,12"
                ));
            };
            if map == PlaceLine::BATTLE {
                PlaceLine::Battle { at }
            } else {
                let mut parsed = Parsed::step(Stage::Setting(PlaceLine::Map {
                    map: map.into(),
                    at,
                }));
                if !is_id(map) {
                    parsed.errors.push(engine::bad_id(map));
                }
                return parsed;
            }
        }
        _ => {
            return Parsed::error(
                "@place needs a map id and a tile (\"@place ch01 3,12\"), \"battle\" and a \
                 tile, or \"none\"",
            );
        }
    };
    Parsed::step(Stage::Setting(place))
}

/// `@dies` or `@joins` (`name`) with `args`: one character id, made a
/// stage by `stage`.
fn parse_roster(name: &str, args: &str, stage: fn(CharacterId) -> Stage) -> Parsed<Stage> {
    let args: Vec<&str> = args.split_whitespace().collect();
    let [id] = args[..] else {
        return Parsed::error(format!(
            "@{name} needs one character id, e.g. \"@{name} hollis\""
        ));
    };
    let mut parsed = Parsed::step(stage(CharacterId(id.into())));
    if !is_id(id) {
        parsed.errors.push(engine::bad_id(id));
    }
    parsed
}

/// `@opening` with `args`: one opening id.
fn parse_opening(args: &str) -> Parsed<Stage> {
    let args: Vec<&str> = args.split_whitespace().collect();
    let [id] = args[..] else {
        return Parsed::error(
            "@opening needs one opening id (a file of assets/openings/, without .ron)",
        );
    };
    let mut parsed = Parsed::step(Stage::Opening(id.into()));
    if !is_id(id) {
        parsed.errors.push(engine::bad_id(id));
    }
    parsed
}

impl Directive for Stage {
    fn parse(name: &str, args: &str) -> Option<Parsed<Self>> {
        match name {
            "place" => Some(parse_place(args)),
            "opening" => Some(parse_opening(args)),
            "dies" => Some(parse_roster(name, args, Stage::Dies)),
            "joins" => Some(parse_roster(name, args, Stage::Joins)),
            _ => None,
        }
    }

    fn print(&self) -> String {
        match self {
            Stage::Setting(place) => format!("place {}", place.args()),
            Stage::Opening(id) => format!("opening {id}"),
            Stage::Dies(character) => format!("dies {}", character.0),
            Stage::Joins(character) => format!("joins {}", character.0),
        }
    }

    fn kind(&self) -> Option<&'static str> {
        match self {
            Stage::Setting(_) => Some("place"),
            Stage::Opening(_) | Stage::Dies(_) | Stage::Joins(_) => None,
        }
    }
}

/// Parses dialogue `source` (errors attributed to `file`) into its scenes,
/// in file order, and every syntax error found. A scene with errors in it
/// is still returned (without the broken lines), so later checks can run.
pub fn parse_dlg(file: &str, source: &str) -> (Vec<ParsedScene>, Vec<ContentError>) {
    engine::parse_dlg(file, source)
}

/// Checks one scene, replaying it to know who is on screen at each line.
/// Character ids are checked against `characters`, expressions against
/// `portraits`, name tokens and names written out against `names`, and
/// `@music` cues against `audio`, when given; and where an `@opening` may
/// go.
pub fn check_scene(
    parsed: &ParsedScene,
    characters: Option<&CharacterTable>,
    portraits: Option<&PortraitTable>,
    names: Option<&Names>,
    audio: Option<&AudioManifest>,
) -> Vec<ContentError> {
    let rules = rules::Rules {
        characters,
        portraits,
        names,
        audio,
    };
    let mut errors = engine::check_scene(parsed, &rules);
    errors.extend(engine::SceneRules::scene_problems(&rules, parsed));
    errors
}

/// Loads and validates every `*.dlg` file in the bundle. Character ids are
/// checked against `characters`, expressions against the characters'
/// `portraits`, name tokens and names written out against `names`, and
/// `@music` cues against `audio`, when given (each is skipped if its files
/// failed to load). Reports every error of every file.
pub fn load(
    characters: Option<&CharacterTable>,
    portraits: Option<&PortraitTable>,
    names: Option<&Names>,
    audio: Option<&AudioManifest>,
) -> Result<DialogueTable, Vec<ContentError>> {
    load_scripts(characters, portraits, names, audio).map(|s| s.table)
}

/// [`load`], keeping where each scene was written.
pub fn load_scripts(
    characters: Option<&CharacterTable>,
    portraits: Option<&PortraitTable>,
    names: Option<&Names>,
    audio: Option<&AudioManifest>,
) -> Result<Scripts, Vec<ContentError>> {
    let mut files = Vec::new();
    let mut errors = Vec::new();
    for result in formats::install()
        .load_each::<engine::DialogueFile<Stage>>(bundle::install(), DIALOGUE_DIR)
        .into_values()
    {
        match result {
            Ok(file) => files.push(file),
            Err(e) => errors.extend(e),
        }
    }
    let rules = rules::Rules {
        characters,
        portraits,
        names,
        audio,
    };
    match engine::scripts_from_parsed(files, &rules) {
        Ok(scripts) if errors.is_empty() => Ok(scripts),
        result => {
            errors.extend(result.err().unwrap_or_default());
            errors.sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));
            Err(errors)
        }
    }
}

/// Parses and validates dialogue files given as `(file name, source)`
/// pairs. Reports every problem, ordered by file and line. Who may be gone
/// when a scene plays isn't checked here: [`check_presence`] needs the
/// battles and the roster.
pub fn from_sources<F: AsRef<str>>(
    files: &[(F, &str)],
    characters: Option<&CharacterTable>,
    portraits: Option<&PortraitTable>,
    names: Option<&Names>,
    audio: Option<&AudioManifest>,
) -> Result<DialogueTable, Vec<ContentError>> {
    scripts_from_sources(files, characters, portraits, names, audio).map(|s| s.table)
}

/// [`from_sources`], keeping where each scene was written.
pub fn scripts_from_sources<F: AsRef<str>>(
    files: &[(F, &str)],
    characters: Option<&CharacterTable>,
    portraits: Option<&PortraitTable>,
    names: Option<&Names>,
    audio: Option<&AudioManifest>,
) -> Result<Scripts, Vec<ContentError>> {
    let rules = rules::Rules {
        characters,
        portraits,
        names,
        audio,
    };
    engine::scripts_from_sources(files, &rules)
}

#[cfg(test)]
mod tests;
