//! One-time contextual tips (`assets/data/tips.ron`, ticket 0406): what
//! triggers each and what it says. The screens decide when a trigger has
//! happened and remember which tips were seen.
//!
//! The tip schema and its checks are the engine's (`shuyi_assets::tip`,
//! ticket 1105); the game adds its triggers ([`TipTrigger`]), its box, its
//! placeholders and the guide who speaks every tip.
use crate::prelude::*;

use alloc::collections::BTreeMap;

use serde::Deserialize;
use shuyi_assets::tip::{self as engine, TipLimits, TipRules};

use crate::bundle;
use crate::error::ContentError;
use crate::keymap::Action;
use crate::portrait::Portrait;

pub use shuyi_assets::tip::placeholders;

/// Path of the tips file inside the asset bundle.
pub const TIPS_PATH: &str = "data/tips.ron";

/// Most text lines a tip may have.
pub const MAX_TEXT_LINES: usize = 3;

/// Longest text line, in characters, before placeholders are filled in.
pub const MAX_LINE_CHARS: usize = 60;

/// Longest tip title, in characters.
pub const MAX_TITLE_CHARS: usize = 30;

/// Most lines of a tip's `how`, its instructions apart from the guide's
/// line (ticket 0475).
pub const MAX_HOW_LINES: usize = 3;

/// The tip box.
pub const LIMITS: TipLimits = TipLimits {
    text_lines: MAX_TEXT_LINES,
    line_chars: MAX_LINE_CHARS,
    title_chars: MAX_TITLE_CHARS,
    how_lines: MAX_HOW_LINES,
};

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
    /// The player is browsing a map whose look can zoom, with no lesson in
    /// play (ticket 0466).
    ZoomAvailable,
    /// A selected unit's move range has a `water` tile: a foot or flying
    /// unit could step into it, a mounted or armored one never can
    /// (ticket 0483).
    FirstWadeInReach,
    /// The forecast is open and a strike of either side is effective by
    /// its weapon kind's trait (a spear on a mounted unit, a bow on a
    /// flying one), not by a spell's element (ticket 0489).
    FirstEffective,
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
pub type Tip = engine::Tip<TipTrigger>;

/// Every tip, in file order.
pub type TipTable = engine::TipTable<TipTrigger>;

/// The guide's expression while `tip` shows: its own, else
/// [`DEFAULT_FACE`].
pub fn face(tip: &Tip) -> &str {
    tip.face_or(DEFAULT_FACE)
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
    let placeholder = |name: &str| name == CURSOR_PLACEHOLDER || Action::from_name(name).is_some();
    let face_problem = |face: &str| {
        guide
            .filter(|g| g.expression(face).is_none())
            .map(|_| format!("the guide's portrait has no expression \"{face}\""))
    };
    let rules = TipRules {
        limits: LIMITS,
        placeholder: &placeholder,
        face_problem: &face_problem,
        default_face: DEFAULT_FACE,
        widths: &shuyi_assets::font::CellWidths::NARROW,
    };
    let table = engine::from_source(file, source, &rules)?;
    let errors: Vec<ContentError> = table
        .tips
        .iter()
        .filter_map(|tip| {
            let problem = spoken_problem(&tip.text, "how")?;
            Some(ContentError::new(
                file,
                format!("tip \"{}\": {problem}", tip.id),
            ))
        })
        .collect();
    if errors.is_empty() {
        Ok(table)
    } else {
        Err(errors)
    }
}

/// What is wrong with the guide's line `text`, if anything: it names a
/// key (a placeholder). The guide is voiced (ticket 0475), and a clip made
/// ahead of time can't say a key the player rebound, so keys go in
/// `keys_go` (a tip's `how`, a lesson's items).
pub fn spoken_problem(text: &str, keys_go: &str) -> Option<String> {
    let name = placeholders(text).into_iter().next()?;
    Some(format!(
        "the text is spoken, so it names no key ({{{name}}}): keys go in the {keys_go}"
    ))
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
            TipTrigger::ZoomAvailable,
            TipTrigger::FirstWadeInReach,
            TipTrigger::FirstEffective,
        ] {
            assert!(table.for_trigger(trigger).is_some(), "{trigger:?}");
        }
    }

    #[test]
    fn placeholders_are_listed_in_order() {
        assert_eq!(placeholders("{A} x {B}{C} {"), ["A", "B", "C"]);
        assert_eq!(placeholders("none {open"), [] as [&str; 0]);
    }

    #[test]
    fn a_good_tip_loads() {
        let source = r#"(tips: [(id: "a", trigger: FirstForecast, title: "T",
            text: "Go on.", how: "Press {Confirm}.\n{Cursor}")])"#;
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
        let faces: Vec<&str> = table.tips.iter().map(face).collect();
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

    /// Ticket 0475: the guide's line is spoken, so a key in a tip's text
    /// fails, and the same key in its `how` loads.
    #[test]
    fn a_key_in_the_spoken_text_fails() {
        let source = r#"(tips: [(id: "a", trigger: FirstForecast, title: "T",
            text: "Press {Confirm} now.")])"#;
        assert_eq!(
            errors(source),
            ["tip \"a\": the text is spoken, so it names no key ({Confirm}): keys go in the how"]
        );
        let source = r#"(tips: [(id: "a", trigger: FirstForecast, title: "T",
            text: "Swing now.", how: "Attack: {Confirm}")])"#;
        assert_eq!(errors(source), Vec::<String>::new());
    }

    /// Every shipped tip has a line for the guide, and none names a key.
    #[test]
    fn the_shipped_tips_lines_name_no_key() {
        for tip in load(None).unwrap().tips {
            assert!(!tip.text.trim().is_empty(), "{}", tip.id);
            assert_eq!(placeholders(&tip.text), [] as [&str; 0], "{}", tip.id);
        }
    }

    #[test]
    fn a_syntax_error_is_reported() {
        assert_eq!(errors("(tips: [").len(), 1);
    }
}
