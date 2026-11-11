//! The Playdate's one-bit look as `check --target` reads it (ticket 0140):
//! its palette and the pattern map beside it
//! (`assets/data/palettes/playdate@onebit.ron`, ticket 1253), the map the
//! Playdate's build draws with. `text` stays empty: roles of one colour
//! share their patterns, and the text's colour is shared with roles drawn
//! black on black, so the test `the_text_roles_are_solid_black` keeps text
//! undithered instead.

use shuyi_cli::OneBitSkin;
use trpg_content::palette::PaletteDef;
use trpg_content::playdate_palette::{PLAYDATE_MAP_PATH, PLAYDATE_PALETTE_PATH};

/// The Playdate's skin, `palette` being its palette.
pub(super) fn skin(palette: PaletteDef) -> Result<OneBitSkin, String> {
    let map = trpg_content::playdate_palette::load_playdate_map()
        .map_err(|errors| super::shown(&errors))?;
    let file = trpg_content::bundle::display_path(PLAYDATE_PALETTE_PATH);
    let map_file = trpg_content::bundle::display_path(PLAYDATE_MAP_PATH);
    Ok(OneBitSkin::new(file, palette, map_file, map))
}
