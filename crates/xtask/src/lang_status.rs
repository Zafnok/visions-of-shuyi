//! `cargo xtask lang-status <code>`: what a language pack lacks (ADR-0045):
//! the **missing** keys (no entry), the **stale** ones (an entry made
//! from English that has since changed) and the **orphaned** dialogue
//! entries (their line was reworded or cut, so it has a new id or none),
//! each with the untranslated line of its scene nearest to what it
//! translated. All show in English in the game and fail no gate; this
//! lists them for whoever translates. Screen text comes first, then the
//! data's, then the dialogue's, each with its counts. Test text (the
//! `test*.dlg` scenes, data ids starting with `test`) is left out, and
//! counted, except in the test pack's report; `--only` and `--file`
//! narrow the report (ticket 0244).

use std::fmt::Write as _;

use trpg_content::{Lang, LangCode, LineText, MadeBy, Orphan};

/// The three parts of a pack's text, in the order the report gives them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// `ui.ron`: the screens' text.
    Screen,
    /// `data.ron`: names, tips, titles and notes.
    Data,
    /// `dialogue/`: lines and captions.
    Dialogue,
}

impl Part {
    const ALL: [Self; 3] = [Self::Screen, Self::Data, Self::Dialogue];

    fn heading(self) -> &'static str {
        match self {
            Self::Screen => "screen text",
            Self::Data => "data",
            Self::Dialogue => "dialogue",
        }
    }
}

/// What to narrow the report to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    /// Only this part (`--only ui|data|dialogue`).
    pub only: Option<Part>,
    /// Only the lines and captions of this `.dlg` file, by stem
    /// (`--file ch01`); this is dialogue only.
    pub file: Option<String>,
}

/// The pack code and options of `lang-status`'s arguments.
pub fn parse_args(args: &[String]) -> Result<(String, Options), String> {
    let mut code = None;
    let mut options = Options::default();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--only" => {
                options.only = Some(match args.next().map(String::as_str) {
                    Some("ui") => Part::Screen,
                    Some("data") => Part::Data,
                    Some("dialogue") => Part::Dialogue,
                    _ => return Err("--only takes ui, data or dialogue".to_owned()),
                });
            }
            "--file" => match args.next() {
                Some(stem) => options.file = Some(stem.trim_end_matches(".dlg").to_owned()),
                None => return Err("--file takes a .dlg file name, like ch01".to_owned()),
            },
            flag if flag.starts_with("--") => return Err(format!("unknown option {flag}")),
            _ if code.is_none() => code = Some(arg.clone()),
            _ => return Err("only one language code".to_owned()),
        }
    }
    if options.file.is_some() && options.only.is_some_and(|part| part != Part::Dialogue) {
        return Err("--file is dialogue; it can't go with --only ui or data".to_owned());
    }
    let code = code.ok_or("no language code")?;
    Ok((code, options))
}

/// What the report counts and prints for one pack.
struct Scope<'a> {
    lang: &'a Lang,
    options: &'a Options,
    /// Whether test text counts (it does for the test pack only).
    tests: bool,
    /// Missing and stale test texts left out so far.
    left_out: usize,
}

impl Scope<'_> {
    fn part_of(&self, key: &str) -> Part {
        if self.lang.has(key) {
            Part::Screen
        } else if self.lang.source().lines.contains_key(key) {
            Part::Dialogue
        } else {
            Part::Data
        }
    }

    fn shows(&self, part: Part) -> bool {
        let file = self.options.file.as_ref().map(|_| Part::Dialogue);
        self.options
            .only
            .or(file)
            .is_none_or(|wanted| wanted == part)
    }

    /// Whether the report lists `key`; counts it when it is left out as
    /// test text.
    fn keeps(&mut self, key: &str) -> bool {
        let part = self.part_of(key);
        if !self.shows(part) {
            return false;
        }
        let line = self.lang.source().lines.get(key);
        if let (Some(stem), Some(line)) = (&self.options.file, line)
            && line.file != *stem
        {
            return false;
        }
        // Test text is the `test*.dlg` scenes' and the data whose id
        // starts with `test` (`names.test_knight`, `chapters.test.title`).
        let test = match (part, line) {
            (Part::Dialogue, Some(line)) => line.file.starts_with("test"),
            (Part::Data, _) => key
                .split('.')
                .nth(1)
                .is_some_and(|id| id.starts_with("test")),
            _ => false,
        };
        if test && !self.tests {
            self.left_out += 1;
            return false;
        }
        true
    }

    fn keeps_orphan(&self, orphan: &Orphan) -> bool {
        if !self.shows(Part::Dialogue) {
            return false;
        }
        match &self.options.file {
            None => true,
            Some(stem) => orphan
                .nearest
                .as_ref()
                .and_then(|key| self.lang.source().lines.get(key))
                .is_some_and(|line| line.file == *stem),
        }
    }
}

/// The report for the pack `code` of `lang`, or why there is none.
pub fn report(lang: &Lang, code: &str, options: &Options) -> Result<String, String> {
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
    let mut scope = Scope {
        lang,
        options,
        tests: code.as_str() == "test",
        left_out: 0,
    };
    let missing: Vec<&String> = status.missing.iter().filter(|k| scope.keeps(k)).collect();
    let stale: Vec<&String> = status.stale.iter().filter(|k| scope.keeps(k)).collect();
    let orphans: Vec<&Orphan> = status
        .orphans
        .iter()
        .filter(|o| scope.keeps_orphan(o))
        .collect();
    let mut out = format!(
        "lang-status {code} ({}, {made_by}): {} missing, {} stale, {} orphaned",
        pack.info.name,
        missing.len(),
        stale.len(),
        orphans.len()
    );
    if scope.left_out > 0 {
        // Writing to a `String` can't fail.
        let _ = write!(out, "; {} test texts left out", scope.left_out);
    }
    out.push('\n');
    for part in Part::ALL.into_iter().filter(|&part| scope.shows(part)) {
        let of_part = |keys: &[&String]| -> Vec<String> {
            keys.iter()
                .filter(|k| scope.part_of(k) == part)
                .map(|k| (*k).clone())
                .collect()
        };
        let (missing, stale) = (of_part(&missing), of_part(&stale));
        let orphans: &[&Orphan] = if part == Part::Dialogue {
            &orphans
        } else {
            &[]
        };
        let _ = write!(
            out,
            "{}: {} missing, {} stale",
            part.heading(),
            missing.len(),
            stale.len()
        );
        if part == Part::Dialogue {
            let _ = write!(out, ", {} orphaned", orphans.len());
        }
        out.push('\n');
        if !missing.is_empty() {
            out.push_str("missing:\n");
        }
        for key in &missing {
            let english = lang.english(key).unwrap_or_default();
            let _ = writeln!(out, "  {key}\n    english: {english:?}");
        }
        if !stale.is_empty() {
            out.push_str("stale:\n");
        }
        for key in &stale {
            let entry = pack.entry(key).map(|e| &e.source);
            let was = entry.or_else(|| pack.line(key).map(|e| &e.source));
            let was = was.map_or("", String::as_str);
            let now = lang.english(key).unwrap_or_default();
            let _ = writeln!(out, "  {key}\n    was: {was:?}\n    now: {now:?}");
        }
        if !orphans.is_empty() {
            out.push_str("orphaned:\n");
        }
        for orphan in orphans {
            write_orphan(&mut out, lang, orphan);
        }
    }
    Ok(out)
}

fn write_orphan(out: &mut String, lang: &Lang, orphan: &Orphan) {
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
            report(&lang, "xx", &Options::default()).unwrap(),
            "lang-status xx (Xx, machine-made): 1 missing, 1 stale, 0 orphaned\n\
             screen text: 1 missing, 1 stale\n\
             missing:\n  a.three\n    english: \"Three\"\n\
             stale:\n  a.two\n    was: \"Too\"\n    now: \"Two\"\n\
             data: 0 missing, 0 stale\n\
             dialogue: 0 missing, 0 stale, 0 orphaned\n"
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
            report(&lang, "xx", &Options::default()).unwrap(),
            "lang-status xx (Xx, by A. Translator): 0 missing, 0 stale, 0 orphaned\n\
             screen text: 0 missing, 0 stale\n\
             data: 0 missing, 0 stale\n\
             dialogue: 0 missing, 0 stale, 0 orphaned\n"
        );
    }

    #[test]
    fn an_unknown_language_names_the_packs() {
        let lang = lang(r#"(name: "Xx", made_by: Machine)"#, "[]");
        assert_eq!(
            report(&lang, "zz", &Options::default()).unwrap_err(),
            "no language pack \"zz\"; the packs are: xx"
        );
        // English is the source, not a pack.
        assert!(report(&lang, "en", &Options::default()).is_err());
        assert_eq!(
            report(&lang, "Not A Code", &Options::default()).unwrap_err(),
            "no language \"Not A Code\"; the packs are: xx"
        );
    }

    /// The test pack keeps one of each on purpose (tickets 0233, 0235),
    /// and translates only a little of the data and one scene.
    #[test]
    fn the_test_pack_has_stale_missing_and_orphaned_text() {
        let content = trpg_content::load_embedded().unwrap();
        let report = report(&content.lang, "test", &Options::default()).unwrap();
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
        assert!(report.contains("stale:\n  title.continue\n"), "{report}");
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
            report(&lang(before.clone()), "xx", &Options::default()).unwrap(),
            "lang-status xx (Xx, machine-made): 0 missing, 0 stale, 0 orphaned\n\
             screen text: 0 missing, 0 stale\n\
             data: 0 missing, 0 stale\n\
             dialogue: 0 missing, 0 stale, 0 orphaned\n"
        );
        // English rewords a line of `gate` and cuts the scene `road`.
        let after = lines_of("@scene gate\n> The gate is barred.\n> Nobody answers.\n@end\n");
        let new_id = id_of(&after, "The gate is barred.");
        assert_eq!(
            report(&lang(after.clone()), "xx", &Options::default()).unwrap(),
            format!(
                "lang-status xx (Xx, machine-made): 1 missing, 0 stale, 2 orphaned\n\
                 screen text: 0 missing, 0 stale\n\
                 data: 0 missing, 0 stale\n\
                 dialogue: 1 missing, 0 stale, 2 orphaned\n\
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
        // `--file` keeps an orphan only if its paired line is in that file.
        let in_file = |stem: &str| Options {
            file: Some(stem.to_owned()),
            ..Options::default()
        };
        let report = |stem| report(&lang(after.clone()), "xx", &in_file(stem)).unwrap();
        assert!(report("t").starts_with(
            "lang-status xx (Xx, machine-made): 1 missing, 0 stale, 1 orphaned
"
        ));
        assert!(report("t").contains(&format!(
            "  {}
    was: \"The gate",
            id_of(&before, "The gate is shut.")
        )));
        assert!(report("other").contains(
            "0 missing, 0 stale, 0 orphaned
"
        ));
    }

    /// English with two `.dlg` files (a real one and a test one) and some
    /// data, and a pack that translates all of it but the test text.
    fn two_files(pack_ui: &str) -> Lang {
        use trpg_content::lang::{LangFile, from_files};

        let table = trpg_content::dialogue::from_sources(
            &[
                (
                    "assets/dialogue/ch01.dlg",
                    "@scene gate\n> The gate is shut.\n@end\n",
                ),
                ("assets/dialogue/test.dlg", "@scene t\n> Test line.\n@end\n"),
            ],
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let mut data = trpg_content::lang::DataText::default();
        data.add_dialogue(&table);
        for (key, text) in [
            ("names.king", "the King"),
            ("names.test_knight", "Test Knight"),
        ] {
            data.data.insert(key.to_owned(), text.to_owned());
        }
        let gate = id_of(&data, "The gate is shut.");
        let dialogue =
            format!(r#"[(key: {gate:?}, source: "The gate is shut.", text: "LA PORTE")]"#);
        let files = [
            ("en/ui.ron", r#"{"a.one": "One"}"#),
            ("xx/lang.ron", r#"(name: "Xx", made_by: Machine)"#),
            ("xx/ui.ron", pack_ui),
            (
                "xx/data.ron",
                r#"[(key: "names.king", source: "the King", text: "le roi")]"#,
            ),
            ("xx/dialogue/ch01.ron", dialogue.as_str()),
        ]
        .map(|(path, source)| LangFile {
            path,
            source: Some(source),
        });
        from_files(&files, Some(data), str::to_owned).unwrap()
    }

    const ONE: &str = r#"[(key: "a.one", source: "One", text: "Un")]"#;

    #[test]
    fn a_pack_that_translates_all_but_the_test_text_lacks_nothing() {
        let lang = two_files(ONE);
        assert_eq!(
            report(&lang, "xx", &Options::default()).unwrap(),
            "lang-status xx (Xx, machine-made): 0 missing, 0 stale, 0 orphaned; \
             2 test texts left out\n\
             screen text: 0 missing, 0 stale\n\
             data: 0 missing, 0 stale\n\
             dialogue: 0 missing, 0 stale, 0 orphaned\n"
        );
    }

    #[test]
    fn only_narrows_the_report_to_a_part() {
        let lang = two_files("[]");
        let only = |part| Options {
            only: Some(part),
            file: None,
        };
        let ui = report(&lang, "xx", &only(Part::Screen)).unwrap();
        assert_eq!(
            ui,
            "lang-status xx (Xx, machine-made): 1 missing, 0 stale, 0 orphaned\n\
             screen text: 1 missing, 0 stale\nmissing:\n  a.one\n    english: \"One\"\n"
        );
        let data = report(&lang, "xx", &only(Part::Data)).unwrap();
        assert!(data.contains("\ndata: 0 missing, 0 stale\n"), "{data}");
        assert!(!data.contains("screen text"), "{data}");
        assert!(!data.contains("a.one"), "{data}");
        let dialogue = report(&lang, "xx", &only(Part::Dialogue)).unwrap();
        assert!(
            dialogue.ends_with("dialogue: 0 missing, 0 stale, 0 orphaned\n"),
            "{dialogue}"
        );
        assert!(!dialogue.contains("a.one"), "{dialogue}");
    }

    #[test]
    fn the_test_pack_keeps_the_test_text() {
        let content = trpg_content::load_embedded().unwrap();
        let options = Options {
            only: Some(Part::Dialogue),
            ..Options::default()
        };
        let report = report(&content.lang, "test", &options).unwrap();
        assert!(!report.contains("left out"), "{report}");
        assert!(report.contains("  test_36a692b7\n    english:"), "{report}");
    }

    #[test]
    fn file_lists_only_that_dlg_files_lines() {
        let lang = two_files("[]");
        let file = |stem: &str| Options {
            file: Some(stem.to_owned()),
            ..Options::default()
        };
        // `ch01` is fully translated; `test` is left out for this pack.
        assert_eq!(
            report(&lang, "xx", &file("ch01")).unwrap(),
            "lang-status xx (Xx, machine-made): 0 missing, 0 stale, 0 orphaned\n\
             dialogue: 0 missing, 0 stale, 0 orphaned\n"
        );
        // The test pack lists the test scene, and only the file asked for.
        let content = trpg_content::load_embedded().unwrap();
        let test = report(&content.lang, "test", &file("test")).unwrap();
        assert!(test.contains("  test_36a692b7\n    english:"), "{test}");
        assert!(!test.contains("screen text"), "{test}");
        let lines = &content.lang.source().lines;
        for (key, line) in lines.iter().filter(|(_, line)| line.file != "test") {
            assert!(
                !test.contains(&format!("  {key}\n    english:")),
                "{key} is from {}",
                line.file
            );
        }
    }

    #[test]
    fn arguments_are_parsed() {
        let args = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let (code, options) = parse_args(&args(&["xx", "--only", "data"])).unwrap();
        assert_eq!((code.as_str(), options.only), ("xx", Some(Part::Data)));
        let (_, options) = parse_args(&args(&["--file", "ch01.dlg", "xx"])).unwrap();
        assert_eq!(options.file.as_deref(), Some("ch01"));
        assert!(parse_args(&args(&[])).is_err());
        assert!(parse_args(&args(&["xx", "yy"])).is_err());
        assert!(parse_args(&args(&["xx", "--only", "nope"])).is_err());
        assert!(parse_args(&args(&["xx", "--only"])).is_err());
        assert!(parse_args(&args(&["xx", "--file"])).is_err());
        assert!(parse_args(&args(&["xx", "--bogus"])).is_err());
        assert_eq!(
            parse_args(&args(&["--bogus"])).unwrap_err(),
            "unknown option --bogus"
        );
        let (_, options) = parse_args(&args(&["xx", "--only", "ui"])).unwrap();
        assert_eq!(options.only, Some(Part::Screen));
        assert!(parse_args(&args(&["xx", "--only", "ui", "--file", "a"])).is_err());
        assert!(parse_args(&args(&["xx", "--only", "dialogue", "--file", "a"])).is_ok());
    }
}
