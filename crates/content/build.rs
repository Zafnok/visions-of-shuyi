//! Rebuild when anything under `assets/` or `assets-test/` (the test files,
//! ADR-0057) changes: `include_dir!` embeds the directory but does not tell
//! cargo to watch it on stable Rust. The same for `assets-private/game/` and
//! `assets-private/test/` when the `private-assets` feature embeds them
//! (ADR-0040), after checking that the first is there.
//!
//! Also writes `bundle_files.rs` into `OUT_DIR`: the sorted table of every
//! file under `assets/` that is the bundle's layer without `std`, where
//! `include_dir` does not build (ADR-0070 §3, `shuyi_assets::Layer::table`).

use std::fmt::Write as _;
use std::path::Path;

/// The bought art, relative to this crate (ADR-0040).
const PRIVATE: &str = "../../assets-private/game";

/// The bought art of the test files, relative to this crate (ADR-0057).
const PRIVATE_TEST: &str = "../../assets-private/test";

/// The story's files, relative to this crate.
const ASSETS: &str = "../../assets";

fn main() {
    println!("cargo:rerun-if-changed={ASSETS}");
    println!("cargo:rerun-if-changed=../../assets-test");
    write_file_table();
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

/// Writes `OUT_DIR/bundle_files.rs`: a `&[(path, bytes)]` expression with
/// one `(path, include_bytes!(…))` per file under `assets/`, sorted by path
/// (`/`-separated, relative to `assets/`), so the bundle can binary-search
/// it. Every file `include_dir` embeds is in it; a test keeps them equal.
fn write_file_table() {
    let mut files = Vec::new();
    walk(Path::new(ASSETS), "", &mut files);
    files.sort_unstable();
    let mut table = String::from("&[\n");
    for path in files {
        let file = format!("/{ASSETS}/{path}");
        writeln!(
            table,
            "    ({path:?}, include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), {file:?}))),"
        )
        .unwrap_or_else(|e| panic!("writing to a String: {e}"));
    }
    table.push_str("]\n");
    let out_dir = std::env::var_os("OUT_DIR").unwrap_or_else(|| panic!("cargo sets OUT_DIR"));
    let out = Path::new(&out_dir).join("bundle_files.rs");
    std::fs::write(&out, table).unwrap_or_else(|e| panic!("writing {}: {e}", out.display()));
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
