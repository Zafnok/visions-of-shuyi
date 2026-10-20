//! What the Preparations screen shows, as plain data: no cells, glyphs,
//! colours or positions. The screen builds a [`PreparationsView`] each
//! frame ([`PreparationsScreen::view`](super::PreparationsScreen::view))
//! and a skin paints it ([`super::glyph::paint`] today); see ADR-0054.

use super::Tab;
use crate::prelude::*;
use crate::widgets::AskView;

/// The whole screen, as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparationsView {
    /// What the screen is called.
    pub heading: String,
    /// The tabs, left to right.
    pub tabs: Vec<TabView>,
    /// Whether the cursor is on the tabs (rather than in the shown tab's
    /// lists).
    pub on_tabs: bool,
    /// What the shown tab holds.
    pub body: PrepBody,
    /// Why the last Confirm did nothing, until the next key.
    pub message: Option<String>,
    /// The "leave Preparations?" question, while it is open. It is shown
    /// in the message's place, its keys on the help line.
    pub leave_question: Option<AskView<bool>>,
    /// The help line: what the keys do here.
    pub help: String,
}

/// One tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabView {
    /// Which tab.
    pub tab: Tab,
    /// Its name.
    pub label: String,
    /// Whether it is the one shown.
    pub shown: bool,
}

/// What the shown tab holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrepBody {
    /// The units and their gear.
    Loadouts(LoadoutsView),
    /// The stock's consumables and the pack.
    Pack(PackView),
    /// Nothing: Confirm opens the Options screen over this one.
    Options,
    /// What the battle starts with, e.g. `4 units · Pack 3/6`.
    Fight(String),
}

/// A titled list with a cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListView<Row> {
    /// The list's title.
    pub title: String,
    /// Every row, in order (a skin shows the ones around the cursor).
    pub rows: Vec<Row>,
    /// The row the cursor is on, or was on when it left the list.
    pub cursor: usize,
    /// What the cursor is doing here.
    pub state: ListState,
}

/// What the cursor is doing in a list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListState {
    /// The cursor isn't here and nothing is chosen.
    Idle,
    /// The cursor went on from this list: its row is the one chosen (the
    /// unit whose slots are shown, the slot whose stock is shown).
    Chosen,
    /// The cursor is in this list.
    Focused,
}

/// The Loadouts tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadoutsView {
    /// The army: the units of this battle, then the ones left out of it.
    pub units: ListView<UnitRowView>,
    /// The chosen unit's gear; `None` for an army of nobody.
    pub gear: Option<GearView>,
}

/// One unit of the army.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitRowView {
    /// Its name.
    pub name: String,
    /// Whether it fights this battle (the others' gear can still be
    /// traded).
    pub deployed: bool,
}

/// The chosen unit's slots and what could go in the chosen one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GearView {
    /// The unit's slots, titled with its name and class.
    pub slots: ListView<SlotRowView>,
    /// The unit's attack speed, and the change the highlighted stock row
    /// would make, e.g. `AS 7 → 6 with Iron Sword`.
    pub speed: Option<String>,
    /// That the unit is left out of this battle, if it is.
    pub benched_note: Option<String>,
    /// What the chosen slot could take: the stock and the other units'.
    pub stock: ListView<ItemRowView>,
}

/// One gear slot of a unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotRowView {
    /// The kind of slot, e.g. `Weapon`.
    pub label: String,
    /// The name of what it holds; `None` for an empty slot.
    pub item: Option<String>,
    /// A weapon's uses left, e.g. `20/25`; empty for anything else.
    pub wear: String,
}

/// One thing a slot could take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemRowView {
    /// Its name (or the row that empties the slot).
    pub name: String,
    /// Its numbers and where it comes from, or why the unit can't use it.
    pub detail: String,
    /// Whether the unit can use it.
    pub usable: bool,
}

/// The Pack tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackView {
    /// The stock's consumables.
    pub spare: ListView<ConsumableRowView>,
    /// The pack, titled with how full it is, e.g. `Pack 3/6`.
    pub pack: ListView<ConsumableRowView>,
}

/// One kind of consumable, in the stock or in the pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumableRowView {
    /// Its name.
    pub name: String,
    /// How many there are.
    pub count: usize,
    /// What it does, e.g. `Restore 10 HP`.
    pub effect: String,
}
