//! This game's input (ADR-0006, ADR-0034): the input mechanism is Shuyi
//! Engine's (`shuyi_ui::input`, tickets 1106 and 1107), generic over a
//! game's [`ActionSet`]; its submodules are re-exported here under their
//! old names. This module is the game's: the aliases with this game's
//! [`Action`] ([`Keymap`], [`InputState`], [`LayoutBindings`], …), the
//! rows of the Key bindings screen and the text keys of its words.
//!
//! [`Key`], [`Chord`], [`Button`] and [`Layout`] are the engine's keymap
//! file's; [`Action`] and the layouts are `trpg-content`'s. All are
//! re-exported here.
//!
//! **Fixed keys** (`docs/design/controls.md`, *Rebinding keys*; ADR-0031):
//! plain `Escape` is Cancel in every keymap (this game's
//! [`fixed_keys`](trpg_content::keymap::ActionSet::fixed_keys)), and
//! `Delete` does nothing in play (the Key bindings screen reads it to empty
//! a slot). Neither can be bound; see [`Chord::is_reserved`].
//! `Shift+Escape` and `Shift+Delete` are ordinary chords.

use crate::prelude::*;
use trpg_content::keymap::format;

use crate::screens::key_bindings::KeyRow;
use crate::screens::key_bindings::RowGroup::{self, Optional, Required};

pub use shuyi_ui::input::{bindings, crank, fixed, frame, help_keys, keymap, pad, state};

pub use bindings::{ButtonSlots, Slots};
pub use crank::Crank;
pub use fixed::{
    TextKey, capture_abort_key_name, clear_slot_key_name, is_capture_abort, is_clear_slot, text_key,
};
pub use help_keys::ARROWS;
pub use pad::{ButtonSet, PadId, PadKind, PadState, Pads};
pub use state::{Device, Input, MAX_REPEATS_PER_UPDATE};
pub use trpg_content::keymap::{
    Action, ActionSet, Button, ButtonChord, Chord, ChordDef, CrankDef, Key, KeyGroup, KeymapDef,
    LEFT_HANDED, Layout, LayoutKeys, PadKeys, RIGHT_HANDED, RepeatDef, SLOTS, StickDef, UiAction,
    load_keymap,
};

/// This game's chord and button lookup ([`keymap::Keymap`]).
pub type Keymap = keymap::Keymap<Action>;

/// This game's held keys and repeat ([`state::InputState`]).
pub type InputState = state::InputState<Action>;

/// One layout's player-edited keys ([`bindings::LayoutBindings`]).
pub type LayoutBindings = bindings::LayoutBindings<Action>;

/// The player-edited controller buttons ([`bindings::PadBindings`]).
pub type PadBindings = bindings::PadBindings<Action>;

/// Every layout's player keys and the buttons, as saved
/// ([`bindings::PlayerKeys`]).
pub type PlayerKeys = bindings::PlayerKeys<Action>;

/// Why a bind was refused ([`bindings::BindError`]).
pub type BindError = bindings::BindError<Action>;

/// One layout's default chord lookup ([`format::Bindings`]).
pub type Bindings = format::Bindings<Action>;

/// The text keys of [`ARROWS`] and of what joins a button chord's two
/// buttons in help text (`A+B`, ticket 1034), in the player's language:
/// the engine's ([`shuyi_ui::Ctx::help_keys`]).
pub use shuyi_ui::ctx::{ARROWS_TEXT, CHORD_JOIN_TEXT};

/// The text keys of what a slot on the Key bindings screen shows while it
/// waits for a key or a controller button (`docs/design/controls.md`,
/// *Rebinding keys* and *Rebinding buttons*): the engine's.
pub use shuyi_ui::screens::key_bindings::{CAPTURE_BUTTON_PROMPT, CAPTURE_PROMPT};

/// This game's rows on the Key bindings screen: every rebindable action
/// but the [crank's](CRANK_ACTIONS), with the text key of its player-facing label (`Unit info`) and its
/// group, in the order the screen lists them (Nick's pick, ticket 0815):
/// the required actions, then the optional ones, as in `controls.md`'s
/// *Required and optional* table, then the dialogue log's own key group.
pub const KEY_ROWS: [KeyRow; 18] = [
    (Action::CursorUp, "key_bindings.action.cursor_up", Required),
    (
        Action::CursorDown,
        "key_bindings.action.cursor_down",
        Required,
    ),
    (
        Action::CursorLeft,
        "key_bindings.action.cursor_left",
        Required,
    ),
    (
        Action::CursorRight,
        "key_bindings.action.cursor_right",
        Required,
    ),
    (Action::Confirm, "key_bindings.action.confirm", Required),
    (Action::Cancel, "key_bindings.action.cancel", Required),
    (Action::EndTurn, "key_bindings.action.end_turn", Required),
    (Action::Select, "key_bindings.action.select", Optional),
    (
        Action::ConfirmEndTurn,
        "key_bindings.action.confirm_end_turn",
        Optional,
    ),
    (Action::PrevUnit, "key_bindings.action.prev_unit", Optional),
    (Action::NextUnit, "key_bindings.action.next_unit", Optional),
    (Action::Info, "key_bindings.action.info", Optional),
    (
        Action::DangerZone,
        "key_bindings.action.danger_zone",
        Optional,
    ),
    (
        Action::ToggleAutoEnd,
        "key_bindings.action.toggle_auto_end",
        Optional,
    ),
    (Action::Rewind, "key_bindings.action.rewind", Optional),
    (Action::ZoomMap, "key_bindings.action.zoom_map", Optional),
    (Action::Menu, "key_bindings.action.menu", Optional),
    (
        Action::DialogueLog,
        "key_bindings.action.dialogue_log",
        RowGroup::Shared(DIALOGUE_LOG_HEADING),
    ),
];

/// The rebindable actions the PC's Key bindings screen doesn't list: only
/// the Playdate's crank binds them (ticket 1035), which no PC device has.
/// The Playdate's lists them ([`PLAYDATE_KEY_ROWS`]).
pub const CRANK_ACTIONS: [Action; 2] = [Action::RewindBack, Action::RewindForward];

/// The rebindable action the Playdate's Key bindings screen doesn't list:
/// its map has one skin, which doesn't zoom (ticket 1045).
pub const NOT_ON_THE_PLAYDATE: [Action; 1] = [Action::ZoomMap];

/// The rows of the Key bindings screen on the Playdate (no keyboard,
/// ticket 1045): [`KEY_ROWS`] with the crank's two after Rewind, less
/// [`NOT_ON_THE_PLAYDATE`].
pub const PLAYDATE_KEY_ROWS: [KeyRow; 19] = [
    KEY_ROWS[0],
    KEY_ROWS[1],
    KEY_ROWS[2],
    KEY_ROWS[3],
    KEY_ROWS[4],
    KEY_ROWS[5],
    KEY_ROWS[6],
    KEY_ROWS[7],
    KEY_ROWS[8],
    KEY_ROWS[9],
    KEY_ROWS[10],
    KEY_ROWS[11],
    KEY_ROWS[12],
    KEY_ROWS[13],
    KEY_ROWS[14],
    (
        Action::RewindBack,
        "key_bindings.action.rewind_back",
        Optional,
    ),
    (
        Action::RewindForward,
        "key_bindings.action.rewind_forward",
        Optional,
    ),
    KEY_ROWS[16],
    KEY_ROWS[17],
];

/// Text key of the heading over the actions of the dialogue log's
/// [`KeyGroup`], which may share keys with the rest.
pub const DIALOGUE_LOG_HEADING: &str = "key_bindings.dialogue_log";

/// The help line of a text box: `Enter done · Backspace delete · Escape
/// cancel`, in `ctx`'s language.
pub fn text_keys_help(ctx: &crate::screen::Ctx) -> String {
    let [done, delete, cancel] = fixed::text_key_names();
    ctx.text_with(
        "text_box.help",
        &[("done", &done), ("delete", &delete), ("cancel", &cancel)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every button name, and every name of a set of directions, can be drawn:
    /// a Sony pad's shapes are glyphs of our own in the atlas.
    #[test]
    fn every_button_name_is_in_the_font() {
        let font = trpg_content::load_embedded().map_or_else(|e| panic!("{e:?}"), |c| c.font);
        let names = PadKind::ALL
            .into_iter()
            .flat_map(|kind| Button::ALL.iter().map(move |&b| kind.button_name(b)))
            .chain(pad::DIRECTION_SETS.map(|(name, _)| name));
        for name in names {
            assert_ne!(name, "");
            for c in name.chars() {
                assert!(
                    font.glyphs.contains_key(&c),
                    "{name}: {c:?} is not in the font"
                );
            }
        }
    }
}
