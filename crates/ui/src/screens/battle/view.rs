//! What the battle screen shows this frame (ADR-0054): its map's
//! [`MapScene`] (ADR-0038) and everything around and over it, as plain
//! data in the player's language. The screen builds a [`BattleView`]
//! ([`BattleScreen::view`](super::BattleScreen::view)) and a skin paints it
//! ([`super::glyph::paint`] today). Nothing here says where a thing goes or
//! how it looks.
//!
//! The forecast, the unit info screen, the fight box, the EXP bar and
//! level-up pages and the rewind screen are still painted from the
//! battle's own state by their old draw functions: [`OldParts`] hands the
//! skin that state until ticket 0248 gives them views of their own.

use trpg_core::{BattleState, Faction, Pos, StatValue, Unit};

use super::HealPopup;
use super::attack::Targeting;
use super::banner::BannerKind;
use super::playback::Playback;
use super::progress::Progress;
use super::rewind::RewindScreen;
use super::stage::view::StageView;
use crate::map_view::MapScene;
use crate::tips::TipView;
use crate::widgets::menu::MenuView;

/// The whole battle screen, as it is this frame.
#[derive(Debug, Clone)]
pub struct BattleView<'a> {
    /// What is on the visible map (ADR-0038): the map skin paints it.
    pub scene: MapScene,
    /// The menu open, if any: a unit's action menu or a list it opened,
    /// the map menu, or the unit list.
    pub menu: Option<BattleMenuView>,
    /// The heal and burn numbers floating over units.
    pub popups: Vec<HealPopup>,
    /// The side panel: the terrain and unit under the cursor. `None` while
    /// the forecast fills it ([`OldParts::forecast`]) and on the rewind
    /// screen.
    pub panel: Option<PanelView>,
    /// The combat stage (ticket 0413): a fight playing back with combat
    /// animations on. It covers the map and the side panel.
    pub stage: Option<StageView>,
    /// The box over the map: the objective page or a yes/no question.
    pub dialog: Option<DialogView>,
    /// The battle notes box shown at the start of the battle.
    pub notes: Option<NotesView>,
    /// Where the units the battle notes are about are drawn, while the
    /// notes are on screen (in their box or on the objective page): a box
    /// listing the notes keeps off their rows. Empty otherwise.
    pub noted: Vec<Pos>,
    /// The phase or outcome banner on screen.
    pub banner: Option<BannerView>,
    /// The one-time tip on screen (ticket 0406).
    pub tip: Option<TipView>,
    /// The line over the help: a combat's message, the preview of what is
    /// being aimed, or the toast.
    pub message: Option<MessageView>,
    /// The toggles' state (zoom, danger zone, auto-end), each with its
    /// key. Empty on the rewind screen, which has no such line.
    pub status: String,
    /// The help line: what the keys do now.
    pub help: String,
    /// The debug tools' key, when they are on.
    pub debug_hint: Option<String>,
    /// The parts still painted from the battle's state.
    pub old: OldParts<'a>,
}

/// The parts of the screen that don't have a view yet (ticket 0248): the
/// battle's state, for their old draw functions to paint.
#[derive(Debug, Clone, Copy)]
pub struct OldParts<'a> {
    /// The battle, as those functions read it.
    pub state: &'a BattleState,
    /// The attack forecast, while an attack or a spell is aimed at an
    /// enemy: it fills the side panel.
    pub forecast: Option<&'a Targeting>,
    /// The unit whose info screen is open.
    pub info: Option<&'a Unit>,
    /// The fight playing back in the plain box (combat animations off).
    pub fight_box: Option<&'a Playback>,
    /// The EXP bar or level-up page on screen.
    pub progress: Option<&'a Progress>,
    /// The rewind screen, while it is open: it replaces everything but the
    /// map (as it was before the action it is on), the help and the debug
    /// hint.
    pub rewind: Option<&'a RewindScreen>,
}

/// A menu and where it opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleMenuView {
    /// Its items and focus.
    pub menu: MenuView,
    /// Where it opens.
    pub place: MenuPlace,
    /// What it shows in its top border, if anything.
    pub header: Option<MenuHeader>,
}

/// Where a menu opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuPlace {
    /// Beside this tile: the unit's (where it will stand) or the cursor's.
    Beside(Pos),
    /// In the middle of the map.
    Centred,
}

/// What a menu shows in its top border: the weapon the focused art would
/// be used with and its durability, or the pack's size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuHeader {
    /// The weapon's name, or the pack's size (`Pack 3/10`).
    pub name: String,
    /// The weapon's durability, if it has one.
    pub uses: Option<UsesView>,
}

/// What is left of a weapon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsesView {
    /// As shown: `20/20`, or `broken`.
    pub text: String,
    /// Whether nothing is left (the weapon is broken).
    pub spent: bool,
}

/// The side panel's hover info: the terrain under the cursor, then the
/// unit on it, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelView {
    /// Whether a unit is selected (the panel is marked while one is).
    pub selected: bool,
    /// The terrain under the cursor.
    pub terrain: Option<TerrainView>,
    /// The unit drawn under the cursor.
    pub unit: Option<PanelUnitView>,
}

/// A tile's terrain in the side panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainView {
    /// Its name.
    pub name: String,
    /// Its defense and avoid bonuses (`Def +1  Avo +20`).
    pub bonuses: String,
    /// What it heals at the start of a turn, if it heals.
    pub heals: Option<String>,
}

/// A unit in the side panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelUnitView {
    /// Its name.
    pub name: String,
    /// Its faction.
    pub faction: Faction,
    /// Its class and level (`Lord  Lv 1`).
    pub class_level: String,
    /// The name of the HP stat.
    pub hp_name: String,
    /// Its HP now.
    pub hp: StatValue,
    /// Its max HP.
    pub max: StatValue,
    /// The name of its faction.
    pub faction_name: String,
}

/// A box over the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogView {
    /// The map menu's `Objective` page.
    Objective(ObjectiveView),
    /// A yes/no question: end the turn, restart or suspend the battle.
    Question(QuestionView),
}

/// The `Objective` page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectiveView {
    /// Its title.
    pub title: String,
    /// What wins the battle.
    pub objective: String,
    /// The turn, with the limit if there is one.
    pub turn: String,
    /// The battle notes under a heading, if the battle has any.
    pub notes: Option<NotesView>,
}

/// The battle notes, under a title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotesView {
    /// The box's title, or the heading over the notes on the objective
    /// page.
    pub title: String,
    /// The notes, wrapped: one string per line.
    pub lines: Vec<String>,
}

/// A yes/no question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionView {
    /// What is asked.
    pub question: String,
    /// How to answer, with the keys.
    pub answers: String,
}

/// A phase or outcome banner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerView {
    /// What it announces.
    pub kind: BannerKind,
    /// The big line (`PLAYER PHASE`, `VICTORY`).
    pub title: String,
    /// The line beneath (the turn), empty for an outcome.
    pub subtitle: String,
}

/// The line over the help.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageView {
    /// What it says.
    pub text: String,
    /// What it is.
    pub kind: MessageKind,
}

/// What the message line says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    /// What a combat says or what the thing aimed will do.
    Said,
    /// A short message for a moment (a toggle flipped).
    Toast,
}

#[cfg(test)]
impl BattleView<'_> {
    /// Every line of text the view shows (not the parts still painted from
    /// the battle's state, [`OldParts`]), for tests asking whether the
    /// screen shows something.
    pub fn texts(&self) -> Vec<&str> {
        let mut texts: Vec<&str> = vec![&self.status, &self.help];
        if let Some(m) = &self.menu {
            texts.extend(m.menu.items.iter().map(|i| i.label.as_str()));
            if let Some(h) = &m.header {
                texts.push(&h.name);
                texts.extend(h.uses.as_ref().map(|u| u.text.as_str()));
            }
        }
        if let Some(p) = &self.panel {
            if let Some(t) = &p.terrain {
                texts.extend([t.name.as_str(), &t.bonuses]);
                texts.extend(t.heals.as_deref());
            }
            if let Some(u) = &p.unit {
                texts.extend([u.name.as_str(), &u.class_level, &u.faction_name]);
            }
        }
        match &self.dialog {
            Some(DialogView::Objective(o)) => {
                texts.extend([o.title.as_str(), &o.objective, &o.turn]);
                if let Some(n) = &o.notes {
                    texts.extend(note_texts(n));
                }
            }
            Some(DialogView::Question(q)) => texts.extend([q.question.as_str(), &q.answers]),
            None => {}
        }
        if let Some(n) = &self.notes {
            texts.extend(note_texts(n));
        }
        if let Some(b) = &self.banner {
            texts.extend([b.title.as_str(), &b.subtitle]);
        }
        if let Some(t) = &self.tip {
            texts.extend([t.title.as_str(), &t.heading, &t.close]);
            texts.extend(t.lines.iter().map(String::as_str));
        }
        texts.extend(self.message.as_ref().map(|m| m.text.as_str()));
        texts
    }

    /// Whether some text of the view ([`texts`](Self::texts)) contains
    /// `text`.
    pub fn shows(&self, text: &str) -> bool {
        self.texts().iter().any(|t| t.contains(text))
    }
}

/// The title and lines of `notes`.
#[cfg(test)]
fn note_texts(notes: &NotesView) -> impl Iterator<Item = &str> {
    std::iter::once(notes.title.as_str()).chain(notes.lines.iter().map(String::as_str))
}
