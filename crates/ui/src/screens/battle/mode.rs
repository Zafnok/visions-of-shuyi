//! What the player is doing on the battle screen (ticket 0403), the Fire
//! Emblem move loop: browse ([`Mode::Idle`]), select a unit and steer its
//! path ([`Mode::Selected`]), watch it walk ([`Mode::Moving`]), pick an
//! action ([`Mode::ActionMenu`]); to attack (0404), pick a target with the
//! forecast ([`Mode::Targeting`]), where left and right swap the weapon, and
//! watch the combat ([`Mode::Combat`]); to attack with a Combat Art or a
//! combat active (0442), pick it in the arts list ([`Mode::List`])
//! first; to cast (0410), pick a spell (the same, [`ListKind::Spells`]) and
//! what to cast it on ([`Mode::CastTarget`]). The move
//! is only sent, as one
//! [`Command::Act`], when an action is chosen, so cancelling is free: until
//! then the unit's new tile is a drawing override ([`Mode::drawn_pos`]) and
//! the [`BattleState`] is untouched.
//!
//! Pointing the cursor at an enemy the selected unit can attack after moving
//! (0428), with a weapon or an attack spell (0430), aims the path at the
//! tile it will attack from; Confirm walks there and goes straight to the
//! forecast on that enemy with what the unit has equipped, where left and
//! right swap between the weapons and spells that reach it. Pointing at a
//! hurt ally with a unit that knows a heal spell (0451) aims the path the
//! same way at a tile a heal reaches it from; Confirm walks there and opens
//! the list of the unit's heals, then the heal's preview on that ally.
//!
//! Around it (0405): the map menu ([`Mode::MapMenu`], [`Mode::UnitList`],
//! [`Mode::Objective`]), the end-turn prompt ([`Mode::EndTurnPrompt`]), the
//! restart and suspend prompts ([`Mode::RestartPrompt`], 0801;
//! [`Mode::SuspendPrompt`], 0802) and
//! the unit info screen ([`Mode::Info`]). Talking (0705): `Talk` in the

//! action menu picks who to talk to ([`Mode::TalkTarget`]); it's free, so
//! the menu opens again after the scene.
//!
//! Transitions are pure ([`step`], [`Mode::tick`]); the screen owns the
//! cursor and applies the [`Effect`]s.

use crate::prelude::*;
use alloc::borrow::Cow;

use trpg_core::{
    BattleState, Command, Equipped, Faction, Phase, Pos, Reach, TileSet, UnitAction, UnitId,
    attack_tiles, path_cost, reachable, threat_area,
};

use super::ai_phase::AiAction;
use super::art_list::{ArtLine, art_lines, arts_menu, can_use_art};
use super::attack::{Targeting, WeaponChoice, attack_tile, weapon_choices};
use super::items::{
    EquipChoice, ItemTargeting, PackGroup, can_equip, can_use_item, equip_choices, equip_command,
    equip_menu, pack_groups, pack_menu,
};
use super::magic::{
    CastTargeting, SpellChoice, can_cast, heal_choices, heal_tile, knows_spells, reaches_an_enemy,
    spell_choices, spell_menu, target_pos,
};
use super::map_menu::{MapEntry, map_menu_with, ready_players, unit_list};
use super::path::steer;
use super::playback::Playback;
use super::skills::{
    SkillChoice, SkillTargeting, can_use_skill, has_skill_menu, skill_choices, skill_menu,
    use_command,
};
use super::split_keys;
use super::walk::{self, Gait};
use crate::input::{Action, Device, Keymap};
use crate::widgets::menu::{Menu, MenuEvent, MenuItem};
use crate::widgets::wrap_step;
use crate::words::Words;

pub use super::walk::WALK_TILES_PER_S;

/// How long Confirm must be held during a walk to skip to its end, in
/// seconds (so the tap that started the walk doesn't skip it). *Tunable.*
pub const HOLD_SKIP_S: f32 = 0.2;

/// A selected unit, its ranges and the path the player is steering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// The unit.
    pub unit: UnitId,
    /// Where it can move.
    pub reach: Reach,
    /// Tiles it can move through (drawn in `move_range`).
    pub moves: TileSet,
    /// Tiles it could attack after moving, other than `moves` (drawn in
    /// `attack_range`).
    pub attack: TileSet,
    /// The path arrow: the unit's tile first, never empty.
    pub path: Vec<Pos>,
    /// The unit under the cursor the path is aimed at, if any: an enemy to
    /// attack (0428) or a hurt ally to heal (0451).
    pub target: Option<UnitId>,
}

impl Selection {
    /// Selects unit `id`: its reach and ranges in `state`, and a path of
    /// just its tile. `None` if it can't be found or placed.
    pub fn new(state: &BattleState, id: UnitId) -> Option<Self> {
        let unit = state.unit(id)?;
        let reach = reachable(
            state.map(),
            state.terrain(),
            state.classes(),
            state.units(),
            id,
        )
        .ok()?;
        let moves = reach.passable();
        let mut attack = TileSet::new(moves.width(), moves.height());
        for (min, max) in unit.attack_ranges(state.classes(), state.items(), state.spells()) {
            for pos in attack_tiles(&reach, min, max).iter() {
                if !moves.contains(pos) {
                    attack.insert(pos);
                }
            }
        }
        let path = vec![reach.origin()];
        Some(Self {
            unit: id,
            reach,
            moves,
            attack,
            path,
            target: None,
        })
    }

    /// The unit's tile.
    pub fn origin(&self) -> Pos {
        self.reach.origin()
    }

    /// Where the path ends.
    pub fn dest(&self) -> Pos {
        self.path.last().copied().unwrap_or_else(|| self.origin())
    }

    /// Steers the path to `to` ([`steer`]).
    pub fn steer(&mut self, to: Pos, state: &BattleState) {
        let valid = |path: &[Pos]| {
            path_cost(
                state.map(),
                state.terrain(),
                state.classes(),
                state.units(),
                self.unit,
                path,
            )
            .is_ok()
        };
        self.path = steer(&self.path, to, &self.reach, valid);
    }
}

/// One entry of the action menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEntry {
    /// Attack: enabled when a weapon can attack someone from there.
    Attack,
    /// Attack with a Combat Art or a combat active (0442): shown when the
    /// unit has one for a weapon it carries, enabled when one can be used
    /// on someone from there.
    Arts,
    /// Cast a spell (0410): shown when the unit knows one, enabled when
    /// one has something to be cast on from there.
    Magic,
    /// Seize the objective tile, when legal there.
    Seize,
    /// Talk to an adjacent unit, when a talk trigger allows it (0705).
    Talk,
    /// Use a non-combat active skill (0412): shown when the unit knows one.
    Skill,
    /// Use a consumable from the battle pack (0407).
    Item,
    /// Change the equipped weapon (0407).
    Equip,
    /// End the unit's action.
    Wait,
}

impl MenuEntry {
    /// The text shown.
    pub fn label(self, words: Words<'_>) -> &str {
        words.text(match self {
            MenuEntry::Attack => "battle.menu.attack",
            MenuEntry::Arts => "battle.menu.arts",
            MenuEntry::Magic => "battle.menu.magic",
            MenuEntry::Seize => "battle.menu.seize",
            MenuEntry::Talk => "battle.menu.talk",
            MenuEntry::Skill => "battle.menu.skill",
            MenuEntry::Item => "battle.menu.item",
            MenuEntry::Equip => "battle.menu.equip",
            MenuEntry::Wait => "battle.menu.wait",
        })
    }

    /// The menu item, disabled if not `enabled`.
    fn item(self, words: Words<'_>, enabled: bool) -> MenuItem {
        if enabled {
            MenuItem::new(self.label(words))
        } else {
            MenuItem::disabled(self.label(words))
        }
    }

    /// Whether the entry can be chosen for `sel`'s unit: `Attack` if a
    /// weapon reaches someone, a list if a line of it can be chosen
    /// ([`ListKind::can_choose`]).
    fn enabled(self, sel: &Selection, weapons: &[WeaponChoice], state: &BattleState) -> bool {
        match self {
            MenuEntry::Attack => !weapons.is_empty(),
            MenuEntry::Seize | MenuEntry::Talk | MenuEntry::Wait => true,
            list => ListKind::of(list, sel, state).is_some_and(|kind| kind.can_choose()),
        }
    }
}

/// What a list the action menu opens lists (ticket 1161), with its lines.
#[derive(Debug, Clone, PartialEq)]
pub enum ListKind {
    /// The unit's Combat Arts and combat actives (0442).
    Arts(Vec<ArtLine>),
    /// The unit's spells (0410), or its heals for the ally it is aimed at
    /// (0451).
    Spells(Vec<SpellChoice>),
    /// The unit's non-combat active skills (0412).
    Skills(Vec<SkillChoice>),
    /// The battle pack's items, grouped (0407).
    Pack(Vec<PackGroup>),
    /// The unit's weapons and attack spells, to equip one (0407).
    Equip(Vec<EquipChoice>),
}

impl ListKind {
    /// The list `entry` opens for `sel`'s unit at its path's end; `None`
    /// for an entry that opens no list.
    fn of(entry: MenuEntry, sel: &Selection, state: &BattleState) -> Option<Self> {
        let (unit, dest) = (sel.unit, sel.dest());
        Some(match entry {
            MenuEntry::Arts => ListKind::Arts(art_lines(state, unit, dest)),
            MenuEntry::Magic => ListKind::Spells(spell_choices(state, unit, dest)),
            MenuEntry::Skill => ListKind::Skills(skill_choices(state, IN_PLAY, sel)),
            MenuEntry::Item => ListKind::Pack(pack_groups(state, unit, dest)),
            MenuEntry::Equip => ListKind::Equip(equip_choices(state, unit)),
            // Attack, Seize, Talk and Wait.
            _ => return None,
        })
    }

    /// The action menu's entry that opens it (Cancel goes back to it).
    pub fn entry(&self) -> MenuEntry {
        match self {
            ListKind::Arts(_) => MenuEntry::Arts,
            ListKind::Spells(_) => MenuEntry::Magic,
            ListKind::Skills(_) => MenuEntry::Skill,
            ListKind::Pack(_) => MenuEntry::Item,
            ListKind::Equip(_) => MenuEntry::Equip,
        }
    }

    /// Whether a line can be chosen: an art or active usable on someone, a
    /// spell with a target, a usable skill, an item usable on someone, two
    /// usable weapons or attack spells.
    fn can_choose(&self) -> bool {
        match self {
            ListKind::Arts(lines) => can_use_art(lines),
            ListKind::Spells(choices) => can_cast(choices),
            ListKind::Skills(choices) => can_use_skill(choices),
            ListKind::Pack(groups) => can_use_item(groups),
            ListKind::Equip(choices) => can_equip(choices),
        }
    }

    /// The list's menu for `unit`, written in `words`.
    fn menu(&self, state: &BattleState, words: Words<'_>, unit: UnitId) -> Menu {
        match self {
            ListKind::Arts(lines) => arts_menu(state, words, lines),
            ListKind::Spells(choices) => spell_menu(state, words, unit, choices),
            ListKind::Skills(choices) => skill_menu(state, words, unit, choices),
            ListKind::Pack(groups) => pack_menu(state, words, groups),
            ListKind::Equip(choices) => equip_menu(state, words, unit, choices),
        }
    }

    /// The list open for `sel`.
    fn open(self, sel: Selection, state: &BattleState) -> Mode {
        let (menu, kind) = (self.menu(state, IN_PLAY, sel.unit), self);
        Mode::List { sel, menu, kind }
    }
}

/// The action menu for `sel`'s unit at its path's end: `Attack` (enabled if
/// some weapon can attack someone there), `Arts` if it has a Combat Art or a
/// combat active for a weapon it carries, `Magic` if the unit knows a spell
/// (enabled if one has a target there), `Seize` if legal there, `Talk` if
/// it has someone next to it to talk to, `Skill` if the unit knows a
/// non-combat active, `Item` and `Equip` (enabled when
/// they can do something), `Wait`.
pub fn menu_entries(sel: &Selection, state: &BattleState) -> Vec<MenuEntry> {
    let mut entries = vec![MenuEntry::Attack];
    if !art_lines(state, sel.unit, sel.dest()).is_empty() {
        entries.push(MenuEntry::Arts);
    }
    if knows_spells(state, sel.unit) {
        entries.push(MenuEntry::Magic);
    }
    if state.can_seize(sel.unit, sel.dest()) {
        entries.push(MenuEntry::Seize);
    }
    if !state.talk_targets(sel.unit, sel.dest()).is_empty() {
        entries.push(MenuEntry::Talk);
    }
    if has_skill_menu(state, sel.unit) {
        entries.push(MenuEntry::Skill);
    }
    entries.extend([MenuEntry::Item, MenuEntry::Equip, MenuEntry::Wait]);
    entries
}

/// An other-faction unit's threat area shown in the browse mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Threat {
    /// The unit.
    pub unit: UnitId,
    /// Every tile it could attack this turn (drawn in `attack_range`).
    pub area: TileSet,
}

/// What the player is doing.
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    /// Browsing the map, maybe with a unit's threat area shown.
    Idle {
        /// The threat area shown, if any.
        threat: Option<Threat>,
    },
    /// A unit is selected; the cursor steers its path.
    Selected(Selection),
    /// The unit walks its path.
    Moving {
        /// The selection being carried out.
        sel: Selection,
        /// Seconds since the walk started.
        t: f32,
        /// Seconds Confirm has been held without a break.
        held: f32,
        /// Walking speed, in tiles per second: the map skin's
        /// ([`WALK_TILES_PER_S`] until the screen sets it).
        pace: f32,
    },
    /// The unit stands at its path's end; the player picks an action.
    ActionMenu {
        /// The selection (restored by Cancel).
        sel: Selection,
        /// The menu widget.
        menu: Menu,
        /// What each menu item does.
        entries: Vec<MenuEntry>,
        /// The weapons that can attack from there (`Attack` is enabled if
        /// there are any).
        weapons: Vec<WeaponChoice>,
    },
    /// A list the action menu opened: the unit's arts (0442), spells
    /// (0410), skills (0412), pack (0407) or weapons to equip (0407).
    List {
        /// The selection.
        sel: Selection,
        /// The list.
        menu: Menu,
        /// What it lists, and each line.
        kind: ListKind,
    },
    /// The player picks a target and reads the forecast.
    Targeting(Box<Targeting>),
    /// The player picks what a spell is cast on: an enemy (with the
    /// forecast), a hurt ally or a tile.
    CastTarget(Box<CastTargeting>),
    /// The player picks who a skill (Shove) is used on.
    SkillTarget(Box<SkillTargeting>),
    /// The player picks who an item is used on.
    ItemTarget(Box<ItemTargeting>),
    /// The player picks who to talk to (0705).
    TalkTarget(TalkTargeting),
    /// An attack's combat plays out.
    Combat(Box<Playback>),
    /// An AI unit's action is shown (0502): the camera pans to it, it
    /// walks, then its combat plays.
    AiAction(Box<AiAction>),
    /// A unit waits to move after its attack ([`BattleState::pending_move`]):
    /// Confirm on one of `tiles` moves it there, Confirm on the unit stays.
    MoveAfter {
        /// The unit.
        unit: UnitId,
        /// Where it may go ([`BattleState::move_after_tiles`]).
        tiles: Vec<Pos>,
    },
    /// The map menu is open.
    MapMenu {
        /// The menu widget.
        menu: Menu,
        /// What each item is.
        entries: Vec<MapEntry>,
    },
    /// The map menu's list of player units.
    UnitList {
        /// The list.
        menu: Menu,
        /// Which unit each item is.
        units: Vec<UnitId>,
    },
    /// The objective and turn are shown.
    Objective,
    /// `End turn with N units ready?`: `EndTurn` again or Confirm ends the
    /// phase (or Confirm end turn instead of `EndTurn`, once it has a key;
    /// see [`Mode::route`]), Cancel backs out.
    EndTurnPrompt {
        /// Units still ready.
        ready: usize,
    },
    /// `Restart the battle from turn 1?`: Confirm restarts
    /// ([`Effect::Restart`]), Cancel goes back to the map menu.
    RestartPrompt,
    /// `Suspend the battle and return to the title?`: Confirm suspends
    /// ([`Effect::Suspend`]), Cancel goes back to the map menu.
    SuspendPrompt,
    /// The info screen for a unit.
    Info {
        /// The unit shown.
        unit: UnitId,
    },
}

/// The language the mode's own lists are built in. Playing only needs a
/// list's focus and which of its lines can be picked, so it never depends
/// on the player's language; the screen paints the lists in that
/// ([`Mode::told`]).
pub(super) const IN_PLAY: Words<'static> = Words::ENGLISH;

impl Mode {
    /// The mode as the screen paints it: its lists of arts,
    /// spells, skills, items and units written in `words`, each focused
    /// where it is. In English that is the mode itself.
    pub fn told(&self, state: &BattleState, words: Words<'_>) -> Cow<'_, Mode> {
        if words.is_english() {
            return Cow::Borrowed(self);
        }
        let mut mode = self.clone();
        match &mut mode {
            Mode::ActionMenu { menu, entries, .. } => {
                let lines = entries.iter().zip(menu.items());
                let items = lines.map(|(e, item)| e.item(words, item.enabled));
                *menu = Menu::new(items.collect()).focused(menu.focus());
            }
            Mode::MapMenu { menu, entries } => {
                let lines = entries.iter().zip(menu.items());
                let items = lines.map(|(e, item)| e.item(words, item.enabled));
                *menu = Menu::new(items.collect()).focused(menu.focus());
            }
            Mode::List { sel, menu, kind } => {
                if let ListKind::Skills(choices) = kind {
                    *choices = skill_choices(state, words, sel);
                }
                *menu = kind.menu(state, words, sel.unit).focused(menu.focus());
            }
            Mode::UnitList { menu, .. } => {
                *menu = unit_list(state, words).0.focused(menu.focus());
            }
            _ => {}
        }
        Cow::Owned(mode)
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Idle { threat: None }
    }
}

/// What the screen must do after a [`step`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Nothing.
    None,
    /// Apply this command to the battle, then continue from
    /// [`Mode::after_command`].
    Apply(Command),
    /// Apply this command, which doesn't end the unit's action (`Equip`,
    /// `Talk`), then reopen the action menu for the unit, still at the end
    /// of the path.
    ApplyStay(Command, Box<Selection>),
    /// Put the cursor on this tile.
    Cursor(Pos),
    /// Start the battle again (the map menu's `Restart Battle`, confirmed).
    Restart,
    /// Save the battle and go back to the title (the map menu's `Suspend`,
    /// confirmed).
    Suspend,
    /// Open the Options screen (the map menu's `Options`); the map menu
    /// stays open under it.
    Options,
    /// Open the Guides list (the map menu's `Guides`, ticket 0458); the
    /// map menu stays open under it.
    Guides,
    /// Do what this action's key does, as if it were pressed now: a
    /// shortcut chosen in the map menu (ticket 1035), which has closed.
    Press(Action),
}

impl Mode {
    /// The mode after the battle changed: [`Mode::MoveAfter`] if a unit
    /// waits to move after its attack, else browsing.
    pub fn after_command(state: &BattleState) -> Mode {
        match state.pending_move() {
            Some(pending) => Mode::MoveAfter {
                unit: pending.unit,
                tiles: state.move_after_tiles(),
            },
            None => Mode::default(),
        }
    }

    /// Whether the cursor keys move the cursor (in the action menu they
    /// move the menu's focus; during a walk they do nothing).
    pub fn cursor_free(&self) -> bool {
        matches!(
            self,
            Mode::Idle { .. } | Mode::Selected(_) | Mode::MoveAfter { .. }
        )
    }

    /// Whether the map is what the player is looking at: browsing, a
    /// unit selected or walking, picking a target, the AI's moves. Not a
    /// menu, a prompt, the unit's page or a fight's playback. The Zoom map
    /// key works only here.
    pub fn shows_map(&self) -> bool {
        matches!(
            self,
            Mode::Idle { .. }
                | Mode::Selected(_)
                | Mode::Moving { .. }
                | Mode::MoveAfter { .. }
                | Mode::Targeting(_)
                | Mode::CastTarget(_)
                | Mode::SkillTarget(_)
                | Mode::ItemTarget(_)
                | Mode::TalkTarget(_)
                | Mode::AiAction(_)
        )
    }

    /// Whether this mode picks something on the map with the cursor (a
    /// unit, its tile, a heal/item/talk target), which the Select key does
    /// once it has one; menus, prompts and the attack forecast (a spell's
    /// too) stay on Confirm (`docs/design/controls.md`, *Optional split
    /// keys*).
    pub fn picks_on_map(&self) -> bool {
        let cast_pick = matches!(self, Mode::CastTarget(t) if t.forecast().is_none());
        cast_pick
            || self.cursor_free()
            || matches!(
                self,
                Mode::SkillTarget(_) | Mode::ItemTarget(_) | Mode::TalkTarget(_)
            )
    }

    /// The action [`step`] sees for `action` in this mode with the player's
    /// `keymap`, or `None` to ignore it. [`step`] reads a pick on the map
    /// and accepting the end-turn prompt as Confirm; this makes the
    /// optional split keys do them instead: on the map
    /// [`split_keys::select_action`] picks (and Confirm does nothing once Select
    /// has a key), and in the prompt [`split_keys::end_turn_accept_actions`]
    /// accept (and End turn does nothing once Confirm end turn has a key).
    pub fn route(&self, action: Action, keymap: &Keymap, device: Device) -> Option<Action> {
        if self.picks_on_map() {
            if action == split_keys::select_action(keymap, device) {
                return Some(Action::Confirm);
            }
            if action == Action::Confirm {
                return None;
            }
        }
        if matches!(self, Mode::EndTurnPrompt { .. }) {
            if split_keys::end_turn_accept_actions(keymap, device).contains(&action) {
                return Some(Action::Confirm);
            }
            if action == Action::EndTurn {
                return None;
            }
        }
        Some(action)
    }

    /// The cursor moved to `to`: a selected unit's path follows it, or is
    /// aimed at the enemy it can attack (0428) or the hurt ally it can heal
    /// (0451) there.
    pub fn cursor_moved(&mut self, to: Pos, state: &BattleState) {
        if let Mode::Selected(sel) = self {
            let unit = state.units().iter().find(|u| u.pos == to).map(|u| u.id);
            let tile = |id| attack_tile(state, sel, id).or_else(|| heal_tile(state, sel, id));
            let aim = unit.and_then(|id| tile(id).map(|t| (id, t)));
            if let Some((id, tile)) = aim {
                if tile != sel.dest() {
                    sel.path = sel.reach.path_to(tile).unwrap_or_else(|| sel.path.clone());
                }
                sel.target = Some(id);
            } else {
                sel.target = None;
                sel.steer(to, state);
            }
        }
    }

    /// The selected unit, while one is selected, walking or choosing.
    pub fn selection(&self) -> Option<&Selection> {
        match self {
            Mode::Selected(sel)
            | Mode::Moving { sel, .. }
            | Mode::ActionMenu { sel, .. }
            | Mode::List { sel, .. } => Some(sel),
            Mode::TalkTarget(t) => Some(&t.sel),
            Mode::Targeting(t) => Some(&t.sel),
            Mode::CastTarget(t) => Some(&t.sel),
            Mode::SkillTarget(t) => Some(&t.sel),
            Mode::ItemTarget(t) => Some(&t.sel),
            Mode::Idle { .. }
            | Mode::MoveAfter { .. }
            | Mode::Combat(_)
            | Mode::AiAction(_)
            | Mode::MapMenu { .. }
            | Mode::UnitList { .. }
            | Mode::Objective
            | Mode::EndTurnPrompt { .. }
            | Mode::RestartPrompt
            | Mode::SuspendPrompt
            | Mode::Info { .. } => None,
        }
    }

    /// Where unit `id` is drawn, if not on its own tile: along its path
    /// while it walks, at the path's end while it chooses an action, an
    /// art or a target.
    pub fn drawn_pos(&self, id: UnitId) -> Option<Pos> {
        match self {
            Mode::Moving { sel, t, pace, .. } if sel.unit == id => Some(walk_pos(sel, t * pace)),
            Mode::Selected(_) | Mode::Moving { .. } => None,
            _ => self
                .selection()
                .filter(|s| s.unit == id)
                .map(Selection::dest),
        }
    }

    /// A walk at `pace` tiles per second (the map skin's); any other mode
    /// as it is.
    #[must_use]
    pub fn at_pace(self, pace: f32) -> Mode {
        match self {
            Mode::Moving { sel, t, held, .. } => Mode::Moving { sel, t, held, pace },
            other => other,
        }
    }

    /// During a walk (the player's or an AI unit's), the walking unit and
    /// how it looks between two tiles of its path ([`walk::gait`]); `None`
    /// in other modes, and once it has arrived.
    pub fn gait(&self) -> Option<(UnitId, Gait)> {
        match self {
            Mode::Moving { sel, t, pace, .. } => {
                Some((sel.unit, walk::gait(&sel.path, t * pace, *pace)?))
            }
            Mode::AiAction(a) => Some((a.unit(), a.gait()?)),
            _ => None,
        }
    }

    /// During a walk (the player's or an AI unit's), the walking unit and
    /// how many tiles it enters in the next [`tick`](Self::tick) with the
    /// same arguments (none if the hold skips the rest of the player's
    /// walk); `None` in other modes.
    pub fn tiles_entered(&self, dt: f32, confirm_held: bool) -> Option<(UnitId, usize)> {
        if let Mode::AiAction(a) = self {
            return Some((a.unit(), a.tiles_entered(dt, confirm_held)));
        }
        let Mode::Moving { sel, t, held, pace } = self else {
            return None;
        };
        let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
        let (to, held) = (t + dt, if confirm_held { held + dt } else { 0.0 });
        let skipped = held >= HOLD_SKIP_S && !walk_done(sel, to * pace);
        let entered = if skipped {
            0
        } else {
            walk_steps(sel, to * pace) - walk_steps(sel, t * pace)
        };
        Some((sel.unit, entered))
    }

    /// Advances a walk by `dt` seconds (`confirm_held`: Confirm is down this
    /// frame). The walk ends, opening the action menu, once the unit
    /// reaches the path's end or Confirm has been held [`HOLD_SKIP_S`].
    /// A combat's playback and an AI action play on (the AI action then
    /// goes on to its combat). Other modes are unchanged.
    #[must_use]
    pub fn tick(self, dt: f32, confirm_held: bool, state: &BattleState) -> Mode {
        if let Mode::Combat(mut playback) = self {
            playback.tick(dt, confirm_held);
            return if playback.done() {
                Mode::after_command(state)
            } else {
                Mode::Combat(playback)
            };
        }
        if let Mode::AiAction(mut action) = self {
            action.tick(dt, confirm_held);
            return if action.done() {
                action.into_then()
            } else {
                Mode::AiAction(action)
            };
        }
        let Mode::Moving { sel, t, held, pace } = self else {
            return self;
        };
        let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
        let (t, held) = (t + dt, if confirm_held { held + dt } else { 0.0 });
        if held >= HOLD_SKIP_S || walk_done(&sel, t * pace) {
            walk_ended(sel, state).0
        } else {
            Mode::Moving { sel, t, held, pace }
        }
    }
}

/// How many of `sel`'s path steps a walk has taken once it has gone
/// `tiles` tiles (at most all of them; none for a negative or NaN
/// distance).
fn walk_steps(sel: &Selection, tiles: f32) -> usize {
    walk::steps(sel.path.len(), tiles)
}

/// Whether the walk along `sel`'s path is over once it has gone `tiles`
/// tiles.
fn walk_done(sel: &Selection, tiles: f32) -> bool {
    walk_steps(sel, tiles) + 1 >= sel.path.len()
}

/// The tile a walk along `sel`'s path is on once it has gone `tiles`
/// tiles.
fn walk_pos(sel: &Selection, tiles: f32) -> Pos {
    sel.path
        .get(walk_steps(sel, tiles))
        .copied()
        .unwrap_or_else(|| sel.origin())
}

/// The target of `sel`'s aimed path, if the cursor is still on it.
fn aimed_at(sel: &Selection, cursor: Pos, state: &BattleState) -> Option<UnitId> {
    let t = sel.target?;
    (state.unit(t)?.pos == cursor).then_some(t)
}

/// A walk ended: the attack on the aimed enemy (0428) or the heal of the
/// aimed ally (0451), else the action menu.
fn walk_ended(sel: Selection, state: &BattleState) -> (Mode, Effect) {
    if sel.target.is_some() {
        open_aimed(sel, state)
    } else {
        (open_menu(sel, state), Effect::None)
    }
}

/// The unit stands at the path's end, aimed at a unit. An enemy (0428,
/// 0430): the forecast on it with the equipped weapon or spell if that
/// reaches it, else the first that does; left and right swap between those
/// that do. A hurt ally (0451): the list of the unit's heals. The action
/// menu if nothing reaches it.
fn open_aimed(sel: Selection, state: &BattleState) -> (Mode, Effect) {
    if let Some(t) = Targeting::aimed(state, sel.clone()) {
        return show_forecast(t, state);
    }
    match open_heals(&sel, state) {
        Some(mode) => (mode, Effect::None),
        None => (open_menu(sel, state), Effect::None),
    }
}

/// The heal list of `sel`'s unit for the ally it is aimed at (Nick, 0451):
/// its heal spells, those that can't reach that ally dimmed. Choosing one
/// opens its preview on that ally; Cancel goes to the action menu, on
/// `Magic`. `None` if no heal reaches it.
fn open_heals(sel: &Selection, state: &BattleState) -> Option<Mode> {
    let heals = ListKind::Spells(heal_choices(state, sel.unit, sel.dest(), sel.target?));
    heals.can_choose().then(|| heals.open(sel.clone(), state))
}

/// The action menu for `sel`.
pub(super) fn open_menu(sel: Selection, state: &BattleState) -> Mode {
    let entries = menu_entries(&sel, state);
    let weapons = weapon_choices(state, &sel);
    let items: Vec<MenuItem> = entries
        .iter()
        .map(|e| e.item(IN_PLAY, e.enabled(&sel, &weapons, state)))
        .collect();
    // It opens on the first of Attack, Arts, Magic and Seize that is enabled
    // (Magic only when a spell reaches an enemy: a forest to burn or an ally
    // to heal doesn't make it the default), else Wait: Item and Equip are
    // never the default.
    let spells = spell_choices(state, sel.unit, sel.dest());
    let first = entries
        .iter()
        .position(|e| match e {
            MenuEntry::Attack | MenuEntry::Arts | MenuEntry::Seize => {
                e.enabled(&sel, &weapons, state)
            }
            MenuEntry::Magic => reaches_an_enemy(state, &spells),
            _ => false,
        })
        .or_else(|| entries.iter().position(|&e| e == MenuEntry::Wait));
    let menu = Menu::new(items).focused(first.unwrap_or(0));
    Mode::ActionMenu {
        sel,
        menu,
        entries,
        weapons,
    }
}

/// The action menu for `sel`, focused on `entry` (back from what it opened).
pub(super) fn back_to_entry(sel: Selection, state: &BattleState, entry: MenuEntry) -> Mode {
    let mut mode = open_menu(sel, state);
    if let Mode::ActionMenu { menu, entries, .. } = &mut mode
        && let Some(at) = entries.iter().position(|&e| e == entry)
    {
        *menu = menu.clone().focused(at);
    }
    mode
}

/// The forecast `t`, with the cursor on its target.
fn show_forecast(t: Targeting, state: &BattleState) -> (Mode, Effect) {
    let at = state.unit(t.target()).map_or(t.sel.dest(), |u| u.pos);
    (Mode::Targeting(Box::new(t)), Effect::Cursor(at))
}

/// Whether `unit` belongs to the player and can still act this phase.
fn is_ready_player(state: &BattleState, id: UnitId) -> bool {
    state.unit(id).is_some_and(|u| {
        u.faction == Faction::Player && Phase::of(u.faction) == state.phase() && !u.acted
    })
}

/// The threat area of unit `id` ([`threat_area`] with its attack ranges).
fn threat_of(state: &BattleState, id: UnitId) -> Option<Threat> {
    let unit = state.unit(id)?;
    let ranges = unit.attack_ranges(state.classes(), state.items(), state.spells());
    let area = threat_area(
        state.map(),
        state.terrain(),
        state.classes(),
        state.units(),
        id,
        &ranges,
    )
    .ok()?;
    Some(Threat { unit: id, area })
}

/// One non-cursor action in `mode`, with the cursor on `cursor`. Cursor
/// movement and unit cycling are the screen's; see [`Mode::cursor_moved`].
/// The map menu lists none of the [shortcuts](MapEntry::SHORTCUTS) (as on
/// the PC); [`step_with`] gives it some.
pub fn step(mode: Mode, action: Action, cursor: Pos, state: &BattleState) -> (Mode, Effect) {
    step_with(mode, action, cursor, state, &[])
}

/// [`step`], with the map menu listing `shortcuts` (ticket 1035: those
/// with no button on a platform without a keyboard, `map_menu::shortcuts`).
pub fn step_with(
    mode: Mode,
    action: Action,
    cursor: Pos,
    state: &BattleState,
    shortcuts: &[MapEntry],
) -> (Mode, Effect) {
    let menu = MenuAt {
        state,
        cursor,
        shortcuts,
    };
    match mode {
        Mode::Idle { threat } => step_idle(threat, action, menu),
        Mode::Selected(sel) => match action {
            Action::Confirm if aimed_at(&sel, cursor, state).is_some() => {
                if sel.path.len() == 1 {
                    open_aimed(sel, state)
                } else {
                    let walk = Mode::Moving {
                        sel,
                        t: 0.0,
                        held: 0.0,
                        pace: WALK_TILES_PER_S,
                    };
                    (walk, Effect::None)
                }
            }
            Action::Confirm if cursor == sel.dest() && sel.reach.is_stoppable(cursor) => {
                if sel.path.len() == 1 {
                    (open_menu(sel, state), Effect::None)
                } else {
                    let walk = Mode::Moving {
                        sel,
                        t: 0.0,
                        held: 0.0,
                        pace: WALK_TILES_PER_S,
                    };
                    (walk, Effect::None)
                }
            }
            Action::Cancel => {
                let origin = sel.origin();
                (Mode::default(), Effect::Cursor(origin))
            }
            _ => (Mode::Selected(sel), Effect::None),
        },
        Mode::Moving { sel, .. } if action == Action::Confirm => walk_ended(sel, state),
        moving @ Mode::Moving { .. } => (moving, Effect::None),
        Mode::ActionMenu {
            sel,
            menu,
            entries,
            weapons,
        } => step_action_menu(sel, menu, entries, weapons, action, state),
        Mode::List { sel, menu, kind } => step_list(sel, menu, kind, action, state),
        // When several weapons or spells can make the attack, left and
        // right swap between them instead of changing the target (0430,
        // 0442).
        Mode::Targeting(mut t) => match action {
            Action::CursorLeft | Action::CursorRight if t.can_swap() => {
                t.swap(action == Action::CursorRight, state);
                aim(*t, state)
            }
            _ => step_target(*t, action, state),
        },
        Mode::CastTarget(t) => step_target(*t, action, state),
        Mode::SkillTarget(t) => step_target(*t, action, state),
        Mode::ItemTarget(t) => step_target(*t, action, state),
        Mode::TalkTarget(t) => step_target(t, action, state),
        Mode::Combat(mut playback) => {
            if action == Action::Cancel {
                playback.skip();
            }
            (Mode::Combat(playback), Effect::None)
        }
        action @ Mode::AiAction(_) => (action, Effect::None),
        Mode::MoveAfter { unit, tiles } => step_move_after(unit, tiles, action, cursor, state),
        other => step_around(other, action, menu),
    }
}

/// Where the map menu opens: in `state`, with the cursor on `cursor` (for
/// `Unit Info`) and the `shortcuts` the platform lists.
#[derive(Clone, Copy)]
struct MenuAt<'a> {
    state: &'a BattleState,
    cursor: Pos,
    shortcuts: &'a [MapEntry],
}

impl MenuAt<'_> {
    /// The map menu, focused on `focus`.
    fn open(self, focus: MapEntry) -> Mode {
        let (menu, entries) =
            map_menu_with(self.state, IN_PLAY, focus, self.shortcuts, self.cursor);
        Mode::MapMenu { menu, entries }
    }
}

/// [`step`] while a unit waits to move after its attack: Confirm on one of
/// `tiles` moves it there, Confirm on the unit stays.
fn step_move_after(
    unit: UnitId,
    tiles: Vec<Pos>,
    action: Action,
    cursor: Pos,
    state: &BattleState,
) -> (Mode, Effect) {
    let here = state.unit(unit).map(|u| u.pos);
    let to = if Some(cursor) == here {
        Some(None)
    } else if tiles.contains(&cursor) {
        Some(Some(cursor))
    } else {
        None
    };
    match to {
        Some(to) if action == Action::Confirm => (
            Mode::default(),
            Effect::Apply(Command::MoveAfter { unit, to }),
        ),
        _ => (Mode::MoveAfter { unit, tiles }, Effect::None),
    }
}

/// [`step`] in the modes around the move loop: the map menu and what it
/// opens, the end-turn prompt, the info screen.
fn step_around(mode: Mode, action: Action, at: MenuAt<'_>) -> (Mode, Effect) {
    let state = at.state;
    match mode {
        Mode::MapMenu { menu, entries } => step_map_menu(menu, entries, action, state),
        Mode::UnitList { mut menu, units } => match menu.handle(action) {
            Some(MenuEvent::Chosen(i)) => {
                let at = units.get(i).and_then(|&id| state.unit(id)).map(|u| u.pos);
                (Mode::default(), at.map_or(Effect::None, Effect::Cursor))
            }
            Some(MenuEvent::Cancelled) => (at.open(MapEntry::Units), Effect::None),
            None => (Mode::UnitList { menu, units }, Effect::None),
        },
        Mode::Objective => match action {
            Action::Confirm | Action::Cancel => (at.open(MapEntry::Objective), Effect::None),
            _ => (Mode::Objective, Effect::None),
        },
        Mode::EndTurnPrompt { ready } => match action {
            Action::EndTurn | Action::Confirm => {
                (Mode::default(), Effect::Apply(Command::EndPhase))
            }
            Action::Cancel => (Mode::default(), Effect::None),
            _ => (Mode::EndTurnPrompt { ready }, Effect::None),
        },
        Mode::RestartPrompt => match action {
            Action::Confirm => (Mode::default(), Effect::Restart),
            Action::Cancel => (at.open(MapEntry::Restart), Effect::None),
            _ => (Mode::RestartPrompt, Effect::None),
        },
        Mode::SuspendPrompt => match action {
            Action::Confirm => (Mode::default(), Effect::Suspend),
            Action::Cancel => (at.open(MapEntry::Suspend), Effect::None),
            _ => (Mode::SuspendPrompt, Effect::None),
        },
        Mode::Info { unit } => step_info(unit, action, state),
        other => (other, Effect::None),
    }
}

/// Ending the turn: the prompt if player units are still ready, else
/// `EndPhase` at once.
fn end_turn(state: &BattleState) -> (Mode, Effect) {
    match ready_players(state) {
        0 => (Mode::default(), Effect::Apply(Command::EndPhase)),
        ready => (Mode::EndTurnPrompt { ready }, Effect::None),
    }
}

/// [`step`] in the action menu: an action chosen sends it or opens what
/// it needs (a target, a list); Cancel goes back to steering the path.
fn step_action_menu(
    sel: Selection,
    mut menu: Menu,
    entries: Vec<MenuEntry>,
    weapons: Vec<WeaponChoice>,
    action: Action,
    state: &BattleState,
) -> (Mode, Effect) {
    match menu.handle(action) {
        Some(MenuEvent::Chosen(i)) if i < entries.len() => {
            let action = match entries[i] {
                MenuEntry::Wait => UnitAction::Wait,
                MenuEntry::Seize => UnitAction::Seize,
                MenuEntry::Talk => return choose_talk(sel, state),
                MenuEntry::Attack => return choose_attack(sel, &weapons, state),
                list => return open_list(list, sel, state),
            };
            let cmd = Command::Act {
                unit: sel.unit,
                dest: sel.dest(),
                action,
            };
            apply(cmd)
        }
        Some(MenuEvent::Cancelled) => {
            let mut sel = sel;
            sel.target = None;
            (Mode::Selected(sel), Effect::None)
        }
        _ => {
            let menu = Mode::ActionMenu {
                sel,
                menu,
                entries,
                weapons,
            };
            (menu, Effect::None)
        }
    }
}

/// [`step`] in the map menu.
fn step_map_menu(
    mut menu: Menu,
    entries: Vec<MapEntry>,
    action: Action,
    state: &BattleState,
) -> (Mode, Effect) {
    match menu.handle(action) {
        Some(MenuEvent::Chosen(i)) => match entries.get(i) {
            Some(MapEntry::Units) => {
                let (menu, units) = unit_list(state, IN_PLAY);
                (Mode::UnitList { menu, units }, Effect::None)
            }
            Some(MapEntry::Objective) => (Mode::Objective, Effect::None),
            Some(MapEntry::Restart) => (Mode::RestartPrompt, Effect::None),
            Some(MapEntry::Suspend) => (Mode::SuspendPrompt, Effect::None),
            Some(MapEntry::EndTurn) => end_turn(state),
            Some(MapEntry::Options) => (Mode::MapMenu { menu, entries }, Effect::Options),
            Some(MapEntry::Guides) => (Mode::MapMenu { menu, entries }, Effect::Guides),
            // A shortcut (ticket 1035): the menu closes and its key acts.
            Some(&shortcut @ (MapEntry::DangerZone | MapEntry::AutoEnd | MapEntry::UnitInfo)) => (
                Mode::default(),
                shortcut.action().map_or(Effect::None, Effect::Press),
            ),
            None => (Mode::MapMenu { menu, entries }, Effect::None),
        },
        Some(MenuEvent::Cancelled) => (Mode::default(), Effect::None),
        None => (Mode::MapMenu { menu, entries }, Effect::None),
    }
}

/// The units the info screen cycles through from `id`: its faction's, in
/// reading order `(y, x)`.
fn same_faction(state: &BattleState, id: UnitId) -> Vec<UnitId> {
    let Some(faction) = state.unit(id).map(|u| u.faction) else {
        return vec![];
    };
    let mut units: Vec<_> = state
        .units()
        .iter()
        .filter(|u| u.faction == faction)
        .collect();
    units.sort_by_key(|u| (u.pos.y, u.pos.x));
    units.iter().map(|u| u.id).collect()
}

/// [`step`] on the info screen: Down or `NextUnit` shows the next unit of
/// the same faction, Up or `PrevUnit` the previous one (wrapping); Cancel
/// or Info closes it with the cursor on the unit last shown.
fn step_info(unit: UnitId, action: Action, state: &BattleState) -> (Mode, Effect) {
    let forward = match action {
        Action::CursorDown | Action::NextUnit => true,
        Action::CursorUp | Action::PrevUnit => false,
        Action::Cancel | Action::Info => {
            let at = state.unit(unit).map(|u| u.pos);
            return (Mode::default(), at.map_or(Effect::None, Effect::Cursor));
        }
        _ => return (Mode::Info { unit }, Effect::None),
    };
    let units = same_faction(state, unit);
    let next = units
        .iter()
        .position(|&u| u == unit)
        .map_or(unit, |i| units[wrap_step(i, units.len(), forward)]);
    (Mode::Info { unit: next }, Effect::None)
}

/// `Attack` chosen in the action menu: the forecast with the equipped weapon
/// if it reaches someone, else the first that does (0442; the action menu
/// again if none does, which the enabled entry rules out).
fn choose_attack(sel: Selection, weapons: &[WeaponChoice], state: &BattleState) -> (Mode, Effect) {
    match Targeting::attack(state, sel.clone(), weapons) {
        Some(t) => show_forecast(t, state),
        None => (back_to_entry(sel, state, MenuEntry::Attack), Effect::None),
    }
}

/// A list chosen in the action menu (`Arts`, `Magic`, `Skill`, `Item`,
/// `Equip`): the list, or the action menu again, on `entry`, if no line of
/// it can be chosen (which the enabled entry rules out).
fn open_list(entry: MenuEntry, sel: Selection, state: &BattleState) -> (Mode, Effect) {
    let mode = match ListKind::of(entry, &sel, state) {
        Some(kind) if kind.can_choose() => kind.open(sel, state),
        _ => back_to_entry(sel, state, entry),
    };
    (mode, Effect::None)
}

/// [`step`] in a list: Confirm does what the line does ([`choose_line`]),
/// Cancel goes back to the action menu, on the entry that opened it.
fn step_list(
    sel: Selection,
    mut menu: Menu,
    kind: ListKind,
    action: Action,
    state: &BattleState,
) -> (Mode, Effect) {
    match menu.handle(action) {
        Some(MenuEvent::Chosen(i)) => choose_line(sel, menu, kind, i, state),
        Some(MenuEvent::Cancelled) => (back_to_entry(sel, state, kind.entry()), Effect::None),
        None => (Mode::List { sel, menu, kind }, Effect::None),
    }
}

/// Line `i` of a list chosen: an art opens its forecast; a spell picks
/// what it is cast on, an item who it is used on, the cursor on the first;
/// a skill that needs no target is used at once, Shove picks who it is used
/// on; a weapon or spell is equipped and the action menu reopens (the unit
/// hasn't acted). The list stays as it is if the line can't be chosen.
fn choose_line(
    sel: Selection,
    menu: Menu,
    kind: ListKind,
    i: usize,
    state: &BattleState,
) -> (Mode, Effect) {
    let (s, m) = (sel.clone(), menu.clone());
    let chosen = match &kind {
        ListKind::Arts(lines) => {
            Targeting::art(state, s, m, lines.clone(), i).map(|t| show_forecast(t, state))
        }
        ListKind::Spells(choices) => {
            CastTargeting::new(state, s, m, choices.clone(), i).map(|t| aim(t, state))
        }
        ListKind::Skills(choices) => match choices.get(i).filter(|c| c.usable) {
            Some(c) if !c.needs_target => Some(apply(use_command(&sel, &c.skill, None))),
            Some(_) => SkillTargeting::new(s, m, choices.clone(), i).map(|t| aim(t, state)),
            None => None,
        },
        ListKind::Pack(groups) => {
            ItemTargeting::new(s, m, groups.clone(), i).map(|t| aim(t, state))
        }
        // (The menu never chooses a dimmed, unusable weapon.)
        ListKind::Equip(choices) => choices.get(i).map(|c| {
            let cmd = equip_command(sel.unit, c.what.clone());
            (Mode::default(), Effect::ApplyStay(cmd, Box::new(s)))
        }),
    };
    chosen.unwrap_or((Mode::List { sel, menu, kind }, Effect::None))
}

/// Sending `cmd`, which ends the unit's action.
fn apply(cmd: Command) -> (Mode, Effect) {
    (Mode::default(), Effect::Apply(cmd))
}

/// `Talk` chosen: pick who to talk to, the cursor on the first (the action
/// menu again if nobody is there, which the menu rules out).
fn choose_talk(sel: Selection, state: &BattleState) -> (Mode, Effect) {
    let targets = state.talk_targets(sel.unit, sel.dest());
    let Some(at) = targets
        .first()
        .and_then(|&id| state.unit(id))
        .map(|u| u.pos)
    else {
        return (back_to_entry(sel, state, MenuEntry::Talk), Effect::None);
    };
    let mode = Mode::TalkTarget(TalkTargeting {
        sel,
        targets,
        index: 0,
    });
    (mode, Effect::Cursor(at))
}

/// Where the cursor goes to show `target`, the user of an item at `sel`'s
/// path end or another unit.
fn target_tile(state: &BattleState, sel: &Selection, target: UnitId) -> Option<Pos> {
    if target == sel.unit {
        Some(sel.dest())
    } else {
        state.unit(target).map(|u| u.pos)
    }
}

/// Picking on the map what a chosen action is used on (ticket 1161): an
/// attack's target, with the forecast; a spell's, a skill's or an item's;
/// who to talk to. [`step_target`] steps them all.
trait Picker: Sized {
    /// Moves to the next target (or the previous one), wrapping.
    fn cycle_target(&mut self, forward: bool, state: &BattleState);
    /// The tile the cursor shows the target on.
    fn tile(&self, state: &BattleState) -> Option<Pos>;
    /// What Confirm does: the command it sends, and the mode after it.
    fn confirm(self, state: &BattleState) -> (Mode, Effect);
    /// What Cancel goes back to: the list or menu it was opened from.
    fn back(self, state: &BattleState) -> Mode;
    /// The mode that picks with it.
    fn into_mode(self) -> Mode;
    /// What any other key does: nothing.
    fn other_key(self, _: &BattleState) -> (Mode, Effect) {
        (self.into_mode(), Effect::None)
    }
}

/// [`step`] while picking a target: the cursor keys and
/// `NextUnit`/`PrevUnit` cycle the targets (right, down and next go
/// forward; left, up and previous go back), the cursor on the one picked.
/// Confirm does what the picker does ([`Picker::confirm`]); Cancel goes
/// back to what it was opened from with the cursor on the unit.
fn step_target<P: Picker>(mut p: P, action: Action, state: &BattleState) -> (Mode, Effect) {
    let forward = match action {
        Action::CursorRight | Action::CursorDown | Action::NextUnit => true,
        Action::CursorLeft | Action::CursorUp | Action::PrevUnit => false,
        Action::Confirm => return p.confirm(state),
        Action::Cancel => {
            let back = p.back(state);
            let at = back.selection().map(Selection::dest);
            return (back, at.map_or(Effect::None, Effect::Cursor));
        }
        _ => return p.other_key(state),
    };
    p.cycle_target(forward, state);
    aim(p, state)
}

/// Picking with `p`, the cursor on its target.
fn aim<P: Picker>(p: P, state: &BattleState) -> (Mode, Effect) {
    let at = p.tile(state);
    (p.into_mode(), at.map_or(Effect::None, Effect::Cursor))
}

impl Picker for Targeting {
    fn cycle_target(&mut self, forward: bool, state: &BattleState) {
        self.cycle(forward, state);
    }

    fn tile(&self, state: &BattleState) -> Option<Pos> {
        state.unit(self.target()).map(|u| u.pos)
    }

    fn confirm(self, _: &BattleState) -> (Mode, Effect) {
        apply(self.command())
    }

    /// The arts list the forecast was opened from, else the action menu,
    /// on `Attack` (`Magic` for a spell).
    fn back(self, state: &BattleState) -> Mode {
        let entry = match self.with {
            Equipped::Weapon(_) => MenuEntry::Attack,
            Equipped::Spell(_) => MenuEntry::Magic,
        };
        match self.arts {
            Some((menu, lines)) => Mode::List {
                sel: self.sel,
                menu,
                kind: ListKind::Arts(lines),
            },
            None => back_to_entry(self.sel, state, entry),
        }
    }

    fn into_mode(self) -> Mode {
        Mode::Targeting(Box::new(self))
    }
}

impl Picker for CastTargeting {
    fn cycle_target(&mut self, forward: bool, state: &BattleState) {
        self.cycle(forward, state);
    }

    fn tile(&self, state: &BattleState) -> Option<Pos> {
        target_pos(state, self.target())
    }

    fn confirm(self, _: &BattleState) -> (Mode, Effect) {
        apply(self.command())
    }

    fn back(self, _: &BattleState) -> Mode {
        let kind = ListKind::Spells(self.choices);
        let (sel, menu) = (self.sel, self.menu);
        Mode::List { sel, menu, kind }
    }

    fn into_mode(self) -> Mode {
        Mode::CastTarget(Box::new(self))
    }
}

impl Picker for SkillTargeting {
    fn cycle_target(&mut self, forward: bool, _: &BattleState) {
        self.cycle(forward);
    }

    fn tile(&self, state: &BattleState) -> Option<Pos> {
        state.unit(self.target()).map(|u| u.pos)
    }

    fn confirm(self, _: &BattleState) -> (Mode, Effect) {
        apply(self.command())
    }

    fn back(self, _: &BattleState) -> Mode {
        let kind = ListKind::Skills(self.choices);
        let (sel, menu) = (self.sel, self.menu);
        Mode::List { sel, menu, kind }
    }

    fn into_mode(self) -> Mode {
        Mode::SkillTarget(Box::new(self))
    }
}

impl Picker for ItemTargeting {
    fn cycle_target(&mut self, forward: bool, _: &BattleState) {
        self.cycle(forward);
    }

    fn tile(&self, state: &BattleState) -> Option<Pos> {
        target_tile(state, &self.sel, self.target())
    }

    fn confirm(self, _: &BattleState) -> (Mode, Effect) {
        apply(self.command())
    }

    fn back(self, _: &BattleState) -> Mode {
        let kind = ListKind::Pack(self.groups);
        let (sel, menu) = (self.sel, self.menu);
        Mode::List { sel, menu, kind }
    }

    fn into_mode(self) -> Mode {
        Mode::ItemTarget(Box::new(self))
    }
}

/// Picking who to talk to (0705): the cursor keys and
/// `NextUnit`/`PrevUnit` cycle `targets`, Confirm talks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TalkTargeting {
    /// The unit talking and its path (it stands at the path's end).
    pub sel: Selection,
    /// The units it can talk to ([`BattleState::talk_targets`]).
    pub targets: Vec<UnitId>,
    /// The one under the cursor.
    pub index: usize,
}

impl Picker for TalkTargeting {
    fn cycle_target(&mut self, forward: bool, _: &BattleState) {
        self.index = wrap_step(self.index, self.targets.len(), forward);
    }

    fn tile(&self, state: &BattleState) -> Option<Pos> {
        let id = *self.targets.get(self.index)?;
        state.unit(id).map(|u| u.pos)
    }

    /// Talks: free, so the action menu opens again after the scene.
    fn confirm(self, state: &BattleState) -> (Mode, Effect) {
        let Some(&target) = self.targets.get(self.index) else {
            return (self.back(state), Effect::None);
        };
        let cmd = Command::Talk {
            unit: self.sel.unit,
            dest: self.sel.dest(),
            target,
        };
        (Mode::default(), Effect::ApplyStay(cmd, Box::new(self.sel)))
    }

    /// The action menu, on `Talk`.
    fn back(self, state: &BattleState) -> Mode {
        back_to_entry(self.sel, state, MenuEntry::Talk)
    }

    /// Puts the cursor back on the target.
    fn other_key(self, state: &BattleState) -> (Mode, Effect) {
        aim(self, state)
    }

    fn into_mode(self) -> Mode {
        Mode::TalkTarget(self)
    }
}

/// [`step`] while browsing: Confirm selects a ready player unit or toggles
/// another faction's unit's threat area, and opens the map menu on an empty
/// tile; Cancel hides the threat area, or opens the map menu if none is
/// shown; `Info` opens the info screen of the unit under the cursor;
/// `EndTurn` ends the phase (asking first if units are still ready).
fn step_idle(threat: Option<Threat>, action: Action, at: MenuAt<'_>) -> (Mode, Effect) {
    let MenuAt { state, cursor, .. } = at;
    let idle = |threat| (Mode::Idle { threat }, Effect::None);
    match action {
        Action::Confirm => {
            let Some(unit) = state.units().iter().find(|u| u.pos == cursor) else {
                return (at.open(MapEntry::Units), Effect::None);
            };
            if is_ready_player(state, unit.id) {
                return match Selection::new(state, unit.id) {
                    Some(sel) => (Mode::Selected(sel), Effect::None),
                    None => idle(threat),
                };
            }
            if unit.faction == Faction::Player {
                return idle(threat);
            }
            if threat.as_ref().is_some_and(|t| t.unit == unit.id) {
                idle(None)
            } else {
                idle(threat_of(state, unit.id))
            }
        }
        Action::Cancel if threat.is_some() => idle(None),
        Action::Cancel | Action::Menu => (at.open(MapEntry::Units), Effect::None),
        Action::Info => match state.units().iter().find(|u| u.pos == cursor) {
            Some(unit) => (Mode::Info { unit: unit.id }, Effect::None),
            None => idle(threat),
        },
        Action::EndTurn => end_turn(state),
        _ => idle(threat),
    }
}

#[cfg(test)]
mod tests {
    use trpg_core::Objective;

    use super::*;
    use crate::screen::tests::ctx;
    use crate::screens::battle::art_list::Technique;
    use crate::screens::battle::attack::can_hit;
    use crate::screens::battle::playback::TIMINGS;
    use crate::screens::battle::quick_battle;
    use crate::screens::battle::testing::{battle_with, skirmish, vaulted, wait};

    fn p(x: i32, y: i32) -> Pos {
        Pos::new(x, y)
    }

    /// A keymap with `pairs` bound.
    fn keys(pairs: &[(&str, Action)]) -> Keymap {
        let bindings = pairs
            .iter()
            .filter_map(|&(c, a)| trpg_content::Chord::parse(c).ok().map(|c| (c, a)));
        Keymap::new(bindings, trpg_content::RepeatDef::default())
    }

    #[test]
    fn route_applies_the_optional_split_keys() {
        use Action::{Cancel, Confirm, ConfirmEndTurn, EndTurn, Select};
        let plain = keys(&[("f", Confirm), ("Space", EndTurn)]);
        let split = keys(&[
            ("f", Confirm),
            ("Space", EndTurn),
            ("g", Select),
            ("Enter", ConfirmEndTurn),
        ]);
        let map = [
            Mode::default(),
            Mode::MoveAfter {
                unit: UnitId(1),
                tiles: vec![],
            },
        ];
        for mode in &map {
            assert!(mode.picks_on_map());
            assert_eq!(mode.route(Confirm, &plain, Device::Keyboard), Some(Confirm));
            assert_eq!(mode.route(Confirm, &split, Device::Keyboard), None);
            assert_eq!(mode.route(Select, &split, Device::Keyboard), Some(Confirm));
            assert_eq!(mode.route(Cancel, &split, Device::Keyboard), Some(Cancel));
            assert_eq!(mode.route(EndTurn, &split, Device::Keyboard), Some(EndTurn));
        }
        // Menus keep Confirm; Select does nothing there.
        let menu = Mode::Objective;
        assert!(!menu.picks_on_map());
        assert_eq!(menu.route(Confirm, &split, Device::Keyboard), Some(Confirm));
        assert_eq!(menu.route(Select, &split, Device::Keyboard), Some(Select));
        let (menu, _) = step(
            menu,
            Select,
            p(0, 0),
            &quick_battle(&ctx().game.content).unwrap(),
        );
        assert_eq!(menu, Mode::Objective);
        // The end-turn prompt.
        let prompt = Mode::EndTurnPrompt { ready: 2 };
        assert_eq!(
            prompt.route(EndTurn, &plain, Device::Keyboard),
            Some(Confirm)
        );
        assert_eq!(
            prompt.route(Confirm, &plain, Device::Keyboard),
            Some(Confirm)
        );
        assert_eq!(prompt.route(EndTurn, &split, Device::Keyboard), None);
        assert_eq!(
            prompt.route(ConfirmEndTurn, &split, Device::Keyboard),
            Some(Confirm)
        );
        assert_eq!(
            prompt.route(Confirm, &split, Device::Keyboard),
            Some(Confirm)
        );
        assert_eq!(prompt.route(Cancel, &split, Device::Keyboard), Some(Cancel));
        // A pad has no button for Select or Confirm end turn: its Confirm
        // still picks and accepts (0230).
        let pad = Device::Pad(crate::input::PadKind::Xbox);
        assert_eq!(Mode::default().route(Confirm, &split, pad), Some(Confirm));
        assert_eq!(prompt.route(EndTurn, &split, pad), Some(Confirm));
    }

    /// The Quick Battle: lord 1 at (3, 5), knight 2 at (4, 6), archer 3 at
    /// (2, 4); brigands 4 at (8, 2) and 5 at (7, 4), raider 6 at (7, 1).
    fn quick() -> BattleState {
        quick_battle(&ctx().game.content).unwrap()
    }

    /// The Quick Battle with `objective`.
    fn with_objective(objective: Objective) -> BattleState {
        let c = ctx();
        let q = quick();
        battle_with(&c, q.map().clone(), q.units().to_vec(), objective)
    }

    /// `actions` from `mode` with the cursor on `cursor`, stopping at the
    /// first effect other than `None`.
    fn run(mode: Mode, actions: &[Action], cursor: Pos, s: &BattleState) -> (Mode, Effect) {
        let mut mode = mode;
        for &a in actions {
            let (m, effect) = step(mode, a, cursor, s);
            if effect != Effect::None {
                return (m, effect);
            }
            mode = m;
        }
        (mode, Effect::None)
    }

    /// The lord selected, with the cursor (and path) steered through
    /// `cursor`.
    fn selected(s: &BattleState, cursor: &[Pos]) -> Selection {
        let (Mode::Selected(mut sel), Effect::None) =
            step(Mode::default(), Action::Confirm, p(3, 5), s)
        else {
            panic!("the lord isn't selected");
        };
        for &to in cursor {
            sel.steer(to, s);
        }
        sel
    }

    #[test]
    fn confirm_on_a_ready_player_unit_selects_it_with_its_ranges() {
        let s = quick();
        let sel = selected(&s, &[]);
        assert_eq!((sel.unit, sel.path.clone()), (UnitId(1), vec![p(3, 5)]));
        assert_eq!((sel.origin(), sel.dest()), (p(3, 5), p(3, 5)));
        // Mov 5, through the knight's tile.
        assert!(sel.moves.contains(p(3, 5)) && sel.moves.contains(p(4, 6)));
        assert!(sel.moves.contains(p(5, 5)) && !sel.moves.contains(p(10, 5)));
        assert!(!sel.attack.is_empty());
        for pos in sel.attack.iter() {
            assert!(!sel.moves.contains(pos), "{pos:?} is in both");
        }
        // The knight's tile is passable but not a place to stop.
        assert!(!sel.reach.is_stoppable(p(4, 6)));
    }

    #[test]
    fn confirm_elsewhere_while_browsing_does_nothing() {
        let mut s = quick();
        // A player unit that has acted.
        wait(&mut s, 2);
        assert_eq!(
            step(Mode::default(), Action::Confirm, p(2, 4), &s),
            (Mode::default(), Effect::None)
        );
        // Other keys.
        for a in [Action::Info, Action::DangerZone, Action::EndTurn] {
            assert_eq!(step(Mode::default(), a, p(3, 5), &s).1, Effect::None);
        }
    }

    #[test]
    fn confirm_moves_only_to_a_stoppable_path_end() {
        let s = quick();
        // Onto the knight: passable, not stoppable. Ignored.
        let sel = selected(&s, &[p(4, 5), p(4, 6)]);
        assert_eq!(sel.dest(), p(4, 6));
        let (mode, effect) = step(Mode::Selected(sel.clone()), Action::Confirm, p(4, 6), &s);
        assert_eq!((mode, effect), (Mode::Selected(sel), Effect::None));
        // The cursor off the path's end (on an unreachable tile). Ignored.
        let sel = selected(&s, &[p(4, 5)]);
        let (mode, _) = step(Mode::Selected(sel.clone()), Action::Confirm, p(13, 0), &s);
        assert_eq!(mode, Mode::Selected(sel));
        // On an empty reachable tile: the walk starts.
        let sel = selected(&s, &[p(4, 5), p(5, 5)]);
        let (mode, effect) = step(Mode::Selected(sel.clone()), Action::Confirm, p(5, 5), &s);
        assert_eq!(effect, Effect::None);
        assert_eq!(
            mode,
            Mode::Moving {
                sel,
                t: 0.0,
                held: 0.0,
                pace: WALK_TILES_PER_S,
            }
        );
        // On the unit itself: straight to the menu.
        let sel = selected(&s, &[]);
        let (mode, _) = step(Mode::Selected(sel), Action::Confirm, p(3, 5), &s);
        assert!(matches!(mode, Mode::ActionMenu { .. }), "{mode:?}");
    }

    #[test]
    fn cancel_while_selected_puts_the_cursor_back_on_the_unit() {
        let s = quick();
        let sel = selected(&s, &[p(4, 5), p(5, 5)]);
        assert_eq!(
            step(Mode::Selected(sel.clone()), Action::Cancel, p(5, 5), &s),
            (Mode::default(), Effect::Cursor(p(3, 5)))
        );
        // Other keys keep the selection.
        assert_eq!(
            step(Mode::Selected(sel.clone()), Action::Info, p(5, 5), &s),
            (Mode::Selected(sel), Effect::None)
        );
    }

    #[test]
    fn the_walk_steps_at_twelve_tiles_a_second_then_opens_the_menu() {
        let s = quick();
        let sel = selected(&s, &[p(4, 5), p(5, 5), p(6, 5)]);
        let mut mode = Mode::Moving {
            sel,
            t: 0.0,
            held: 0.0,
            pace: WALK_TILES_PER_S,
        };
        let lord = UnitId(1);
        assert_eq!(mode.drawn_pos(lord), Some(p(3, 5)));
        assert_eq!(mode.drawn_pos(UnitId(2)), None);
        assert!(!mode.cursor_free());
        let tick = 1.0 / WALK_TILES_PER_S;
        mode = mode.tick(tick * 1.01, false, &s);
        assert_eq!(mode.drawn_pos(lord), Some(p(4, 5)));
        mode = mode.tick(tick, false, &s);
        assert_eq!(mode.drawn_pos(lord), Some(p(5, 5)));
        // A bad frame time changes nothing.
        mode = mode.tick(f32::NAN, false, &s);
        mode = mode.tick(-1.0, false, &s);
        assert_eq!(mode.drawn_pos(lord), Some(p(5, 5)));
        // Keys other than Confirm are ignored while walking.
        for a in [Action::Cancel, Action::CursorLeft, Action::Info] {
            let (m, effect) = step(mode.clone(), a, p(6, 5), &s);
            assert_eq!((&m, effect), (&mode, Effect::None));
        }
        mode = mode.tick(tick, false, &s);
        // At the end: the menu, the unit drawn there.
        assert!(matches!(mode, Mode::ActionMenu { .. }), "{mode:?}");
        assert_eq!(mode.drawn_pos(lord), Some(p(6, 5)));
        assert_eq!(mode.drawn_pos(UnitId(2)), None);
        assert!(!mode.cursor_free());
        assert_eq!(mode.selection().map(|s| s.unit), Some(lord));
        // Browsing: nothing is drawn elsewhere, nothing is selected.
        assert_eq!(Mode::default().drawn_pos(lord), None);
        assert_eq!(Mode::default().selection(), None);
        assert_eq!(Mode::default().tick(1.0, true, &s), Mode::default());
    }

    #[test]
    fn holding_or_pressing_confirm_skips_the_walk() {
        let s = quick();
        let sel = selected(&s, &[p(4, 5), p(5, 5), p(6, 5), p(7, 5)]);
        let walk = Mode::Moving {
            sel,
            t: 0.0,
            held: 0.0,
            pace: WALK_TILES_PER_S,
        };
        let menu = |m: &Mode| matches!(m, Mode::ActionMenu { .. });
        // Held, but not long enough; released; held again long enough.
        let short = HOLD_SKIP_S * 0.6;
        let mut mode = walk.clone().tick(short, true, &s);
        assert!(!menu(&mode));
        mode = mode.tick(0.0, false, &s).tick(short, true, &s);
        assert!(!menu(&mode), "the hold restarted when released");
        mode = mode.tick(short, true, &s);
        assert!(menu(&mode));
        // The hold alone skips: early in the walk, held just past the limit.
        let Mode::Moving { sel, .. } = walk.clone() else {
            unreachable!()
        };
        let nearly = Mode::Moving {
            sel,
            t: 0.0,
            held: short,
            pace: WALK_TILES_PER_S,
        };
        assert!(menu(&nearly.clone().tick(short, true, &s)));
        assert!(!menu(&nearly.tick(short, false, &s)));
        // A new press skips at once.
        let (mode, _) = step(walk, Action::Confirm, p(7, 5), &s);
        assert!(menu(&mode));
    }

    /// The lord walked to (5, 5), its menu open.
    fn lord_menu(s: &BattleState) -> Mode {
        let sel = selected(s, &[p(4, 5), p(5, 5)]);
        let walk = Mode::Moving {
            sel,
            t: 0.0,
            held: 0.0,
            pace: WALK_TILES_PER_S,
        };
        walk.tick(1.0, false, s)
    }

    #[test]
    fn the_menu_offers_a_disabled_attack_and_wait_then_waits() {
        let s = quick();
        let mode = lord_menu(&s);
        let Mode::ActionMenu { menu, entries, .. } = &mode else {
            panic!("{mode:?}");
        };
        assert_eq!(
            entries,
            &[
                MenuEntry::Attack,
                MenuEntry::Arts,
                MenuEntry::Skill,
                MenuEntry::Item,
                MenuEntry::Equip,
                MenuEntry::Wait
            ]
        );
        let labels: Vec<(&str, bool)> = menu
            .items()
            .iter()
            .map(|i| (i.label.as_str(), i.enabled))
            .collect();
        // Nobody is hurt (no item to use) and nobody is in reach of a sword
        // or its arts, but the lord has two weapons and its Inspire has
        // allies near to inspire.
        let want = [
            ("Attack", false),
            ("Arts", false),
            ("Skill", true),
            ("Item", false),
            ("Equip", true),
            ("Wait", true),
        ];
        assert_eq!(labels, want);
        // Up and Down stay on Wait; Confirm sends the move and the wait.
        let (mode, effect) = run(
            mode,
            &[Action::CursorUp, Action::CursorDown, Action::Confirm],
            p(5, 5),
            &s,
        );
        assert_eq!(mode, Mode::default());
        assert_eq!(
            effect,
            Effect::Apply(Command::Act {
                unit: UnitId(1),
                dest: p(5, 5),
                action: UnitAction::Wait,
            })
        );
    }

    #[test]
    fn cancel_in_the_menu_goes_back_to_the_same_path() {
        let s = quick();
        let mode = lord_menu(&s);
        let sel = mode.selection().cloned().unwrap();
        assert_eq!(sel.path, [p(3, 5), p(4, 5), p(5, 5)]);
        assert_eq!(
            step(mode, Action::Cancel, p(5, 5), &s),
            (Mode::Selected(sel), Effect::None)
        );
    }

    #[test]
    fn a_menu_line_with_no_entry_behind_it_does_nothing() {
        let s = quick();
        let Mode::ActionMenu {
            sel,
            menu,
            entries,
            weapons,
        } = lord_menu(&s)
        else {
            panic!("no action menu");
        };
        // One line more than there are entries, focused.
        let mut items = menu.items().to_vec();
        items.push(MenuItem::new("Extra"));
        let menu = Menu::new(items).focused(entries.len());
        let mode = Mode::ActionMenu {
            sel,
            menu,
            entries,
            weapons,
        };
        let (same, effect) = step(mode.clone(), Action::Confirm, p(5, 5), &s);
        assert_eq!((same, effect), (mode, Effect::None));
    }

    #[test]
    fn seize_is_offered_only_where_it_is_legal() {
        let seize = |by_lord| Objective::Seize {
            pos: p(5, 5),
            by_lord,
            turn_limit: None,
        };
        let s = with_objective(seize(true));
        let Mode::ActionMenu { entries, menu, .. } = lord_menu(&s) else {
            panic!("no menu");
        };
        let all = [
            MenuEntry::Attack,
            MenuEntry::Arts,
            MenuEntry::Seize,
            MenuEntry::Skill,
            MenuEntry::Item,
            MenuEntry::Equip,
            MenuEntry::Wait,
        ];
        assert_eq!(entries, all);
        assert_eq!(menu.focus(), 2, "focus on Seize");
        let (_, effect) = run(lord_menu(&s), &[Action::Confirm], p(5, 5), &s);
        assert_eq!(
            effect,
            Effect::Apply(Command::Act {
                unit: UnitId(1),
                dest: p(5, 5),
                action: UnitAction::Seize,
            })
        );
        // Elsewhere, or by a non-lord where the lord must seize: no Seize.
        let sel = selected(&s, &[p(4, 5)]);
        let plain = [
            MenuEntry::Attack,
            MenuEntry::Arts,
            MenuEntry::Skill,
            MenuEntry::Item,
            MenuEntry::Equip,
            MenuEntry::Wait,
        ];
        assert_eq!(menu_entries(&sel, &s), plain);
        let mut knight = sel;
        knight.unit = UnitId(2);
        knight.path = vec![p(5, 5)];
        assert_eq!(menu_entries(&knight, &s), plain);
        let s = with_objective(seize(false));
        assert_eq!(menu_entries(&knight, &s), all);
        assert_eq!(
            all.map(|entry| entry.label(Words::ENGLISH)),
            ["Attack", "Arts", "Seize", "Skill", "Item", "Equip", "Wait"]
        );
    }

    /// The unit whose threat area `mode` shows, if any.
    fn shown(mode: &Mode) -> Option<UnitId> {
        match mode {
            Mode::Idle { threat: Some(t) } => Some(t.unit),
            _ => None,
        }
    }

    #[test]
    fn confirm_on_an_enemy_toggles_its_threat_area() {
        let s = quick();
        let (mode, effect) = step(Mode::default(), Action::Confirm, p(8, 2), &s);
        assert_eq!((shown(&mode), effect), (Some(UnitId(4)), Effect::None));
        let Mode::Idle {
            threat: Some(threat),
        } = &mode
        else {
            panic!("{mode:?}");
        };
        // Its own tile, a tile it can walk to and hit from, not a far one.
        assert!(threat.area.contains(p(8, 2)));
        assert!(threat.area.contains(p(8, 7)));
        assert!(!threat.area.contains(p(0, 7)));
        // Browsing on with it shown: the cursor may move, cycling doesn't
        // hide it.
        assert!(mode.cursor_free());
        let (mode, _) = step(mode, Action::NextUnit, p(8, 2), &s);
        assert_eq!(shown(&mode), Some(UnitId(4)));
        // Another enemy: its area instead.
        let (mode, _) = step(mode, Action::Confirm, p(7, 1), &s);
        assert_eq!(shown(&mode), Some(UnitId(6)));
        // The same enemy again hides it.
        let (mode, _) = step(mode, Action::Confirm, p(7, 1), &s);
        assert_eq!(mode, Mode::default());
        // Cancel hides it rather than leaving.
        let (mode, _) = step(Mode::default(), Action::Confirm, p(8, 2), &s);
        assert_eq!(
            step(mode, Action::Cancel, p(8, 2), &s),
            (Mode::default(), Effect::None)
        );
        let (menu, effect) = step(Mode::default(), Action::Cancel, p(8, 2), &s);
        assert!(matches!(menu, Mode::MapMenu { .. }), "{menu:?}");
        assert_eq!(effect, Effect::None);
    }

    #[test]
    fn a_player_unit_that_acted_keeps_the_threat_area_and_a_ready_one_hides_it() {
        let mut s = quick();
        wait(&mut s, 2);
        let (mode, _) = step(Mode::default(), Action::Confirm, p(8, 2), &s);
        let (mode, _) = step(mode, Action::Confirm, p(2, 4), &s);
        assert_eq!(shown(&mode), Some(UnitId(4)));
        let (mode, _) = step(mode, Action::Confirm, p(3, 5), &s);
        assert!(matches!(mode, Mode::Selected(_)), "{mode:?}");
    }

    #[test]
    fn a_move_after_an_attack_waits_for_a_tile_or_the_unit() {
        let s = vaulted(&ctx());
        let mode = Mode::after_command(&s);
        let Mode::MoveAfter { unit, tiles } = &mode else {
            panic!("{mode:?}");
        };
        assert_eq!(*unit, UnitId(3));
        assert_eq!(tiles, &s.move_after_tiles());
        assert!(!tiles.is_empty());
        assert!(mode.cursor_free());
        assert_eq!(mode.selection(), None);
        let to = tiles[0];
        // Anything but Confirm on a highlighted tile or on the unit: waits.
        for (a, at) in [
            (Action::Cancel, to),
            (Action::Confirm, p(0, 0)),
            (Action::Info, p(8, 4)),
        ] {
            let (m, effect) = step(mode.clone(), a, at, &s);
            assert_eq!((&m, effect), (&mode, Effect::None));
        }
        let moved = |at| step(mode.clone(), Action::Confirm, at, &s);
        assert_eq!(
            moved(to),
            (
                Mode::default(),
                Effect::Apply(Command::MoveAfter {
                    unit: UnitId(3),
                    to: Some(to),
                })
            )
        );
        assert_eq!(
            moved(p(8, 4)).1,
            Effect::Apply(Command::MoveAfter {
                unit: UnitId(3),
                to: None,
            })
        );
        // Without a pending move: browsing.
        assert_eq!(Mode::after_command(&quick()), Mode::default());
    }

    /// The skirmish's lord walked to (7, 2), its menu open.
    fn skirmish_menu(s: &BattleState) -> Mode {
        let (Mode::Selected(mut sel), _) = step(Mode::default(), Action::Confirm, p(6, 2), s)
        else {
            panic!("the lord isn't selected");
        };
        sel.steer(p(7, 2), s);
        let (walk, _) = step(Mode::Selected(sel), Action::Confirm, p(7, 2), s);
        walk.tick(1.0, false, s)
    }

    /// The targeting state of `mode`.
    fn targeting(mode: &Mode) -> &Targeting {
        match mode {
            Mode::Targeting(t) => t,
            _ => panic!("{mode:?}"),
        }
    }

    #[test]
    fn attack_is_enabled_when_a_weapon_reaches_and_opens_the_forecast() {
        use trpg_core::Equipped::Weapon;

        let s = skirmish(&ctx(), 20);
        let menu = skirmish_menu(&s);
        let Mode::ActionMenu {
            menu: m, weapons, ..
        } = &menu
        else {
            panic!("{menu:?}");
        };
        assert!(m.items()[0].enabled);
        assert_eq!(m.focus(), 0, "focus on Attack");
        assert_eq!(weapons.len(), 2);
        // Attack: straight to the forecast with the equipped iron sword
        // (0442: no weapon list), the cursor on the first target.
        let (mode, effect) = step(menu, Action::Confirm, p(7, 2), &s);
        assert_eq!(effect, Effect::Cursor(p(7, 1)));
        let t = targeting(&mode);
        assert_eq!((&t.with, t.target()), (&Weapon(0), UnitId(6)));
        assert_eq!(t.options, [Weapon(0), Weapon(1)]);
        assert_eq!((&t.technique, &t.arts), (&Technique::Attack, &None));
        assert_eq!(mode.selection().map(|s| s.unit), Some(UnitId(1)));
        assert_eq!(mode.drawn_pos(UnitId(1)), Some(p(7, 2)));
        assert!(!mode.cursor_free());
        // Right swaps to the steel sword, on the same target; left swaps
        // back (wrapping either way with two).
        let (steel, effect) = step(mode.clone(), Action::CursorRight, p(7, 1), &s);
        assert_eq!(effect, Effect::Cursor(p(7, 1)));
        let t = targeting(&steel);
        assert_eq!((&t.with, t.target()), (&Weapon(1), UnitId(6)));
        assert_eq!(t.options, [Weapon(0), Weapon(1)]);
        let (iron, _) = step(steel.clone(), Action::CursorLeft, p(7, 1), &s);
        assert_eq!(iron, mode);
        let (wrapped, _) = step(mode, Action::CursorLeft, p(7, 1), &s);
        assert_eq!(wrapped, steel);
        // Cancel: back to the menu, on Attack, the cursor on the lord.
        let (back, effect) = step(steel, Action::Cancel, p(7, 1), &s);
        assert_eq!(effect, Effect::Cursor(p(7, 2)));
        let Mode::ActionMenu { menu, entries, .. } = &back else {
            panic!("{back:?}");
        };
        assert_eq!(entries[menu.focus()], MenuEntry::Attack);
    }

    #[test]
    fn targeting_cycles_with_up_down_and_the_unit_keys_and_confirm_attacks() {
        let s = skirmish(&ctx(), 20);
        let (mut mode, _) = step(skirmish_menu(&s), Action::Confirm, p(7, 2), &s);
        let target = |m: &Mode| targeting(m).target();
        // Up and down change the target (0442: there is no arts list), as
        // the unit keys do; down and next go forward.
        for (a, id, at) in [
            (Action::CursorDown, 4, p(8, 2)),
            (Action::NextUnit, 6, p(7, 1)),
            (Action::CursorUp, 4, p(8, 2)),
            (Action::PrevUnit, 6, p(7, 1)),
        ] {
            let (m, effect) = step(mode, a, p(0, 0), &s);
            assert_eq!(
                (target(&m), effect),
                (UnitId(id), Effect::Cursor(at)),
                "{a:?}"
            );
            assert_eq!(targeting(&m).with, trpg_core::Equipped::Weapon(0), "{a:?}");
            mode = m;
        }
        // With a single weapon to attack with, left and right change the
        // target too.
        let Mode::Targeting(mut single) = mode.clone() else {
            unreachable!()
        };
        single.options.truncate(1);
        let mut one = Mode::Targeting(single);
        for (a, id, at) in [
            (Action::CursorRight, 4, p(8, 2)),
            (Action::CursorLeft, 6, p(7, 1)),
        ] {
            let (m, effect) = step(one, a, p(0, 0), &s);
            assert_eq!(
                (target(&m), effect),
                (UnitId(id), Effect::Cursor(at)),
                "{a:?}"
            );
            one = m;
        }
        // Other keys do nothing.
        let (same, effect) = step(mode.clone(), Action::Info, p(7, 1), &s);
        assert_eq!((&same, effect), (&mode, Effect::None));
        let (after, effect) = step(mode, Action::Confirm, p(7, 1), &s);
        assert_eq!(after, Mode::default());
        assert_eq!(
            effect,
            Effect::Apply(Command::Act {
                unit: UnitId(1),
                dest: p(7, 2),
                action: UnitAction::Attack {
                    target: UnitId(6),
                    slot: 0,
                    active: None,
                    art: None,
                },
            })
        );
    }

    /// Ticket 0442: an art is chosen in the arts menu, then its target; the
    /// attack carries the art, and Cancel goes back to the arts menu, then
    /// to the action menu on `Arts`.
    #[test]
    fn arts_opens_the_arts_menu_then_the_forecast_and_cancel_goes_back() {
        use trpg_core::Equipped::Weapon;

        let s = skirmish(&ctx(), 20);
        let menu = skirmish_menu(&s);
        let Mode::ActionMenu {
            entries, menu: m, ..
        } = &menu
        else {
            panic!("{menu:?}");
        };
        assert_eq!(entries[..2], [MenuEntry::Attack, MenuEntry::Arts]);
        assert!(m.items()[1].enabled);
        let (on_arts, _) = step(menu, Action::CursorDown, p(7, 2), &s);
        let (arts, effect) = step(on_arts, Action::Confirm, p(7, 2), &s);
        assert_eq!(effect, Effect::None);
        let Mode::List {
            menu: m,
            kind: ListKind::Arts(lines),
            ..
        } = &arts
        else {
            panic!("{arts:?}");
        };
        let names: Vec<String> = lines
            .iter()
            .map(|l| l.technique.name(&s, Words::ENGLISH))
            .collect();
        assert_eq!(names, ["Flowing Cut", "Guard Break"]);
        assert!(lines.iter().all(|l| l.slots == [0, 1] && l.slot == 0));
        assert_eq!(m.focus(), 0);
        assert_eq!(arts.selection().map(|s| s.unit), Some(UnitId(1)));
        assert_eq!(arts.drawn_pos(UnitId(1)), Some(p(7, 2)));
        assert!(!arts.cursor_free() && !arts.picks_on_map());
        // Guard Break: its forecast on the first target, the cursor on it.
        let (on_break, _) = step(arts, Action::CursorDown, p(7, 2), &s);
        let (mode, effect) = step(on_break.clone(), Action::Confirm, p(7, 2), &s);
        assert_eq!(effect, Effect::Cursor(p(7, 1)));
        let guard_break = Technique::Art(trpg_core::ArtId::new("guard_break"));
        let t = targeting(&mode);
        assert_eq!((&t.with, t.target()), (&Weapon(0), UnitId(6)));
        assert_eq!(t.technique, guard_break);
        assert_eq!(t.options, [Weapon(0), Weapon(1)]);
        assert_eq!(t.preview.art, Some(trpg_core::ArtId::new("guard_break")));
        // Down changes the target and right swaps the weapon; the art stays.
        let (mode, _) = step(mode, Action::CursorDown, p(7, 1), &s);
        let (mode, effect) = step(mode, Action::CursorRight, p(8, 2), &s);
        assert_eq!(effect, Effect::Cursor(p(8, 2)));
        let t = targeting(&mode);
        assert_eq!((&t.with, t.target()), (&Weapon(1), UnitId(4)));
        assert_eq!(t.technique, guard_break);
        assert_eq!(t.preview.art, Some(trpg_core::ArtId::new("guard_break")));
        // Confirm: the attack carries the art.
        let (after, effect) = step(mode.clone(), Action::Confirm, p(8, 2), &s);
        assert_eq!(after, Mode::default());
        assert_eq!(
            effect,
            Effect::Apply(Command::Act {
                unit: UnitId(1),
                dest: p(7, 2),
                action: UnitAction::Attack {
                    target: UnitId(4),
                    slot: 1,
                    active: None,
                    art: Some(trpg_core::ArtId::new("guard_break")),
                },
            })
        );
        // Cancel: the arts menu as it was, the cursor on the lord; Cancel
        // again: the action menu, on Arts.
        let (back, effect) = step(mode, Action::Cancel, p(8, 2), &s);
        assert_eq!(effect, Effect::Cursor(p(7, 2)));
        assert_eq!(back, on_break);
        let (back, effect) = step(back, Action::Cancel, p(7, 2), &s);
        assert_eq!(effect, Effect::None);
        let Mode::ActionMenu { menu, entries, .. } = &back else {
            panic!("{back:?}");
        };
        assert_eq!(entries[menu.focus()], MenuEntry::Arts);
    }

    /// An enemy only an art reaches (the brigand beside the archer, with
    /// Close Shot) is attacked through `Arts`, not `Attack` or by pointing.
    #[test]
    fn an_enemy_only_an_art_reaches_is_attacked_through_arts() {
        let s = skirmish(&ctx(), 20);
        let (Mode::Selected(sel), _) = step(Mode::default(), Action::Confirm, p(8, 4), &s) else {
            panic!("the archer isn't selected");
        };
        assert!(can_hit(&s, UnitId(3), p(8, 4), UnitId(4)));
        assert!(!can_hit(&s, UnitId(3), p(8, 4), UnitId(5)));
        let (menu, _) = step(Mode::Selected(sel.clone()), Action::Confirm, p(8, 4), &s);
        let (on_arts, _) = step(menu, Action::CursorDown, p(8, 4), &s);
        let (arts, _) = step(on_arts, Action::Confirm, p(8, 4), &s);
        let Mode::List {
            kind: ListKind::Arts(lines),
            ..
        } = &arts
        else {
            panic!("{arts:?}");
        };
        let names: Vec<String> = lines
            .iter()
            .map(|l| l.technique.name(&s, Words::ENGLISH))
            .collect();
        assert_eq!(names, ["Close Shot", "Pinning Shot", "Vault"]);
        let (mode, effect) = step(arts, Action::Confirm, p(8, 4), &s);
        assert_eq!(effect, Effect::Cursor(p(8, 2)));
        let t = targeting(&mode);
        assert_eq!(t.targets, [UnitId(4), UnitId(5)]);
        assert!(!t.can_swap());
        // With the brigand two tiles up gone, `Attack` is dimmed and the
        // menu opens on `Arts`.
        let mut units = s.units().to_vec();
        units.retain(|u| u.id != UnitId(4));
        let rout = Objective::Rout { turn_limit: None };
        let s = battle_with(&ctx(), s.map().clone(), units, rout);
        let (menu, _) = step(Mode::Selected(sel), Action::Confirm, p(8, 4), &s);
        let Mode::ActionMenu { menu, entries, .. } = &menu else {
            panic!("{menu:?}");
        };
        assert!(!menu.items()[0].enabled);
        assert_eq!(entries[menu.focus()], MenuEntry::Arts);
    }

    #[test]
    fn arts_with_nothing_to_use_stays_in_the_menu() {
        let s = skirmish(&ctx(), 20);
        let sel = Selection::new(&s, UnitId(1)).unwrap();
        // The lord where it starts reaches nobody: the lines are dimmed.
        let lines = art_lines(&s, sel.unit, sel.dest());
        assert_eq!(lines.len(), 2);
        assert!(!can_use_art(&lines));
        let (mode, effect) = open_list(MenuEntry::Arts, sel.clone(), &s);
        assert_eq!(effect, Effect::None);
        assert!(matches!(mode, Mode::ActionMenu { .. }), "{mode:?}");
        // A dimmed line is never chosen.
        let menu = arts_menu(&s, IN_PLAY, &lines);
        let arts = Mode::List {
            sel: sel.clone(),
            menu: menu.clone(),
            kind: ListKind::Arts(lines.clone()),
        };
        let (same, effect) = step(arts.clone(), Action::Confirm, p(6, 2), &s);
        assert_eq!((same, effect), (arts, Effect::None));
        assert_eq!(Targeting::art(&s, sel, menu, lines, 0), None);
    }

    #[test]
    fn attack_with_no_weapon_in_reach_stays_in_the_menu() {
        let s = skirmish(&ctx(), 20);
        let sel = Selection::new(&s, UnitId(1)).unwrap();
        let (mode, effect) = choose_attack(sel, &[], &s);
        assert_eq!(effect, Effect::None);
        let Mode::ActionMenu { menu, entries, .. } = &mode else {
            panic!("{mode:?}");
        };
        assert_eq!(entries[menu.focus()], MenuEntry::Wait, "Attack is disabled");
    }

    #[test]
    fn one_weapon_goes_straight_to_targeting_and_back_to_the_menu() {
        let s = skirmish(&ctx(), 20);
        // The archer at (8, 4) attacks from where it stands.
        let (Mode::Selected(sel), _) = step(Mode::default(), Action::Confirm, p(8, 4), &s) else {
            panic!("the archer isn't selected");
        };
        let (menu, _) = step(Mode::Selected(sel), Action::Confirm, p(8, 4), &s);
        let (mode, effect) = step(menu, Action::Confirm, p(8, 4), &s);
        assert_eq!(effect, Effect::Cursor(p(8, 2)));
        let Mode::Targeting(t) = &mode else {
            panic!("{mode:?}");
        };
        // The brigand two tiles up; the one beside it only with Close Shot.
        assert_eq!((t.targets.clone(), t.can_swap()), (vec![UnitId(4)], false));
        let (back, effect) = step(mode, Action::Cancel, p(8, 2), &s);
        assert_eq!(effect, Effect::Cursor(p(8, 4)));
        let Mode::ActionMenu { menu, entries, .. } = &back else {
            panic!("{back:?}");
        };
        assert_eq!(entries[menu.focus()], MenuEntry::Attack);
    }

    #[test]
    fn during_a_combat_only_cancel_counts_and_the_end_goes_back_to_browsing() {
        let c = ctx();
        let before = skirmish(&c, 20);
        let mut after = before.clone();
        let cmd = Command::Act {
            unit: UnitId(1),
            dest: p(7, 2),
            action: UnitAction::Attack {
                target: UnitId(4),
                slot: 0,
                active: None,
                art: None,
            },
        };
        let events = after.apply(&cmd).unwrap();
        let playback = Playback::new(
            Words::ENGLISH,
            &events,
            before.units(),
            after.fallen(),
            TIMINGS,
        )
        .unwrap();
        let total = playback.total();
        let combat = Mode::Combat(Box::new(playback));
        assert!(!combat.cursor_free());
        assert_eq!(
            (combat.selection(), combat.drawn_pos(UnitId(1))),
            (None, None)
        );
        for a in [Action::Confirm, Action::CursorLeft, Action::NextUnit] {
            let (m, effect) = step(combat.clone(), a, p(8, 2), &after);
            assert_eq!((&m, effect), (&combat, Effect::None));
        }
        // Halfway, still playing; at the end, browsing.
        let half = combat.clone().tick(total / 2.0, false, &after);
        assert!(matches!(half, Mode::Combat(_)));
        assert_eq!(half.tick(total, false, &after), Mode::default());
        // Cancel skips: the frame's tick ends it.
        let (skipped, effect) = step(combat, Action::Cancel, p(8, 2), &after);
        assert_eq!(effect, Effect::None);
        assert_eq!(skipped.tick(1.0 / 60.0, false, &after), Mode::default());
    }

    /// The unit at `at` selected, its path at its origin.
    fn pick(s: &BattleState, at: Pos) -> Selection {
        let (Mode::Selected(sel), _) = step(Mode::default(), Action::Confirm, at, s) else {
            panic!("no unit at {at:?}");
        };
        sel
    }

    /// `sel` with the cursor moved onto `at`.
    fn aim(sel: Selection, at: Pos, s: &BattleState) -> Selection {
        let mut mode = Mode::Selected(sel);
        mode.cursor_moved(at, s);
        let Mode::Selected(sel) = mode else {
            panic!("not selected");
        };
        sel
    }

    #[test]
    fn pointing_at_an_enemy_aims_the_path_at_the_cheapest_tile_that_can_hit() {
        let s = skirmish(&ctx(), 20);
        // The lord at (6, 2), the raider (unit 6) at (7, 1): not adjacent.
        let start = pick(&s, p(6, 2));
        assert!(!can_hit(&s, UnitId(1), p(6, 2), UnitId(6)));
        let sel = aim(start.clone(), p(7, 1), &s);
        assert_eq!(sel.target, Some(UnitId(6)));
        let dest = sel.dest();
        assert!(start.reach.is_stoppable(dest) && can_hit(&s, UnitId(1), dest, UnitId(6)));
        for t in start.reach.stoppable().iter() {
            if can_hit(&s, UnitId(1), t, UnitId(6)) {
                assert!(start.reach.cost(dest) <= start.reach.cost(t), "{t:?}");
            }
        }
        assert_eq!(Some(sel.path), start.reach.path_to(dest));
    }

    #[test]
    fn pointing_at_an_enemy_keeps_a_path_that_already_hits() {
        let s = skirmish(&ctx(), 20);
        let mut start = pick(&s, p(6, 2));
        start.steer(p(7, 2), &s);
        for enemy in [p(7, 1), p(8, 2)] {
            let sel = aim(start.clone(), enemy, &s);
            assert_eq!(sel.path, start.path);
            assert!(sel.target.is_some());
        }
    }

    #[test]
    fn an_archer_points_from_two_tiles_away() {
        let s = quick();
        let enemy = s.unit(UnitId(5)).unwrap().pos;
        let sel = aim(pick(&s, p(2, 4)), enemy, &s);
        assert_eq!(sel.target, Some(UnitId(5)));
        let d = sel.dest();
        assert_eq!((d.x - enemy.x).abs() + (d.y - enemy.y).abs(), 2);
    }

    #[test]
    fn confirm_on_the_aimed_enemy_walks_then_opens_the_forecast_and_cancel_backs_out() {
        let s = skirmish(&ctx(), 20);
        let sel = aim(pick(&s, p(6, 2)), p(7, 1), &s);
        let dest = sel.dest();
        let (walk, effect) = step(Mode::Selected(sel), Action::Confirm, p(7, 1), &s);
        assert_eq!(effect, Effect::None);
        assert!(matches!(walk, Mode::Moving { .. }), "{walk:?}");
        let mut mode = walk.tick(1.0, false, &s);
        // Two swords reach the raider: the forecast with the equipped one
        // (0430); right and left swap between them, wrapping.
        let with = |mode: &Mode| match mode {
            Mode::Targeting(t) => (t.with.clone(), t.target()),
            other => panic!("{other:?}"),
        };
        assert_eq!(with(&mode), (Equipped::Weapon(0), UnitId(6)));
        for (key, slot) in [
            (Action::CursorRight, 1),
            (Action::CursorRight, 0),
            (Action::CursorLeft, 1),
            (Action::CursorLeft, 0),
        ] {
            let (next, effect) = step(mode, key, p(7, 1), &s);
            assert_eq!(effect, Effect::Cursor(p(7, 1)));
            assert_eq!(with(&next), (Equipped::Weapon(slot), UnitId(6)));
            mode = next;
        }
        // Cancel: the action menu at the tile, on Attack, then the path.
        let (menu, effect) = step(mode, Action::Cancel, p(7, 1), &s);
        assert_eq!(effect, Effect::Cursor(dest));
        let Mode::ActionMenu {
            sel,
            menu: m,
            entries,
            ..
        } = &menu
        else {
            panic!("{menu:?}");
        };
        assert_eq!(sel.dest(), dest);
        assert_eq!(entries[m.focus()], MenuEntry::Attack);
        let (back, _) = step(menu, Action::Cancel, dest, &s);
        let Mode::Selected(sel) = back else {
            panic!("{back:?}");
        };
        assert_eq!((sel.target, sel.dest()), (None, dest));
    }

    #[test]
    fn a_single_weapon_goes_straight_to_the_forecast_without_walking() {
        let s = skirmish(&ctx(), 20);
        // The archer already hits the brigand from where it stands.
        let sel = aim(pick(&s, p(8, 4)), p(8, 2), &s);
        assert_eq!((sel.path.len(), sel.target), (1, Some(UnitId(4))));
        let (mode, effect) = step(Mode::Selected(sel), Action::Confirm, p(8, 2), &s);
        assert_eq!(effect, Effect::Cursor(p(8, 2)));
        assert!(matches!(mode, Mode::Targeting(_)), "{mode:?}");
    }

    #[test]
    fn an_enemy_no_tile_can_hit_leaves_the_path_and_confirm_does_nothing() {
        let s = quick();
        // The lord at (3, 5) can't reach the raider at (7, 1) this turn.
        let start = pick(&s, p(3, 5));
        let far = s.unit(UnitId(6)).unwrap().pos;
        let sel = aim(start.clone(), far, &s);
        assert_eq!((sel.target, &sel.path), (None, &start.path));
        let (mode, effect) = step(Mode::Selected(sel.clone()), Action::Confirm, far, &s);
        assert_eq!((mode, effect), (Mode::Selected(sel), Effect::None));
    }

    #[test]
    fn an_aim_no_weapon_reaches_from_the_path_end_falls_back_to_the_action_menu() {
        let s = skirmish(&ctx(), 20);
        // Aimed at the raider but still standing at (6, 2), out of reach.
        let mut sel = pick(&s, p(6, 2));
        sel.target = Some(UnitId(6));
        let (mode, effect) = open_aimed(sel, &s);
        assert_eq!(effect, Effect::None);
        assert!(matches!(mode, Mode::ActionMenu { .. }), "{mode:?}");
    }

    #[test]
    fn the_forecast_opens_on_the_aimed_enemy_even_when_it_is_not_the_first_target() {
        let s = skirmish(&ctx(), 20);
        // From (7, 2) the raider (7, 1) comes first in (y, x) order; aim at
        // the brigand (8, 2) instead.
        let mut start = pick(&s, p(6, 2));
        start.steer(p(7, 2), &s);
        let sel = aim(start, p(8, 2), &s);
        let (mode, _) = step(Mode::Selected(sel), Action::Confirm, p(8, 2), &s);
        let targeting = mode.tick(1.0, false, &s);
        let Mode::Targeting(t) = &targeting else {
            panic!("{targeting:?}");
        };
        assert_eq!(t.target(), UnitId(4));
        // The raider is still a target: the next one, wrapping.
        assert_eq!(t.targets, [UnitId(4), UnitId(6)]);
        // A swap keeps the brigand under the cursor.
        let (swapped, effect) = step(targeting, Action::CursorRight, p(8, 2), &s);
        assert_eq!(effect, Effect::Cursor(p(8, 2)));
        let Mode::Targeting(t) = &swapped else {
            panic!("{swapped:?}");
        };
        assert_eq!(
            (&t.with, &t.targets),
            (&Equipped::Weapon(1), &vec![UnitId(4), UnitId(6)])
        );
    }

    /// The labels of `menu`'s lines.
    fn labels(menu: &Menu) -> Vec<String> {
        menu.items().iter().map(|i| i.label.clone()).collect()
    }

    /// The list `mode` shows.
    fn list(mode: &Mode) -> &Menu {
        match mode {
            Mode::List { menu, .. } | Mode::UnitList { menu, .. } => menu,
            _ => panic!("{mode:?}"),
        }
    }

    /// Ticket 0243: the action menu and the map menu are painted in the
    /// player's language too, with their focus and dimmed lines kept.
    #[test]
    fn the_action_and_map_menus_are_told_in_the_players_language() {
        let c = ctx();
        let s = skirmish(&c, 20);
        let code = trpg_content::LangCode::new(trpg_content::lang::TEST).unwrap();
        let words = Words::new(&c.game.content.lang, &code);
        let menu_of = |mode: &Mode| match mode {
            Mode::ActionMenu { menu, .. } | Mode::MapMenu { menu, .. } => menu.clone(),
            _ => panic!("{mode:?}"),
        };
        let enabled =
            |menu: &Menu| -> Vec<bool> { menu.items().iter().map(|i| i.enabled).collect() };
        // In the enemy's phase `End Turn` is dimmed.
        let mut theirs = s.clone();
        theirs.apply(&Command::EndPhase).unwrap();
        let open_map_menu = |state, focus| {
            let shortcuts = &[];
            let cursor = Pos::new(0, 0);
            MenuAt {
                state,
                cursor,
                shortcuts,
            }
            .open(focus)
        };
        let end_turn = open_map_menu(&s, MapEntry::EndTurn);
        let dimmed = open_map_menu(&theirs, MapEntry::Units);
        assert_ne!(enabled(&menu_of(&end_turn)), enabled(&menu_of(&dimmed)));
        let menus = [
            (skirmish_menu(&s), &s, "Attack", "ATTACK"),
            (end_turn, &s, "End Turn", "END TURN"),
            (dimmed, &theirs, "Units", "UNITS"),
        ];
        for (mode, state, english, shouted) in menus {
            let (ours, told) = (menu_of(&mode), mode.told(state, words).into_owned());
            let theirs = menu_of(&told);
            assert!(labels(&ours).iter().any(|l| l == english), "{english}");
            assert!(labels(&theirs).iter().any(|l| l == shouted), "{shouted}");
            assert!(!labels(&theirs).iter().any(|l| l == english), "{english}");
            assert_eq!(theirs.focus(), ours.focus(), "{english}");
            assert_eq!(enabled(&theirs), enabled(&ours), "{english}");
            assert_eq!(told.selection(), mode.selection());
            assert!(matches!(mode.told(state, IN_PLAY), Cow::Borrowed(_)));
        }
    }

    /// The screen paints the mode's lists in the player's language
    /// (ticket 0235); what is played stays as it is.
    #[test]
    fn a_mode_is_told_in_the_players_language_with_its_focus_kept() {
        use crate::words::testing::{shout, shouting};

        let c = ctx();
        let s = skirmish(&c, 20);
        let (lang, code) = (shouting(&c.game.content), shout());
        let words = Words::new(&lang, &code);
        let Mode::ActionMenu { sel, .. } = skirmish_menu(&s) else {
            panic!("no action menu");
        };
        let (units, ids) = unit_list(&s, IN_PLAY);
        let skills = skill_choices(&s, Words::ENGLISH, &sel);
        // Each list, with a name it shows in English.
        let lists = [
            (open_list(MenuEntry::Arts, sel.clone(), &s).0, "Guard Break"),
            (
                // No ally is near enough to inspire: the list, unopened.
                Mode::List {
                    menu: skill_menu(&s, IN_PLAY, sel.unit, &skills),
                    sel: sel.clone(),
                    kind: ListKind::Skills(skills),
                },
                "Inspire",
            ),
            (open_list(MenuEntry::Equip, sel.clone(), &s).0, "Iron Sword"),
            (
                Mode::UnitList {
                    menu: units,
                    units: ids,
                },
                "Test Knight",
            ),
        ];
        for (mode, name) in lists {
            // Move the focus off where a list opens, if it can move.
            let (mode, _) = step(mode, Action::CursorDown, p(7, 2), &s);
            let has = |mode: &Mode, name: &str| labels(list(mode)).join("\n").contains(name);
            assert!(has(&mode, name), "{name}: {:?}", labels(list(&mode)));
            // English is the mode itself.
            assert!(matches!(mode.told(&s, Words::ENGLISH), Cow::Borrowed(_)));
            assert!(matches!(mode.told(&s, IN_PLAY), Cow::Borrowed(_)));
            let told = mode.told(&s, words).into_owned();
            assert!(has(&told, &name.to_uppercase()), "{name}");
            assert!(!has(&told, name), "{name}");
            let (ours, theirs) = (list(&mode), list(&told));
            assert_eq!(theirs.focus(), ours.focus(), "{name}");
            assert_eq!(theirs.items().len(), ours.items().len(), "{name}");
            let enabled =
                |menu: &Menu| -> Vec<bool> { menu.items().iter().map(|i| i.enabled).collect() };
            assert_eq!(enabled(theirs), enabled(ours), "{name}");
            // Only the words differ: playing on from either is the same.
            assert_eq!(std::mem::discriminant(&told), std::mem::discriminant(&mode));
            assert_eq!(told.selection(), mode.selection());
        }
        // A mode without a list is told as it is.
        for mode in [Mode::default(), Mode::Objective, skirmish_menu(&s)] {
            assert_eq!(mode.told(&s, words).into_owned(), mode);
        }
    }
}
