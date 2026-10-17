//! The game's entries in a platform's own menu: the Playdate's system menu,
//! behind its Menu button, which holds three of a game's own entries
//! (ticket 1035, `docs/design/playdate.md`, *Buttons and the crank*):
//! Danger zone, Dialogue log and Options.
//!
//! Choosing one does what the same thing does in the game: the front end
//! hands the screen on top the entry's [`action`](SystemMenuEntry::action)
//! with [`Game::act`](shuyi_ui::Game::act), which toggles the danger zone
//! on the battle map, opens the dialogue log while a scene plays, and opens
//! Options where a menu has an Options entry (the title, the battle map,
//! Preparations). Where the screen has no such
//! thing, nothing happens. The front end only adds the entries and
//! forwards the choice.

use crate::input::Action;
use crate::screen::Ctx;

/// One of the game's entries in the platform's menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemMenuEntry {
    /// Toggles the enemy danger zone.
    DangerZone,
    /// Opens the dialogue log.
    DialogueLog,
    /// Opens the Options screen.
    Options,
}

impl SystemMenuEntry {
    /// Every entry, in the order the menu lists them.
    pub const ALL: [SystemMenuEntry; 3] = [Self::DangerZone, Self::DialogueLog, Self::Options];

    /// The entry's name in the player's language.
    pub fn label(self, ctx: &Ctx) -> &str {
        ctx.text(match self {
            Self::DangerZone => "system_menu.danger_zone",
            Self::DialogueLog => "system_menu.dialogue_log",
            Self::Options => "system_menu.options",
        })
    }

    /// The action choosing it hands the game.
    pub fn action(self) -> Action {
        match self {
            Self::DangerZone => Action::DangerZone,
            Self::DialogueLog => Action::DialogueLog,
            Self::Options => Action::Options,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::tests::ctx;

    #[test]
    fn three_entries_named_in_the_players_language() {
        let c = ctx();
        let labels = SystemMenuEntry::ALL.map(|e| e.label(&c).to_owned());
        assert_eq!(labels, ["Danger Zone", "Dialogue Log", "Options"]);
        let actions = SystemMenuEntry::ALL.map(SystemMenuEntry::action);
        assert_eq!(
            actions,
            [Action::DangerZone, Action::DialogueLog, Action::Options]
        );
    }
}
