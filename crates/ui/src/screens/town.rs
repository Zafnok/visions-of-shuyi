//! A town's shop (ticket 1007, `docs/design/world-structure.md` *Towns*):
//! the on-map shop screen of 0409, its tabs, lines and look, opened from
//! the world map, trading with the party's stock and gold
//! ([`Campaign::trade`]).
//!
//! - **Buy** puts the item in the stock.
//! - **Sell** sells from the stock (its weapons, each copy a line, then
//!   its other items, counted): up and down set how many before Confirm,
//!   and a special weapon asks first, as on a map (0329). A specialty
//!   store's refusal shows why in place of the item's numbers (0320).
//! - **Repair** (a Blacksmith) mends the stock's weapons and those the
//!   army holds, the holder's name beside each.
//!
//! Each deal is made at once (there is no battle to send it to, as on
//! Preparations). Cancel leaves, popping with the campaign as the shop
//! left it, for the world map to take back.
//!
//! [`TownScreen::view`] says what it shows; the on-map shop's skin paints
//! it (ADR-0054).

use crate::prelude::*;

use trpg_core::{
    Campaign, Gold, ItemDef, ItemId, Shop, StockItem, TownTrade, WEAPON_SLOTS, WeaponDef,
    repair_cost, sell_price,
};

use super::battle::glyph::shop as glyph;
use super::battle::shop::{
    HELP, HELP_ONE_TAB, HELP_SELL, HELP_SPECIAL, SELL_SPECIAL, SellConfirm, Tab, gold_text,
    kind_key, stat_line, wont_take_key,
};
use super::battle::view::{SellConfirmView, ShopLineView, ShopScreenView};
use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::widgets::{Ask, AskView, ListCursor};

/// One line of a tab: the deal, its item and how much.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deal {
    /// What Confirm does.
    pub trade: TownTrade,
    /// The item.
    pub item: ItemId,
    /// How many there are (a counted item of the stock).
    pub count: usize,
    /// Its price, sell price or repair cost.
    pub gold: Gold,
    /// A weapon's durability left and its full durability.
    pub durability: Option<(u32, u32)>,
    /// Who holds the weapon to repair (a roster index); `None`: the stock.
    pub holder: Option<usize>,
    /// Whether the deal can be made now.
    pub enabled: bool,
}

/// A town's shop screen. See the module docs.
#[derive(Debug, Clone)]
pub struct TownScreen {
    campaign: Campaign,
    shop: Shop,
    /// The town's name.
    town: String,
    tab: usize,
    cursor: ListCursor,
    selling: Option<SellConfirm>,
    special: Option<(usize, Ask<bool>)>,
}

impl TownScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "town_shop";

    /// `shop` of town `town` for `campaign`, on its first tab.
    pub fn new(campaign: Campaign, shop: Shop, town: String) -> Self {
        Self {
            campaign,
            shop,
            town,
            tab: 0,
            cursor: ListCursor::new(0),
            selling: None,
            special: None,
        }
    }

    /// The campaign as the shop has it.
    pub fn campaign(&self) -> &Campaign {
        &self.campaign
    }

    /// The tab shown.
    pub fn tab(&self) -> Tab {
        Tab::of(self.shop.kind)[self.tab]
    }

    /// The lines of the tab shown, each enabled if the deal can be made.
    pub fn deals(&self, ctx: &Ctx) -> Vec<Deal> {
        let items = &ctx.game.content.items;
        let mut deals = match self.tab() {
            Tab::Buy => self
                .shop
                .stock
                .iter()
                .filter_map(|id| Some(deal(TownTrade::Buy(id.clone()), id, items.get(id)?.price())))
                .collect(),
            Tab::Sell => self.sales(ctx),
            Tab::Repair => self.repairs(ctx),
        };
        for d in &mut deals {
            let mut test = self.campaign.clone();
            d.enabled = test.trade(&self.shop, items, &d.trade).is_ok();
        }
        deals
    }

    /// The stock's weapons, each copy a line, then its other items with
    /// how many there are; seals never (no shop buys them).
    fn sales(&self, ctx: &Ctx) -> Vec<Deal> {
        let items = &ctx.game.content.items;
        let kind = self.shop.kind;
        let stock = &self.campaign.stock;
        let mut deals: Vec<Deal> = stock
            .weapons
            .iter()
            .enumerate()
            .filter_map(|(i, copy)| {
                let def = items.weapon(&copy.def)?;
                let gold = sell_price(kind, items.get(&copy.def)?);
                Some(Deal {
                    durability: Some((copy.durability_left, def.durability)),
                    ..deal(TownTrade::Sell(StockItem::Weapon(i)), &copy.def, gold)
                })
            })
            .collect();
        for (id, count) in &stock.items {
            let Some(def) = items.get(id).filter(|d| !matches!(d, ItemDef::Seal(_))) else {
                continue;
            };
            deals.push(Deal {
                count: usize::try_from(*count).unwrap_or(usize::MAX),
                ..deal(
                    TownTrade::Sell(StockItem::Item(id.clone())),
                    id,
                    sell_price(kind, def),
                )
            });
        }
        deals
    }

    /// Every weapon that lost durability: the stock's, then each unit's of
    /// the army, in slot order.
    fn repairs(&self, ctx: &Ctx) -> Vec<Deal> {
        let items = &ctx.game.content.items;
        let stock = self
            .campaign
            .stock
            .weapons
            .iter()
            .enumerate()
            .filter_map(|(i, copy)| {
                let (def, copy) = worn(items, copy)?;
                Some((TownTrade::RepairStock(i), None, def, copy))
            });
        let held = self
            .campaign
            .roster
            .iter()
            .enumerate()
            .flat_map(|(unit, u)| {
                (0..WEAPON_SLOTS).filter_map(move |slot| {
                    let (def, copy) = worn(items, u.loadout.weapon(slot)?)?;
                    Some((TownTrade::RepairHeld { unit, slot }, Some(unit), def, copy))
                })
            });
        stock
            .chain(held)
            .map(|(trade, holder, def, copy): (_, _, &WeaponDef, _)| Deal {
                durability: Some((copy.durability_left, def.durability)),
                holder,
                ..deal(trade, &copy.def, repair_cost(def, copy))
            })
            .collect()
    }

    /// Confirm on the highlighted line: buys or repairs at once; a Sell
    /// line asks how many first (or, for a special weapon, whether at
    /// all).
    fn choose(&mut self, ctx: &mut Ctx) {
        let line = self.cursor.focus();
        let Some(d) = self.deals(ctx).into_iter().nth(line) else {
            ctx.audio.menu(MenuSound::Denied);
            return;
        };
        if !d.enabled {
            ctx.audio.menu(MenuSound::Denied);
            return;
        }
        if let TownTrade::Sell(_) = d.trade {
            let special = ctx
                .game
                .content
                .items
                .weapon(&d.item)
                .is_some_and(WeaponDef::is_special);
            if special {
                let ask = Ask::new(
                    SELL_SPECIAL,
                    [(false, "battle.shop.no"), (true, "battle.shop.yes")],
                );
                self.special = Some((line, ask.cancelled_by(false)));
            } else {
                self.selling = Some(SellConfirm { line, count: 1 });
            }
            ctx.audio.menu(MenuSound::Select);
            return;
        }
        let done = self.make(ctx, &d.trade, 1);
        self.after(ctx, done);
    }

    /// Makes `trade` `times` times; whether every one was made.
    fn make(&mut self, ctx: &Ctx, trade: &TownTrade, times: usize) -> bool {
        let items = &ctx.game.content.items;
        (0..times).all(|_| self.campaign.trade(&self.shop, items, trade).is_ok())
    }

    /// The sound of a deal made or refused; the highlight stays on a line.
    fn after(&mut self, ctx: &mut Ctx, done: bool) {
        ctx.audio.menu(if done {
            MenuSound::Select
        } else {
            MenuSound::Denied
        });
        let len = self.deals(ctx).len();
        let focus = self.cursor.focus().min(len.saturating_sub(1));
        self.cursor = ListCursor::new(len).at(focus);
    }

    /// The confirmation of `sale`: up and down change how many (1 to the
    /// line's count), Confirm sells them, Cancel goes back.
    fn step_sell(&mut self, ctx: &mut Ctx, sale: SellConfirm, action: Action) {
        let Some(d) = self.deals(ctx).into_iter().nth(sale.line) else {
            self.selling = None;
            return;
        };
        let count = match action {
            Action::CursorUp => sale.count + 1,
            Action::CursorDown => sale.count.saturating_sub(1),
            Action::Confirm => {
                self.selling = None;
                let done = self.make(ctx, &d.trade, sale.count);
                self.after(ctx, done);
                return;
            }
            Action::Cancel => {
                self.selling = None;
                ctx.audio.menu(MenuSound::Cancel);
                return;
            }
            _ => return,
        };
        if (1..=d.count).contains(&count) {
            self.selling = Some(SellConfirm { count, ..sale });
            ctx.audio.menu(MenuSound::Move);
        }
    }

    /// The question before selling a special weapon: Yes sells it, No and
    /// Cancel go back.
    fn step_special(&mut self, ctx: &mut Ctx, action: Action) {
        let Some((line, ask)) = self.special.as_mut() else {
            return;
        };
        let line = *line;
        let Some(answer) = ask.handle(action, &mut ctx.audio) else {
            return;
        };
        self.special = None;
        match (answer, self.deals(ctx).into_iter().nth(line)) {
            (true, Some(d)) => {
                let done = self.make(ctx, &d.trade, 1);
                self.after(ctx, done);
            }
            _ => ctx.audio.menu(MenuSound::Cancel),
        }
    }

    /// One action: up and down move the highlight (wrapping), left and
    /// right the tab, Confirm makes the deal, Cancel leaves.
    fn step(&mut self, ctx: &mut Ctx, action: Action) -> Transition {
        if let Some(sale) = self.selling {
            self.step_sell(ctx, sale, action);
            return Transition::None;
        }
        if self.special.is_some() {
            self.step_special(ctx, action);
            return Transition::None;
        }
        match action {
            Action::Cancel => {
                ctx.audio.menu(MenuSound::Cancel);
                return Transition::pop_with(self.campaign.clone());
            }
            Action::CursorDown | Action::CursorUp => {
                let len = self.deals(ctx).len();
                let mut cursor =
                    ListCursor::new(len).at(self.cursor.focus().min(len.saturating_sub(1)));
                if cursor.step(action == Action::CursorDown, true, |_| true) {
                    ctx.audio.menu(MenuSound::Move);
                }
                self.cursor = cursor;
            }
            Action::CursorLeft | Action::CursorRight => {
                // A town's shop has one tab or two (never the Basic shop's
                // three, `check_town`): left and right both switch.
                if Tab::of(self.shop.kind).len() == 2 {
                    self.tab = 1 - self.tab;
                    self.cursor = ListCursor::new(self.deals(ctx).len());
                    ctx.audio.menu(MenuSound::Move);
                }
            }
            Action::Confirm => self.choose(ctx),
            _ => {}
        }
        Transition::None
    }

    /// The line `d` shows: `Iron Sword  460  Mt 5 …`, a refusal's reason,
    /// or a repair as `12/20 → 20/20  Test Lord`.
    fn line(&self, ctx: &Ctx, d: &Deal) -> ShopLineView {
        let words = ctx.words();
        let items = &ctx.game.content.items;
        let mut name = words.item(&d.item, items).to_owned();
        if d.count > 1 {
            name = format!("{name} ×{}", d.count);
        }
        let def = items.get(&d.item);
        let refused = matches!(d.trade, TownTrade::Sell(_))
            && def.is_some_and(|def| !self.shop.kind.takes(def));
        if let Some(key) = wont_take_key(self.shop.kind).filter(|_| refused) {
            return ShopLineView {
                name,
                gold: String::new(),
                detail: ctx.text(key).to_owned(),
                enabled: false,
            };
        }
        let stats = def.map(|def| stat_line(words, def)).unwrap_or_default();
        let detail = match (&d.trade, d.durability) {
            (TownTrade::RepairStock(_) | TownTrade::RepairHeld { .. }, Some((left, max))) => {
                let whose = d.holder.and_then(|i| self.campaign.roster.get(i));
                let whose = whose.map_or_else(|| ctx.text("prep.stock"), |u| words.unit(u));
                format!("{left}/{max} → {max}/{max}  {whose}")
            }
            (_, Some((left, max))) => format!("{left}/{max}  {stats}"),
            _ => stats,
        };
        ShopLineView {
            name,
            gold: gold_text(d.gold),
            detail,
            enabled: d.enabled,
        }
    }

    /// The confirmation of the sale being asked about.
    fn sell_view(&self, ctx: &Ctx, deals: &[Deal]) -> Option<SellConfirmView> {
        let sale = self.selling?;
        let d = deals.get(sale.line)?;
        let item = ctx.words().item(&d.item, &ctx.game.content.items);
        let total = d
            .gold
            .saturating_mul(Gold::try_from(sale.count).unwrap_or(Gold::MAX));
        Some(SellConfirmView {
            question: ctx.text_with("battle.shop.sell_question", &[("item", &item)]),
            count: sale.count,
            max: d.count,
            gold: ctx.text_with("battle.shop.sell_for", &[("gold", &gold_text(total))]),
        })
    }

    /// The question before selling the special weapon asked about.
    fn special_view(&self, ctx: &Ctx, deals: &[Deal]) -> Option<AskView<bool>> {
        let (line, ask) = self.special.as_ref()?;
        let d = deals.get(*line)?;
        let item = ctx.words().item(&d.item, &ctx.game.content.items);
        let gold = gold_text(d.gold);
        let args: [(&str, &dyn core::fmt::Display); 2] = [("item", &item), ("gold", &gold)];
        Some(ask.view_with_keys(ctx, ctx.help_keys(), &args))
    }

    /// The whole screen (ADR-0054), in the player's language and with
    /// their keys.
    pub fn view(&self, ctx: &Ctx) -> ShopScreenView {
        let deals = self.deals(ctx);
        let tabs = Tab::of(self.shop.kind);
        let help = match (self.selling, &self.special, tabs.len()) {
            (Some(_), ..) => HELP_SELL,
            (None, Some(_), _) => HELP_SPECIAL,
            (None, None, 1) => HELP_ONE_TAB,
            (None, None, _) => HELP,
        };
        ShopScreenView {
            shop: ctx.text(kind_key(self.shop.kind)).to_owned(),
            gold: ctx.text_with(
                "battle.shop.gold",
                &[("gold", &gold_text(self.campaign.gold))],
            ),
            unit: self.town.clone(),
            tabs: tabs.iter().map(|t| ctx.text(t.key()).to_owned()).collect(),
            tab: self.tab,
            lines: deals.iter().map(|d| self.line(ctx, d)).collect(),
            focus: self.cursor.focus().min(deals.len().saturating_sub(1)),
            empty: ctx.text("battle.shop.empty").to_owned(),
            selling: self.sell_view(ctx, &deals),
            special: self.special_view(ctx, &deals),
            help: ctx.text_with(help, &[]),
            debug_hint: crate::screens::debug_hint(ctx),
        }
    }
}

/// `copy` and its weapon, if it lost durability.
fn worn<'a>(
    items: &'a trpg_core::ItemTable,
    copy: &'a trpg_core::WeaponInstance,
) -> Option<(&'a WeaponDef, &'a trpg_core::WeaponInstance)> {
    let def = items.weapon(&copy.def)?;
    (copy.durability_left < def.durability).then_some((def, copy))
}

/// A deal of `trade` on `item` for `gold`, enabled until checked.
fn deal(trade: TownTrade, item: &ItemId, gold: Gold) -> Deal {
    Deal {
        trade,
        item: item.clone(),
        count: 1,
        gold,
        durability: None,
        holder: None,
        enabled: true,
    }
}

impl Screen<Shuyi> for TownScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn actions(&self) -> Option<Vec<crate::input::Action>> {
        Some(crate::screens::actions::TOWN.to_vec())
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        for &action in &input.actions {
            let t = self.step(ctx, action);
            if !matches!(t, Transition::None) {
                return t;
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
}

#[cfg(test)]
mod tests;
