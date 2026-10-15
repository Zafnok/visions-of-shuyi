//! What the class-choice screen shows, as plain data: no cells, glyphs,
//! colours or positions. The screen builds a [`ClassChangeView`] each
//! frame ([`ClassChangeScreen::view`](super::ClassChangeScreen::view)) and
//! a skin paints it ([`super::glyph::paint`] today); see ADR-0054.

use crate::prelude::*;
use trpg_core::StatValue;

use crate::screens::battle::view::ProgressView;

/// The whole screen, as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassChangeView {
    /// What fills the screen.
    pub body: ClassChangeBody,
    /// The help line: what the keys do here.
    pub help: String,
}

/// The columns, the change's result, or nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassChangeBody {
    /// Picking a class, and the question asked before the change.
    Columns(Box<ColumnsView>),
    /// The change was made: the page on screen, as the battle screen
    /// shows a level-up; `None` once the last has closed.
    Result(Option<Box<ProgressView>>),
    /// Nothing left to show: the screen closes.
    Done,
}

/// The classes the unit can change to, side by side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnsView {
    /// What the screen is called: a promotion or a class change.
    pub title: String,
    /// The unit's name.
    pub unit: String,
    /// The unit's class and level, e.g. `Guard  Lv 12`.
    pub class_level: String,
    /// The seal the focused class takes and how many are in stock; `None`
    /// when no class is offered.
    pub seal: Option<SealView>,
    /// What is said when the unit has no class to change to; `None` when
    /// it has.
    pub none: Option<String>,
    /// The columns on show, left to right (a window of
    /// [`COLUMNS`](super::COLUMNS) over the classes offered).
    pub columns: Vec<ColumnView>,
    /// The mark that there are classes off to the left, when there are.
    pub more_left: Option<String>,
    /// The mark that there are classes off to the right, when there are.
    pub more_right: Option<String>,
    /// The heading over a column's weapons.
    pub weapons_heading: String,
    /// The heading over a column's active skill.
    pub active_heading: String,
    /// The mark after a stat that ends at its hard ceiling (`MAX`).
    pub max_mark: String,
    /// The question asked before the change, while it is open.
    pub question: Option<QuestionView>,
}

/// A seal and how many the stock holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealView {
    /// The seal's name.
    pub name: String,
    /// How many are in stock (none means the change can't be made).
    pub held: u32,
}

/// One class's column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnView {
    /// Whether the cursor is on it.
    pub focused: bool,
    /// Whether the change into it can be made (a refused class can't be
    /// chosen; [`refused`](Self::refused) says why).
    pub offered: bool,
    /// The class's name.
    pub name: String,
    /// Its tier, e.g. `Tier 2`.
    pub tier: String,
    /// For a class change, the class level the unit has saved in it or
    /// that it is new to it; `None` for a promotion.
    pub record: Option<String>,
    /// The Mov row: `Mov  4 → 6`, or `Mov  5` if it stays.
    pub mov: String,
    /// The class's movement type, e.g. `mounted`.
    pub movement: String,
    /// The class's tags that are on, by name.
    pub tags: Vec<String>,
    /// Each weapon kind the class wields, with the unit's rank now and on
    /// entering the class (`Spear    D → C`).
    pub weapons: Vec<String>,
    /// The active skill the class gives, if any.
    pub active: Option<String>,
    /// The unit's stats before and after the change. A refused class
    /// changes nothing: its rows show the stats as they are.
    pub stats: Vec<StatRowView>,
    /// Why the change is refused, if it is.
    pub refused: Option<String>,
}

/// One stat of a column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatRowView {
    /// The stat's short name, e.g. `Str`.
    pub name: String,
    /// The unit's value now.
    pub before: StatValue,
    /// Its value after the change.
    pub after: StatValue,
    /// Whether it ends at (or past) its hard ceiling.
    pub at_max: bool,
}

/// The question asked before the change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionView {
    /// The question, e.g. `Promote Test Knight to Iron Rider?`.
    pub ask: String,
    /// What it costs, e.g. `Uses 1 Tier 2 Seal (1 in stock).`.
    pub cost: String,
    /// The keys that answer it.
    pub keys: String,
}
