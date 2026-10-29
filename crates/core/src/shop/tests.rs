//! Tests of the shop rules. Test items: `sword` (weapon, 1000 gold, 40
//! durability), `bow` (weapon, 480 gold, 45 durability), `free_knife`
//! (weapon, 0 gold), `vest` (leather armour, 300), `robe` (cloth armour,
//! 120), `plate` (plate armour, 800), `ring` (accessory, 999), `potion`
//! (consumable, 300).

use proptest::prelude::*;

use super::*;
use crate::class::ArmourWeight;
use crate::combat::DamageType;
use crate::item::{AccessoryDef, ArmourDef, ConsumableDef, ConsumableEffect, SealDef, SealKind};
use crate::stats::Stats;
use crate::weapon::{WeaponKind, WeaponRank};

fn weapon(price: u32, durability: u32) -> ItemDef {
    ItemDef::Weapon(WeaponDef {
        name: "w".into(),
        kind: WeaponKind::Sword,
        rank: WeaponRank::E,
        might: 5,
        hit: 90,
        crit: 0,
        weight: 5,
        min_range: 1,
        max_range: 1,
        damage_type: DamageType::Physical,
        durability,
        effective: vec![],
        arts: vec![],
        price,
        special: None,
    })
}

fn armour(weight_class: ArmourWeight, price: u32) -> ItemDef {
    ItemDef::Armour(ArmourDef {
        name: "a".into(),
        weight_class,
        bonus: Stats::default(),
        weight: 0,
        price,
    })
}

fn items() -> ItemTable {
    let entries = [
        ("robe", armour(ArmourWeight::Cloth, 120)),
        ("plate", armour(ArmourWeight::Plate, 800)),
        ("sword", weapon(1000, 40)),
        ("bow", weapon(480, 45)),
        ("free_knife", weapon(0, 10)),
        (
            "vest",
            ItemDef::Armour(ArmourDef {
                name: "Vest".into(),
                weight_class: ArmourWeight::Leather,
                bonus: Stats::default(),
                weight: 0,
                price: 300,
            }),
        ),
        (
            "ring",
            ItemDef::Accessory(AccessoryDef {
                name: "Ring".into(),
                bonus: Stats::default(),
                price: 999,
            }),
        ),
        (
            "potion",
            ItemDef::Consumable(ConsumableDef {
                name: "Potion".into(),
                effect: ConsumableEffect::Heal(10),
                price: 300,
            }),
        ),
        (
            "seal",
            ItemDef::Seal(SealDef {
                name: "Seal".into(),
                kind: SealKind::Reclass,
            }),
        ),
    ];
    ItemTable {
        items: entries
            .into_iter()
            .map(|(k, v)| (ItemId::new(k), v))
            .collect(),
        ..ItemTable::default()
    }
}

fn id(s: &str) -> ItemId {
    ItemId::new(s)
}

fn shop(kind: ShopKind, stock: &[&str]) -> Shop {
    Shop {
        kind,
        stock: stock.iter().map(|s| id(s)).collect(),
    }
}

fn armoury() -> Shop {
    shop(
        ShopKind::Armoury,
        &["sword", "vest", "ring", "potion", "ghost"],
    )
}

fn vendor() -> Shop {
    shop(ShopKind::Vendor, &["potion", "sword"])
}

fn smith() -> Shop {
    shop(ShopKind::Blacksmith, &[])
}

fn copy(def: &str, left: u32) -> WeaponInstance {
    WeaponInstance {
        def: id(def),
        durability_left: left,
    }
}

fn weapon_def(name: &str) -> WeaponDef {
    items().weapon(&id(name)).cloned().unwrap()
}

#[test]
fn repair_cost_table() {
    // (weapon, durability left, cost): ceil(price / 2 × missing / durability).
    let cases = [
        ("sword", 0, 500),  // fully used: half the price
        ("sword", 20, 250), // half used
        ("sword", 39, 13),  // 1 missing: ceil(12.5)
        ("sword", 40, 0),   // nothing missing
        ("bow", 38, 38),    // 7 of 45 missing: ceil(37.33)
        ("bow", 0, 240),
        ("free_knife", 0, 0),
    ];
    for (name, left, cost) in cases {
        assert_eq!(
            repair_cost(&weapon_def(name), &copy(name, left)),
            cost,
            "{name} at {left}"
        );
    }
}

#[test]
fn repair_cost_never_overflows() {
    let mut def = weapon_def("sword");
    def.price = u32::MAX;
    def.durability = u32::MAX;
    assert_eq!(repair_cost(&def, &copy("sword", 0)), u32::MAX / 2 + 1);
    // A zero-durability weapon (invalid data) doesn't divide by zero.
    def.durability = 0;
    assert_eq!(repair_cost(&def, &copy("sword", 0)), 0);
}

#[test]
fn sell_price_is_half_rounded_down() {
    let t = items();
    let price = |s: &str| sell_price(ShopKind::Vendor, t.get(&id(s)).unwrap());
    assert_eq!(price("sword"), 500);
    assert_eq!(price("ring"), 499);
    assert_eq!(price("vest"), 150);
    assert_eq!(price("free_knife"), 0);
}

#[test]
fn pay_takes_gold_or_refuses() {
    let mut gold = 100;
    assert_eq!(pay(&mut gold, 100), Ok(()));
    assert_eq!(gold, 0);
    assert_eq!(
        pay(&mut gold, 1),
        Err(ShopError::NotEnoughGold { cost: 1, gold: 0 })
    );
    assert_eq!(gold, 0);
}

#[test]
fn kinds_stock_and_buy() {
    let t = items();
    let stocks = |k: ShopKind, s: &str| k.stocks(t.get(&id(s)).unwrap());
    assert!(stocks(ShopKind::Armoury, "sword"));
    assert!(stocks(ShopKind::Armoury, "vest"));
    assert!(stocks(ShopKind::Armoury, "ring"));
    assert!(!stocks(ShopKind::Armoury, "potion"));
    assert!(stocks(ShopKind::Vendor, "potion"));
    assert!(!stocks(ShopKind::Vendor, "sword"));
    assert!(!stocks(ShopKind::Blacksmith, "sword"));
    assert!(!stocks(ShopKind::Blacksmith, "potion"));
    // No shop sells seals.
    for kind in [ShopKind::Armoury, ShopKind::Vendor, ShopKind::Blacksmith] {
        assert!(!stocks(kind, "seal"));
    }
    assert!(ShopKind::Armoury.buys());
    assert!(ShopKind::Vendor.buys());
    assert!(!ShopKind::Blacksmith.buys());
}

#[test]
fn buy_pays_the_price() {
    let t = items();
    let mut gold = 1500;
    assert_eq!(buy(&armoury(), &t, &mut gold, &id("sword")), Ok(1000));
    assert_eq!(gold, 500);
    assert_eq!(buy(&vendor(), &t, &mut gold, &id("potion")), Ok(300));
    assert_eq!(gold, 200);
}

#[test]
fn buy_errors_change_nothing() {
    let t = items();
    let mut gold = 900;
    let cases = [
        (
            armoury(),
            "sword",
            ShopError::NotEnoughGold {
                cost: 1000,
                gold: 900,
            },
        ),
        (armoury(), "bow", ShopError::NotSoldHere(id("bow"))),
        // Listed, but not what an armoury sells.
        (armoury(), "potion", ShopError::NotSoldHere(id("potion"))),
        (vendor(), "sword", ShopError::NotSoldHere(id("sword"))),
        (armoury(), "ghost", ShopError::UnknownItem(id("ghost"))),
        (smith(), "sword", ShopError::NotSoldHere(id("sword"))),
    ];
    for (shop, item, err) in cases {
        assert_eq!(buy(&shop, &t, &mut gold, &id(item)), Err(err), "{item}");
        assert_eq!(gold, 900);
    }
}

#[test]
fn sell_gives_half_price() {
    let t = items();
    let mut gold = 10;
    assert_eq!(sell(&armoury(), &t, &mut gold, &id("ring")), Ok(499));
    assert_eq!(gold, 509);
    // Shops buy anything, listed or not.
    assert_eq!(sell(&vendor(), &t, &mut gold, &id("bow")), Ok(240));
    assert_eq!(gold, 749);
    let mut rich = u32::MAX - 1;
    assert_eq!(sell(&vendor(), &t, &mut rich, &id("sword")), Ok(500));
    assert_eq!(rich, u32::MAX);
}

#[test]
fn sell_errors_change_nothing() {
    let t = items();
    let mut gold = 10;
    assert_eq!(
        sell(&smith(), &t, &mut gold, &id("ring")),
        Err(ShopError::DoesNotBuy)
    );
    assert_eq!(
        sell(&armoury(), &t, &mut gold, &id("ghost")),
        Err(ShopError::UnknownItem(id("ghost")))
    );
    // No shop buys a seal.
    assert_eq!(
        sell(&armoury(), &t, &mut gold, &id("seal")),
        Err(ShopError::NotBought(id("seal")))
    );
    assert_eq!(gold, 10);
}

#[test]
fn repair_restores_durability() {
    let t = items();
    let mut gold = 300;
    let mut w = copy("sword", 20);
    assert_eq!(repair(&smith(), &t, &mut gold, &mut w), Ok(250));
    assert_eq!((gold, w.durability_left), (50, 40));
    // A broken weapon can be repaired.
    let mut broken = copy("bow", 0);
    let mut gold = 240;
    assert_eq!(repair(&smith(), &t, &mut gold, &mut broken), Ok(240));
    assert_eq!((gold, broken.durability_left), (0, 45));
}

#[test]
fn repair_errors_change_nothing() {
    let t = items();
    let cases = [
        (armoury(), copy("sword", 20), ShopError::DoesNotRepair),
        (vendor(), copy("sword", 20), ShopError::DoesNotRepair),
        (
            smith(),
            copy("sword", 40),
            ShopError::NothingToRepair(id("sword")),
        ),
        (
            smith(),
            copy("sword", 0),
            ShopError::NotEnoughGold {
                cost: 500,
                gold: 499,
            },
        ),
        (smith(), copy("vest", 0), ShopError::NotAWeapon(id("vest"))),
        (
            smith(),
            copy("ghost", 0),
            ShopError::UnknownItem(id("ghost")),
        ),
    ];
    for (shop, w, err) in cases {
        let mut gold = 499;
        let mut after = w.clone();
        assert_eq!(repair(&shop, &t, &mut gold, &mut after), Err(err));
        assert_eq!((gold, after), (499, w));
    }
}

#[test]
fn add_to_stock_by_kind() {
    let t = items();
    let mut stock = Stock::default();
    assert_eq!(add_to_stock(&mut stock, &t, &id("sword")), Ok(()));
    assert_eq!(add_to_stock(&mut stock, &t, &id("potion")), Ok(()));
    assert_eq!(
        add_to_stock(&mut stock, &t, &id("ghost")),
        Err(ShopError::UnknownItem(id("ghost")))
    );
    assert_eq!(stock.weapons, [copy("sword", 40)]);
    assert_eq!(stock.count(&id("potion")), 1);
    assert_eq!(stock.items.len(), 1);
}

#[test]
fn errors_display() {
    let cases = [
        (
            ShopError::NotEnoughGold { cost: 5, gold: 2 },
            "costs 5 gold, the party has 2",
        ),
        (ShopError::NotSoldHere(id("x")), "\"x\" isn't sold here"),
        (ShopError::DoesNotBuy, "this shop doesn't buy items"),
        (ShopError::DoesNotRepair, "this shop doesn't repair"),
        (ShopError::NothingToRepair(id("x")), "\"x\" needs no repair"),
        (ShopError::NoItem, "no such item"),
        (ShopError::NotAWeapon(id("x")), "\"x\" is not a weapon"),
        (ShopError::NotBought(id("x")), "no shop buys \"x\""),
        (ShopError::UnknownItem(id("x")), "unknown item \"x\""),
        (ShopError::NotTaken(id("x")), "this store doesn't buy \"x\""),
    ];
    for (e, text) in cases {
        assert_eq!(e.to_string(), text);
    }
}

// ---- ShopSession ------------------------------------------------------------

#[test]
fn session_buys_into_the_stock() {
    let (t, shop) = (items(), armoury());
    let (mut gold, mut stock) = (1500, Stock::default());
    let mut s = ShopSession::new(&shop, &t, &mut gold, &mut stock);
    assert_eq!(s.shop(), &shop);
    assert_eq!(s.buy(&id("sword")), Ok(1000));
    assert_eq!(s.buy(&id("vest")), Ok(300));
    assert_eq!(
        s.buy(&id("ring")),
        Err(ShopError::NotEnoughGold {
            cost: 999,
            gold: 200
        })
    );
    assert_eq!(s.gold(), 200);
    assert_eq!(s.stock().weapons, [copy("sword", 40)]);
    assert_eq!(s.stock().count(&id("vest")), 1);
    assert_eq!((gold, stock.items.len()), (200, 1));
}

#[test]
fn session_sells_from_the_stock() {
    let (t, shop) = (items(), vendor());
    let mut gold = 0;
    let mut stock = Stock::default();
    stock.add(id("ring"));
    stock.weapons.push(copy("bow", 3));
    let mut s = ShopSession::new(&shop, &t, &mut gold, &mut stock);
    assert_eq!(s.sell_item(&id("ring")), Ok(499));
    assert_eq!(s.sell_item(&id("ring")), Err(ShopError::NoItem));
    assert_eq!(s.sell_weapon(1), Err(ShopError::NoItem));
    assert_eq!(s.sell_weapon(0), Ok(240));
    assert_eq!(s.sell_weapon(0), Err(ShopError::NoItem));
    assert_eq!((gold, stock), (739, Stock::default()));
}

#[test]
fn session_blacksmith_repairs_and_doesnt_buy() {
    let (t, shop) = (items(), smith());
    let mut gold = 300;
    let mut stock = Stock::default();
    stock.add(id("ring"));
    stock.weapons.push(copy("sword", 20));
    let mut s = ShopSession::new(&shop, &t, &mut gold, &mut stock);
    assert_eq!(s.sell_item(&id("ring")), Err(ShopError::DoesNotBuy));
    assert_eq!(s.sell_weapon(0), Err(ShopError::DoesNotBuy));
    assert_eq!(
        s.buy(&id("sword")),
        Err(ShopError::NotSoldHere(id("sword")))
    );
    assert_eq!(s.repair_weapon(1), Err(ShopError::NoItem));
    assert_eq!(s.repair_weapon(0), Ok(250));
    assert_eq!(
        s.repair_weapon(0),
        Err(ShopError::NothingToRepair(id("sword")))
    );
    assert_eq!(gold, 50);
    assert_eq!(stock.weapons, [copy("sword", 40)]);
    assert_eq!(stock.count(&id("ring")), 1);
}

/// Every item in `stock`, weapons included.
fn held(stock: &Stock) -> u32 {
    stock.items.values().sum::<u32>() + u32::try_from(stock.weapons.len()).unwrap()
}

#[derive(Debug, Clone)]
enum Op {
    Buy(&'static str),
    SellItem(&'static str),
    SellWeapon(usize),
    Repair(usize),
}

fn arb_op() -> impl Strategy<Value = Op> {
    let names = || prop::sample::select(vec!["sword", "bow", "vest", "ring", "potion", "ghost"]);
    prop_oneof![
        names().prop_map(Op::Buy),
        names().prop_map(Op::SellItem),
        (0usize..4).prop_map(Op::SellWeapon),
        (0usize..4).prop_map(Op::Repair),
    ]
}

proptest! {
    #[test]
    fn random_shopping_conserves_gold_and_items(
        start in 0u32..5000,
        kind in prop::sample::select(vec![
            ShopKind::Armoury, ShopKind::Vendor, ShopKind::Blacksmith, ShopKind::Alchemist, ShopKind::Tailor,
        ]),
        worn in prop::collection::vec(0u32..=40, 0..3),
        ops in prop::collection::vec(arb_op(), 0..40),
    ) {
        let t = items();
        let shop = Shop {
            kind,
            stock: if kind == ShopKind::Blacksmith {
                vec![]
            } else {
                ["sword", "bow", "vest", "ring", "potion"].map(id).to_vec()
            },
        };
        let mut gold = start;
        let mut stock = Stock {
            weapons: worn.iter().map(|&l| copy("sword", l)).collect(),
            ..Stock::default()
        };
        let initial = held(&stock);
        let (mut bought, mut sold, mut spent, mut earned) = (0u32, 0u32, 0u64, 0u64);
        let mut s = ShopSession::new(&shop, &t, &mut gold, &mut stock);
        for op in ops {
            let before = (s.gold(), s.stock().clone());
            let result = match op {
                Op::Buy(i) => s.buy(&id(i)).map(|p| { bought += 1; spent += u64::from(p); }),
                Op::SellItem(i) => s.sell_item(&id(i)).map(|p| { sold += 1; earned += u64::from(p); }),
                Op::SellWeapon(i) => s.sell_weapon(i).map(|p| { sold += 1; earned += u64::from(p); }),
                Op::Repair(i) => s.repair_weapon(i).map(|p| spent += u64::from(p)),
            };
            if result.is_err() {
                prop_assert_eq!((s.gold(), s.stock().clone()), before);
            }
            prop_assert_eq!(u64::from(s.gold()) + spent, u64::from(start) + earned);
            prop_assert_eq!(held(s.stock()) + sold, initial + bought);
        }
    }
}

#[test]
fn the_basic_shop_sells_what_the_chapter_unlocked() {
    let basic = BasicStock {
        items: vec![
            (ItemId("potion".into()), Unlock::Chapter(2)),
            (ItemId("sword".into()), Unlock::Chapter(2)),
            (ItemId("bow".into()), Unlock::Chapter(5)),
            (ItemId("vest".into()), Unlock::Chapter(8)),
        ],
    };
    let ids = |chapter| -> Vec<String> {
        let shop = basic.shop(chapter);
        assert_eq!(shop.kind, ShopKind::Basic);
        shop.stock.into_iter().map(|i| i.0).collect()
    };
    assert_eq!(ids(1), [] as [String; 0]);
    assert_eq!(ids(2), ["potion", "sword"]);
    assert_eq!(ids(4), ["potion", "sword"]);
    assert_eq!(ids(5), ["potion", "sword", "bow"]);
    assert_eq!(ids(8), ["potion", "sword", "bow", "vest"]);
    assert!(Unlock::Chapter(3).reached(3) && !Unlock::Chapter(3).reached(2));
}

#[test]
fn the_basic_shop_sells_gear_and_consumables_buys_and_repairs() {
    let items = items();
    let def = |i: &str| items.get(&ItemId(i.into())).cloned().unwrap();
    let basic = ShopKind::Basic;
    for (item, sold) in [
        ("sword", true),
        ("vest", true),
        ("potion", true),
        ("ring", false),
    ] {
        assert_eq!(basic.stocks(&def(item)), sold, "{item}");
    }
    assert!(basic.buys() && basic.repairs());
    assert!(ShopKind::Blacksmith.repairs());
    assert!(!ShopKind::Armoury.repairs() && !ShopKind::Vendor.repairs());
    let shop = Shop {
        kind: basic,
        stock: vec![],
    };
    let mut gold = 300;
    let mut copy = WeaponInstance {
        def: ItemId("sword".into()),
        durability_left: 20,
    };
    // ceil(1000 / 2 × 20 / 40) = 250.
    assert_eq!(repair(&shop, &items, &mut gold, &mut copy), Ok(250));
    assert_eq!((gold, copy.durability_left), (50, 40));
}

/// Ticket 0329: no shop stocks a special weapon; it sells for half its
/// price and repairs by the usual formula, like any weapon.
#[test]
fn special_weapons_are_never_stocked_but_sell_and_repair() {
    let ItemDef::Weapon(mut def) = weapon(3000, 30) else {
        unreachable!("weapon() is a weapon");
    };
    def.special = Some(crate::item::SpecialWeapon::default());
    let special = ItemDef::Weapon(def.clone());
    for kind in [
        ShopKind::Armoury,
        ShopKind::Vendor,
        ShopKind::Blacksmith,
        ShopKind::Basic,
    ] {
        assert!(!kind.stocks(&special), "{kind:?}");
    }
    assert_eq!(sell_price(ShopKind::Armoury, &special), 1500);
    let copy = WeaponInstance {
        def: id("moonblade"),
        durability_left: 15,
    };
    assert_eq!(repair_cost(&def, &copy), 750);
}

// ---- Specialty stores (ticket 0320) -----------------------------------------

fn alchemist() -> Shop {
    shop(ShopKind::Alchemist, &["potion"])
}

fn tailor() -> Shop {
    shop(ShopKind::Tailor, &["robe", "vest"])
}

#[test]
fn an_alchemist_pays_full_for_a_potion_and_refuses_a_sword() {
    let t = items();
    let mut gold = 0;
    assert_eq!(sell(&alchemist(), &t, &mut gold, &id("potion")), Ok(300));
    assert_eq!(
        sell(&alchemist(), &t, &mut gold, &id("sword")),
        Err(ShopError::NotTaken(id("sword")))
    );
    assert_eq!(gold, 300);
    // A vendor pays half for the same potion.
    assert_eq!(sell(&vendor(), &t, &mut gold, &id("potion")), Ok(150));
}

#[test]
fn a_tailor_pays_full_for_cloth_and_leather_and_refuses_plate() {
    let t = items();
    let mut gold = 0;
    assert_eq!(sell(&tailor(), &t, &mut gold, &id("robe")), Ok(120));
    assert_eq!(sell(&tailor(), &t, &mut gold, &id("vest")), Ok(300));
    for refused in ["plate", "potion", "ring", "sword"] {
        assert_eq!(
            sell(&tailor(), &t, &mut gold, &id(refused)),
            Err(ShopError::NotTaken(id(refused))),
            "{refused}"
        );
    }
    // A seal is still refused as no shop buys it.
    assert_eq!(
        sell(&tailor(), &t, &mut gold, &id("seal")),
        Err(ShopError::NotBought(id("seal")))
    );
    assert_eq!(gold, 420);
}

#[test]
fn specialty_stores_sell_and_take_only_their_goods() {
    let t = items();
    let def = |s: &str| t.get(&id(s)).unwrap();
    for (item, alchemist, tailor) in [
        ("potion", true, false),
        ("robe", false, true),
        ("vest", false, true),
        ("plate", false, false),
        ("sword", false, false),
        ("ring", false, false),
        ("seal", false, false),
    ] {
        for (kind, goods) in [(ShopKind::Alchemist, alchemist), (ShopKind::Tailor, tailor)] {
            assert_eq!(kind.stocks(def(item)), goods, "{kind:?} stocks {item}");
            assert_eq!(kind.takes(def(item)), goods, "{kind:?} takes {item}");
        }
    }
    for kind in [ShopKind::Alchemist, ShopKind::Tailor] {
        assert!(
            kind.is_specialty() && kind.buys() && !kind.repairs(),
            "{kind:?}"
        );
    }
    // The other kinds take anything but a seal; a blacksmith nothing.
    for kind in [ShopKind::Armoury, ShopKind::Vendor, ShopKind::Basic] {
        assert!(!kind.is_specialty());
        assert!(
            kind.takes(def("plate")) && !kind.takes(def("seal")),
            "{kind:?}"
        );
    }
    assert!(!ShopKind::Blacksmith.takes(def("sword")));
}

#[test]
fn a_tailor_session_refuses_what_it_wont_take() {
    let (t, shop) = (items(), tailor());
    let mut gold = 0;
    let mut stock = Stock::default();
    stock.add(id("plate"));
    stock.add(id("robe"));
    stock.weapons.push(copy("sword", 40));
    let mut s = ShopSession::new(&shop, &t, &mut gold, &mut stock);
    assert_eq!(
        s.sell_item(&id("plate")),
        Err(ShopError::NotTaken(id("plate")))
    );
    assert_eq!(s.sell_weapon(0), Err(ShopError::NotTaken(id("sword"))));
    assert_eq!(s.sell_item(&id("robe")), Ok(120));
    assert_eq!(
        s.buy(&id("vest")),
        Err(ShopError::NotEnoughGold {
            cost: 300,
            gold: 120
        })
    );
    assert_eq!(gold, 120);
    assert_eq!((stock.count(&id("plate")), stock.weapons.len()), (1, 1));
}

proptest! {
    #[test]
    fn no_shop_pays_more_than_the_price(
        price in 0u32..=u32::MAX,
        kind in prop::sample::select(vec![
            ShopKind::Armoury, ShopKind::Vendor, ShopKind::Blacksmith, ShopKind::Basic,
            ShopKind::Alchemist, ShopKind::Tailor,
        ]),
        weight in prop::sample::select(vec![
            ArmourWeight::Cloth, ArmourWeight::Leather, ArmourWeight::Chain, ArmourWeight::Plate,
        ]),
    ) {
        let potion = ItemDef::Consumable(ConsumableDef {
            name: "p".into(),
            effect: ConsumableEffect::Heal(1),
            price,
        });
        for item in [weapon(price, 1), armour(weight, price), potion] {
            prop_assert!(sell_price(kind, &item) <= item.price());
        }
    }
}
