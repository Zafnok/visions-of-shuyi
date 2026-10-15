//! The results of a won battle (ticket 0810,
//! `docs/design/death-and-difficulty.md`, *Results screen*): one screen with
//! the clear gold, the rewind charges left and the EXP bonus they gave, and
//! every deployed unit's EXP bar filling at once; then the level-up pages
//! (0602) of the units the bonus levelled, one at a time.
//!
//! Everything shown comes from `core`'s [`BattleRewards`]; this only times
//! and draws it. Confirm or Cancel fills the bars at once, and holding
//! Confirm fills them [`ProgressTimings::fast`] times as fast; once they are
//! full a press goes on. Each level-up page still waits for its own press
//! (Nick), as after a combat.
//!
//! **The look is a skin** (ADR-0054): this module decides what the screen
//! shows and does, and says it as a [`ResultsView`]
//! ([`ResultsScreen::view`]); [`glyph::paint`] draws that view.

pub mod glyph;
pub mod view;

use crate::prelude::*;
pub use view::{BonusView, GoldView, ResultsBody, ResultsView, RewindsView, TallyView, UnitView};

use trpg_core::progression::EXP_PER_LEVEL;
use trpg_core::{BattleRewards, BattleState, Gold, Level};

use super::battle::progress::{self, PROGRESS_TIMINGS, Progress, ProgressTimings, playing_help};
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::screen::{Ctx, FrameInput, Screen, Transition};
use crate::words::Words;

/// The key of the screen's heading, on the panel's border, in the
/// language files ([`Ctx::text`]).
pub const HEADING: &str = "results.heading";
/// The key of the gold line.
pub const GOLD_LABEL: &str = "results.gold";
/// The key of the rewind charges line.
pub const REWINDS_LABEL: &str = "results.rewinds";
/// The key of the bonus line.
pub const BONUS_LABEL: &str = "results.bonus";
/// The key of the mark on a unit the bonus levelled.
pub const LEVEL_UP: &str = "results.level_up";

/// Seconds before the bars start to fill. *Tunable.*
pub const INTRO_S: f32 = 0.5;
/// Seconds the bars take to fill. *Tunable.*
pub const FILL_S: f32 = 1.0;

/// One deployed unit's row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Its name.
    pub name: String,
    /// Its class's name.
    pub class: String,
    /// Its level before the bonus.
    pub level: Level,
    /// Its EXP towards the next level before the bonus.
    pub exp: u32,
    /// The bonus EXP it got.
    pub gained: u32,
}

/// The results of a won battle. Pops once the player has seen them and
/// every level-up page.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultsScreen {
    clear_gold: Gold,
    gold: Gold,
    unused: u8,
    charges: u8,
    bonus: u32,
    rows: Vec<Row>,
    /// Seconds since the screen opened.
    t: f32,
    timings: ProgressTimings,
    /// Seconds before the bars start to fill, and seconds they take.
    intro_s: f32,
    fill_s: f32,
    /// The level-up pages still to show after the bars.
    pages: Option<Progress>,
    /// Whether the bars are done with and the pages are up.
    paging: bool,
}

impl ResultsScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "results";

    /// The results `rewards` of the won battle `state`, which began with
    /// `charges` rewind charges; `gold` is the party's gold now.
    pub fn new(
        rewards: &BattleRewards,
        (state, words): (&BattleState, Words<'_>),
        charges: u8,
        gold: Gold,
    ) -> Self {
        let class = |u: &trpg_core::Unit| words.class_of(&u.class, state.classes()).to_owned();
        // No unused charge, no bonus: no bars to show.
        let deployed = rewards.deployed.iter().filter(|_| rewards.bonus_exp > 0);
        let rows = deployed
            .map(|u| Row {
                name: words.unit(u).to_owned(),
                class: class(u),
                level: u.level,
                exp: u.exp,
                gained: rewards.exp_gained(u.id),
            })
            .collect();
        let timings = PROGRESS_TIMINGS;
        let pages = Progress::new(&rewards.events, &rewards.deployed, (state, words), timings)
            .and_then(Progress::without_exp_bars);
        Self {
            clear_gold: rewards.clear_gold,
            gold,
            unused: rewards.unused_charges,
            charges,
            bonus: rewards.bonus_exp,
            rows,
            t: 0.0,
            timings,
            intro_s: INTRO_S,
            fill_s: FILL_S,
            pages,
            paging: false,
        }
    }

    /// The same screen with its bars waiting `intro_s` seconds and filling
    /// over `fill_s` (the game uses [`INTRO_S`] and [`FILL_S`]).
    #[must_use]
    pub fn with_timings(mut self, intro_s: f32, fill_s: f32) -> Self {
        (self.intro_s, self.fill_s) = (intro_s, fill_s);
        self
    }

    /// Seconds from the screen opening to full bars.
    fn full_at(&self) -> f32 {
        self.intro_s + self.fill_s
    }

    /// The units' rows.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// The level-up pages after the bars, if the bonus levelled anyone.
    pub fn pages(&self) -> Option<&Progress> {
        self.pages.as_ref()
    }

    /// Whether the level-up pages are up (the bars are done with).
    pub fn paging(&self) -> bool {
        self.paging
    }

    /// Whether the bars are full (at once, when there are none).
    pub fn filled(&self) -> bool {
        self.rows.is_empty() || self.t >= self.full_at()
    }

    /// How much of each unit's bonus the bars show: 0 to 1.
    fn fill(&self) -> f32 {
        if self.filled() {
            return 1.0;
        }
        ((self.t - self.intro_s) / self.fill_s).clamp(0.0, 1.0)
    }

    /// The EXP `row`'s bar shows, counted from its old EXP (so past
    /// [`EXP_PER_LEVEL`] once it has levelled).
    pub fn exp_shown(&self, row: &Row) -> u32 {
        // Bonuses are small (at most a few levels): exact in f32.
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let moved = ((row.gained as f32 * self.fill()) + 1e-3).floor() as u32;
        row.exp.saturating_add(moved.min(row.gained))
    }

    /// Confirm or Cancel: fills the bars, or goes on from full bars (to the
    /// level-up pages, or closing), or presses the level-up page. Returns
    /// whether the screen is over.
    fn press(&mut self) -> bool {
        if self.paging {
            if let Some(pages) = &mut self.pages {
                pages.confirm();
            }
        } else if self.filled() {
            self.paging = true;
        } else {
            self.t = self.full_at();
        }
        self.over()
    }

    /// Whether the bars and every page are done with.
    fn over(&self) -> bool {
        self.paging && self.pages.as_ref().is_none_or(Progress::done)
    }

    /// The bottom help line.
    pub fn help(&self, ctx: &Ctx) -> String {
        let played = match &self.pages {
            Some(pages) if self.paging => pages.page_played(),
            _ => self.filled(),
        };
        playing_help(ctx, played)
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint`]): the tally, or the level-up page once the bars
    /// are done with, and the help line.
    pub fn view(&self, ctx: &Ctx) -> ResultsView {
        let body = match &self.pages {
            Some(pages) if self.paging => {
                let faces = (&ctx.content.portraits, &ctx.lead);
                ResultsBody::LevelUp(progress::view(ctx.words(), faces, pages).map(Box::new))
            }
            _ => ResultsBody::Tally(self.tally(ctx)),
        };
        ResultsView {
            body,
            help: self.help(ctx),
        }
    }

    /// The gold, rewinds and bonus lines and the units' bars.
    fn tally(&self, ctx: &Ctx) -> TallyView {
        let gold = (self.clear_gold > 0).then(|| GoldView {
            label: ctx.text(GOLD_LABEL).to_owned(),
            gained: self.clear_gold,
            now: ctx.text_with("results.gold_now", &[("gold", &self.gold)]),
        });
        let left = ctx.text_with(
            "results.rewinds_left",
            &[("unused", &self.unused), ("charges", &self.charges)],
        );
        let units = self.rows.iter().map(|row| {
            let shown = self.exp_shown(row);
            let levels = shown / EXP_PER_LEVEL;
            let level = row.level.saturating_add(levels);
            UnitView {
                // check-text: not a data name (the view's own)
                name: row.name.clone(),
                class: row.class.clone(),
                level: ctx.text_with("results.level", &[("level", &level)]),
                exp_label: ctx.text("results.exp").to_owned(),
                exp: shown % EXP_PER_LEVEL,
                exp_max: EXP_PER_LEVEL,
                level_up: (levels > 0).then(|| ctx.text(LEVEL_UP).to_owned()),
            }
        });
        TallyView {
            heading: ctx.text(HEADING).to_owned(),
            gold,
            rewinds: RewindsView {
                label: ctx.text(REWINDS_LABEL).to_owned(),
                left,
            },
            bonus: (!self.rows.is_empty()).then(|| BonusView {
                label: ctx.text(BONUS_LABEL).to_owned(),
                exp: self.bonus,
            }),
            units: units.collect(),
        }
    }
}

impl Screen for ResultsScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, _ctx: &mut Ctx, input: &FrameInput) -> Transition {
        for action in &input.actions {
            if matches!(action, Action::Confirm | Action::Cancel) && self.press() {
                return Transition::Pop;
            }
        }
        let held = input.is_held(Action::Confirm);
        let dt = if input.dt.is_finite() {
            input.dt.max(0.0)
        } else {
            0.0
        };
        if self.paging {
            if let Some(pages) = &mut self.pages {
                pages.tick(dt, held);
            }
            if self.over() {
                return Transition::Pop;
            }
        } else if !self.filled() {
            let speed = if held { self.timings.fast } else { 1.0 };
            self.t = (self.t + dt * speed).min(self.full_at());
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
