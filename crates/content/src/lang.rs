//! Languages (ADR-0045 §1–2): English screen text by key
//! (`assets/lang/en/ui.ron`), and the packs that overlay it
//! (`assets/lang/<code>/`): their screen text (`ui.ron`), the data's
//! names, tips, titles and notes (`data.ron`) and the dialogue's lines
//! and captions (`dialogue/*.ron`). `assets/lang/README.md` lists every
//! key. A pack entry carries the English it was made from; when that is
//! no longer today's English the entry is **stale**, and a key with no
//! entry is **missing**. Both show in English and fail nothing.
//!
//! The packs, their checks and the lookups by key are the engine's
//! (`shuyi_assets::lang`, ticket 1105). The game adds its folder, its data
//! text ([`DataText`]: the names of classes, items… and the lookups that
//! give them), the rules of its tips and lessons, the lead's tokens, and a
//! line's variants for each gender of the lead (`text_m`, `text_f`).
use crate::prelude::*;

mod data;

use alloc::collections::BTreeMap;
use core::ops::Deref;
use once_cell::race::OnceBox;

use shuyi_assets::lang::{self as engine, PackRules};
use trpg_core::lead::LeadGender;

use crate::bundle;
use crate::dialogue::{LIMITS, LeadTokens, Scene};
use crate::error::ContentError;

pub use data::{DataText, Tables, from_tables};
pub use shuyi_assets::lang::{
    ENGLISH, Entry, LangCode, LangFile, LangInfo, LangPack, LangStatus, LineEntry, LineKind,
    LineText, MadeBy, Orphan, PackSources, Source, SourceLine, english_from_source, text_width,
};

/// Directory of the languages inside the asset bundle.
pub const LANG_DIR: &str = "lang";

/// The code of the pack the tests use. It is in the bundle, but never
/// offered to players.
pub const TEST: &str = "test";

/// The directory's own description (`assets/lang/README.md`): no
/// language's file.
const README: &str = "README.md";

/// The variant of a pack's dialogue line said of a male lead (`text_m`).
pub const MALE: &str = "m";
/// The variant of a pack's dialogue line said of a female lead (`text_f`).
pub const FEMALE: &str = "f";

/// What the game's packs keep to besides English: the rules of tips and
/// lessons, the lead's tokens, the dialogue box, and a line's text for
/// each gender of the lead.
pub const RULES: PackRules<'static> = PackRules {
    data_problem: data::text_problem,
    tokens: &LeadTokens,
    limits: LIMITS,
    variants: &[MALE, FEMALE],
};

/// The variant of a pack's line said of a lead of `gender`.
pub const fn variant(gender: LeadGender) -> &'static str {
    match gender {
        LeadGender::Male => MALE,
        LeadGender::Female => FEMALE,
    }
}

/// English's text and every pack: the engine's, with the lookups of the
/// game's data text and of the lead's lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Lang(engine::Lang);

impl Deref for Lang {
    type Target = engine::Lang;

    fn deref(&self) -> &engine::Lang {
        &self.0
    }
}

impl From<engine::Lang> for Lang {
    fn from(lang: engine::Lang) -> Self {
        Self(lang)
    }
}

impl Lang {
    /// English screen text with `packs` over it, and no data or dialogue.
    pub fn new(english: BTreeMap<String, String>, packs: BTreeMap<LangCode, LangPack>) -> Self {
        Self(engine::Lang::new(english, packs))
    }

    /// English screen text and `source` (the data's and the dialogue's
    /// English), with `packs` over them.
    pub fn with_data(
        english: BTreeMap<String, String>,
        source: DataText,
        packs: BTreeMap<LangCode, LangPack>,
    ) -> Self {
        Self(engine::Lang::with_source(english, source, packs))
    }

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
        self.0.line(code, key, english, variant(gender))
    }

    /// `scene` in language `code`, for a lead of `gender`: every line,
    /// reply and caption the pack translates in its language, the others
    /// in English. The lines keep their ids (a voice clip is found by
    /// them) and their tokens.
    #[must_use]
    pub fn scene(&self, code: &LangCode, scene: &Scene, gender: LeadGender) -> Scene {
        self.0.scene(code, scene, variant(gender))
    }
}

/// English's screen text in the embedded bundle, by key: what
/// [`load`] gives every [`Lang`], for code that names things in English
/// with no [`Lang`] at hand. Empty if the file is broken (loading the
/// content reports that).
pub fn english_ui() -> &'static BTreeMap<String, String> {
    static ENGLISH_UI: OnceBox<BTreeMap<String, String>> = OnceBox::new();
    ENGLISH_UI.get_or_init(|| {
        let path = format!("{LANG_DIR}/{ENGLISH}/ui.ron");
        let english = bundle::file(&path).map(|source| english_from_source(&path, source));
        Box::new(english.and_then(Result::ok).unwrap_or_default())
    })
}

/// Parses and validates a pack with screen text only: its `lang.ron`
/// (`info`) and its `ui.ron` (`ui`), checked against `english`. See
/// [`pack_from_files`].
pub fn pack_from_sources(
    info: Source<'_>,
    ui: Source<'_>,
    english: &BTreeMap<String, String>,
) -> Result<LangPack, Vec<ContentError>> {
    engine::pack_from_sources(info, ui, english, &RULES)
}

/// Parses and validates the pack made of `sources`, its screen text
/// checked against `english` and its data and dialogue text against
/// `data` (left out without it: the data files failed to load), by the
/// game's [`RULES`]. Reports every problem found.
pub fn pack_from_files(
    sources: &PackSources<'_>,
    english: &BTreeMap<String, String>,
    data: Option<&DataText>,
) -> Result<LangPack, Vec<ContentError>> {
    engine::pack_from_files(sources, english, data, &RULES)
}

/// Loads and validates English and every pack in the embedded bundle.
/// The packs' data and dialogue text is checked against `data`, and left
/// out without it.
pub fn load(data: Option<DataText>) -> Result<Lang, Vec<ContentError>> {
    let prefix = format!("{LANG_DIR}/");
    let paths = bundle::files_under(LANG_DIR);
    let files: Vec<LangFile<'_>> = paths
        .iter()
        .map(|path| LangFile {
            path: path.strip_prefix(&prefix).unwrap_or(path),
            source: bundle::file(path),
        })
        .filter(|file| file.path != README)
        .collect();
    from_files(&files, data, |path| {
        bundle::display_path(&format!("{LANG_DIR}/{path}"))
    })
}

/// Validates the languages made of `files` by the game's [`RULES`]: see
/// [`engine::from_files`].
pub fn from_files(
    files: &[LangFile<'_>],
    data: Option<DataText>,
    display: impl Fn(&str) -> String,
) -> Result<Lang, Vec<ContentError>> {
    engine::from_files(files, data, display, &RULES).map(Lang)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(code: &str) -> LangCode {
        LangCode::new(code).unwrap()
    }

    /// Ticket 0243: English for code with no `Lang` at hand is the text
    /// every `Lang` is loaded with.
    #[test]
    fn the_bundles_english_is_what_the_languages_load() {
        let english = english_ui();
        assert_eq!(
            english.get("title.new_game").map(String::as_str),
            Some("New Game")
        );
        assert!(english.len() > 100, "{}", english.len());
        let lang = load(None).unwrap();
        for (key, text) in english {
            assert_eq!(lang.english(key), Some(text.as_str()), "{key}");
        }
        assert!(std::ptr::eq(english, english_ui()));
    }

    #[test]
    fn the_embedded_languages_load() {
        let lang = load(None).unwrap();
        assert_eq!(lang.english("title.new_game"), Some("New Game"));
        let test = code(TEST);
        assert_eq!(lang.codes().collect::<Vec<_>>(), [&test]);
        assert_eq!(lang.text(&test, "title.new_game"), "NEW GAME");
        // The test pack keeps one stale and one missing entry.
        let status = lang.status(&test).unwrap();
        assert_eq!(status.missing, ["title.credits"]);
        assert_eq!(status.stale, ["title.continue"]);
        assert_eq!(status.orphans, []);
    }

    #[test]
    fn each_gender_of_the_lead_has_its_variant() {
        assert_eq!(variant(LeadGender::Male), "m");
        assert_eq!(variant(LeadGender::Female), "f");
        assert_eq!(RULES.variants, ["m", "f"]);
    }
}

#[cfg(test)]
mod data_tests;
