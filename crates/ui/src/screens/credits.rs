//! The credits screen (ticket 0808; `docs/design/audio.md` rule 3): every
//! third-party work the game uses, grouped by kind, in a scrolling panel.
//! Each entry shows its title, author, license and source link, which is
//! what CC BY asks for. The list is [`trpg_content::Credits`]: the audio
//! manifest's credits and `assets/data/credits.ron`, merged.
//!
//! The list rolls by itself (Nick, PR 148: "credits should auto scroll …
//! manual paging/scrolling can be toggled"), and the title music plays.
//! Confirm stops and restarts the rolling; the cursor keys stop it and
//! scroll by hand.
//!
//! **The look is a skin** (ADR-0054): this module decides what the screen
//! shows and does, and says it as a [`CreditsView`]
//! ([`CreditsScreen::view`]); [`glyph::paint`] draws that view.

pub mod glyph;
pub mod view;

use crate::prelude::*;
pub use view::{CreditRowKind, CreditRowView, CreditsView};

use trpg_content::{CreditGroup, Credits};

use super::title::TITLE_MUSIC;
use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::widgets::help::SEPARATOR;
use crate::widgets::word_wrap;

/// The key of the screen's title, on the panel's border, in the language
/// files ([`Ctx::text`]).
pub const TITLE: &str = "credits.title";

/// Rows of the list on screen at once.
const LIST_H: usize = 25;

/// Seconds between rows while the list rolls by itself (*tunable*): an
/// entry is three rows, so one goes by every second and a half.
pub const ROW_SECS: f32 = 0.5;
/// Seconds the list rests at its top before it starts to roll, and at its
/// bottom before it starts again from the top (*tunable*).
pub const REST_SECS: f32 = 2.0;

/// The key of the heading a group's credits go under.
pub fn heading(group: CreditGroup) -> &'static str {
    match group {
        CreditGroup::Music => "credits.group.music",
        CreditGroup::SoundEffects => "credits.group.sound_effects",
        CreditGroup::Art => "credits.group.art",
        CreditGroup::Fonts => "credits.group.fonts",
        CreditGroup::Software => "credits.group.software",
    }
}

/// The characters that fit on a row of the list, which the list is wrapped
/// to. (The skin indents each kind; a test of the skin pins these to its
/// panel.)
fn width(kind: CreditRowKind) -> usize {
    match kind {
        CreditRowKind::Heading => 91,
        CreditRowKind::Entry => 89,
        CreditRowKind::Detail => 87,
    }
}

/// The whole list as rows of text, wrapped to the panel: each group that
/// has credits under its [`heading`], then a blank row, then its entries
/// (`"Title" by Author`, then `License · link`) with a blank row after
/// each. No blank row at the end. In `ctx`'s language.
fn rows(ctx: &Ctx) -> Vec<(CreditRowKind, String)> {
    let credits: &Credits = &ctx.game.content.credits;
    let mut rows = Vec::new();
    let mut add = |kind: CreditRowKind, text: &str| {
        if text.is_empty() {
            rows.push((kind, String::new()));
        }
        for line in word_wrap(text, width(kind)) {
            rows.push((kind, line));
        }
    };
    for group in CreditGroup::ALL {
        let mut entries = credits.in_group(group).peekable();
        if entries.peek().is_none() {
            continue;
        }
        add(CreditRowKind::Heading, ctx.text(heading(group)));
        add(CreditRowKind::Heading, "");
        for e in entries {
            let by = ctx.text_with(
                "credits.entry",
                &[("title", &e.title), ("author", &e.author)],
            );
            add(CreditRowKind::Entry, &by);
            add(
                CreditRowKind::Detail,
                &format!("{}{SEPARATOR}{}", e.license, e.source),
            );
            add(CreditRowKind::Entry, "");
        }
    }
    rows.pop();
    rows
}

/// The credits screen. The list rolls up a row at a time by itself, rests
/// at the bottom and starts again from the top. Confirm stops or restarts
/// the rolling; cursor up and down stop it and scroll a row at a time;
/// Cancel closes the screen.
#[derive(Debug, Clone)]
pub struct CreditsScreen {
    /// The list, wrapped ([`rows`]).
    rows: Vec<(CreditRowKind, String)>,
    /// Index in `rows` of the first row on screen.
    top: usize,
    /// Whether the list is rolling by itself.
    rolling: bool,
    /// Seconds until the list rolls its next row.
    wait: f32,
    /// Whether the title music has been asked for.
    music_on: bool,
}

impl CreditsScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "credits";

    /// The screen showing the top of `ctx`'s credits, about to roll.
    pub fn new(ctx: &Ctx) -> Self {
        Self {
            rows: rows(ctx),
            top: 0,
            rolling: true,
            wait: REST_SECS,
            music_on: false,
        }
    }

    /// Whether the list is rolling by itself.
    pub fn is_rolling(&self) -> bool {
        self.rolling
    }

    /// The highest `top`: the last row at the bottom of the list.
    fn max_top(&self) -> usize {
        self.rows.len().saturating_sub(LIST_H)
    }

    /// Whether there are rows above the ones on screen.
    pub fn more_above(&self) -> bool {
        self.top > 0
    }

    /// Whether there are rows below the ones on screen.
    pub fn more_below(&self) -> bool {
        self.top < self.max_top()
    }

    /// Scrolls one row down or up, with the menu's tick; nothing (and no
    /// sound) at either end.
    fn scroll(&mut self, ctx: &mut Ctx, down: bool) {
        let to = if down {
            (self.top + 1).min(self.max_top())
        } else {
            self.top.saturating_sub(1)
        };
        if to != self.top {
            self.top = to;
            ctx.audio.menu(MenuSound::Move);
        }
    }

    /// Lets `dt` seconds pass while the list rolls: a row every
    /// [`ROW_SECS`], a rest of [`REST_SECS`] at the bottom, then the top
    /// again and the same rest there. Silent. A list that fits on screen
    /// stays put.
    ///
    /// One frame rolls at most once round the list (every row and the
    /// jump back to the top), so a huge frame time can't keep it busy; a
    /// frame longer than that puts the list back at its top, resting.
    fn roll(&mut self, dt: f32) {
        if !self.rolling || self.max_top() == 0 {
            return;
        }
        self.wait -= dt;
        for _ in 0..=self.max_top() {
            if self.wait > 0.0 {
                return;
            }
            if self.more_below() {
                self.top += 1;
                self.wait += if self.more_below() {
                    ROW_SECS
                } else {
                    REST_SECS
                };
            } else {
                self.top = 0;
                self.wait += REST_SECS;
            }
        }
        if self.wait <= 0.0 {
            self.top = 0;
            self.wait = REST_SECS;
        }
    }

    /// Confirm: stops the rolling, or starts it again from the rows on
    /// screen after one [`ROW_SECS`].
    fn toggle_rolling(&mut self, ctx: &mut Ctx) {
        self.rolling = !self.rolling;
        self.wait = ROW_SECS;
        ctx.audio.menu(MenuSound::Select);
    }

    /// The bottom help line: the cursor keys `scroll`, the Confirm key
    /// `pause` while the list rolls and `auto-scroll` while it doesn't,
    /// the Cancel key `back`, named from the active keymap.
    pub fn help(&self, ctx: &Ctx) -> String {
        let key = if self.rolling {
            "credits.help.rolling"
        } else {
            "credits.help.stopped"
        };
        ctx.text_with(key, &[])
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint`]): the rows on screen, whether more are above or
    /// below them, and the help line.
    pub fn view(&self, ctx: &Ctx) -> CreditsView {
        let on_screen = self.rows.iter().skip(self.top).take(LIST_H);
        CreditsView {
            title: ctx.text(TITLE).to_owned(),
            rows: on_screen
                .map(|(kind, text)| CreditRowView {
                    kind: *kind,
                    text: text.clone(),
                })
                .collect(),
            more_above: self.more_above(),
            more_below: self.more_below(),
            help: self.help(ctx),
        }
    }
}

impl Screen<Shuyi> for CreditsScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        if !self.music_on {
            // The music state ignores a request for the track already on
            // (the title's, when the screen is opened from there).
            ctx.audio.play_music(TITLE_MUSIC);
            self.music_on = true;
        }
        self.roll(input.dt());
        for &action in &input.actions {
            match action {
                Action::CursorUp | Action::CursorDown => {
                    // Scrolling by hand takes over from the rolling.
                    self.rolling = false;
                    self.scroll(ctx, action == Action::CursorDown);
                }
                Action::Confirm => self.toggle_rolling(ctx),
                Action::Cancel => {
                    ctx.audio.menu(MenuSound::Cancel);
                    return Transition::Pop;
                }
                _ => {}
            }
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
