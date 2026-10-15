//! What the EXP bar, level-up and class pages show (ticket 0602, 0603,
//! ADR-0054): the page on screen, as far as it has played. Built by
//! [`progress::view`](super::super::progress::view); the battle screen,
//! the class change and the results screen show it.

use trpg_core::StatValue;

use crate::tips::view::FaceView;

/// The page on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressView {
    /// The EXP bar filling.
    Exp(ExpView),
    /// A level up or a promotion: the unit's face and its stats.
    Gains(GainsView),
    /// Class progress: class levels, mastery, what was learned or stowed.
    Class(ClassProgressView),
}

/// The EXP bar of one unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpView {
    /// The unit's name.
    pub name: String,
    /// What EXP is called.
    pub exp_name: String,
    /// The EXP the bar shows towards the next level (it starts over at
    /// each level).
    pub exp: u32,
    /// The EXP that fills the bar.
    pub max: u32,
}

/// A level up or a promotion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GainsView {
    /// The unit's face, if it has a portrait.
    pub face: Option<FaceView>,
    /// What stands in the face's place without one.
    pub placeholder: String,
    /// The banner: `LEVEL UP!` or `PROMOTED!`.
    pub banner: String,
    /// The unit's name.
    pub name: String,
    /// What changed: `Lv 4 → 5`, or `Guard → Iron Rider`.
    pub line: String,
    /// The stat rows revealed so far, in order.
    pub stats: Vec<StatGainView>,
}

/// One stat row of a level up or a promotion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatGainView {
    /// The stat's short name (`Str`).
    pub name: String,
    /// Its value before.
    pub before: StatValue,
    /// What it gained (0: it stayed).
    pub gain: StatValue,
}

/// A unit's class progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassProgressView {
    /// The unit's name.
    pub name: String,
    /// The class line: `Exile  CL 3 → 4`, `Guard → Mage` or the class.
    pub line: String,
    /// `CLASS MASTERED!`, if it was.
    pub mastered: Option<String>,
    /// A `Learned: <name>` line per skill or spell, then a `To stock:
    /// <item>` line per item sent to the stock.
    pub lines: Vec<String>,
}

#[cfg(test)]
impl ProgressView {
    /// Every line of text the page shows, for tests.
    pub fn texts(&self) -> Vec<String> {
        match self {
            Self::Exp(e) => vec![e.name.clone(), e.exp_name.clone(), e.exp.to_string()],
            Self::Gains(g) => {
                let mut texts = vec![g.banner.clone(), g.name.clone(), g.line.clone()];
                texts.extend(g.face.is_none().then(|| g.placeholder.clone()));
                texts.extend(g.stats.iter().map(|s| {
                    let parts = crate::screens::battle::progress::stat_row_parts;
                    let (row, gain) = parts(&s.name, s.before, s.gain);
                    row + &gain.unwrap_or_default()
                }));
                texts
            }
            Self::Class(c) => {
                let mut texts = vec![c.name.clone(), c.line.clone()];
                texts.extend(c.mastered.clone());
                texts.extend(c.lines.iter().cloned());
                texts
            }
        }
    }
}
