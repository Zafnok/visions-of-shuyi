//! The Key bindings screen (tickets 0815 and 0816; `docs/design/controls.md`,
//! *Rebinding keys* and *Rebinding buttons*): Shuyi Engine's
//! (`shuyi_ui::screens::key_bindings`, ticket 1112) with this game's rows
//! ([`KEY_ROWS`]), re-exported here under its old names. Its look is the
//! engine's glyph skin ([`glyph::paint`], ADR-0054), or on the Playdate
//! [`playdate::paint`] ([`SKINS`], ADR-0083).
//!
//! With no keyboard (the Playdate, ticket 1045) it is the controller side
//! alone, and lists the Playdate's rows ([`PLAYDATE_KEY_ROWS`]): the
//! crank's two as well, and no `Zoom map`.

pub mod playdate;

pub use shuyi_ui::screens::key_bindings::*;

use crate::glyph_buffer::GlyphBuffer;
use crate::input::{Action, KEY_ROWS, PLAYDATE_KEY_ROWS};
use crate::screen::{Ctx, Keyboard, Shuyi};
use crate::screens::Skins;

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

/// The screen with this game's rows: [`KEY_ROWS`], or with no keyboard
/// [`PLAYDATE_KEY_ROWS`]; painted by the skin of the context's skin set.
pub fn screen(ctx: &Ctx) -> KeyBindingsScreen {
    KeyBindingsScreen::new(ctx, rows(ctx)).painted_by(paint)
}

/// The rows `ctx` lists.
pub fn rows(ctx: &Ctx) -> &'static [KeyRow] {
    match ctx.keyboard {
        Keyboard::Present => &KEY_ROWS,
        Keyboard::Absent => &PLAYDATE_KEY_ROWS,
    }
}

/// The screen's skins: the engine's glyph skin on the PC, and the
/// Playdate's.
pub const SKINS: Skins<KeyBindingsView> = Skins {
    glyph: glyph::paint::<Shuyi>,
    playdate: playdate::paint,
};

/// Paints `view` with the skin of `ctx`'s skin set ([`SKINS`]).
fn paint(ctx: &Ctx, view: &KeyBindingsView, buf: &mut GlyphBuffer) {
    SKINS.paint(ctx, view, buf);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{CRANK_ACTIONS, NOT_LISTED_YET, NOT_ON_THE_PLAYDATE};
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
            // engine-friction: engine#2394
            .filter(|a| !NOT_LISTED_YET.contains(a))
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
        // The world map's rows, once listed (engine#2394).
        let c = ctx();
        assert_eq!(
            label(&c, &crate::input::WORLD_MAP_ROWS, Action::WorldArmy),
            "Your army"
        );
        assert_eq!(
            label(&c, &crate::input::WORLD_MAP_ROWS, Action::WorldFind),
            "Find"
        );
    }

    /// The Playdate's rows (ticket 1045): every rebindable action once,
    /// the crank's too, but `Zoom map`; required first, the crank's after
    /// Rewind; the screen lists them with no keyboard, labelled.
    #[test]
    fn the_playdates_rows_add_the_cranks_and_leave_out_zoom() {
        let mut listed: Vec<Action> = PLAYDATE_KEY_ROWS.iter().map(|r| r.0).collect();
        let required = listed.iter().take_while(|a| a.is_required()).count();
        assert_eq!(required, 7);
        assert!(listed[required..].iter().all(|a| !a.is_required()));
        let rewind = listed.iter().position(|&a| a == Action::Rewind).unwrap();
        assert_eq!(listed[rewind + 1..rewind + 3], CRANK_ACTIONS);
        listed.sort();
        let mut rebindable: Vec<Action> = Action::ALL
            .into_iter()
            .filter(|a| a.is_rebindable() && !NOT_ON_THE_PLAYDATE.contains(a))
            .collect();
        rebindable.sort();
        assert_eq!(listed, rebindable);
        let mut c = ctx();
        assert_eq!(rows(&c), &KEY_ROWS);
        c.keyboard = Keyboard::Absent;
        assert_eq!(rows(&c), &PLAYDATE_KEY_ROWS);
        let view = screen(&c).view(&c);
        assert_eq!(view.switch, None);
        assert_eq!(view.row(Action::RewindBack).unwrap().label, "Rewind back");
        assert_eq!(
            view.row(Action::RewindForward).unwrap().label,
            "Rewind forward"
        );
        assert!(view.row(Action::ZoomMap).is_none());
    }
}
