//! The game's data text a pack's `data.ron` translates (ADR-0045 §2):
//! every name, tip, lesson, title and note in the data files, by key
//! ([`from_tables`]), and the lookups that give each in a language. English
//! stays in the data files; a lookup is handed today's English and returns
//! the pack's text only when its entry was made from exactly that. The
//! dialogue's lines, the names table and the lookup by key are the
//! engine's.
use crate::prelude::*;

use alloc::collections::BTreeMap;

use shuyi_assets::lang::SourceText;
use trpg_core::{
    ArtDef, ArtTable, BattleDef, BattleNote, ClassId, ClassTable, ItemId, ItemTable, SkillDef,
    SkillTable, SpellDef, SpellTable,
};

use super::{Lang, LangCode, text_width};
use crate::chapter::ChapterDef;
use crate::dialogue::DialogueTable;
use crate::lesson::{self, Lesson, LessonTable};
use crate::names::Names;
use crate::terrain::TerrainDef;
use crate::tip::{MAX_LINE_CHARS, MAX_TEXT_LINES, MAX_TITLE_CHARS, Tip, TipTable};

/// Today's English for everything `data.ron` and `dialogue/*.ron` can
/// translate, by key: data text by key (`<file>.<id>.<field>`,
/// `names.<id>`, `tips.<id>.title`, `tips.<id>.text`, `lessons.<id>.title`,
/// `lessons.<id>.text`, `lessons.<id>.item_<n>` from 1), dialogue lines by
/// line id and captions by caption key, and the names table.
pub type DataText = SourceText;

/// The loaded content [`DataText`] is read from.
#[derive(Debug, Clone, Copy)]
pub struct Tables<'a> {
    /// Classes.
    pub classes: &'a ClassTable,
    /// Items.
    pub items: &'a ItemTable,
    /// Spells.
    pub spells: &'a SpellTable,
    /// Skills.
    pub skills: &'a SkillTable,
    /// Combat Arts.
    pub arts: &'a ArtTable,
    /// Terrain.
    pub terrain: &'a TerrainDef,
    /// The names table.
    pub names: &'a Names,
    /// Tips.
    pub tips: &'a TipTable,
    /// Lessons.
    pub lessons: &'a LessonTable,
    /// Chapters by id.
    pub chapters: &'a BTreeMap<String, ChapterDef>,
    /// Battles by id.
    pub battles: &'a BTreeMap<String, BattleDef>,
    /// Dialogue scenes.
    pub dialogue: &'a DialogueTable,
}

/// The key of the `name` of `id` in data file `file` (`items.potion.name`).
fn name_key(file: &str, id: &str) -> String {
    format!("{file}.{id}.name")
}

/// The key of tip `id`'s title.
fn tip_title_key(id: &str) -> String {
    format!("tips.{id}.title")
}

/// The key of tip `id`'s text.
fn tip_text_key(id: &str) -> String {
    format!("tips.{id}.text")
}

/// What every key of a lesson's text starts with.
const LESSON_PREFIX: &str = "lessons.";

/// The key of lesson `id`'s title.
fn lesson_title_key(id: &str) -> String {
    format!("{LESSON_PREFIX}{id}.title")
}

/// The key of lesson `id`'s text.
fn lesson_text_key(id: &str) -> String {
    format!("{LESSON_PREFIX}{id}.text")
}

/// The key of lesson `id`'s item number `index` (from 0):
/// `lessons.<id>.item_1`.
fn lesson_item_key(id: &str, index: usize) -> String {
    format!("{LESSON_PREFIX}{id}.item_{}", index + 1)
}

/// The key of chapter `id`'s title.
fn chapter_key(id: &str) -> String {
    format!("chapters.{id}.title")
}

/// What every key of a terrain's name starts with.
const TERRAIN_PREFIX: &str = "terrain.";

/// What every key of a battle file's text (its notes, its exit) starts with.
const BATTLE_PREFIX: &str = "battles.";

/// The key of battle `id`'s note number `index` (from 0): `battles.<id>.note_1`.
fn note_key(id: &str, index: usize) -> String {
    format!("{BATTLE_PREFIX}{id}.note_{}", index + 1)
}

/// The key of battle `id`'s exit name: `battles.<id>.exit`.
fn exit_key(id: &str) -> String {
    format!("{BATTLE_PREFIX}{id}.exit")
}

/// Whether `key` is the field of a tip that ends its key with `field`.
fn tip_field(key: &str, field: &str) -> bool {
    let rest = key.strip_prefix("tips.");
    rest.is_some_and(|rest| rest.strip_suffix(field).is_some())
}

/// Whether `key` is a tip's title.
fn is_tip_title_key(key: &str) -> bool {
    tip_field(key, ".title")
}

/// Whether `key` is a tip's text.
fn is_tip_text_key(key: &str) -> bool {
    tip_field(key, ".text")
}

/// What is wrong with a pack's text `text` for the data key `key`, if
/// anything: a tip or a lesson keeps to the limits of English ones,
/// counted in cells. (A name keeps to the engine's rules.)
pub(super) fn text_problem(key: &str, text: &str) -> Option<String> {
    if let Some(field) = key.strip_prefix(LESSON_PREFIX) {
        let field = field.rsplit('.').next().unwrap_or(field);
        return match field {
            "title" => title_problem(text),
            "text" => lesson::text_problem(text),
            _ => lesson::item_problem(text),
        };
    }
    if is_tip_title_key(key) {
        return title_problem(text);
    }
    if !is_tip_text_key(key) {
        return None;
    }
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() || lines.len() > MAX_TEXT_LINES {
        return Some(format!("the text must have 1 to {MAX_TEXT_LINES} lines"));
    }
    lines
        .iter()
        .find(|line| text_width(line) > MAX_LINE_CHARS)
        .map(|line| format!("a text line is over {MAX_LINE_CHARS} cells: {line:?}"))
}

/// What is wrong with a tip's or a lesson's title `text`, if anything.
fn title_problem(text: &str) -> Option<String> {
    let bad = text.is_empty() || text_width(text) > MAX_TITLE_CHARS;
    bad.then(|| format!("the title must be 1 to {MAX_TITLE_CHARS} cells"))
}

/// Every translatable text of `tables`, by key.
pub fn from_tables(tables: &Tables<'_>) -> DataText {
    let mut data = BTreeMap::new();
    let mut name = |file: &str, id: &str, name: &str| {
        data.insert(name_key(file, id), name.to_owned());
    };
    for class in tables.classes.classes.values() {
        name("classes", &class.id.0, &class.name);
    }
    for (id, item) in &tables.items.items {
        name("items", &id.0, item.name());
    }
    for spell in tables.spells.spells.values() {
        name("spells", &spell.id.0, &spell.name);
    }
    for skill in tables.skills.skills.values() {
        name("skills", &skill.id.0, &skill.name);
    }
    for art in tables.arts.arts.values() {
        name("arts", &art.id.0, &art.name);
    }
    let terrain = tables.terrain;
    for (display, rules) in terrain.display.terrains.iter().zip(&terrain.rules.terrains) {
        name("terrain", &display.id, &rules.name);
    }
    for tip in &tables.tips.tips {
        data.insert(tip_title_key(&tip.id), tip.title.clone());
        data.insert(tip_text_key(&tip.id), tip.text.clone());
    }
    for lesson in &tables.lessons.lessons {
        data.insert(lesson_title_key(&lesson.id), lesson.title.clone());
        data.insert(lesson_text_key(&lesson.id), lesson.text.clone());
        for (i, item) in lesson.items.iter().enumerate() {
            data.insert(lesson_item_key(&lesson.id, i), item.label.clone());
        }
    }
    for chapter in tables.chapters.values() {
        data.insert(chapter_key(&chapter.id), chapter.title.clone());
    }
    for (id, battle) in tables.battles {
        for (i, note) in battle.battle_notes.iter().enumerate() {
            data.insert(note_key(id, i), note.text.clone());
        }
        if let Some(exit) = &battle.exit_name {
            data.insert(exit_key(id), exit.clone());
        }
    }
    let mut text = DataText::new(data, tables.names.clone());
    text.add_dialogue(tables.dialogue);
    text
}

impl Lang {
    /// The name of class `id` in language `code`; `english` is its name
    /// in the classes file.
    pub fn class_name<'a>(&'a self, code: &LangCode, id: &ClassId, english: &'a str) -> &'a str {
        self.0
            .data_text(code, || name_key("classes", &id.0), english)
    }

    /// The name of item `id` in language `code`; `english` is its name in
    /// the items file.
    pub fn item_name<'a>(&'a self, code: &LangCode, id: &ItemId, english: &'a str) -> &'a str {
        self.0.data_text(code, || name_key("items", &id.0), english)
    }

    /// The name of `spell` in language `code`.
    pub fn spell_name<'a>(&'a self, code: &LangCode, spell: &'a SpellDef) -> &'a str {
        self.0
            .data_text(code, || name_key("spells", &spell.id.0), &spell.name)
    }

    /// The name of `skill` in language `code`.
    pub fn skill_name<'a>(&'a self, code: &LangCode, skill: &'a SkillDef) -> &'a str {
        self.0
            .data_text(code, || name_key("skills", &skill.id.0), &skill.name)
    }

    /// The name of `art` in language `code`.
    pub fn art_name<'a>(&'a self, code: &LangCode, art: &'a ArtDef) -> &'a str {
        self.0
            .data_text(code, || name_key("arts", &art.id.0), &art.name)
    }

    /// The name of the terrain the terrain file calls `english`, in
    /// language `code`. A battle knows its terrains by number, not by the
    /// file's string id, so the terrain is found by its English name.
    pub fn terrain_name<'a>(&'a self, code: &LangCode, english: &'a str) -> &'a str {
        self.0.found(code, TERRAIN_PREFIX, english)
    }

    /// The title of `tip` in language `code`.
    pub fn tip_title<'a>(&'a self, code: &LangCode, tip: &'a Tip) -> &'a str {
        self.0
            .data_text(code, || tip_title_key(&tip.id), &tip.title)
    }

    /// The text of `tip` in language `code`, placeholders and all.
    pub fn tip_text<'a>(&'a self, code: &LangCode, tip: &'a Tip) -> &'a str {
        self.0.data_text(code, || tip_text_key(&tip.id), &tip.text)
    }

    /// The title of `lesson` in language `code`.
    pub fn lesson_title<'a>(&'a self, code: &LangCode, lesson: &'a Lesson) -> &'a str {
        self.0
            .data_text(code, || lesson_title_key(&lesson.id), &lesson.title)
    }

    /// The text of `lesson` in language `code`, placeholders and all.
    pub fn lesson_text<'a>(&'a self, code: &LangCode, lesson: &'a Lesson) -> &'a str {
        self.0
            .data_text(code, || lesson_text_key(&lesson.id), &lesson.text)
    }

    /// The label of `lesson`'s item number `index` (from 0) in language
    /// `code`, placeholders and all; empty if it has no such item.
    pub fn lesson_item<'a>(&'a self, code: &LangCode, lesson: &'a Lesson, index: usize) -> &'a str {
        let Some(item) = lesson.items.get(index) else {
            return "";
        };
        self.0
            .data_text(code, || lesson_item_key(&lesson.id, index), &item.label)
    }

    /// The title of `chapter` in language `code`.
    pub fn chapter_title<'a>(&'a self, code: &LangCode, chapter: &'a ChapterDef) -> &'a str {
        self.0
            .data_text(code, || chapter_key(&chapter.id), &chapter.title)
    }

    /// The text of battle note `note` in language `code`. A battle in play
    /// doesn't know which file it came from, so the note is found by its
    /// English among the battle files' notes; one that is in no file (a
    /// battle made by a test) stays as it is.
    pub fn battle_note<'a>(&'a self, code: &LangCode, note: &'a BattleNote) -> &'a str {
        self.0.found(code, BATTLE_PREFIX, &note.text)
    }

    /// The name of a battle's exit, `english`, in language `code`: found by
    /// its English among the battle files' exits, as a note is.
    pub fn exit_name<'a>(&'a self, code: &LangCode, english: &'a str) -> &'a str {
        self.0.found(code, BATTLE_PREFIX, english)
    }
}
