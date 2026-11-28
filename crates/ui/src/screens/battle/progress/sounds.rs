//! The EXP bar's and the level-up page's sounds (ticket 0605,
//! `docs/design/audio.md`). Nick picked one for each of five moments:
//!
//! 1. the bar filling ([`EXP_FILL`], as long as the fill),
//! 2. the bar reaching 100, a level up ([`EXP_FULL`]),
//! 3. the level-up page opening ([`LEVEL_UP`], a fanfare the stat rows
//!    wait for: [`ProgressTimings::fanfare`](super::ProgressTimings::fanfare)),
//! 4. each stat that grew appearing ([`LEVEL_UP_STATS`], a note higher for
//!    each),
//! 5. `CLASS MASTERED!` or `Learned: <name>` ([`CLASS_MASTERED`]).
//!
//! [`Progress::take_sounds`] compares the pages' clock with the last time
//! it was asked. Skipping forward skips the sounds forward too (Nick: "unless
//! player skips the animation and then it can skip the sound forward too"):
//! a fill or a fanfare the player hurried past (Confirm held or pressed)
//! stops where its part of the page ends, and the stat rows a Confirm press
//! revealed at once play nothing. While Confirm is held, several rows can
//! appear in one frame; that frame plays one tick, the last row's.

use crate::prelude::*;

use super::{Page, Progress};

/// The EXP bar filling: a whistle rising under chip ticks, 0.6 s.
pub const EXP_FILL: &str = "exp_fill";
/// The bar reaching 100: a sparkle.
pub const EXP_FULL: &str = "exp_full";
/// The level-up page opening: a chip fanfare.
pub const LEVEL_UP: &str = "level_up";
/// The first, second, … stat that grew on a level-up page: a pluck, each
/// a note higher.
pub const LEVEL_UP_STATS: [&str; 7] = [
    "level_up_stat_1",
    "level_up_stat_2",
    "level_up_stat_3",
    "level_up_stat_4",
    "level_up_stat_5",
    "level_up_stat_6",
    "level_up_stat_7",
];
/// A class box with a mastery or something learned: bells and a chord.
pub const CLASS_MASTERED: &str = "class_mastered";

/// A sound the pages ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressSound {
    /// Start this cue as the pages' long sound (the fill, the fanfare),
    /// stopping the one before if it still plays.
    Start(&'static str),
    /// Play this cue once.
    Play(&'static str),
    /// Stop the long sound: the player hurried past its part of the page.
    Stop,
}

/// What [`Progress::take_sounds`] last heard: the page and its clock, and
/// how the player moved the clock since.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct Heard {
    /// The page and its time when last asked; `None` before the first ask.
    at: Option<(usize, f32)>,
    /// Confirm was held while the clock ran.
    pub(super) hurried: bool,
    /// Confirm finished a page's animation at once.
    pub(super) skipped: bool,
}

impl Progress {
    /// The sounds that start since the last call (the first call hears the
    /// first page open). Call it once a frame, after
    /// [`tick`](Self::tick) and [`confirm`](Self::confirm).
    pub fn take_sounds(&mut self) -> Vec<ProgressSound> {
        let heard = core::mem::take(&mut self.heard);
        self.heard.at = Some((self.page, self.t));
        let Some(page) = self.page() else {
            return Vec::new();
        };
        let (from, opened) = match heard.at {
            Some((p, t)) if p == self.page => (t, false),
            _ => (0.0, true),
        };
        let hurried = heard.hurried || heard.skipped;
        // Whether the clock passed the end of the part of the page a long
        // sound goes with, hurried: that sound stops.
        let cut = |end: f32| hurried && (opened || from < end) && self.t >= end;
        let mut out = Vec::new();
        match page {
            Page::Exp(p) => {
                if opened {
                    out.push(ProgressSound::Start(EXP_FILL));
                }
                let before = if opened { p.from } else { self.exp_at(p, from) };
                let per_level = trpg_core::progression::EXP_PER_LEVEL;
                if self.exp_at(p, self.t) / per_level > before / per_level {
                    out.push(ProgressSound::Play(EXP_FULL));
                }
                if cut(self.timings.exp_fill) {
                    out.push(ProgressSound::Stop);
                }
            }
            Page::LevelUp(l) => {
                if opened {
                    out.push(ProgressSound::Start(LEVEL_UP));
                }
                if cut(self.timings.fanfare) {
                    out.push(ProgressSound::Stop);
                }
                let shown = if opened { 0 } else { self.rows_at(page, from) };
                let now = self.rows_at(page, self.t);
                let grown = |row: usize| l.gains.0[row] != 0;
                let newest = (shown..now).rev().find(|&row| grown(row));
                if let Some(row) = newest.filter(|_| !heard.skipped) {
                    let nth = (0..row).filter(|&r| grown(r)).count();
                    out.push(ProgressSound::Play(LEVEL_UP_STATS[nth]));
                }
            }
            Page::Class(c) => {
                if opened && (c.mastered || !c.learned.is_empty()) {
                    out.push(ProgressSound::Play(CLASS_MASTERED));
                }
            }
            Page::Promotion(_) => {}
        }
        out
    }
}
