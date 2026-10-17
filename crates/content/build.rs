//! Rebuild when anything under `assets/` or `assets-test/` (the test files,
//! ADR-0057) changes: `include_dir!` embeds the directory but does not tell
//! cargo to watch it on stable Rust. The same for `assets-private/game/` and
//! `assets-private/test/` when the `private-assets` feature embeds them
//! (ADR-0040), after checking that the first is there.
//!
//! Also writes, into `OUT_DIR`, the tables of files that are the bundle's
//! layer without `std`, where `include_dir` does not build (ADR-0070 §3,
//! `shuyi_assets::Layer::table`): `bundle_files.rs`, every file under
//! `assets/`, sorted; `playdate_files.rs`, the same without the files
//! `playdate-bundle.txt` leaves out (ticket 1036), and
//! `playdate_left_out.rs`, the paths of those.
//!
//! The Playdate's table holds a one-bit picture in place of each PNG it
//! draws one-bit (ticket 1040, ADR-0085): `src/playdate_images.rs` says
//! which and how, and this converts them into `OUT_DIR/onebit/`.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use shuyi_assets::bitmap::EXTENSION;
use shuyi_assets::bitmap::convert::{Rgba, convert, png_pixels};

/// The Playdate's one-bit pictures: which and how (the game's table, also
/// `trpg_content::playdate_images`).
#[path = "src/playdate_images.rs"]
#[allow(dead_code)] // the build uses only part of it
mod playdate_images;

/// The bought art, relative to this crate (ADR-0040).
const PRIVATE: &str = "../../assets-private/game";

/// The bought art of the test files, relative to this crate (ADR-0057).
const PRIVATE_TEST: &str = "../../assets-private/test";

/// The story's files, relative to this crate.
const ASSETS: &str = "../../assets";

/// What the Playdate build leaves out of `assets/` (ticket 1036).
const PLAYDATE_LIST: &str = "playdate-bundle.txt";

fn main() {
    println!("cargo:rerun-if-changed={ASSETS}");
    println!("cargo:rerun-if-changed=../../assets-test");
    println!("cargo:rerun-if-changed={PLAYDATE_LIST}");
    write_file_tables();
    if std::env::var_os("CARGO_FEATURE_PRIVATE_ASSETS").is_some() {
        assert!(
            Path::new(PRIVATE).is_dir(),
            "the `private-assets` feature needs the bought art in assets-private/game/, \
             which isn't there. Fetch it with `cargo xtask private-assets` (it needs \
             access to the private repository), or build without the feature to use \
             the public placeholders."
        );
        println!("cargo:rerun-if-changed={PRIVATE}");
        println!("cargo:rerun-if-changed={PRIVATE_TEST}");
    }
}

/// Writes the tables (see the top of this file). Each is a `&[(path,
/// bytes)]` expression with one `(path, include_bytes!(…))` per file,
/// sorted by path (`/`-separated, relative to `assets/`), so the bundle can
/// binary-search it. Every file `include_dir` embeds is in
/// `bundle_files.rs`; a test keeps them equal.
fn write_file_tables() {
    let mut files = Vec::new();
    walk(Path::new(ASSETS), "", &mut files);
    files.sort_unstable();
    let list = std::fs::read_to_string(PLAYDATE_LIST)
        .unwrap_or_else(|e| panic!("reading {PLAYDATE_LIST}: {e}"));
    let left_out = left_out(&list, &files).unwrap_or_else(|e| panic!("{PLAYDATE_LIST}: {e}"));
    let kept: Vec<String> = files
        .iter()
        .filter(|f| !left_out.contains(f))
        .cloned()
        .collect();
    let one_bit = convert_pictures(&kept);
    write_out("bundle_files.rs", &table(files.iter(), &BTreeSet::new()));
    write_out("playdate_files.rs", &table(kept.iter(), &one_bit));
    let mut paths = String::from("&[\n");
    for path in &left_out {
        writeln!(paths, "    {path:?},").unwrap_or_else(|e| panic!("writing to a String: {e}"));
    }
    paths.push_str("]\n");
    write_out("playdate_left_out.rs", &paths);
}

/// The table expression of `files`, each of `one_bit` being its one-bit
/// picture in `OUT_DIR` ([`convert_pictures`]).
fn table<'a>(files: impl Iterator<Item = &'a String>, one_bit: &BTreeSet<String>) -> String {
    let mut table = String::from("&[\n");
    for path in files {
        let (root, file) = if one_bit.contains(path) {
            ("OUT_DIR", format!("/{}", one_bit_file(path)))
        } else {
            ("CARGO_MANIFEST_DIR", format!("/{ASSETS}/{path}"))
        };
        writeln!(
            table,
            "    ({path:?}, include_bytes!(concat!(env!({root:?}), {file:?}))),"
        )
        .unwrap_or_else(|e| panic!("writing to a String: {e}"));
    }
    table.push_str("]\n");
    table
}

/// Where in `OUT_DIR` the one-bit picture of the PNG at `path` goes.
fn one_bit_file(path: &str) -> String {
    let stem = path.rsplit_once('.').map_or(path, |(stem, _)| stem);
    format!("onebit/{stem}.{EXTENSION}")
}

/// Converts each picture of `files` the Playdate draws one-bit into
/// `OUT_DIR` ([`one_bit_file`]), returning their paths.
fn convert_pictures(files: &[String]) -> BTreeSet<String> {
    let read = |path: &str| std::fs::read_to_string(Path::new(ASSETS).join(path)).ok();
    let jobs = playdate_images::jobs(files, read).unwrap_or_else(|e| panic!("assets/{e}"));
    for job in &jobs {
        let fail = |e: String| -> ! { panic!("assets/{}: {e}", job.path) };
        let png =
            std::fs::read(Path::new(ASSETS).join(job.path)).unwrap_or_else(|e| fail(e.to_string()));
        let (width, height, rgba) = png_pixels(&png).unwrap_or_else(|e| fail(e));
        let image = Rgba::new(width, height, &rgba)
            .unwrap_or_else(|| fail("decoded to the wrong number of pixels".into()));
        let file = convert(&image, job.scale, &job.kind.methods()).unwrap_or_else(|e| fail(e));
        let out = out_dir().join(one_bit_file(job.path));
        if let Some(folder) = out.parent() {
            std::fs::create_dir_all(folder)
                .unwrap_or_else(|e| panic!("making {}: {e}", folder.display()));
        }
        std::fs::write(&out, file).unwrap_or_else(|e| panic!("writing {}: {e}", out.display()));
    }
    jobs.iter().map(|job| job.path.to_string()).collect()
}

/// Cargo's `OUT_DIR`.
fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_else(|| panic!("cargo sets OUT_DIR")))
}

/// Writes `contents` to the file `name` in `OUT_DIR`.
fn write_out(name: &str, contents: &str) {
    let out = out_dir().join(name);
    std::fs::write(&out, contents).unwrap_or_else(|e| panic!("writing {}: {e}", out.display()));
}

/// The paths of `files` (sorted) that `list` leaves out: each of its lines
/// but blank ones and `#` comments is a file's path, a folder's
/// (`audio/sfx/`, everything below it) or `**/` and a file name (every file
/// of that name). A line that matches no file is an error.
fn left_out(list: &str, files: &[String]) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for (n, line) in list.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let matches = |path: &&String| {
            if let Some(name) = line.strip_prefix("**/") {
                path.rsplit('/').next() == Some(name)
            } else if line.ends_with('/') {
                path.starts_with(line)
            } else {
                path.as_str() == line
            }
        };
        let before = out.len();
        out.extend(files.iter().filter(matches).cloned());
        if out.len() == before {
            return Err(format!("line {}: `{line}` is no file under assets/", n + 1));
        }
    }
    out.sort_unstable();
    out.dedup();
    Ok(out)
}

/// Adds the path (`prefix` + its name, `/`-separated) of every file in `dir`
/// and the directories below it to `files`.
fn walk(dir: &Path, prefix: &str, files: &mut Vec<String>) {
    let entries =
        std::fs::read_dir(dir).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()));
        let name = entry.file_name();
        let name = name
            .to_str()
            .unwrap_or_else(|| panic!("a file name in {} isn't UTF-8", dir.display()));
        let path = format!("{prefix}{name}");
        if entry.path().is_dir() {
            walk(&entry.path(), &format!("{path}/"), files);
        } else {
            files.push(path);
        }
    }
}
