//! The battle map's view (ADR-0038): a screen says *what* is on the visible
//! map as a [`MapScene`] (plain data), and a [`MapSkin`] says how it looks.
//! The [glyph skin](MapSkin::glyph) is what a build without the bought art
//! shows; a [sprite skin](MapSkin::sprite) paints from a tileset file, and
//! the game starts with the one of the bought art when it has it
//! (ADR-0049).
//!
//! The scene, the ground, the ranges, the path, the cursor and the threat
//! arrows are the tactics kit's (`shuyi_ui::grid`, tickets 1115 and 2313,
//! ADR-0088): this game's skins pair the kit's with the painters of its
//! units (their letters, HP bars, effect marks and pictures) and its ranges
//! and colors. The Playdate's skin is the kit's glyph skin in a one-bit
//! style.
//!
//! Anything new shown on the map (a village, a spell's flash) is added to
//! the scene and painted by every skin; it is never drawn straight into the
//! buffer by a screen.

pub mod glyph;
pub mod place;
pub mod playdate;
pub mod scene;
pub mod skin;
pub mod sprite;

/// The kit's corner rule (ticket 1115: `shuyi_ui::grid::corners`).
pub use shuyi_ui::grid::corners;
/// Where a scene's tiles go in pixels (ticket 1115:
/// `shuyi_ui::grid::layout`).
pub use shuyi_ui::grid::layout as grid;
/// The path's arrow (ticket 1115: `shuyi_ui::grid::path`).
pub use shuyi_ui::grid::path;
/// The threat arrows (ticket 1115: `shuyi_ui::grid::threat`).
pub use shuyi_ui::grid::threat;

use trpg_content::Content;

use crate::screen::SkinSet;
use crate::settings::MapLook;

pub use place::{Place, screen_tiles};
pub use scene::{
    CursorStyle, CursorView, Facing, MapScene, RangeName, STANDING_FRAME, TerrainLooks, ThreatView,
    TileView, UnitEffects, UnitView, range,
};
pub use skin::{MapSkin, Units};

/// The tileset the game paints its battle maps with when it has it: the
/// bought art's, which only a build with the private assets has
/// (ADR-0040).
pub const GAME_TILESET: &str = "tiny_tales";

/// The generated test tileset with terrain tiles (`cargo xtask
/// test-tileset`).
pub const TEST_TILESET: &str = "test";

/// The generated test tileset of unit sheets, with no terrain tiles
/// (`cargo xtask test-tileset`).
pub const TEST_UNITS_TILESET: &str = "test_units";

/// The skin the game starts with: the [sprite skin](MapSkin::sprite) of
/// the [`GAME_TILESET`] if `content` has it, else the
/// [glyph skin](MapSkin::glyph).
pub fn default_skin(content: &Content) -> MapSkin {
    skin_named(content, GAME_TILESET).unwrap_or_else(MapSkin::glyph)
}

/// The skin for the player's map look (ticket 0824): with
/// [`MapLook::Pictures`] and the [`GAME_TILESET`] in `content`, its picture
/// skin ([`default_skin`]); otherwise the [glyph skin](MapSkin::glyph),
/// whatever the look.
pub fn skin_for_look(look: MapLook, content: &Content) -> MapSkin {
    match look {
        MapLook::Pictures => default_skin(content),
        MapLook::Glyphs => MapSkin::glyph(),
    }
}

/// The map skin of skin set `set` for the player's map look: on the PC
/// ([`SkinSet::Glyph`]) [`skin_for_look`]'s; on the Playdate its
/// [one-bit skin](MapSkin::playdate), whatever the look (its bought art
/// in one bit is ticket 0067's question).
pub fn skin_for(set: SkinSet, look: MapLook, content: &Content) -> MapSkin {
    match set {
        SkinSet::Glyph => skin_for_look(look, content),
        SkinSet::Playdate => MapSkin::playdate(),
    }
}

/// Whether `content` has pictures to paint the map with, so that the map
/// look is a choice ([`GAME_TILESET`] present).
pub fn has_pictures(content: &Content) -> bool {
    content.tilesets.contains_key(GAME_TILESET)
}

/// The skin called `name`: `"glyph"`; `"playdate"`; `"sprite"` and `"sprite_units"`,
/// painting with the [`TEST_TILESET`] and the [`TEST_UNITS_TILESET`]; or
/// the id of any tileset of `content`. `None` for any other name, or
/// without that tileset. For the debug menu and tests.
pub fn skin_named(content: &Content, name: &str) -> Option<MapSkin> {
    let id = match name {
        "glyph" => return Some(MapSkin::glyph()),
        "playdate" => return Some(MapSkin::playdate()),
        "sprite" => TEST_TILESET,
        "sprite_units" => TEST_UNITS_TILESET,
        id => id,
    };
    let tileset = content.tilesets.get(id)?.clone();
    Some(MapSkin::sprite(tileset))
}

/// The skin after `current` in the debug menu's round: the glyph skin,
/// then each tileset of `content` by id, then the glyph skin again.
pub fn next_skin(content: &Content, current: &MapSkin) -> MapSkin {
    let mut ids = content.tilesets.keys();
    if let Some(id) = current.tileset_id() {
        // Past the current one (all of them, if the content lacks it).
        ids.by_ref().take_while(|next| next.as_str() != id).count();
    }
    ids.next()
        .and_then(|id| skin_named(content, id))
        .unwrap_or_else(MapSkin::glyph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skins_are_found_by_name() {
        let content = trpg_content::load_embedded().unwrap();
        let found = |name| skin_named(&content, name).map(|s| (s.name(), s.tileset_id().is_none()));
        assert_eq!(found("glyph"), Some(("glyph", true)));
        let tileset = |name| {
            let skin = skin_named(&content, name)?;
            Some((skin.name(), skin.tileset_id()?.to_owned()))
        };
        assert_eq!(tileset("sprite"), Some(("sprite", "test".to_owned())));
        assert_eq!(tileset("test"), tileset("sprite"));
        let units = Some(("sprite_units", "test_units".to_owned()));
        assert_eq!(tileset("sprite_units"), units);
        assert_eq!(tileset("test_units"), units);
        // The tileset with layers and looks paints the whole map too.
        let auto = Some(("sprite", "test_auto".to_owned()));
        assert_eq!(tileset("test_auto"), auto);
        assert!(skin_named(&content, "ascii").is_none());
        let mut bare = content;
        bare.tilesets.clear();
        assert!(skin_named(&bare, "sprite").is_none());
        assert!(skin_named(&bare, "sprite_units").is_none());
        assert!(skin_named(&bare, "glyph").is_some());
    }

    #[test]
    fn the_map_look_picks_the_skin() {
        let mut content = trpg_content::load_embedded().unwrap();
        let name = |look, content: &Content| skin_for_look(look, content).name();
        // Without the bought art, every look is glyphs.
        assert!(!has_pictures(&content));
        assert_eq!(name(MapLook::Pictures, &content), "glyph");
        assert_eq!(name(MapLook::Glyphs, &content), "glyph");
        let mut tileset = content.tilesets[TEST_UNITS_TILESET].clone();
        tileset.id = GAME_TILESET.to_owned();
        content.tilesets.insert(GAME_TILESET.to_owned(), tileset);
        assert!(has_pictures(&content));
        assert_eq!(name(MapLook::Pictures, &content), "sprite_units");
        assert_eq!(name(MapLook::Glyphs, &content), "glyph");
    }

    #[test]
    fn the_skin_set_picks_the_map_skin() {
        let mut content = trpg_content::load_embedded().unwrap();
        let mut tileset = content.tilesets[TEST_UNITS_TILESET].clone();
        tileset.id = GAME_TILESET.to_owned();
        content.tilesets.insert(GAME_TILESET.to_owned(), tileset);
        let name = |set, look| skin_for(set, look, &content).name();
        assert_eq!(name(SkinSet::Glyph, MapLook::Pictures), "sprite_units");
        assert_eq!(name(SkinSet::Glyph, MapLook::Glyphs), "glyph");
        assert_eq!(name(SkinSet::Playdate, MapLook::Pictures), "playdate");
        assert_eq!(name(SkinSet::Playdate, MapLook::Glyphs), "playdate");
        let found = skin_named(&content, "playdate").map(|s| s.name());
        assert_eq!(found, Some("playdate"));
    }

    #[test]
    fn the_game_starts_with_the_bought_arts_skin_only_when_it_has_it() {
        let mut content = trpg_content::load_embedded().unwrap();
        // A build without the private assets: glyphs.
        assert!(!content.tilesets.contains_key(GAME_TILESET));
        let skin = default_skin(&content);
        assert_eq!((skin.name(), skin.tileset_id()), ("glyph", None));
        // With a tileset of that id: its skin.
        let mut tileset = content.tilesets[TEST_UNITS_TILESET].clone();
        tileset.id = GAME_TILESET.to_owned();
        content.tilesets.insert(GAME_TILESET.to_owned(), tileset);
        let skin = default_skin(&content);
        assert_eq!(skin.name(), "sprite_units");
        assert_eq!(skin.tileset_id(), Some(GAME_TILESET));
    }

    #[test]
    fn the_debug_round_goes_through_every_tileset_and_back_to_glyphs() {
        let mut content = trpg_content::load_embedded().unwrap();
        let mut skin = MapSkin::glyph();
        let mut round = Vec::new();
        for _ in 0..5 {
            skin = next_skin(&content, &skin);
            round.push(skin.tileset_id().map(str::to_owned));
        }
        let id = |s: &str| Some(s.to_owned());
        let tilesets = [id("test"), id("test_auto"), id("test_units")];
        assert_eq!(round[..3], tilesets);
        assert_eq!(round[3..], [None, id("test")]);
        // A skin whose tileset is gone: back to glyphs.
        content.tilesets.clear();
        assert_eq!(next_skin(&content, &skin).name(), "glyph");
        assert_eq!(next_skin(&content, &MapSkin::glyph()).name(), "glyph");
    }
}
