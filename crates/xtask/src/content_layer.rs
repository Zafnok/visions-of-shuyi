//! The game's files on disk as the build embeds them (ADR-0117): the
//! content from the private repository, in `assets-private/content/`, laid
//! over `assets/` when it is there. The tools that read the files from disk
//! read this view: `check-script`, `voice` and `check-keys` (and `lint`,
//! which runs it). `lines` and `lang-status` read the embedded bundle,
//! which has the content in it already.

use std::fs;
use std::path::{Path, PathBuf};

use shuyi_cli::Project;
use trpg_content::bundle::{CONTENT_DISPLAY_ROOT, DISPLAY_ROOT};

/// The content's folder, from the repository's root.
pub const CONTENT: &str = CONTENT_DISPLAY_ROOT;

/// The folders `assets/`'s files are read from, from the repository's
/// root, in order: the content's, then `assets/` itself.
pub const STORY: [&str; 2] = [CONTENT, DISPLAY_ROOT];

/// The files directly in the folder `dir` of each of `roots` (folders of
/// `root`, top layer first) whose name ends in `ext`, each name once: from
/// the first root that has it, as the bundle reads them. Each with the name
/// it is shown under (`assets-private/content/dialogue/ch02.dlg`), sorted
/// by that name. The content's folder may be missing; any other is an
/// error.
pub fn files_in(
    root: &Path,
    roots: &[&str],
    dir: &str,
    ext: &str,
) -> Result<Vec<(String, PathBuf)>, String> {
    let mut out: Vec<(String, String, PathBuf)> = Vec::new();
    for layer in roots {
        let folder = root.join(layer).join(dir);
        if *layer == CONTENT && !folder.is_dir() {
            continue;
        }
        let entries =
            fs::read_dir(&folder).map_err(|e| format!("can't read {}: {e}", shown(&folder)))?;
        let mut paths: Vec<PathBuf> = entries.filter_map(|e| Some(e.ok()?.path())).collect();
        paths.sort();
        for path in paths {
            let file = path.file_name().unwrap_or_default().to_string_lossy();
            if !file.ends_with(ext) || out.iter().any(|(name, _, _)| *name == file) {
                continue;
            }
            let shown = format!("{layer}/{dir}/{file}");
            out.push((file.into_owned(), shown, path));
        }
    }
    let mut out: Vec<(String, PathBuf)> = out
        .into_iter()
        .map(|(_, shown, path)| (shown, path))
        .collect();
    out.sort();
    Ok(out)
}

/// `path` with forward slashes, as problems name files.
fn shown(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// `project` with the content in what `check-keys` scans when it is there
/// (`check_keys.assets`), its keymap file left out as `assets/`'s is: the
/// project `check-keys` and `lint` run on.
pub fn project(mut project: Project) -> Project {
    if !project.root.join(CONTENT).is_dir() {
        return project;
    }
    if let Some(keys) = project.config.check_keys.as_mut() {
        let prefix = format!("{DISPLAY_ROOT}/");
        let content: Vec<String> = keys
            .not_text
            .iter()
            .filter_map(|path| path.strip_prefix(&prefix))
            .map(|path| format!("{CONTENT}/{path}"))
            .collect();
        keys.not_text.extend(content);
        keys.assets.insert(0, CONTENT.to_owned());
    }
    project
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A repository with `assets/dialogue/` and, with `content`, a
    /// `assets-private/content/dialogue/`, under the system's temp folder.
    pub(crate) fn repo(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xtask-content-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for (path, text) in files {
            let path = dir.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        dir
    }

    #[test]
    fn a_content_file_replaces_the_assets_file_of_its_name() {
        let root = repo(
            "files",
            &[
                ("assets/dialogue/a.dlg", "public a"),
                ("assets/dialogue/b.dlg", "public b"),
                ("assets/dialogue/README.md", "notes"),
                ("assets-private/content/dialogue/a.dlg", "private a"),
                ("assets-private/content/dialogue/c.dlg", "private c"),
            ],
        );
        let files = files_in(&root, &STORY, "dialogue", ".dlg").unwrap();
        let read: Vec<(&str, String)> = files
            .iter()
            .map(|(shown, path)| (shown.as_str(), fs::read_to_string(path).unwrap()))
            .collect();
        assert_eq!(
            read,
            [
                (
                    "assets-private/content/dialogue/a.dlg",
                    "private a".to_owned()
                ),
                (
                    "assets-private/content/dialogue/c.dlg",
                    "private c".to_owned()
                ),
                ("assets/dialogue/b.dlg", "public b".to_owned()),
            ]
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn without_the_content_only_assets_is_read_and_it_must_be_there() {
        let root = repo("public", &[("assets/dialogue/a.dlg", "public a")]);
        let files = files_in(&root, &STORY, "dialogue", ".dlg").unwrap();
        let names: Vec<&str> = files.iter().map(|(shown, _)| shown.as_str()).collect();
        assert_eq!(names, ["assets/dialogue/a.dlg"]);
        let missing = files_in(&root, &STORY, "maps", ".map").unwrap_err();
        assert!(missing.starts_with("can't read "), "{missing}");
        fs::remove_dir_all(&root).unwrap();
    }

    /// `check-keys` scans the content's scripts and data too, but its
    /// keymap file, when the content is there; the project is as it was
    /// when not.
    #[test]
    fn check_keys_scans_the_content_when_it_is_there() {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = Project::load(&repository).unwrap().config;
        let root = repo(
            "keys",
            &[
                ("assets/dialogue/a.dlg", "@scene a\n> Go.\n@end\n"),
                ("assets-test/dialogue/t.dlg", "@scene t\n> Go.\n@end\n"),
                (
                    "assets-private/content/dialogue/a.dlg",
                    "@scene a\n> Press Enter to go.\n@end\n",
                ),
                (
                    "assets-private/content/data/keymap.ron",
                    "(enter: \"Enter\")",
                ),
            ],
        );
        let layered = project(Project::new(&root, config.clone()));
        let keys = layered.config.check_keys.as_ref().unwrap();
        assert_eq!(keys.assets, [CONTENT, "assets", "assets-test"]);
        assert!(
            keys.not_text
                .contains(&format!("{CONTENT}/data/keymap.ron"))
        );
        let rules = shuyi_cli::project::CheckKeys {
            rust: Vec::new(),
            ..keys.clone()
        };
        let errors = shuyi_cli::check_keys::run(&rules, &root);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(
            errors[0].starts_with("assets-private/content/dialogue/a.dlg:2"),
            "{errors:?}"
        );
        fs::remove_dir_all(root.join("assets-private")).unwrap();
        let public = project(Project::new(&root, config.clone()));
        assert_eq!(public.config, config);
        fs::remove_dir_all(&root).unwrap();
    }
}
