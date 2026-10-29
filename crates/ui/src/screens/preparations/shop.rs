//! The Preparations screen's `Shop` tab (ticket 0443,
//! `docs/design/weapons-and-items.md`, *The Preparations shop*): the
//! battle's basic shop, with Buy, Sell and Repair as the on-map shop
//! screen has them (0409) and its lines, header and painter
//! ([`crate::screens::battle::shop`]). Buy puts the item in the stock,
//! Sell takes one from the stock (what is packed or worn is put back
//! first), Repair fills any weapon of the stock or of the army's loadouts;
//! all with the setup's gold, through [`trpg_core::Preparations`]. Unlike
//! the on-map shop, each choice is made at once: Preparations has no
//! battle to send it to.

use crate::prelude::*;

use trpg_core::{
    Gold, ItemDef, ItemId, PrepUnit, ShopError, StockItem, WEAPON_SLOTS, WeaponAt, repair_cost,
    sell_price,
};

use super::{Focus, PreparationsScreen};
use crate::audio::MenuSound;
use crate::input::Action;
use crate::screen::Ctx;
use crate::screens::battle::shop::{SellConfirm, Tab as ShopTab, gold_text, kind_key, stat_line};
use crate::screens::battle::view::{SellConfirmView, ShopLineView, ShopScreenView};
use crate::widgets::wrap_step;
use crate::words::Words;

/// Text key of the help line in the shop's lists.
pub const HELP: &str = "prep.help.shop";
/// Text key of the help line while a sale is asked about.
pub const HELP_SELL: &str = "battle.shop.help_sell";
/// Text key of what the tab says when the battle has no shop.
pub const NO_SHOP: &str = "prep.shop.none";

/// What a line of the Shop tab does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trade {
    /// Buys one of this item into the stock.
    Buy(ItemId),
    /// Sells this from the stock.
    Sell(StockItem),
    /// Repairs this weapon.
    Repair(WeaponAt),
}

/// One line of the Shop tab: what it does and for how much.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deal {
    /// What Confirm does.
    pub trade: Trade,
    /// The item.
    pub item: ItemId,
    /// How many of it the stock holds (a Sell line).
    pub count: usize,
    /// Its price, sell price or repair cost.
    pub gold: Gold,
    /// A weapon's durability left and its full durability.
    pub durability: Option<(u32, u32)>,
    /// Whose weapon a Repair line mends; `None` for the stock's.
    pub holder: Option<PrepUnit>,
    /// Whether the party can pay for it.
    pub enabled: bool,
}

impl Deal {
    fn new(trade: Trade, item: &ItemId, gold: Gold) -> Self {
        Self {
            trade,
            item: item.clone(),
            count: 1,
            gold,
            durability: None,
            holder: None,
            enabled: true,
        }
    }
}

impl PreparationsScreen {
    /// The shop's tabs (Buy, Sell, Repair).
    fn shop_tabs() -> &'static [ShopTab] {
        ShopTab::of(trpg_core::ShopKind::Basic)
    }

    /// The shop tab shown.
    pub fn shop_tab(&self) -> ShopTab {
        Self::shop_tabs()[self.shop_tab]
    }

    /// The highlighted line of the shop.
    pub fn deal_focus(&self) -> usize {
        self.deal
    }

    /// The sale being asked about, if any.
    pub fn selling(&self) -> Option<SellConfirm> {
        self.selling
    }

    /// The lines of the shop tab shown, for the army as it is now; none
    /// without a shop.
    pub fn deals(&self) -> Vec<Deal> {
        let Some(shop) = &self.prep.shop else {
            return Vec::new();
        };
        let setup = &self.prep.setup;
        let items = &setup.items;
        let mut deals = match self.shop_tab() {
            ShopTab::Buy => shop
                .stock
                .iter()
                .filter_map(|id| {
                    Some(Deal::new(
                        Trade::Buy(id.clone()),
                        id,
                        items.get(id)?.price(),
                    ))
                })
                .collect(),
            ShopTab::Sell => self.sales(),
            ShopTab::Repair => self.repairs(),
        };
        if self.shop_tab() != ShopTab::Sell {
            for deal in &mut deals {
                deal.enabled = deal.gold <= setup.gold;
            }
        }
        deals
    }

    /// The stock's weapons, each copy a line, then its other items with
    /// how many there are. Seals are never listed: no shop buys them.
    fn sales(&self) -> Vec<Deal> {
        let setup = &self.prep.setup;
        let items = &setup.items;
        let weapons = setup.stock.weapons.iter().enumerate();
        let mut deals: Vec<Deal> = weapons
            .filter_map(|(i, copy)| {
                let def = items.weapon(&copy.def)?;
                let gold = sell_price(items.get(&copy.def)?);
                Some(Deal {
                    durability: Some((copy.durability_left, def.durability)),
                    ..Deal::new(Trade::Sell(StockItem::Weapon(i)), &copy.def, gold)
                })
            })
            .collect();
        for (id, count) in &setup.stock.items {
            let Some(def) = items.get(id).filter(|d| !matches!(d, ItemDef::Seal(_))) else {
                continue;
            };
            deals.push(Deal {
                count: usize::try_from(*count).unwrap_or(usize::MAX),
                ..Deal::new(
                    Trade::Sell(StockItem::Item(id.clone())),
                    id,
                    sell_price(def),
                )
            });
        }
        deals
    }

    /// Every weapon that lost durability: the stock's, then each unit's
    /// of the army, in slot order.
    fn repairs(&self) -> Vec<Deal> {
        let items = &self.prep.setup.items;
        let stock = (0..self.prep.setup.stock.weapons.len()).map(WeaponAt::Stock);
        let held = self
            .prep
            .units()
            .into_iter()
            .flat_map(|(who, _)| (0..WEAPON_SLOTS).map(move |slot| WeaponAt::Held(who, slot)));
        stock
            .chain(held)
            .filter_map(|at| {
                let copy = self.prep.weapon(at)?;
                let def = items.weapon(&copy.def)?;
                (copy.durability_left < def.durability).then(|| Deal {
                    durability: Some((copy.durability_left, def.durability)),
                    holder: match at {
                        WeaponAt::Held(who, _) => Some(who),
                        WeaponAt::Stock(_) => None,
                    },
                    ..Deal::new(Trade::Repair(at), &copy.def, repair_cost(def, copy))
                })
            })
            .collect()
    }

    /// Opens the shop's lists from the tabs, if the battle has a shop.
    pub(super) fn open_shop(&mut self, ctx: &mut Ctx) {
        if self.prep.shop.is_some() {
            self.focus = Focus::Shop;
            ctx.audio.menu(MenuSound::Select);
        } else {
            ctx.audio.menu(MenuSound::Denied);
        }
    }

    /// One action in the shop's lists: up and down move the highlight
    /// (wrapping), left and right the shop's tab, Confirm makes the deal,
    /// Cancel goes back to the screen's tabs. While a sale is asked about,
    /// up and down set how many to sell instead.
    pub(super) fn handle_shop(&mut self, action: Action, ctx: &mut Ctx) {
        if let Some(sale) = self.selling {
            self.step_sell(sale, action, ctx);
            return;
        }
        match action {
            Action::CursorUp | Action::CursorDown => {
                let down = action == Action::CursorDown;
                self.deal = Self::moved(self.deal, self.deals().len(), down, ctx);
            }
            Action::CursorLeft | Action::CursorRight => {
                let forward = action == Action::CursorRight;
                self.shop_tab = wrap_step(self.shop_tab, Self::shop_tabs().len(), forward);
                self.deal = 0;
                ctx.audio.menu(MenuSound::Move);
            }
            Action::Confirm => self.make_deal(ctx),
            Action::Cancel => self.back(Focus::Tabs, ctx),
            _ => {}
        }
    }

    /// Confirm on the highlighted line: buys or repairs at once; a Sell
    /// line asks how many first.
    fn make_deal(&mut self, ctx: &mut Ctx) {
        let line = self.deal;
        let Some(deal) = self.deals().into_iter().nth(line) else {
            ctx.audio.menu(MenuSound::Denied);
            return;
        };
        let done = match &deal.trade {
            _ if !deal.enabled => Err(ShopError::NoItem),
            Trade::Sell(_) => {
                self.selling = Some(SellConfirm { line, count: 1 });
                ctx.audio.menu(MenuSound::Select);
                return;
            }
            Trade::Buy(item) => self.prep.buy(item),
            Trade::Repair(at) => self.prep.repair(*at),
        };
        self.after_deal(done.is_ok(), ctx);
    }

    /// The confirmation of `sale`: up and down change how many (1 to the
    /// line's count, no wrapping), Confirm sells them, Cancel goes back.
    fn step_sell(&mut self, sale: SellConfirm, action: Action, ctx: &mut Ctx) {
        let Some(deal) = self.deals().into_iter().nth(sale.line) else {
            self.selling = None;
            return;
        };
        let count = match action {
            Action::CursorUp => sale.count + 1,
            Action::CursorDown => sale.count.saturating_sub(1),
            Action::Confirm => {
                self.selling = None;
                let Trade::Sell(item) = &deal.trade else {
                    return;
                };
                // A weapon copy is one line; a counted item sells `count`.
                let mut sold = 0;
                for _ in 0..sale.count {
                    if self.prep.sell(item).is_ok() {
                        sold += 1;
                    }
                }
                self.after_deal(sold > 0, ctx);
                return;
            }
            Action::Cancel => {
                self.selling = None;
                ctx.audio.menu(MenuSound::Cancel);
                return;
            }
            _ => return,
        };
        if (1..=deal.count).contains(&count) {
            self.selling = Some(SellConfirm { count, ..sale });
            ctx.audio.menu(MenuSound::Move);
        }
    }

    /// The sound of a deal made or refused; the highlight stays on a line
    /// (one sold or repaired may be gone).
    fn after_deal(&mut self, done: bool, ctx: &mut Ctx) {
        ctx.audio.menu(if done {
            MenuSound::Select
        } else {
            MenuSound::Denied
        });
        self.deal = self.deal.min(self.deals().len().saturating_sub(1));
    }

    /// The line `deal` shows, in `words`: `Iron Sword  460  Mt 5 …`, a
    /// repair as `12/20 → 20/20  Test Lord`.
    fn deal_line(&self, ctx: &Ctx, words: Words<'_>, deal: &Deal) -> ShopLineView {
        let items = &self.prep.setup.items;
        let mut name = words.item(&deal.item, items).to_owned();
        if deal.count > 1 {
            name = format!("{name} ×{}", deal.count);
        }
        let stats = items.get(&deal.item).map(|def| stat_line(words, def));
        let detail = match (&deal.trade, deal.durability) {
            (Trade::Repair(_), Some((left, max))) => {
                let whose = deal.holder.and_then(|who| self.prep.unit(who));
                let whose = whose.map_or_else(|| ctx.text("prep.stock"), |u| words.unit(u));
                format!("{left}/{max} → {max}/{max}  {whose}")
            }
            (_, Some((left, max))) => format!("{left}/{max}  {}", stats.unwrap_or_default()),
            _ => stats.unwrap_or_default(),
        };
        ShopLineView {
            name,
            gold: gold_text(deal.gold),
            detail,
            enabled: deal.enabled,
        }
    }

    /// The confirmation of the sale being asked about.
    fn sell_view(&self, ctx: &Ctx, deals: &[Deal]) -> Option<SellConfirmView> {
        let sale = self.selling?;
        let deal = deals.get(sale.line)?;
        let item = ctx.words().item(&deal.item, &self.prep.setup.items);
        let total = deal
            .gold
            .saturating_mul(Gold::try_from(sale.count).unwrap_or(Gold::MAX));
        Some(SellConfirmView {
            question: ctx.text_with("battle.shop.sell_question", &[("item", &item)]),
            count: sale.count,
            max: deal.count,
            gold: ctx.text_with("battle.shop.sell_for", &[("gold", &gold_text(total))]),
        })
    }

    /// The Shop tab, as the on-map shop's view (its help row unused: the
    /// screen's help line says the keys).
    pub(super) fn shop_view(&self, ctx: &Ctx) -> ShopScreenView {
        let words = ctx.words();
        let deals = self.deals();
        let empty = if self.prep.shop.is_some() {
            "battle.shop.empty"
        } else {
            NO_SHOP
        };
        ShopScreenView {
            shop: ctx.text(kind_key(trpg_core::ShopKind::Basic)).to_owned(),
            gold: ctx.text_with(
                "battle.shop.gold",
                &[("gold", &gold_text(self.prep.setup.gold))],
            ),
            unit: String::new(),
            tabs: Self::shop_tabs()
                .iter()
                .map(|t| ctx.text(t.key()).to_owned())
                .collect(),
            tab: self.shop_tab,
            lines: deals
                .iter()
                .map(|d| self.deal_line(ctx, words, d))
                .collect(),
            focus: self.deal,
            empty: ctx.text(empty).to_owned(),
            selling: self.sell_view(ctx, &deals),
            help: String::new(),
            debug_hint: None,
        }
    }
}
