//! Rebuild when anything under `assets/` or `assets-test/` (the test files,
//! ADR-0057) changes. The same for `assets-private/game/` and
//! `assets-private/test/` when the `private-assets` feature embeds them
//! (ADR-0040), after checking that the first is there, and for
//! `assets-private/content/`, the game's content from the private
//! repository, embedded over `assets/` whenever it is there (ADR-0117).
//!
//! Also writes, into `OUT_DIR`, the tables of files that are the bundle's
//! layers (`shuyi_assets::Layer::table`), each every file of its folder but
//! the `README.md`s, which no build embeds (ADR-0105), sorted:
//! `bundle_files.rs` of `assets/`, `test_files.rs` of `assets-test/`,
//! `private_files.rs` and `private_test_files.rs` of the private folders
//! (empty without the feature), `content_files.rs` of the content (empty
//! without it, and without its `tests/`, the tests' own data, ADR-0118);
//! `playdate_files.rs`, `assets/` without the files
//! `playdate-bundle.txt` leaves out (ticket 1036), and
//! `playdate_left_out.rs`, the paths of those, in `assets/` or the content.
//!
//! The Playdate's table holds a one-bit picture in place of each PNG it
//! draws one-bit (ticket 1040, ADR-0085): `src/playdate_images.rs` says
//! which and how, and this converts them into `OUT_DIR/onebit/`.
//!
//! With the `private-assets` feature, `playdate_private_files.rs` is the
//! Playdate's private layer (ticket 1061): `assets-private/game/` the same
//! way, its pictures in `OUT_DIR/onebit-private/`, a touched-up `.1bit`
//! beside a PNG taken as it is. Without the feature it is empty.
//! `playdate_content_files.rs` is the content's the same way, its pictures
//! in `OUT_DIR/onebit-content/`.
//!
//! `demo_files.rs` and `demo_private_files.rs` are the Chapter 1 demo's
//! (ticket 0914, ADR-0119): `assets/` with the content over it, and
//! `assets-private/game/`, without
//! what `demo/bundle.txt` leaves out, cut as it says
//! (`src/demo_bundle.rs`), the cut copies in `OUT_DIR/demo/`. When an
//! export of the `demo` edition names `demo/` as `SHUYI_CONTENT`, the
//! build gets the `demo_edition` cfg and the bundle is those tables
//! instead of the whole of `assets/` and the content.
//! The demo is Chapter 1 of the content: without the checkout its tables
//! are empty and the `demo` edition doesn't build (ADR-0121).
//!
//! When an export of the `story-preview` edition (the writer's script
//! preview, ticket 0764) names `story-preview/` as `SHUYI_CONTENT`, every
//! table leaves out what `story-preview/bundle.txt` lists (the scripts):
//! the program holds no scene but the ones in the folder it reads.

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

/// The demo's edition file (ticket 0914).
#[path = "src/edition.rs"]
mod edition;

/// What the demo embeds and how it is cut (ticket 0914), relative to this
/// crate.
#[path = "src/demo_bundle.rs"]
#[allow(dead_code)] // the build uses only part of it
mod demo_bundle;

/// The demo edition's content root, relative to this crate (ticket 0914).
const DEMO: &str = "demo";

/// Its list of what it leaves out and cuts.
const DEMO_LIST: &str = "demo/bundle.txt";

/// The writer's preview's content root, relative to this crate (ticket
/// 0764).
const PREVIEW: &str = "story-preview";

/// Its list of what it leaves out.
const PREVIEW_LIST: &str = "story-preview/bundle.txt";

/// What every table leaves out: the preview edition's list, in its build.
static LEFT_OUT: std::sync::OnceLock<demo_bundle::Plan> = std::sync::OnceLock::new();

/// The bought art, relative to this crate (ADR-0040).
const PRIVATE: &str = "../../assets-private/game";

/// The bought art of the test files, relative to this crate (ADR-0057).
const PRIVATE_TEST: &str = "../../assets-private/test";

/// The game's content from the private repository, relative to this crate
/// (ADR-0117): embedded over `assets/` whenever it is there.
const CONTENT: &str = "../../assets-private/content";

/// The private repository's checkout, relative to this crate (ADR-0040).
const CHECKOUT: &str = "../../assets-private";

/// The story's files, relative to this crate.
const ASSETS: &str = "../../assets";

/// The test files, relative to this crate (ADR-0057).
const TEST: &str = "../../assets-test";

/// The name of the notes in an asset folder, which no build embeds
/// (ADR-0105).
const README: &str = "README.md";

/// The content's own test data (ADR-0118): what the tests of the real
/// content check it against, read from disk by them, never embedded.
const CONTENT_TESTS: &str = "tests";

/// What the Playdate build leaves out of `assets/` (ticket 1036).
const PLAYDATE_LIST: &str = "playdate-bundle.txt";

fn main() {
    println!("cargo:rerun-if-changed={ASSETS}");
    println!("cargo:rerun-if-changed={TEST}");
    println!("cargo:rerun-if-changed={PLAYDATE_LIST}");
    println!("cargo:rerun-if-changed={DEMO}");
    println!("cargo:rerun-if-changed={PREVIEW}");
    println!("cargo:rerun-if-env-changed=SHUYI_CONTENT");
    println!("cargo::rustc-check-cfg=cfg(demo_edition)");
    let mut demo = false;
    if let Some(root) = std::env::var_os("SHUYI_CONTENT") {
        let named = Path::new(&root).canonicalize().ok();
        let is = |dir: &str| named.is_some() && named == Path::new(dir).canonicalize().ok();
        if is(DEMO) {
            println!("cargo:rustc-cfg=demo_edition");
            demo = true;
        } else if is(PREVIEW) {
            let list = std::fs::read_to_string(PREVIEW_LIST)
                .unwrap_or_else(|e| panic!("reading {PREVIEW_LIST}: {e}"));
            let plan =
                demo_bundle::Plan::parse(&list).unwrap_or_else(|e| panic!("{PREVIEW_LIST}: {e}"));
            let _ = LEFT_OUT.set(plan);
        } else {
            panic!(
                "SHUYI_CONTENT is {}, which is no edition's content root this game has: \
                 crates/content/demo is the demo's, crates/content/story-preview the \
                 writer's preview's (shuyi.ron's `editions`)",
                root.display()
            );
        }
    }
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
    let content = Path::new(CONTENT).is_dir();
    watch_content(content);
    write_file_tables(private, content, demo);
}

/// Tells cargo when the content layer changes (ADR-0117): the content when
/// it is there. Else what a fetch that brings it changes, so the build that
/// follows has it: the checkout's git index, or the checkout's folder. A
/// path cargo watches must be there (while one is missing, cargo rebuilds
/// this crate on every build), so with no checkout at all this makes the
/// folder, empty and git-ignored, for `cargo shuyi private-assets` to clone
/// into. The test that every layer is its folder (`bundle.rs`) catches a
/// build that still missed one.
fn watch_content(content: bool) {
    if content {
        println!("cargo:rerun-if-changed={CONTENT}");
        return;
    }
    let checkout = Path::new(CHECKOUT);
    if !checkout.is_dir() {
        // A read-only tree can't have it: then nothing is watched.
        let _ = std::fs::create_dir_all(checkout);
    }
    let index = checkout.join(".git/index");
    if index.is_file() {
        println!("cargo:rerun-if-changed={CHECKOUT}/.git/index");
    } else if checkout.is_dir() {
        println!("cargo:rerun-if-changed={CHECKOUT}");
    }
}

/// Where a table's entry is read from: the environment variable naming a
/// folder (`CARGO_MANIFEST_DIR` or `OUT_DIR`) and the file's path in it.
type Source = (&'static str, String);

/// Writes the tables (see the top of this file). Each is a `&[(path,
/// bytes)]` expression with one `(path, include_bytes!(…))` per file,
/// sorted by path (`/`-separated, relative to its folder), so the bundle can
/// binary-search it. With `private`, the private layers too
/// ([`playdate_table`], [`whole_table`]); with `content`, the content's;
/// without, they are empty.
fn write_file_tables(private: bool, content: bool, demo: bool) {
    let mut files = Vec::new();
    walk(Path::new(ASSETS), "", &mut files);
    files.sort_unstable();
    let mut content_files = Vec::new();
    if content {
        walk(Path::new(CONTENT), "", &mut content_files);
        content_files.sort_unstable();
    }
    let mut both: Vec<String> = files.iter().chain(&content_files).cloned().collect();
    both.sort_unstable();
    both.dedup();
    if let (Some(plan), true) = (LEFT_OUT.get(), content) {
        plan.left_out(&both)
            .unwrap_or_else(|e| panic!("{PREVIEW_LIST}: {e}"));
    }
    let list = std::fs::read_to_string(PLAYDATE_LIST)
        .unwrap_or_else(|e| panic!("reading {PLAYDATE_LIST}: {e}"));
    let left_out = left_out(&list, &both).unwrap_or_else(|e| panic!("{PLAYDATE_LIST}: {e}"));
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
    let (playdate_content, own) = if content {
        let below = Below {
            folders: &[ASSETS],
            files: &files,
        };
        (
            playdate_table(CONTENT, "onebit-content", &list, &below),
            whole_table(CONTENT),
        )
    } else {
        (empty(), empty())
    };
    write_out("playdate_content_files.rs", &playdate_content);
    write_out("content_files.rs", &own);
    let (playdate_private, game, test) = if private {
        let below = Below {
            folders: &[CONTENT, ASSETS],
            files: &both,
        };
        (
            playdate_table(PRIVATE, "onebit-private", &list, &below),
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
    if content {
        write_demo(&both, &content_files, private);
    } else {
        write_no_demo(demo);
    }
}

/// The demo's tables without the content (ADR-0117): empty. The demo is
/// Chapter 1 of the game's content, so a build without the checkout (a
/// fork) has none, and an export of the demo edition needs the checkout.
fn write_no_demo(demo: bool) {
    assert!(
        !demo,
        "the demo edition is Chapter 1 of the game's content, which isn't in          assets-private/content/. Fetch it with `cargo shuyi private-assets` (it needs          access to the private repository)."
    );
    write_out(
        "demo_files.rs",
        "&[]
",
    );
    write_out(
        "demo_private_files.rs",
        "&[]
",
    );
}

/// Writes the demo's tables (ticket 0914): `demo_files.rs` of `assets/`
/// with the content over it (`files`, both sorted; `content`, those the
/// content has, sorted, each read from there) and the edition's file,
/// `demo_private_files.rs` of `assets-private/game/` with `private` (empty
/// without), each without what `demo/bundle.txt` leaves out, the files it
/// cuts written cut into `OUT_DIR/demo/` and `OUT_DIR/demo-private/`.
fn write_demo(files: &[String], content: &[String], private: bool) {
    let list =
        std::fs::read_to_string(DEMO_LIST).unwrap_or_else(|e| panic!("reading {DEMO_LIST}: {e}"));
    let plan = demo_bundle::Plan::parse(&list).unwrap_or_else(|e| panic!("{DEMO_LIST}: {e}"));
    plan.left_out(files)
        .unwrap_or_else(|e| panic!("{DEMO_LIST}: {e}"));
    let edition = edition::EDITION_FILE.to_owned();
    let mut kept: Vec<String> = files
        .iter()
        .filter(|f| !plan.leaves_out(f))
        .cloned()
        .collect();
    kept.push(edition.clone());
    kept.sort_unstable();
    write_out(&format!("demo/{edition}"), edition::DEMO);
    let source = |path: &String| -> Source {
        if *path == edition {
            ("OUT_DIR", format!("/demo/{edition}"))
        } else if content.binary_search(path).is_ok() {
            demo_copy(&plan, CONTENT, "demo", path)
        } else {
            demo_copy(&plan, ASSETS, "demo", path)
        }
    };
    write_out("demo_files.rs", &table(kept.iter(), source));
    let mut art = Vec::new();
    if private {
        walk(Path::new(PRIVATE), "", &mut art);
        art.retain(|f| !plan.leaves_out(f));
        art.sort_unstable();
    }
    let source = |path: &String| demo_copy(&plan, PRIVATE, "demo-private", path);
    write_out("demo_private_files.rs", &table(art.iter(), source));
}

/// Where the demo's table reads the file at `path` in `folder`: the file,
/// or its cut copy (`demo_bundle::Plan::rewrite`), written into `OUT_DIR`'s
/// folder `dir`.
fn demo_copy(plan: &demo_bundle::Plan, folder: &str, dir: &str, path: &String) -> Source {
    let Ok(text) = std::fs::read_to_string(Path::new(folder).join(path)) else {
        return as_is(folder, path);
    };
    let shown = folder.trim_start_matches("../../");
    match plan.rewrite(path, &text) {
        Ok(Some(cut)) => {
            write_out(&format!("{dir}/{path}"), &cut);
            ("OUT_DIR", format!("/{dir}/{path}"))
        }
        Ok(None) => as_is(folder, path),
        Err(e) => panic!("{DEMO_LIST}: {shown}/{e}"),
    }
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

/// The layers a Playdate layer lies over, for the battler sidecars it
/// doesn't have: their folders, in the order they are read, and their
/// files (sorted).
struct Below<'a> {
    folders: &'a [&'static str],
    files: &'a [String],
}

/// The Playdate's table of a layer over `assets/`, the folder `layer`: the
/// bought art in `assets-private/game/` (ticket 1061) or the content in
/// `assets-private/content/` (ADR-0117). Every file but those `list` leaves
/// out, each picture the Playdate draws one-bit converted into `OUT_DIR`'s
/// folder `dir`, or, when a touched-up `<stem>.1bit` lies beside the PNG
/// (ticket 1042), that file as it is. A fighter's scale is its sidecar's
/// here, else the one in the layers `below`, as the layered bundle reads it.
fn playdate_table(layer: &'static str, dir: &str, list: &str, below: &Below<'_>) -> String {
    let mut files = Vec::new();
    walk(Path::new(layer), "", &mut files);
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
        below
            .files
            .iter()
            .filter(|path| is_sidecar(path) && files.binary_search(path).is_err())
            .cloned(),
    );
    let read = |path: &str| {
        std::iter::once(layer)
            .chain(below.folders.iter().copied())
            .find_map(|folder| std::fs::read_to_string(Path::new(folder).join(path)).ok())
    };
    let one_bit = convert_pictures(layer, dir, &to_convert, read);
    let source = |path: &String| -> Source {
        if let Some(file) = touched.get(path) {
            ("CARGO_MANIFEST_DIR", format!("/{layer}/{file}"))
        } else if one_bit.contains(path) {
            ("OUT_DIR", format!("/{}", one_bit_file(dir, path)))
        } else {
            ("CARGO_MANIFEST_DIR", format!("/{layer}/{path}"))
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

/// The table expression of `files`, each read from its `source`, but those
/// the writer's preview leaves out in its build ([`LEFT_OUT`]).
fn table<'a>(
    files: impl Iterator<Item = &'a String>,
    source: impl Fn(&String) -> Source,
) -> String {
    let mut table = String::from("&[\n");
    let left_out = |path: &str| LEFT_OUT.get().is_some_and(|plan| plan.leaves_out(path));
    for path in files.filter(|path| !left_out(path)) {
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

/// Writes `contents` to the file `name` in `OUT_DIR`, making its folder.
fn write_out(name: &str, contents: &str) {
    let out = out_dir().join(name);
    if let Some(folder) = out.parent() {
        std::fs::create_dir_all(folder)
            .unwrap_or_else(|e| panic!("making {}: {e}", folder.display()));
    }
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

/// The paths of `files` (sorted: `assets/`'s and the content's) that
/// `list` leaves out ([`matches`]). A rule that matches none of them is an
/// error.
fn left_out(list: &str, files: &[String]) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for rule in rules(list) {
        let before = out.len();
        out.extend(files.iter().filter(|path| matches(&rule, path)).cloned());
        if out.len() == before {
            return Err(format!(
                "line {}: `{}` is no file under assets/ or assets-private/content/",
                rule.0, rule.1
            ));
        }
    }
    out.sort_unstable();
    out.dedup();
    Ok(out)
}

/// Adds the path (`prefix` + its name, `/`-separated) of every file in `dir`
/// and the directories below it to `files`, but the [`README`]s and the
/// content's [`CONTENT_TESTS`].
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
            if dir == Path::new(CONTENT) && name == CONTENT_TESTS {
                continue;
            }
            walk(&entry.path(), &format!("{path}/"), files);
        } else if name != README {
            files.push(path);
        }
    }
}
