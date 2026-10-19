//! The Profile screen (ticket 0837), opened from the title's `Profile`
//! item (`docs/design/options.md`, *Where options are kept*): the player
//! profiles, the one in use marked, then a row that adds one.
//!
//! Confirm on a profile opens what can be done to it: use it (the title
//! then shows its options and saves), rename it, delete it (asking first,
//! focus on No; never the only profile). Adding asks for a name, uses the
//! new profile and opens the first-launch layout picker for it; it starts
//! from the default options. A name is typed on the keyboard or spelled on
//! the letter grid with a controller ([`crate::widgets::name_entry`]), up
//! to [`MAX_NAME_CHARS`], not another profile's. Cancel in a name box with
//! a name typed and not kept asks whether to keep it, throw it away or go
//! back, focus on going back (*Keeping changed options is the player's
//! choice*).
//!
//! **The look is a skin** (ADR-0054): this module says what the screen
//! shows as a [`ProfileView`] ([`ProfileScreen::view`]); [`glyph::paint`]
//! draws it, or on the Playdate [`playdate::paint`] ([`SKINS`], ADR-0083).
//! Tests of what happened read the view or the profiles.

pub mod glyph;
pub mod playdate;
pub mod view;

use crate::prelude::*;
pub use view::{ActionsView, ChoiceView, ProfileView, QuestionView, RowView};

use super::options::LeaveChoice;
use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::{Action, text_keys_help};
use crate::profiles::{MAX_NAME_CHARS, MAX_PROFILES, ProfileError, ProfileId};
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::screens::Skins;
use crate::widgets::name_entry::{NameBox, NameEntry, NameEvent, NameRules, finished};

/// The screen's skins: the PC's and the Playdate's.
pub const SKINS: Skins<ProfileView> = Skins {
    glyph: glyph::paint,
    playdate: playdate::paint,
};

/// Text key ([`Ctx::text`]) of the screen title.
pub const TITLE: &str = "profile.title";
/// Text key of the words marking the profile in use.
pub const CURRENT: &str = "profile.current";
/// Text key of the row that adds a profile.
pub const ADD: &str = "profile.add";
/// Text key of the name box's title.
pub const NAME_BOX: &str = "profile.name_box";
/// Text key of the question asked before deleting.
pub const DELETE_QUESTION: &str = "profile.question.delete";
/// Text key of the question asked on backing out of a name box.
pub const KEEP_QUESTION: &str = "profile.question.keep";
/// Text key of the message when the name is another profile's.
pub const NAME_TAKEN_MESSAGE: &str = "profile.message.name_taken";
/// Text key of the message when there are the most profiles already.
pub const TOO_MANY_MESSAGE: &str = "profile.message.too_many";
/// Text key of the message if the profiles couldn't be saved.
pub const NOT_SAVED_MESSAGE: &str = "profile.message.not_saved";

/// What a profile's name may hold: the lead's letters, and digits (the
/// game names profiles `Player 2`).
pub const NAME_RULES: NameRules = NameRules {
    max_chars: MAX_NAME_CHARS,
    digits: true,
};

/// One row of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    /// A profile.
    Profile(ProfileId),
    /// Adds a profile.
    Add,
}

/// What can be done to a profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileAction {
    /// Use it (not offered for the one in use).
    Switch,
    /// Rename it.
    Rename,
    /// Delete it (not offered for the only profile).
    Delete,
}

impl ProfileAction {
    /// The text key of its name.
    pub const fn key(self) -> &'static str {
        match self {
            ProfileAction::Switch => "profile.action.switch",
            ProfileAction::Rename => "profile.action.rename",
            ProfileAction::Delete => "profile.action.delete",
        }
    }
}

/// An answer to the question asked before deleting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteChoice {
    /// Delete it.
    Yes,
    /// Don't.
    No,
}

impl DeleteChoice {
    /// Every answer, in the order shown.
    pub const ALL: [DeleteChoice; 2] = [DeleteChoice::Yes, DeleteChoice::No];

    /// The text key of its name.
    pub const fn key(self) -> &'static str {
        match self {
            DeleteChoice::Yes => "profile.answer.yes",
            DeleteChoice::No => "profile.answer.no",
        }
    }
}

/// What a name box is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamePurpose {
    /// A new profile.
    Add,
    /// A new name for this profile.
    Rename(ProfileId),
}

/// Where the name is typed.
#[derive(Debug, Clone, PartialEq, Eq)]
enum NameInput {
    /// On the keyboard.
    Typing(NameBox),
    /// On the letter grid (a controller).
    Grid(NameEntry),
}

impl NameInput {
    fn name(&self) -> &str {
        match self {
            NameInput::Typing(b) => b.name(),
            NameInput::Grid(e) => e.name(),
        }
    }
}

/// An open name box.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Naming {
    purpose: NamePurpose,
    /// The name it opened with: the profile's, or nothing for a new one.
    original: String,
    input: NameInput,
    /// The answer in focus while the question asked on backing out is
    /// open.
    leaving: Option<LeaveChoice>,
}

impl Naming {
    /// Whether a name is typed that isn't kept: not blank, and not the one
    /// the box opened with.
    fn unkept(&self) -> bool {
        finished(self.input.name()).is_some_and(|name| name != self.original)
    }
}

/// The profiles, the one in use marked, and a row to add one. Cancel goes
/// back to the title.
#[derive(Debug, Clone)]
pub struct ProfileScreen {
    /// The focused row: a profile's place in the list, or the list's
    /// length for the row that adds one.
    focus: usize,
    /// The focused item while the focused profile's menu is open.
    actions: Option<usize>,
    /// The name box, while open.
    naming: Option<Naming>,
    /// The profile asked about and the answer in focus, while the question
    /// asked before deleting is open.
    deleting: Option<(ProfileId, DeleteChoice)>,
    /// The text key of the message under the panel, if any.
    message: Option<&'static str>,
}

impl ProfileScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "profile";

    /// The screen focused on the profile in use.
    pub fn new(ctx: &Ctx) -> Self {
        let profiles = ctx.profiles();
        let current = profiles
            .all()
            .iter()
            .position(|p| p.id == profiles.current());
        Self {
            focus: current.unwrap_or(0),
            actions: None,
            naming: None,
            deleting: None,
            message: None,
        }
    }

    /// Every row, top to bottom.
    pub fn rows(ctx: &Ctx) -> Vec<RowKind> {
        let profiles = ctx.profiles().all().iter();
        let mut rows: Vec<RowKind> = profiles.map(|p| RowKind::Profile(p.id)).collect();
        rows.push(RowKind::Add);
        rows
    }

    /// The focused row.
    pub fn focused(&self, ctx: &Ctx) -> RowKind {
        let rows = Self::rows(ctx);
        rows.get(self.focus).copied().unwrap_or(RowKind::Add)
    }

    /// The focused profile, if a profile row is focused.
    fn focused_profile(&self, ctx: &Ctx) -> Option<ProfileId> {
        match self.focused(ctx) {
            RowKind::Profile(id) => Some(id),
            RowKind::Add => None,
        }
    }

    /// What can be done to `id`: use it unless it is in use, rename it,
    /// delete it unless it is the only one.
    pub fn actions_for(ctx: &Ctx, id: ProfileId) -> Vec<ProfileAction> {
        let profiles = ctx.profiles();
        let mut actions = Vec::new();
        if profiles.current() != id {
            actions.push(ProfileAction::Switch);
        }
        actions.push(ProfileAction::Rename);
        if profiles.all().len() > 1 {
            actions.push(ProfileAction::Delete);
        }
        actions
    }

    /// Whether another profile may be added.
    fn can_add(ctx: &Ctx) -> bool {
        ctx.profiles().all().len() < MAX_PROFILES
    }

    /// What the name box is for, while it is open.
    pub fn naming(&self) -> Option<NamePurpose> {
        self.naming.as_ref().map(|n| n.purpose)
    }

    /// The name typed so far, while the name box is open.
    pub fn typed(&self) -> Option<&str> {
        self.naming.as_ref().map(|n| n.input.name())
    }

    /// The text key of the message under the panel, if any.
    pub fn message(&self) -> Option<&str> {
        self.message
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint`]).
    pub fn view(&self, ctx: &Ctx) -> ProfileView {
        let profiles = ctx.profiles();
        let can_add = Self::can_add(ctx);
        let rows = Self::rows(ctx).into_iter().enumerate().map(|(i, kind)| {
            let focused = i == self.focus;
            match kind {
                RowKind::Profile(id) => RowView {
                    kind,
                    label: profile_name(ctx, id),
                    current: id == profiles.current(),
                    focused,
                    enabled: true,
                },
                RowKind::Add => RowView {
                    kind,
                    label: ctx.text(ADD).to_owned(),
                    current: false,
                    focused,
                    enabled: can_add,
                },
            }
        });
        let actions = self.actions.zip(self.focused_profile(ctx)).map(|(at, id)| {
            let items = Self::actions_for(ctx, id).into_iter().enumerate();
            ActionsView {
                title: profile_name(ctx, id),
                items: items
                    .map(|(i, action)| ChoiceView {
                        label: ctx.text(action.key()).to_owned(),
                        focused: i == at,
                    })
                    .collect(),
            }
        });
        let title = ctx.text(NAME_BOX);
        let (typing, grid) = match self.naming.as_ref().map(|n| &n.input) {
            Some(NameInput::Typing(b)) => (Some(b.view(ctx, title)), None),
            Some(NameInput::Grid(e)) => (None, Some(e.view(ctx, title))),
            None => (None, None),
        };
        ProfileView {
            title: ctx.text(TITLE).to_owned(),
            rows: rows.collect(),
            current_mark: ctx.text(CURRENT).to_owned(),
            actions,
            typing,
            grid,
            question: self.question(ctx),
            message: self.message.map(|key| match key {
                TOO_MANY_MESSAGE => ctx.text_with(key, &[("n", &MAX_PROFILES)]),
                key => ctx.text(key).to_owned(),
            }),
            help: self.help(ctx),
        }
    }

    /// The question open, if any, as the view shows it.
    fn question(&self, ctx: &Ctx) -> Option<QuestionView> {
        let choice = |label: &str, focused| ChoiceView {
            label: label.to_owned(),
            focused,
        };
        if let Some(focus) = self.naming.as_ref().and_then(|n| n.leaving) {
            let choices = LeaveChoice::ALL.into_iter();
            return Some(QuestionView {
                text: ctx.text(KEEP_QUESTION).to_owned(),
                choices: choices
                    .map(|c| choice(ctx.text(c.key()), c == focus))
                    .collect(),
            });
        }
        let (id, focus) = self.deleting?;
        let name = profile_name(ctx, id);
        let choices = DeleteChoice::ALL.into_iter();
        Some(QuestionView {
            text: ctx.text_with(DELETE_QUESTION, &[("name", &name)]),
            choices: choices
                .map(|c| choice(ctx.text(c.key()), c == focus))
                .collect(),
        })
    }

    /// The bottom help line, naming the keys of the active keymap (or the
    /// text box's own keys while typing).
    pub fn help(&self, ctx: &Ctx) -> String {
        let key = match &self.naming {
            Some(n) if n.leaving.is_some() => "profile.help.question",
            Some(Naming {
                input: NameInput::Typing(_),
                ..
            }) => return text_keys_help(ctx),
            Some(_) => "name_entry.help.grid",
            None if self.deleting.is_some() => "profile.help.question",
            None if self.actions.is_some() => "profile.help.actions",
            None => match self.focused(ctx) {
                RowKind::Add if Self::can_add(ctx) => "profile.help.add",
                RowKind::Add => "profile.help.none",
                RowKind::Profile(_) => "profile.help.list",
            },
        };
        ctx.text_with(key, &[])
    }

    /// Opens the name box for `purpose`, starting from `name`: the letter
    /// grid if a controller button opened it (`pad`), else the typing box.
    fn open_name(&mut self, ctx: &mut Ctx, purpose: NamePurpose, name: &str, pad: bool) {
        let input = if pad {
            NameInput::Grid(NameEntry::new(name, NAME_RULES))
        } else {
            NameInput::Typing(NameBox::new(name, NAME_RULES))
        };
        self.naming = Some(Naming {
            purpose,
            original: name.to_owned(),
            input,
            leaving: None,
        });
        ctx.audio.menu(MenuSound::Select);
    }

    /// Confirm on the list (`pad`: with a controller button).
    fn confirm(&mut self, ctx: &mut Ctx, pad: bool) {
        match self.focused(ctx) {
            RowKind::Profile(_) => {
                self.actions = Some(0);
                ctx.audio.menu(MenuSound::Select);
            }
            RowKind::Add if Self::can_add(ctx) => self.open_name(ctx, NamePurpose::Add, "", pad),
            RowKind::Add => {
                self.message = Some(TOO_MANY_MESSAGE);
                ctx.audio.menu(MenuSound::Denied);
            }
        }
    }

    /// An action while the focused profile's menu is open on item `at`.
    fn act(&mut self, ctx: &mut Ctx, at: usize, action: Action, pad: bool) -> Transition {
        let Some(id) = self.focused_profile(ctx) else {
            self.actions = None;
            return Transition::None;
        };
        let actions = Self::actions_for(ctx, id);
        match action {
            Action::CursorUp | Action::CursorDown => {
                let n = actions.len();
                let to = if action == Action::CursorDown {
                    (at + 1) % n
                } else {
                    (at + n - 1) % n
                };
                self.actions = Some(to);
                ctx.audio.menu(MenuSound::Move);
            }
            Action::Cancel => {
                self.actions = None;
                ctx.audio.menu(MenuSound::Cancel);
            }
            Action::Confirm => {
                self.actions = None;
                match actions.get(at) {
                    Some(ProfileAction::Switch) => return Self::switch(ctx, id),
                    Some(ProfileAction::Rename) => {
                        let name = profile_name(ctx, id);
                        self.open_name(ctx, NamePurpose::Rename(id), &name, pad);
                    }
                    Some(ProfileAction::Delete) => {
                        self.deleting = Some((id, DeleteChoice::No));
                        ctx.audio.menu(MenuSound::Select);
                    }
                    None => {}
                }
            }
            _ => {}
        }
        Transition::None
    }

    /// Uses the profile `id` and goes back to the title, which then shows
    /// its options and saves.
    fn switch(ctx: &mut Ctx, id: ProfileId) -> Transition {
        // The switch happens even if the list can't be saved (the next
        // launch then opens on the profile used before).
        let _ = ctx.switch_profile(id);
        ctx.audio.menu(MenuSound::Select);
        Transition::Pop
    }

    /// The message for `error`, refused by the profiles.
    fn refused(&mut self, ctx: &mut Ctx, error: &ProfileError) {
        self.message = Some(match error {
            ProfileError::NameTaken => NAME_TAKEN_MESSAGE,
            ProfileError::TooMany => TOO_MANY_MESSAGE,
            _ => NOT_SAVED_MESSAGE,
        });
        ctx.audio.menu(MenuSound::Denied);
    }

    /// Keeps the name typed: adds the profile (then uses it and opens the
    /// layout picker for it) or renames one. A name the profiles refuse
    /// leaves the box open with a message.
    fn keep_name(&mut self, ctx: &mut Ctx) -> Transition {
        let Some(naming) = &self.naming else {
            return Transition::None;
        };
        let Some(name) = finished(naming.input.name()) else {
            ctx.audio.menu(MenuSound::Denied);
            return Transition::None;
        };
        match naming.purpose {
            NamePurpose::Rename(id) => {
                if let Err(e) = ctx.rename_profile(id, &name) {
                    self.refused(ctx, &e);
                    return Transition::None;
                }
            }
            NamePurpose::Add => match ctx.add_profile(&name) {
                Ok(id) => {
                    self.naming = None;
                    self.message = None;
                    let _ = ctx.switch_profile(id);
                    // The first-launch setup, for the new profile; then
                    // back to the title, like a switch.
                    return Transition::Replace(Box::new(super::layout_picker::first(ctx)));
                }
                Err(e) => {
                    self.refused(ctx, &e);
                    return Transition::None;
                }
            },
        }
        self.naming = None;
        self.message = None;
        Transition::None
    }

    /// The name box asked to close (Cancel): with a name typed and not
    /// kept it asks first, focused on going back; otherwise it closes.
    fn cancel_name(&mut self, ctx: &mut Ctx) {
        let Some(naming) = &mut self.naming else {
            return;
        };
        if naming.unkept() {
            naming.leaving = Some(LeaveChoice::Back);
            ctx.audio.menu(MenuSound::Select);
        } else {
            self.naming = None;
            self.message = None;
            ctx.audio.menu(MenuSound::Cancel);
        }
    }

    /// One frame of the name box (no question open).
    fn type_name(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        let Some(naming) = &mut self.naming else {
            return Transition::None;
        };
        let event = match &mut naming.input {
            NameInput::Typing(b) => b.handle(input, ctx),
            NameInput::Grid(e) => {
                let mut event = NameEvent::Open;
                for &action in &input.actions {
                    event = e.handle(action, ctx);
                    if event != NameEvent::Open {
                        break;
                    }
                }
                event
            }
        };
        match event {
            NameEvent::Open => Transition::None,
            NameEvent::Done => self.keep_name(ctx),
            NameEvent::Cancel => {
                self.cancel_name(ctx);
                Transition::None
            }
        }
    }

    /// An action while the question asked on backing out of the name box
    /// is open, focused on `focus`: the cursor moves between the answers,
    /// Confirm picks one, Cancel goes back to the box. Returns the
    /// transition and whether the question closed.
    fn answer_leave(
        &mut self,
        ctx: &mut Ctx,
        focus: LeaveChoice,
        action: Action,
    ) -> (Transition, bool) {
        let choice = match action {
            Action::CursorUp | Action::CursorLeft | Action::CursorDown | Action::CursorRight => {
                let forward = matches!(action, Action::CursorDown | Action::CursorRight);
                let to = stepped(&LeaveChoice::ALL, focus, forward);
                if to != focus {
                    self.set_leaving(Some(to));
                    ctx.audio.menu(MenuSound::Move);
                }
                return (Transition::None, false);
            }
            Action::Confirm => focus,
            Action::Cancel => LeaveChoice::Back,
            _ => return (Transition::None, false),
        };
        self.set_leaving(None);
        match choice {
            LeaveChoice::Keep => (self.keep_name(ctx), true),
            LeaveChoice::Discard => {
                self.naming = None;
                self.message = None;
                ctx.audio.menu(MenuSound::Cancel);
                (Transition::None, true)
            }
            LeaveChoice::Back => {
                ctx.audio.menu(MenuSound::Cancel);
                (Transition::None, true)
            }
        }
    }

    fn set_leaving(&mut self, leaving: Option<LeaveChoice>) {
        if let Some(naming) = &mut self.naming {
            naming.leaving = leaving;
        }
    }

    /// An action while the question asked before deleting `id` is open,
    /// focused on `focus`. Returns whether the question closed.
    fn answer_delete(
        &mut self,
        ctx: &mut Ctx,
        id: ProfileId,
        focus: DeleteChoice,
        action: Action,
    ) -> bool {
        let choice = match action {
            Action::CursorUp | Action::CursorLeft | Action::CursorDown | Action::CursorRight => {
                let forward = matches!(action, Action::CursorDown | Action::CursorRight);
                let to = stepped(&DeleteChoice::ALL, focus, forward);
                if to != focus {
                    self.deleting = Some((id, to));
                    ctx.audio.menu(MenuSound::Move);
                }
                return false;
            }
            Action::Confirm => focus,
            Action::Cancel => DeleteChoice::No,
            _ => return false,
        };
        self.deleting = None;
        if choice == DeleteChoice::No {
            ctx.audio.menu(MenuSound::Cancel);
            return true;
        }
        match ctx.delete_profile(id) {
            Ok(()) => ctx.audio.menu(MenuSound::Select),
            Err(e) => self.refused(ctx, &e),
        }
        // A row fewer: the focus stays in the list.
        self.focus = self.focus.min(ctx.profiles().all().len().saturating_sub(1));
        true
    }

    /// Moves the focus one row up or down, wrapping.
    fn move_row(&mut self, ctx: &mut Ctx, down: bool) {
        let n = Self::rows(ctx).len();
        self.focus = if down {
            (self.focus + 1) % n
        } else {
            (self.focus + n - 1) % n
        };
        self.message = None;
        ctx.audio.menu(MenuSound::Move);
    }
}

/// The name the player gave the profile `id` (empty if there is none).
fn profile_name(ctx: &Ctx, id: ProfileId) -> String {
    let profile = ctx.profiles().get(id);
    // check-text: not a data name (the player typed it)
    profile.map(|p| p.name.clone()).unwrap_or_default()
}

/// `all`'s value one step after (`forward`) or before `current`, stopping
/// at the ends.
fn stepped<T: Copy + PartialEq>(all: &[T], current: T, forward: bool) -> T {
    let i = all.iter().position(|&v| v == current).unwrap_or(0);
    let to = if forward { i + 1 } else { i.saturating_sub(1) };
    all.get(to).copied().unwrap_or(current)
}

impl Screen<Shuyi> for ProfileScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        let typing = self.naming.as_ref().is_some_and(|n| n.leaving.is_none());
        if typing {
            // The game's keys do nothing while the box is open (on the
            // grid they are its cursor).
            return self.type_name(ctx, input);
        }
        for &action in &input.actions {
            if let Some(focus) = self.naming.as_ref().and_then(|n| n.leaving) {
                let (transition, closed) = self.answer_leave(ctx, focus, action);
                if closed {
                    // The answer ends this frame's keys.
                    return transition;
                }
                continue;
            }
            if let Some((id, focus)) = self.deleting {
                if self.answer_delete(ctx, id, focus, action) {
                    break;
                }
                continue;
            }
            if let Some(at) = self.actions {
                let transition = self.act(ctx, at, action, input.pad_pressed());
                if !matches!(transition, Transition::None) {
                    return transition;
                }
                if self.actions.is_none() {
                    // The key that chose isn't the next menu's.
                    break;
                }
                continue;
            }
            match action {
                Action::CursorUp => self.move_row(ctx, false),
                Action::CursorDown => self.move_row(ctx, true),
                Action::Confirm => {
                    self.confirm(ctx, input.pad_pressed());
                    // The key that opened something isn't its first.
                    if self.actions.is_some() || self.naming.is_some() {
                        break;
                    }
                }
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
        SKINS.paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
