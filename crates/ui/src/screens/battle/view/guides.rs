//! What the Guides list shows (ticket 0458, ADR-0054): every topic by
//! title, which is highlighted, and that one's text. Built by
//! [`GuidesScreen::list_view`](super::super::guides::GuidesScreen::list_view).

use crate::prelude::*;

/// The whole Guides screen: the list over the map and the side panel, the
/// help and the debug hint. Built by
/// [`GuidesScreen::view`](super::super::guides::GuidesScreen::view).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuidesScreenView {
    /// The list.
    pub list: GuidesView,
    /// The help line: what the keys do.
    pub help: String,
    /// The debug tools' key, when they are on.
    pub debug_hint: Option<String>,
}

/// The Guides list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuidesView {
    /// Its title.
    pub title: String,
    /// Every topic's title, in list order.
    pub topics: Vec<String>,
    /// Which topic is highlighted.
    pub focus: usize,
    /// The highlighted topic's title, over its text.
    pub heading: String,
    /// The highlighted topic's text, its keys named, one entry per line;
    /// a lesson's checklist follows its text after an empty line.
    pub lines: Vec<String>,
}

#[cfg(test)]
impl GuidesView {
    /// Every line of text the list shows, for tests.
    pub fn texts(&self) -> Vec<String> {
        let mut texts = vec![self.title.clone(), self.heading.clone()];
        texts.extend(self.topics.iter().cloned());
        texts.extend(self.lines.iter().cloned());
        texts
    }
}
