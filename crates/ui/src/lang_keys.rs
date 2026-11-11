//! The key placeholders of every language pack's screen text (ticket 0253):
//! a capitalized `{Name}` names a key, so it must be `{Cursor}` or one of
//! the game's [`Action`]s, checked by the engine's
//! [`check_key_placeholders`]. A misspelled one would reach the player as
//! braces.

use crate::prelude::*;
use alloc::collections::BTreeMap;
use shuyi_assets::bundle::Bundle;
use shuyi_assets::lang::{Entry, english_from_source};
use shuyi_ui::text::check_key_placeholders;
use trpg_content::keymap::Action;
use trpg_content::lang::LANG_DIR;
use trpg_content::{ContentError, LangCode};

/// Checks every `lang/<code>/ui.ron` in `bundle`: English's map of key to
/// text, and each pack's entries by their translated text. A file that
/// doesn't parse is left to the language loader, which reports it.
///
/// # Errors
///
/// Every placeholder that names no key, each naming its file and line
/// (`SH2008`).
pub fn check_ui_text_keys(bundle: &Bundle) -> Result<(), Vec<ContentError>> {
    let english = LangCode::default().to_string();
    let mut errors = Vec::new();
    for path in bundle.files_under(LANG_DIR) {
        if !path.ends_with("/ui.ron") {
            continue;
        }
        let Some(source) = bundle.file(path) else {
            continue;
        };
        let code = path.split('/').nth(1);
        let text: Option<BTreeMap<String, String>> = if code == Some(english.as_str()) {
            english_from_source(path, source).ok()
        } else {
            ron::from_str::<Vec<Entry>>(source)
                .ok()
                .map(|entries| entries.into_iter().map(|e| (e.key, e.text)).collect())
        };
        if let Some(Err(found)) =
            text.map(|text| check_key_placeholders::<Action>(bundle, path, &text))
        {
            errors.extend(found);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shuyi_assets::bundle::Layer;
    use shuyi_assets::error::codes;

    /// A bundle of `files` in a layer named `assets`.
    fn bundle(files: &'static [(&'static str, &'static [u8])]) -> Bundle {
        Bundle::new(vec![Layer::table("assets", files)])
    }

    #[test]
    fn the_cursor_every_action_and_a_value_pass() {
        let actions: Vec<String> = Action::ALL
            .iter()
            .map(|a| format!("{{{}}}", a.name()))
            .collect();
        let english = format!(
            "{{\n  \"a\": \"{} {{Cursor}} {{name}}\",\n}}",
            actions.join(" ")
        );
        let english: &'static str = Box::leak(english.into_boxed_str());
        let files: &'static [(&str, &[u8])] =
            Box::leak(Box::new([("lang/en/ui.ron", english.as_bytes())]));
        assert_eq!(check_ui_text_keys(&bundle(files)), Ok(()));
    }

    #[test]
    fn a_misspelled_action_fails_with_its_file_and_line_in_english_and_a_pack() {
        static FILES: &[(&str, &[u8])] = &[
            (
                "lang/en/ui.ron",
                b"{\n  \"a\": \"{Confirm} ok\",\n  \"b\": \"{Confirn} ok\",\n}",
            ),
            (
                "lang/xx/ui.ron",
                b"[\n  (key: \"a\", source: \"{Confirm} ok\", text: \"{Abort} OK\"),\n]",
            ),
        ];
        let errors = check_ui_text_keys(&bundle(FILES)).unwrap_err();
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(errors.iter().all(|e| e.code == Some(codes::UNKNOWN_NAME)));
        let messages: Vec<String> = errors.iter().map(ToString::to_string).collect();
        assert!(
            messages[0].starts_with("assets/lang/en/ui.ron:3: \"b\": unknown action \"{Confirn}\""),
            "{messages:?}"
        );
        assert!(
            messages[1].starts_with("assets/lang/xx/ui.ron:2: \"a\": unknown action \"{Abort}\""),
            "{messages:?}"
        );
    }

    /// Every embedded pack: the source English, and the `test` pack in a
    /// debug build.
    #[test]
    fn every_embedded_pack_names_only_known_keys() {
        let bundle = trpg_content::bundle::install();
        let packs = bundle
            .files_under(LANG_DIR)
            .into_iter()
            .filter(|p| p.ends_with("/ui.ron"))
            .count();
        assert!(packs >= 2, "{packs}");
        check_ui_text_keys(bundle).unwrap();
    }
}
