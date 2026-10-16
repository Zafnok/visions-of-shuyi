//! Loading RON assets from the embedded bundle with `file:line:col` errors.
//! The parsing is Shuyi Engine's (`shuyi_assets::ron`, ticket 1103).
use serde::de::DeserializeOwned;

pub use shuyi_assets::ron::parse_ron;

use crate::bundle;
use crate::error::ContentError;

/// Loads and deserializes the RON asset at `path` (relative to `assets/`).
pub fn load_ron<T: DeserializeOwned>(path: &str) -> Result<T, ContentError> {
    shuyi_assets::ron::load_ron(bundle::install(), path)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn missing_file_names_the_path() {
        let err = load_ron::<BTreeMap<String, u8>>("data/nope.ron")
            .err()
            .unwrap_or_else(|| ContentError::new("", "expected an error"));
        assert_eq!(err.file, "assets/data/nope.ron");
        assert_eq!(err.line, None);
        assert!(err.message.contains("not found"));
    }

    #[test]
    fn loads_embedded_file() {
        let m: Result<BTreeMap<String, String>, _> = load_ron("data/palette.ron");
        assert!(m.is_ok_and(|m| m.contains_key("player")));
    }
}
