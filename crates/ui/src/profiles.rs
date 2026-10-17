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

use crate::save::{LAST_SAVE_KEY, SLOTS, SUSPEND_KEY, slot_key};
use crate::screen::{KEYBINDINGS_KEY, LAYOUT_KEY};
use crate::settings::SETTINGS_KEY;
use crate::tips::TIPS_SEEN_KEY;

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
    }
}
