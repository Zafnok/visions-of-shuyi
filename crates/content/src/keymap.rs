//! This game's keymap (`assets/data/keymap.ron`, ADR-0015): its [`Action`]s,
//! which [`KeyGroup`]s may share a key, and its layouts' names. The file
//! format, the keys, chords and controller buttons are Shuyi Engine's,
//! generic over a game's [`ActionSet`] (`shuyi_assets::keymap`, tickets 1106
//! and 1107), re-exported here as [`format`](mod@format); the aliases here
//! ([`KeymapDef`], [`LayoutKeys`], …) fill in this game's actions.
use crate::prelude::*;

use core::fmt;

use crate::error::ContentError;
use crate::formats;

pub use shuyi_assets::keymap as format;
pub use shuyi_assets::keymap::action_set;

pub use format::{
    ActionSet, Button, ButtonChord, Chord, ChordDef, CrankDef, Key, LAYOUT_NAME_MAX, Layout,
    RepeatDef, SLOTS, StickDef, UiAction,
};

/// Path of the keymap inside the asset bundle.
pub const KEYMAP_PATH: &str = "data/keymap.ron";

/// This game's validated keymap.
pub type KeymapDef = format::KeymapDef<Action>;

/// One of this game's chord lookups ([`format::Bindings`]).
pub type Bindings = format::Bindings<Action>;

/// One layout's default keys ([`format::LayoutKeys`]).
pub type LayoutKeys = format::LayoutKeys<Action>;

/// The default controller buttons ([`format::PadKeys`]).
pub type PadKeys = format::PadKeys<Action>;

/// The right-handed layout (`docs/design/controls.md`): arrow keys steer;
/// the left hand acts from the `A S D F` row.
pub const RIGHT_HANDED: Layout = Layout::named("RightHanded");

/// The left-handed layout: `W A S D` steer; the right hand acts from the
/// `J K L ;` row.
pub const LEFT_HANDED: Layout = Layout::named("LeftHanded");

/// Loads and validates the embedded keymap.
pub fn load_keymap() -> Result<KeymapDef, Vec<ContentError>> {
    formats::load(KEYMAP_PATH)
}

/// The keys every keymap has on top of its own bindings, whatever the
/// player binds (`docs/design/controls.md`, *Rebinding keys*): plain
/// `Escape` cancels.
const FIXED_KEYS: [(Chord, Action); 1] = [(Chord::plain(Key::Escape), Action::Cancel)];

/// Everything a player can ask for (ADR-0006). Screens only ever see these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    /// Move the cursor one tile left.
    CursorLeft,
    /// Move the cursor one tile down.
    CursorDown,
    /// Move the cursor one tile up.
    CursorUp,
    /// Move the cursor one tile right.
    CursorRight,
    /// Confirm / select.
    Confirm,
    /// Cancel / back. Screens with nothing to cancel open the menu instead.
    Cancel,
    /// Unit info / details.
    Info,
    /// Toggle the enemy danger zone.
    DangerZone,
    /// Jump to the next ready unit.
    NextUnit,
    /// Jump to the previous ready unit.
    PrevUnit,
    /// End the player phase (asks for confirmation).
    EndTurn,
    /// Open the map menu.
    Menu,
    /// Debug overlay (developer).
    Debug,
    /// Toggle auto-end of the player phase (`docs/design/turn-structure.md`).
    ToggleAutoEnd,
    /// Open the turn rewind (`docs/design/death-and-difficulty.md`).
    Rewind,
    /// Pick a unit, tile or target on the map with the cursor. Optional:
    /// with no key, Confirm does it (`docs/design/controls.md`, *Optional
    /// split keys*).
    Select,
    /// Accept the end-turn prompt. Optional: with no key, End turn pressed
    /// again does it (`docs/design/controls.md`, *Optional split keys*).
    ConfirmEndTurn,
    /// Switch the battle map between its two zooms
    /// (`docs/design/look-and-feel.md`, *Zoom*). Optional.
    ZoomMap,
    /// Open the dialogue log while a scene plays
    /// (`docs/design/look-and-feel.md`, *Dialogue log*). Optional, and in
    /// a [`KeyGroup`] of its own (`docs/design/controls.md`, *Dialogue
    /// log*).
    DialogueLog,
    /// One step further back in the turn rewind, opening it first
    /// (`docs/design/playdate.md`, *Buttons and the crank*): the Playdate's
    /// crank wound back one notch. Optional; bound only in the `playdate`
    /// table, so the Key bindings screen doesn't list it (ticket 1035).
    RewindBack,
    /// One step towards the present in the open turn rewind: the crank
    /// wound forwards one notch. As [`RewindBack`](Self::RewindBack).
    RewindForward,
    /// Open the Options screen where a menu has an Options entry (the
    /// title menu, the map menu, Preparations). It has no key: only a
    /// platform's own menu sends it (the Playdate's system menu, ticket
    /// 1035), so it can't be rebound.
    Options,
    /// Move the world map's cursor to the army (ticket 1007,
    /// `docs/design/controls.md`, *World map*). Optional, in the
    /// [`KeyGroup::WorldMap`].
    WorldArmy,
    /// Open the world map's Find list: jump to a kind of place and cycle
    /// through them (ticket 1007). Optional, in the [`KeyGroup::WorldMap`].
    WorldFind,
}

/// Which actions may share a key or button (`docs/design/controls.md`,
/// *Dialogue log*): only actions of the same group clash, since actions of
/// different groups are used in different places. A key is on at most one
/// action of each group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyGroup {
    /// The map, the menus and everything else.
    General,
    /// The dialogue log, opened while a scene plays.
    DialogueLog,
    /// The world map's own shortcuts (ticket 1007): the world map is
    /// never on screen with a battle, so they may share the battle's keys
    /// and buttons (Nick: "different contexts can reuse buttons").
    WorldMap,
}

impl KeyGroup {
    /// Every group, in the order the Key bindings screen lists them.
    pub const ALL: [KeyGroup; 3] = [Self::General, Self::DialogueLog, Self::WorldMap];
}

impl Action {
    /// Every action, in declaration order.
    pub const ALL: [Action; 24] = [
        Self::CursorLeft,
        Self::CursorDown,
        Self::CursorUp,
        Self::CursorRight,
        Self::Confirm,
        Self::Cancel,
        Self::Info,
        Self::DangerZone,
        Self::NextUnit,
        Self::PrevUnit,
        Self::EndTurn,
        Self::Menu,
        Self::Debug,
        Self::ToggleAutoEnd,
        Self::Rewind,
        Self::Select,
        Self::ConfirmEndTurn,
        Self::ZoomMap,
        Self::DialogueLog,
        Self::RewindBack,
        Self::RewindForward,
        Self::Options,
        Self::WorldArmy,
        Self::WorldFind,
    ];

    /// The name used in keymap files (the variant name).
    pub fn name(self) -> &'static str {
        const NAMES: [&str; 24] = [
            "CursorLeft",
            "CursorDown",
            "CursorUp",
            "CursorRight",
            "Confirm",
            "Cancel",
            "Info",
            "DangerZone",
            "NextUnit",
            "PrevUnit",
            "EndTurn",
            "Menu",
            "Debug",
            "ToggleAutoEnd",
            "Rewind",
            "Select",
            "ConfirmEndTurn",
            "ZoomMap",
            "DialogueLog",
            "RewindBack",
            "RewindForward",
            "Options",
            "WorldArmy",
            "WorldFind",
        ];
        NAMES[self as usize]
    }

    /// Looks an action up by its [`name`](Self::name) (exact match).
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|a| a.name() == name)
    }

    /// Whether holding the key re-emits the action (the four cursor moves).
    pub fn is_repeatable(self) -> bool {
        matches!(
            self,
            Self::CursorLeft | Self::CursorDown | Self::CursorUp | Self::CursorRight
        )
    }

    /// Whether the player must always keep at least one key on this action:
    /// the *Required* column of `docs/design/controls.md`'s *Rebinding keys*
    /// table (the four cursor moves, Confirm, Cancel and End turn). The rest
    /// may be left with no key.
    pub fn is_required(self) -> bool {
        matches!(
            self,
            Self::CursorLeft
                | Self::CursorDown
                | Self::CursorUp
                | Self::CursorRight
                | Self::Confirm
                | Self::Cancel
                | Self::EndTurn
        )
    }

    /// The [`KeyGroup`] whose actions this one's keys may not be shared
    /// with.
    pub fn group(self) -> KeyGroup {
        match self {
            Self::DialogueLog => KeyGroup::DialogueLog,
            Self::WorldArmy | Self::WorldFind => KeyGroup::WorldMap,
            _ => KeyGroup::General,
        }
    }

    /// Whether the player can rebind this action. The developer Debug key
    /// can't (`docs/design/controls.md`: it is not on the Key bindings
    /// screen), nor can Options, which has no key.
    pub fn is_rebindable(self) -> bool {
        !matches!(self, Self::Debug | Self::Options)
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl ActionSet for Action {
    type Group = KeyGroup;

    const ALL: &'static [Self] = &Action::ALL;

    fn name(self) -> &'static str {
        Action::name(self)
    }

    fn is_repeatable(self) -> bool {
        Action::is_repeatable(self)
    }

    fn is_required(self) -> bool {
        Action::is_required(self)
    }

    fn is_rebindable(self) -> bool {
        Action::is_rebindable(self)
    }

    fn group(self) -> KeyGroup {
        Action::group(self)
    }

    fn ui(self) -> Option<UiAction> {
        match self {
            Self::CursorUp => Some(UiAction::Up),
            Self::CursorDown => Some(UiAction::Down),
            Self::CursorLeft => Some(UiAction::Left),
            Self::CursorRight => Some(UiAction::Right),
            Self::Confirm => Some(UiAction::Confirm),
            Self::Cancel => Some(UiAction::Cancel),
            Self::Debug => Some(UiAction::Debug),
            Self::Info
            | Self::DangerZone
            | Self::NextUnit
            | Self::PrevUnit
            | Self::EndTurn
            | Self::Menu
            | Self::ToggleAutoEnd
            | Self::Rewind
            | Self::Select
            | Self::ConfirmEndTurn
            | Self::ZoomMap
            | Self::DialogueLog
            | Self::RewindBack
            | Self::RewindForward
            | Self::Options
            | Self::WorldArmy
            | Self::WorldFind => None,
        }
    }

    fn fixed_keys() -> &'static [(Chord, Self)] {
        &FIXED_KEYS
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;

    use super::*;

    #[test]
    fn key_and_action_tables_line_up() {
        assert_eq!(Key::ALL.len(), 26 + 10 + 1 + 4 + 5 + 12 + 10 + 6 + 10 + 5);
        for (i, k) in Key::ALL.iter().enumerate() {
            assert_eq!(*k as usize, i);
            assert_eq!(Key::from_name(k.name()), Some(*k));
        }
        for (i, a) in Action::ALL.iter().enumerate() {
            assert_eq!(*a as usize, i);
            assert_eq!(Action::from_name(&a.to_string()), Some(*a));
        }
        assert_eq!(Action::from_name("cursorleft"), None);
    }

    #[test]
    fn only_cursor_actions_repeat() {
        let repeatable: Vec<Action> = Action::ALL
            .into_iter()
            .filter(|a| a.is_repeatable())
            .collect();
        assert_eq!(repeatable, Action::ALL[..4].to_vec());
        assert!(
            Action::ALL[..4]
                .iter()
                .all(|a| a.name().starts_with("Cursor"))
        );
    }

    #[test]
    fn embedded_pad_buttons_match_the_design() {
        // docs/design/controls.md, Controller → Default buttons.
        use Button::{
            DpadDown, DpadLeft, DpadRight, DpadUp, East, LeftShoulder, LeftStickDown,
            LeftStickLeft, LeftStickRight, LeftStickUp, LeftTrigger, North, RightShoulder,
            RightTrigger, Select, South, Start, West,
        };
        let k = load_keymap().unwrap_or_default();
        let design: PadKeys = [
            // D-pad and left stick both move the cursor.
            (Action::CursorLeft, vec![DpadLeft, LeftStickLeft]),
            (Action::CursorDown, vec![DpadDown, LeftStickDown]),
            (Action::CursorUp, vec![DpadUp, LeftStickUp]),
            (Action::CursorRight, vec![DpadRight, LeftStickRight]),
            (Action::Confirm, vec![South]),
            (Action::Cancel, vec![East]),
            (Action::PrevUnit, vec![LeftShoulder]),
            (Action::NextUnit, vec![RightShoulder]),
            (Action::Info, vec![North]),
            (Action::DangerZone, vec![West]),
            (Action::EndTurn, vec![Start]),
            (Action::ToggleAutoEnd, vec![Select]),
            (Action::Rewind, vec![LeftTrigger]),
            (Action::ZoomMap, vec![RightTrigger]),
            // Another key group, so it shares Danger zone's button.
            (Action::DialogueLog, vec![West]),
            // The world map's group: Unit info's and Danger zone's buttons.
            (Action::WorldArmy, vec![North]),
            (Action::WorldFind, vec![West]),
            // No button: the map menu, the optional split keys, Debug,
            // the Playdate's crank and system menu.
            (Action::Menu, vec![]),
            (Action::Select, vec![]),
            (Action::ConfirmEndTurn, vec![]),
            (Action::Debug, vec![]),
            (Action::RewindBack, vec![]),
            (Action::RewindForward, vec![]),
            (Action::Options, vec![]),
        ]
        .map(|(a, buttons)| (a, buttons.into_iter().map(ButtonChord::from).collect()))
        .into();
        assert_eq!(design.len(), Action::ALL.len());
        assert_eq!(k.pad, design);
        // Stick thresholds: tunable, press above release.
        assert_eq!(
            k.stick,
            StickDef {
                press_percent: 50,
                release_percent: 35
            }
        );
    }

    /// Ticket 1034: the Playdate's buttons are `docs/design/playdate.md`'s
    /// (*Buttons and the crank*): the D-pad moves the cursor, A confirms,
    /// B cancels, A + B together ends the turn and hold B + left / right
    /// picks the previous / next ready unit. Ticket 1035: the crank wound
    /// back steps the rewind back, wound forwards towards the present, a
    /// notch being 30° (tunable). Nothing else has a button: the map menu
    /// and the system menu have those.
    #[test]
    fn embedded_playdate_buttons_match_the_design() {
        use Button::{CrankBack, CrankForward, DpadDown, DpadLeft, DpadRight, DpadUp, East, West};
        let k = load_keymap().unwrap_or_default();
        let one = |b| vec![ButtonChord::single(b)];
        let two = |a, b| ButtonChord::pair(a, b).into_iter().collect::<Vec<_>>();
        let mut design: PadKeys = Action::ALL.into_iter().map(|a| (a, vec![])).collect();
        design.extend([
            (Action::CursorLeft, one(DpadLeft)),
            (Action::CursorDown, one(DpadDown)),
            (Action::CursorUp, one(DpadUp)),
            (Action::CursorRight, one(DpadRight)),
            (Action::Confirm, one(East)),
            (Action::Cancel, one(West)),
            (Action::EndTurn, two(East, West)),
            (Action::PrevUnit, two(West, DpadLeft)),
            (Action::NextUnit, two(West, DpadRight)),
            (Action::RewindBack, one(CrankBack)),
            (Action::RewindForward, one(CrankForward)),
        ]);
        assert_eq!(k.playdate, design);
        assert_eq!(k.crank, CrankDef { notch_degrees: 30 });
        // The PC's table is its own.
        assert_ne!(k.pad, k.playdate);
        assert_eq!(k.clone().for_playdate().pad, design);
        // The chord window: tunable, starting at 80 ms.
        assert_eq!(k.chord, ChordDef { window_ms: 80 });
    }

    #[test]
    fn the_game_lists_its_two_layouts_right_handed_first() {
        let k = load_keymap().unwrap_or_else(|e| panic!("{e:?}"));
        assert_eq!(
            k.layout_names,
            [
                (RIGHT_HANDED, "layout_picker.right_handed".to_owned()),
                (LEFT_HANDED, "layout_picker.left_handed".to_owned()),
            ]
        );
    }

    #[test]
    fn the_games_actions_map_onto_the_ui_actions_once_each() {
        let ui = |a: Action| ActionSet::ui(a);
        assert_eq!(ui(Action::CursorUp), Some(UiAction::Up));
        assert_eq!(ui(Action::CursorDown), Some(UiAction::Down));
        assert_eq!(ui(Action::CursorLeft), Some(UiAction::Left));
        assert_eq!(ui(Action::CursorRight), Some(UiAction::Right));
        assert_eq!(ui(Action::Confirm), Some(UiAction::Confirm));
        assert_eq!(ui(Action::Cancel), Some(UiAction::Cancel));
        assert_eq!(ui(Action::Debug), Some(UiAction::Debug));
        let mapped = Action::ALL.iter().filter(|&&a| ui(a).is_some()).count();
        assert_eq!(mapped, 7);
        assert_eq!(Action::for_ui(UiAction::Cancel), Some(Action::Cancel));
        assert_eq!(
            <Action as ActionSet>::fixed_keys(),
            [(Chord::plain(Key::Escape), Action::Cancel)]
        );
        assert_eq!(<Action as ActionSet>::ALL, Action::ALL);
        for a in Action::ALL {
            assert_eq!(ActionSet::name(a), a.name());
            assert_eq!(<Action as ActionSet>::from_name(a.name()), Some(a));
            assert_eq!(ActionSet::is_required(a), a.is_required());
            assert_eq!(ActionSet::is_repeatable(a), a.is_repeatable());
            assert_eq!(ActionSet::is_rebindable(a), a.is_rebindable());
            assert_eq!(ActionSet::group(a), a.group());
        }
    }

    /// Looks `chord` up in the embedded keymap's `layout`, among the
    /// [`KeyGroup::General`] actions.
    fn embedded(layout: Layout) -> impl Fn(&str) -> Option<Action> {
        embedded_in(layout, KeyGroup::General)
    }

    /// Looks `chord` up in the embedded keymap's `layout`, among `group`'s
    /// actions.
    fn embedded_in(layout: Layout, group: KeyGroup) -> impl Fn(&str) -> Option<Action> {
        let bindings = load_keymap()
            .ok()
            .and_then(|k| k.bindings(layout, group))
            .unwrap_or_default();
        move |s| Chord::parse(s).ok().and_then(|c| bindings.get(&c).copied())
    }

    #[test]
    fn embedded_right_handed_layout_matches_the_design() {
        // docs/design/controls.md, right-handed column.
        let get = embedded(RIGHT_HANDED);
        assert_eq!(get("Left"), Some(Action::CursorLeft));
        assert_eq!(get("Down"), Some(Action::CursorDown));
        assert_eq!(get("Up"), Some(Action::CursorUp));
        assert_eq!(get("Right"), Some(Action::CursorRight));
        assert_eq!(get("f"), Some(Action::Confirm));
        assert_eq!(get("d"), Some(Action::Cancel));
        // Escape is a fixed key, not in the file (see trpg-ui's Keymap).
        assert_eq!(get("Escape"), None);
        assert_eq!(get("a"), Some(Action::PrevUnit));
        assert_eq!(get("s"), Some(Action::NextUnit));
        assert_eq!(get("e"), Some(Action::Info));
        assert_eq!(get("w"), Some(Action::DangerZone));
        assert_eq!(get("Space"), Some(Action::EndTurn));
        assert_eq!(get("Shift+Space"), Some(Action::ToggleAutoEnd));
        assert_eq!(get("F2"), Some(Action::Debug));
        assert_eq!(get("r"), Some(Action::Rewind));
        assert_eq!(get("q"), Some(Action::ZoomMap));
        let log = embedded_in(RIGHT_HANDED, KeyGroup::DialogueLog);
        assert_eq!(log("b"), Some(Action::DialogueLog));
        let world = embedded_in(RIGHT_HANDED, KeyGroup::WorldMap);
        assert_eq!(world("c"), Some(Action::WorldArmy));
        assert_eq!(world("x"), Some(Action::WorldFind));
    }

    #[test]
    fn embedded_left_handed_layout_matches_the_design() {
        // docs/design/controls.md, left-handed column.
        let get = embedded(LEFT_HANDED);
        assert_eq!(get("a"), Some(Action::CursorLeft));
        assert_eq!(get("s"), Some(Action::CursorDown));
        assert_eq!(get("w"), Some(Action::CursorUp));
        assert_eq!(get("d"), Some(Action::CursorRight));
        assert_eq!(get("j"), Some(Action::Confirm));
        assert_eq!(get("k"), Some(Action::Cancel));
        assert_eq!(get("Escape"), None);
        assert_eq!(get(";"), Some(Action::PrevUnit));
        assert_eq!(get("l"), Some(Action::NextUnit));
        assert_eq!(get("i"), Some(Action::Info));
        assert_eq!(get("o"), Some(Action::DangerZone));
        assert_eq!(get("Space"), Some(Action::EndTurn));
        assert_eq!(get("Shift+Space"), Some(Action::ToggleAutoEnd));
        assert_eq!(get("F2"), Some(Action::Debug));
        assert_eq!(get("u"), Some(Action::Rewind));
        assert_eq!(get("p"), Some(Action::ZoomMap));
        let log = embedded_in(LEFT_HANDED, KeyGroup::DialogueLog);
        assert_eq!(log("n"), Some(Action::DialogueLog));
        // Finger mirrors of c and x.
        let world = embedded_in(LEFT_HANDED, KeyGroup::WorldMap);
        assert_eq!(world(","), Some(Action::WorldArmy));
        assert_eq!(world("."), Some(Action::WorldFind));
    }

    #[test]
    fn embedded_keymap_has_exactly_the_design_bindings() {
        // 15 chords per layout: nothing bound beyond the design table.
        let k = load_keymap().unwrap_or_default();
        for layout in [RIGHT_HANDED, LEFT_HANDED] {
            let general = k.bindings(layout, KeyGroup::General);
            assert_eq!(general.map(|b| b.len()), Some(15), "{layout}");
            let log = k.bindings(layout, KeyGroup::DialogueLog);
            assert_eq!(log.map(|b| b.len()), Some(1), "{layout}");
            let world = k.bindings(layout, KeyGroup::WorldMap);
            assert_eq!(world.map(|b| b.len()), Some(2), "{layout}");
            assert_eq!(k.layouts.get(&layout).map(BTreeMap::len), Some(24));
            // The crank's and the system menu's actions: no key (1035).
            for none in [Action::RewindBack, Action::RewindForward, Action::Options] {
                assert_eq!(k.chords(layout, none), [], "{layout} {none}");
            }
            assert!(!Action::DialogueLog.is_required());
            assert!(!Action::ZoomMap.is_required());
            // The optional split keys start with no key (controls.md).
            for split in [Action::Select, Action::ConfirmEndTurn] {
                assert_eq!(k.chords(layout, split), [], "{layout} {split}");
                assert!(!split.is_required());
            }
        }
        assert_eq!(k.layouts.len(), 2);
        // Before a layout is chosen: either hand's up/down and confirm keys.
        let picker: Bindings = [
            ("Up", Action::CursorUp),
            ("w", Action::CursorUp),
            ("Down", Action::CursorDown),
            ("s", Action::CursorDown),
            ("f", Action::Confirm),
            ("j", Action::Confirm),
            ("Enter", Action::Confirm),
            ("Space", Action::Confirm),
        ]
        .into_iter()
        .filter_map(|(c, a)| Chord::parse(c).ok().map(|c| (c, a)))
        .collect();
        assert_eq!(picker.len(), 8);
        assert_eq!(k.layout_picker, picker);
        assert_eq!(
            k.repeat,
            RepeatDef {
                delay_ms: 300,
                interval_ms: 55
            }
        );
    }

    #[test]
    fn required_and_rebindable_actions_follow_the_design() {
        // docs/design/controls.md, Rebinding keys.
        use Action::{Cancel, Confirm, CursorDown, CursorLeft, CursorRight, CursorUp, EndTurn};
        let required: Vec<Action> = Action::ALL
            .into_iter()
            .filter(|a| a.is_required())
            .collect();
        assert_eq!(
            required,
            [
                CursorLeft,
                CursorDown,
                CursorUp,
                CursorRight,
                Confirm,
                Cancel,
                EndTurn
            ]
        );
        let fixed: Vec<Action> = Action::ALL
            .into_iter()
            .filter(|a| !a.is_rebindable())
            .collect();
        // Options has no key at all (the Playdate's system menu, 1035).
        assert_eq!(fixed, [Action::Debug, Action::Options]);
    }
}
