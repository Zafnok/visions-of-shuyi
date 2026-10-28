//! The shop screen (ticket 0409, `docs/design/weapons-and-items.md`,
//! *Money and shops*): a player unit on an on-map Armoury, Vendor or
//! Blacksmith picks `Shop` in its action menu and this screen opens over
//! the battle. An Armoury or a Vendor has a Buy and a Sell tab, a
//! Blacksmith a Repair tab. Each line shows its gold; a line the party
//! can't afford (or the core would refuse) is dimmed. The header shows the
//! shop and the party's gold as it would be after what was chosen so far.
//!
//! Nothing changes until the screen closes: every choice is collected
//! into one [`UnitAction::Shop`], which the battle sends then
//! ([`ShopVisit`]). Leaving with nothing chosen sends nothing, so the unit
//! hasn't acted and its action menu is back. What the screen shows after a
//! choice is the battle with those transactions applied to a copy
//! (`preview`); legality is the core's ([`BattleState::check`]).
//!
//! [`ShopScreen::view`] says what it shows; the battle skin paints it
//! (ADR-0054).

use crate::prelude::*;

use trpg_core::{
    BattleState, Command, Gold, ItemDef, ItemId, Pos, SellFrom, Shop, ShopKind, ShopTxn,
    UnitAction, UnitId, WEAPON_SLOTS, repair_cost, sell_price,
};

use super::glyph::shop as glyph;
use super::info::{bonus_text, range_text};
use super::items::effect_text;
use super::view::{ShopLineView, ShopScreenView};
use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::widgets::ListCursor;
use crate::words::Words;

/// Text key of the help line while it is open, for a shop with tabs.
pub const HELP: &str = "battle.shop.help";

/// Text key of the help line for a shop with one tab (a Blacksmith).
pub const HELP_ONE_TAB: &str = "battle.shop.help_one";

/// A tab of the shop screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// What the shop sells.
    Buy,
    /// What the unit and the battle pack could sell.
    Sell,
    /// The unit's worn weapons (a Blacksmith).
    Repair,
}

impl Tab {
    /// The tabs a shop of `kind` has.
    pub fn of(kind: ShopKind) -> &'static [Tab] {
        match kind {
            ShopKind::Armoury | ShopKind::Vendor => &[Tab::Buy, Tab::Sell],
            ShopKind::Blacksmith => &[Tab::Repair],
        }
    }

    /// The key of its name.
    fn key(self) -> &'static str {
        match self {
            Tab::Buy => "battle.shop.buy",
            Tab::Sell => "battle.shop.sell",
            Tab::Repair => "battle.shop.repair",
        }
    }
}

/// The key of the name of a shop of `kind`.
pub fn kind_key(kind: ShopKind) -> &'static str {
    match kind {
        ShopKind::Armoury => "battle.shop.armoury",
        ShopKind::Vendor => "battle.shop.vendor",
        ShopKind::Blacksmith => "battle.shop.blacksmith",
    }
}

/// `1 250`: gold with its thousands apart.
pub fn gold_text(gold: Gold) -> String {
    let digits = gold.to_string();
    let mut out = String::new();
    for (i, d) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(d);
    }
    out
}

/// What the screen hands the battle when it closes: the unit's shop
/// visit, to send as one [`UnitAction::Shop`] unless `txns` is empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopVisit {
    /// The unit.
    pub unit: UnitId,
    /// The shop's tile, where the unit stands.
    pub dest: Pos,
    /// What was chosen, in order.
    pub txns: Vec<ShopTxn>,
}

impl ShopVisit {
    /// The command it sends, if anything was chosen.
    pub fn command(&self) -> Option<Command> {
        (!self.txns.is_empty()).then(|| Command::Act {
            unit: self.unit,
            dest: self.dest,
            action: UnitAction::Shop {
                txns: self.txns.clone(),
            },
        })
    }
}

/// One line of a tab: what choosing it does and for how much.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    /// The transaction.
    pub txn: ShopTxn,
    /// The item.
    pub item: ItemId,
    /// How many of it there are (pack items sold as one line).
    pub count: usize,
    /// Its price, sell price or repair cost.
    pub gold: Gold,
    /// A weapon to repair: its durability left and its full durability.
    pub durability: Option<(u32, u32)>,
    /// Whether it can be chosen now.
    pub enabled: bool,
}

/// What an input did on the shop screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopEffect {
    /// Nothing.
    None,
    /// The highlight or the tab moved.
    Moved,
    /// A line was chosen.
    Chosen,
    /// The highlighted line can't be chosen.
    Denied,
    /// Close the screen.
    Close,
}

/// The open shop screen.
#[derive(Debug, Clone)]
pub struct ShopScreen {
    visit: ShopVisit,
    shop: Shop,
    /// The battle as it was when the screen opened.
    base: BattleState,
    /// The battle after `visit.txns`, on a copy.
    preview: BattleState,
    tab: usize,
    cursor: ListCursor,
}

impl ShopScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "shop";

    /// The shop on `dest` for `unit` of `state`, on its first tab; `None`
    /// if there is no shop there.
    pub fn new(state: &BattleState, unit: UnitId, dest: Pos) -> Option<Self> {
        let shop = state.map().shop(dest)?.clone();
        let mut screen = Self {
            visit: ShopVisit {
                unit,
                dest,
                txns: vec![],
            },
            shop,
            base: state.clone(),
            preview: state.clone(),
            tab: 0,
            cursor: ListCursor::new(0),
        };
        screen.cursor = ListCursor::new(screen.offers().len());
        Some(screen)
    }

    /// The tab shown.
    pub fn tab(&self) -> Tab {
        Tab::of(self.shop.kind)[self.tab]
    }

    /// The highlighted line.
    pub fn focus(&self) -> usize {
        self.cursor.focus()
    }

    /// What was chosen so far.
    pub fn visit(&self) -> &ShopVisit {
        &self.visit
    }

    /// The party's gold after what was chosen.
    pub fn gold(&self) -> Gold {
        self.preview.gold()
    }

    /// The command for `txns` at this shop.
    fn command(&self, txns: Vec<ShopTxn>) -> Command {
        Command::Act {
            unit: self.visit.unit,
            dest: self.visit.dest,
            action: UnitAction::Shop { txns },
        }
    }

    /// Whether the core takes `txn` after what was chosen.
    fn takes(&self, txn: &ShopTxn) -> bool {
        let mut txns = self.visit.txns.clone();
        txns.push(txn.clone());
        self.base.check(&self.command(txns)).is_ok()
    }

    /// The lines of the tab shown, for the battle after what was chosen.
    pub fn offers(&self) -> Vec<Offer> {
        let mut offers = match self.tab() {
            Tab::Buy => self.buys(),
            Tab::Sell => self.sells(),
            Tab::Repair => self.repairs(),
        };
        for offer in &mut offers {
            offer.enabled = self.takes(&offer.txn);
        }
        offers
    }

    /// What the shop sells.
    fn buys(&self) -> Vec<Offer> {
        let items = self.preview.items();
        let stock = self.shop.stock.iter();
        stock
            .filter_map(|item| {
                let def = items.get(item)?;
                Some(offer(
                    ShopTxn::Buy { item: item.clone() },
                    item,
                    def.price(),
                ))
            })
            .collect()
    }

    /// What the unit wears and carries, then the battle pack's items (a
    /// line per item, sold one at a time). Seals are never listed: no
    /// shop buys them.
    fn sells(&self) -> Vec<Offer> {
        let items = self.preview.items();
        let Some(unit) = self.preview.unit(self.visit.unit) else {
            return vec![];
        };
        let loadout = &unit.loadout;
        let weapons = (0..WEAPON_SLOTS)
            .filter_map(|slot| Some((SellFrom::Weapon(slot), loadout.weapon(slot)?.def.clone())));
        let worn = [
            (SellFrom::Armour, loadout.armour.clone()),
            (SellFrom::Accessory, loadout.accessory.clone()),
        ];
        let worn = worn
            .into_iter()
            .filter_map(|(from, item)| Some((from, item?)));
        let mut out: Vec<Offer> = vec![];
        for (from, item) in weapons.chain(worn) {
            if let Some(def) = items.get(&item).filter(|d| !is_seal(d)) {
                out.push(offer(ShopTxn::Sell { from }, &item, sell_price(def)));
            }
        }
        let first = out.len();
        for (index, item) in self.preview.pack().items.iter().enumerate() {
            let Some(def) = items.get(item).filter(|d| !is_seal(d)) else {
                continue;
            };
            if let Some(o) = out[first..].iter_mut().find(|o| &o.item == item) {
                o.count += 1;
            } else {
                let from = SellFrom::Pack(index);
                out.push(offer(ShopTxn::Sell { from }, item, sell_price(def)));
            }
        }
        out
    }

    /// The unit's weapons that lost durability.
    fn repairs(&self) -> Vec<Offer> {
        let items = self.preview.items();
        let Some(unit) = self.preview.unit(self.visit.unit) else {
            return vec![];
        };
        (0..WEAPON_SLOTS)
            .filter_map(|slot| {
                let copy = unit.loadout.weapon(slot)?;
                let def = items.weapon(&copy.def)?;
                (copy.durability_left < def.durability).then(|| Offer {
                    durability: Some((copy.durability_left, def.durability)),
                    ..offer(ShopTxn::Repair { slot }, &copy.def, repair_cost(def, copy))
                })
            })
            .collect()
    }

    /// Handles one action: up and down move the highlight (wrapping), left
    /// and right the tab, Confirm chooses the line, Cancel closes.
    pub fn step(&mut self, action: Action) -> ShopEffect {
        match action {
            Action::Cancel => ShopEffect::Close,
            Action::CursorDown | Action::CursorUp => {
                let down = action == Action::CursorDown;
                if self.cursor.step(down, true, |_| true) {
                    ShopEffect::Moved
                } else {
                    ShopEffect::None
                }
            }
            Action::CursorLeft | Action::CursorRight => {
                let tabs = Tab::of(self.shop.kind).len();
                if tabs < 2 {
                    return ShopEffect::None;
                }
                self.tab = if action == Action::CursorRight {
                    (self.tab + 1) % tabs
                } else {
                    (self.tab + tabs - 1) % tabs
                };
                self.cursor = ListCursor::new(self.offers().len());
                ShopEffect::Moved
            }
            Action::Confirm => self.choose(),
            _ => ShopEffect::None,
        }
    }

    /// Chooses the highlighted line, if the core takes it.
    fn choose(&mut self) -> ShopEffect {
        let Some(offer) = self.offers().into_iter().nth(self.cursor.focus()) else {
            return ShopEffect::Denied;
        };
        if !offer.enabled {
            return ShopEffect::Denied;
        }
        self.visit.txns.push(offer.txn);
        let mut preview = self.base.clone();
        if preview
            .apply(&self.command(self.visit.txns.clone()))
            .is_err()
        {
            // `takes` said yes, so this doesn't happen; undo it if it does.
            self.visit.txns.pop();
            return ShopEffect::Denied;
        }
        self.preview = preview;
        // A line sold or repaired may be gone.
        self.cursor.set_len(self.offers().len());
        ShopEffect::Chosen
    }

    /// The line `offer` shows, in `words`.
    fn line(&self, words: Words<'_>, offer: &Offer) -> ShopLineView {
        let items = self.preview.items();
        let mut name = words.item(&offer.item, items).to_owned();
        if offer.count > 1 {
            name = format!("{name} ×{}", offer.count);
        }
        let detail = match offer.durability {
            Some((left, max)) => format!("{left}/{max} → {max}/{max}"),
            None => items
                .get(&offer.item)
                .map(|def| stat_line(words, def))
                .unwrap_or_default(),
        };
        ShopLineView {
            name,
            gold: gold_text(offer.gold),
            detail,
            enabled: offer.enabled,
        }
    }

    /// The whole screen (ADR-0054), in the player's language and with
    /// their keys.
    pub fn view(&self, ctx: &Ctx) -> ShopScreenView {
        let words = ctx.words();
        let offers = self.offers();
        let tabs = Tab::of(self.shop.kind);
        let unit = self.preview.unit(self.visit.unit);
        ShopScreenView {
            shop: ctx.text(kind_key(self.shop.kind)).to_owned(),
            gold: ctx.text_with(
                "battle.shop.gold",
                &[("gold", &gold_text(self.preview.gold()))],
            ),
            unit: unit.map(|u| words.unit(u).to_owned()).unwrap_or_default(),
            tabs: tabs.iter().map(|t| ctx.text(t.key()).to_owned()).collect(),
            tab: self.tab,
            lines: offers.iter().map(|o| self.line(words, o)).collect(),
            focus: self.cursor.focus(),
            empty: ctx.text("battle.shop.empty").to_owned(),
            help: ctx.text_with(if tabs.len() > 1 { HELP } else { HELP_ONE_TAB }, &[]),
            debug_hint: crate::screens::debug_hint(ctx),
        }
    }
}

/// An offer of `txn` on `item` for `gold`, enabled until checked.
fn offer(txn: ShopTxn, item: &ItemId, gold: Gold) -> Offer {
    Offer {
        txn,
        item: item.clone(),
        count: 1,
        gold,
        durability: None,
        enabled: true,
    }
}

fn is_seal(def: &ItemDef) -> bool {
    matches!(def, ItemDef::Seal(_))
}

/// A line of what `def` is: a weapon's numbers, an armour's or an
/// accessory's bonuses, what a consumable does.
pub fn stat_line(words: Words<'_>, def: &ItemDef) -> String {
    match def {
        ItemDef::Weapon(w) => words.text_with(
            "battle.info.weapon_numbers",
            &[
                ("might", &w.might),
                ("hit", &w.hit),
                ("crit", &w.crit),
                ("range", &range_text(w.min_range, w.max_range)),
                ("weight", &w.weight),
            ],
        ),
        ItemDef::Armour(a) => bonus_text(words, &a.bonus),
        ItemDef::Accessory(a) => bonus_text(words, &a.bonus),
        ItemDef::Consumable(c) => effect_text(words, c.effect),
        ItemDef::Seal(_) => String::new(),
    }
}

impl Screen<Shuyi> for ShopScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    /// Steps the screen for each action, with its menu sound; Cancel pops
    /// back to the battle with the [`ShopVisit`].
    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        for &action in &input.actions {
            match self.step(action) {
                ShopEffect::None => {}
                ShopEffect::Moved => ctx.audio.menu(MenuSound::Move),
                ShopEffect::Chosen => ctx.audio.menu(MenuSound::Select),
                ShopEffect::Denied => ctx.audio.menu(MenuSound::Denied),
                ShopEffect::Close => {
                    ctx.audio.menu(MenuSound::Cancel);
                    return Transition::pop_with(self.visit.clone());
                }
            }
        }
        Transition::None
    }

    /// Paints the whole screen: the battle under it isn't drawn.
    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
