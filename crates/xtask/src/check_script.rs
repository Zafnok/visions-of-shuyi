//! `cargo xtask check-script [file]`: the checks a dialogue script goes
//! through (`assets/dialogue/README.md`), without building the tests, for
//! whoever writes scripts (ticket 0723, `assets-private/story/writers-guide.md`).
//! Reads the scripts as they are on disk; prints one `file:line: message`
//! per problem, then a count. The checks and the report are
//! `trpg_writer`'s, which the writer's `story-check` runs too (ticket
//! 0152); this file finds the game's scripts.

use std::path::Path;

use trpg_content::bundle::TEST_DISPLAY_ROOT;
use trpg_content::dialogue::{DIALOGUE_DIR, DIALOGUE_EXTENSION};
use trpg_writer::Script;

use crate::content_layer;

pub const USAGE: &str = "usage: cargo xtask check-script [file]\n\n\
checks the dialogue scripts (assets/dialogue/*.dlg, with the content's in\n\
assets-private/content/dialogue/ over them, and the test scripts in\n\
assets-test/dialogue/) and prints every problem as\n  \
<file>:<line>: <what is wrong, and the fix where there is one>\n\
then how many there are. exits 0 when there are none.\n\
with a file, only that file's problems are listed: a script in assets/dialogue/,\n\
a draft anywhere else (checked together with the game's scripts), or a Markdown\n\
file, whose ```dlg blocks are checked.";

/// The scripts to check: every `.dlg` file of `<root>/assets/dialogue/`,
/// with the content's (`assets-private/content/dialogue/`, ADR-0117) over
/// it as the build lays them, and of `<root>/assets-test/dialogue/` (the
/// test files', ADR-0057: the content is checked with them, as in a test
/// build), by name; then `extra` if it isn't one of them. Returns them with
/// the name `extra` is reported under.
pub fn scripts(root: &Path, extra: Option<&Path>) -> Result<(Vec<Script>, Option<String>), String> {
    let (dir, ext) = (DIALOGUE_DIR, DIALOGUE_EXTENSION);
    let story = content_layer::files_in(root, &content_layer::STORY, dir, ext)?;
    let tests = content_layer::files_in(root, &[TEST_DISPLAY_ROOT], dir, ext)?;
    let mut files: Vec<_> = story.into_iter().chain(tests).collect();
    files.sort();
    trpg_writer::gather(&files, extra)
}

/// The command that lists every problem.
pub const COMMAND: &str = "cargo xtask check-script";

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use trpg_writer::{clean, problems, report, shown};

    use super::*;

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// A draft with a curly quote and a name written out.
    const DRAFT: &str = "@scene draft_723\n@left lead neutral\n\
                         > It\u{2019}s raining.\n> Ellery is late.\n@end\n";

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

    /// The game's own scripts, wherever the content is (ADR-0118: a test
    /// of the content).
    #[test]
    fn the_repository_scripts_have_no_problems() {
        if trpg_content::bundle::skip_without_the_game() {
            return;
        }
        let (scripts, only) = scripts(&root(), None).unwrap();
        assert_eq!(only, None);
        let names: Vec<&str> = scripts.iter().map(|(n, _)| n.as_str()).collect();
        let chapter_1 = |name: &&str| name.ends_with("/dialogue/ch01.dlg");
        assert!(names.iter().any(chapter_1), "{names:?}");
        assert!(
            names.contains(&"assets-test/dialogue/test.dlg"),
            "{names:?}"
        );
        assert!(names.is_sorted());
        let dlg = |n: &&str| n.ends_with(DIALOGUE_EXTENSION);
        assert!(names.iter().all(dlg), "{names:?}");
        let problems = problems(&scripts);
        assert_eq!(problems, []);
        assert!(clean(&problems, None));
        let expected = format!("0 problems in {} scripts\n", scripts.len());
        assert_eq!(report(&problems, None, scripts.len(), COMMAND), expected);
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
            report(&problems, only.as_deref(), scripts.len(), COMMAND),
            format!(
                "{name}:3:5: non-ASCII punctuation '\u{2019}'; use ' instead\n\
                 {name}:4: \"Ellery\" is the lead's default name; write {{lead}} for the \
                 name the player chose\n\
                 2 problems in {name}\n"
            )
        );
    }

    #[test]
    fn an_empty_direction_is_reported_and_a_direction_is_accepted() {
        let draft = Draft::new("direction.dlg");
        fs::write(
            &draft.0,
            "@scene draft_752\n@left lead neutral\nlead (): Go.\n@delivery\n\
             lead (quietly): Stay.\n> (slowly): It rains.\n@delivery low\n> Night falls.\n@end\n",
        )
        .unwrap();
        let (scripts, only) = scripts(&root(), Some(&draft.0)).unwrap();
        let name = shown(&draft.0);
        assert_eq!(
            report(&problems(&scripts), only.as_deref(), scripts.len(), COMMAND),
            format!(
                "{name}:3: a line's ( ) has no direction in it\n\
                 {name}:4: @delivery needs a direction, or \"clear\"\n\
                 2 problems in {name}\n"
            )
        );
    }

    /// The writer's `story-check` (ticket 0152), run on a folder holding
    /// the game's scripts as the mirror holds them, prints what
    /// `check-script` prints for the same draft.
    #[test]
    fn story_check_prints_what_check_script_prints() {
        let draft = Draft::new("same.dlg");
        let (scripts, only) = scripts(&root(), Some(&draft.0)).unwrap();
        let problems = problems(&scripts);
        let ours = report(&problems, only.as_deref(), scripts.len(), COMMAND);
        let code = u8::from(!clean(&problems, only.as_deref()));

        let mirror = std::env::temp_dir().join(format!("{}_mirror_dialogue", std::process::id()));
        let _ = fs::remove_dir_all(&mirror);
        fs::create_dir_all(&mirror).unwrap();
        let game = scripts.len() - 1;
        for (i, (_, text)) in scripts[..game].iter().enumerate() {
            fs::write(mirror.join(format!("{i:02}.dlg")), text).unwrap();
        }
        let arg = draft.0.to_string_lossy().into_owned();
        let theirs = trpg_writer::story_check(&[arg], &mirror);
        fs::remove_dir_all(&mirror).unwrap();
        assert!(ours.contains("non-ASCII punctuation"), "{ours}");
        assert_eq!(theirs, (code, ours, String::new()));
    }

    /// The content's scripts (ADR-0117) replace `assets/`'s of their name
    /// and are reported under their own; the test scripts are still read.
    #[test]
    fn the_contents_scripts_are_read_over_assets_scripts() {
        let repo = crate::content_layer::tests::repo(
            "check-script",
            &[
                ("assets/dialogue/ch01.dlg", "public"),
                ("assets/dialogue/ch02.dlg", "public two"),
                ("assets-test/dialogue/test.dlg", "test"),
                ("assets-private/content/dialogue/ch01.dlg", "private"),
            ],
        );
        let extra = repo.join("assets-private/content/dialogue/ch01.dlg");
        let (scripts, only) = scripts(&repo, Some(&extra)).unwrap();
        fs::remove_dir_all(&repo).unwrap();
        let content = "assets-private/content/dialogue/ch01.dlg";
        assert_eq!(only.as_deref(), Some(content));
        let read: Vec<(&str, &str)> = scripts
            .iter()
            .map(|(n, t)| (n.as_str(), t.as_str()))
            .collect();
        assert_eq!(
            read,
            [
                (content, "private"),
                ("assets-test/dialogue/test.dlg", "test"),
                ("assets/dialogue/ch02.dlg", "public two"),
            ]
        );
    }

    #[test]
    fn a_script_of_the_game_is_checked_once_under_its_own_name() {
        let path = root().join("assets-test/dialogue/test.dlg");
        let (with, only) = scripts(&root(), Some(&path)).unwrap();
        let (without, _) = scripts(&root(), None).unwrap();
        assert_eq!(only.as_deref(), Some("assets-test/dialogue/test.dlg"));
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

    /// Every example of the writer's guide passes the checker (0723). The
    /// guide is in the private repository's `story/` (ticket 0159).
    #[test]
    fn the_writers_guide_examples_have_no_problems() {
        // The guide's examples name the game's cast (a test of the content).
        if trpg_content::bundle::skip_without_the_game() {
            return;
        }
        let Some(guide) = trpg_content::bundle::story_doc("writers-guide.md") else {
            return;
        };
        let (scripts, only) = scripts(&root(), Some(&guide)).unwrap();
        let text = &scripts.last().unwrap().1;
        let scenes = text.lines().filter(|l| l.starts_with("@scene ")).count();
        assert!(scenes >= 8, "the guide has {scenes} example scenes");
        let problems = problems(&scripts);
        assert_eq!(
            report(&problems, only.as_deref(), scripts.len(), COMMAND),
            format!("0 problems in {}\n", shown(&guide))
        );
    }
}
