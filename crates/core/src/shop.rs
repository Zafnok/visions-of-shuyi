//! Gold, shops and chests: buying, selling and repairing, in battle (the
//! `Shop` action, [`crate::battle`]) and between chapters ([`ShopSession`]).
//!
//! Source: `docs/design/weapons-and-items.md`, "Money and shops" (0003).
//!
//! # Rules
//!
//! - **Gold** is party-wide and never negative (`u32`). Gains saturate.
//! - **Shops** ([`Shop`]) have a kind and a stock list:
//!   - an **Armoury** sells the weapons, armour and accessories on its list;
//!   - a **Vendor** sells the consumables on its list;
//!   - a **Blacksmith** repairs weapons (its list is empty);
//!   - the **Basic** shop of Preparations (ticket 0443) sells the
//!     weapons, armour and consumables on its list and repairs. Its list
//!     is the [`BasicStock`] reached by the chapter.
//!
//!   Armouries, vendors and the basic shop **buy** any item the party has, for half its
//!   price rounded down ([`sell_price`]). Blacksmiths don't buy.
//!   No shop ever stocks a **special weapon** (`combat-arts.md`), so a
//!   sold one is gone; the UI asks before selling one. It repairs by the
//!   usual formula.
//!   **Seals** (`progression.md`) are neither sold nor bought by any shop
//!   yet: where they come from is chapter and shop data (0009 and later).
//! - **Repair** ([`repair_cost`]) brings a weapon back to full durability
//!   for `ceil(price / 2 × missing / durability)` gold, where `missing` is
//!   the durability it lost. A weapon at full durability can't be repaired.
//! - Every function here checks before it changes anything: on `Err`, the
//!   gold and the item are unchanged.
//! - **Chests** hold [`Loot`]: gold or one item.

use crate::prelude::*;

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::item::{ItemDef, ItemId, ItemTable, Stock, WeaponDef, WeaponInstance};

/// Party gold.
pub type Gold = u32;

/// What a shop does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ShopKind {
    /// Sells weapons, armour and accessories; buys anything.
    Armoury,
    /// Sells consumables; buys anything.
    Vendor,
    /// Repairs weapons.
    Blacksmith,
    /// The Preparations shop: sells weapons, armour and consumables, buys
    /// anything and repairs. Never on a map.
    Basic,
}

impl ShopKind {
    /// Whether this kind of shop may have `item` on its list. No shop
    /// stocks a special weapon.
    pub fn stocks(self, item: &ItemDef) -> bool {
        match self {
            ShopKind::Armoury => match item {
                ItemDef::Weapon(w) => !w.is_special(),
                ItemDef::Armour(_) | ItemDef::Accessory(_) => true,
                _ => false,
            },
            ShopKind::Vendor => matches!(item, ItemDef::Consumable(_)),
            ShopKind::Blacksmith => false,
            ShopKind::Basic => match item {
                ItemDef::Weapon(w) => !w.is_special(),
                ItemDef::Armour(_) | ItemDef::Consumable(_) => true,
                _ => false,
            },
        }
    }

    /// Whether this kind of shop repairs weapons.
    pub fn repairs(self) -> bool {
        matches!(self, ShopKind::Blacksmith | ShopKind::Basic)
    }

    /// Whether this kind of shop buys items from the party.
    pub fn buys(self) -> bool {
        self != ShopKind::Blacksmith
    }
}

/// A shop: on a map tile, or in a town between chapters.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Shop {
    /// What it does.
    pub kind: ShopKind,
    /// What it sells, in menu order.
    pub stock: Vec<ItemId>,
}

/// What makes the basic shop sell an item (`weapons-and-items.md`, *The
/// Preparations shop*). Only the chapter for now; a special unlock (a
/// support rank, 0044) will be another variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Unlock {
    /// From this chapter's Preparations on.
    Chapter(u32),
}

impl Unlock {
    /// Whether it is reached in chapter `chapter`.
    pub fn reached(self, chapter: u32) -> bool {
        match self {
            Unlock::Chapter(from) => chapter >= from,
        }
    }
}

/// Everything the basic shop can sell, each with what unlocks it, in menu
/// order (`assets/data/basic_shop.ron`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BasicStock {
    /// Each item and its unlock.
    pub items: Vec<(ItemId, Unlock)>,
}

impl BasicStock {
    /// The basic shop of chapter `chapter`: every item unlocked by then.
    /// Lower tiers stay on sale.
    pub fn shop(&self, chapter: u32) -> Shop {
        let reached = self.items.iter().filter(|(_, u)| u.reached(chapter));
        Shop {
            kind: ShopKind::Basic,
            stock: reached.map(|(item, _)| item.clone()).collect(),
        }
    }
}

/// What a chest holds (or, later, a village gives).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Loot {
    /// This much gold.
    Gold(Gold),
    /// One item (weapons at full durability).
    Item(ItemId),
}

/// Why a shop transaction was refused. Nothing changed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShopError {
    /// The party can't pay.
    NotEnoughGold {
        /// The price.
        cost: Gold,
        /// The party's gold.
        gold: Gold,
    },
    /// This shop doesn't sell the item.
    NotSoldHere(ItemId),
    /// This shop doesn't buy items (a blacksmith).
    DoesNotBuy,
    /// This shop doesn't repair (not a blacksmith or the basic shop).
    DoesNotRepair,
    /// The weapon is at full durability.
    NothingToRepair(ItemId),
    /// The item to sell or repair isn't there.
    NoItem,
    /// The item isn't a weapon, so it can't be repaired.
    NotAWeapon(ItemId),
    /// The item id is not in the item table.
    UnknownItem(ItemId),
    /// No shop buys this item (a seal).
    NotBought(ItemId),
}

impl fmt::Display for ShopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShopError::NotEnoughGold { cost, gold } => {
                write!(f, "costs {cost} gold, the party has {gold}")
            }
            ShopError::NotSoldHere(i) => write!(f, "\"{}\" isn't sold here", i.0),
            ShopError::DoesNotBuy => f.write_str("this shop doesn't buy items"),
            ShopError::DoesNotRepair => f.write_str("this shop doesn't repair"),
            ShopError::NothingToRepair(i) => write!(f, "\"{}\" needs no repair", i.0),
            ShopError::NoItem => f.write_str("no such item"),
            ShopError::NotAWeapon(i) => write!(f, "\"{}\" is not a weapon", i.0),
            ShopError::UnknownItem(i) => write!(f, "unknown item \"{}\"", i.0),
            ShopError::NotBought(i) => write!(f, "no shop buys \"{}\"", i.0),
        }
    }
}

impl core::error::Error for ShopError {}

/// The price at which a shop buys an item from the party: half its price,
/// rounded down.
pub fn sell_price(item: &ItemDef) -> Gold {
    item.price() / 2
}

/// The gold to bring `copy` of `weapon` back to full durability:
/// `ceil(price / 2 × missing / durability)`, in integer maths. 0 when
/// nothing is missing.
pub fn repair_cost(weapon: &WeaponDef, copy: &WeaponInstance) -> Gold {
    let durability = u64::from(weapon.durability.max(1));
    let missing = u64::from(weapon.durability.saturating_sub(copy.durability_left));
    let cost = (u64::from(weapon.price) * missing).div_ceil(2 * durability);
    Gold::try_from(cost).unwrap_or(Gold::MAX)
}

/// Takes `cost` from `gold`, or refuses if there isn't enough.
pub fn pay(gold: &mut Gold, cost: Gold) -> Result<(), ShopError> {
    *gold = gold
        .checked_sub(cost)
        .ok_or(ShopError::NotEnoughGold { cost, gold: *gold })?;
    Ok(())
}

/// Buys `item` from `shop`: pays its price and returns it. The caller puts
/// the item where it goes.
pub fn buy(
    shop: &Shop,
    items: &ItemTable,
    gold: &mut Gold,
    item: &ItemId,
) -> Result<Gold, ShopError> {
    if !shop.stock.contains(item) {
        return Err(ShopError::NotSoldHere(item.clone()));
    }
    let def = items
        .get(item)
        .ok_or_else(|| ShopError::UnknownItem(item.clone()))?;
    if !shop.kind.stocks(def) {
        return Err(ShopError::NotSoldHere(item.clone()));
    }
    let price = def.price();
    pay(gold, price)?;
    Ok(price)
}

/// Sells `item` to `shop`: adds its sell price to `gold` and returns it.
/// The caller removes the item.
pub fn sell(
    shop: &Shop,
    items: &ItemTable,
    gold: &mut Gold,
    item: &ItemId,
) -> Result<Gold, ShopError> {
    if !shop.kind.buys() {
        return Err(ShopError::DoesNotBuy);
    }
    let def = items
        .get(item)
        .ok_or_else(|| ShopError::UnknownItem(item.clone()))?;
    if matches!(def, ItemDef::Seal(_)) {
        return Err(ShopError::NotBought(item.clone()));
    }
    let price = sell_price(def);
    *gold = gold.saturating_add(price);
    Ok(price)
}

/// Repairs `copy` at `shop` to full durability, paying for it. Returns the
/// cost.
pub fn repair(
    shop: &Shop,
    items: &ItemTable,
    gold: &mut Gold,
    copy: &mut WeaponInstance,
) -> Result<Gold, ShopError> {
    if !shop.kind.repairs() {
        return Err(ShopError::DoesNotRepair);
    }
    let def = match items.get(&copy.def) {
        None => return Err(ShopError::UnknownItem(copy.def.clone())),
        Some(ItemDef::Weapon(w)) => w,
        Some(_) => return Err(ShopError::NotAWeapon(copy.def.clone())),
    };
    if copy.durability_left >= def.durability {
        return Err(ShopError::NothingToRepair(copy.def.clone()));
    }
    let cost = repair_cost(def, copy);
    pay(gold, cost)?;
    copy.durability_left = def.durability;
    Ok(cost)
}

/// Adds one `item` to `stock`: weapons as a new copy at full durability.
/// Refuses unknown items (nothing changes).
pub fn add_to_stock(stock: &mut Stock, items: &ItemTable, item: &ItemId) -> Result<(), ShopError> {
    match items.get(item) {
        None => Err(ShopError::UnknownItem(item.clone())),
        Some(ItemDef::Weapon(_)) => {
            stock.weapons.extend(items.new_weapon(item));
            Ok(())
        }
        Some(_) => {
            stock.add(item.clone());
            Ok(())
        }
    }
}

/// A shop visit outside battle (between chapters, or in a town node):
/// buying into and selling from the party's stock with the campaign's gold.
/// Uses the same rules as the in-battle `Shop` action.
#[derive(Debug)]
pub struct ShopSession<'a> {
    shop: &'a Shop,
    items: &'a ItemTable,
    gold: &'a mut Gold,
    stock: &'a mut Stock,
}

impl<'a> ShopSession<'a> {
    /// Opens `shop` for a party with `gold` and `stock`.
    pub fn new(
        shop: &'a Shop,
        items: &'a ItemTable,
        gold: &'a mut Gold,
        stock: &'a mut Stock,
    ) -> Self {
        Self {
            shop,
            items,
            gold,
            stock,
        }
    }

    /// The shop.
    pub fn shop(&self) -> &Shop {
        self.shop
    }

    /// The party's gold.
    pub fn gold(&self) -> Gold {
        *self.gold
    }

    /// The party's stock.
    pub fn stock(&self) -> &Stock {
        self.stock
    }

    /// Buys one `item` into the stock. Returns the price paid.
    pub fn buy(&mut self, item: &ItemId) -> Result<Gold, ShopError> {
        let mut gold = *self.gold;
        let price = buy(self.shop, self.items, &mut gold, item)?;
        add_to_stock(self.stock, self.items, item)?;
        *self.gold = gold;
        Ok(price)
    }

    /// Sells one `item` (not a weapon) from the stock. Returns the gold got.
    pub fn sell_item(&mut self, item: &ItemId) -> Result<Gold, ShopError> {
        if self.stock.count(item) == 0 {
            return Err(ShopError::NoItem);
        }
        let price = sell(self.shop, self.items, self.gold, item)?;
        self.stock.take(item);
        Ok(price)
    }

    /// Sells the stock's weapon copy `index`. Returns the gold got.
    pub fn sell_weapon(&mut self, index: usize) -> Result<Gold, ShopError> {
        let copy = self.stock.weapons.get(index).ok_or(ShopError::NoItem)?;
        let price = sell(self.shop, self.items, self.gold, &copy.def)?;
        self.stock.weapons.remove(index);
        Ok(price)
    }

    /// Repairs the stock's weapon copy `index`. Returns the cost.
    pub fn repair_weapon(&mut self, index: usize) -> Result<Gold, ShopError> {
        let copy = self.stock.weapons.get_mut(index).ok_or(ShopError::NoItem)?;
        repair(self.shop, self.items, self.gold, copy)
    }
}

#[cfg(test)]
mod tests;
