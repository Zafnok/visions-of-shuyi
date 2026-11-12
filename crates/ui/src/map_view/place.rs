//! A map shown behind something else (ticket 0732): the place a
//! conversation is drawn over (`docs/design/look-and-feel.md`, *Places
//! behind conversations*). A [`Place`] is the whole map, its look and the
//! units on it; [`Place::scene`] is the part of it a screen shows, centred
//! on a tile, for the map skin to paint like the battle map.

use crate::prelude::*;

use trpg_content::{Content, MapDef, MapLook};
use trpg_core::{BattleDef, BattleState, Faction, Grid, Pos, TerrainId};

use super::scene::{MapScene, UnitView};
use crate::glyph_buffer::Rect;
use crate::screen::Ctx;
use crate::screens::battle::camera::Camera;

/// How many tiles, across × down, `ctx`'s map skin fits on the whole
/// screen: the size of a place shown behind everything.
pub fn screen_tiles(ctx: &Ctx) -> (i32, i32) {
    let console = ctx.console;
    let screen = Rect::new(0, 0, i32::from(console.cols), i32::from(console.rows));
    ctx.game.map_skin.view_tiles(screen)
}

/// Who is left standing on the field of a battle that ended as `state`
/// (Nick, 0738: "show who's left standing"): the player's units still on
/// it, where they stood, on no side ([`UnitView::sided`]) as everyone on a
/// conversation's place is, facing the camera. Not the enemies (the
/// battle was won) and not a unit down at 0 HP.
pub fn left_standing(state: &BattleState) -> Vec<UnitView> {
    let player = state
        .units()
        .iter()
        .filter(|u| u.faction == Faction::Player);
    player.map(|u| UnitView::of(u).on_no_side()).collect()
}

/// A whole map to show behind something: its terrain, how it looks and the
/// units standing on it.
#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    /// The terrain of each tile.
    pub tiles: Grid<TerrainId>,
    /// How the map looks.
    pub look: MapLook,
    /// The units on it, where they stand.
    pub units: Vec<UnitView>,
}

impl Place {
    /// The map of map file `def`, with nobody on it.
    pub fn of_map(def: &MapDef) -> Self {
        Self {
            tiles: def.map.tiles.clone(),
            look: def.look.clone(),
            units: Vec::new(),
        }
    }

    /// The map of battle `def` as it looks in `content` (in the battle's
    /// own light), with nobody on it: the field before the battle (or after
    /// it, with [`left_standing`] on it).
    pub fn of_battle(content: &Content, def: &BattleDef) -> Self {
        Self {
            tiles: def.map.tiles.clone(),
            look: content.battle_look(def),
            units: Vec::new(),
        }
    }

    /// The part of the map a view of `view` tiles (across, down) shows
    /// with `centre` as near its middle as the map allows (a map smaller
    /// than the view is centred in it), and the units on that part. No
    /// cursor, ranges or path.
    pub fn scene(&self, centre: Pos, view: (i32, i32)) -> MapScene {
        let (w, h) = (self.tiles.width(), self.tiles.height());
        let origin = Camera::centred_on(centre, w, h, view).origin;
        let mut scene = MapScene::new(origin, view);
        scene.set_terrain(|pos| self.tiles.get(pos).copied());
        scene.look.clone_from(&self.look);
        for unit in &self.units {
            scene.push_unit(unit.clone());
        }
        scene
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: i32, y: i32) -> Pos {
        Pos::new(x, y)
    }

    /// On the test files' `test_small` map (14 × 8).
    #[test]
    fn a_place_is_centred_on_its_tile_as_far_as_the_map_allows() {
        let content = trpg_content::load_embedded().unwrap();
        let place = Place::of_map(&content.maps["test_small"]);
        assert_eq!((place.tiles.width(), place.tiles.height()), (14, 8));
        assert_eq!(place.units, [] as [crate::map_view::scene::UnitView; 0]);
        // A view smaller than the map: the tile in the middle.
        let scene = place.scene(p(7, 4), (10, 6));
        assert_eq!((scene.origin, scene.size), (p(2, 1), (10, 6)));
        assert_eq!(
            scene.tile(p(7, 4)).and_then(|t| t.terrain),
            place.tiles.get(p(7, 4)).copied()
        );
        // Near an edge: the view stops at it.
        assert_eq!(place.scene(p(0, 7), (10, 6)).origin, p(0, 2));
        assert_eq!(place.scene(p(13, 0), (10, 6)).origin, p(4, 0));
        // A view bigger than the map: the map in its middle, off the map
        // around it.
        let wide = place.scene(p(3, 6), (50, 32));
        assert_eq!(wide.origin, p(-18, -12));
        assert_eq!(wide.tile(p(-1, 0)).map(|t| t.terrain), Some(None));
        assert!(wide.tile(p(0, 0)).is_some_and(|t| t.terrain.is_some()));
        assert_eq!(wide.look, content.maps["test_small"].look);
    }

    /// On the test battle `test`: its brigand is on the part shown, its
    /// party off it.
    #[test]
    fn a_battles_place_is_its_field_in_its_light_and_shows_its_units() {
        let content = trpg_content::load_embedded().unwrap();
        let def = &content.battles["test"];
        let mut place = Place::of_battle(&content, def);
        assert_eq!(place.tiles, def.map.tiles);
        assert_eq!(place.look, content.battle_look(def));
        let campaign = trpg_content::battle_campaign(
            &content,
            def,
            trpg_core::GameMode::Classic,
            trpg_core::LeadProfile::new("Lead", trpg_core::LeadGender::Male),
        );
        let setup = campaign.battle_setup(def, &content.tables());
        place.units = setup.units.iter().map(UnitView::of).collect();
        let scene = place.scene(p(12, 2), (10, 6));
        // Only the units on the part shown.
        assert_ne!(scene.units, [] as [crate::map_view::scene::UnitView; 0]);
        assert!(scene.units.len() < place.units.len());
        assert!(scene.units.iter().all(|u| scene.contains(u.pos)));
    }
}
