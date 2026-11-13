//! The Chapter 1 demo's end (ticket 0914, `docs/design/demo.md`): after
//! Chapter 1's last scene, no "Save your progress?": "Thanks for playing,
//! {lead}" in silence, then "Export save?". *Yes* offers the campaign as
//! the full game's own save after Chapter 1 would hold it, a file the full
//! game imports (0873) and goes on to Chapter 2 from; then the title.

use crate::prelude::*;
use alloc::collections::BTreeMap;

use trpg_core::save::SaveFile;

use super::{FlowScreen, Stage};
use crate::save::{self, slot_key};
use crate::save_files::{self, Export};
use crate::screen::Ctx;
use crate::screens::game_over::DemoEndScreen;
use crate::screens::save::SavePromptScreen;

/// The slot the exported save is in, in the file: the first, as Load
/// Game's export of slot 1 names it.
const SLOT: usize = 1;

impl FlowScreen {
    /// The demo's end card, in silence.
    pub(super) fn demo_end(&mut self, ctx: &mut Ctx) {
        ctx.audio.stop_music();
        let lead = self.campaign.as_ref().map_or(&ctx.game.lead, |c| &c.lead);
        // check-text: not a data name (the player's own)
        let lead = lead.name.clone();
        self.show(Stage::DemoEnd, Box::new(DemoEndScreen::new(ctx, &lead)));
    }

    /// After the card: "Export save?".
    pub(super) fn demo_export_question(&mut self, ctx: &Ctx) {
        let question = SavePromptScreen::export_demo(ctx);
        self.show(Stage::DemoExport, Box::new(question));
    }

    /// *Yes*: offers the campaign's Chapter 1 save as one file, as Load
    /// Game exports slot 1 alone. The demo stores no save: the file is
    /// the only copy. Nothing is shown after it, so the flow ends.
    pub(super) fn demo_export(&self, ctx: &mut Ctx) {
        let Some(campaign) = &self.campaign else {
            return;
        };
        let Ok(text) = save::encode(&SaveFile::chapter_cleared(campaign.clone())) else {
            return;
        };
        let id = ctx.profiles().current();
        let name = ctx
            .profiles()
            .all()
            .iter()
            .find(|p| p.id == id)
            // check-text: not a data name (the player's own)
            .map(|p| p.name.clone())
            .unwrap_or_default();
        let what = Export::Slot {
            slot: SLOT,
            settings: false,
        };
        let file = save_files::export_name(&name, id, what);
        save_files::offer(ctx, &file, BTreeMap::from([(slot_key(SLOT), text)]));
    }
}
