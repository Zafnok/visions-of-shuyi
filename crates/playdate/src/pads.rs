//! The Playdate's buttons (ticket 1033): the SDK's button bits become one
//! controller of [`PadKind::Playdate`] fed through `trpg-ui`'s [`Pads`],
//! so they reach screens as `Action`s through the player's pad bindings
//! (ADR-0034, the `keyboard-input` skill). Like `shuyi-app`'s `pads.rs` it
//! only translates; the only file in this crate that names buttons.
//!
//! A is reported at the binding position `East` and B at `West`, as the
//! engine's profile has them, so the `playdate` table of
//! `assets/data/keymap.ron` makes the D-pad move the cursor, A confirm and
//! B cancel (`docs/design/playdate.md`, *Buttons
//! and the crank*).

use alloc::vec::Vec;

use trpg_ui::RawInputEvent;
use trpg_ui::input::{Button, PadKind, PadState, Pads, StickDef};

/// The SDK's `PDButtons` bits (`pd_api_sys.h`; `shim/shim.c` checks this
/// copy against the header), each with the binding position it reports.
const BUTTONS: [(u32, Button); 6] = [
    (0x01, Button::DpadLeft),
    (0x02, Button::DpadRight),
    (0x04, Button::DpadUp),
    (0x08, Button::DpadDown),
    (0x10, Button::West),
    (0x20, Button::East),
];

/// The Playdate's one pad: never unplugged, no sticks.
const PAD: usize = 0;

/// The pad state of the buttons whose bits are set in `bits`.
pub fn state(bits: u32) -> PadState {
    PadState {
        buttons: BUTTONS
            .iter()
            .filter(|&&(bit, _)| bits & bit != 0)
            .map(|&(_, button)| button)
            .collect(),
        ..PadState::default()
    }
}

/// The Playdate's buttons, read once per frame.
#[derive(Debug, Clone)]
pub struct PlaydatePad {
    pads: Pads,
}

impl PlaydatePad {
    /// No button held yet; `stick` is the keymap's stick thresholds (the
    /// Playdate has no stick, but [`Pads`] wants them).
    pub fn new(stick: StickDef) -> Self {
        Self {
            pads: Pads::new(stick),
        }
    }

    /// This frame's button releases, then presses: `current` is the bits
    /// held now and `pushed` the bits pressed since the last frame. A
    /// button pressed and let go between two frames counts as held for
    /// this one, so a quick tap is never lost while a frame takes long.
    pub fn poll(&mut self, current: u32, pushed: u32) -> Vec<RawInputEvent> {
        let state = state(current | pushed);
        RawInputEvent::from_pads(&mut self.pads, &[(PAD, PadKind::Playdate, state)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEFT: u32 = 1;
    const UP: u32 = 4;
    const B: u32 = 16;
    const A: u32 = 32;

    fn pad() -> PlaydatePad {
        PlaydatePad::new(StickDef {
            press_percent: 50,
            release_percent: 35,
        })
    }

    fn down(button: Button) -> RawInputEvent {
        RawInputEvent::PadDown(button, PadKind::Playdate)
    }

    /// The ticket: A is `East` (Confirm by default), B is `West` (Cancel),
    /// the D-pad is the D-pad.
    #[test]
    fn a_is_east_b_is_west_and_the_d_pad_is_the_d_pad() {
        let mut p = pad();
        assert_eq!(p.poll(A, A), [down(Button::East)]);
        assert_eq!(
            p.poll(B | UP | LEFT, B | UP | LEFT),
            [
                RawInputEvent::PadUp(Button::East),
                down(Button::West),
                down(Button::DpadUp),
                down(Button::DpadLeft),
            ]
        );
        let all = state(0b11_1111);
        assert_eq!(all.buttons.iter().count(), 6);
        assert_eq!(state(1 << 6), PadState::default(), "no other bits");
    }

    #[test]
    fn a_held_button_is_one_press_until_it_is_let_go() {
        let mut p = pad();
        assert_eq!(p.poll(A, A), [down(Button::East)]);
        assert_eq!(p.poll(A, 0), []);
        assert_eq!(p.poll(0, 0), [RawInputEvent::PadUp(Button::East)]);
        assert_eq!(p.poll(0, 0), []);
    }

    /// Pressed and let go between two frames: held for one frame.
    #[test]
    fn a_tap_between_two_frames_is_not_lost() {
        let mut p = pad();
        assert_eq!(p.poll(0, B), [down(Button::West)]);
        assert_eq!(p.poll(0, 0), [RawInputEvent::PadUp(Button::West)]);
    }
}
