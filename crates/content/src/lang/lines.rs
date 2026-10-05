//! A pack's dialogue text (`dialogue/<file>.ron`, ADR-0045): one entry per
//! line id or caption key. Rewording an English line gives it a new id, so
//! the old entry becomes an **orphan**: it fails nothing, and
//! [`Lang::status`] pairs it with the new line it most likely belongs to.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};
use trpg_core::lead::LeadGender;

use super::data::{DataText, LineKind};
use super::{Lang, LangCode, LangPack};
use crate::dialogue::{
    ChoiceOption, LineId, MAX_OPTION_LEN, MAX_TEXT_LEN, Scene, Step, caption_key, longest_width,
    scene_of, token_problems,
};
use crate::error::ContentError;
use crate::names::Names;
use crate::ron_loader::parse_ron;

/// What every caption key starts with.
const CAPTION_PREFIX: &str = "caption.";

/// A dialogue entry as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLine {
    key: String,
    source: String,
    #[serde(default, deserialize_with = "written")]
    text: Option<String>,
    #[serde(default, deserialize_with = "written")]
    text_m: Option<String>,
    #[serde(default, deserialize_with = "written")]
    text_f: Option<String>,
}

/// A field that may be left out, written as its bare value when it is
/// there (`text: "..."`, not `text: Some("...")`).
fn written<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    String::deserialize(deserializer).map(Some)
}

/// A line in a pack's language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineText {
    /// The same whoever the lead is (`text`).
    One(String),
    /// One per gender of the lead (`text_m`, `text_f`), for a language
    /// where the lead's gender changes more than a pronoun.
    ByGender {
        /// With a male lead.
        male: String,
        /// With a female lead.
        female: String,
    },
}

impl LineText {
    /// The text for a lead of `gender`.
    pub fn for_lead(&self, gender: LeadGender) -> &str {
        match (self, gender) {
            (LineText::One(text), _) => text,
            (LineText::ByGender { male, .. }, LeadGender::Male) => male,
            (LineText::ByGender { female, .. }, LeadGender::Female) => female,
        }
    }

    /// Every text of the entry.
    fn all(&self) -> Vec<&str> {
        match self {
            LineText::One(text) => vec![text],
            LineText::ByGender { male, female } => vec![male, female],
        }
    }
}

/// One dialogue line or caption of a pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineEntry {
    /// The line id, or the caption key.
    pub key: String,
    /// The English the text was made from.
    pub source: String,
    /// The text in the pack's language.
    pub text: LineText,
}

/// A pack entry for a dialogue line that English no longer has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Orphan {
    /// The entry's key: the id the line had.
    pub key: String,
    /// The English it was made from.
    pub source: String,
    /// Its translation.
    pub text: LineText,
    /// The key of the line of the same scene, of the same kind and without
    /// an entry, whose English is nearest to `source`: most likely the
    /// reworded line. `None` if the scene has no such line.
    pub nearest: Option<String>,
}

/// The id of the scene the line id or caption key `key` belongs to, or
/// `None` if `key` is neither.
fn scene_of_key(key: &str) -> Option<&str> {
    scene_of(key.strip_prefix(CAPTION_PREFIX).unwrap_or(key))
}

/// Whether `key` is a caption key.
fn is_caption(key: &str) -> bool {
    key.starts_with(CAPTION_PREFIX)
}

/// The most cells a line of `kind` may take, if it has a limit.
fn limit(kind: LineKind) -> Option<usize> {
    match kind {
        LineKind::Speech => Some(MAX_TEXT_LEN),
        LineKind::Reply => Some(MAX_OPTION_LEN),
        LineKind::Caption => None,
    }
}

/// Parses and validates the dialogue entries of the pack file `file`
/// (`source`), adding them to `lines` and every problem to `errors`.
///
/// An entry has `text`, or both `text_m` and `text_f`. Its key is a line
/// id or a caption key, used once in the pack. Its text follows the token
/// rule of English dialogue (only lead tokens and name tokens with an id
/// in `data`'s names table). When English has the line, the text also
/// keeps to the line's length limit, counted in cells, with every name
/// token as long as the longest of `names` (the pack's).
pub(super) fn add_lines(
    (file, source): (&str, &str),
    data: &DataText,
    names: &Names,
    lines: &mut BTreeMap<String, LineEntry>,
    errors: &mut Vec<ContentError>,
) {
    let raw = parse_ron::<Vec<RawLine>>(file, source)
        .map_err(|e| errors.push(e))
        .unwrap_or_default();
    for entry in raw {
        let mut err = |message: String| {
            errors.push(ContentError::new(
                file,
                format!("\"{}\": {message}", entry.key),
            ));
        };
        if scene_of_key(&entry.key).is_none() {
            err("not a line id or a caption key".to_owned());
        }
        if lines.contains_key(&entry.key) {
            err("the key is used twice".to_owned());
        }
        let text = match (entry.text, entry.text_m, entry.text_f) {
            (Some(text), None, None) => LineText::One(text),
            (None, Some(male), Some(female)) => LineText::ByGender { male, female },
            _ => {
                err("an entry has `text`, or both `text_m` and `text_f`".to_owned());
                continue;
            }
        };
        let english = data.lines.get(&entry.key);
        for text in text.all() {
            if text.trim().is_empty() {
                err("the text must not be empty".to_owned());
            }
            for problem in token_problems(text, Some(&data.names)) {
                err(problem);
            }
            let width = longest_width(text, Some(names.longest()));
            match english.and_then(|line| limit(line.kind)) {
                Some(limit) if width > limit => {
                    err(format!("text is {width} cells; the limit is {limit}"));
                }
                _ => {}
            }
        }
        let entry = LineEntry {
            key: entry.key,
            source: entry.source,
            text,
        };
        lines.insert(entry.key.clone(), entry);
    }
}

/// How many single-character edits turn `a` into `b` (Levenshtein).
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let substituted = diagonal + usize::from(ca != cb);
            diagonal = row[j + 1];
            row[j + 1] = substituted.min(diagonal + 1).min(row[j] + 1);
        }
    }
    row[b.len()]
}

impl LangPack {
    /// The pack's entry for the dialogue line or caption `key`, if it has
    /// one.
    pub fn line(&self, key: &str) -> Option<&LineEntry> {
        self.lines.get(key)
    }

    /// The pack's dialogue entries English has no line for, each paired
    /// with the nearest untranslated line of its scene, in key order.
    pub(super) fn orphans(&self, data: &DataText) -> Vec<Orphan> {
        let orphan = |entry: &LineEntry| {
            let scene = scene_of_key(&entry.key).unwrap_or_default();
            let caption = is_caption(&entry.key);
            let nearest = data
                .lines
                .iter()
                .filter(|(key, line)| {
                    line.scene == scene
                        && (line.kind == LineKind::Caption) == caption
                        && !self.lines.contains_key(*key)
                })
                .min_by_key(|(_, line)| distance(&entry.source, &line.text))
                .map(|(key, _)| key.clone());
            Orphan {
                key: entry.key.clone(),
                source: entry.source.clone(),
                text: entry.text.clone(),
                nearest,
            }
        };
        self.lines
            .values()
            .filter(|entry| !data.lines.contains_key(&entry.key))
            .map(orphan)
            .collect()
    }
}

impl Lang {
    /// The dialogue line or caption `key` in language `code`, for a lead
    /// of `gender`: the pack's text when its entry was made from exactly
    /// `english`; otherwise `english`.
    pub fn line<'a>(
        &'a self,
        code: &LangCode,
        key: &str,
        english: &'a str,
        gender: LeadGender,
    ) -> &'a str {
        let entry = self.packs.get(code).and_then(|pack| pack.lines.get(key));
        match entry {
            Some(entry) if entry.source == english => entry.text.for_lead(gender),
            _ => english,
        }
    }

    /// `scene` in language `code`, for a lead of `gender`: every line,
    /// reply and caption the pack translates in its language, the others
    /// in English. The lines keep their ids (a voice clip is found by
    /// them) and their tokens.
    #[must_use]
    pub fn scene(&self, code: &LangCode, scene: &Scene, gender: LeadGender) -> Scene {
        let mut scene = scene.clone();
        if self.packs.contains_key(code) {
            let id = scene.id.clone();
            self.translate(code, &id, &mut scene.steps, gender);
        }
        scene
    }

    /// Puts the steps `steps` of scene `scene` in language `code`.
    fn translate(&self, code: &LangCode, scene: &str, steps: &mut [Step], gender: LeadGender) {
        let line = |id: &LineId, text: &mut String| {
            let shown = self.line(code, id.as_str(), text, gender).to_owned();
            *text = shown;
        };
        for step in steps {
            match step {
                Step::Caption { text } => {
                    let shown = self.line(code, &caption_key(scene, text), text, gender);
                    *text = shown.to_owned();
                }
                Step::Say { text, line: id, .. } | Step::Narrate { text, line: id } => {
                    line(id, text);
                }
                Step::Choice { options } => {
                    for ChoiceOption {
                        text,
                        line: id,
                        steps,
                        ..
                    } in options
                    {
                        line(id, text);
                        self.translate(code, scene, steps, gender);
                    }
                }
                Step::If {
                    then, otherwise, ..
                } => {
                    self.translate(code, scene, then, gender);
                    self.translate(code, scene, otherwise, gender);
                }
                Step::Place { .. } | Step::Clear { .. } | Step::Music(_) => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_counts_single_character_edits() {
        assert_eq!(distance("", ""), 0);
        assert_eq!(distance("abc", "abc"), 0);
        assert_eq!(distance("", "abc"), 3);
        assert_eq!(distance("abc", ""), 3);
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(distance("flaw", "lawn"), 2);
        assert_eq!(distance("abc", "cab"), 2);
        assert_eq!(distance("abcdef", "abdef"), 1);
        assert_eq!(distance("abdef", "abcdef"), 1);
        // Characters, not bytes.
        assert_eq!(distance("né", "ne"), 1);
    }

    #[test]
    fn a_key_names_its_scene() {
        assert_eq!(scene_of_key("ch1_gate_0123abcd"), Some("ch1_gate"));
        assert_eq!(scene_of_key("ch1_gate_0123abcd_12"), Some("ch1_gate"));
        assert_eq!(scene_of_key("caption.ch1_gate_0123abcd"), Some("ch1_gate"));
        assert_eq!(scene_of_key("ch1_gate"), None);
        assert_eq!(scene_of_key("caption.ch1_gate"), None);
        assert!(is_caption("caption.a_0123abcd"));
        assert!(!is_caption("a_0123abcd"));
    }

    #[test]
    fn only_text_boxes_and_replies_have_a_limit() {
        assert_eq!(limit(LineKind::Speech), Some(MAX_TEXT_LEN));
        assert_eq!(limit(LineKind::Reply), Some(MAX_OPTION_LEN));
        assert_eq!(limit(LineKind::Caption), None);
    }
}
