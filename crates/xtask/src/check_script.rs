//! `cargo xtask check-script [file]`: the checks a dialogue script goes
//! through (`assets/dialogue/README.md`), without building the tests, for
//! whoever writes scripts (ticket 0723, `docs/story/writers-guide.md`).
//! Reads the scripts as they are on disk; prints one `file:line: message`
//! per problem, then a count. No rule lives here: every message is
//! `trpg_content`'s.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use trpg_content::ContentError;
use trpg_content::dialogue::{DIALOGUE_DIR, DIALOGUE_EXTENSION};

pub const USAGE: &str = "usage: cargo xtask check-script [file]\n\n\
checks the dialogue scripts (assets/dialogue/*.dlg) and prints every problem as\n  \
<file>:<line>: <what is wrong, and the fix where there is one>\n\
then how many there are. exits 0 when there are none.\n\
with a file, only that file's problems are listed: a script in assets/dialogue/,\n\
a draft anywhere else (checked together with the game's scripts), or a Markdown\n\
file, whose ```dlg blocks are checked.";

/// The fence that opens a script example in a Markdown file.
const DLG_FENCE: &str = "```dlg";
/// The fence that closes one.
const FENCE: &str = "```";

/// One script: the name its problems are reported under, and its text.
pub type Script = (String, String);

/// `markdown` with only the lines of its ` ```dlg ` blocks kept, every
/// other line blank, so a problem's line number is the Markdown file's.
pub fn dlg_blocks(markdown: &str) -> String {
    let mut inside = false;
    let mut out = String::new();
    for line in markdown.lines() {
        let fence = line.trim_end();
        if inside && fence == FENCE {
            inside = false;
        } else if inside {
            out.push_str(line);
        } else if fence == DLG_FENCE {
            inside = true;
        }
        out.push('\n');
    }
    out
}

/// Forward slashes, as problems name files.
fn shown(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// `path` read as a script: a Markdown file's ` ```dlg ` blocks, any other
/// file whole.
fn read_script(path: &Path) -> Result<String, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("can't read {}: {e}", shown(path)))?;
    let markdown = path.extension().is_some_and(|e| e == "md");
    Ok(if markdown { dlg_blocks(&text) } else { text })
}

/// Whether `a` and `b` are the same file.
fn same_file(a: &Path, b: &Path) -> bool {
    matches!((a.canonicalize(), b.canonicalize()), (Ok(a), Ok(b)) if a == b)
}

/// The scripts to check: every `.dlg` file of `<root>/assets/dialogue/`,
/// by name, then `extra` if it isn't one of them. Returns them with the
/// name `extra` is reported under.
pub fn scripts(root: &Path, extra: Option<&Path>) -> Result<(Vec<Script>, Option<String>), String> {
    let dir = root.join("assets").join(DIALOGUE_DIR);
    let entries = fs::read_dir(&dir).map_err(|e| format!("can't read {}: {e}", shown(&dir)))?;
    let mut paths: Vec<_> = entries.filter_map(|e| Some(e.ok()?.path())).collect();
    paths.retain(|p| shown(p).ends_with(DIALOGUE_EXTENSION));
    paths.sort();
    let mut only = None;
    let mut out = Vec::new();
    for path in &paths {
        let file = path.file_name().unwrap_or_default().to_string_lossy();
        let name = format!("assets/{DIALOGUE_DIR}/{file}");
        if extra.is_some_and(|e| same_file(e, path)) {
            only = Some(name.clone());
        }
        out.push((name, read_script(path)?));
    }
    if let (Some(extra), None) = (extra, &only) {
        out.push((shown(extra), read_script(extra)?));
        only = Some(shown(extra));
    }
    Ok((out, only))
}

/// Every problem `trpg_content` finds with `scripts` as the game's
/// dialogue files.
pub fn problems(scripts: &[Script]) -> Vec<ContentError> {
    let files: Vec<(&str, &str)> = scripts
        .iter()
        .map(|(name, text)| (name.as_str(), text.as_str()))
        .collect();
    match trpg_content::load_with_scripts(&files) {
        Ok(_) => Vec::new(),
        Err(errors) => errors.0,
    }
}

/// `1 problem`, `2 problems`.
fn count(n: usize) -> String {
    format!("{n} problem{}", if n == 1 { "" } else { "s" })
}

/// The problems to list: all of them, or with `only` that file's.
fn listed<'a>(problems: &'a [ContentError], only: Option<&str>) -> Vec<&'a ContentError> {
    problems
        .iter()
        .filter(|p| only.is_none_or(|file| p.file == file))
        .collect()
}

/// Whether [`report`] lists no problem.
pub fn clean(problems: &[ContentError], only: Option<&str>) -> bool {
    listed(problems, only).is_empty()
}

/// What to print for `problems`: one line each, then the count. With
/// `only`, the problems of that file alone, and a last line if other
/// files have some. `checked` is how many scripts were read.
pub fn report(problems: &[ContentError], only: Option<&str>, checked: usize) -> String {
    let listed = listed(problems, only);
    let mut out = String::new();
    // Writing to a `String` can't fail.
    for problem in &listed {
        let _ = writeln!(out, "{problem}");
    }
    let n = count(listed.len());
    let others = problems.len() - listed.len();
    if let Some(file) = only {
        let _ = writeln!(out, "{n} in {file}");
        if others > 0 {
            let _ = writeln!(
                out,
                "({} in other files: run `cargo xtask check-script` to list them)",
                count(others)
            );
        }
    } else {
        let scripts = if checked == 1 { "script" } else { "scripts" };
        let _ = writeln!(out, "{n} in {checked} {scripts}");
    }
    out
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// A draft with a curly quote and a name written out.
    const DRAFT: &str = "@scene draft_723\n@left lead neutral\n\
                         > It\u{2019}s raining.\n> Emeric is late.\n@end\n";

    /// [`DRAFT`] as a file outside `assets/dialogue/`, deleted on drop.
    struct Draft(PathBuf);

    impl Draft {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("{}_{name}", std::process::id()));
            fs::write(&path, DRAFT).unwrap();
            Self(path)
        }
    }

    impl Drop for Draft {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    #[test]
    fn the_repository_scripts_have_no_problems() {
        let (scripts, only) = scripts(&root(), None).unwrap();
        assert_eq!(only, None);
        let names: Vec<&str> = scripts.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"assets/dialogue/ch01.dlg"), "{names:?}");
        assert!(names.is_sorted());
        let dlg = |n: &&str| n.ends_with(DIALOGUE_EXTENSION);
        assert!(names.iter().all(dlg), "{names:?}");
        let problems = problems(&scripts);
        assert_eq!(problems, []);
        assert!(clean(&problems, None));
        let expected = format!("0 problems in {} scripts\n", scripts.len());
        assert_eq!(report(&problems, None, scripts.len()), expected);
    }

    #[test]
    fn a_curly_quote_and_a_written_out_name_are_reported_with_their_fixes() {
        let draft = Draft::new("draft.dlg");
        let (scripts, only) = scripts(&root(), Some(&draft.0)).unwrap();
        let name = shown(&draft.0);
        assert_eq!(only.as_deref(), Some(name.as_str()));
        assert_eq!(scripts.last().map(|s| s.0.as_str()), Some(name.as_str()));
        let problems = problems(&scripts);
        assert!(!clean(&problems, only.as_deref()));
        assert_eq!(
            report(&problems, only.as_deref(), scripts.len()),
            format!(
                "{name}:3:5: non-ASCII punctuation '\u{2019}'; use ' instead\n\
                 {name}:4: \"Emeric\" is written out; write {{n:king}} (\"Emeric\") so a \
                 rename reaches this line\n\
                 2 problems in {name}\n"
            )
        );
    }

    #[test]
    fn a_script_of_the_game_is_checked_once_under_its_own_name() {
        let path = root().join("assets/dialogue/ch01.dlg");
        let (with, only) = scripts(&root(), Some(&path)).unwrap();
        let (without, _) = scripts(&root(), None).unwrap();
        assert_eq!(only.as_deref(), Some("assets/dialogue/ch01.dlg"));
        assert_eq!(with, without);
    }

    #[test]
    fn a_file_that_cannot_be_read_is_an_error() {
        let missing = root().join("no_such_script.dlg");
        let error = scripts(&root(), Some(&missing)).unwrap_err();
        assert!(error.starts_with("can't read "), "{error}");
        assert!(error.contains("no_such_script.dlg"), "{error}");
        let error = scripts(&root().join("no_such_dir"), None).unwrap_err();
        assert!(error.starts_with("can't read "), "{error}");
        assert!(error.contains("no_such_dir/assets/dialogue"), "{error}");
    }

    fn problem(file: &str, line: u32) -> ContentError {
        ContentError::new(file, "bad").at(line, None)
    }

    #[test]
    fn the_report_ends_with_a_count() {
        let problems = [problem("a.dlg", 2), problem("b.dlg", 7)];
        assert_eq!(
            report(&problems, None, 3),
            "a.dlg:2: bad\nb.dlg:7: bad\n2 problems in 3 scripts\n"
        );
        assert_eq!(
            report(&problems[..1], None, 1),
            "a.dlg:2: bad\n1 problem in 1 script\n"
        );
        assert!(!clean(&problems, None));
        assert!(clean(&[], None));
    }

    #[test]
    fn with_a_file_only_its_problems_are_listed() {
        let problems = [
            problem("a.dlg", 2),
            problem("b.dlg", 7),
            problem("b.dlg", 9),
        ];
        assert_eq!(
            report(&problems, Some("a.dlg"), 3),
            "a.dlg:2: bad\n1 problem in a.dlg\n\
             (2 problems in other files: run `cargo xtask check-script` to list them)\n"
        );
        assert_eq!(
            report(&problems[1..2], Some("a.dlg"), 3),
            "0 problems in a.dlg\n\
             (1 problem in other files: run `cargo xtask check-script` to list them)\n"
        );
        assert_eq!(
            report(&problems[..1], Some("a.dlg"), 3),
            "a.dlg:2: bad\n1 problem in a.dlg\n"
        );
        assert!(clean(&problems[1..], Some("a.dlg")));
        assert!(!clean(&problems, Some("a.dlg")));
    }

    #[test]
    fn only_the_dlg_blocks_of_a_markdown_file_are_kept_on_their_lines() {
        let markdown = "# Guide\n```dlg\n@scene a\n  > Hi.\n```\ntext\n```\n@scene no\n```\n\
                        ```dlg  \n@end\n```\n";
        assert_eq!(
            dlg_blocks(markdown),
            "\n\n@scene a\n  > Hi.\n\n\n\n\n\n\n@end\n\n"
        );
    }

    #[test]
    fn a_markdown_draft_is_read_as_its_dlg_blocks() {
        let draft = Draft::new("draft.md");
        fs::write(&draft.0, format!("{DRAFT}```dlg\n> Hi.\n```\n")).unwrap();
        assert_eq!(read_script(&draft.0).unwrap(), "\n\n\n\n\n\n> Hi.\n\n");
    }

    /// Every example of the writer's guide passes the checker (0723).
    #[test]
    fn the_writers_guide_examples_have_no_problems() {
        let guide = root().join("docs/story/writers-guide.md");
        let (scripts, only) = scripts(&root(), Some(&guide)).unwrap();
        let text = &scripts.last().unwrap().1;
        let scenes = text.lines().filter(|l| l.starts_with("@scene ")).count();
        assert!(scenes >= 8, "the guide has {scenes} example scenes");
        let problems = problems(&scripts);
        assert_eq!(
            report(&problems, only.as_deref(), scripts.len()),
            format!("0 problems in {}\n", shown(&guide))
        );
    }
}
