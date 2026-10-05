//! The title screen: `New Game` starts the game flow ([`FlowScreen`],
//! ticket 0801); `Continue` (shown only while there is a suspend save)
//! carries on the suspended battle and `Load Game` opens the save slots
//! (0802); debug builds add `Quick Battle`, a test battle through the same
//! flow; `Options` opens the [`OptionsScreen`] (0805); `Credits` opens the
//! [`CreditsScreen`] (0808).
//!
//! **The look is a skin** (ADR-0054). This module decides what the screen
//! shows and does, and says it as plain data: a [`TitleView`]
//! ([`TitleScreen::view`]). [`glyph::paint`] draws that view as glyphs.
//! Tests of what happened read the view; only [`glyph`]'s tests read cells.

pub mod glyph;
pub mod view;

pub use view::TitleView;

use super::{CreditsScreen, OptionsScreen, debug_hint};
use crate::audio::MenuSound;
use crate::flow::FlowScreen;
use crate::glyph_buffer::GlyphBuffer;
use crate::save;
use crate::screen::{Ctx, FrameInput, KeyPrompt, Screen, Transition};
use crate::widgets::{Menu, MenuEvent, MenuItem};

/// The title screen's music cue (`audio.md`).
pub const TITLE_MUSIC: &str = "title";

/// One line of the title menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    /// Carries on the suspended battle.
    Continue,
    /// Starts a new game.
    NewGame,
    /// Opens the save slots.
    LoadGame,
    /// Debug: straight into a test battle.
    QuickBattle,
    /// Opens the options.
    Options,
    /// Opens the credits.
    Credits,
    /// Quits.
    Quit,
}

impl Item {
    /// The key of its label in the language files ([`Ctx::text`]).
    fn key(self) -> &'static str {
        match self {
            Self::Continue => "title.continue",
            Self::NewGame => "title.new_game",
            Self::LoadGame => "title.load_game",
            Self::QuickBattle => "title.quick_battle",
            Self::Options => "title.options",
            Self::Credits => "title.credits",
            Self::Quit => "title.quit",
        }
    }
}

/// Title, subtitle and a menu (`Continue` while a battle is suspended,
/// `New Game`, `Load Game`, `Options`, `Credits`, `Quit`), with a help line naming
/// the keys of the active layout.
#[derive(Debug, Clone)]
pub struct TitleScreen {
    menu: Menu,
    /// What each menu line is, in menu order.
    items: Vec<Item>,
    /// Whether the saves may have changed since the menu was built (the
    /// screen has no "shown" hook): set when another screen is opened, and
    /// the next update builds the menu again ([`refresh`](Self::refresh)).
    stale: bool,
    /// Why `Continue` couldn't, shown under the menu until the next key.
    notice: Option<String>,
    /// Whether the title music was asked for since this screen was last
    /// shown. The screen has no "shown" hook, so starting the game flow
    /// (whose battles play their own music) clears it and the next update
    /// asks again.
    music_on: bool,
    /// Whether the "press any key" prompt ([`Ctx::key_prompt`]) is over.
    /// It shows once per launch.
    prompt_done: bool,
}

impl TitleScreen {
    /// The title screen with `New Game` focused, as it is with no saves
    /// (the first update looks for them; [`refreshed`](Self::refreshed)
    /// does at once), labelled in `ctx`'s language.
    pub fn new(ctx: &Ctx) -> Self {
        Self::with_debug(ctx, false)
    }

    /// The title screen with a debug `Quick Battle` item after `Load
    /// Game`, which plays a test battle through the game flow.
    pub fn with_quick_battle(ctx: &Ctx) -> Self {
        Self::with_debug(ctx, true)
    }

    fn with_debug(ctx: &Ctx, quick_battle: bool) -> Self {
        let (menu, items) = Self::build(ctx, quick_battle, false, false);
        Self {
            menu,
            items,
            stale: true,
            notice: None,
            music_on: false,
            prompt_done: false,
        }
    }

    /// The menu and what its lines are: `Continue` first if a battle is
    /// `suspended`; `Load Game` disabled unless a slot is `saved`. Nothing
    /// to back out of on the title screen.
    fn build(ctx: &Ctx, quick_battle: bool, suspended: bool, saved: bool) -> (Menu, Vec<Item>) {
        let mut items = vec![Item::NewGame, Item::LoadGame];
        if suspended {
            items.insert(0, Item::Continue);
        }
        if quick_battle {
            items.push(Item::QuickBattle);
        }
        items.push(Item::Options);
        items.push(Item::Credits);
        items.push(Item::Quit);
        let entries = items.iter().map(|&item| {
            let label = ctx.text(item.key());
            if item == Item::LoadGame && !saved {
                MenuItem::disabled(label)
            } else {
                MenuItem::new(label)
            }
        });
        (Menu::new(entries.collect()).without_cancel(), items)
    }

    /// Builds the menu for the saves in `ctx`'s storage, in `ctx`'s
    /// language. If its lines come out different (a battle was suspended
    /// or continued, a first save made), the focus goes to its first item;
    /// otherwise it stays.
    pub fn refresh(&mut self, ctx: &Ctx) {
        self.stale = false;
        let storage = ctx.storage.as_ref();
        let suspended = save::has_suspend(storage);
        let saved = save::any_slot(storage);
        let quick_battle = self.items.contains(&Item::QuickBattle);
        let (mut menu, items) = Self::build(ctx, quick_battle, suspended, saved);
        let enabled = |m: &Menu| m.items().iter().map(|i| i.enabled).collect::<Vec<_>>();
        if items == self.items && enabled(&menu) == enabled(&self.menu) {
            menu = menu.focused(self.menu.focus());
        }
        self.menu = menu;
        self.items = items;
    }

    /// The same screen with its menu built for `ctx`'s saves
    /// ([`refresh`](Self::refresh)).
    #[must_use]
    pub fn refreshed(mut self, ctx: &Ctx) -> Self {
        self.refresh(ctx);
        self
    }

    /// The notice under the menu, if any.
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    /// Opens `screen` over the title; the saves and the music may change
    /// under it, so the title looks for both again when it is back.
    fn open(&mut self, screen: Box<dyn Screen>) -> Transition {
        self.stale = true;
        self.music_on = false;
        Transition::Push(screen)
    }

    /// Whether the title is still showing "press any key" instead of its
    /// menu.
    fn waiting(&self, ctx: &Ctx) -> bool {
        ctx.key_prompt != KeyPrompt::Off && !self.prompt_done
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint`]): the title and subtitle, then either the "press
    /// any key" prompt or the menu with its notice and help line, and the
    /// debug hint.
    pub fn view(&self, ctx: &Ctx) -> TitleView {
        let waiting = self.waiting(ctx);
        TitleView {
            title: ctx.text("title.title").to_owned(),
            subtitle: ctx.text("title.subtitle").to_owned(),
            prompt: waiting.then(|| ctx.text("title.press_any_key").to_owned()),
            menu: (!waiting).then(|| self.menu.view()),
            notice: self.notice.clone().filter(|_| !waiting),
            help: (!waiting).then(|| Self::help(ctx)),
            debug_hint: debug_hint(ctx),
        }
    }

    /// The bottom help line: the cursor keys `move`, the Confirm key
    /// `select`, the Cancel key `back`, named from the active keymap.
    pub fn help(ctx: &Ctx) -> String {
        ctx.text_with("title.help", &[])
    }
}

impl Screen for TitleScreen {
    fn name(&self) -> &'static str {
        "title"
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        if self.waiting(ctx) {
            // A key pressed on another screen first (the layout picker)
            // counts too. The key that ends the wait does nothing else.
            if ctx.key_prompt == KeyPrompt::Pressed {
                self.prompt_done = true;
                ctx.audio.play_music(TITLE_MUSIC);
                self.music_on = true;
            }
            return Transition::None;
        }
        if !self.music_on {
            // The music state ignores a request for the track already on.
            ctx.audio.play_music(TITLE_MUSIC);
            self.music_on = true;
        }
        if self.stale {
            self.refresh(ctx);
        }
        for &action in &input.actions {
            self.notice = None;
            let chosen = match self.menu.handle_with_sound(action, &mut ctx.audio) {
                Some(MenuEvent::Chosen(i)) => self.items.get(i).copied(),
                Some(MenuEvent::Cancelled) | None => None,
            };
            match chosen {
                Some(Item::Continue) => match FlowScreen::resume(ctx) {
                    Ok(flow) => return self.open(Box::new(flow)),
                    // The save stays; the title says why it can't go on.
                    Err(e) => {
                        self.notice = Some(e.to_string());
                        ctx.audio.menu(MenuSound::Denied);
                    }
                },
                Some(Item::NewGame) => return self.open(Box::new(FlowScreen::new_game())),
                Some(Item::LoadGame) => return self.open(Box::new(FlowScreen::load_game(ctx))),
                // The test data always builds (tested); should it ever
                // not, the item does nothing.
                Some(Item::QuickBattle) => {
                    if let Some(flow) = FlowScreen::quick_battle(ctx) {
                        return self.open(Box::new(flow));
                    }
                }
                // Not `open`: the options and the credits change neither
                // the saves nor the music (they play the title's).
                Some(Item::Options) => return Transition::Push(Box::new(OptionsScreen::new())),
                Some(Item::Credits) => return Transition::Push(Box::new(CreditsScreen::new(ctx))),
                Some(Item::Quit) => return Transition::Quit,
                _ => {}
            }
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
