//! The save screens (ticket 0802, `docs/design/death-and-difficulty.md`):
//! "Save your progress?" after a chapter's victory ([`SavePromptScreen`])
//! and the slot picker ([`SlotPickerScreen`]), which saves the campaign into
//! one of the [`SLOTS`](crate::save::SLOTS) slots (asking before it overwrites one) or loads one.
//!
//! Both pop with what happened, for the screen below (the game flow,
//! [`crate::flow`]). The slot picker is made to be
//! opened from other menus too (the world map's `Save`, 1007).
//!
//! On Load Game the Unit info key exports the save under the cursor as a
//! file, after asking whether the options and key bindings go with it,
//! and on a desktop a line says where the saves are (ticket 0872,
//! `docs/design/options.md`, *Saves outside the game*; [`crate::save_files`]).
//! The first time it opens with a save to export, the guide's tip says so
//! ([`TipScreen`], ticket 0874). The import key reads a file into this
//! profile or a new one ([`ImportScreen`], ticket 0873).
//!
//! **The look is a skin** (ADR-0054): this module decides what the screens
//! show and do, and says it as a [`SavePromptView`] and a
//! [`SlotPickerView`]; [`glyph`] paints them.

pub mod glyph;
pub mod view;

use crate::prelude::*;
pub use view::{SavePromptView, SlotColumnsView, SlotEntryView, SlotPickerView, SlotView};

use trpg_content::TipTrigger;
use trpg_core::{Campaign, GameMode, SaveFile};

use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::save::{self, Slot, playtime_text, slot_key};
use crate::save_files::import::Channel;
use crate::save_files::{self, Export, ExportNotice, Exported};
use crate::screen::{Ctx, FrameInput, Screen, ScreenResult, Shuyi, Transition};
use crate::screens::files;
use crate::screens::pick_one;
use crate::screens::save_import::{ImportDone, ImportScreen};
use crate::tips::TipScreen;
use crate::widgets::{Ask, ListCursor, Menu, MenuItem};

/// The key of the question after a chapter's victory in the language
/// files ([`Ctx::text`]).
pub const SAVE_QUESTION: &str = "save_prompt.question";
/// The key of "Overwrite slot N?".
const OVERWRITE_QUESTION: &str = "save.overwrite_question";
/// The key of the slot picker's title when saving.
pub const SAVE_TITLE: &str = "save.title.save";
/// The key of the slot picker's title when loading.
pub const LOAD_TITLE: &str = "save.title.load";
/// The key of what an empty slot shows.
pub const EMPTY_SLOT: &str = "save.empty";
/// The key of the question before a save is exported.
pub const EXPORT_QUESTION: &str = "save.export_question";

/// Slots shown at once.
const VISIBLE: usize = 20;

/// "Save your progress?" with `Yes` / `No`. Pops with the answer (a
/// `bool`, `true` for `Yes`) once one is chosen; there is nothing to back
/// out of.
#[derive(Debug, Clone)]
pub struct SavePromptScreen {
    menu: Menu<bool>,
}

impl SavePromptScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "save_prompt";

    /// The question with `Yes` focused, labelled in `ctx`'s language.
    pub fn new(ctx: &Ctx) -> Self {
        let items = [("save_prompt.yes", true), ("save_prompt.no", false)]
            .map(|(key, yes)| MenuItem::new(ctx.text(key)).with_value(yes))
            .to_vec();
        Self {
            menu: Menu::new(items).without_cancel(),
        }
    }

    /// The bottom help line.
    pub fn help(ctx: &Ctx) -> String {
        ctx.text_with("save_prompt.help", &[])
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint_prompt`]): the question, `Yes` and `No`, and the
    /// help line.
    pub fn view(&self, ctx: &Ctx) -> SavePromptView {
        SavePromptView {
            question: ctx.text(SAVE_QUESTION).to_owned(),
            menu: self.menu.view(),
            help: Self::help(ctx),
        }
    }
}

impl Screen<Shuyi> for SavePromptScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn actions(&self) -> Option<Vec<crate::input::Action>> {
        Some(crate::screens::actions::SAVE_PROMPT.to_vec())
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        pick_one(&mut self.menu, ctx, input)
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint_prompt(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }
}

/// What the slot picker is open for.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Purpose {
    /// Saving this: a chapter save, or a world-map save (1007).
    Save(Box<SaveFile>),
    /// Loading a chapter save.
    Load,
}

/// What the player did in the slot picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotOutcome {
    /// The campaign was saved into this slot (1 to [`SLOTS`](crate::save::SLOTS)).
    Saved(usize),
    /// This slot's save was chosen to load.
    Loaded(usize, Box<SaveFile>),
}

/// The [`SLOTS`](crate::save::SLOTS) save slots, 20 at a time: each with its chapter,
/// game mode, army size and playtime. Confirm saves into the focused slot
/// (the help line reads `overwrite` and "Overwrite slot N?" is asked if it
/// holds something) or loads it, and pops with what it did (a
/// [`SlotOutcome`]); Cancel pops with nothing (back).
#[derive(Debug, Clone)]
pub struct SlotPickerScreen {
    purpose: Purpose,
    slots: Vec<Slot>,
    /// The focused slot (slot number − 1) and the slots shown.
    cursor: ListCursor,
    /// "Overwrite slot N?", while it is open.
    asking: Option<Ask<bool>>,
    /// "Export your options and key bindings too?", while it is open.
    exporting: Option<Ask<bool>>,
    /// The message under the panel, if any.
    message: Option<String>,
    /// Where the last export went.
    notice: ExportNotice,
    /// What the last import did, until the cursor moves.
    imported: Option<String>,
    /// Whether the guide's tip on exporting was looked at (ticket 0874):
    /// the first update does.
    tip_asked: bool,
}

impl SlotPickerScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "save_slots";

    /// The picker for saving `campaign` (its chapter just won), focused on
    /// the first empty slot (Nick, PR #141), so that Confirm never
    /// overwrites a save by reflex; the first slot if none is empty.
    pub fn save(ctx: &Ctx, campaign: Campaign) -> Self {
        Self::saving(ctx, SaveFile::chapter_cleared(campaign))
    }

    /// The picker for saving `campaign` on the world map (1007), focused
    /// as [`save`](Self::save) is.
    pub fn save_world(ctx: &Ctx, campaign: Campaign) -> Self {
        Self::saving(ctx, SaveFile::world_map(campaign))
    }

    fn saving(ctx: &Ctx, file: SaveFile) -> Self {
        let slots = save::slots(&ctx.storage, &ctx.game.content, ctx.words());
        let empty = slots.iter().position(|s| *s == Slot::Empty);
        Self::new(Purpose::Save(Box::new(file)), slots, empty)
    }

    /// The picker for loading, focused on the first slot with a save.
    pub fn load(ctx: &Ctx) -> Self {
        let slots = save::slots(&ctx.storage, &ctx.game.content, ctx.words());
        let saved = slots.iter().position(|s| matches!(s, Slot::Saved(_)));
        Self::new(Purpose::Load, slots, saved)
    }

    fn new(purpose: Purpose, slots: Vec<Slot>, focus: Option<usize>) -> Self {
        let mut cursor = ListCursor::new(slots.len()).at(focus.unwrap_or(0));
        cursor.window(VISIBLE);
        Self {
            purpose,
            slots,
            cursor,
            asking: None,
            exporting: None,
            message: None,
            notice: ExportNotice::default(),
            imported: None,
            tip_asked: false,
        }
    }

    /// The same picker focused on slot `slot` (1 to the number of slots).
    #[cfg(test)]
    fn at(mut self, slot: usize) -> Self {
        self.cursor = self.cursor.at(slot - 1);
        self.cursor.window(VISIBLE);
        self
    }

    /// Whether Confirm on the focused slot would replace what it holds
    /// (asked about first): saving, on a slot that isn't empty.
    pub fn would_overwrite(&self) -> bool {
        self.is_saving()
            && self
                .slots
                .get(self.cursor.focus())
                .is_some_and(|s| *s != Slot::Empty)
    }

    /// The focused slot's number (1 to [`SLOTS`](crate::save::SLOTS)).
    pub fn focused_slot(&self) -> usize {
        self.cursor.focus() + 1
    }

    /// Whether the picker saves (rather than loads).
    pub fn is_saving(&self) -> bool {
        matches!(self.purpose, Purpose::Save(_))
    }

    /// Whether "Overwrite slot N?" or the export question is open.
    pub fn is_asking(&self) -> bool {
        self.asking.is_some() || self.exporting.is_some()
    }

    /// Whether the export question is open.
    pub fn is_asking_export(&self) -> bool {
        self.exporting.is_some()
    }

    /// Whether the save under the cursor can be exported: loading, a save
    /// that reads, on a platform with files a player reaches.
    pub fn can_export(&self, ctx: &Ctx) -> bool {
        !self.is_saving()
            && save_files::can_export(&ctx.platform)
            && matches!(self.slots.get(self.cursor.focus()), Some(Slot::Saved(_)))
    }

    /// The export key on the focused slot: asks whether the options and
    /// key bindings go with it, the cursor on No (Nick, 0872).
    fn ask_export(&mut self, ctx: &mut Ctx) {
        if !self.can_export(ctx) {
            return;
        }
        self.message = None;
        let answers = [(true, "save.answer.yes"), (false, "save.answer.no")];
        let ask = Ask::new(EXPORT_QUESTION, answers)
            .focused_on(false)
            .cancelled_by(false);
        self.exporting = Some(ask);
        ctx.audio.menu(MenuSound::Select);
    }

    /// An action while the export question is open: Cancel closes it,
    /// either answer exports.
    fn answer_export(&mut self, ctx: &mut Ctx, action: Action) {
        let Some(ask) = self.exporting.as_mut() else {
            return;
        };
        if action == Action::Cancel {
            self.exporting = None;
            ctx.audio.menu(MenuSound::Cancel);
            return;
        }
        let Some(settings) = ask.handle(action, &mut ctx.audio) else {
            return;
        };
        self.exporting = None;
        let what = Export::Slot {
            slot: self.focused_slot(),
            settings,
        };
        let id = ctx.profiles().current();
        if let Ok(file) = save_files::export(ctx, id, what) {
            self.notice.offered(file);
            ctx.audio.menu(MenuSound::Select);
        } else {
            self.notice.heard(Some(Exported::Failed));
            ctx.audio.menu(MenuSound::Denied);
        }
    }

    /// The platform's answer to the last export ([`save_files::hear`]).
    pub fn heard(&mut self, exported: Option<Exported>) {
        self.notice.heard(exported);
    }

    /// The guide's tip on exporting a save (ticket 0874), if it is due:
    /// loading, with a save in the list, where saves can be exported, the
    /// first time ([`TipScreen::once`]).
    fn export_tip(&self, ctx: &mut Ctx) -> Option<TipScreen> {
        let saved = self.slots.iter().any(|s| matches!(s, Slot::Saved(_)));
        let due = !self.is_saving() && saved && save_files::can_export(&ctx.platform);
        due.then(|| TipScreen::once(ctx, TipTrigger::FirstLoadGame))
            .flatten()
    }

    /// The message under the panel, if any.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// The overwrite question for the focused slot.
    pub fn overwrite_question(&self, ctx: &Ctx) -> String {
        ctx.text_with(OVERWRITE_QUESTION, &[("slot", &self.slot_text())])
    }

    /// The focused slot's number as the overwrite question shows it.
    fn slot_text(&self) -> String {
        format!("{:02}", self.focused_slot())
    }

    /// Moves the focus one slot down or up, wrapping.
    fn step(&mut self, ctx: &mut Ctx, down: bool) {
        if !self.cursor.step(down, true, |_| true) {
            return;
        }
        self.cursor.window(VISIBLE);
        self.message = None;
        self.notice.clear();
        self.imported = None;
        ctx.audio.menu(MenuSound::Move);
    }

    /// Writes the campaign into the focused slot. Returns the pop with
    /// [`SlotOutcome::Saved`] if it was saved; a failure shows as the
    /// message.
    fn write(&mut self, ctx: &mut Ctx) -> Option<Transition> {
        let Purpose::Save(file) = &self.purpose else {
            return None;
        };
        let slot = self.focused_slot();
        match save::write(&mut ctx.storage, &slot_key(slot), file) {
            Ok(()) => {
                ctx.audio.menu(MenuSound::Select);
                Some(Transition::pop_with(SlotOutcome::Saved(slot)))
            }
            Err(e) => {
                self.message = Some(e.text(ctx));
                ctx.audio.menu(MenuSound::Denied);
                None
            }
        }
    }

    /// Confirm on the focused slot. Returns the pop, if the picker is
    /// done.
    fn confirm(&mut self, ctx: &mut Ctx) -> Option<Transition> {
        self.message = None;
        if self.is_saving() {
            if !self.would_overwrite() {
                return self.write(ctx);
            }
            self.asking = Some(Ask::yes_no(OVERWRITE_QUESTION, "save.overwrite_answers"));
            ctx.audio.menu(MenuSound::Select);
            return None;
        }
        match self.slots.get(self.cursor.focus()) {
            Some(Slot::Saved(summary)) => {
                let save = Box::new(summary.save.clone());
                ctx.audio.menu(MenuSound::Select);
                let loaded = SlotOutcome::Loaded(self.focused_slot(), save);
                Some(Transition::pop_with(loaded))
            }
            Some(Slot::Unreadable(e)) => {
                self.message = Some(e.text(ctx));
                ctx.audio.menu(MenuSound::Denied);
                None
            }
            Some(Slot::Empty) | None => {
                ctx.audio.menu(MenuSound::Denied);
                None
            }
        }
    }

    /// An action while the overwrite question is open. Returns the pop, if
    /// the picker is done.
    fn answer(&mut self, ctx: &mut Ctx, action: Action) -> Option<Transition> {
        let yes = self.asking.as_mut()?.handle(action, &mut ctx.audio)?;
        self.asking = None;
        if yes {
            return self.write(ctx);
        }
        ctx.audio.menu(MenuSound::Cancel);
        None
    }

    /// The bottom help line.
    pub fn help(&self, ctx: &Ctx) -> String {
        // A slot with something in it says so before it is chosen.
        let folder = save_files::place(&ctx.platform).is_some();
        let key = if self.is_asking_export() {
            "save.help.export_question"
        } else if self.is_asking() {
            "save.help.asking"
        } else if self.would_overwrite() {
            "save.help.overwrite"
        } else if self.is_saving() {
            "save.help.save_here"
        } else if self.can_export(ctx) && folder {
            "save.help.load_export_folder"
        } else if self.can_export(ctx) {
            "save.help.load_export"
        } else if folder {
            "save.help.load_folder"
        } else if ImportScreen::can_open(ctx) {
            "save.help.load_import"
        } else {
            "save.help.load"
        };
        ctx.text_with(key, &[])
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint_slots`]): the slots in view with what each holds,
    /// the message, the overwrite question if it is open, and the help
    /// line.
    pub fn view(&self, ctx: &Ctx) -> SlotPickerView {
        let top = self.cursor.first_shown();
        let shown = top..(top + VISIBLE).min(self.slots.len());
        // Which slots are on screen.
        let (first, count) = (shown.start + 1, self.slots.len());
        let range = ctx.text_with(
            "save.range",
            &[("first", &first), ("last", &shown.end), ("count", &count)],
        );
        let slots = shown.map(|i| self.slot_view(ctx, i)).collect();
        SlotPickerView {
            title: ctx
                .text(if self.is_saving() {
                    SAVE_TITLE
                } else {
                    LOAD_TITLE
                })
                .to_owned(),
            columns: SlotColumnsView {
                slot: ctx.text("save.column.slot").to_owned(),
                chapter: ctx.text("save.column.chapter").to_owned(),
                mode: ctx.text("save.column.mode").to_owned(),
                army: ctx.text("save.column.army").to_owned(),
                time: ctx.text("save.column.time").to_owned(),
            },
            slots,
            range,
            message: self.message.clone(),
            notice: self
                .imported
                .clone()
                .or_else(|| self.notice.shown().map(|n| n.text(ctx))),
            question: self.asking.as_ref().map(|ask| {
                let slot = self.slot_text();
                ask.view_with_keys(ctx, ctx.help_keys(), &[("slot", &slot)])
            }),
            export_question: self.exporting.as_ref().map(|ask| ask.view(ctx)),
            place: if self.is_saving() {
                None
            } else {
                files::place_view(ctx)
            },
            help: self.help(ctx),
        }
    }

    /// Slot `index` as the view shows it.
    fn slot_view(&self, ctx: &Ctx, index: usize) -> SlotView {
        let entry = match self.slots.get(index) {
            Some(Slot::Saved(saved)) => SlotEntryView::Saved {
                chapter: saved.title(ctx),
                mode: saved.mode,
                mode_name: mode_name(ctx, saved.mode).to_owned(),
                army: army_text(ctx, saved.roster),
                playtime: playtime_text(saved.playtime_s),
            },
            Some(Slot::Unreadable(e)) => SlotEntryView::Unreadable(e.text(ctx)),
            Some(Slot::Empty) | None => SlotEntryView::Empty(ctx.text(EMPTY_SLOT).to_owned()),
        };
        SlotView {
            number: index + 1,
            focused: index == self.cursor.focus(),
            entry,
        }
    }
}

/// A game mode's name, in `ctx`'s language.
pub fn mode_name(ctx: &Ctx, mode: GameMode) -> &str {
    ctx.text(match mode {
        GameMode::Classic => "save.mode.classic",
        GameMode::Casual => "save.mode.casual",
    })
}

/// An army of `n` as text: `1 unit`, `3 units`.
pub fn army_text(ctx: &Ctx, n: usize) -> String {
    let key = if n == 1 {
        "save.army.one"
    } else {
        "save.army.many"
    };
    ctx.text_with(key, &[("count", &n)])
}

impl Screen<Shuyi> for SlotPickerScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn actions(&self) -> Option<Vec<crate::input::Action>> {
        Some(crate::screens::actions::SAVE_SLOTS.to_vec())
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        if !core::mem::replace(&mut self.tip_asked, true)
            && let Some(tip) = self.export_tip(ctx)
        {
            return Transition::Push(Box::new(tip));
        }
        for &action in &input.actions {
            if self.is_asking_export() {
                self.answer_export(ctx, action);
                continue;
            }
            let done = if self.is_asking() {
                self.answer(ctx, action)
            } else {
                match action {
                    Action::CursorDown => {
                        self.step(ctx, true);
                        None
                    }
                    Action::CursorUp => {
                        self.step(ctx, false);
                        None
                    }
                    Action::Confirm => self.confirm(ctx),
                    Action::Info => {
                        // Only a save that loads: `can_export`.
                        self.ask_export(ctx);
                        None
                    }
                    Action::WorldFind if !self.is_saving() => {
                        files::open_folder(ctx, self.notice.shown());
                        None
                    }
                    files::IMPORT if !self.is_saving() => {
                        let cursor = self.focused_slot();
                        let import = ImportScreen::open(ctx, Channel::LoadGame { cursor });
                        import.map(|import| {
                            self.message = None;
                            self.imported = None;
                            Transition::Push(Box::new(import))
                        })
                    }
                    Action::Cancel => {
                        ctx.audio.menu(MenuSound::Cancel);
                        Some(Transition::Pop)
                    }
                    _ => None,
                }
            };
            if let Some(pop) = done {
                return pop;
            }
            if self.is_asking() && matches!(action, Action::Confirm | Action::Info) {
                // The key that opened the question isn't its answer.
                break;
            }
        }
        Transition::None
    }

    /// After an import (0873): the slots read again, the cursor on the
    /// slot the save went into, and the import's notice.
    fn uncovered(&mut self, ctx: &mut Ctx, result: Option<ScreenResult>) -> Transition {
        let Some(done) = result.and_then(ScreenResult::take::<ImportDone>) else {
            return Transition::None;
        };
        self.slots = save::slots(&ctx.storage, &ctx.game.content, ctx.words());
        if let Some(slot) = done.slot {
            self.cursor = self.cursor.clone().at(slot - 1);
            self.cursor.window(VISIBLE);
        }
        self.imported = Some(done.notice);
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint_slots(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn core::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
