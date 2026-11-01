//! Rebuild when anything under `assets/` or `assets-test/` (the test files,
//! ADR-0057) changes. The same for `assets-private/game/` and
//! `assets-private/test/` when the `private-assets` feature embeds them
//! (ADR-0040), after checking that the first is there.
//!
//! Also writes, into `OUT_DIR`, the tables of files that are the bundle's
//! layers (`shuyi_assets::Layer::table`), each every file of its folder but
//! the `README.md`s, which no build embeds (ADR-0105), sorted:
//! `bundle_files.rs` of `assets/`, `test_files.rs` of `assets-test/`,
//! `private_files.rs` and `private_test_files.rs` of the private folders
//! (empty without the feature); `playdate_files.rs`, `assets/` without the
//! files `playdate-bundle.txt` leaves out (ticket 1036), and
//! `playdate_left_out.rs`, the paths of those.
//!
//! The Playdate's table holds a one-bit picture in place of each PNG it
//! draws one-bit (ticket 1040, ADR-0085): `src/playdate_images.rs` says
//! which and how, and this converts them into `OUT_DIR/onebit/`.
//!
//! With the `private-assets` feature, `playdate_private_files.rs` is the
//! Playdate's private layer (ticket 1061): `assets-private/game/` the same
//! way, its pictures in `OUT_DIR/onebit-private/`, a touched-up `.1bit`
//! beside a PNG taken as it is. Without the feature it is empty.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use shuyi_assets::bitmap::EXTENSION;
use shuyi_assets::bitmap::convert::png_pixels;

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

/// The test files, relative to this crate (ADR-0057).
const TEST: &str = "../../assets-test";

/// The name of the notes in an asset folder, which no build embeds
/// (ADR-0105).
const README: &str = "README.md";

/// What the Playdate build leaves out of `assets/` (ticket 1036).
const PLAYDATE_LIST: &str = "playdate-bundle.txt";

fn main() {
    println!("cargo:rerun-if-changed={ASSETS}");
    println!("cargo:rerun-if-changed={TEST}");
    println!("cargo:rerun-if-changed={PLAYDATE_LIST}");
    let private = std::env::var_os("CARGO_FEATURE_PRIVATE_ASSETS").is_some();
    if private {
        assert!(
            Path::new(PRIVATE).is_dir(),
            "the `private-assets` feature needs the bought art in assets-private/game/, \
             which isn't there. Fetch it with `cargo shuyi private-assets` (it needs \
             access to the private repository), or build without the feature to use \
             the public placeholders."
        );
        println!("cargo:rerun-if-changed={PRIVATE}");
        println!("cargo:rerun-if-changed={PRIVATE_TEST}");
    }
    write_file_tables(private);
}

/// Where a table's entry is read from: the environment variable naming a
/// folder (`CARGO_MANIFEST_DIR` or `OUT_DIR`) and the file's path in it.
type Source = (&'static str, String);

/// Writes the tables (see the top of this file). Each is a `&[(path,
/// bytes)]` expression with one `(path, include_bytes!(…))` per file,
/// sorted by path (`/`-separated, relative to its folder), so the bundle can
/// binary-search it. With `private`, the private layers too
/// ([`private_table`], [`whole_table`]); without, they are empty.
fn write_file_tables(private: bool) {
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
    let read = |path: &str| std::fs::read_to_string(Path::new(ASSETS).join(path)).ok();
    let one_bit = convert_pictures(ASSETS, "onebit", &kept, read);
    let as_is = |path: &String| as_is(ASSETS, path);
    let playdate = |path: &String| -> Source {
        if one_bit.contains(path) {
            ("OUT_DIR", format!("/{}", one_bit_file("onebit", path)))
        } else {
            as_is(path)
        }
    };
    write_out("bundle_files.rs", &table(files.iter(), as_is));
    write_out("playdate_files.rs", &table(kept.iter(), playdate));
    write_out("test_files.rs", &whole_table(TEST));
    let empty = || String::from("&[]\n");
    let (playdate_private, game, test) = if private {
        (
            private_table(&list, &files),
            whole_table(PRIVATE),
            whole_table(PRIVATE_TEST),
        )
    } else {
        (empty(), empty(), empty())
    };
    write_out("playdate_private_files.rs", &playdate_private);
    write_out("private_files.rs", &game);
    write_out("private_test_files.rs", &test);
    let mut paths = String::from("&[\n");
    for path in &left_out {
        writeln!(paths, "    {path:?},").unwrap_or_else(|e| panic!("writing to a String: {e}"));
    }
    paths.push_str("]\n");
    write_out("playdate_left_out.rs", &paths);
}

/// The file at `path` in `folder`, as it is.
fn as_is(folder: &str, path: &String) -> Source {
    ("CARGO_MANIFEST_DIR", format!("/{folder}/{path}"))
}

/// The table of every file in `folder`; empty if it isn't there, as
/// `assets-private/test/` may not be.
fn whole_table(folder: &str) -> String {
    let mut files = Vec::new();
    if Path::new(folder).is_dir() {
        walk(Path::new(folder), "", &mut files);
    }
    files.sort_unstable();
    table(files.iter(), |path| as_is(folder, path))
}

/// The Playdate's table of the bought art in `assets-private/game/`
/// (ticket 1061): every file but those `list` leaves out, each picture the
/// Playdate draws one-bit converted into `OUT_DIR/onebit-private/`, or,
/// when a touched-up `<stem>.1bit` lies beside the PNG (ticket 1042), that
/// file as it is. A fighter's scale is its sidecar's here, else the one in
/// `assets/` (`public`, its files), as the layered bundle reads it.
fn private_table(list: &str, public: &[String]) -> String {
    let mut files = Vec::new();
    walk(Path::new(PRIVATE), "", &mut files);
    files.sort_unstable();
    let rules = rules(list);
    files.retain(|path| !rules.iter().any(|rule| matches(rule, path)));
    let touched = playdate_images::touch_ups(&files);
    files.retain(|path| !touched.values().any(|file| file == path));
    let mut to_convert: Vec<String> = files
        .iter()
        .filter(|path| !touched.contains_key(*path))
        .cloned()
        .collect();
    to_convert.extend(
        public
            .iter()
            .filter(|path| is_sidecar(path) && files.binary_search(path).is_err())
            .cloned(),
    );
    let read = |path: &str| {
        std::fs::read_to_string(Path::new(PRIVATE).join(path))
            .or_else(|_| std::fs::read_to_string(Path::new(ASSETS).join(path)))
            .ok()
    };
    let one_bit = convert_pictures(PRIVATE, "onebit-private", &to_convert, read);
    let source = |path: &String| -> Source {
        if let Some(file) = touched.get(path) {
            ("CARGO_MANIFEST_DIR", format!("/{PRIVATE}/{file}"))
        } else if one_bit.contains(path) {
            (
                "OUT_DIR",
                format!("/{}", one_bit_file("onebit-private", path)),
            )
        } else {
            ("CARGO_MANIFEST_DIR", format!("/{PRIVATE}/{path}"))
        }
    };
    table(files.iter(), source)
}

/// Whether `path` is a battler sidecar (`battlers/<name>.ron`), which sets
/// a fighter's scale.
fn is_sidecar(path: &str) -> bool {
    path.strip_prefix("battlers/")
        .and_then(|name| name.rsplit_once('.'))
        .is_some_and(|(name, ext)| !name.contains('/') && ext.eq_ignore_ascii_case("ron"))
}

/// The table expression of `files`, each read from its `source`.
fn table<'a>(
    files: impl Iterator<Item = &'a String>,
    source: impl Fn(&String) -> Source,
) -> String {
    let mut table = String::from("&[\n");
    for path in files {
        let (root, file) = source(path);
        writeln!(
            table,
            "    ({path:?}, include_bytes!(concat!(env!({root:?}), {file:?}))),"
        )
        .unwrap_or_else(|e| panic!("writing to a String: {e}"));
    }
    table.push_str("]\n");
    table
}

/// Where in `OUT_DIR`, below its folder `dir`, the one-bit picture of the
/// PNG at `path` goes.
fn one_bit_file(dir: &str, path: &str) -> String {
    let stem = path.rsplit_once('.').map_or(path, |(stem, _)| stem);
    format!("{dir}/{stem}.{EXTENSION}")
}

/// Converts each picture of `files` (in the folder `folder`) the Playdate
/// draws one-bit into `OUT_DIR`'s folder `dir` ([`one_bit_file`]),
/// returning their paths. `read` gives a battler sidecar's text.
fn convert_pictures(
    folder: &str,
    dir: &str,
    files: &[String],
    read: impl Fn(&str) -> Option<String>,
) -> BTreeSet<String> {
    let shown = folder.trim_start_matches("../../");
    let jobs = playdate_images::jobs(files, read).unwrap_or_else(|e| panic!("{shown}/{e}"));
    for job in &jobs {
        let fail = |e: String| -> ! { panic!("{shown}/{}: {e}", job.path) };
        let png =
            std::fs::read(Path::new(folder).join(job.path)).unwrap_or_else(|e| fail(e.to_string()));
        let (width, height, rgba) = png_pixels(&png).unwrap_or_else(|e| fail(e));
        let file = playdate_images::make(width, height, &rgba, job).unwrap_or_else(|e| fail(e));
        let out = out_dir().join(one_bit_file(dir, job.path));
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

/// The rules of `list` with their line numbers: each line but blank ones
/// and `#` comments.
fn rules(list: &str) -> Vec<(usize, &str)> {
    list.lines()
        .enumerate()
        .map(|(n, line)| (n + 1, line.trim()))
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// Whether a rule of `playdate-bundle.txt` leaves out the file at `path`:
/// the rule is a file's path or a folder's (`audio/sfx/`, everything below
/// it).
fn matches(&(_, rule): &(usize, &str), path: &str) -> bool {
    if rule.ends_with('/') {
        path.starts_with(rule)
    } else {
        path == rule
    }
}

/// The paths of `files` (sorted) that `list` leaves out ([`matches`]). A
/// rule that matches no file of `assets/` is an error.
fn left_out(list: &str, files: &[String]) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for rule in rules(list) {
        let before = out.len();
        out.extend(files.iter().filter(|path| matches(&rule, path)).cloned());
        if out.len() == before {
            return Err(format!(
                "line {}: `{}` is no file under assets/",
                rule.0, rule.1
            ));
        }
    }
    out.sort_unstable();
    out.dedup();
    Ok(out)
}

/// Adds the path (`prefix` + its name, `/`-separated) of every file in `dir`
/// and the directories below it to `files`, but the [`README`]s.
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
        } else if name != README {
            files.push(path);
        }
    }
}
