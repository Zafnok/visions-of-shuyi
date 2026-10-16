//! The English a pack's `data.ron` and `dialogue/` files translate
//! (ADR-0045 §2): every name, tip, lesson, title and note in the data files and
//! every dialogue line and caption, by key ([`DataText`]), and the lookups
//! that give each in a language. English stays in the data files; a lookup
//! is handed today's English and returns the pack's text only when its
//! entry was made from exactly that.
use crate::prelude::*;

use alloc::collections::BTreeMap;

use trpg_core::{
    ArtDef, ArtTable, BattleDef, BattleNote, ClassId, ClassTable, ItemId, ItemTable, SkillDef,
    SkillTable, SpellDef, SpellTable,
};

use super::{Lang, LangCode, text_width};
use crate::chapter::ChapterDef;
use crate::dialogue::{DialogueTable, REPLY_SPEAKER, Step, caption_key};
use crate::lesson::{self, Lesson, LessonTable};
use crate::names::Names;
use crate::terrain::TerrainDef;
use crate::tip::{MAX_LINE_CHARS, MAX_TEXT_LINES, MAX_TITLE_CHARS, Tip, TipTable};

/// What kind of dialogue text a key is for: its length limit depends on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    /// A speech or narration line (one text box).
    Speech,
    /// One of the lead's replies (a menu row).
    Reply,
    /// A `@caption`.
    Caption,
}

/// One dialogue line or caption in English.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLine {
    /// The id of its scene.
    pub scene: String,
    /// Its text as written (tokens unexpanded).
    pub text: String,
    /// What it is.
    pub kind: LineKind,
    /// The stem of the `.dlg` file its scene is in (`ch01` for
    /// `ch01.dlg`); empty if the dialogue wasn't read from files.
    pub file: String,
}

/// Today's English for everything `data.ron` and `dialogue/*.ron` can
/// translate, by key.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataText {
    /// Data text by key: `<file>.<id>.<field>`, `names.<id>`,
    /// `tips.<id>.title`, `tips.<id>.text`, `lessons.<id>.title`,
    /// `lessons.<id>.text`, `lessons.<id>.item_<n>` (from 1).
    pub data: BTreeMap<String, String>,
    /// Dialogue lines by line id, and captions by [`caption_key`].
    pub lines: BTreeMap<String, SourceLine>,
    /// The names table: what a name token in a pack's text may refer to.
    pub names: Names,
}

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

/// The key of the name `id` of the names table.
fn names_key(id: &str) -> String {
    format!("names.{id}")
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

/// What every key of a battle note starts with.
const NOTE_PREFIX: &str = "battles.";

/// The key of battle `id`'s note number `index` (from 0): `battles.<id>.note_1`.
fn note_key(id: &str, index: usize) -> String {
    format!("{NOTE_PREFIX}{id}.note_{}", index + 1)
}

/// Whether `key` is a display name of the names table.
fn is_names_key(key: &str) -> bool {
    key.starts_with("names.")
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
/// anything: a name is not empty and holds no brace (names can't hold
/// tokens); a tip keeps to the limits of English tips, counted in cells.
pub(super) fn text_problem(key: &str, text: &str) -> Option<String> {
    if is_names_key(key) {
        if text.trim().is_empty() {
            return Some("a name must not be empty".to_owned());
        }
        return text
            .contains(['{', '}'])
            .then(|| "a name can't hold a brace".to_owned());
    }
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

/// `ch01` for `assets/dialogue/ch01.dlg`.
fn file_stem(path: &str) -> &str {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    name.strip_suffix(".dlg").unwrap_or(name)
}

/// Adds the captions of `steps` (of scene `scene`) to `lines`.
fn add_captions(scene: &str, file: &str, steps: &[Step], lines: &mut BTreeMap<String, SourceLine>) {
    for step in steps {
        match step {
            Step::Caption { text } => {
                let line = SourceLine {
                    scene: scene.to_owned(),
                    text: text.clone(),
                    kind: LineKind::Caption,
                    file: file.to_owned(),
                };
                lines.insert(caption_key(scene, text), line);
            }
            Step::Choice { options } => {
                for option in options {
                    add_captions(scene, file, &option.steps, lines);
                }
            }
            Step::If {
                then, otherwise, ..
            } => {
                add_captions(scene, file, then, lines);
                add_captions(scene, file, otherwise, lines);
            }
            Step::Place { .. }
            | Step::Clear { .. }
            | Step::Say { .. }
            | Step::Narrate { .. }
            | Step::Music(_)
            | Step::Setting(_) => {}
        }
    }
}

impl DataText {
    /// Every translatable text of `tables`, by key.
    pub fn from_tables(tables: &Tables<'_>) -> Self {
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
        for (id, value) in &tables.names.names {
            data.insert(names_key(id), value.clone());
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
        }
        let mut text = Self {
            data,
            lines: BTreeMap::new(),
            names: tables.names.clone(),
        };
        text.add_dialogue(tables.dialogue);
        text
    }

    /// Adds every line, reply and caption of `dialogue`.
    pub fn add_dialogue(&mut self, dialogue: &DialogueTable) {
        for scene in dialogue.scenes.values() {
            let file = dialogue
                .files
                .get(&scene.id)
                .map_or("", |path| file_stem(path));
            for line in scene.lines() {
                let kind = if line.speaker == REPLY_SPEAKER {
                    LineKind::Reply
                } else {
                    LineKind::Speech
                };
                let source = SourceLine {
                    scene: scene.id.clone(),
                    text: line.text.to_owned(),
                    kind,
                    file: file.to_owned(),
                };
                self.lines.insert(line.id.as_str().to_owned(), source);
            }
            add_captions(&scene.id, file, &scene.steps, &mut self.lines);
        }
    }
}

impl Lang {
    /// `english` in language `code`: the pack's `data.ron` text for the key
    /// `key` gives, when its entry was made from exactly `english`;
    /// otherwise `english`. The key is only built when `code` has a pack.
    fn data_text<'a>(
        &'a self,
        code: &LangCode,
        key: impl FnOnce() -> String,
        english: &'a str,
    ) -> &'a str {
        let Some(pack) = self.packs.get(code) else {
            return english;
        };
        match pack.data.get(&key()) {
            Some(entry) if entry.source == english => &entry.text,
            _ => english,
        }
    }

    /// The name of class `id` in language `code`; `english` is its name
    /// in the classes file.
    pub fn class_name<'a>(&'a self, code: &LangCode, id: &ClassId, english: &'a str) -> &'a str {
        self.data_text(code, || name_key("classes", &id.0), english)
    }

    /// The name of item `id` in language `code`; `english` is its name in
    /// the items file.
    pub fn item_name<'a>(&'a self, code: &LangCode, id: &ItemId, english: &'a str) -> &'a str {
        self.data_text(code, || name_key("items", &id.0), english)
    }

    /// The name of `spell` in language `code`.
    pub fn spell_name<'a>(&'a self, code: &LangCode, spell: &'a SpellDef) -> &'a str {
        self.data_text(code, || name_key("spells", &spell.id.0), &spell.name)
    }

    /// The name of `skill` in language `code`.
    pub fn skill_name<'a>(&'a self, code: &LangCode, skill: &'a SkillDef) -> &'a str {
        self.data_text(code, || name_key("skills", &skill.id.0), &skill.name)
    }

    /// The name of `art` in language `code`.
    pub fn art_name<'a>(&'a self, code: &LangCode, art: &'a ArtDef) -> &'a str {
        self.data_text(code, || name_key("arts", &art.id.0), &art.name)
    }

    /// The name of the terrain the terrain file calls `english`, in
    /// language `code`. A battle knows its terrains by number, not by the
    /// file's string id, so the terrain is found by its English name.
    pub fn terrain_name<'a>(&'a self, code: &LangCode, english: &'a str) -> &'a str {
        self.found(code, TERRAIN_PREFIX, english)
    }

    /// The display name `id` of the names table in language `code`;
    /// `english` is its value in the table.
    pub fn name<'a>(&'a self, code: &LangCode, id: &str, english: &'a str) -> &'a str {
        self.data_text(code, || names_key(id), english)
    }

    /// The names table in language `code`: each name the pack translates
    /// in its language, the others in English. What a name token in a
    /// line shown in `code` is filled from.
    pub fn names(&self, code: &LangCode, names: &Names) -> Names {
        if !self.packs.contains_key(code) {
            return names.clone();
        }
        let translated = |(id, english): (&String, &String)| {
            (id.clone(), self.name(code, id, english).to_owned())
        };
        Names {
            names: names.names.iter().map(translated).collect(),
        }
    }

    /// The title of `tip` in language `code`.
    pub fn tip_title<'a>(&'a self, code: &LangCode, tip: &'a Tip) -> &'a str {
        self.data_text(code, || tip_title_key(&tip.id), &tip.title)
    }

    /// The text of `tip` in language `code`, placeholders and all.
    pub fn tip_text<'a>(&'a self, code: &LangCode, tip: &'a Tip) -> &'a str {
        self.data_text(code, || tip_text_key(&tip.id), &tip.text)
    }

    /// The title of `lesson` in language `code`.
    pub fn lesson_title<'a>(&'a self, code: &LangCode, lesson: &'a Lesson) -> &'a str {
        self.data_text(code, || lesson_title_key(&lesson.id), &lesson.title)
    }

    /// The text of `lesson` in language `code`, placeholders and all.
    pub fn lesson_text<'a>(&'a self, code: &LangCode, lesson: &'a Lesson) -> &'a str {
        self.data_text(code, || lesson_text_key(&lesson.id), &lesson.text)
    }

    /// The label of `lesson`'s item number `index` (from 0) in language
    /// `code`, placeholders and all; empty if it has no such item.
    pub fn lesson_item<'a>(&'a self, code: &LangCode, lesson: &'a Lesson, index: usize) -> &'a str {
        let Some(item) = lesson.items.get(index) else {
            return "";
        };
        self.data_text(code, || lesson_item_key(&lesson.id, index), &item.label)
    }

    /// The title of `chapter` in language `code`.
    pub fn chapter_title<'a>(&'a self, code: &LangCode, chapter: &'a ChapterDef) -> &'a str {
        self.data_text(code, || chapter_key(&chapter.id), &chapter.title)
    }

    /// The text of battle note `note` in language `code`. A battle in play
    /// doesn't know which file it came from, so the note is found by its
    /// English among the battle files' notes; one that is in no file (a
    /// battle made by a test) stays as it is.
    pub fn battle_note<'a>(&'a self, code: &LangCode, note: &'a BattleNote) -> &'a str {
        self.found(code, NOTE_PREFIX, &note.text)
    }

    /// `english` in language `code`, for a text whose key isn't known
    /// where it is shown: the pack's text for a key starting with `prefix`
    /// whose English is `english` today and was when the entry was made;
    /// otherwise `english`.
    fn found<'a>(&'a self, code: &LangCode, prefix: &str, english: &'a str) -> &'a str {
        let Some(pack) = self.packs.get(code) else {
            return english;
        };
        let end = format!("{prefix}\u{10ffff}");
        self.source
            .data
            .range(prefix.to_owned()..end)
            .filter(|&(_, today)| today == english)
            .find_map(|(key, today)| pack.data.get(key).filter(|e| e.source == *today))
            .map_or(english, |entry| entry.text.as_str())
    }
}
