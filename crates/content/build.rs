//! Rebuild when anything under `assets/` or `assets-test/` (the test files,
//! ADR-0057) changes: `include_dir!` embeds the directory but does not tell
//! cargo to watch it on stable Rust. The same for `assets-private/game/` and
//! `assets-private/test/` when the `private-assets` feature embeds them
//! (ADR-0040), after checking that the first is there.

use std::path::Path;

/// The bought art, relative to this crate (ADR-0040).
const PRIVATE: &str = "../../assets-private/game";

/// The bought art of the test files, relative to this crate (ADR-0057).
const PRIVATE_TEST: &str = "../../assets-private/test";

fn main() {
    println!("cargo:rerun-if-changed=../../assets");
    println!("cargo:rerun-if-changed=../../assets-test");
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
