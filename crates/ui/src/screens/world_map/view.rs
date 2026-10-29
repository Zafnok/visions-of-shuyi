//! What the world map screen shows, as plain data: no cells, glyphs or
//! colours. The screen builds a [`WorldMapView`] each frame
//! ([`WorldMapScreen::view`](super::WorldMapScreen::view)) and a skin paints
//! it ([`super::glyph::paint`]); see ADR-0054 and ADR-0099.

use crate::prelude::*;

use trpg_core::Pos;

use crate::widgets::MenuView;

/// What a place shows on the map: its kind's mark (the symbol plate, Nick,
/// 1007). Fixed and random skirmishes come with ticket 1008.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceMark {
    /// A story battle not yet won.
    StoryBattle,
    /// A side quest not yet won.
    SideQuest,
    /// A town.
    Town,
    /// A capital.
    Capital,
    /// A battle won.
    Cleared,
    /// A place to pass through.
    Empty,
}

/// One open place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceView {
    /// Where it is, in cells of the map.
    pub pos: Pos,
    /// Its mark.
    pub mark: PlaceMark,
    /// Its level marker (`Lv7`), for a battle not yet won.
    pub level: Option<String>,
}

/// The panel beside the map: the place under the cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelView {
    /// The place's name.
    pub title: String,
    /// The act and the chapter being played.
    pub act: String,
    /// What the place is (`Story battle`).
    pub kind: String,
    /// More about it, a line each (`Enemy level 7`, `Won`, `You are here`).
    pub lines: Vec<String>,
}

/// A list open over the map: its title and its menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListView {
    /// What it is (`Find`, `Millbrook`).
    pub title: String,
    /// Its rows.
    pub menu: MenuView,
}

/// The whole screen, as it is this frame.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldMapView {
    /// The world map, by id (its ground, size and picture are the map
    /// file's look).
    pub map: String,
    /// The open places.
    pub places: Vec<PlaceView>,
    /// The open roads: each one's cells where it turns, end to end.
    pub roads: Vec<Vec<Pos>>,
    /// Where the army is drawn, in cells of the map.
    pub army: Pos,
    /// The army's label: the lead's two letters.
    pub army_label: String,
    /// Where the cursor is (a place's cell), and its plate is as wide as
    /// the army's when it is on the army.
    pub cursor: Pos,
    /// Whether the cursor is on the army.
    pub cursor_on_army: bool,
    /// How bright the cursor is this frame (it pulses), 0 to 1.
    pub cursor_brightness: f32,
    /// The place under the cursor.
    pub panel: PanelView,
    /// The menu, the Find list or a town's shops, when one is open.
    pub list: Option<ListView>,
    /// The help line.
    pub help: String,
}
