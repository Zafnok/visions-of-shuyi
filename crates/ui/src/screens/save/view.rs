//! What the save screens show, as plain data: no cells, glyphs, colours or
//! positions. Each screen builds its view each frame
//! ([`SavePromptScreen::view`](super::SavePromptScreen::view),
//! [`SlotPickerScreen::view`](super::SlotPickerScreen::view)) and a skin
//! paints it ([`super::glyph`] today); see ADR-0054.

use trpg_core::GameMode;

use crate::widgets::MenuView;

/// "Save your progress?" with its answers, as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavePromptView {
    /// The question.
    pub question: String,
    /// `Yes` and `No`.
    pub menu: MenuView,
    /// The help line: what the keys do here.
    pub help: String,
}

/// The slot picker, as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotPickerView {
    /// What the picker is for: saving or loading.
    pub title: String,
    /// The names of the columns.
    pub columns: SlotColumnsView,
    /// The slots on show, in order: a window of the whole list that has the
    /// focused slot in it.
    pub slots: Vec<SlotView>,
    /// Which slots are on show out of how many, e.g. `6-25 of 30`.
    pub range: String,
    /// Why something didn't work, until the next key.
    pub message: Option<String>,
    /// The "Overwrite slot N?" question, while it is open.
    pub question: Option<OverwriteView>,
    /// The help line: what the keys do here.
    pub help: String,
}

/// The names of the slot columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotColumnsView {
    /// The slot number's.
    pub slot: String,
    /// The chapter's.
    pub chapter: String,
    /// The game mode's.
    pub mode: String,
    /// The army size's.
    pub army: String,
    /// The playtime's.
    pub time: String,
}

/// One slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotView {
    /// Its number (1 to the number of slots).
    pub number: usize,
    /// Whether the cursor is on it.
    pub focused: bool,
    /// What it holds.
    pub entry: SlotEntryView,
}

/// What a slot holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotEntryView {
    /// Nothing; the text says so.
    Empty(String),
    /// A save that can be loaded.
    Saved {
        /// The chapter it goes on with (or `<title> (cleared)`).
        chapter: String,
        /// Classic or Casual.
        mode: GameMode,
        /// The mode's name.
        mode_name: String,
        /// The army's size as text, e.g. `3 units`.
        army: String,
        /// The time played as text, e.g. `1:02:05`.
        playtime: String,
    },
    /// A save that can't be read; the text says why.
    Unreadable(String),
}

/// "Overwrite slot N?" with the keys that answer it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverwriteView {
    /// The question.
    pub question: String,
    /// What the keys answer, e.g. `f yes / d no`.
    pub answers: String,
}
