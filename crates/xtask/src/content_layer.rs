// engine-friction: engine#3443, moves (game 1400 deletes it)
//! The game's files on disk as the build embeds them (ADR-0117): the
//! content from the private repository, in `assets-private/content/`, laid
//! over `assets/` when it is there. `check-leaks` reads this view, and so do
//! `check-script` and `voice` in `trpg-tools`, whose `content_layer` adds
//! the project `check-keys` scans.
//!
//! The two folders are `trpg_content::bundle`'s `CONTENT_DISPLAY_ROOT` and
//! `DISPLAY_ROOT`, written out here so this crate builds without the game
//! (ADR-0129); a test in `trpg-tools` keeps them equal.

use std::fs;
use std::path::{Path, PathBuf};

/// The content's folder, from the repository's root.
pub const CONTENT: &str = "assets-private/content";

/// The public files' folder, from the repository's root.
pub const ASSETS: &str = "assets";

/// The folders `assets/`'s files are read from, from the repository's
/// root, in order: the content's, then `assets/` itself.
pub const STORY: [&str; 2] = [CONTENT, ASSETS];

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

#[cfg(test)]
mod tests {
    use super::*;

    /// A repository of `files` under the system's temp folder.
    fn repo(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xtask-layer-{name}-{}", std::process::id()));
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
                ("assets/dialogue/notes.txt", "not a script"),
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
}
