//! The Profile screen (ticket 0837), opened from the title's `Profile`
//! item (`docs/design/options.md`, *Where options are kept*): Shuyi
//! Engine's profiles screen (`shuyi_ui::screens::profiles`, ticket 1164)
//! in this game's [`ProfileScreen`], which adds its saves outside the game
//! (0872, *Saves outside the game*): the export key exports the profile
//! under the cursor as a file ([`save_files`]), and on a desktop a line
//! says where the saves are, with a key that opens the folder; the import
//! key reads a file into a new profile ([`ImportScreen`], 0873). Adding a
//! profile opens the first-launch layout picker for it
//! ([`layout_picker::first`]). The first time it opens where a profile
//! can be exported, the guide's tip says so ([`TipScreen`], 0874). Its look is the engine's glyph skin
//! ([`glyph::paint`], ADR-0054) with the place line under it, or on the
//! Playdate [`playdate::paint`] ([`SKINS`], ADR-0083).

pub mod playdate;

use crate::prelude::*;
use core::ops::Deref;
pub use shuyi_ui::screens::profiles::*;

use super::layout_picker;
use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::profiles::MAX_PROFILES;
use crate::save_files::import::Channel;
use crate::save_files::{self, Export, ExportNotice, Exported};
use crate::screen::{Ctx, FrameInput, Screen, ScreenResult, Shuyi, Transition};
use crate::screens::Skins;
use crate::screens::files::{self, PlaceView};
use crate::screens::save_import::{ImportDone, ImportScreen};
use crate::tips::TipScreen;
use trpg_content::TipTrigger;

/// The engine's profiles screen on this game.
pub type ProfileList = ProfilesScreen<Shuyi>;

/// Text key of the help line on a profile that can be exported.
const HELP_EXPORT: &str = "profile.help.list_export";
/// Text key of that help line where the saves are in a folder.
const HELP_EXPORT_FOLDER: &str = "profile.help.list_export_folder";
/// Text key of the help line on the row that adds a profile, where a file
/// can be imported as one.
const HELP_ADD_IMPORT: &str = "profile.help.add_import";

/// The Profile screen: the engine's list ([`ProfileList`], which it
/// derefs to) with the export key and the place line.
#[derive(Debug)]
pub struct ProfileScreen {
    list: ProfileList,
    notice: ExportNotice,
    /// What the last import did, until the cursor moves.
    imported: Option<String>,
    /// Whether the guide's tip on exporting was looked at (ticket 0874):
    /// the first update does.
    tip_asked: bool,
}

impl Deref for ProfileScreen {
    type Target = ProfileList;

    fn deref(&self) -> &ProfileList {
        &self.list
    }
}

/// What the Profile screen shows, for a skin: the engine's list, its help
/// and message lines as this game words them, and where the saves are.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileView {
    /// The list.
    pub list: ProfilesView,
    /// Where the saves are: on a desktop.
    pub place: Option<PlaceView>,
}

/// The screen focused on the profile in use, opening the layout picker
/// once a profile is added.
pub fn screen(ctx: &Ctx) -> ProfileScreen {
    let list = ProfileList::new(ctx).opening_after_add(layout_picker_for_new);
    ProfileScreen {
        list,
        notice: ExportNotice::default(),
        imported: None,
        tip_asked: false,
    }
}

/// The first-launch layout picker, for the profile just added.
fn layout_picker_for_new(ctx: &mut Ctx) -> Box<dyn Screen<Shuyi>> {
    Box::new(layout_picker::first(ctx))
}

impl ProfileScreen {
    /// Name reported by [`Screen::name`]: the engine list's.
    pub const NAME: &'static str = ProfileList::NAME;

    /// Whether the list is on its own: no menu, name box or question open.
    fn list_alone(&self, ctx: &Ctx) -> bool {
        if self.list.naming().is_some() {
            return false;
        }
        let view = self.list.view(ctx);
        view.actions.is_none() && view.deleting.is_none() && view.leaving.is_none()
    }

    /// The profile under the cursor, if it can be exported here.
    fn exportable(&self, ctx: &Ctx) -> Option<crate::profiles::ProfileId> {
        match self.list.focused(ctx) {
            RowKind::Profile(id) if save_files::can_export(&ctx.platform) => Some(id),
            _ => None,
        }
    }

    /// Exports the profile under the cursor.
    fn export(&mut self, ctx: &mut Ctx) {
        let Some(id) = self.exportable(ctx) else {
            return;
        };
        if let Ok(file) = save_files::export(ctx, id, Export::Profile) {
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

    /// The screen as it is now, as plain data for a skin to paint.
    pub fn view(&self, ctx: &Ctx) -> ProfileView {
        let mut list = self.list.view(ctx);
        let place = files::place_view(ctx);
        if self.list_alone(ctx) && self.exportable(ctx).is_some() {
            let key = if place.is_some() {
                HELP_EXPORT_FOLDER
            } else {
                HELP_EXPORT
            };
            list.help = ctx.text_with(key, &[]);
        } else if self.list_alone(ctx)
            && self.list.focused(ctx) == RowKind::Add
            && ctx.profiles().all().len() < MAX_PROFILES
            && ImportScreen::can_open(ctx)
        {
            list.help = ctx.text_with(HELP_ADD_IMPORT, &[]);
        }
        if list.message.is_none() {
            list.message.clone_from(&self.imported);
        }
        if list.message.is_none() {
            list.message = self.notice.shown().map(|n| n.text(ctx));
        }
        ProfileView { list, place }
    }
}

impl Screen<Shuyi> for ProfileScreen {
    fn name(&self) -> &'static str {
        self.list.name()
    }

    fn actions(&self) -> Option<Vec<Action>> {
        let mut actions = self.list.actions()?;
        actions.extend_from_slice(files::ACTIONS);
        Some(actions)
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        if !core::mem::replace(&mut self.tip_asked, true)
            && save_files::can_export(&ctx.platform)
            && let Some(tip) = TipScreen::once(ctx, TipTrigger::FirstProfileScreen)
        {
            return Transition::Push(Box::new(tip));
        }
        let ours = input.actions.iter().any(|a| files::ACTIONS.contains(a));
        if ours && self.list_alone(ctx) {
            for &action in &input.actions {
                match action {
                    Action::Info => self.export(ctx),
                    Action::WorldFind => files::open_folder(ctx, self.notice.shown()),
                    files::IMPORT => {
                        if let Some(import) = ImportScreen::open(ctx, Channel::Profile) {
                            self.imported = None;
                            return Transition::Push(Box::new(import));
                        }
                    }
                    _ => {}
                }
            }
        }
        if input.actions.iter().any(|a| files::MOVES.contains(a)) {
            self.notice.clear();
            self.imported = None;
        }
        self.list.update(ctx, input)
    }

    fn uncovered(&mut self, _ctx: &mut Ctx, result: Option<ScreenResult>) -> Transition {
        if let Some(done) = result.and_then(ScreenResult::take::<ImportDone>) {
            self.imported = Some(done.notice);
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        let view = self.view(ctx);
        SKINS.paint(ctx, &view.list, buf);
        if let Some(place) = &view.place {
            files::paint_place(ctx, place, buf);
        }
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn core::any::Any> {
        Some(self)
    }
}

/// The screen's skins: the engine's glyph skin on the PC, and the
/// Playdate's.
pub const SKINS: Skins<ProfilesView> = Skins {
    glyph: glyph::paint::<Shuyi>,
    playdate: playdate::paint,
};

#[cfg(test)]
mod tests;
