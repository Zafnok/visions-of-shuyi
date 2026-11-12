//! The content layer (ADR-0117): the game's content from the private
//! repository, in `assets-private/content/`, is laid over `assets/`, so a
//! file there replaces `assets/`'s at its path. This test makes a content
//! folder of its own, so it needs no private checkout: what the build
//! embeds from the real one is checked by `bundle`'s own test that every
//! layer is its folder.

use std::collections::BTreeMap;
use std::path::Path;

use trpg_content::bundle::{self, Bundle, CONTENT_DISPLAY_ROOT, Layer};

/// The files of `dir` and below, by `/`-separated path, but the READMEs,
/// which no build embeds (ADR-0105): what `build.rs` reads into the table.
fn read_folder(dir: &Path, prefix: &str, files: &mut BTreeMap<String, Vec<u8>>) {
    let fail = |e: std::io::Error| -> ! { panic!("reading {}: {e}", dir.display()) };
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| fail(e)) {
        let entry = entry.unwrap_or_else(|e| fail(e));
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = format!("{prefix}{name}");
        if entry.path().is_dir() {
            read_folder(&entry.path(), &format!("{path}/"), files);
        } else if name != "README.md" {
            let bytes = std::fs::read(entry.path()).unwrap_or_else(|e| fail(e));
            files.insert(path, bytes);
        }
    }
}

/// A content folder with its own palette, whose `player` colour differs
/// from `assets/`'s, loads with the game's loaders: the palette is the
/// content's, shown under its root, and every other file is still
/// `assets/`'s.
#[test]
fn a_file_in_the_content_replaces_the_assets_file_at_its_path() {
    let dir = std::env::temp_dir().join(format!("trpg-content-layer-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let palette = bundle::file("data/palette.ron").unwrap();
    let key = "\"player\": \"";
    let at = palette.find(key).unwrap() + key.len();
    let ours = format!("{}#010203{}", &palette[..at], &palette[at + 7..]);
    std::fs::create_dir_all(dir.join("data")).unwrap();
    std::fs::write(dir.join("data/palette.ron"), &ours).unwrap();
    std::fs::write(dir.join("README.md"), "notes, not content").unwrap();

    let mut files = BTreeMap::new();
    read_folder(&dir, "", &mut files);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(files.keys().collect::<Vec<_>>(), ["data/palette.ron"]);
    let layer = Layer::owned(CONTENT_DISPLAY_ROOT.to_owned(), files).unwrap();
    let layered: &'static Bundle =
        Box::leak(Box::new(Bundle::new(bundle::layers_with(Some(layer)))));

    let (content, shown, other) = bundle::reading(layered, || {
        (
            trpg_content::load_embedded(),
            bundle::display_path("data/palette.ron"),
            bundle::display_path("data/terrain.ron"),
        )
    });
    let content = content.unwrap_or_else(|errors| panic!("{errors}"));
    assert_eq!(content.palette.get("player"), Some([1, 2, 3]));
    assert_eq!(shown, "assets-private/content/data/palette.ron");
    assert_eq!(other, "assets/data/terrain.ron");
    // Outside it, the build's own bundle.
    assert_ne!(
        trpg_content::load_embedded().unwrap().palette.get("player"),
        Some([1, 2, 3])
    );
}
