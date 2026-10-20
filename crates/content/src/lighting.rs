//! The lights a battle map can be in (`assets/data/lighting.ron`, ticket
//! 0438, ADR-0058): each a recipe for shading the pictures of the ground.
//! A map names its light in its header's `look`
//! ([`MapLook::light`](crate::MapLook::light)); a battle file may name
//! another. Look data only: the rules never see it. Lights are Shuyi
//! Engine's (`shuyi_assets::lighting`, ticket 1104); this module names the
//! game's file and its default light.
use crate::prelude::*;

pub use shuyi_assets::lighting::{Light, Lighting};

use crate::error::ContentError;
use crate::formats;

/// Path of the lights' file inside the asset bundle.
pub const LIGHTING_PATH: &str = "data/lighting.ron";
/// The light of a map that names none. The file must have it.
pub const NOON: &str = shuyi_assets::grid_map::DEFAULT_LIGHT;

/// Loads the embedded lights' file.
pub fn load() -> Result<Lighting, Vec<ContentError>> {
    formats::load(LIGHTING_PATH)
}

/// Parses lights' file `source`, attributing errors to `file`. Every
/// number must be from 0 to 1, and there must be a [`NOON`]. Reports every
/// problem found.
pub fn from_source(file: &str, source: &str) -> Result<Lighting, Vec<ContentError>> {
    shuyi_assets::lighting::from_source(file, source, NOON)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Render C3's light (ticket 0038): 35% to grey, then × 0.62, 0.56,
    /// 0.46.
    const C3: Light = Light {
        grey: 350,
        tint: [620, 560, 460],
    };

    #[test]
    fn noon_must_exist() {
        let source = r#"{"dusk": (grey: 0.35, tint: (0.62, 0.56, 0.46))}"#;
        let errors = from_source("l.ron", source).err().unwrap_or_default();
        let messages: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
        assert_eq!(
            messages,
            ["no light \"noon\": it is the light of a map that names none"]
        );
    }

    #[test]
    fn the_embedded_file_has_noon_dawn_dusk_and_night() {
        let lighting = load().unwrap_or_else(|e| panic!("{e:?}"));
        // Noon shows the tiles as bought; dusk is render C3.
        assert_eq!(lighting.get(NOON), Some(Light::PLAIN));
        assert_eq!(lighting.get("dusk"), Some(C3));
        let night = lighting.get("night").unwrap();
        // Night is darker than dusk, and bluer.
        assert!(night.tint[0] < C3.tint[0] && night.tint[1] < C3.tint[1]);
        assert!(night.tint[2] > night.tint[0]);
        // Dawn (0733) is lighter than dusk, and redder than it is green.
        let dawn = lighting.get("dawn").unwrap();
        assert!(dawn.grey < C3.grey && dawn.tint[2] > C3.tint[2]);
        assert!(dawn.tint[0] > dawn.tint[1]);
    }
}
