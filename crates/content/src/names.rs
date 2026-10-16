//! The names table (`assets/data/names.ron`, ticket 0709): every display
//! name of the story (characters, places, factions, terms) by stable id
//! (`docs/story/names.md`), so a rename is one line. Named characters take
//! their name from it, and dialogue text refers to a name with a token,
//! `{n:<id>}` (or `{N:<id>}` for a capital first letter), filled in by
//! [`Names::substitute`] when the scene is shown.
//!
//! The table and its tokens are the engine's (`shuyi_assets::names`,
//! ticket 1105); this module names the game's file.
use crate::prelude::*;

use crate::bundle;
use crate::error::ContentError;

pub use shuyi_assets::names::{
    NameToken, Names, from_source, is_name_id, literal_form, name_token, with_part,
};

/// Path of the names table inside the asset bundle.
pub const NAMES_PATH: &str = "data/names.ron";

/// Loads and validates the embedded names table.
pub fn load() -> Result<Names, Vec<ContentError>> {
    let display = bundle::display_path(NAMES_PATH);
    let source = bundle::file(NAMES_PATH).ok_or_else(|| {
        vec![ContentError::new(
            &display,
            "file not found in asset bundle",
        )]
    })?;
    let names = from_source(&display, source)?;
    match bundle::test_part(NAMES_PATH) {
        Some((file, part)) => with_part(names, &display, &file, part),
        None => Ok(names),
    }
}

#[cfg(test)]
mod tests;
