//! The Key bindings screen (tickets 0815 and 0816; `docs/design/controls.md`,
//! *Rebinding keys* and *Rebinding buttons*): Shuyi Engine's
//! (`shuyi_ui::screens::key_bindings`, ticket 1112) with this game's rows
//! ([`KEY_ROWS`]), re-exported here under its old names. Its look is the
//! engine's glyph skin ([`glyph::paint`], ADR-0054).

pub use shuyi_ui::screens::key_bindings::*;

use crate::input::{Action, KEY_ROWS};
use crate::screen::{Ctx, Shuyi};

/// The Key bindings screen for this game
/// ([`shuyi_ui::screens::key_bindings::KeyBindingsScreen`]).
pub type KeyBindingsScreen = shuyi_ui::screens::key_bindings::KeyBindingsScreen<Shuyi>;

/// One row of the screen, as this game gives it: the action, the text key
/// of its label and what it sits under.
pub type KeyRow = shuyi_ui::screens::key_bindings::KeyRow<Action>;

/// What the screen shows ([`view::KeyBindingsView`]).
pub type KeyBindingsView = view::KeyBindingsView<Action>;

/// A heading and the actions under it ([`view::GroupView`]).
pub type GroupView = view::GroupView<Action>;

/// One action's row ([`view::RowView`]).
pub type RowView = view::RowView<Action>;

/// The screen with this game's rows ([`KEY_ROWS`]).
pub fn screen(ctx: &Ctx) -> KeyBindingsScreen {
    KeyBindingsScreen::new(ctx, &KEY_ROWS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::CRANK_ACTIONS;
    use crate::screen::tests::ctx;

    /// This game's rows: every rebindable action once but the crank's,
    /// which only the Playdate binds (ticket 1035), required first.
    #[test]
    fn rows_list_every_rebindable_action_once_required_first() {
        let mut listed: Vec<Action> = KEY_ROWS.iter().map(|r| r.0).collect();
        let required = listed.iter().take_while(|a| a.is_required()).count();
        assert_eq!(required, 7);
        assert!(listed[required..].iter().all(|a| !a.is_required()));
        listed.sort();
        let mut rebindable: Vec<Action> = Action::ALL
            .into_iter()
            .filter(|a| a.is_rebindable() && !CRANK_ACTIONS.contains(a))
            .collect();
        rebindable.sort();
        assert_eq!(listed, rebindable);
        let c = ctx();
        assert_eq!(label(&c, &KEY_ROWS, Action::Info), "Unit info");
        let view = screen(&c).view(&c);
        let headings: Vec<_> = view.groups.iter().map(|g| g.heading.as_str()).collect();
        assert_eq!(
            headings,
            ["Must have a key", "Optional", "In the dialogue log"]
        );
        assert_eq!(view.groups[2].rows[0].label, "Open the log");
    }
}
