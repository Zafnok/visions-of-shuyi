//! Help text that names keys and controller buttons, for this game's
//! screens: [`HelpKeys`] with this game's actions, and the helpers that
//! name an action's keys (generic, from
//! [`input::help_keys`](crate::input::help_keys)). Names always come from
//! the active [`Keymap`](crate::input::Keymap), never string literals
//! (ADR-0015, ADR-0031), and follow the device the player pressed last.

use crate::input::Action;

pub use crate::input::help_keys::{
    NOT_MAPPED, SEPARATOR, all_key_names, cursor_keys_name, help_line, key_name,
};

/// What help text names keys from, for this game's actions
/// ([`help_keys::HelpKeys`](crate::input::help_keys::HelpKeys)).
pub type HelpKeys<'a> = crate::input::help_keys::HelpKeys<'a, Action>;
