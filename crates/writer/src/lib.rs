//! The checks a dialogue script goes through (`assets/dialogue/README.md`),
//! for whoever writes scripts: `cargo xtask check-script` in the game's
//! repository (ticket 0723) and `story-check` in the writer's mirror
//! (ticket 0152, ADR-0122). Reads the scripts as they are on disk; reports
//! one `file:line: message` per problem, then a count. No rule lives here:
//! every message is `trpg_content`'s.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use trpg_content::ContentError;
use trpg_content::dialogue::DIALOGUE_EXTENSION;

/// The mirror's folder of scripts, which `story-check` reads by default.
pub const DIALOGUE: &str = "dialogue";

/// `story-check`'s help.
pub const STORY_CHECK_USAGE: &str = "usage: story-check [folder or file]\n\n\
checks the dialogue scripts (every .dlg file in dialogue/, or in the folder\n\
named) and prints every problem as\n  \
<file>:<line>: <what is wrong, and the fix where there is one>\n\
then how many there are. exits 0 when there are none.\n\
with a file, only that file's problems are listed: a script in dialogue/, a\n\
draft anywhere else (checked together with dialogue/'s scripts), or a\n\
Markdown file, whose ```dlg blocks are checked.";

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
pub fn shown(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// `path` read as a script: a Markdown file's ` ```dlg ` blocks, any other
/// file whole.
pub fn read_script(path: &Path) -> Result<String, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("can't read {}: {e}", shown(path)))?;
    let markdown = path.extension().is_some_and(|e| e == "md");
    Ok(if markdown { dlg_blocks(&text) } else { text })
}

/// Whether `a` and `b` are the same file.
fn same_file(a: &Path, b: &Path) -> bool {
    matches!((a.canonicalize(), b.canonicalize()), (Ok(a), Ok(b)) if a == b)
}

/// The `.dlg` files directly in `dir`, each named `<dir>/<file>` as
/// problems show it, sorted by name.
pub fn scripts_in(dir: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("can't read {}: {e}", shown(dir)))?;
    let mut files: Vec<(String, PathBuf)> = entries
        .filter_map(|e| Some(e.ok()?.path()))
        .filter(|path| path.is_file() && shown(path).ends_with(DIALOGUE_EXTENSION))
        .map(|path| (shown(&path), path))
        .collect();
    files.sort();
    Ok(files)
}

/// `files` (each with its name, sorted) read as the game's scripts, then
/// `extra` if it isn't one of them. Returns them with the name `extra` is
/// reported under.
pub fn gather(
    files: &[(String, PathBuf)],
    extra: Option<&Path>,
) -> Result<(Vec<Script>, Option<String>), String> {
    let mut only = None;
    let mut out = Vec::new();
    for (name, path) in files {
        if extra.is_some_and(|e| same_file(e, path)) {
            only = Some(name.clone());
        }
        out.push((name.clone(), read_script(path)?));
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
/// files have some, which says to run `command` to list them. `checked`
/// is how many scripts were read.
pub fn report(
    problems: &[ContentError],
    only: Option<&str>,
    checked: usize,
    command: &str,
) -> String {
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
                "({} in other files: run `{command}` to list them)",
                count(others)
            );
        }
    } else {
        let scripts = if checked == 1 { "script" } else { "scripts" };
        let _ = writeln!(out, "{n} in {checked} {scripts}");
    }
    out
}

/// Checks `scripts` and returns the exit code and what to print: 0 when
/// [`report`] lists no problem, 1 when it does.
pub fn check(scripts: &[Script], only: Option<&str>, command: &str) -> (u8, String) {
    let problems = problems(scripts);
    let out = report(&problems, only, scripts.len(), command);
    (u8::from(!clean(&problems, only)), out)
}

/// `story-check` with `args`, `dialogue` being the folder it reads when
/// none is named: the exit code (0 no problem, 1 problems, 2 a bad command
/// line or a file it can't read), stdout and stderr.
pub fn story_check(args: &[String], dialogue: &Path) -> (u8, String, String) {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        return (0, format!("{STORY_CHECK_USAGE}\n"), String::new());
    }
    let (folder, extra) = match args {
        [] => (dialogue, None),
        [path] if Path::new(path).is_dir() => (Path::new(path), None),
        [path] => (dialogue, Some(Path::new(path))),
        _ => return (2, String::new(), format!("{STORY_CHECK_USAGE}\n")),
    };
    let gathered = scripts_in(folder).and_then(|files| gather(&files, extra));
    match gathered {
        Ok((scripts, only)) => {
            let (code, out) = check(&scripts, only.as_deref(), "story-check");
            (code, out, String::new())
        }
        Err(e) => (2, String::new(), format!("story-check: {e}\n")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A folder of its own under the system's temporary folder, deleted on
    /// drop.
    struct Folder(PathBuf);

    impl Folder {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("trpg-writer-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn problem(file: &str, line: u32) -> ContentError {
        ContentError::new(file, "bad").at(line, None)
    }

    #[test]
    fn the_report_ends_with_a_count() {
        let problems = [problem("a.dlg", 2), problem("b.dlg", 7)];
        assert_eq!(
            report(&problems, None, 3, "x"),
            "a.dlg:2: bad\nb.dlg:7: bad\n2 problems in 3 scripts\n"
        );
        assert_eq!(
            report(&problems[..1], None, 1, "x"),
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
        let command = "cargo xtask check-script";
        assert_eq!(
            report(&problems, Some("a.dlg"), 3, command),
            "a.dlg:2: bad\n1 problem in a.dlg\n\
             (2 problems in other files: run `cargo xtask check-script` to list them)\n"
        );
        assert_eq!(
            report(&problems[1..2], Some("a.dlg"), 3, "story-check"),
            "0 problems in a.dlg\n\
             (1 problem in other files: run `story-check` to list them)\n"
        );
        assert_eq!(
            report(&problems[..1], Some("a.dlg"), 3, command),
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
        let folder = Folder::new("markdown");
        let draft = folder.0.join("draft.md");
        fs::write(&draft, "@scene x\n> Hi.\n```dlg\n> Hi.\n```\n").unwrap();
        assert_eq!(read_script(&draft).unwrap(), "\n\n\n> Hi.\n\n");
    }

    /// Only the `.dlg` files directly in the folder, sorted, named by the
    /// folder as given; a draft in it is checked once, under that name.
    #[test]
    fn a_folder_s_scripts_are_its_dlg_files() {
        let folder = Folder::new("folder");
        for name in ["b.dlg", "a.dlg", "README.md"] {
            fs::write(folder.0.join(name), name).unwrap();
        }
        fs::create_dir(folder.0.join("sub.dlg")).unwrap();
        let files = scripts_in(&folder.0).unwrap();
        let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
        let dir = shown(&folder.0);
        assert_eq!(names, [format!("{dir}/a.dlg"), format!("{dir}/b.dlg")]);
        let (scripts, only) = gather(&files, Some(&folder.0.join("b.dlg"))).unwrap();
        assert_eq!(only, Some(format!("{dir}/b.dlg")));
        assert_eq!(scripts.len(), 2);
        assert_eq!(scripts[1].1, "b.dlg");
    }

    #[test]
    fn a_file_or_folder_that_cannot_be_read_is_an_error() {
        let folder = Folder::new("missing");
        let missing = folder.0.join("no_such_script.dlg");
        let error = gather(&[], Some(&missing)).unwrap_err();
        assert!(error.starts_with("can't read "), "{error}");
        assert!(error.contains("no_such_script.dlg"), "{error}");
        let error = scripts_in(&folder.0.join("no_such_dir")).unwrap_err();
        assert!(error.contains("no_such_dir"), "{error}");
    }

    #[test]
    fn story_check_reads_its_command_line() {
        let folder = Folder::new("args");
        let none = folder.0.join("none");
        let (code, out, _) = story_check(&["--help".to_owned()], &none);
        assert_eq!((code, out.starts_with("usage: story-check")), (0, true));
        let (code, _, err) = story_check(&["a".to_owned(), "b".to_owned()], &none);
        assert_eq!((code, err.starts_with("usage: story-check")), (2, true));
        let (code, _, err) = story_check(&[], &none);
        assert_eq!(code, 2);
        assert!(err.starts_with("story-check: can't read "), "{err}");
    }
}
