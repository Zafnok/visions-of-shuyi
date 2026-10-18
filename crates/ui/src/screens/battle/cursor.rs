//! The browsing cursor (ADR-0018, `docs/design/look-and-feel.md`): the
//! tactics kit's virtual cursor (`shuyi_ui::grid::cursor`, ticket 1115),
//! moved one tile per step by this game's cursor actions ([`dir`]), never
//! off the map; no jump keys (`docs/design/controls.md`). It pulses
//! between full and about half brightness, never off. A map skin draws it
//! ([`crate::map_view`], ADR-0024).

use trpg_core::Dir;

use crate::input::Action;

pub use crate::map_view::CursorStyle;
pub use shuyi_ui::grid::cursor::{BLINK_MIN, BLINK_PERIOD, Cursor};

/// The way a `Cursor…` action moves the cursor; `None` for any other
/// action.
pub const fn dir(action: Action) -> Option<Dir> {
    match action {
        Action::CursorLeft => Some(Dir::Left),
        Action::CursorRight => Some(Dir::Right),
        Action::CursorUp => Some(Dir::Up),
        Action::CursorDown => Some(Dir::Down),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use trpg_core::Pos;

    use super::*;

    #[test]
    fn the_cursor_actions_step_one_tile_and_no_other_action_moves_it() {
        let mut c = Cursor::new(Pos::new(1, 1));
        let mut step = |action| dir(action).is_some_and(|d| c.step(d, 3, 3));
        assert!(step(Action::CursorLeft));
        assert!(!step(Action::CursorLeft));
        assert!(step(Action::CursorUp));
        assert!(step(Action::CursorRight));
        assert!(step(Action::CursorDown));
        assert!(!step(Action::Confirm));
        assert!(!step(Action::NextUnit));
        assert_eq!(c.pos, Pos::new(1, 1));
        assert_eq!(dir(Action::CursorDown), Some(Dir::Down));
        assert_eq!(dir(Action::Cancel), None);
    }
}
