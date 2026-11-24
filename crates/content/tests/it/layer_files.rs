//! The build script's walk of a layer's folder (ticket 1347), on a
//! fixture folder of its own: what `build.rs` puts in each table, and
//! the engine's error for two paths that differ only by case.

use std::fs;
use std::path::{Path, PathBuf};

#[path = "../../src/layer_files.rs"]
mod build_walk;

use build_walk::layer_files;

/// An empty folder of its own for test `name`.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "trpg-content-layer-files-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    dir
}

/// Writes `text` at `path` under `root`, making its folders.
fn put(root: &Path, path: &str, text: &str) {
    let file = root.join(path);
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| panic!("{}: {e}", parent.display()));
    }
    fs::write(&file, text).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
}

/// A layer's files are every file at any depth, `/`-separated and sorted
/// by their bytes, but the `README.md`s at every depth and what a skip
/// pattern matches (the content's `tests/`, ADR-0118).
#[test]
fn a_layer_is_every_file_sorted_but_the_readmes_and_its_skips() {
    let root = scratch("walk");
    put(&root, "README.md", "notes");
    put(&root, "maps/README.md", "notes");
    put(&root, "maps/one.map", "one");
    put(&root, "Zebra.ron", "z");
    put(&root, "audio/sfx/hit.ogg", "hit");
    put(&root, "tests/expected.ron", "data");
    let files = layer_files(&root, &["tests/**"]).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(files, ["Zebra.ron", "audio/sfx/hit.ogg", "maps/one.map"]);
    let all = layer_files(&root, &[]).unwrap_or_else(|e| panic!("{e}"));
    assert!(all.contains(&"tests/expected.ron".to_owned()), "{all:?}");
    let _ = fs::remove_dir_all(&root);
}

/// Two paths that differ only by case are one file on Windows and macOS:
/// the walk refuses them with the engine's error, naming both. On a disk
/// that ignores case (Windows, macOS) the second write is the first file,
/// so there is nothing to refuse: the check runs where the disk allows the
/// pair (CI's Linux).
#[test]
fn two_paths_differing_only_by_case_fail_with_the_engines_error() {
    let root = scratch("case");
    put(&root, "portraits/Hero.png", "upper");
    put(&root, "portraits/hero.png", "lower");
    let pair = fs::read_dir(root.join("portraits")).map_or(0, Iterator::count) == 2;
    let result = layer_files(&root, &[]);
    let _ = fs::remove_dir_all(&root);
    if !pair {
        return;
    }
    match result {
        Err(error @ shuyi_embed::Error::SameButCase(..)) => {
            let shown = error.to_string();
            assert!(
                shown.contains("`portraits/Hero.png`") && shown.contains("`portraits/hero.png`"),
                "{shown}"
            );
        }
        other => panic!("not the engine's case error: {other:?}"),
    }
}
