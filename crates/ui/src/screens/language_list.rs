//! The first-launch language list (ticket 0825,
//! `docs/design/voices-languages-and-script.md`, *Picking a language and
//! saying it is machine-made*), and the names it and the Options screen's
//! Language row show ([`names`]).
//!
//! A first launch asks for the language once, before anything else is
//! asked: [`game::start`](crate::game::start) opens [`LanguageListScreen`]
//! over the title, with the layout picker under it on a keyboard, while no
//! language and no layout are picked ([`asks`]: only once the game has a
//! language besides English). The list starts on the language in use,
//! which the engine picked from the system's languages (English when the
//! game has no pack for them); one press of Confirm keeps it, and the
//! layout picker under it is next. Once picked it is never shown again:
//! the Options row changes it.
//!
//! Each language is shown by its own name, with its pack's note when a
//! machine made it ([`MACHINE_NOTE`], in the pack's own language), so
//! whether a person checks it is the pack's word, not this code's.
//!
//! **The look is a skin** (ADR-0054): the view is the save prompt's (a
//! title, the list, a help line) and so is the paint
//! ([`glyph::paint_prompt`]).

use crate::prelude::*;
use shuyi_assets::lang::MadeBy;
use trpg_content::lang::TEST;

use crate::glyph_buffer::GlyphBuffer;
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::screens::layout_picker::LayoutPickerScreen;
use crate::screens::save::{SavePromptView, glyph};
use crate::widgets::{Menu, MenuEvent, MenuItem};

/// The text key of a machine-made pack's note beside its name, in its own
/// language: each pack says it in its `ui.ron` (`(机器翻译，母语者审校)`
/// for Chinese, which a native reader checks).
pub const MACHINE_NOTE: &str = "language.machine_note";

/// The key of the list's title ([`Ctx::text`]): the Options row's label.
pub const TITLE: &str = shuyi_ui::screens::options::LANGUAGE_ROW;

// engine-friction: engine#3257, moves (game 1418 deletes it)
/// Every language the game offers ([`Ctx::languages_offered`]: the source,
/// then the packs, the test pack only with debug tools), each by its own
/// name, a machine-made one with its pack's [`MACHINE_NOTE`] after it.
pub fn names(ctx: &Ctx) -> Vec<String> {
    let lang = &ctx.game.content.lang;
    let named = |(code, name): (_, String)| match lang.pack(&code) {
        Some(pack) if pack.info.made_by == MadeBy::Machine => {
            // check-text: a name and its note, both the pack's own text
            format!("{name} {}", lang.text(&code, MACHINE_NOTE))
        }
        _ => name,
    };
    ctx.languages_offered().into_iter().map(named).collect()
}

// engine-friction: engine#3257, moves (game 1418 deletes it)
/// Which of [`names`] is in use: the first tag of its chain the game
/// offers, as the engine's Language row reads it.
pub fn in_use(ctx: &Ctx) -> usize {
    let offered = ctx.languages_offered();
    let chain = ctx.game.content.lang.chain(&ctx.lang);
    chain
        .iter()
        .find_map(|tag| offered.iter().position(|(code, _)| code == tag))
        .unwrap_or(0)
}

/// The `i`-th of [`names`], as the language picked: previewed (Options
/// keeps or throws it away) unless `save`. Returns whether it changed
/// anything; `false` for no such language.
pub fn pick(ctx: &mut Ctx, i: usize, save: bool) -> bool {
    let Some((code, _)) = ctx.languages_offered().into_iter().nth(i) else {
        return false;
    };
    if save {
        let before = ctx.settings().language.clone();
        // If saving fails the language is still used this session; the
        // player is just asked again next launch.
        let _ = ctx.change_settings(|s| s.language = Some(code));
        ctx.settings().language != before
    } else {
        ctx.preview_settings(|s| s.language = Some(code))
    }
}

/// Whether a start asks for the language: no language and no layout picked
/// yet, and a language to pick besides the source (the test pack, a
/// developer's, doesn't count). A player whose settings are from before
/// the list (they have a layout) isn't asked: Options changes it.
pub fn asks(ctx: &Ctx) -> bool {
    let lang = &ctx.game.content.lang;
    let choice = lang.codes().any(|code| code.as_str() != TEST);
    choice && ctx.settings().language.is_none() && ctx.layout().is_none()
}

// engine-friction: engine#3257, moves (game 1418 deletes it)
/// The language list: one line per language, the one in use focused. It
/// can't be backed out of; it pops once one is picked.
#[derive(Debug, Clone)]
pub struct LanguageListScreen {
    /// One line per language: its index in [`names`].
    menu: Menu<usize>,
}

// engine-friction: engine#3257, moves (game 1418 deletes it)
impl LanguageListScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "language_list";

    /// The list with the language in use focused.
    pub fn new(ctx: &Ctx) -> Self {
        let items = names(ctx)
            .into_iter()
            .enumerate()
            .map(|(i, name)| MenuItem::new(name).with_value(i));
        let menu = Menu::new(items.collect()).without_cancel();
        Self {
            menu: menu.focused(in_use(ctx)),
        }
    }

    /// The list as it is now, as plain data for a skin to paint: the
    /// title, every language by its own name and the help line (the layout
    /// picker's: no layout is picked yet).
    pub fn view(&self, ctx: &Ctx) -> SavePromptView {
        SavePromptView {
            question: ctx.text(TITLE).to_owned(),
            menu: self.menu.view(),
            help: LayoutPickerScreen::help(ctx),
        }
    }
}

// engine-friction: engine#3257, moves (game 1418 deletes it)
impl Screen<Shuyi> for LanguageListScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn actions(&self) -> Option<Vec<crate::input::Action>> {
        Some(crate::screens::actions::LANGUAGE_LIST.to_vec())
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        for &action in &input.actions {
            if let Some(MenuEvent::Chosen(i)) = self.menu.handle_with_sound(action, &mut ctx.audio)
            {
                pick(ctx, i, true);
                return Transition::Pop;
            }
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint_prompt(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
