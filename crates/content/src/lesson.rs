//! Checklist lessons (`assets/data/lessons.ron`, ticket 0456,
//! `docs/design/tutorial.md`, *The lessons*): the guide's box with a list
//! of things to do, ticked as the player does them, in any order. What
//! starts each, at which tutorial level it shows, and what ticks each
//! item. The battle screen decides when a trigger has happened and when
//! an item is done; the campaign remembers which lessons are done.
use crate::prelude::*;

use alloc::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use trpg_core::{BattleDef, TutorialLevel};

use crate::bundle;
use crate::error::ContentError;
use crate::keymap::Action;
use crate::lang::text_width;
use crate::portrait::Portrait;
use crate::ron_loader::parse_ron;
use crate::tip::{
    CURSOR_PLACEHOLDER, DEFAULT_FACE, GUIDE_PORTRAIT, MAX_LINE_CHARS, MAX_TITLE_CHARS, TipTrigger,
    placeholders,
};

/// Path of the lessons file inside the asset bundle.
pub const LESSONS_PATH: &str = "data/lessons.ron";

/// Most text lines a lesson may have, above its checklist.
pub const MAX_TEXT_LINES: usize = 2;

/// Most items a lesson's checklist may have: the box stays within half
/// the map's height.
pub const MAX_ITEMS: usize = 6;

/// Longest item label, in characters, before placeholders are filled in:
/// with its `[ ] ` mark it is as wide as a text line.
pub const MAX_ITEM_CHARS: usize = MAX_LINE_CHARS - 4;

/// At which tutorial levels a lesson shows (`tutorial.md`, *How familiar
/// are you?*).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum LessonLevel {
    /// The basic controls: only at [`TutorialLevel::Full`].
    Full,
    /// What is different in this game: at `Full` and `Short`.
    Short,
}

impl LessonLevel {
    /// Whether a lesson of this level shows in a campaign at `level`.
    pub fn shows_at(self, level: TutorialLevel) -> bool {
        match self {
            Self::Full => level == TutorialLevel::Full,
            Self::Short => matches!(level, TutorialLevel::Full | TutorialLevel::Short),
        }
    }
}

/// What ticks an item: something the battle screen notices the player
/// do, never a raw key (the `keyboard-input` skill).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DoneWhen {
    /// The battle used the action (a cursor direction, Info, Next unit,
    /// Rewind, Danger zone…): pressed while the map was the player's to
    /// play, not taken by a box over it.
    Pressed(Action),
    /// A unit of the player's was selected.
    UnitSelected,
    /// The player picked where a selected unit goes (staying put too), or
    /// moved it after its action.
    UnitMoved,
    /// The attack forecast opened.
    ForecastOpened,
    /// A unit of the player's attacked.
    Attacked,
    /// The rewind screen opened.
    RewindOpened,
    /// The player ended their phase. The only item that lets the turn end
    /// while its lesson is open.
    TurnEnded,
}

/// [`DoneWhen`] as written: an action by name (`Pressed("CursorUp")`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
enum RawDone {
    Pressed(String),
    UnitSelected,
    UnitMoved,
    ForecastOpened,
    Attacked,
    RewindOpened,
    TurnEnded,
}

impl RawDone {
    /// The condition, if its action's name is known.
    fn parse(&self) -> Option<DoneWhen> {
        Some(match self {
            Self::Pressed(name) => DoneWhen::Pressed(Action::from_name(name)?),
            Self::UnitSelected => DoneWhen::UnitSelected,
            Self::UnitMoved => DoneWhen::UnitMoved,
            Self::ForecastOpened => DoneWhen::ForecastOpened,
            Self::Attacked => DoneWhen::Attacked,
            Self::RewindOpened => DoneWhen::RewindOpened,
            Self::TurnEnded => DoneWhen::TurnEnded,
        })
    }
}

/// One thing to do in a lesson.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LessonItem {
    /// What it says, with `{Action}` placeholders for key names.
    pub label: String,
    /// What ticks it.
    pub done_when: DoneWhen,
}

/// One lesson.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lesson {
    /// Stable id (`snake_case`); what the campaign keeps once it is done.
    pub id: String,
    /// At which tutorial levels it shows.
    pub level: LessonLevel,
    /// What starts it (the tips' triggers).
    pub trigger: TipTrigger,
    /// The battles it may start in, by id; any battle if empty.
    pub battles: Vec<String>,
    /// Its heading in the box.
    pub title: String,
    /// What it says above the checklist: lines separated by `\n`, with
    /// `{Action}` placeholders for key names.
    pub text: String,
    /// The checklist, in the order shown.
    pub items: Vec<LessonItem>,
    /// The guide's expression while it shows; [`DEFAULT_FACE`] if none.
    pub face: Option<String>,
}

impl Lesson {
    /// The guide's expression while the lesson shows.
    pub fn face(&self) -> &str {
        self.face.as_deref().unwrap_or(DEFAULT_FACE)
    }

    /// Whether it may start in battle `battle` (`None`: a battle that is
    /// no file's, as a screen test makes).
    pub fn starts_in(&self, battle: Option<&str>) -> bool {
        self.battles.is_empty() || battle.is_some_and(|b| self.battles.iter().any(|id| id == b))
    }

    /// Whether one of its items is done by `done`.
    pub fn has(&self, done: DoneWhen) -> bool {
        self.items.iter().any(|item| item.done_when == done)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawItem {
    label: String,
    done_when: RawDone,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLesson {
    id: String,
    level: LessonLevel,
    trigger: TipTrigger,
    #[serde(default)]
    battles: Vec<String>,
    title: String,
    text: String,
    items: Vec<RawItem>,
    #[serde(default, deserialize_with = "some_name")]
    face: Option<String>,
}

/// A name written plainly (`face: "sad"`, not `Some("sad")`).
fn some_name<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    String::deserialize(d).map(Some)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    lessons: Vec<RawLesson>,
}

/// Every lesson, in file order (the story's, then the test files').
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LessonTable {
    /// The lessons.
    pub lessons: Vec<Lesson>,
}

impl LessonTable {
    /// The lesson `id`.
    pub fn get(&self, id: &str) -> Option<&Lesson> {
        self.lessons.iter().find(|l| l.id == id)
    }

    /// The lessons `trigger` starts, in file order.
    pub fn for_trigger(&self, trigger: TipTrigger) -> impl Iterator<Item = &Lesson> {
        self.lessons.iter().filter(move |l| l.trigger == trigger)
    }
}

/// What the lessons are checked against: the battles they may name and
/// the guide's portrait their faces must be expressions of. A check is
/// skipped for what is `None` (it failed to load).
#[derive(Debug, Clone, Copy, Default)]
pub struct Refs<'a> {
    /// Battles by id.
    pub battles: Option<&'a BTreeMap<String, BattleDef>>,
    /// Portraits by id; the guide's is [`GUIDE_PORTRAIT`].
    pub faces: Option<&'a BTreeMap<String, Portrait>>,
}

/// Loads and validates the embedded lessons file, with the test files'
/// lessons when the build has them (ADR-0057).
pub fn load(refs: Refs<'_>) -> Result<LessonTable, Vec<ContentError>> {
    let display = bundle::display_path(LESSONS_PATH);
    let source = bundle::file(LESSONS_PATH).ok_or_else(|| {
        vec![ContentError::new(
            &display,
            "file not found in asset bundle",
        )]
    })?;
    let part = bundle::test_part(LESSONS_PATH);
    let part = part.as_ref().map(|(file, source)| (file.as_str(), *source));
    from_sources(&display, source, part, refs)
}

/// Parses and validates lessons `source`, attributing errors to `file`.
/// Reports every problem found.
pub fn from_source(
    file: &str,
    source: &str,
    refs: Refs<'_>,
) -> Result<LessonTable, Vec<ContentError>> {
    from_sources(file, source, None, refs)
}

/// [`from_source`] with the lessons of `part` (a file name and its
/// source: the test files' lessons) added after `source`'s. An id both
/// have is used twice, an error.
pub fn from_sources(
    file: &str,
    source: &str,
    part: Option<(&str, &str)>,
    refs: Refs<'_>,
) -> Result<LessonTable, Vec<ContentError>> {
    let mut files = vec![(file, source)];
    files.extend(part);
    let mut errors = Vec::new();
    let mut ids = BTreeSet::new();
    let mut lessons = Vec::new();
    for (file, source) in files {
        let raw: RawFile = match parse_ron(file, source) {
            Ok(raw) => raw,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        for raw in raw.lessons {
            let mut problems = problems(&raw, refs);
            if !ids.insert(raw.id.clone()) {
                problems.push("the id is used twice".to_owned());
            }
            let what = |p: String| ContentError::new(file, format!("lesson \"{}\": {p}", raw.id));
            if problems.is_empty() {
                lessons.push(validated(raw));
            } else {
                errors.extend(problems.into_iter().map(what));
            }
        }
    }
    if errors.is_empty() {
        Ok(LessonTable { lessons })
    } else {
        Err(errors)
    }
}

/// `raw` as a [`Lesson`]; every action name in it is known.
fn validated(raw: RawLesson) -> Lesson {
    let items = raw.items.into_iter().filter_map(|item| {
        Some(LessonItem {
            done_when: item.done_when.parse()?,
            label: item.label,
        })
    });
    Lesson {
        items: items.collect(),
        id: raw.id,
        level: raw.level,
        trigger: raw.trigger,
        battles: raw.battles,
        title: raw.title,
        text: raw.text,
        face: raw.face,
    }
}

/// What is wrong with lesson `raw`, besides its id being used twice.
fn problems(raw: &RawLesson, refs: Refs<'_>) -> Vec<String> {
    let mut out = Vec::new();
    let id_ok = !raw.id.is_empty()
        && raw
            .id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    if !id_ok {
        out.push("the id must be lowercase letters, digits and `_`".to_owned());
    }
    if raw.title.is_empty() || text_width(&raw.title) > MAX_TITLE_CHARS {
        out.push(format!(
            "the title must be 1 to {MAX_TITLE_CHARS} characters"
        ));
    }
    out.extend(text_problem(&raw.text));
    if raw.items.is_empty() || raw.items.len() > MAX_ITEMS {
        out.push(format!("a lesson must have 1 to {MAX_ITEMS} items"));
    }
    let mut seen = BTreeSet::new();
    for item in &raw.items {
        out.extend(item_problem(&item.label));
        match item.done_when.parse() {
            None => out.push(format!("unknown action in {:?}", item.done_when)),
            Some(done) if !seen.insert(done) => {
                out.push(format!("two items are done by {done:?}"));
            }
            Some(_) => {}
        }
    }
    for text in core::iter::once(&raw.text).chain(raw.items.iter().map(|i| &i.label)) {
        for name in placeholders(text) {
            if name != CURSOR_PLACEHOLDER && Action::from_name(name).is_none() {
                out.push(format!("unknown placeholder {{{name}}}"));
            }
        }
    }
    if let Some(battles) = refs.battles {
        for id in raw.battles.iter().filter(|id| !battles.contains_key(*id)) {
            out.push(format!("no battle \"{id}\""));
        }
    }
    let guide = refs.faces.and_then(|f| f.get(GUIDE_PORTRAIT));
    let face = raw.face.as_deref().unwrap_or(DEFAULT_FACE);
    if guide.is_some_and(|g| g.expression(face).is_none()) {
        out.push(format!("the guide's portrait has no expression \"{face}\""));
    }
    out
}

/// What is wrong with a lesson's text `text`, if anything: 1 to
/// [`MAX_TEXT_LINES`] lines of at most [`MAX_LINE_CHARS`] cells. A
/// translation keeps to the same.
pub fn text_problem(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() || lines.len() > MAX_TEXT_LINES {
        return Some(format!("the text must have 1 to {MAX_TEXT_LINES} lines"));
    }
    lines
        .iter()
        .find(|line| text_width(line) > MAX_LINE_CHARS)
        .map(|line| format!("a text line is over {MAX_LINE_CHARS} cells: {line:?}"))
}

/// What is wrong with an item's label `label`, if anything: one line of 1
/// to [`MAX_ITEM_CHARS`] cells. A translation keeps to the same.
pub fn item_problem(label: &str) -> Option<String> {
    let ok = !label.is_empty() && !label.contains('\n') && text_width(label) <= MAX_ITEM_CHARS;
    (!ok).then(|| format!("an item must be one line of 1 to {MAX_ITEM_CHARS} cells: {label:?}"))
}

#[cfg(test)]
mod tests;
