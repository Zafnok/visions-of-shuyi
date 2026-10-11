//! `cargo xtask lang-status <code>`: what a language pack lacks (ADR-0045):
//! the **missing** keys (no entry), the **stale** ones (an entry made
//! from English that has since changed) and the **orphaned** dialogue
//! entries (their line was reworded or cut, so it has a new id or none),
//! each with the untranslated line of its scene nearest to what it
//! translated. All show in English in the game and fail no gate; this
//! lists them for whoever translates. Screen text comes first, then the
//! data's, then the dialogue's.

use std::fmt::Write as _;

use trpg_content::{Lang, LangCode, LineText, MadeBy};

/// The report for the pack `code` of `lang`, or why there is none.
pub fn report(lang: &Lang, code: &str) -> Result<String, String> {
    let known = || {
        let codes: Vec<&str> = lang.codes().map(LangCode::as_str).collect();
        format!("the packs are: {}", codes.join(", "))
    };
    let code = LangCode::new(code).ok_or_else(|| format!("no language \"{code}\"; {}", known()))?;
    let (Some(pack), Some(status)) = (lang.pack(&code), lang.status(&code)) else {
        return Err(format!("no language pack \"{code}\"; {}", known()));
    };
    let made_by = match &pack.info.made_by {
        MadeBy::Machine => "machine-made".to_owned(),
        MadeBy::Human(credit) => format!("by {credit}"),
    };
    let mut out = format!(
        "lang-status {code} ({}, {made_by}): {} missing, {} stale, {} orphaned\n",
        pack.info.name,
        status.missing.len(),
        status.stale.len(),
        status.orphans.len()
    );
    if !status.missing.is_empty() {
        out.push_str("missing:\n");
    }
    for key in &status.missing {
        let english = lang.english(key).unwrap_or_default();
        // Writing to a `String` can't fail.
        let _ = writeln!(out, "  {key}\n    english: {english:?}");
    }
    if !status.stale.is_empty() {
        out.push_str("stale:\n");
    }
    for key in &status.stale {
        let entry = pack.entry(key).map(|e| &e.source);
        let was = entry.or_else(|| pack.line(key).map(|e| &e.source));
        let was = was.map_or("", String::as_str);
        let now = lang.english(key).unwrap_or_default();
        let _ = writeln!(out, "  {key}\n    was: {was:?}\n    now: {now:?}");
    }
    if !status.orphans.is_empty() {
        out.push_str("orphaned:\n");
    }
    for orphan in &status.orphans {
        let _ = writeln!(out, "  {}\n    was: {:?}", orphan.key, orphan.source);
        match &orphan.text {
            LineText::One(text) => {
                let _ = writeln!(out, "    text: {text:?}");
            }
            LineText::ByGender { male, female } => {
                let _ = writeln!(out, "    text_m: {male:?}\n    text_f: {female:?}");
            }
        }
        match &orphan.nearest {
            Some(key) => {
                let now = lang.english(key).unwrap_or_default();
                let _ = writeln!(out, "    nearest: {key}\n    now: {now:?}");
            }
            None => out.push_str("    nearest: no untranslated line in its scene\n"),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use trpg_content::lang::{english_from_source, pack_from_sources};

    use super::*;

    fn lang(info: &str, ui: &str) -> Lang {
        let english = r#"{"a.one": "One", "a.two": "Two", "a.three": "Three"}"#;
        let english = english_from_source("en/ui.ron", english).unwrap();
        let pack = pack_from_sources(("l", info), ("u", ui), &english).unwrap();
        let packs = BTreeMap::from([(LangCode::new("xx").unwrap(), pack)]);
        Lang::new(english, packs)
    }

    #[test]
    fn lists_missing_and_stale_keys() {
        let ui = r#"[
            (key: "a.one", source: "One", text: "Un"),
            (key: "a.two", source: "Too", text: "Deux"),
        ]"#;
        let lang = lang(r#"(name: "Xx", made_by: Machine)"#, ui);
        assert_eq!(
            report(&lang, "xx").unwrap(),
            "lang-status xx (Xx, machine-made): 1 missing, 1 stale, 0 orphaned\n\
             missing:\n  a.three\n    english: \"Three\"\n\
             stale:\n  a.two\n    was: \"Too\"\n    now: \"Two\"\n"
        );
    }

    #[test]
    fn a_complete_pack_is_one_line() {
        let ui = r#"[
            (key: "a.one", source: "One", text: "Un"),
            (key: "a.two", source: "Two", text: "Deux"),
            (key: "a.three", source: "Three", text: "Trois"),
        ]"#;
        let lang = lang(r#"(name: "Xx", made_by: Human("A. Translator"))"#, ui);
        assert_eq!(
            report(&lang, "xx").unwrap(),
            "lang-status xx (Xx, by A. Translator): 0 missing, 0 stale, 0 orphaned\n"
        );
    }

    #[test]
    fn an_unknown_language_names_the_packs() {
        let lang = lang(r#"(name: "Xx", made_by: Machine)"#, "[]");
        assert_eq!(
            report(&lang, "zz").unwrap_err(),
            "no language pack \"zz\"; the packs are: xx"
        );
        // English is the source, not a pack.
        assert!(report(&lang, "en").is_err());
        assert_eq!(
            report(&lang, "Not A Code").unwrap_err(),
            "no language \"Not A Code\"; the packs are: xx"
        );
    }

    /// The test pack keeps one of each on purpose (tickets 0233, 0235),
    /// and translates only a little of the data and one scene.
    #[test]
    fn the_test_pack_has_stale_missing_and_orphaned_text() {
        let content = trpg_content::load_embedded().unwrap();
        let report = report(&content.lang, "test").unwrap();
        let first = report.lines().next().unwrap_or_default();
        assert!(
            first.starts_with("lang-status test (TEST, machine-made): "),
            "{first}"
        );
        assert!(first.ends_with(" missing, 2 stale, 1 orphaned"), "{first}");
        assert!(report.contains("missing:\n  title.credits\n"), "{report}");
        // A data name and a dialogue line nobody translated.
        assert!(
            report.contains("  items.iron_spear.name\n    english: \"Iron Spear\"\n"),
            "{report}"
        );
        assert!(
            report.contains("  test_36a692b7\n    english: \"Let's move.\"\n"),
            "{report}"
        );
        assert!(report.contains("stale:\n  title.subtitle\n"), "{report}");
        let axe = "  items.iron_axe.name\n    was: \"Old Axe\"\n    now: \"Iron Axe\"\n";
        assert!(report.contains(axe), "{report}");
        // The orphan is paired with the reworded line.
        assert!(
            report.ends_with(
                "orphaned:\n  test_00000000\n    was: \"It is always about you.\"\n    \
                 text: \"IT IS ALWAYS ABOUT YOU.\"\n    nearest: test_92dc3bd5\n    \
                 now: \"It's always about you.\"\n"
            ),
            "{report}"
        );
    }

    /// The English of the lines of `dlg`, by line id.
    fn lines_of(dlg: &str) -> trpg_content::lang::DataText {
        let table = trpg_content::dialogue::from_sources(&[("t.dlg", dlg)], None, None, None, None)
            .unwrap();
        let mut data = trpg_content::lang::DataText::default();
        data.add_dialogue(&table);
        data
    }

    /// The id of the line of `data` that says `text`.
    fn id_of(data: &trpg_content::lang::DataText, text: &str) -> String {
        let line = data.lines.iter().find(|(_, line)| line.text == text);
        line.map(|(id, _)| id.clone()).unwrap()
    }

    /// Rewording an English line orphans its entry, which is paired with
    /// the new line; an orphan in a scene with nothing left to translate
    /// has no pair.
    #[test]
    fn a_reworded_line_is_an_orphan_paired_with_the_new_line() {
        use trpg_content::lang::{DataText, LangFile, from_files};

        let before = lines_of(
            "@scene gate\n> The gate is shut.\n> Nobody answers.\n@end\n\
             @scene road\n> Dust.\n@end\n",
        );
        // A pack that translates every line of that English.
        let entries: Vec<String> = before
            .lines
            .iter()
            .map(|(id, line)| {
                let (source, text) = (&line.text, line.text.to_uppercase());
                format!("(key: {id:?}, source: {source:?}, text: {text:?})")
            })
            .collect();
        let pack = format!("[{}]", entries.join(","));
        let lang = |data: DataText| {
            let files = [
                ("en/ui.ron", "{}"),
                ("xx/lang.ron", r#"(name: "Xx", made_by: Machine)"#),
                ("xx/ui.ron", "[]"),
                ("xx/dialogue/t.ron", pack.as_str()),
            ]
            .map(|(path, source)| LangFile {
                path,
                source: Some(source),
            });
            from_files(&files, Some(data), str::to_owned).unwrap()
        };
        assert_eq!(
            report(&lang(before.clone()), "xx").unwrap(),
            "lang-status xx (Xx, machine-made): 0 missing, 0 stale, 0 orphaned\n"
        );
        // English rewords a line of `gate` and cuts the scene `road`.
        let after = lines_of("@scene gate\n> The gate is barred.\n> Nobody answers.\n@end\n");
        let new_id = id_of(&after, "The gate is barred.");
        assert_eq!(
            report(&lang(after), "xx").unwrap(),
            format!(
                "lang-status xx (Xx, machine-made): 1 missing, 0 stale, 2 orphaned\n\
                 missing:\n  {new_id}\n    english: \"The gate is barred.\"\n\
                 orphaned:\n  {gate}\n    was: \"The gate is shut.\"\n    \
                 text: \"THE GATE IS SHUT.\"\n    \
                 nearest: {new_id}\n    now: \"The gate is barred.\"\n  \
                 {road}\n    was: \"Dust.\"\n    text: \"DUST.\"\n    \
                 nearest: no untranslated line in its scene\n",
                gate = id_of(&before, "The gate is shut."),
                road = id_of(&before, "Dust."),
            )
        );
    }
}
