//! The files of a layer's folder, for `build.rs`, which writes each
//! layer's table of them (ticket 1347), and for the test of it. Not a
//! module of the crate: `build.rs` and the test include it by path.

use std::path::Path;

/// The name of the notes in an asset folder, which no build embeds
/// (ADR-0105).
pub const README: &str = "README.md";

/// The paths of every file in `folder` at any depth (`/`-separated,
/// relative to it), sorted, but the [`README`]s and the files a pattern of
/// `skip` matches (`shuyi_embed::matches`). A name that isn't UTF-8 and two
/// paths that differ only by case, which are one file on Windows and macOS,
/// are the engine's error.
pub fn layer_files(folder: &Path, skip: &[&str]) -> Result<Vec<String>, shuyi_embed::Error> {
    let mut patterns = vec![format!("**/{README}")];
    patterns.extend(skip.iter().map(|pattern| (*pattern).to_owned()));
    let walk = shuyi_embed::walk(folder, &patterns)?;
    Ok(walk.files.into_iter().map(|entry| entry.path).collect())
}
