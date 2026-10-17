//! The optional split keys (`docs/design/controls.md`, *Optional split
//! keys*): which action picks on the map and which accept the end-turn
//! prompt, given the player's keymap and the device they use. The battle's
//! own rules, so they live here and not with the generic input (ticket
//! 1106).

use crate::input::{Action, Device, Keymap};
use crate::widgets::help::HelpKeys;

/// The action that picks on the map with the cursor (a unit, its tile, a
/// target): [`Action::Select`] once it has a key, else [`Action::Confirm`].
/// Per device: a key for the keyboard, a button for a pad.
pub fn select_action(keymap: &Keymap, device: Device) -> Action {
    if keymap.has_input(Action::Select, device) {
        Action::Select
    } else {
        Action::Confirm
    }
}

/// The actions that accept the end-turn prompt, the one help text names
/// first: [`Action::ConfirmEndTurn`] and Confirm once it has a key, else
/// End turn (pressed again) and Confirm. Per device, like
/// [`select_action`].
pub fn end_turn_accept_actions(keymap: &Keymap, device: Device) -> [Action; 2] {
    if keymap.has_input(Action::ConfirmEndTurn, device) {
        [Action::ConfirmEndTurn, Action::Confirm]
    } else {
        [Action::EndTurn, Action::Confirm]
    }
}

/// [`select_action`] for the keys help text names.
pub fn help_select_action(keys: HelpKeys<'_>) -> Action {
    select_action(&keys, keys.device())
}

/// [`end_turn_accept_actions`] for the keys help text names.
pub fn help_end_turn_accept_actions(keys: HelpKeys<'_>) -> [Action; 2] {
    end_turn_accept_actions(&keys, keys.device())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{Button, Chord, Key, PadKind, RIGHT_HANDED, load_keymap};

    const PAD: Device = Device::Pad(PadKind::Xbox);

    fn defaults() -> Keymap {
        let def = load_keymap().unwrap_or_else(|e| panic!("{e:?}"));
        Keymap::for_layout(&def, RIGHT_HANDED)
    }

    #[test]
    fn select_falls_back_to_confirm_until_it_has_a_key() {
        let km = defaults();
        assert_eq!(select_action(&km, Device::Keyboard), Action::Confirm);
        assert_eq!(select_action(&km, PAD), Action::Confirm);
        let bound = Keymap::new([(Chord::plain(Key::G), Action::Select)], km.repeat());
        assert_eq!(select_action(&bound, Device::Keyboard), Action::Select);
        // A key alone doesn't split it on a controller, nor a button alone on
        // the keyboard.
        assert_eq!(select_action(&bound, PAD), Action::Confirm);
        let button = Keymap::new([], km.repeat()).with_pad([(Button::North, Action::Select)]);
        assert_eq!(select_action(&button, PAD), Action::Select);
        assert_eq!(select_action(&button, Device::Keyboard), Action::Confirm);
        let keys = HelpKeys::new(&button, PAD);
        assert_eq!(help_select_action(keys), Action::Select);
        assert_eq!(help_select_action(keys.on_keyboard()), Action::Confirm);
    }

    #[test]
    fn end_turn_is_accepted_by_end_turn_until_confirm_end_turn_has_a_key() {
        let km = defaults();
        let default = [Action::EndTurn, Action::Confirm];
        assert_eq!(end_turn_accept_actions(&km, Device::Keyboard), default);
        assert_eq!(end_turn_accept_actions(&km, PAD), default);
        let split = [Action::ConfirmEndTurn, Action::Confirm];
        let bound = Keymap::new(
            [(Chord::plain(Key::Enter), Action::ConfirmEndTurn)],
            km.repeat(),
        );
        assert_eq!(end_turn_accept_actions(&bound, Device::Keyboard), split);
        assert_eq!(end_turn_accept_actions(&bound, PAD), default);
        let button =
            Keymap::new([], km.repeat()).with_pad([(Button::North, Action::ConfirmEndTurn)]);
        assert_eq!(end_turn_accept_actions(&button, PAD), split);
        assert_eq!(end_turn_accept_actions(&button, Device::Keyboard), default);
        let keys = HelpKeys::new(&button, PAD);
        assert_eq!(help_end_turn_accept_actions(keys), split);
        assert_eq!(help_end_turn_accept_actions(keys.on_keyboard()), default);
    }
}
