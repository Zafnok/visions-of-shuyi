//! `cargo xtask test-card`: writes `assets-test/images/test_card.png`, the
//! public test image for sprite items (ticket 0231, ADR-0038).
//!
//! The card is Shuyi's (`shuyi_cli::test_card`, ticket 1117): 16×16, a
//! one-pixel white border around four coloured quadrants, so a flip, a
//! crop and a scale each show. It is generated, not art, and the same code
//! always writes the same pixels.

use std::fs;
use std::path::Path;

use trpg_content::bundle::test_display_path;
use trpg_content::image::TEST_CARD_PATH;

use shuyi_cli::codec::encode_png;
use shuyi_cli::test_card::{SIZE, pixels};

/// Runs the command: writes the card under repo root `root`.
pub fn run(root: &Path) -> Result<String, String> {
    let shown = test_display_path(TEST_CARD_PATH);
    let path = root.join(&shown);
    let png = encode_png(SIZE, SIZE, &pixels())?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    }
    fs::write(&path, png).map_err(|e| format!("writing {}: {e}", path.display()))?;
    Ok(format!("test-card: wrote {shown} ({SIZE}×{SIZE})"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use shuyi_cli::codec::decode_png;

    #[test]
    fn committed_card_is_what_the_tool_makes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = root.join(test_display_path(TEST_CARD_PATH));
        let committed = fs::read(&path).unwrap();
        assert_eq!(
            decode_png(&committed).unwrap(),
            (SIZE, SIZE, pixels()),
            "rerun `cargo xtask test-card`"
        );
    }

    #[test]
    fn run_writes_the_card_and_names_it() {
        let root = std::env::temp_dir().join(format!("xtask-test-card-{}", std::process::id()));
        let summary = run(&root).unwrap();
        assert_eq!(
            summary,
            "test-card: wrote assets-test/images/test_card.png (16×16)"
        );
        let written = fs::read(root.join("assets-test/images/test_card.png")).unwrap();
        assert_eq!(decode_png(&written).unwrap(), (16, 16, pixels()));
        // A root that can't hold directories fails with the path.
        let file = root.join("file");
        fs::write(&file, "x").unwrap();
        let err = run(&file).unwrap_err();
        assert!(err.starts_with("creating "), "{err}");
        // The card's own path taken by a directory fails the write.
        let blocked = root.join("blocked");
        fs::create_dir_all(blocked.join("assets-test/images/test_card.png")).unwrap();
        let err = run(&blocked).unwrap_err();
        assert!(err.starts_with("writing "), "{err}");
        fs::remove_dir_all(&root).unwrap();
    }
}
