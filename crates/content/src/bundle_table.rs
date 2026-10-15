//! The asset bundle without `std` (the Playdate build, ADR-0070 §3):
//! `include_dir` needs `std`, so `build.rs` writes a table of every file
//! under `assets/`, sorted by path, and this module reads it. Same public
//! functions as the `std` bundle (`bundle.rs`), which a test keeps listing
//! the same files with the same bytes. A build without `std` is a device
//! build: it has no test files (ADR-0057) and no private layer (ADR-0040).
use crate::prelude::*;

/// Every file under `assets/`: its path (relative to `assets/`,
/// `/`-separated) and contents, sorted by path.
static FILES: &[(&str, &[u8])] = include!(concat!(env!("OUT_DIR"), "/bundle_files.rs"));

/// Whether this build embeds the test files: never without `std`.
pub const TEST_FILES: bool = false;

/// Prefix used when showing an asset path to a human (`assets/data/x.ron`).
pub const DISPLAY_ROOT: &str = "assets";

/// The same prefix for a file that comes from the private assets
/// (`assets-private/game/portraits/x.png`).
pub const PRIVATE_DISPLAY_ROOT: &str = "assets-private/game";

/// The same prefix for a test file (`assets-test/maps/test_small.map`).
pub const TEST_DISPLAY_ROOT: &str = "assets-test";

/// The same prefix for a test file that is bought art
/// (`assets-private/test/portraits/test_lord/neutral.png`).
pub const PRIVATE_TEST_DISPLAY_ROOT: &str = "assets-private/test";

/// Runs `f`. With `std` it hides the test files while `f` runs; this build
/// has none.
pub fn story_only<T>(f: impl FnOnce() -> T) -> T {
    f()
}

/// The contents of the file at `path`.
fn find(path: &str) -> Option<&'static [u8]> {
    let at = FILES.binary_search_by(|(p, _)| (*p).cmp(path)).ok()?;
    Some(FILES[at].1)
}

/// Returns the UTF-8 contents of the embedded file at `path` (relative to
/// `assets/`, `/`-separated), or `None` if it is missing or not valid UTF-8.
pub fn file(path: &str) -> Option<&'static str> {
    core::str::from_utf8(find(path)?).ok()
}

/// The test files' own file at `path`: never, without test files.
pub fn test_part(_path: &str) -> Option<(String, &'static str)> {
    None
}

/// Returns the raw bytes of the embedded file at `path` (relative to
/// `assets/`, `/`-separated), e.g. an image, or `None` if it is missing.
pub fn bytes(path: &str) -> Option<&'static [u8]> {
    find(path)
}

/// The paths (sorted) of the files inside `dir`: directly inside it, or
/// with `deep` also in any directory below it.
fn list(dir: &str, deep: bool) -> Vec<&'static str> {
    FILES
        .iter()
        .map(|(path, _)| *path)
        .filter(|path| {
            let rest = if dir.is_empty() {
                Some(*path)
            } else {
                path.strip_prefix(dir)
                    .and_then(|rest| rest.strip_prefix('/'))
            };
            rest.is_some_and(|rest| deep || !rest.contains('/'))
        })
        .collect()
}

/// Lists the paths (relative to `assets/`, `/`-separated, sorted) of every
/// file directly inside directory `dir`. A missing directory yields an empty
/// list. Use `""` for the bundle root.
pub fn files_in(dir: &str) -> Vec<&'static str> {
    list(dir, false)
}

/// Lists the paths (relative to `assets/`, `/`-separated, sorted) of every
/// file inside directory `dir` or any directory below it. A missing
/// directory yields an empty list. Use `""` for the whole bundle.
pub fn files_under(dir: &str) -> Vec<&'static str> {
    list(dir, true)
}

/// The human-facing name of an asset path, e.g. `assets/data/palette.ron`.
pub fn display_path(path: &str) -> String {
    format!("{DISPLAY_ROOT}/{path}")
}

/// The human-facing name of the test file at `path`, there or not: where a
/// tool that makes a test file writes it.
pub fn test_display_path(path: &str) -> String {
    format!("{TEST_DISPLAY_ROOT}/{path}")
}

/// Whether `shown` (a human-facing name, from [`display_path`]) names a
/// file of the private assets: bought art (ADR-0040), the game's or the
/// test files'.
pub fn is_private(shown: &str) -> bool {
    [PRIVATE_DISPLAY_ROOT, PRIVATE_TEST_DISPLAY_ROOT]
        .iter()
        .any(|root| {
            shown
                .strip_prefix(root)
                .is_some_and(|rest| rest.starts_with('/'))
        })
}
