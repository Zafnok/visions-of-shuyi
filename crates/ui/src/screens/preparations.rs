//! The Preparations screen (ticket 0408, `docs/design/weapons-and-items.md`,
//! *Loadout* and *Battle pack*): before a battle, the player changes the
//! army's loadouts and fills the shared pack from the stock, then starts
//! the battle with `Fight!`.
//!
//! The screen edits a [`Preparations`] (the battle's setup and the units
//! left out of it) through the rules in [`trpg_core::prep`] and hands it
//! back ([`PreparationsScreen::prep`]).
//!
//! Controls, by [`Focus`]:
//!
//! - **Tabs** (`Loadouts`, `Pack`, `Fight!`): Left and Right pick one,
//!   Confirm (or Down) opens it; on `Fight!` Confirm starts the battle.
//!   Cancel does nothing unless the caller allows leaving; then it asks
//!   [`LEAVE_QUESTION`].
//! - **Loadouts**: the units (the ones left out of this battle after the
//!   others, marked as such, so their gear can be traded: Nick, 0408), then the
//!   chosen unit's slots, then the stock for that slot; Confirm goes one list deeper, Cancel one back. Confirm
//!   on a stock item puts it in the slot (what was there goes back to the
//!   stock); the list also has what the other units hold in such a slot,
//!   and Confirm on one of those swaps the two units' items; items the unit can't use say why and can't
//!   be taken. The attack speed line shows the change before it is made.
//! - **Pack**: the stock's consumables on the left, the pack on the right;
//!   Left and Right switch sides, Confirm moves one item across. A full
//!   pack takes no more and says so.
//!
//! **The look is a skin** (ADR-0054): this module decides what the screen
//! shows and does, and says it as a [`PreparationsView`]; [`glyph`] paints
//! it.

pub mod glyph;
pub mod view;

use crate::prelude::*;
pub use view::{
    ConsumableRowView, GearView, ItemRowView, ListState, ListView, LoadoutsView, PackView,
    PrepBody, PreparationsView, SlotRowView, TabView, UnitRowView,
};

use core::fmt::Display;

use trpg_core::{
    ArmourWeight, BattleSetup, Equipped, GearSlot, ItemDef, ItemId, PrepError, PrepUnit,
    Preparations, StatValue, StockItem, Unit, Unusable, WeaponKind,
};

use super::battle::info::bonus_text;
use super::battle::items::effect_text;
use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::screen::{Ctx, FrameInput, Screen, ScreenResult, Shuyi, Transition};
use crate::widgets::wrap_step;
use crate::words::Words;

/// Text key of the question Cancel asks on the tabs, when leaving is
/// allowed. Like all the screen's text it is in `assets/lang/en/ui.ron`
/// under `prep.` (ADR-0045).
pub const LEAVE_QUESTION: &str = "prep.leave_question";
/// Text key of the stock row that empties the slot.
pub const PUT_BACK: &str = "prep.put_back";
/// Text key of what is said when a slot is chosen and the stock has
/// nothing for it.
pub const NOTHING_IN_STOCK: &str = "prep.nothing_in_stock";

/// One of the screen's tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// The units' weapons, armour and accessories.
    Loadouts,
    /// The shared consumables.
    Pack,
    /// Opens the Options screen (0805): the one place a Classic campaign
    /// can switch to Casual (`death-and-difficulty.md`).
    Options,
    /// Starts the battle.
    Fight,
}

impl Tab {
    /// Every tab, left to right.
    pub const ALL: [Tab; 4] = [Tab::Loadouts, Tab::Pack, Tab::Options, Tab::Fight];

    /// The text key of the tab's name.
    pub const fn key(self) -> &'static str {
        match self {
            Tab::Loadouts => "prep.tab.loadouts",
            Tab::Pack => "prep.tab.pack",
            Tab::Options => "prep.tab.options",
            Tab::Fight => "prep.tab.fight",
        }
    }
}

/// Which list the cursor is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    /// The tabs.
    Tabs,
    /// Loadouts: the units.
    Units,
    /// Loadouts: the chosen unit's slots.
    Slots,
    /// Loadouts: the stock for the chosen slot.
    Stock,
    /// Pack: the stock's consumables.
    Spare,
    /// Pack: the pack.
    Pack,
}

/// Where a row of the stock list takes its item from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// The row that empties the slot: its item goes back to the stock.
    PutBack,
    /// The stock.
    Stock(StockItem),
    /// This slot of another unit: the two units swap.
    Unit(PrepUnit, GearSlot),
}

/// One row of the stock list beside the slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockRow {
    /// What Confirm takes.
    pub from: Source,
    /// The item's name (or the row that empties the slot).
    pub name: String,
    /// Its numbers and where it comes from, or why the unit can't use it.
    pub detail: String,
    /// Why the unit can't use it, if it can't.
    pub unusable: Option<Unusable>,
}

/// Why a unit can't use an item, as the player reads it: `needs rank D`.
pub fn reason_text(ctx: &Ctx, why: Unusable) -> String {
    match why {
        Unusable::Kind(kind) => ctx.text(kind_key(kind)).to_owned(),
        Unusable::Rank(rank) => {
            let rank = format!("{rank:?}");
            ctx.text_with("prep.reason.rank", &[("rank", &rank)])
        }
        Unusable::Armour(weight) => ctx.text(weight_key(weight)).to_owned(),
    }
}

const fn kind_key(kind: WeaponKind) -> &'static str {
    match kind {
        WeaponKind::Sword => "prep.reason.kind.sword",
        WeaponKind::Spear => "prep.reason.kind.spear",
        WeaponKind::Axe => "prep.reason.kind.axe",
        WeaponKind::Bow => "prep.reason.kind.bow",
        WeaponKind::Gauntlet => "prep.reason.kind.gauntlet",
    }
}

const fn weight_key(weight: ArmourWeight) -> &'static str {
    match weight {
        ArmourWeight::Light => "prep.reason.armour.light",
        ArmourWeight::Medium => "prep.reason.armour.medium",
        ArmourWeight::Heavy => "prep.reason.armour.heavy",
    }
}

/// Before a battle: loadouts and the pack. See the module docs. `Fight!`
/// pops with the [`Preparations`] made; leaving pops with nothing.
#[derive(Debug, Clone)]
pub struct PreparationsScreen {
    prep: Preparations,
    /// Whether Cancel on the tabs may leave.
    can_leave: bool,
    tab: Tab,
    focus: Focus,
    /// The chosen unit, among the army's ([`Preparations::units`]).
    unit: usize,
    /// The chosen slot, among the unit's slots.
    slot: usize,
    /// The row of the stock list beside the slots.
    stock: usize,
    /// The row of the stock's consumables.
    spare: usize,
    /// The row of the pack.
    pack: usize,
    /// Why the last Confirm did nothing.
    message: Option<String>,
    /// Whether [`LEAVE_QUESTION`] is open.
    leaving: bool,
    /// The `Options` tab was opened: the frame opens the Options screen.
    open_options: bool,
}

impl PreparationsScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "preparations";

    /// The screen for `prep`, on the `Loadouts` tab. `can_leave`: Cancel
    /// on the tabs offers to leave.
    pub fn new(prep: Preparations, can_leave: bool) -> Self {
        Self {
            prep,
            can_leave,
            tab: Tab::Loadouts,
            focus: Focus::Tabs,
            unit: 0,
            slot: 0,
            stock: 0,
            spare: 0,
            pack: 0,
            message: None,
            leaving: false,
            open_options: false,
        }
    }

    /// The battle as prepared so far.
    pub fn setup(&self) -> &BattleSetup {
        &self.prep.setup
    }

    /// The campaign switched to Casual (on the Options screen opened from
    /// here): the battle being prepared starts in Casual.
    pub fn switch_to_casual(&mut self) {
        self.prep.setup.mode = trpg_core::GameMode::Casual;
    }

    /// The battle and the units left out of it, as prepared so far.
    pub fn prep(&self) -> &Preparations {
        &self.prep
    }

    /// The tab shown.
    pub fn tab(&self) -> Tab {
        self.tab
    }

    /// Where the cursor is.
    pub fn focus(&self) -> Focus {
        self.focus
    }

    /// The message on screen, if any.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Whether [`LEAVE_QUESTION`] is open.
    pub fn is_asking_leave(&self) -> bool {
        self.leaving
    }

    /// The army's units: the deployed ones, then the ones left out.
    fn units(&self) -> Vec<(PrepUnit, &Unit)> {
        self.prep.units()
    }

    /// The chosen unit.
    fn unit(&self) -> Option<&Unit> {
        self.who().and_then(|who| self.prep.unit(who))
    }

    /// Which unit is chosen.
    fn who(&self) -> Option<PrepUnit> {
        self.units().get(self.unit).map(|(who, _)| *who)
    }

    /// The chosen unit's slots.
    fn slots(&self) -> Vec<GearSlot> {
        self.who()
            .map_or_else(Vec::new, |who| self.prep.gear_slots(who))
    }

    /// The chosen slot.
    fn gear_slot(&self) -> Option<GearSlot> {
        self.slots().get(self.slot).copied()
    }

    /// The name of item `id`.
    fn item_name(&self, words: Words<'_>, id: &ItemId) -> String {
        words.item(id, &self.setup().items).to_owned()
    }

    /// What `slot` of `unit` holds: its name and, for a weapon, its
    /// durability (`20/25`).
    fn held(&self, words: Words<'_>, unit: &Unit, slot: GearSlot) -> Option<(String, String)> {
        match slot {
            GearSlot::Weapon(s) => {
                let copy = unit.loadout.weapon(s)?;
                let max = self
                    .setup()
                    .items
                    .weapon(&copy.def)
                    .map_or(0, |w| w.durability);
                Some((
                    self.item_name(words, &copy.def),
                    format!("{:>2}/{max}", copy.durability_left),
                ))
            }
            GearSlot::Armour => {
                let id = unit.loadout.armour.as_ref()?;
                Some((self.item_name(words, id), String::new()))
            }
            GearSlot::Accessory => {
                let id = unit.loadout.accessory.as_ref()?;
                Some((self.item_name(words, id), String::new()))
            }
        }
    }

    /// The stock rows for the chosen unit's chosen slot: the row that
    /// empties the slot (if it holds something), then every stock item of
    /// the slot's kind, the ones the unit can't use with the reason.
    pub fn stock_rows(&self, ctx: &Ctx) -> Vec<StockRow> {
        let (Some(unit), Some(slot)) = (self.unit(), self.gear_slot()) else {
            return Vec::new();
        };
        let items = &self.prep.setup.items;
        let mut rows = Vec::new();
        if self.held(ctx.words(), unit, slot).is_some() {
            rows.push(StockRow {
                from: Source::PutBack,
                name: ctx.text(PUT_BACK).to_owned(),
                detail: String::new(),
                unusable: None,
            });
        }
        let row = |item: StockItem, id: &ItemId, name: String, detail: String| {
            let unusable = self.who().and_then(|who| self.prep.unusable(who, id));
            let detail = unusable.map_or(detail, |why| reason_text(ctx, why));
            StockRow {
                from: Source::Stock(item),
                name,
                detail,
                unusable,
            }
        };
        match slot {
            GearSlot::Weapon(_) => {
                for (i, copy) in self.prep.setup.stock.weapons.iter().enumerate() {
                    let Some(def) = items.weapon(&copy.def) else {
                        continue;
                    };
                    let range = if def.min_range == def.max_range {
                        def.min_range.to_string()
                    } else {
                        format!("{}-{}", def.min_range, def.max_range)
                    };
                    let (might, hit, weight) = (
                        format!("{:>2}", def.might),
                        format!("{:>3}", def.hit),
                        format!("{:>2}", def.weight),
                    );
                    let stats = ctx.text_with(
                        "prep.weapon_detail",
                        &[
                            ("might", &might),
                            ("hit", &hit),
                            ("weight", &weight),
                            ("range", &range),
                        ],
                    );
                    let detail =
                        format!("{stats:<21} {:>2}/{}", copy.durability_left, def.durability);
                    rows.push(row(
                        StockItem::Weapon(i),
                        &copy.def,
                        ctx.words().item(&copy.def, items).to_owned(),
                        detail,
                    ));
                }
            }
            GearSlot::Armour | GearSlot::Accessory => {
                for (id, count) in &self.prep.setup.stock.items {
                    let detail = match (slot, items.get(id)) {
                        (GearSlot::Armour, Some(ItemDef::Armour(a))) => {
                            let bonus = bonus_text(ctx.words(), &a.bonus);
                            let detail = [("bonus", &bonus as &dyn Display), ("weight", &a.weight)];
                            ctx.text_with("prep.armour_detail", &detail)
                        }
                        (GearSlot::Accessory, Some(ItemDef::Accessory(a))) => {
                            bonus_text(ctx.words(), &a.bonus)
                        }
                        _ => continue,
                    };
                    let name = self.item_name(ctx.words(), id);
                    rows.push(row(
                        StockItem::Item(id.clone()),
                        id,
                        name,
                        format!("×{count}  {detail}"),
                    ));
                }
            }
        }
        rows.extend(self.unit_rows(ctx, slot));
        rows
    }

    /// The rows for what the other units hold in slots of `slot`'s kind
    /// that the chosen unit can use, in unit then slot order:
    /// `Steel Bow  25/25  from Test Scout`.
    fn unit_rows(&self, ctx: &Ctx, slot: GearSlot) -> Vec<StockRow> {
        let Some(who) = self.who() else {
            return Vec::new();
        };
        let same_kind =
            |other: GearSlot| core::mem::discriminant(&other) == core::mem::discriminant(&slot);
        let mut rows = Vec::new();
        for (holder, unit) in self.units() {
            if holder == who {
                continue;
            }
            let slots = self.prep.gear_slots(holder);
            for theirs in slots.into_iter().filter(|s| same_kind(*s)) {
                let id = match theirs {
                    GearSlot::Weapon(s) => unit.loadout.weapon(s).map(|w| &w.def),
                    GearSlot::Armour => unit.loadout.armour.as_ref(),
                    GearSlot::Accessory => unit.loadout.accessory.as_ref(),
                };
                let held = self.held(ctx.words(), unit, theirs);
                let (Some(id), Some((name, wear))) = (id, held) else {
                    continue;
                };
                // Only what this unit could take: the stock's own rows
                // say what it can't use, and an army's worth more would
                // bury the list.
                if self.prep.unusable(who, id).is_some() {
                    continue;
                }
                // Nor the same armour or accessory the slot already holds.
                let mine = match slot {
                    GearSlot::Weapon(_) => None,
                    GearSlot::Armour => self.unit().and_then(|u| u.loadout.armour.as_ref()),
                    GearSlot::Accessory => self.unit().and_then(|u| u.loadout.accessory.as_ref()),
                };
                if mine == Some(id) {
                    continue;
                }
                let from = ctx.text_with("prep.from", &[("unit", &ctx.words().unit(unit))]);
                let detail = if wear.is_empty() {
                    from
                } else {
                    format!("{wear}  {from}")
                };
                rows.push(StockRow {
                    from: Source::Unit(holder, theirs),
                    name,
                    detail,
                    unusable: None,
                });
            }
        }
        rows
    }

    /// The stock's consumables with how many there are, in id order.
    pub fn spare(&self) -> Vec<(ItemId, u32)> {
        let items = &self.prep.setup.items;
        let stock = self.prep.setup.stock.items.iter();
        stock
            .filter(|(id, _)| items.consumable(id).is_some())
            .map(|(id, n)| (id.clone(), *n))
            .collect()
    }

    /// The pack grouped by item with how many of each, in the order each
    /// was first packed.
    pub fn packed(&self) -> Vec<(ItemId, usize)> {
        let mut groups: Vec<(ItemId, usize)> = Vec::new();
        for id in &self.prep.setup.pack.items {
            match groups.iter_mut().find(|(g, _)| g == id) {
                Some((_, n)) => *n += 1,
                None => groups.push((id.clone(), 1)),
            }
        }
        groups
    }

    /// `Pack 3/6`.
    pub fn pack_header(&self, ctx: &Ctx) -> String {
        let pack = &self.prep.setup.pack;
        let args: [(&str, &dyn Display); 2] = [("packed", &pack.items.len()), ("cap", &pack.cap)];
        ctx.text_with("prep.pack_header", &args)
    }

    /// The army after Confirm on stock row `row`, if that changes it.
    fn after(&self, row: &StockRow) -> Option<Preparations> {
        let (unit, slot) = (self.who()?, self.gear_slot()?);
        let mut after = self.prep.clone();
        let done = match &row.from {
            Source::Stock(item) => after.gear_from_stock(unit, slot, item),
            Source::PutBack => after.gear_to_stock(unit, slot),
            Source::Unit(from, theirs) => after.gear_from_unit(unit, slot, *from, *theirs),
        };
        done.ok().map(|()| after)
    }

    /// The weapon slot whose attack speed is shown: the chosen slot if the
    /// cursor is on a weapon slot (or in its stock list), else `None` (the
    /// equipped weapon).
    fn speed_slot(&self) -> Option<usize> {
        match (self.focus, self.gear_slot()) {
            (Focus::Slots | Focus::Stock, Some(GearSlot::Weapon(s))) => Some(s),
            _ => None,
        }
    }

    /// The name of the weapon `unit` attacks with from `slot` (`None`: its
    /// equipped weapon or spell); `no weapon` without one.
    fn in_hand(ctx: &Ctx, setup: &BattleSetup, unit: &Unit, slot: Option<usize>) -> String {
        let words = ctx.words();
        let equipped = slot
            .map(Equipped::Weapon)
            .or_else(|| unit.loadout.equipped.clone());
        let name = match equipped {
            Some(Equipped::Weapon(s)) => unit
                .loadout
                .weapon(s)
                .filter(|w| setup.items.weapon(&w.def).is_some())
                .map(|w| words.item(&w.def, &setup.items).to_owned()),
            Some(Equipped::Spell(spell)) => {
                let spell = setup.spells.get(&spell);
                spell.map(|s| words.spell(s).to_owned())
            }
            None => None,
        };
        name.unwrap_or_else(|| ctx.text("prep.no_weapon").to_owned())
    }

    /// The chosen unit's attack speed line: `AS 6 with Iron Sword`, and
    /// while a stock row is highlighted the change taking it would make,
    /// `AS 6 → 4 with Steel Sword`. The speed is with the weapon of the
    /// highlighted weapon slot in hand, else with the equipped weapon.
    pub fn speed_line(&self, ctx: &Ctx) -> Option<String> {
        let (who, unit) = (self.who()?, self.unit()?);
        let slot = self.speed_slot();
        let now: StatValue = self.prep.attack_speed(who, slot)?;
        let highlighted = self.stock_rows(ctx).into_iter().nth(self.stock);
        let after = (self.focus == Focus::Stock)
            .then_some(highlighted)
            .flatten()
            .and_then(|row| self.after(&row));
        let Some(after) = after else {
            let with = Self::in_hand(ctx, &self.prep.setup, unit, slot);
            let args: [(&str, &dyn Display); 2] = [("now", &now), ("weapon", &with)];
            return Some(ctx.text_with("prep.speed", &args));
        };
        let then = after.attack_speed(who, slot).unwrap_or(now);
        let with = after
            .unit(who)
            .map(|u| Self::in_hand(ctx, &after.setup, u, slot))
            .unwrap_or_default();
        let args: [(&str, &dyn Display); 3] = [("now", &now), ("then", &then), ("weapon", &with)];
        Some(ctx.text_with("prep.speed_change", &args))
    }

    /// The bottom help line.
    pub fn help(&self, ctx: &Ctx) -> String {
        let key = match (self.leaving, self.focus, self.tab, self.can_leave) {
            (true, ..) => "prep.help.leaving",
            (_, Focus::Tabs, Tab::Fight, true) => "prep.help.fight_leave",
            (_, Focus::Tabs, Tab::Fight, false) => "prep.help.fight",
            (_, Focus::Tabs, _, true) => "prep.help.tabs_leave",
            (_, Focus::Tabs, _, false) => "prep.help.tabs",
            (_, Focus::Units, ..) => "prep.help.units",
            (_, Focus::Slots, ..) => "prep.help.slots",
            (_, Focus::Stock, ..) => "prep.help.stock",
            (_, Focus::Spare, ..) => "prep.help.spare",
            (_, Focus::Pack, ..) => "prep.help.pack",
        };
        ctx.text_with(key, &[])
    }

    /// Opens the shown tab (Confirm or Down on the tabs). Returns whether
    /// the screen is done (`Fight!`).
    fn open_tab(&mut self, ctx: &mut Ctx) -> Option<Transition> {
        let focus = match self.tab {
            Tab::Loadouts => (!self.units().is_empty()).then_some(Focus::Units),
            Tab::Pack if !self.spare().is_empty() => Some(Focus::Spare),
            Tab::Pack => (!self.packed().is_empty()).then_some(Focus::Pack),
            Tab::Options => {
                ctx.audio.menu(MenuSound::Select);
                self.open_options = true;
                return None;
            }
            Tab::Fight => {
                ctx.audio.menu(MenuSound::Select);
                return Some(Transition::pop_with(self.prep.clone()));
            }
        };
        match focus {
            Some(focus) => {
                self.focus = focus;
                ctx.audio.menu(MenuSound::Select);
            }
            None => ctx.audio.menu(MenuSound::Denied),
        }
        None
    }

    /// Moves a list cursor from `i` in a list of `len`, with the move sound
    /// if it moved.
    fn moved(i: usize, len: usize, down: bool, ctx: &mut Ctx) -> usize {
        let next = wrap_step(i, len, down);
        if next != i {
            ctx.audio.menu(MenuSound::Move);
        }
        next
    }

    /// Goes back to `focus` with the cancel sound.
    fn back(&mut self, focus: Focus, ctx: &mut Ctx) {
        self.focus = focus;
        ctx.audio.menu(MenuSound::Cancel);
    }

    /// Confirm on the highlighted stock row: takes it (or empties the slot)
    /// and goes back to the slots; an item the unit can't use is refused.
    fn take(&mut self, ctx: &mut Ctx) {
        let Some(row) = self.stock_rows(ctx).into_iter().nth(self.stock) else {
            return;
        };
        if let Some(after) = self.after(&row) {
            self.prep = after;
            self.focus = Focus::Slots;
            self.stock = 0;
            ctx.audio.menu(MenuSound::Select);
            return;
        }
        ctx.audio.menu(MenuSound::Denied);
        if let (Some(why), Some(unit)) = (row.unusable, self.unit()) {
            let reason = reason_text(ctx, why);
            let name = ctx.words().unit(unit);
            let args: [(&str, &dyn Display); 2] = [("unit", &name), ("reason", &reason)];
            self.message = Some(ctx.text_with("prep.cant", &args));
        }
    }

    /// Confirm on the highlighted consumable of the stock: packs one.
    fn pack_one(&mut self, ctx: &mut Ctx) {
        let Some((item, _)) = self.spare().into_iter().nth(self.spare) else {
            return;
        };
        match self.prep.setup.pack_from_stock(&item) {
            Ok(()) => ctx.audio.menu(MenuSound::Select),
            Err(e) => {
                ctx.audio.menu(MenuSound::Denied);
                if e == PrepError::PackFull {
                    let pack = &self.prep.setup.pack;
                    let args: [(&str, &dyn Display); 2] =
                        [("packed", &pack.items.len()), ("cap", &pack.cap)];
                    self.message = Some(ctx.text_with("prep.pack_full", &args));
                }
            }
        }
        let left = self.spare().len();
        self.spare = self.spare.min(left.saturating_sub(1));
        if left == 0 {
            self.focus = Focus::Pack;
        }
    }

    /// Confirm on the highlighted pack item: puts one back in the stock.
    fn unpack_one(&mut self, ctx: &mut Ctx) {
        let Some((item, _)) = self.packed().into_iter().nth(self.pack) else {
            return;
        };
        if self.prep.setup.pack_to_stock(&item).is_ok() {
            ctx.audio.menu(MenuSound::Select);
        }
        let left = self.packed().len();
        self.pack = self.pack.min(left.saturating_sub(1));
        if left == 0 {
            self.focus = Focus::Spare;
        }
    }

    /// Left or Right on the Pack tab: to the other list, if it has rows.
    fn switch_side(&mut self, ctx: &mut Ctx) {
        let (other, rows) = match self.focus {
            Focus::Spare => (Focus::Pack, self.packed().len()),
            _ => (Focus::Spare, self.spare().len()),
        };
        if rows > 0 {
            self.focus = other;
            ctx.audio.menu(MenuSound::Move);
        }
    }

    /// Handles one action. Returns the pop, if the screen is done.
    fn handle(&mut self, action: Action, ctx: &mut Ctx) -> Option<Transition> {
        use Action::{Cancel, Confirm, CursorDown, CursorLeft, CursorRight, CursorUp};
        if self.leaving {
            match action {
                Confirm => {
                    ctx.audio.menu(MenuSound::Select);
                    return Some(Transition::Pop);
                }
                Cancel => {
                    self.leaving = false;
                    ctx.audio.menu(MenuSound::Cancel);
                }
                _ => {}
            }
            return None;
        }
        // The Playdate's system menu (ticket 1035): as the Options tab.
        if action == Action::Options {
            ctx.audio.menu(MenuSound::Select);
            self.open_options = true;
            return None;
        }
        if matches!(
            action,
            Cancel | Confirm | CursorDown | CursorLeft | CursorRight | CursorUp
        ) {
            self.message = None;
        }
        let down = action == CursorDown;
        match (self.focus, action) {
            (Focus::Tabs, CursorLeft | CursorRight) => {
                let n = Tab::ALL.len();
                let i = Tab::ALL.iter().position(|&t| t == self.tab).unwrap_or(0);
                self.tab = Tab::ALL[wrap_step(i, n, action == CursorRight)];
                ctx.audio.menu(MenuSound::Move);
            }
            (Focus::Tabs, Confirm) => return self.open_tab(ctx),
            (Focus::Tabs, CursorDown) if !matches!(self.tab, Tab::Fight | Tab::Options) => {
                return self.open_tab(ctx);
            }
            (Focus::Tabs, Cancel) if self.can_leave => {
                self.leaving = true;
                ctx.audio.menu(MenuSound::Select);
            }

            (Focus::Units, CursorUp | CursorDown) => {
                self.unit = Self::moved(self.unit, self.units().len(), down, ctx);
                self.slot = 0;
            }
            (Focus::Units, Confirm) => {
                self.focus = Focus::Slots;
                self.slot = 0;
                ctx.audio.menu(MenuSound::Select);
            }
            (Focus::Units | Focus::Spare | Focus::Pack, Cancel) => self.back(Focus::Tabs, ctx),

            (Focus::Slots, CursorUp | CursorDown) => {
                self.slot = Self::moved(self.slot, self.slots().len(), down, ctx);
            }
            (Focus::Slots, Confirm) => {
                if self.stock_rows(ctx).is_empty() {
                    self.message = Some(ctx.text(NOTHING_IN_STOCK).to_owned());
                    ctx.audio.menu(MenuSound::Denied);
                } else {
                    self.focus = Focus::Stock;
                    self.stock = 0;
                    ctx.audio.menu(MenuSound::Select);
                }
            }
            (Focus::Slots, Cancel) => self.back(Focus::Units, ctx),

            (Focus::Stock, CursorUp | CursorDown) => {
                let rows = self.stock_rows(ctx).len();
                self.stock = Self::moved(self.stock, rows, down, ctx);
            }
            (Focus::Stock, Confirm) => self.take(ctx),
            (Focus::Stock, Cancel) => self.back(Focus::Slots, ctx),

            (Focus::Spare, CursorUp | CursorDown) => {
                self.spare = Self::moved(self.spare, self.spare().len(), down, ctx);
            }
            (Focus::Spare, Confirm) => self.pack_one(ctx),
            (Focus::Pack, CursorUp | CursorDown) => {
                self.pack = Self::moved(self.pack, self.packed().len(), down, ctx);
            }
            (Focus::Pack, Confirm) => self.unpack_one(ctx),
            (Focus::Spare | Focus::Pack, CursorLeft | CursorRight) => self.switch_side(ctx),
            _ => {}
        }
        None
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint`]): the tabs, the shown tab's lists with their
    /// cursors, the message or the leave question, and the help line.
    pub fn view(&self, ctx: &Ctx) -> PreparationsView {
        let tabs = Tab::ALL.into_iter().map(|tab| TabView {
            tab,
            label: ctx.text(tab.key()).to_owned(),
            shown: tab == self.tab,
        });
        let body = match self.tab {
            Tab::Loadouts => PrepBody::Loadouts(self.loadouts_view(ctx)),
            Tab::Pack => PrepBody::Pack(self.pack_view(ctx)),
            Tab::Options => PrepBody::Options,
            Tab::Fight => {
                let units = self.prep.setup.player_units().count();
                let pack = self.pack_header(ctx);
                let args: [(&str, &dyn Display); 2] = [("units", &units), ("pack", &pack)];
                PrepBody::Fight(ctx.text_with("prep.ready", &args))
            }
        };
        PreparationsView {
            heading: ctx.text("prep.heading").to_owned(),
            tabs: tabs.collect(),
            on_tabs: self.focus == Focus::Tabs,
            body,
            message: self.message.clone(),
            leave_question: self.leaving.then(|| ctx.text(LEAVE_QUESTION).to_owned()),
            help: self.help(ctx),
        }
    }

    /// What the cursor is doing in the list that has it at `focus`; its
    /// row is the chosen one while the cursor is in a list `after` it.
    fn list_state(&self, focus: Focus, after: &[Focus]) -> ListState {
        if self.focus == focus {
            ListState::Focused
        } else if after.contains(&self.focus) {
            ListState::Chosen
        } else {
            ListState::Idle
        }
    }

    /// The Loadouts tab: units, the chosen unit's slots and attack speed,
    /// and the stock for the chosen slot.
    fn loadouts_view(&self, ctx: &Ctx) -> LoadoutsView {
        let rows = self.units().into_iter().map(|(who, u)| UnitRowView {
            name: ctx.words().unit(u).to_owned(),
            deployed: matches!(who, PrepUnit::Deployed(_)),
        });
        let units = ListView {
            title: ctx.text("prep.units").to_owned(),
            rows: rows.collect(),
            cursor: self.unit,
            state: self.list_state(Focus::Units, &[Focus::Slots, Focus::Stock]),
        };
        let gear = self.unit().map(|unit| self.gear_view(ctx, unit));
        LoadoutsView { units, gear }
    }

    /// The chosen `unit`'s slots, attack speed and stock list.
    fn gear_view(&self, ctx: &Ctx, unit: &Unit) -> GearView {
        let words = ctx.words();
        let class = words.class_of(&unit.class, &self.prep.setup.classes);
        let rows = self.slots().into_iter().map(|slot| {
            let label = ctx.text(match slot {
                GearSlot::Weapon(_) => "prep.slot.weapon",
                GearSlot::Armour => "prep.slot.armour",
                GearSlot::Accessory => "prep.slot.accessory",
            });
            let (item, wear) = match self.held(words, unit, slot) {
                Some((name, wear)) => (Some(name), wear),
                None => (None, String::new()),
            };
            SlotRowView {
                label: label.to_owned(),
                item,
                wear,
            }
        });
        let slots = ListView {
            title: format!("{} · {class}", words.unit(unit)),
            rows: rows.collect(),
            cursor: self.slot,
            state: self.list_state(Focus::Slots, &[Focus::Stock]),
        };
        let speed = self.speed_line(ctx);
        let benched = matches!(self.who(), Some(PrepUnit::Benched(_)));
        let benched_note =
            (speed.is_some() && benched).then(|| ctx.text("prep.not_in_battle").to_owned());
        let title = ctx.text(match self.gear_slot() {
            Some(GearSlot::Weapon(_)) => "prep.stock.weapons",
            Some(GearSlot::Armour) => "prep.stock.armour",
            Some(GearSlot::Accessory) | None => "prep.stock.accessories",
        });
        let rows = self.stock_rows(ctx).into_iter().map(|r| ItemRowView {
            // check-text: not a data name (the row's own)
            name: r.name,
            detail: r.detail,
            usable: r.unusable.is_none(),
        });
        let stock = ListView {
            title: title.to_owned(),
            rows: rows.collect(),
            cursor: self.stock,
            state: self.list_state(Focus::Stock, &[]),
        };
        GearView {
            slots,
            speed,
            benched_note,
            stock,
        }
    }

    /// `count` of consumable `id`, with what it does.
    fn consumable_row(&self, words: Words<'_>, id: &ItemId, count: usize) -> ConsumableRowView {
        let effect = self.prep.setup.items.consumable(id).map(|c| c.effect);
        ConsumableRowView {
            name: self.item_name(words, id),
            count,
            effect: effect.map(|e| effect_text(words, e)).unwrap_or_default(),
        }
    }

    /// The Pack tab: the stock's consumables and the pack.
    fn pack_view(&self, ctx: &Ctx) -> PackView {
        let words = ctx.words();
        let spare = self.spare().into_iter().map(|(id, n)| {
            let n = usize::try_from(n).unwrap_or(usize::MAX);
            self.consumable_row(words, &id, n)
        });
        let packed = self.packed().into_iter();
        let packed = packed.map(|(id, n)| self.consumable_row(words, &id, n));
        PackView {
            spare: ListView {
                title: ctx.text("prep.stock").to_owned(),
                rows: spare.collect(),
                cursor: self.spare,
                state: self.list_state(Focus::Spare, &[]),
            },
            pack: ListView {
                title: self.pack_header(ctx),
                rows: packed.collect(),
                cursor: self.pack,
                state: self.list_state(Focus::Pack, &[]),
            },
        }
    }
}

impl Screen<Shuyi> for PreparationsScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        for &action in &input.actions {
            if let Some(pop) = self.handle(action, ctx) {
                return pop;
            }
            if core::mem::take(&mut self.open_options) {
                return Transition::Push(Box::new(super::options::screen()));
            }
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }

    /// Back from the Options screen, where the campaign may have switched
    /// to Casual (0805): the battle being prepared does too.
    fn uncovered(&mut self, ctx: &mut Ctx, _result: Option<ScreenResult>) -> Transition {
        if ctx.game.campaign_mode == Some(trpg_core::GameMode::Casual) {
            self.switch_to_casual();
        }
        Transition::None
    }
}

#[cfg(test)]
mod tests;
