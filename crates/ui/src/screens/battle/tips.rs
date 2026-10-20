//! The battle's one-time tips (0406): which triggers have fired, the tips
//! waiting to be shown, and which the player has already seen.

use crate::prelude::*;
use alloc::collections::VecDeque;

use trpg_content::TipTrigger;
use trpg_core::TutorialLevel;

use crate::screen::Ctx;
use crate::tips::TipsSeen;

/// The tips of one battle screen. A trigger that fires is queued if its tip
/// has not been seen; the screen shows the queue front when the moment is
/// right, and only then is it recorded as seen (ticket 0467), so it shows
/// at most once per profile and a tip that never reached the screen (its
/// battle retried or left first) is queued again the next time its
/// trigger fires. The player dismisses it once it is on screen.
#[derive(Debug, Clone, Default)]
pub struct TipState {
    /// Loaded from storage on first use.
    seen: Option<TipsSeen>,
    /// Triggers that fired since the last [`absorb`](Self::absorb).
    fired: Vec<TipTrigger>,
    /// Tips waiting to be shown, first in front.
    queue: VecDeque<TipTrigger>,
    /// The tip on screen as the last frame ended, if any.
    on_screen: Option<TipTrigger>,
    /// Whether a screen is over the battle (a scene it started, ticket
    /// 0455; the rewind screen), so no tip shows.
    covered: bool,
    /// Whether tips are off, or the campaign's tutorial level shows none,
    /// as of the last [`absorb`](Self::absorb): the queue waits, hidden
    /// (ticket 0841).
    off: bool,
}

impl TipState {
    /// Notes that `trigger` happened.
    pub fn fire(&mut self, trigger: TipTrigger) {
        if !self.fired.contains(&trigger) {
            self.fired.push(trigger);
        }
    }

    /// Whether tips show in `ctx`: they are on, and the campaign's
    /// tutorial level (if any) shows them (0835).
    pub fn shown_in(ctx: &Ctx) -> bool {
        ctx.game.tips_enabled
            && ctx
                .game
                .campaign_tutorial
                .is_none_or(TutorialLevel::shows_tips)
    }

    /// Queues the fired triggers whose tip exists and hasn't been seen
    /// (nor is queued already). With tips off, or a campaign whose tutorial
    /// level shows none, they are just forgotten, and the tips already
    /// queued wait hidden until tips show again (the level may change
    /// mid-battle in Options, 0841).
    pub fn absorb(&mut self, ctx: &mut Ctx) {
        let fired = core::mem::take(&mut self.fired);
        self.off = !Self::shown_in(ctx);
        if self.off {
            return;
        }
        let seen = self
            .seen
            .get_or_insert_with(|| TipsSeen::load(&ctx.storage));
        for trigger in fired {
            let Some(tip) = ctx.game.content.tips.for_trigger(trigger) else {
                continue;
            };
            if !seen.contains(&tip.id) && !self.queue.contains(&trigger) {
                self.queue.push_back(trigger);
            }
        }
    }

    /// Notes which tip is on screen as the frame ends (`None` for none),
    /// and saves it as seen.
    pub fn show(&mut self, trigger: Option<TipTrigger>, ctx: &mut Ctx) {
        self.on_screen = trigger;
        let Some(tip) = trigger.and_then(|t| ctx.game.content.tips.for_trigger(t)) else {
            return;
        };
        let seen = self
            .seen
            .get_or_insert_with(|| TipsSeen::load(&ctx.storage));
        seen.mark(&tip.id, &mut ctx.storage);
    }

    /// Whether `trigger`'s tip was on screen as the last frame ended: only
    /// then may a key dismiss it.
    pub fn on_screen(&self, trigger: TipTrigger) -> bool {
        self.on_screen == Some(trigger)
    }

    /// The queued tips' triggers, first to show first: none while tips
    /// are off.
    pub fn queued(&self) -> impl Iterator<Item = TipTrigger> + '_ {
        self.queue.iter().copied().filter(|_| !self.off)
    }

    /// Notes whether a screen is over the battle: while one is, no tip
    /// shows.
    pub fn set_covered(&mut self, covered: bool) {
        self.covered = covered;
    }

    /// Whether a screen is over the battle.
    pub fn covered(&self) -> bool {
        self.covered
    }

    /// Drops `trigger`'s tip from the queue.
    pub fn dismiss(&mut self, trigger: TipTrigger) {
        self.queue.retain(|&t| t != trigger);
    }
}
