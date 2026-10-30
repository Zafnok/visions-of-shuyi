//! Tests of a town's shop on the test world map's towns: Millbrook's
//! Vendor (potions) and Armory, Kingsgate's Tailor (a leather vest) and
//! Blacksmith.

use insta::assert_snapshot;
use trpg_content::battle_campaign;
use trpg_core::{GameMode, LeadGender, LeadProfile, NodeKind, ShopKind};

use super::*;
use crate::harness::Harness;
use crate::screen::tests::ctx;
use crate::screens::popped_with;

/// The test battle's army with 2 000 gold and two potions in the stock.
fn army(ctx: &Ctx) -> Campaign {
    let content = &ctx.game.content;
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let mut c = battle_campaign(content, &content.battles["test"], GameMode::Classic, lead);
    c.gold = 2000;
    c.stock.add(ItemId::new("potion"));
    c.stock.add(ItemId::new("potion"));
    c
}

/// Town `town`'s shop of `kind` on the test world map.
fn shop(ctx: &Ctx, town: &str, kind: ShopKind) -> Shop {
    let map = &ctx.game.content.worldmaps["test_world"].map;
    let Some(NodeKind::Town { shops, .. }) = map.node(town).map(|n| &n.kind) else {
        panic!("no town {town}");
    };
    shops
        .iter()
        .find(|s| s.kind == kind)
        .cloned()
        .expect("a shop")
}

fn press(s: &mut TownScreen, ctx: &mut Ctx, actions: &[Action]) -> Transition {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    s.update(ctx, &input)
}

#[test]
fn buying_puts_the_item_in_the_stock_at_once() {
    let mut ctx = ctx();
    let vendor = shop(&ctx, "home", ShopKind::Vendor);
    let mut s = TownScreen::new(army(&ctx), vendor, "Millbrook".into());
    press(&mut s, &mut ctx, &[Action::Confirm]);
    assert_eq!(s.campaign().gold, 1700);
    assert_eq!(s.campaign().stock.count(&ItemId::new("potion")), 3);
    let view = s.view(&ctx);
    assert_eq!(view.unit, "Millbrook");
    assert_eq!(view.gold, "Gold 1 700");
}

#[test]
fn selling_asks_how_many_first() {
    let mut ctx = ctx();
    let vendor = shop(&ctx, "home", ShopKind::Vendor);
    let mut s = TownScreen::new(army(&ctx), vendor, "Millbrook".into());
    press(&mut s, &mut ctx, &[Action::CursorRight]);
    assert_eq!(s.tab(), Tab::Sell);
    let view = s.view(&ctx);
    let potion = view
        .lines
        .iter()
        .find(|l| l.name.starts_with("Potion"))
        .expect("potions");
    assert_eq!(potion.name, "Potion ×2");
    let line = view
        .lines
        .iter()
        .position(|l| l.name == potion.name)
        .unwrap_or(0);
    for _ in 0..line {
        press(&mut s, &mut ctx, &[Action::CursorDown]);
    }
    press(&mut s, &mut ctx, &[Action::Confirm, Action::CursorUp]);
    let selling = s.view(&ctx).selling.expect("asked");
    assert_eq!((selling.count, selling.max), (2, 2));
    press(&mut s, &mut ctx, &[Action::CursorDown]);
    assert_eq!(s.view(&ctx).selling.map(|v| v.count), Some(1));
    press(&mut s, &mut ctx, &[Action::CursorUp]);
    press(&mut s, &mut ctx, &[Action::Confirm]);
    assert_eq!(s.campaign().stock.count(&ItemId::new("potion")), 0);
    assert_eq!(s.campaign().gold, 2300);
}

/// Up and down move between the lines: the second one is bought.
#[test]
fn down_then_confirm_buys_the_second_line() {
    let mut ctx = ctx();
    let armory = shop(&ctx, "home", ShopKind::Armoury);
    let mut s = TownScreen::new(army(&ctx), armory, "Millbrook".into());
    press(&mut s, &mut ctx, &[Action::CursorDown, Action::Confirm]);
    assert_eq!(s.campaign().stock.count(&ItemId::new("leather_vest")), 1);
    assert_eq!(s.campaign().stock.weapons.len(), 0);
    press(&mut s, &mut ctx, &[Action::CursorUp, Action::Confirm]);
    assert_eq!(s.campaign().stock.weapons.len(), 1);
}

/// Cancel while a sale is asked about sells nothing and goes back to the
/// list.
#[test]
fn cancel_on_a_sale_sells_nothing() {
    let mut ctx = ctx();
    let vendor = shop(&ctx, "home", ShopKind::Vendor);
    let mut s = TownScreen::new(army(&ctx), vendor, "Millbrook".into());
    press(&mut s, &mut ctx, &[Action::CursorRight]);
    let line = s
        .view(&ctx)
        .lines
        .iter()
        .position(|l| l.name.starts_with("Potion"));
    for _ in 0..line.expect("potions") {
        press(&mut s, &mut ctx, &[Action::CursorDown]);
    }
    press(&mut s, &mut ctx, &[Action::Confirm]);
    assert!(s.view(&ctx).selling.is_some());
    let t = press(&mut s, &mut ctx, &[Action::Cancel]);
    assert!(matches!(t, Transition::None), "the screen stays");
    assert!(s.view(&ctx).selling.is_none());
    assert_eq!(s.campaign().stock.count(&ItemId::new("potion")), 2);
}

/// A weapon of the stock sells with its durability before its numbers;
/// left and right switch the two tabs.
#[test]
fn a_weapon_sells_with_its_durability() {
    let mut ctx = ctx();
    let armory = shop(&ctx, "home", ShopKind::Armoury);
    let mut c = army(&ctx);
    let sword = ctx
        .game
        .content
        .items
        .new_weapon(&ItemId::new("iron_sword"));
    c.stock.weapons.extend(sword);
    let mut s = TownScreen::new(c, armory, "Millbrook".into());
    press(&mut s, &mut ctx, &[Action::CursorLeft]);
    assert_eq!(s.tab(), Tab::Sell);
    let view = s.view(&ctx);
    assert!(
        view.lines[0].detail.starts_with("20/20  Mt 5"),
        "{}",
        view.lines[0].detail
    );
    press(&mut s, &mut ctx, &[Action::CursorRight]);
    assert_eq!(s.tab(), Tab::Buy);
}

/// A Tailor won't take what isn't cloth or leather: the line says why,
/// and Confirm on it does nothing.
#[test]
fn a_tailor_refuses_a_sword_and_says_why() {
    let mut ctx = ctx();
    let tailor = shop(&ctx, "city", ShopKind::Tailor);
    let mut c = army(&ctx);
    let sword = ctx
        .game
        .content
        .items
        .new_weapon(&ItemId::new("iron_sword"));
    c.stock.weapons.extend(sword);
    let mut s = TownScreen::new(c, tailor, "Kingsgate".into());
    press(&mut s, &mut ctx, &[Action::CursorRight]);
    let view = s.view(&ctx);
    let sword = &view.lines[0];
    assert!(sword.name.starts_with("Iron Sword"));
    assert!(!sword.enabled);
    assert_eq!(sword.gold, "");
    assert_eq!(
        sword.detail,
        "The Tailor buys only cloth and leather armor."
    );
    let before = s.campaign().clone();
    press(&mut s, &mut ctx, &[Action::Confirm]);
    assert_eq!(*s.campaign(), before);
    assert!(s.view(&ctx).selling.is_none());
}

/// A Blacksmith mends the weapons the army holds, the holder's name
/// beside each.
#[test]
fn a_blacksmith_repairs_what_the_army_holds() {
    let mut ctx = ctx();
    let smith = shop(&ctx, "city", ShopKind::Blacksmith);
    let mut c = army(&ctx);
    let copy = c.roster[0].loadout.weapons[0].as_mut().expect("armed");
    copy.durability_left = 1;
    let mut s = TownScreen::new(c, smith, "Kingsgate".into());
    assert_eq!(s.tab(), Tab::Repair);
    let line = s.view(&ctx).lines[0].clone();
    assert!(line.detail.ends_with("Mara"), "{}", line.detail);
    press(&mut s, &mut ctx, &[Action::Confirm]);
    let held = s.campaign().roster[0].loadout.weapon(0).expect("armed");
    let full = ctx.game.content.items.weapon(&held.def).expect("a weapon");
    assert_eq!(held.durability_left, full.durability);
    assert!(s.campaign().gold < 2000);
    assert_eq!(s.view(&ctx).lines.len(), 0);
}

/// Up from the first line wraps to the last (three lines to sell).
#[test]
fn up_from_the_first_line_wraps_to_the_last() {
    let mut ctx = ctx();
    let armory = shop(&ctx, "home", ShopKind::Armoury);
    let mut c = army(&ctx);
    c.stock.add(ItemId::new("leather_vest"));
    let sword = ctx
        .game
        .content
        .items
        .new_weapon(&ItemId::new("iron_sword"));
    c.stock.weapons.extend(sword);
    let mut s = TownScreen::new(c, armory, "Millbrook".into());
    press(&mut s, &mut ctx, &[Action::CursorRight]);
    assert_eq!(s.view(&ctx).lines.len(), 3);
    press(&mut s, &mut ctx, &[Action::CursorUp]);
    assert_eq!(s.view(&ctx).focus, 2);
    press(&mut s, &mut ctx, &[Action::CursorDown]);
    assert_eq!(s.view(&ctx).focus, 0);
}

/// A Blacksmith has one tab: left and right leave it there.
#[test]
fn a_one_tab_shop_stays_on_its_tab() {
    let mut ctx = ctx();
    let smith = shop(&ctx, "city", ShopKind::Blacksmith);
    let mut s = TownScreen::new(army(&ctx), smith, "Kingsgate".into());
    press(&mut s, &mut ctx, &[Action::CursorRight, Action::CursorLeft]);
    assert_eq!(s.tab(), Tab::Repair);
    assert_eq!(s.view(&ctx).tab, 0);
}

/// The sounds `actions` play on `s`.
fn sounds(s: &mut TownScreen, ctx: &mut Ctx, actions: &[Action]) -> Vec<String> {
    ctx.audio.take();
    press(s, ctx, actions);
    ctx.audio
        .take()
        .iter()
        .filter(|r| matches!(r, crate::audio::AudioRequest::PlaySound { .. }))
        .filter_map(|r| r.cue().map(str::to_owned))
        .collect()
}

/// A deal made plays the select sound; one refused, the denied sound.
#[test]
fn a_deal_sounds_made_or_refused() {
    let mut ctx = ctx();
    let vendor = shop(&ctx, "home", ShopKind::Vendor);
    let mut s = TownScreen::new(army(&ctx), vendor.clone(), "Millbrook".into());
    assert_eq!(
        sounds(&mut s, &mut ctx, &[Action::Confirm]),
        ["menu_select"]
    );
    let mut poor = army(&ctx);
    poor.gold = 0;
    let mut s = TownScreen::new(poor, vendor, "Millbrook".into());
    assert_eq!(
        sounds(&mut s, &mut ctx, &[Action::Confirm]),
        ["menu_cancel"]
    );
}

/// A special weapon asks before it is sold, on No: only Yes sells it.
#[test]
fn a_special_weapon_asks_before_it_is_sold() {
    let mut ctx = ctx();
    let armory = shop(&ctx, "home", ShopKind::Armoury);
    let mut c = army(&ctx);
    let axe = ctx
        .game
        .content
        .items
        .new_weapon(&ItemId::new("houndmasters_axe"));
    c.stock.weapons.extend(axe);
    let mut s = TownScreen::new(c, armory, "Millbrook".into());
    press(&mut s, &mut ctx, &[Action::CursorRight, Action::Confirm]);
    let ask = s.view(&ctx).special.expect("asked");
    let answers: Vec<_> = ask.answers.iter().map(|a| a.label.as_str()).collect();
    assert_eq!(answers, ["No", "Yes"]);
    assert!(s.view(&ctx).selling.is_none());
    press(&mut s, &mut ctx, &[Action::Confirm]);
    assert!(s.view(&ctx).special.is_none());
    assert_eq!(s.campaign().stock.weapons.len(), 1, "No sells nothing");
    press(
        &mut s,
        &mut ctx,
        &[Action::Confirm, Action::CursorRight, Action::Confirm],
    );
    assert_eq!(s.campaign().stock.weapons.len(), 0);
    assert_eq!(s.campaign().gold, 2000 + 450);
}

/// Cancel leaves with the campaign as the shop left it.
#[test]
fn cancel_leaves_with_the_campaign() {
    let mut ctx = ctx();
    let vendor = shop(&ctx, "home", ShopKind::Vendor);
    let mut s = TownScreen::new(army(&ctx), vendor, "Millbrook".into());
    press(&mut s, &mut ctx, &[Action::Confirm]);
    let t = press(&mut s, &mut ctx, &[Action::Cancel]);
    let back: Campaign = popped_with(t).expect("the campaign");
    assert_eq!(back.gold, 1700);
}

#[test]
fn town_shop_snapshot() {
    let ctx = ctx();
    let armory = shop(&ctx, "home", ShopKind::Armoury);
    let s = TownScreen::new(army(&ctx), armory, "Millbrook".into());
    let mut h = Harness::with_screen(Box::new(s));
    h.keys("");
    assert_snapshot!(h.snapshot());
}
