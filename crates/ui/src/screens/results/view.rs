//! What the results screen shows, as plain data: no cells, glyphs, colours
//! or positions. The screen builds a [`ResultsView`] each frame
//! ([`ResultsScreen::view`](super::ResultsScreen::view)) and a skin paints
//! it ([`super::glyph::paint`] today); see ADR-0054.

use trpg_core::Gold;

use crate::screens::battle::progress::Progress;

/// The whole screen, as it is this frame.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultsView {
    /// What fills the screen.
    pub body: ResultsBody,
    /// The help line: what the keys do here.
    pub help: String,
}

/// The tally or a level-up page.
#[derive(Debug, Clone, PartialEq)]
pub enum ResultsBody {
    /// The gold, the rewind charges, the bonus and every unit's EXP bar.
    Tally(TallyView),
    /// A level-up page of a unit the bonus levelled, as the battle screen
    /// shows it after a combat. (The page is still the battle screen's own
    /// state, which its skin paints; it becomes a view with the rest of the
    /// battle screen, ticket 0242.)
    LevelUp(Box<Progress>),
}

/// The tally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TallyView {
    /// What the screen is called.
    pub heading: String,
    /// The clear gold, if the battle gave any.
    pub gold: Option<GoldView>,
    /// The rewind charges left.
    pub rewinds: RewindsView,
    /// The bonus EXP each unit got; `None` when no unit got any (no bars
    /// are shown then).
    pub bonus: Option<BonusView>,
    /// The deployed units that got the bonus, one EXP bar each.
    pub units: Vec<UnitView>,
}

/// The clear gold line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoldView {
    /// What the line is called.
    pub label: String,
    /// The gold the battle gave.
    pub gained: Gold,
    /// The gold the party has now, as text.
    pub now: String,
}

/// The rewind charges line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewindsView {
    /// What the line is called.
    pub label: String,
    /// How many were left out of how many, as text.
    pub left: String,
}

/// The bonus EXP line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BonusView {
    /// What the line is called.
    pub label: String,
    /// The EXP each unit got.
    pub exp: u32,
}

/// One unit's EXP bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitView {
    /// Its name.
    pub name: String,
    /// Its class's name.
    pub class: String,
    /// Its level as the bar stands, as text (it counts up as the bar
    /// passes a level).
    pub level: String,
    /// What the bar is called.
    pub exp_label: String,
    /// The EXP the bar shows towards the next level (a bar that passed a
    /// level starts over).
    pub exp: u32,
    /// The EXP that fills a bar.
    pub exp_max: u32,
    /// The mark on a unit the bonus has levelled, once the bar passes it.
    pub level_up: Option<String>,
}
