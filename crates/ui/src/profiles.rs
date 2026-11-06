//! Player profiles (ticket 0836, ADR-0069; `docs/design/options.md`,
//! *Where options are kept*): a profile holds a player's options, keys,
//! tips seen, save slots and suspend save, each under its own key with
//! the profile's prefix (`p1-settings`, `p2-slot_03`). The mechanism is
//! Shuyi Engine's (`shuyi_ui::profiles`, ticket 1109), re-exported here;
//! this game adds the keys a player's data was kept under before it had
//! profiles ([`keys_from_before_profiles`]), which a first launch with
//! profiles moves into `Player 1`.

use crate::prelude::*;

pub use shuyi_ui::profiles::*;

use shuyi_ui::ctx::PROFILE_NUMBERED_TEXT;
use trpg_content::LangCode;

use crate::save::{self, LAST_SAVE_KEY, SLOTS, SUSPEND_KEY, slot_key};
use crate::screen::{Ctx, KEYBINDINGS_KEY, LAYOUT_KEY};
use crate::settings::{SETTINGS_KEY, Settings};
use crate::storage::Storage;
use crate::tips::TIPS_SEEN_KEY;

/// [`Storage`] key, in the profile, of the mark that it was set up
/// ([`SetUp::Kept`]): only whether it is there counts.
pub const SET_UP_KEY: &str = "set_up";

/// Whether the profile in use was ever set up (ticket 0842): renamed from
/// the number it was made with, an option kept, or a game saved. Until it
/// is, the title has the guide suggest the Profile screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetUp {
    /// Never: its name is the one it was made with, its settings are the
    /// defaults but for the layout, and it has no save.
    No,
    /// It is, from its data, but that isn't kept yet ([`keep_set_up`]).
    Seen,
    /// It is, for good: [`SET_UP_KEY`] is kept, so that deleting its last
    /// save (a suspend save, continued) or putting the options back
    /// doesn't make it new again.
    Kept,
}

impl SetUp {
    /// Whether the profile was set up.
    pub fn done(self) -> bool {
        self != Self::No
    }
}

/// Whether the profile in use in `ctx` was set up ([`SetUp`]). Reads the
/// storage: the title asks when it is shown, not each frame.
pub fn set_up(ctx: &Ctx) -> SetUp {
    let storage = &ctx.storage;
    if matches!(storage.read(SET_UP_KEY), Ok(Some(_))) {
        return SetUp::Kept;
    }
    let profiles = ctx.profiles();
    let renamed = profiles.get(profiles.current()).is_some_and(|p| {
        // Made as the engine names it: in English, whatever the language.
        let lang = &ctx.game.content.lang;
        let numbered = lang.text(&LangCode::default(), PROFILE_NUMBERED_TEXT);
        let n = profiles.current().get();
        // check-text: not a data name (the player's own)
        p.name != shuyi_ui::text::fill_values(numbered, &[("n", &n)])
    });
    // The layout picked on a first launch is saved in the settings too.
    let kept_options = storage
        .read(SETTINGS_KEY)
        .ok()
        .flatten()
        .and_then(|text| Settings::from_ron(&text).ok())
        .is_some_and(|s| Settings::default().with_layout_of(&s) != s);
    let saved = save::any_slot(storage) || save::has_suspend(storage);
    if renamed || kept_options || saved {
        SetUp::Seen
    } else {
        SetUp::No
    }
}

/// Keeps the mark that the profile in use in `storage` was set up
/// ([`SetUp::Kept`]).
///
/// # Errors
///
/// When the storage can't be written; the title tries again the next
/// time it is shown.
pub fn keep_set_up(storage: &mut dyn Storage) -> Result<(), crate::storage::StorageError> {
    storage.write(SET_UP_KEY, "1")
}

/// The keys a player's options, keys, tips seen and saves were kept under
/// before profiles (ADR-0050, ADR-0031, ADR-0039, ticket 0406), which
/// [`migrate`] moves into [`ProfileId::FIRST`].
pub fn keys_from_before_profiles() -> Vec<String> {
    let named = [
        SETTINGS_KEY,
        KEYBINDINGS_KEY,
        LAYOUT_KEY,
        TIPS_SEEN_KEY,
        SUSPEND_KEY,
        LAST_SAVE_KEY,
    ];
    let slots = (1..=SLOTS).map(slot_key);
    named.into_iter().map(str::to_owned).chain(slots).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::tests::ctx;

    /// Every key the game keeps for a player is a key of any profile, and
    /// all of them move into `Player 1`.
    #[test]
    fn every_key_from_before_profiles_fits_every_profile() {
        let keys = keys_from_before_profiles();
        assert_eq!(keys.len(), 6 + 30);
        assert_eq!(keys[..3], ["settings", "keybindings", "layout"]);
        assert_eq!(keys.last().map(String::as_str), Some("slot_30"));
        for key in &keys {
            assert!(fits_every_profile(key), "{key}");
        }
        assert!(fits_every_profile(SET_UP_KEY));
    }

    fn ok<T, E: core::fmt::Display>(result: Result<T, E>) -> T {
        result.unwrap_or_else(|e| panic!("{e}"))
    }

    /// Ticket 0842: a new profile isn't set up, a layout picked doesn't
    /// count, and each of a rename, an option kept and a game saved
    /// (a slot or a suspend save) does.
    #[test]
    fn a_profile_is_set_up_by_a_rename_an_option_kept_or_a_save() {
        let fresh = || {
            let mut c = ctx();
            ok(c.choose_layout(crate::input::RIGHT_HANDED));
            c
        };
        assert_eq!(set_up(&fresh()), SetUp::No);
        let mut c = fresh();
        ok(c.rename_profile(ProfileId::FIRST, "Mei"));
        assert_eq!(set_up(&c), SetUp::Seen);
        let mut c = fresh();
        c.preview_settings(|s| s.music_volume = 3);
        assert_eq!(set_up(&c), SetUp::No, "previewed, not kept");
        ok(c.keep_settings());
        assert_eq!(set_up(&c), SetUp::Seen);
        for key in [slot_key(4), SUSPEND_KEY.to_owned()] {
            let mut c = fresh();
            ok(c.storage.write(&key, "anything"));
            assert_eq!(set_up(&c), SetUp::Seen, "{key}");
        }
    }

    /// Once kept, it stays set up whatever its data says: its suspend save
    /// continued, its options put back.
    #[test]
    fn a_kept_set_up_is_for_good() {
        let mut c = ctx();
        ok(c.storage.write(SUSPEND_KEY, "anything"));
        ok(keep_set_up(&mut c.storage));
        ok(c.storage.delete(SUSPEND_KEY));
        assert_eq!(set_up(&c), SetUp::Kept);
        assert!(SetUp::Kept.done() && SetUp::Seen.done() && !SetUp::No.done());
    }

    /// Another profile has its own: a new one isn't set up, and setting
    /// it up leaves the first one as it was.
    #[test]
    fn each_profile_is_set_up_on_its_own() {
        let mut c = ctx();
        ok(keep_set_up(&mut c.storage));
        let second = ok(c.add_profile("Player 2"));
        ok(c.switch_profile(second));
        assert_eq!(set_up(&c), SetUp::No);
        ok(c.rename_profile(second, "Mei"));
        assert_eq!(set_up(&c), SetUp::Seen);
        let mut c = ctx();
        let second = ok(c.add_profile("Player 2"));
        ok(c.switch_profile(second));
        ok(keep_set_up(&mut c.storage));
        ok(c.switch_profile(ProfileId::FIRST));
        assert_eq!(set_up(&c), SetUp::No);
    }
}
