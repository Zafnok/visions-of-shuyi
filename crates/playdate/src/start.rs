//! The game as the Playdate starts it (ticket 1033): its console, palette
//! and buttons, no keyboard, the storage it is given. Plain code, so the
//! test that plays Chapter 1 against the Playdate's bundle
//! (`chapter_1.rs`, ticket 1036) starts the game as the device does.

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use trpg_content::PaletteDef;
use trpg_ui::input::{Device, PadKind};
use trpg_ui::screen::{IdleMotion, Keyboard, Lessons};
use trpg_ui::{ConsoleSizes, ConsoleSpec, Ctx, KeyPrompt, Palette, ShuyiCtx, Storage, UiColor};

/// Loads the game's data and sets the context up as the Playdate runs the
/// game, its settings read from `storage`; `seed` picks the title's music.
/// The caller logs the context's warnings (`take_warnings`).
pub fn context(storage: Box<dyn Storage>, seed: u64) -> Result<Ctx, String> {
    with_storage(data(seed)?, storage)
}

/// [`context`]'s first step: the game's data, and the context as the
/// Playdate runs the game, before the storage.
pub fn data(seed: u64) -> Result<Ctx, String> {
    let mut ctx = Ctx::embedded().map_err(|e| e.to_string())?;
    ctx.console = ConsoleSpec::PLAYDATE;
    ctx.platform.keyboard = Keyboard::Absent;
    ctx.device = Device::Pad(PadKind::Playdate);
    // Its own buttons (A + B and the B chords), before the player's are
    // loaded with the storage.
    ctx.use_playdate_buttons();
    // Its one-bit skins (ADR-0070 §4): the map's, and the screens' as they
    // come (1038 and later).
    ctx.use_playdate_skins();
    // The debug tools are made of the test files, which a build without
    // `std` does not have (ADR-0057).
    ctx.debug_tools = false;
    ctx.game.tips_enabled = true;
    ctx.game.lessons = Lessons::On;
    ctx.idle_motion = IdleMotion::On;
    ctx.music_seed = seed;
    ctx.key_prompt = KeyPrompt::Waiting;
    Ok(ctx)
}

/// [`context`]'s second step: the player's settings and saves, read from
/// `storage`, then the Playdate's palette.
pub fn with_storage(ctx: Ctx, storage: Box<dyn Storage>) -> Result<Ctx, String> {
    let mut ctx = ctx.with_storage(storage);
    // After the storage: loading the settings applies their colour theme.
    ctx.palette = palette(&ctx.game.content.palette)?;
    Ok(ctx)
}

/// The Playdate's palette (ADR-0074), over the default theme's names.
fn palette(base: &PaletteDef) -> Result<Palette, String> {
    let def = trpg_content::ink::load_playdate(base).map_err(|errors| {
        let errors: Vec<String> = errors.iter().map(ToString::to_string).collect();
        errors.join("; ")
    })?;
    // check-text: not player text (a build problem for the log)
    Palette::new::<UiColor>(&def)
        .map_err(|missing| alloc::format!("the Playdate palette lacks {missing:?}"))
}

#[cfg(test)]
mod tests {
    use trpg_content::Ink;
    use trpg_ui::screen::SkinSet;
    use trpg_ui::{MemoryStorage, Rgb};

    use super::*;

    /// The Playdate's console, buttons and guide, its palette of inks only,
    /// and the title waiting for a press.
    #[test]
    fn the_context_is_the_playdates() {
        let ctx = context(Box::new(MemoryStorage::new()), 7).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(ctx.console, ConsoleSpec::PLAYDATE);
        assert_eq!(ctx.platform.keyboard, Keyboard::Absent);
        assert_eq!(ctx.device, Device::Pad(PadKind::Playdate));
        assert!(!ctx.debug_tools);
        assert!(ctx.game.tips_enabled);
        assert_eq!(ctx.game.lessons, Lessons::On);
        assert_eq!(ctx.idle_motion, IdleMotion::On);
        assert_eq!(ctx.music_seed, 7);
        assert_eq!(ctx.key_prompt, KeyPrompt::Waiting);
        // Painted with its own skins, the settings just loaded or not.
        assert_eq!(ctx.game.skin_set, SkinSet::Playdate);
        assert_eq!(ctx.game.map_skin.name(), "playdate");
        assert!(ctx.palette.iter().count() > 10);
        for (name, rgb) in ctx.palette.iter() {
            let ink = Ink::ALL.iter().find(|ink| Rgb::from(ink.code()) == rgb);
            assert!(ink.is_some(), "{name} is no ink: {rgb:?}");
        }
    }
}
