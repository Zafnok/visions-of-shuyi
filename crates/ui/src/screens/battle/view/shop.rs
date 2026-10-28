//! What the shop screen shows (ticket 0409, ADR-0054): the shop and the
//! party's gold, its tabs, the lines of the tab shown and which is
//! highlighted. Built by
//! [`ShopScreen::view`](super::super::shop::ShopScreen::view).

use crate::prelude::*;

/// The whole shop screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopScreenView {
    /// The shop's name (`Armory`).
    pub shop: String,
    /// The party's gold after what was chosen (`Gold 1 250`).
    pub gold: String,
    /// The unit shopping.
    pub unit: String,
    /// The tabs' names.
    pub tabs: Vec<String>,
    /// Which tab is shown.
    pub tab: usize,
    /// The tab's lines.
    pub lines: Vec<ShopLineView>,
    /// Which line is highlighted.
    pub focus: usize,
    /// What a tab with no lines says.
    pub empty: String,
    /// The help line: what the keys do.
    pub help: String,
    /// The debug tools' key, when they are on.
    pub debug_hint: Option<String>,
}

/// One line of a tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopLineView {
    /// The item, with how many there are if more than one.
    pub name: String,
    /// Its price, sell price or repair cost.
    pub gold: String,
    /// What the item is (`Mt 5  Hit 90 …`), or the repair (`12/20 → 20/20`).
    pub detail: String,
    /// Whether it can be chosen; dimmed if not.
    pub enabled: bool,
}

#[cfg(test)]
impl ShopScreenView {
    /// Every line of text the screen shows, for tests.
    pub fn texts(&self) -> Vec<String> {
        let mut texts = vec![self.shop.clone(), self.gold.clone(), self.unit.clone()];
        texts.extend(self.tabs.iter().cloned());
        for l in &self.lines {
            texts.extend([l.name.clone(), l.gold.clone(), l.detail.clone()]);
        }
        texts.push(self.help.clone());
        texts
    }
}
