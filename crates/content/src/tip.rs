//! One-time contextual tips (`assets/data/tips.ron`, ticket 0406): what
//! triggers each and what it says. The screens decide when a trigger has
//! happened and remember which tips were seen.
//!
//! The tip schema and its checks are the engine's (`shuyi_assets::tip`,
//! ticket 1105); the game adds its triggers ([`TipTrigger`]), its box, its
//! placeholders and the guide who speaks every tip.
use crate::keymap::ActionSet;
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

/// Longest trigger name, in bytes.
pub const MAX_TRIGGER_CHARS: usize = 32;

/// What makes a tip or a lesson appear: a name, as `tips.ron` and
/// `lessons.ron` write it (`trigger: FirstForecast`), with no list of its
/// own here (ticket 1310). The ui's registry of trigger rows is the list
/// of known names and says when each fires; a ui test fails on a name the
/// files use with no row. The constants below are the names the game's
/// code and tests write; a new trigger needs none.
///
/// The name is kept in place (a short identifier, at most
/// [`MAX_TRIGGER_CHARS`] bytes), so a trigger is `Copy` and needs no
/// allocation. Names order as their text does.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TipTrigger {
    /// The name's bytes, then zeros.
    bytes: [u8; MAX_TRIGGER_CHARS],
}

impl TipTrigger {
    /// The trigger named `name`, a Rust-style identifier of at most
    /// [`MAX_TRIGGER_CHARS`] bytes (checked when built at compile time:
    /// the build fails on a bad one).
    pub const fn new(name: &str) -> Self {
        match Self::try_new(name) {
            Some(t) => t,
            None => panic!("a trigger name is an identifier of at most 32 bytes"),
        }
    }

    /// The trigger named `name`, if it is an identifier (ASCII letters,
    /// digits and `_`, not starting with a digit) of at most
    /// [`MAX_TRIGGER_CHARS`] bytes.
    pub const fn try_new(name: &str) -> Option<Self> {
        let src = name.as_bytes();
        if src.is_empty() || src.len() > MAX_TRIGGER_CHARS || src[0].is_ascii_digit() {
            return None;
        }
        let mut bytes = [0; MAX_TRIGGER_CHARS];
        let mut i = 0;
        while i < src.len() {
            let b = src[i];
            if !(b.is_ascii_alphanumeric() || b == b'_') {
                return None;
            }
            bytes[i] = b;
            i += 1;
        }
        Some(Self { bytes })
    }

    /// The name, as the files write it.
    pub fn name(&self) -> &str {
        let len = self
            .bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(MAX_TRIGGER_CHARS);
        // Only ASCII is ever stored (`try_new`).
        core::str::from_utf8(&self.bytes[..len]).unwrap_or_default()
    }
}

/// The names the game's code and tests write, spelled as the files spell
/// them so `TipTrigger::FirstForecast` reads as the enum's variant did.
#[allow(non_upper_case_globals)]
impl TipTrigger {
    /// A battle starts.
    pub const FirstBattleStart: Self = Self::new("FirstBattleStart");
    /// The player selects a unit.
    pub const FirstUnitSelected: Self = Self::new("FirstUnitSelected");
    /// A selected unit could reach an enemy.
    pub const FirstEnemyInRange: Self = Self::new("FirstEnemyInRange");
    /// The attack forecast opens.
    pub const FirstForecast: Self = Self::new("FirstForecast");
    /// A player unit is badly hurt.
    pub const FirstLowHp: Self = Self::new("FirstLowHp");
    /// A player unit levels up.
    pub const FirstLevelUp: Self = Self::new("FirstLevelUp");
    /// The enemy phase begins.
    pub const FirstEnemyPhase: Self = Self::new("FirstEnemyPhase");
    /// The cursor rests on an enemy, so its danger zone is worth knowing.
    pub const DangerZoneAvailable: Self = Self::new("DangerZoneAvailable");
    /// The player selects a unit that has a skill (ticket 0458).
    pub const FirstSkillUnit: Self = Self::new("FirstSkillUnit");
    /// The action menu opens for a unit that knows a spell (0458).
    pub const FirstCaster: Self = Self::new("FirstCaster");
    /// The cursor rests on a tile whose terrain is not plain ground: a
    /// Def, Avoid or healing bonus, or a movement cost other than 1
    /// (0458).
    pub const FirstTerrain: Self = Self::new("FirstTerrain");
    /// The player is browsing a map whose look can zoom, with no lesson in
    /// play (ticket 0466).
    pub const ZoomAvailable: Self = Self::new("ZoomAvailable");
    /// A selected unit's move range has a `water` tile: a foot or flying
    /// unit could step into it, a mounted or armored one never can
    /// (ticket 0483).
    pub const FirstWadeInReach: Self = Self::new("FirstWadeInReach");
    /// The forecast is open and a strike of either side is effective by
    /// its weapon kind's trait (a spear on a mounted unit, a bow on a
    /// flying one), not by a spell's element (ticket 0489).
    pub const FirstEffective: Self = Self::new("FirstEffective");
    /// The forecast is open and left and right swap the weapon: its unit
    /// has more than one weapon (or spell) to attack with there (ticket
    /// 0463).
    pub const FirstWeaponSwap: Self = Self::new("FirstWeaponSwap");
    /// The Load Game list opens with a save in it, where saves can be
    /// exported (ticket 0874).
    pub const FirstLoadGame: Self = Self::new("FirstLoadGame");
    /// The Profile screen opens, where saves can be exported (0874).
    pub const FirstProfileScreen: Self = Self::new("FirstProfileScreen");
    // The triggers below start lessons only (ticket 0457): no tip uses them
    // yet.
    /// In the player's phase, every unit of the player's has acted.
    pub const NoUnitLeft: Self = Self::new("NoUnitLeft");
    /// In turn 1's player phase, back on the map after a unit of the
    /// player's has acted (ticket 1304): never in a later turn.
    pub const FirstUnitActed: Self = Self::new("FirstUnitActed");
    /// The player's phase of turn 2 begins: there is a turn to undo.
    pub const SecondTurn: Self = Self::new("SecondTurn");
    /// The threat arrows show: an enemy could attack a unit of the
    /// player's where it stands or is about to move.
    pub const ThreatArrowsShown: Self = Self::new("ThreatArrowsShown");
    /// A unit's action menu opens with `Arts` it can use.
    pub const ArtsAvailable: Self = Self::new("ArtsAvailable");
    /// A unit's spell list opens with a spell it could cast on a tile
    /// (to change it) from there.
    pub const TileSpellAvailable: Self = Self::new("TileSpellAvailable");
}

/// The bare name, as the files write it (the engine's checks print a
/// trigger with `{:?}`: "`FirstForecast` already has a tip").
impl core::fmt::Debug for TipTrigger {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

impl core::fmt::Display for TipTrigger {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

/// Read from a bare identifier, as an enum's variant was
/// (`trigger: FirstForecast`).
impl<'de> Deserialize<'de> for TipTrigger {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Name;
        impl serde::de::Visitor<'_> for Name {
            type Value = TipTrigger;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(
                    f,
                    "a trigger's name, an identifier of at most {MAX_TRIGGER_CHARS} bytes"
                )
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<TipTrigger, E> {
                TipTrigger::try_new(v)
                    .ok_or_else(|| E::invalid_value(serde::de::Unexpected::Str(v), &self))
            }
        }
        d.deserialize_identifier(Name)
    }
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
    from_source(&display, &source, faces)
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
        widths: &crate::font::widths(),
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
            TipTrigger::FirstForecast,
            TipTrigger::FirstLowHp,
            TipTrigger::FirstLevelUp,
            TipTrigger::FirstEnemyPhase,
            TipTrigger::DangerZoneAvailable,
            TipTrigger::FirstSkillUnit,
            TipTrigger::FirstCaster,
            TipTrigger::FirstTerrain,
            TipTrigger::FirstWadeInReach,
            TipTrigger::FirstEffective,
            TipTrigger::FirstLoadGame,
            TipTrigger::FirstProfileScreen,
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
        assert!(has("over 60 cells"), "{all:?}");
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
        // Three lines over: the engine reports the problem once (engine
        // 2261), beside the title's.
        let source = source.replace(&line, &format!("{line}x"));
        let all = errors(&source);
        assert_eq!(all.len(), 2, "{all:?}");
        assert!(all.iter().any(|m| m.contains("over 60 cells")), "{all:?}");
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

    /// Ticket 1310: a trigger is a name, so a tip with a name this crate
    /// has never heard of loads, and is found by that name.
    #[test]
    fn a_new_trigger_name_loads_with_no_edit_here() {
        let source = r#"(tips: [(id: "a", trigger: SomeNewTrigger, title: "T", text: "x")])"#;
        let table = from_source("t.ron", source, None).unwrap();
        let name = TipTrigger::new("SomeNewTrigger");
        assert_eq!(table.for_trigger(name), Some(&table.tips[0]));
        assert_eq!(name.name(), "SomeNewTrigger");
        assert_eq!(format!("{name:?} {name}"), "SomeNewTrigger SomeNewTrigger");
        // The constants are the same names the files write.
        let forecast = r#"(tips: [(id: "a", trigger: FirstForecast, title: "T", text: "x")])"#;
        let table = from_source("t.ron", forecast, None).unwrap();
        assert_eq!(table.tips[0].trigger, TipTrigger::FirstForecast);
        assert_eq!(TipTrigger::FirstForecast.name(), "FirstForecast");
    }

    /// A name is an identifier of at most 32 bytes; a string, an empty or
    /// a longer name fails to load.
    #[test]
    fn a_trigger_name_is_a_short_identifier() {
        assert!(TipTrigger::try_new("A_b9").is_some());
        let longest = "A".repeat(MAX_TRIGGER_CHARS);
        assert_eq!(TipTrigger::new(&longest).name(), longest);
        for bad in ["", "9Lives", "Two words", "Ünicode", &format!("{longest}A")] {
            assert_eq!(TipTrigger::try_new(bad), None, "{bad:?}");
        }
        let tip = |trigger: &str| {
            format!(r#"(tips: [(id: "a", trigger: {trigger}, title: "T", text: "x")])"#)
        };
        assert_eq!(errors(&tip(&longest)), Vec::<String>::new());
        let too_long = errors(&tip(&format!("{longest}A")));
        assert_eq!(too_long.len(), 1);
        let expected = "an identifier of at most 32 bytes";
        assert!(too_long[0].contains(expected), "{too_long:?}");
        assert_eq!(errors(&tip("\"FirstForecast\"")).len(), 1);
        // A name used twice is still caught, by its name.
        let twice = r#"(tips: [(id: "a", trigger: NewOne, title: "T", text: "x"),
            (id: "b", trigger: NewOne, title: "T", text: "x")])"#;
        assert!(
            errors(twice)
                .iter()
                .any(|m| m.contains("NewOne already has a tip"))
        );
    }

    /// Names order as their text does (the engine's duplicate check keeps
    /// them in a set).
    #[test]
    fn names_order_as_their_text() {
        let mut names = ["SecondTurn", "FirstLowHp", "First", "FirstLevelUp"].map(TipTrigger::new);
        names.sort();
        assert_eq!(
            names.map(|n| n.name().to_owned()),
            ["First", "FirstLevelUp", "FirstLowHp", "SecondTurn"]
        );
    }

    #[test]
    fn a_syntax_error_is_reported() {
        assert_eq!(errors("(tips: [").len(), 1);
    }
}
