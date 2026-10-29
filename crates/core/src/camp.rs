//! Camp events (ticket 1003, `docs/design/supports.md`, *Rules: camp
//! events*): short, optional, scripted scenes listed at camp once the
//! story reaches their point. They give nothing: no points, stats or
//! items.
//!
//! # Rules
//!
//! - An event is **available** ([`CampEvent::available`]) when every story
//!   flag it names is set, every battle it names has been won, and every
//!   character it names is in the army (alive and recruited).
//! - Viewing an event marks it **seen** ([`Campaign::see_camp_event`]); the
//!   seen events are saved with the campaign. A seen event can be watched
//!   again; only an unseen one is new.
//!
//! The events themselves come from `assets/data/camp.ron`
//! (`trpg_content::camp`); which events exist is the story pipeline's.

use crate::prelude::*;

use crate::campaign::Campaign;
use crate::unit::CharacterId;

/// One camp event, as the camp file describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampEvent {
    /// The event's id, unique among camp events.
    pub id: String,
    /// The list's name for it (English; the language packs translate it).
    pub title: String,
    /// The dialogue scene it plays.
    pub scene: String,
    /// Story flags that must all be set.
    pub flags: Vec<String>,
    /// Battles (by id) that must all have been won.
    pub battles: Vec<String>,
    /// Characters who must all be in the army.
    pub units: Vec<CharacterId>,
}

impl CampEvent {
    /// Whether the event is listed at camp in `campaign` (see the module
    /// docs).
    pub fn available(&self, campaign: &Campaign) -> bool {
        let flag = |f: &String| campaign.flags.get(f) == Some(&true);
        let won = |b: &String| campaign.battles_won.contains(b);
        let here = |c: &CharacterId| campaign.member(c).is_some();
        self.flags.iter().all(flag) && self.battles.iter().all(won) && self.units.iter().all(here)
    }
}

#[cfg(test)]
mod tests;
