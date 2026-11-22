//! Writes `OUT_DIR/wins.rs`: every `wins/<battle>.ron` (the bots' own
//! recorded wins, ticket 0531) as `(battle, include_str!(…))`, sorted by
//! battle, so a battle's wins are found by its id with no list to keep.
//! Rebuilds when the folder changes.

use std::fmt::Write as _;
use std::path::Path;

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("wins");
    println!("cargo::rerun-if-changed={}", dir.display());
    let mut files: Vec<(String, String)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "ron")
                && let Some(battle) = path.file_stem().and_then(|s| s.to_str())
            {
                println!("cargo::rerun-if-changed={}", path.display());
                files.push((battle.to_owned(), path.display().to_string()));
            }
        }
    }
    files.sort();
    let mut out = String::from("&[\n");
    for (battle, path) in &files {
        let _ = writeln!(out, "    ({battle:?}, include_str!({path:?})),");
    }
    out.push_str("]\n");
    let out_dir = std::env::var("OUT_DIR").unwrap_or_else(|e| panic!("OUT_DIR: {e}"));
    let written = Path::new(&out_dir).join("wins.rs");
    std::fs::write(&written, out).unwrap_or_else(|e| panic!("{}: {e}", written.display()));
}
