//! One-time contextual tips (`assets/data/tips.ron`, ticket 0406): what
//! triggers each and what it says. The screens decide when a trigger has
//! happened and remember which tips were seen.
use crate::prelude::*;

use alloc::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use crate::bundle;
use crate::error::ContentError;
use crate::keymap::Action;
use crate::lang::text_width;
use crate::portrait::Portrait;
use crate::ron_loader::parse_ron;

/// Path of the tips file inside the asset bundle.
pub const TIPS_PATH: &str = "data/tips.ron";

/// Most text lines a tip may have.
pub const MAX_TEXT_LINES: usize = 3;

/// Longest text line, in characters, before placeholders are filled in.
pub const MAX_LINE_CHARS: usize = 60;

/// Longest tip title, in characters.
pub const MAX_TITLE_CHARS: usize = 30;

/// The portrait of the guide, who speaks every tip
/// (`docs/design/tutorial.md`, *The guide*; ticket 0455). Only the bought
/// art has it: without it the tip box has no face.
pub const GUIDE_PORTRAIT: &str = "guide";

/// The guide's expression in a tip that names none.
pub const DEFAULT_FACE: &str = "smile";

/// The placeholder for the keys that move the cursor (`{Cursor}`); every
/// other placeholder is an [`Action`] name, e.g. `{Confirm}`.
pub const CURSOR_PLACEHOLDER: &str = "Cursor";

/// What makes a tip appear.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub enum TipTrigger {
    /// A battle starts.
    FirstBattleStart,
    /// The player selects a unit.
    FirstUnitSelected,
    /// A selected unit could reach an enemy.
    FirstEnemyInRange,
    /// The attack forecast opens.
    FirstForecast,
    /// A player unit is badly hurt.
    FirstLowHp,
    /// A player unit levels up.
    FirstLevelUp,
    /// The enemy phase begins.
    FirstEnemyPhase,
    /// The cursor rests on an enemy, so its danger zone is worth knowing.
    DangerZoneAvailable,
    /// The player selects a unit that has a skill (ticket 0458).
    FirstSkillUnit,
    /// The action menu opens for a unit that knows a spell (0458).
    FirstCaster,
    /// The cursor rests on a tile whose terrain is not plain ground: a
    /// Def, Avoid or healing bonus, or a movement cost other than 1
    /// (0458).
    FirstTerrain,
    // The triggers below start lessons only (ticket 0457): no tip uses them
    // yet.
    /// In the player's phase, every unit of the player's has acted.
    NoUnitLeft,
    /// The player's phase of turn 2 begins: there is a turn to undo.
    SecondTurn,
    /// The threat arrows show: an enemy could attack a unit of the
    /// player's where it stands or is about to move.
    ThreatArrowsShown,
    /// A unit's action menu opens with `Arts` it can use.
    ArtsAvailable,
    /// A unit's spell list opens with a spell it could cast on a tile
    /// (to change it) from there.
    TileSpellAvailable,
}

/// One tip.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tip {
    /// Stable id (`snake_case`); what is remembered once the tip was seen.
    pub id: String,
    /// What makes it appear.
    pub trigger: TipTrigger,
    /// Shown in the box's border.
    pub title: String,
    /// The body: lines separated by `\n`, with `{Action}` placeholders
    /// for key names.
    pub text: String,
    /// The guide's expression while the tip shows; [`DEFAULT_FACE`] if
    /// none. Written `face: "sad"`.
    #[serde(default, deserialize_with = "some_name")]
    pub face: Option<String>,
}

/// A name written plainly (`face: "sad"`, not `Some("sad")`).
fn some_name<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    String::deserialize(d).map(Some)
}

impl Tip {
    /// The guide's expression while the tip shows.
    pub fn face(&self) -> &str {
        self.face.as_deref().unwrap_or(DEFAULT_FACE)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    tips: Vec<Tip>,
}

/// Every tip, in file order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TipTable {
    /// The tips.
    pub tips: Vec<Tip>,
}

impl TipTable {
    /// The tip that `trigger` shows, if the file has one.
    pub fn for_trigger(&self, trigger: TipTrigger) -> Option<&Tip> {
        self.tips.iter().find(|t| t.trigger == trigger)
    }
}

/// Loads and validates the embedded tips file, each tip's face against
/// the guide's portrait in `faces` when there is one.
pub fn load(faces: Option<&BTreeMap<String, Portrait>>) -> Result<TipTable, Vec<ContentError>> {
    let display = bundle::display_path(TIPS_PATH);
    let source = bundle::file(TIPS_PATH).ok_or_else(|| {
        vec![ContentError::new(
            &display,
            "file not found in asset bundle",
        )]
    })?;
    from_source(&display, source, faces)
}

/// Parses and validates tips `source`, attributing errors to `file`, each
/// tip's face against the guide's portrait in `faces` when there is one.
/// Reports every problem found.
pub fn from_source(
    file: &str,
    source: &str,
    faces: Option<&BTreeMap<String, Portrait>>,
) -> Result<TipTable, Vec<ContentError>> {
    let guide = faces.and_then(|f| f.get(GUIDE_PORTRAIT));
    let raw: RawFile = parse_ron(file, source).map_err(|e| vec![e])?;
    let mut errors = Vec::new();
    let mut ids = BTreeSet::new();
    let mut triggers = BTreeSet::new();
    for tip in &raw.tips {
        let mut err = |message: String| {
            errors.push(ContentError::new(
                file,
                format!("tip \"{}\": {message}", tip.id),
            ));
        };
        let id_ok = !tip.id.is_empty()
            && tip
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
        if !id_ok {
            err("the id must be lowercase letters, digits and `_`".into());
        }
        if !ids.insert(tip.id.as_str()) {
            err("the id is used twice".into());
        }
        if !triggers.insert(tip.trigger) {
            err(format!("{:?} already has a tip", tip.trigger));
        }
        if tip.title.is_empty() || text_width(&tip.title) > MAX_TITLE_CHARS {
            err(format!(
                "the title must be 1 to {MAX_TITLE_CHARS} characters"
            ));
        }
        let lines: Vec<&str> = tip.text.lines().collect();
        if lines.is_empty() || lines.len() > MAX_TEXT_LINES {
            err(format!("the text must have 1 to {MAX_TEXT_LINES} lines"));
        }
        for line in lines {
            if text_width(line) > MAX_LINE_CHARS {
                err(format!(
                    "a text line is over {MAX_LINE_CHARS} characters: {line:?}"
                ));
            }
        }
        for name in placeholders(&tip.text) {
            if name != CURSOR_PLACEHOLDER && Action::from_name(name).is_none() {
                err(format!("unknown placeholder {{{name}}}"));
            }
        }
        if let Some(guide) = guide
            && guide.expression(tip.face()).is_none()
        {
            err(format!(
                "the guide's portrait has no expression \"{}\"",
                tip.face()
            ));
        }
    }
    if errors.is_empty() {
        Ok(TipTable { tips: raw.tips })
    } else {
        Err(errors)
    }
}

/// The names between braces in `text`, in order (`{Confirm}` gives
/// `Confirm`). An unclosed `{` is ignored.
pub fn placeholders(text: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut rest = text;
    while let Some((_, after)) = rest.split_once('{') {
        let Some((name, tail)) = after.split_once('}') else {
            break;
        };
        names.push(name);
        rest = tail;
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn errors(source: &str) -> Vec<String> {
        from_source("t.ron", source, None)
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.message)
            .collect()
    }

    #[test]
    fn the_shipped_file_loads_and_covers_every_trigger() {
        let table = load(None).unwrap();
        for trigger in [
            TipTrigger::FirstBattleStart,
            TipTrigger::FirstUnitSelected,
            TipTrigger::FirstEnemyInRange,
            TipTrigger::FirstForecast,
            TipTrigger::FirstLowHp,
            TipTrigger::FirstLevelUp,
            TipTrigger::FirstEnemyPhase,
            TipTrigger::DangerZoneAvailable,
            TipTrigger::FirstSkillUnit,
            TipTrigger::FirstCaster,
            TipTrigger::FirstTerrain,
        ] {
            assert!(table.for_trigger(trigger).is_some(), "{trigger:?}");
        }
    }

    #[test]
    fn placeholders_are_listed_in_order() {
        assert_eq!(placeholders("{A} x {B}{C} {"), ["A", "B", "C"]);
        assert!(placeholders("none {open").is_empty());
    }

    #[test]
    fn a_good_tip_loads() {
        let source = r#"(tips: [(id: "a", trigger: FirstForecast, title: "T",
            text: "Press {Confirm}.\n{Cursor}")])"#;
        let table = from_source("t.ron", source, None).unwrap();
        assert_eq!(table.tips.len(), 1);
        assert_eq!(
            table.for_trigger(TipTrigger::FirstForecast),
            Some(&table.tips[0])
        );
        assert_eq!(table.for_trigger(TipTrigger::FirstLowHp), None);
    }

    #[test]
    fn bad_tips_are_reported() {
        let long = "x".repeat(61);
        let source = format!(
            r#"(tips: [
            (id: "A", trigger: FirstForecast, title: "", text: "{{Nope}}"),
            (id: "A", trigger: FirstForecast, title: "T", text: "1\n2\n3\n4"),
            (id: "c", trigger: FirstLowHp, title: "T", text: "{long}"),
            (id: "d", trigger: FirstLowHp, title: "T", text: ""),
            ])"#
        );
        let all = errors(&source);
        let has = |part: &str| all.iter().any(|m| m.contains(part));
        assert!(has("lowercase letters"), "{all:?}");
        assert!(has("used twice"), "{all:?}");
        assert!(has("already has a tip"), "{all:?}");
        assert!(has("the title must be"), "{all:?}");
        assert!(has("unknown placeholder {Nope}"), "{all:?}");
        assert!(has("1 to 3 lines"), "{all:?}");
        assert!(has("over 60 characters"), "{all:?}");
    }

    #[test]
    fn length_limits_are_inclusive() {
        let title = "t".repeat(MAX_TITLE_CHARS);
        let line = "x".repeat(MAX_LINE_CHARS);
        let source = format!(
            r#"(tips: [(id: "a", trigger: FirstForecast, title: "{title}",
            text: "{line}
{line}
{line}")])"#
        );
        assert_eq!(errors(&source), Vec::<String>::new());
        let source = source.replace(&title, &format!("{title}t"));
        assert_eq!(errors(&source).len(), 1, "{source}");
        let source = source.replace(&line, &format!("{line}x"));
        assert_eq!(errors(&source).len(), 4);
    }

    /// Ticket 0455: a tip's face is the guide's expression, `smile` by
    /// default, checked against the guide's portrait when there is one.
    #[test]
    fn faces_are_checked_against_the_guides_portrait() {
        let source = r#"(tips: [
            (id: "a", trigger: FirstForecast, title: "T", text: "x"),
            (id: "b", trigger: FirstLowHp, title: "T", text: "x", face: "sad"),
            (id: "c", trigger: FirstLevelUp, title: "T", text: "x", face: "smug"),
            ])"#;
        // Without the guide's portrait any face goes.
        let table = from_source("t.ron", source, None).unwrap();
        let faces: Vec<&str> = table.tips.iter().map(Tip::face).collect();
        assert_eq!(faces, [DEFAULT_FACE, "sad", "smug"]);
        assert_eq!(table.tips[0].face, None);
        // A guide with the required expressions plus `smile`.
        let names = [
            "neutral",
            "happy",
            "angry",
            "sad",
            "surprised",
            DEFAULT_FACE,
        ];
        let paths = names.map(|n| -> &'static str { format!("portraits/guide/{n}.png").leak() });
        let pngs = paths.map(|p| (p, crate::image::png_header(64, 64)));
        let images = crate::image::from_files(pngs.iter().map(|(p, png)| (*p, &png[..]))).unwrap();
        let sidecar = names
            .map(|n| format!("{n:?}: \"guide/{n}.png\""))
            .join(", ");
        let sidecar = format!("(character: \"guide\", expressions: {{{sidecar}}})");
        let mut guide =
            crate::portrait::parse_portrait("guide.ron", "guide", &sidecar, &images).unwrap();
        let with = |guide: &Portrait| {
            let faces = BTreeMap::from([(GUIDE_PORTRAIT.to_owned(), guide.clone())]);
            from_source("t.ron", source, Some(&faces))
                .err()
                .unwrap_or_default()
                .into_iter()
                .map(|e| e.message)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            with(&guide),
            ["tip \"c\": the guide's portrait has no expression \"smug\""]
        );
        // Without `smile`, a tip that names no face fails too.
        guide.expressions.retain(|e| e.name != DEFAULT_FACE);
        assert_eq!(with(&guide).len(), 2);
        assert!(with(&guide)[0].contains("no expression \"smile\""));
        // Another character's portrait is not the guide's.
        let others = BTreeMap::from([("test_lord".to_owned(), guide)]);
        assert!(from_source("t.ron", source, Some(&others)).is_ok());
    }

    #[test]
    fn a_syntax_error_is_reported() {
        assert_eq!(errors("(tips: [").len(), 1);
    }
}
