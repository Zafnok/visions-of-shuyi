//! The basic shop of the Preparations screen (`assets/data/basic_shop.ron`,
//! ticket 0443; `docs/design/weapons-and-items.md`, *The Preparations
//! shop*): every item it can sell with what unlocks it. The rules are in
//! `trpg_core::shop` ([`BasicStock`]).
use crate::prelude::*;

use serde::Deserialize;
use trpg_core::{BasicStock, ItemDef, ItemId, ItemTable, Unlock};

use crate::bundle;
use crate::error::ContentError;
use crate::formats::{self, DataTable};
use crate::ron_loader::parse_ron;

/// Path of the basic shop file inside the asset bundle.
pub const BASIC_SHOP_PATH: &str = "data/basic_shop.ron";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    items: Vec<RawRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRow {
    item: String,
    unlock: Unlock,
}

/// Loads the embedded file, its items checked against `items`; without
/// them (the item file failed) the file alone.
pub fn load(items: Option<&ItemTable>) -> Result<BasicStock, Vec<ContentError>> {
    let stock = formats::table::<BasicStock>(BASIC_SHOP_PATH)?;
    let errors = items.map_or_else(Vec::new, |items| {
        check(&bundle::display_path(BASIC_SHOP_PATH), &stock, items)
    });
    if errors.is_empty() {
        Ok(stock)
    } else {
        Err(errors)
    }
}

impl DataTable for BasicStock {
    const NAME: &'static str = "basic shop";

    fn from_source(file: &str, source: &str) -> Result<Self, Vec<ContentError>> {
        from_source(file, source)
    }
}

/// Parses `source`, attributing errors to `file`: every item at most once,
/// every chapter from 1. Reports every problem found.
pub fn from_source(file: &str, source: &str) -> Result<BasicStock, Vec<ContentError>> {
    let raw: RawFile = parse_ron(file, source).map_err(|e| vec![e])?;
    let mut errors = Vec::new();
    let mut items: Vec<(ItemId, Unlock)> = Vec::new();
    for row in raw.items {
        let id = ItemId::new(&row.item);
        if items.iter().any(|(i, _)| *i == id) {
            errors.push(ContentError::new(
                file,
                format!("item \"{}\" is listed twice", row.item),
            ));
            continue;
        }
        if row.unlock == Unlock::Chapter(0) {
            errors.push(ContentError::new(
                file,
                format!("item \"{}\": chapters start at 1", row.item),
            ));
        }
        items.push((id, row.unlock));
    }
    if errors.is_empty() {
        Ok(BasicStock { items })
    } else {
        Err(errors)
    }
}

/// What is wrong with `stock`'s items in `items`: one it doesn't have, or
/// one the basic shop never sells (an accessory or a seal).
pub fn check(file: &str, stock: &BasicStock, items: &ItemTable) -> Vec<ContentError> {
    let problem = |id: &ItemId| match items.get(id) {
        None => Some("no such item"),
        Some(ItemDef::Accessory(_)) => Some("accessories are never basics"),
        Some(ItemDef::Seal(_)) => Some("no shop sells seals"),
        Some(_) => None,
    };
    let rows = stock.items.iter();
    rows.filter_map(|(id, _)| {
        let why = problem(id)?;
        Some(ContentError::new(file, format!("item \"{}\": {why}", id.0)))
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn messages(errors: Vec<ContentError>) -> Vec<String> {
        errors.into_iter().map(|e| e.message).collect()
    }

    #[test]
    fn the_embedded_file_follows_the_design() {
        let items = crate::item::load().ok();
        let stock = load(items.as_ref());
        let Ok(stock) = stock else {
            panic!("{stock:?}");
        };
        let ids = |chapter| -> Vec<String> {
            let shop = stock.shop(chapter);
            shop.stock.into_iter().map(|i| i.0).collect()
        };
        let iron = [
            "potion",
            "iron_sword",
            "iron_spear",
            "iron_axe",
            "iron_bow",
            "iron_gauntlets",
            "linen_robe",
            "leather_vest",
            "chain_mail",
            "iron_plate",
        ];
        let steel = [
            "steel_sword",
            "steel_spear",
            "steel_axe",
            "steel_bow",
            "steel_gauntlets",
        ];
        assert_eq!(ids(1), [] as [String; 0]);
        // Chapter 2: no Steel, no Hi-Potion.
        let mut two = ids(2);
        two.sort();
        let mut want: Vec<String> = iron.iter().map(|s| (*s).to_owned()).collect();
        want.sort();
        assert_eq!(two, want);
        // Chapter 5: Steel too, no Hi-Potion.
        let five = ids(5);
        assert!(
            steel
                .iter()
                .chain(&iron)
                .all(|i| five.iter().any(|f| f == i))
        );
        assert_eq!(five.len(), iron.len() + steel.len());
        assert!(!five.iter().any(|i| i == "hi_potion"));
        // Chapter 8: the Hi-Potion, the Potion still on sale.
        let eight = ids(8);
        assert!(eight.iter().any(|i| i == "hi_potion"));
        assert!(eight.iter().any(|i| i == "potion"));
        assert_eq!(eight.len(), five.len() + 1);
    }

    #[test]
    fn every_bad_row_is_reported() {
        let source = r#"(items: [
            (item: "potion", unlock: Chapter(2)),
            (item: "potion", unlock: Chapter(5)),
            (item: "iron_sword", unlock: Chapter(0)),
            (item: "nope", unlock: Chapter(2)),
            (item: "speed_ring", unlock: Chapter(2)),
            (item: "tier_2_seal", unlock: Chapter(2)),
        ])"#;
        assert_eq!(
            messages(from_source("b.ron", source).err().unwrap_or_default()),
            [
                "item \"potion\" is listed twice",
                "item \"iron_sword\": chapters start at 1"
            ]
        );
        let fixed = source
            .replace("(item: \"potion\", unlock: Chapter(5)),", "")
            .replace("Chapter(0)", "Chapter(1)");
        let stock = from_source("b.ron", &fixed).unwrap_or_default();
        let items = crate::item::load().unwrap_or_default();
        assert_eq!(
            messages(check("b.ron", &stock, &items)),
            [
                "item \"nope\": no such item",
                "item \"speed_ring\": accessories are never basics",
                "item \"tier_2_seal\": no shop sells seals",
            ]
        );
        assert!(from_source("b.ron", "(items: [(item: \"potion\")])").is_err());
        assert!(from_source("b.ron", "(items: [], extra: 1)").is_err());
    }
}
