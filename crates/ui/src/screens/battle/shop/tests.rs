//! Tests of the shop screen and the `Shop`, `Open` and `Evacuate` entries
//! (ticket 0409), on the Quick Battle's map (`test_small`: an Armoury at
//! (5, 3), a Vendor at (6, 3), a Blacksmith at (7, 3), a chest of 500
//! gold at (8, 3) and one with an Elixir at (9, 3)) and the village test
//! battle.

use insta::assert_snapshot;
use trpg_core::{
    BattlePack, BattleSetup, BattleState, GameMode, HouseState, ItemId, LeadGender, LeadProfile,
    Objective, UnitId,
};

use super::*;
use crate::color::Rgb;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::glyph_buffer::{Cell, GlyphBuffer};
use crate::screen::tests::ctx;
use crate::screens::battle::mode::{MenuEntry, Mode};
use crate::screens::battle::testing::{Stacked, Values as _, quick_units, setup};
use crate::screens::battle::{BattleScreen, TOAST_S};
use crate::words::Words;
use trpg_content::battle_campaign;
use trpg_core::lead::DEFAULT_NAME;

const ARMOURY: Pos = Pos::new(5, 3);
const VENDOR: Pos = Pos::new(6, 3);
const BLACKSMITH: Pos = Pos::new(7, 3);
const GOLD_CHEST: Pos = Pos::new(8, 3);
const ELIXIR_CHEST: Pos = Pos::new(9, 3);

/// The lord (unit 1).
const LORD: UnitId = UnitId(1);

/// The Quick Battle with the lord standing on `at` and the party holding
/// `gold`; the brigand that stood on (8, 2), over the chests, moved off
/// to (12, 6).
fn battle_at(at: Pos, gold: Gold) -> BattleState {
    worn_at(at, gold, None)
}

/// [`battle_at`], the lord's first weapon with `left` durability left if
/// given.
fn worn_at(at: Pos, gold: Gold, left: Option<u32>) -> BattleState {
    packed_at(at, gold, left, 0)
}

/// [`worn_at`], with `potions` Potions in the battle pack.
fn packed_at(at: Pos, gold: Gold, left: Option<u32>, potions: usize) -> BattleState {
    let c = ctx();
    let (map, mut units) = quick_units(&c);
    units[0].pos = at;
    if let Some(left) = left
        && let Some(copy) = units[0].loadout.weapons[0].as_mut()
    {
        copy.durability_left = left;
    }
    for u in &mut units {
        if u.pos == Pos::new(8, 2) {
            u.pos = Pos::new(12, 6);
        }
    }
    let rout = Objective::Rout { turn_limit: None };
    let pack = BattlePack {
        items: vec![ItemId::new("potion"); potions],
        cap: potions,
    };
    BattleState::new(BattleSetup {
        gold,
        pack,
        ..setup(&c, map, units, rout)
    })
    .0
}

fn press(s: &mut Stacked, c: &mut Ctx, actions: &[Action]) {
    s.press(c, actions);
}

/// The battle `state` with the lord's action menu open where it stands.
fn menu_open(c: &mut Ctx, state: BattleState) -> Stacked {
    let mut s = Stacked::new(BattleScreen::new(state));
    press(&mut s, c, &[]);
    while s.shown_tip().is_some() {
        press(&mut s, c, &[Action::Confirm]);
    }
    press(&mut s, c, &[Action::Confirm, Action::Confirm]);
    s.frame(c, &[], 0.5);
    assert!(
        matches!(s.mode(), Mode::ActionMenu { .. }),
        "{:?}",
        s.mode()
    );
    s
}

/// The action menu's entries.
fn entries(s: &Stacked) -> Vec<MenuEntry> {
    let Mode::ActionMenu { menu, .. } = s.mode() else {
        panic!("{:?}", s.mode());
    };
    menu.values()
}

/// Moves the action menu's focus to `entry` and chooses it.
fn choose(s: &mut Stacked, c: &mut Ctx, entry: MenuEntry) {
    for _ in 0..entries(s).len() {
        let Mode::ActionMenu { menu, .. } = s.mode() else {
            panic!("{:?}", s.mode());
        };
        if menu.value(menu.focus()) == Some(&entry) {
            press(s, c, &[Action::Confirm]);
            return;
        }
        press(s, c, &[Action::CursorDown]);
    }
    panic!("no {entry:?} in {:?}", entries(s));
}

/// The lord's shop screen opened on `at`.
fn shop_open(c: &mut Ctx, at: Pos, gold: Gold) -> Stacked {
    let mut s = menu_open(c, battle_at(at, gold));
    choose(&mut s, c, MenuEntry::Shop);
    assert!(s.shop().is_some(), "{:?}", s.mode());
    s
}

fn price(c: &Ctx, item: &str) -> Gold {
    c.game
        .content
        .items
        .get(&ItemId::new(item))
        .unwrap()
        .price()
}

fn lord(s: &Stacked) -> &trpg_core::Unit {
    s.state().unit(LORD).unwrap()
}

#[test]
fn gold_is_shown_with_its_thousands_apart() {
    assert_eq!(gold_text(0), "0");
    assert_eq!(gold_text(999), "999");
    assert_eq!(gold_text(1250), "1 250");
    assert_eq!(gold_text(1_000_000), "1 000 000");
}

#[test]
fn the_menu_offers_shop_open_and_evacuate_only_where_they_apply() {
    let mut c = ctx();
    let s = menu_open(&mut c, battle_at(VENDOR, 0));
    assert!(entries(&s).contains(&MenuEntry::Shop));
    assert!(!entries(&s).contains(&MenuEntry::Open));
    let s = menu_open(&mut c, battle_at(GOLD_CHEST, 0));
    assert!(entries(&s).contains(&MenuEntry::Open));
    assert!(!entries(&s).contains(&MenuEntry::Shop));
    // On a plain tile: none of them.
    let s = menu_open(&mut c, battle_at(Pos::new(4, 3), 0));
    for e in [MenuEntry::Shop, MenuEntry::Open, MenuEntry::Evacuate] {
        assert!(!entries(&s).contains(&e), "{e:?}");
    }
}

/// Acceptance: a Potion bought on a Vendor tile costs its price, goes
/// into the pack and ends the unit's action.
#[test]
fn buying_a_potion_pays_fills_the_pack_and_ends_the_action() {
    let mut c = ctx();
    let mut s = shop_open(&mut c, VENDOR, 1000);
    let pack = s.state().pack().items.len();
    let shop = s.shop().unwrap();
    assert_eq!(shop.tab(), Tab::Buy);
    let view = shop.view(&c);
    assert_eq!(view.shop, "Vendor");
    assert_eq!(view.gold, "Gold 1 000");
    assert_eq!(view.tabs, ["Buy", "Sell"]);
    assert_eq!(view.lines[0].name, "Potion");
    assert_eq!(view.lines[0].gold, gold_text(price(&c, "potion")));
    assert!(view.lines[0].enabled);
    press(&mut s, &mut c, &[Action::Confirm]);
    // Not sent yet: the screen shows the gold after it.
    assert_eq!(s.state().gold(), 1000);
    let left = 1000 - price(&c, "potion");
    assert_eq!(s.shop().unwrap().gold(), left);
    press(&mut s, &mut c, &[Action::Cancel]);
    assert!(s.shop().is_none());
    assert_eq!(s.state().gold(), left);
    assert_eq!(s.state().pack().items.len(), pack + 1);
    assert!(lord(&s).acted);
    assert!(matches!(s.mode(), Mode::Idle { .. }), "{:?}", s.mode());
}

/// Acceptance: leaving without buying sends nothing; the unit hasn't
/// acted and its action menu is back.
#[test]
fn leaving_with_nothing_chosen_sends_nothing() {
    let mut c = ctx();
    let mut s = shop_open(&mut c, VENDOR, 1000);
    press(&mut s, &mut c, &[Action::Cancel]);
    assert!(s.shop().is_none());
    assert_eq!(s.state().gold(), 1000);
    assert!(!lord(&s).acted);
    assert_eq!(s.history().commands(), []);
    assert!(entries(&s).contains(&MenuEntry::Shop));
}

#[test]
fn what_the_party_cant_afford_is_dimmed_and_refused() {
    let mut c = ctx();
    let mut s = shop_open(&mut c, ARMOURY, 0);
    let view = s.shop().unwrap().view(&c);
    assert_eq!(view.shop, "Armory");
    assert!(view.lines.iter().all(|l| !l.enabled), "{:?}", view.lines);
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.shop().unwrap().visit().txns, Vec::<ShopTxn>::new());
    // Each item's numbers or bonuses.
    assert!(view.lines[0].detail.starts_with("Mt "), "{:?}", view.lines);
}

#[test]
fn several_buys_go_in_one_shop_action() {
    let mut c = ctx();
    let mut s = shop_open(&mut c, VENDOR, 1000);
    press(&mut s, &mut c, &[Action::Confirm, Action::Confirm]);
    let txns = s.shop().unwrap().visit().txns.clone();
    assert_eq!(txns.len(), 2);
    press(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(s.history().commands().len(), 1);
    assert_eq!(s.state().gold(), 1000 - 2 * price(&c, "potion"));
}

#[test]
fn the_sell_tab_pays_half_and_lists_the_pack_once_per_item() {
    let mut c = ctx();
    let mut s = shop_open(&mut c, VENDOR, 0);
    press(&mut s, &mut c, &[Action::CursorRight]);
    let shop = s.shop().unwrap();
    assert_eq!(shop.tab(), Tab::Sell);
    let offers = shop.offers();
    let weapon = lord(&s).loadout.weapon(0).unwrap().def.clone();
    assert_eq!(offers[0].item, weapon);
    assert_eq!(offers[0].gold, price(&c, &weapon.0) / 2);
    let potions = offers.iter().filter(|o| o.item.0 == "potion").count();
    assert!(potions <= 1, "{offers:?}");
    // A weapon: asked about, one only.
    press(&mut s, &mut c, &[Action::Confirm]);
    let sale = s.shop().unwrap().selling();
    assert_eq!(sale, Some(SellConfirm { line: 0, count: 1 }));
    press(&mut s, &mut c, &[Action::CursorUp]);
    assert_eq!(s.shop().unwrap().selling().unwrap().count, 1);
    press(&mut s, &mut c, &[Action::Confirm, Action::Cancel]);
    assert_eq!(s.state().gold(), price(&c, &weapon.0) / 2);
    assert!(lord(&s).loadout.weapon(0).is_none());
}

/// Acceptance: a worn weapon repaired at a Blacksmith.
#[test]
fn a_blacksmith_repairs_a_worn_weapon() {
    let mut c = ctx();
    let weapon = battle_at(BLACKSMITH, 0).units()[0].loadout.weapons[0].clone();
    let weapon = weapon.unwrap().def;
    let full = c.game.content.items.weapon(&weapon).unwrap().durability;
    let mut s = menu_open(&mut c, worn_at(BLACKSMITH, 1000, Some(full - 5)));
    choose(&mut s, &mut c, MenuEntry::Shop);
    let shop = s.shop().unwrap();
    assert_eq!(shop.tab(), Tab::Repair);
    let view = shop.view(&c);
    assert_eq!(view.tabs, ["Repair"]);
    let line = &view.lines[0];
    assert_eq!(line.detail, format!("{}/{full} → {full}/{full}", full - 5));
    let cost: Gold = line.gold.replace(' ', "").parse().unwrap();
    assert!(cost > 0);
    press(&mut s, &mut c, &[Action::Confirm]);
    // Repaired: the line is gone.
    assert_eq!(s.shop().unwrap().offers(), Vec::<Offer>::new());
    press(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(s.state().gold(), 1000 - cost);
    assert_eq!(lord(&s).loadout.weapon(0).unwrap().durability_left, full);
    assert!(lord(&s).acted);
}

/// Acceptance: a chest opened says what it gave.
#[test]
fn opening_a_chest_says_what_was_inside() {
    let mut c = ctx();
    let mut s = menu_open(&mut c, battle_at(GOLD_CHEST, 100));
    choose(&mut s, &mut c, MenuEntry::Open);
    assert_eq!(s.state().gold(), 600);
    assert_eq!(s.toast(), Some("Got 500 gold."));
    assert!(lord(&s).acted);
    let mut s = menu_open(&mut c, battle_at(ELIXIR_CHEST, 0));
    let pack = s.state().pack().items.len();
    choose(&mut s, &mut c, MenuEntry::Open);
    assert_eq!(s.toast(), Some("Got Elixir."));
    assert_eq!(s.state().pack().items.len(), pack + 1);
    // It fades.
    s.frame(&mut c, &[], TOAST_S + 0.1);
    assert_eq!(s.toast(), None);
}

/// Acceptance: a house evacuated says so, and the objective page counts
/// it.
#[test]
fn evacuating_a_house_saves_it() {
    let mut c = ctx();
    let content = &c.game.content.clone();
    let def = &content.battles["test_village"];
    let lead = LeadProfile::new(DEFAULT_NAME, LeadGender::Male);
    let campaign = battle_campaign(content, def, GameMode::Classic, lead);
    let mut setup = campaign.battle_setup(def, &content.tables());
    let village = setup.village.clone().unwrap();
    let house = village.houses[0];
    let lead = setup.units.iter_mut().find(|u| u.is_lord).unwrap();
    lead.pos = house;
    let (state, _) = BattleState::new(setup);
    let id = state.units().iter().find(|u| u.is_lord).unwrap().id;
    let mut s = menu_open(&mut c, state);
    let line = |s: &Stacked| super::super::loot::village_text(s.state(), Words::ENGLISH);
    let total = village.houses.len();
    let before = format!(
        "Save at least {} of {total} houses (0 saved, 0 lost)",
        village.minimum
    );
    assert_eq!(line(&s).as_deref(), Some(before.as_str()));
    choose(&mut s, &mut c, MenuEntry::Evacuate);
    assert_eq!(s.toast(), Some("House saved."));
    let saved = s.state().village().unwrap().house(house);
    assert_eq!(saved, Some(HouseState::Saved));
    assert!(s.state().unit(id).unwrap().acted);
    let after = format!(
        "Save at least {} of {total} houses (1 saved, 0 lost)",
        village.minimum
    );
    assert_eq!(line(&s).as_deref(), Some(after.as_str()));
}

#[test]
fn the_side_panel_shows_the_partys_gold() {
    let mut c = ctx();
    let s = menu_open(&mut c, battle_at(VENDOR, 1250));
    assert!(s.view(&c).shows("Gold 1 250"));
}

fn render(s: &Stacked, c: &Ctx) -> GlyphBuffer {
    let blank = Cell::new(' ', Rgb::new(0, 0, 0), Rgb::new(0, 0, 0));
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, blank);
    s.paint(c, &mut buf);
    buf
}

#[test]
fn the_shop_renders() {
    let mut c = ctx();
    let s = shop_open(&mut c, ARMOURY, 1250);
    assert_snapshot!("armoury_buy", render(&s, &c).to_snapshot(&c.palette));
}

#[test]
fn the_blacksmith_renders() {
    let mut c = ctx();
    let mut s = menu_open(&mut c, worn_at(BLACKSMITH, 1250, Some(12)));
    choose(&mut s, &mut c, MenuEntry::Shop);
    assert_snapshot!("blacksmith_repair", render(&s, &c).to_snapshot(&c.palette));
}

/// The Vendor's Sell tab with 3 Potions in the pack, the Potions'
/// confirmation open.
fn selling_potions(c: &mut Ctx) -> Stacked {
    let mut s = menu_open(c, packed_at(VENDOR, 0, None, 3));
    choose(&mut s, c, MenuEntry::Shop);
    press(&mut s, c, &[Action::CursorRight]);
    let offers = s.shop().unwrap().offers();
    let line = offers.iter().position(|o| o.item.0 == "potion").unwrap();
    assert_eq!(offers[line].count, 3);
    for _ in 0..line {
        press(&mut s, c, &[Action::CursorDown]);
    }
    press(&mut s, c, &[Action::Confirm]);
    assert!(s.shop().unwrap().selling().is_some());
    s
}

/// Nick (2026-10-28): selling asks first, with up and down for how many.
#[test]
fn up_and_down_set_how_many_to_sell_between_one_and_all() {
    let mut c = ctx();
    let mut s = selling_potions(&mut c);
    let count = |s: &Stacked| s.shop().unwrap().selling().unwrap().count;
    assert_eq!(count(&s), 1);
    press(&mut s, &mut c, &[Action::CursorDown]);
    assert_eq!(count(&s), 1);
    press(
        &mut s,
        &mut c,
        &[Action::CursorUp, Action::CursorUp, Action::CursorUp],
    );
    assert_eq!(count(&s), 3);
    press(&mut s, &mut c, &[Action::CursorDown]);
    assert_eq!(count(&s), 2);
    let view = s.shop().unwrap().view(&c);
    let sell = view.selling.unwrap();
    assert_eq!(sell.question, "Sell Potion?");
    assert_eq!((sell.count, sell.max), (2, 3));
    let each = price(&c, "potion") / 2;
    assert_eq!(sell.gold, format!("For {} gold", gold_text(2 * each)));
    // Nothing is sold until Confirm.
    assert_eq!(s.shop().unwrap().visit().txns, Vec::<ShopTxn>::new());
    press(&mut s, &mut c, &[Action::Confirm]);
    let shop = s.shop().unwrap();
    assert_eq!(shop.selling(), None);
    assert_eq!(shop.visit().txns.len(), 2);
    assert_eq!(shop.gold(), 2 * each);
    press(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(s.state().pack().items, [ItemId::new("potion")]);
    assert_eq!(s.state().gold(), 2 * each);
    assert!(lord(&s).acted);
}

#[test]
fn cancel_in_the_confirmation_sells_nothing() {
    let mut c = ctx();
    let mut s = selling_potions(&mut c);
    press(&mut s, &mut c, &[Action::CursorUp, Action::Cancel]);
    let shop = s.shop().expect("still in the shop");
    assert_eq!(shop.selling(), None);
    assert_eq!(shop.visit().txns, Vec::<ShopTxn>::new());
    press(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(s.state().pack().items.len(), 3);
    assert!(!lord(&s).acted);
}

#[test]
fn the_sale_renders() {
    let mut c = ctx();
    let mut s = selling_potions(&mut c);
    press(&mut s, &mut c, &[Action::CursorUp]);
    assert_snapshot!("sell_confirm", render(&s, &c).to_snapshot(&c.palette));
}

#[test]
fn up_and_down_move_through_the_lines_and_wrap() {
    let mut c = ctx();
    let mut s = shop_open(&mut c, ARMOURY, 0);
    let focus = |s: &Stacked| s.shop().unwrap().focus();
    assert_eq!(focus(&s), 0);
    press(&mut s, &mut c, &[Action::CursorDown]);
    assert_eq!(focus(&s), 1);
    press(&mut s, &mut c, &[Action::CursorUp, Action::CursorUp]);
    assert_eq!(focus(&s), 2);
    assert_eq!(s.shop().unwrap().view(&c).focus, 2);
}

#[test]
fn left_and_right_switch_tabs_and_a_blacksmith_has_one() {
    let mut c = ctx();
    let mut s = shop_open(&mut c, VENDOR, 0);
    let mut shop = s.shop().unwrap().clone();
    assert_eq!(shop.step(Action::CursorLeft), ShopEffect::Moved);
    assert_eq!(shop.tab(), Tab::Sell);
    assert_eq!(shop.step(Action::CursorRight), ShopEffect::Moved);
    assert_eq!(shop.tab(), Tab::Buy);
    assert_eq!(shop.step(Action::CursorRight), ShopEffect::Moved);
    assert_eq!(shop.tab(), Tab::Sell);
    let mut s2 = menu_open(&mut c, worn_at(BLACKSMITH, 0, Some(3)));
    choose(&mut s2, &mut c, MenuEntry::Shop);
    let mut smith = s2.shop().unwrap().clone();
    assert_eq!(smith.step(Action::CursorLeft), ShopEffect::None);
    assert_eq!(smith.step(Action::CursorRight), ShopEffect::None);
    assert_eq!(smith.tab(), Tab::Repair);
    press(&mut s, &mut c, &[Action::Cancel]);
}

#[test]
fn a_line_names_how_many_only_past_one_and_seals_are_never_listed() {
    let mut c = ctx();
    // A seal in the pack: no shop buys it.
    let units = battle_at(VENDOR, 0).units().to_vec();
    let (map, _) = quick_units(&c);
    let rout = Objective::Rout { turn_limit: None };
    let pack = BattlePack {
        items: ["potion", "tier_2_seal", "potion"]
            .map(ItemId::new)
            .to_vec(),
        cap: 3,
    };
    let state = BattleState::new(BattleSetup {
        pack,
        ..setup(&c, map, units, rout)
    })
    .0;
    let mut s = menu_open(&mut c, state);
    choose(&mut s, &mut c, MenuEntry::Shop);
    press(&mut s, &mut c, &[Action::CursorRight]);
    let view = s.shop().unwrap().view(&c);
    let names: Vec<&str> = view.lines.iter().map(|l| l.name.as_str()).collect();
    assert!(names.contains(&"Potion ×2"), "{names:?}");
    assert!(!names.iter().any(|n| n.contains("Seal")), "{names:?}");
    // A single item has no count.
    assert!(!names[0].contains('×'), "{names:?}");
}

#[test]
fn the_screen_is_named_shop() {
    let mut c = ctx();
    let s = shop_open(&mut c, VENDOR, 0);
    let shop = s.shop().unwrap();
    assert_eq!(crate::screen::Screen::name(shop), "shop");
}

#[test]
fn confirming_a_sale_sells_and_cancel_goes_back() {
    let mut c = ctx();
    let s = selling_potions(&mut c);
    let mut shop = s.shop().unwrap().clone();
    assert_eq!(shop.step(Action::CursorDown), ShopEffect::None);
    assert_eq!(shop.step(Action::CursorUp), ShopEffect::Moved);
    assert_eq!(shop.step(Action::CursorLeft), ShopEffect::None);
    let mut back = shop.clone();
    assert_eq!(back.step(Action::Cancel), ShopEffect::Back);
    assert_eq!(back.selling(), None);
    assert_eq!(shop.step(Action::Confirm), ShopEffect::Chosen);
    assert_eq!(shop.visit().txns.len(), 2);
    // Back on the Sell list: Confirm asks again.
    assert_eq!(shop.step(Action::Confirm), ShopEffect::Asked);
}

#[test]
fn a_buy_is_chosen_and_one_the_party_cant_pay_is_denied() {
    let mut c = ctx();
    let s = shop_open(&mut c, VENDOR, 1000);
    let mut shop = s.shop().unwrap().clone();
    assert_eq!(shop.step(Action::Confirm), ShopEffect::Chosen);
    assert_eq!(shop.visit().txns.len(), 1);
    let s = shop_open(&mut c, VENDOR, 0);
    let mut broke = s.shop().unwrap().clone();
    assert_eq!(broke.step(Action::Confirm), ShopEffect::Denied);
    assert_eq!(broke.visit().txns, Vec::<ShopTxn>::new());
}

/// Selling two of the second item in the pack takes those two, never
/// the first item's copy.
#[test]
fn selling_several_takes_only_that_items_copies() {
    let mut c = ctx();
    let units = battle_at(VENDOR, 0).units().to_vec();
    let (map, _) = quick_units(&c);
    let rout = Objective::Rout { turn_limit: None };
    let pack = BattlePack {
        items: ["elixir", "potion", "potion", "potion"]
            .map(ItemId::new)
            .to_vec(),
        cap: 4,
    };
    let state = BattleState::new(BattleSetup {
        pack,
        ..setup(&c, map, units, rout)
    })
    .0;
    let mut s = menu_open(&mut c, state);
    choose(&mut s, &mut c, MenuEntry::Shop);
    press(&mut s, &mut c, &[Action::CursorRight]);
    let offers = s.shop().unwrap().offers();
    let line = offers.iter().position(|o| o.item.0 == "potion").unwrap();
    for _ in 0..line {
        press(&mut s, &mut c, &[Action::CursorDown]);
    }
    press(
        &mut s,
        &mut c,
        &[Action::Confirm, Action::CursorUp, Action::Confirm],
    );
    press(&mut s, &mut c, &[Action::Cancel]);
    let left = ["elixir", "potion"].map(ItemId::new);
    assert_eq!(s.state().pack().items, left);
}

// ---- Special weapons (0329) --------------------------------------------------

/// The Vendor's Sell tab with the lord also carrying the Moonblade (its
/// second weapon, the second line), the Moonblade's question open.
fn selling_the_moonblade(c: &mut Ctx) -> Stacked {
    let (map, mut units) = quick_units(c);
    units[0].pos = VENDOR;
    units[0].loadout.weapons[1] = Some(trpg_core::WeaponInstance {
        def: ItemId::new("moonblade"),
        durability_left: 30,
    });
    for u in &mut units {
        if u.pos == Pos::new(8, 2) {
            u.pos = Pos::new(12, 6);
        }
    }
    let rout = Objective::Rout { turn_limit: None };
    let state = BattleState::new(setup(c, map, units, rout)).0;
    let mut s = menu_open(c, state);
    choose(&mut s, c, MenuEntry::Shop);
    press(&mut s, c, &[Action::CursorRight]);
    let offers = s.shop().unwrap().offers();
    assert_eq!(offers[1].item, ItemId::new("moonblade"));
    press(&mut s, c, &[Action::CursorDown, Action::Confirm]);
    s
}

/// `combat-arts.md`, *Special weapons*: selling one asks first, with No
/// and Yes, the cursor on No.
#[test]
fn selling_a_special_weapon_asks_first_on_no() {
    let mut c = ctx();
    let s = selling_the_moonblade(&mut c);
    let shop = s.shop().unwrap();
    assert_eq!(shop.selling_special(), Some(1));
    assert_eq!(shop.selling(), None);
    let view = shop.view(&c);
    let ask = view.special.unwrap();
    assert_eq!(
        ask.text,
        "Sell Moonblade for 1 500? No shop sells it, so it can't be bought back."
    );
    let answers: Vec<_> = ask.answers.iter().map(|a| a.label.as_str()).collect();
    assert_eq!(answers, ["No", "Yes"]);
    assert_eq!(ask.focused(), Some(false));
    assert_eq!(view.help, ctx_help(&c));
}

fn ctx_help(c: &Ctx) -> String {
    c.text_with(HELP_SPECIAL, &[])
}

#[test]
fn only_yes_sells_a_special_weapon() {
    let mut c = ctx();
    let s = selling_the_moonblade(&mut c);
    let base = s.shop().unwrap().clone();
    // Confirm on No, and Cancel, sell nothing.
    for action in [Action::Confirm, Action::Cancel] {
        let mut shop = base.clone();
        assert_eq!(shop.step(action), ShopEffect::Back, "{action:?}");
        assert_eq!(shop.selling_special(), None);
        assert_eq!(shop.visit().txns, Vec::<ShopTxn>::new());
    }
    // To Yes and back to No.
    let mut shop = base.clone();
    assert_eq!(shop.step(Action::CursorRight), ShopEffect::Moved);
    assert_eq!(shop.step(Action::CursorLeft), ShopEffect::Moved);
    assert_eq!(shop.step(Action::Confirm), ShopEffect::Back);
    // Yes sells it, for half its price.
    let mut shop = base;
    assert_eq!(shop.step(Action::CursorRight), ShopEffect::Moved);
    assert_eq!(shop.step(Action::Confirm), ShopEffect::Chosen);
    assert_eq!(shop.visit().txns.len(), 1);
    assert_eq!(shop.gold(), 1500);
    assert!(shop.offers().iter().all(|o| o.item.0 != "moonblade"));
}

#[test]
fn the_special_sale_renders() {
    let mut c = ctx();
    let s = selling_the_moonblade(&mut c);
    assert_snapshot!("sell_special", render(&s, &c).to_snapshot(&c.palette));
}

// ---- Specialty stores (0320) -------------------------------------------------

/// The lord on a Tailor (put on the Vendor's tile for the test: towns,
/// where Tailors stand, come with 1007), wearing a Leather Vest, with two
/// Potions in the pack, its Sell tab open.
fn tailor_sell(c: &mut Ctx) -> Stacked {
    let (mut map, mut units) = quick_units(c);
    let stock = ["linen_robe", "leather_vest"].map(ItemId::new).to_vec();
    let tailor = trpg_core::TileFeature::Shop(Shop {
        kind: ShopKind::Tailor,
        stock,
    });
    map.features.insert(VENDOR, tailor);
    units[0].pos = VENDOR;
    units[0].loadout.armour = Some(ItemId::new("leather_vest"));
    for u in &mut units {
        if u.pos == Pos::new(8, 2) {
            u.pos = Pos::new(12, 6);
        }
    }
    let rout = Objective::Rout { turn_limit: None };
    let pack = BattlePack {
        items: vec![ItemId::new("potion"); 2],
        cap: 2,
    };
    let state = BattleState::new(BattleSetup {
        pack,
        ..setup(c, map, units, rout)
    })
    .0;
    let mut s = menu_open(c, state);
    choose(&mut s, c, MenuEntry::Shop);
    press(&mut s, c, &[Action::CursorRight]);
    assert_eq!(s.shop().unwrap().tab(), Tab::Sell);
    s
}

/// Acceptance: a Tailor's sell list shows what it pays (full price for
/// its goods) and dims what it won't take, saying why.
#[test]
fn a_tailor_pays_full_for_its_goods_and_dims_the_rest_with_the_reason() {
    let mut c = ctx();
    let s = tailor_sell(&mut c);
    let view = s.shop().unwrap().view(&c);
    assert_eq!(view.shop, "Tailor");
    assert_eq!(view.tabs, ["Buy", "Sell"]);
    let line = |name: &str| view.lines.iter().find(|l| l.name == name).unwrap();
    let vest = line("Leather Vest");
    assert!(vest.enabled);
    assert_eq!(vest.gold, gold_text(price(&c, "leather_vest")));
    let why = "The Tailor buys only cloth and leather armor.";
    let potions = line("Potion ×2");
    assert!(!potions.enabled);
    assert_eq!((potions.gold.as_str(), potions.detail.as_str()), ("", why));
    let weapon = &view.lines[0];
    assert!(!weapon.enabled && weapon.detail == why, "{weapon:?}");
    // A refused line can't be chosen.
    let mut shop = s.shop().unwrap().clone();
    assert_eq!(shop.step(Action::Confirm), ShopEffect::Denied);
}

/// A Vendor pays half for the same Leather Vest, and its lines say what
/// the items are.
#[test]
fn a_vendor_pays_half_and_gives_no_reason() {
    let mut c = ctx();
    let (map, mut units) = quick_units(&c);
    units[0].pos = VENDOR;
    units[0].loadout.armour = Some(ItemId::new("leather_vest"));
    let rout = Objective::Rout { turn_limit: None };
    let state = BattleState::new(setup(&c, map, units, rout)).0;
    let mut s = menu_open(&mut c, state);
    choose(&mut s, &mut c, MenuEntry::Shop);
    press(&mut s, &mut c, &[Action::CursorRight]);
    let view = s.shop().unwrap().view(&c);
    let vest = view
        .lines
        .iter()
        .find(|l| l.name == "Leather Vest")
        .unwrap();
    assert_eq!(vest.gold, gold_text(price(&c, "leather_vest") / 2));
    assert!(!vest.detail.contains("buys only"), "{vest:?}");
}

#[test]
fn the_tailor_sell_list_renders() {
    let mut c = ctx();
    let s = tailor_sell(&mut c);
    assert_snapshot!("tailor_sell", render(&s, &c).to_snapshot(&c.palette));
}

/// Each specialty store names its goods when it refuses an item; the
/// other shops give no reason.
#[test]
fn each_specialty_store_says_what_it_buys() {
    let c = ctx();
    let reason = |kind| wont_take_key(kind).map(|k| c.text(k).to_owned());
    assert_eq!(
        reason(ShopKind::Alchemist).as_deref(),
        Some("The Alchemist buys only consumables.")
    );
    assert_eq!(
        reason(ShopKind::Tailor).as_deref(),
        Some("The Tailor buys only cloth and leather armor.")
    );
    for kind in [
        ShopKind::Armoury,
        ShopKind::Vendor,
        ShopKind::Blacksmith,
        ShopKind::Basic,
    ] {
        assert_eq!(reason(kind), None, "{kind:?}");
    }
    assert_eq!(c.text(kind_key(ShopKind::Alchemist)), "Alchemist");
    assert_eq!(Tab::of(ShopKind::Alchemist), [Tab::Buy, Tab::Sell]);
}

#[test]
fn shop_declares_its_actions() {
    let screen = ShopScreen::new(&battle_at(VENDOR, 0), LORD, VENDOR).unwrap();
    crate::screens::actions::assert_declares(&screen, crate::screens::actions::SHOP);
}
