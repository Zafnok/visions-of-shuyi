//! The game's palettes and dialogue reloading as their files change, in a
//! build with debug tools (ticket 1209, ADR-0124). Under `cargo shuyi run
//! --watch` the game reads its folders on disk (`crates/app`), the
//! platform says which files changed, [`HotContent`] puts them in the
//! game's view of the folders (`bundle::take_changes`) and loads the content
//! again when a palette or a `.dlg` file is among them. It takes the
//! palettes and the dialogue from it and nothing else: a battle's rules
//! data changes only by `Command` (ADR-0004), so the maps, battles and the
//! rest keep what the game started with. When the content fails to load,
//! the old palettes and scenes stay, and its errors, each with its file
//! and line, show over the frame until a load succeeds.

use crate::prelude::*;
use shuyi_assets::hot::Reload;
use shuyi_ui::module::{Hook, Install, Module};
use trpg_content::bundle;
use trpg_content::dialogue::DIALOGUE_DIR;
use trpg_content::{Content, ContentErrors, Theme};

use crate::color::{Palette, UiColor};
use crate::game::Game;
use crate::screen::{Ctx, DEBUG_TOOLS, Shuyi};

/// What a failed load is noted as (`Ctx::note_reload`).
pub const ID: &str = "content";

/// The module: no state of its own, since what it reloads is the
/// content's.
#[derive(Debug, Default)]
pub struct HotContent;

impl HotContent {
    /// `game` with the module installed, in a build with debug tools;
    /// else `game` as it is.
    ///
    /// # Errors
    ///
    /// The game has a module of its name already.
    pub fn install(game: Game) -> Result<Game, String> {
        if !DEBUG_TOOLS {
            return Ok(game);
        }
        game.with_module(Self).map_err(|e| e.to_string())
    }
}

/// Whether a change to the file at `path` (in the bundle) reloads: a
/// theme's palette, or a scene file.
pub fn reloads(path: &str) -> bool {
    let scene = path
        .strip_prefix(DIALOGUE_DIR)
        .and_then(|file| file.strip_prefix('/'))
        .and_then(|file| file.rsplit_once('.'))
        .is_some_and(|(_, ext)| ext == "dlg");
    scene || Theme::ALL.iter().any(|theme| theme.path() == path)
}

impl Module<Shuyi> for HotContent {
    fn name(&self) -> &'static str {
        "hot_content"
    }

    fn install(&mut self, install: &mut Install<'_, Shuyi>) {
        install.hook(Hook::Reloaded);
    }

    fn reloaded(&mut self, ctx: &mut Ctx, changed: &[String]) {
        // engine-friction: 1267 (a `read_folder` that leaves files out)
        bundle::take_changes(changed);
        if !changed.iter().any(|path| reloads(path)) {
            return;
        }
        let reload = match trpg_content::load_embedded() {
            Ok(content) => {
                take(ctx, content);
                Reload::Reloaded
            }
            Err(ContentErrors(errors)) => Reload::Failed(errors),
        };
        ctx.note_reload(ID, &reload);
    }
}

/// Takes the palettes and the dialogue of `content`, and paints with the
/// palette of the theme in the settings from the next frame on.
fn take(ctx: &mut Ctx, content: Content) {
    let ours = &mut ctx.game.content;
    ours.palette = content.palette;
    ours.themes = content.themes;
    ours.dialogue = content.dialogue;
    ours.stand_ins = content.stand_ins;
    let theme = ctx.settings().theme;
    let def = ctx.game.content.themes.get(&theme);
    if let Some(palette) = def.and_then(|def| Palette::new::<UiColor>(def).ok()) {
        ctx.palette = palette;
    }
}
