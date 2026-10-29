//! Tests of the Preparations screen on the debug Quick Battle
//! (`assets-test/battles/quick.ron`): the test lord (swords), knight (spears,
//! armour up to plate), archer and mage, the test scout left out of the
//! battle with a Steel Bow, a stock of three weapons, three armours, three
//! accessories, six Potions and two Elixirs, 3 000 gold, a pack cap of 6, and
//! Chapter 5's basic shop.

use crate::input::{LEFT_HANDED, RIGHT_HANDED};
use insta::assert_snapshot;
use trpg_content::battle_campaign;
use trpg_core::{GameMode, WeaponAt, WeaponRank};

use super::*;
use crate::audio::AudioRequest;
use crate::harness::Harness;
use crate::screen::tests::ctx;
use crate::screens::battle::QUICK_BATTLE;
use crate::screens::popped_with;

use Action::{Cancel, Confirm, CursorDown, CursorLeft, CursorRight, CursorUp};

/// The Quick Battle's setup, bench and shop, before Preparations.
pub(super) fn setup(c: &Ctx) -> Preparations {
    let content = &c.game.content;
    let def = &content.battles[QUICK_BATTLE];
    let campaign = battle_campaign(content, def, GameMode::Classic, c.game.lead.clone());
    let number = content.chapters["quick"].number;
    Preparations {
        setup: campaign.battle_setup(def, &content.tables()),
        bench: campaign.bench(def),
        shop: number.map(|n| content.basic_shop.shop(n)),
    }
}

pub(super) fn screen(c: &Ctx) -> PreparationsScreen {
    PreparationsScreen::new(setup(c), true)
}

/// One update with `actions`: the transition, as text.
pub(super) fn update(s: &mut PreparationsScreen, c: &mut Ctx, actions: &[Action]) -> String {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    format!("{:?}", s.update(c, &input))
}

/// The sounds `actions` play.
fn sounds(s: &mut PreparationsScreen, c: &mut Ctx, actions: &[Action]) -> Vec<String> {
    c.audio.take();
    update(s, c, actions);
    c.audio
        .take()
        .iter()
        .filter(|r| matches!(r, AudioRequest::PlaySound { .. }))
        .filter_map(|r| r.cue().map(str::to_owned))
        .collect()
}

fn potion() -> ItemId {
    ItemId::new("potion")
}

/// The lord's weapons, by slot.
fn lord_weapons(s: &PreparationsScreen) -> Vec<Option<String>> {
    let lord = &s.setup().units[0];
    (0..3)
        .map(|i| lord.loadout.weapon(i).map(|w| w.def.0.clone()))
        .collect()
}

/// The stock rows, each as its name and its detail (in the columns the
/// glyph skin gives them).
fn stock_texts(s: &PreparationsScreen, c: &Ctx) -> Vec<String> {
    let rows = s.stock_rows(c).into_iter();
    rows.map(|r| format!("{:<15} {}", r.name, r.detail).trim_end().to_owned())
        .collect()
}

/// The stock row that empties the slot.
const PUT_BACK: &str = "(put back in stock)";

/// The Quick Battle's Preparations, through the title.
fn harness() -> Harness {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    h.keys("Down f");
    assert_eq!(h.screens(), ["title", "game_flow", "preparations"]);
    h
}

#[test]
fn opens_on_the_loadouts_tab_with_an_empty_pack() {
    let c = ctx();
    let s = screen(&c);
    assert_eq!(s.name(), "preparations");
    assert!(!s.is_overlay());
    assert_eq!((s.tab(), s.focus()), (Tab::Loadouts, Focus::Tabs));
    assert_eq!(s.message(), None);
    assert!(!s.is_asking_leave());
    assert_eq!(s.pack_header(&c), "Pack 0/6");
    assert_eq!(s.packed(), [] as [(trpg_core::ItemId, usize); 0]);
    assert_eq!(
        s.spare(),
        [(ItemId::new("elixir"), 2), (ItemId::new("potion"), 6)]
    );
    assert_eq!(
        Tab::ALL.map(|t| c.text(t.key())),
        ["Loadouts", "Pack", "Shop", "Options", "Fight!"]
    );
}

#[test]
fn left_and_right_pick_a_tab_and_fight_starts_the_battle() {
    let mut c = ctx();
    let mut s = screen(&c);
    assert_eq!(sounds(&mut s, &mut c, &[CursorRight]), ["menu_move"]);
    assert_eq!(s.tab(), Tab::Pack);
    update(&mut s, &mut c, &[CursorRight]);
    assert_eq!(s.tab(), Tab::Shop);
    // Options (0805): Down doesn't open it; Confirm opens the Options
    // screen over this one, once.
    update(&mut s, &mut c, &[CursorRight]);
    assert_eq!(s.tab(), Tab::Options);
    assert_eq!(update(&mut s, &mut c, &[CursorDown, CursorUp]), "None");
    assert_eq!(
        update(&mut s, &mut c, &[Confirm, CursorRight]),
        "Push(options)"
    );
    assert_eq!((s.tab(), s.focus()), (Tab::Options, Focus::Tabs));
    assert_eq!(update(&mut s, &mut c, &[]), "None");
    update(&mut s, &mut c, &[CursorRight]);
    assert_eq!(s.tab(), Tab::Fight);
    // Down opens a tab, but never starts the battle.
    assert_eq!(update(&mut s, &mut c, &[CursorDown, CursorUp]), "None");
    assert_eq!(s.focus(), Focus::Tabs);
    update(&mut s, &mut c, &[CursorRight]);
    assert_eq!(s.tab(), Tab::Loadouts, "wraps");
    update(&mut s, &mut c, &[CursorLeft]);
    assert_eq!(s.tab(), Tab::Fight);
    // Fight! pops with what was prepared, for the game flow below.
    let input = FrameInput::new(vec![Confirm], 0.0, vec![]);
    let prepared = popped_with::<Preparations>(s.update(&mut c, &input));
    assert_eq!(prepared.as_ref(), Some(s.prep()));
}

#[test]
fn a_stock_weapon_goes_into_an_empty_slot() {
    let mut c = ctx();
    let mut s = screen(&c);
    // Loadouts → the lord → his third slot → the stock.
    assert_eq!(sounds(&mut s, &mut c, &[Confirm]), ["menu_select"]);
    assert_eq!(s.focus(), Focus::Units);
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(s.focus(), Focus::Slots);
    assert_eq!(sounds(&mut s, &mut c, &[CursorDown]), ["menu_move"]);
    update(&mut s, &mut c, &[CursorDown, Confirm]);
    assert_eq!(s.focus(), Focus::Stock);
    assert_eq!(
        stock_texts(&s, &c),
        [
            "Steel Spear     can't use spears",
            "Iron Axe        can't use axes",
            "Iron Sword      Mt 5 Hit 90 Wt 2 Rng1 20/20",
        ]
    );
    assert_eq!(
        s.speed_line(&c).as_deref(),
        Some("AS 7 with no weapon"),
        "a row he can't take changes nothing"
    );
    update(&mut s, &mut c, &[CursorUp]);
    assert_eq!(
        s.speed_line(&c).as_deref(),
        Some("AS 7 → 5 with Iron Sword")
    );
    assert_eq!(sounds(&mut s, &mut c, &[Confirm]), ["menu_select"]);
    assert_eq!(s.focus(), Focus::Slots);
    assert_eq!(
        lord_weapons(&s),
        ["iron_sword", "steel_sword", "iron_sword"].map(|w| Some(w.to_owned()))
    );
    assert_eq!(s.setup().stock.weapons.len(), 2);
    assert_eq!(s.speed_line(&c).as_deref(), Some("AS 5 with Iron Sword"));
    // The slot now offers to put it back.
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(stock_texts(&s, &c)[0], PUT_BACK);
    assert_eq!(s.speed_line(&c).as_deref(), Some("AS 5 → 7 with no weapon"));
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(lord_weapons(&s)[2], None);
    assert_eq!(s.setup().stock.weapons.len(), 3);
}

#[test]
fn an_item_the_unit_cant_use_is_refused_with_the_reason() {
    let mut c = ctx();
    let mut s = screen(&c);
    update(&mut s, &mut c, &[Confirm, Confirm, CursorDown, CursorDown]);
    update(&mut s, &mut c, &[Confirm]);
    let before = s.setup().clone();
    assert_eq!(sounds(&mut s, &mut c, &[Confirm]), ["menu_cancel"]);
    assert_eq!(s.message(), Some("Test Lord can't use spears."));
    assert_eq!(s.focus(), Focus::Stock);
    assert_eq!(s.setup(), &before);
    assert!(s.stock_rows(&c)[0].unusable.is_some());
    // The next key clears the message.
    update(&mut s, &mut c, &[CursorDown]);
    assert_eq!(s.message(), None);
}

#[test]
fn reasons_read_as_the_player_sees_them() {
    let c = ctx();
    assert_eq!(
        reason_text(&c, Unusable::Rank(WeaponRank::D)),
        "needs rank D"
    );
    let kinds = [
        (WeaponKind::Sword, "can't use swords"),
        (WeaponKind::Spear, "can't use spears"),
        (WeaponKind::Axe, "can't use axes"),
        (WeaponKind::Bow, "can't use bows"),
        (WeaponKind::Gauntlet, "can't use gauntlets"),
    ];
    for (kind, text) in kinds {
        assert_eq!(reason_text(&c, Unusable::Kind(kind)), text);
    }
    let weights = [
        (ArmourWeight::Cloth, "can't wear cloth armor"),
        (ArmourWeight::Leather, "can't wear leather armor"),
        (ArmourWeight::Chain, "can't wear chain armor"),
        (ArmourWeight::Plate, "can't wear plate armor"),
    ];
    for (weight, text) in weights {
        assert_eq!(reason_text(&c, Unusable::Armour(weight)), text);
    }
}

#[test]
fn armour_and_accessories_come_from_the_stock_with_their_counts() {
    let mut c = ctx();
    let mut s = screen(&c);
    // The lord's armour slot: he wears a Leather Vest.
    update(
        &mut s,
        &mut c,
        &[Confirm, Confirm, CursorUp, CursorUp, Confirm],
    );
    assert_eq!(
        stock_texts(&s, &c),
        [
            PUT_BACK,
            // The Exile wears leather at most.
            "Chain Mail      can't wear chain armor",
            "Iron Plate      can't wear plate armor",
            "Linen Robe      ×1  Res +4  Wt 0",
            // The mage's, to swap for; the knight wears chain, the archer
            // the same vest.
            "Linen Robe      from Test Mage",
        ]
    );
    // With no weapon slot highlighted, the speed is the equipped weapon's:
    // the robe weighs nothing, the vest 1.
    update(&mut s, &mut c, &[CursorDown, CursorDown, CursorDown]);
    assert_eq!(
        s.speed_line(&c).as_deref(),
        Some("AS 5 → 6 with Iron Sword")
    );
    update(&mut s, &mut c, &[Confirm]);
    let lord = &s.setup().units[0];
    assert_eq!(lord.loadout.armour, Some(ItemId::new("linen_robe")));
    assert_eq!(s.setup().stock.count(&ItemId::new("leather_vest")), 1);
    assert_eq!(s.speed_line(&c).as_deref(), Some("AS 6 with Iron Sword"));
    // The accessory slot: empty, so nothing to put back.
    update(&mut s, &mut c, &[CursorDown, Confirm]);
    assert_eq!(
        stock_texts(&s, &c),
        [
            "Focus Charm     ×1  Dex +2",
            "Power Ring      ×1  Str +2",
            "Speed Ring      ×1  Spd +2",
        ]
    );
    update(&mut s, &mut c, &[CursorUp]);
    assert_eq!(
        s.speed_line(&c).as_deref(),
        Some("AS 6 → 8 with Iron Sword")
    );
    update(&mut s, &mut c, &[Confirm]);
    let lord = &s.setup().units[0];
    assert_eq!(lord.loadout.accessory, Some(ItemId::new("speed_ring")));
}

#[test]
fn a_slot_with_nothing_in_stock_says_so() {
    let mut c = ctx();
    let mut s = screen(&c);
    s.prep.setup.stock = trpg_core::Stock::default();
    // The lord's empty third slot.
    update(&mut s, &mut c, &[Confirm, Confirm, CursorDown, CursorDown]);
    assert_eq!(sounds(&mut s, &mut c, &[Confirm]), ["menu_cancel"]);
    assert_eq!(s.focus(), Focus::Slots);
    assert_eq!(s.message(), Some("Nothing in stock for this slot."));
    // A filled slot can still be put back.
    update(&mut s, &mut c, &[CursorUp, Confirm]);
    assert_eq!(s.focus(), Focus::Stock);
    assert_eq!(stock_texts(&s, &c), [PUT_BACK]);
}

#[test]
fn cancel_goes_back_one_list_at_a_time() {
    let mut c = ctx();
    let mut s = screen(&c);
    update(&mut s, &mut c, &[Confirm, CursorDown, Confirm, Confirm]);
    assert_eq!(s.focus(), Focus::Stock);
    // The knight: the second unit.
    assert_eq!(s.unit().map(|u| u.name.as_str()), Some("Test Knight"));
    for back in [Focus::Slots, Focus::Units, Focus::Tabs] {
        assert_eq!(sounds(&mut s, &mut c, &[Cancel]), ["menu_cancel"]);
        assert_eq!(s.focus(), back);
    }
    // Left and Right do nothing inside the loadouts.
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(
        sounds(&mut s, &mut c, &[CursorLeft, CursorRight]),
        [] as [std::string::String; 0]
    );
    assert_eq!((s.tab(), s.focus()), (Tab::Loadouts, Focus::Units));
    // The unit list wraps, and a new unit starts on its first slot.
    update(
        &mut s,
        &mut c,
        &[Confirm, CursorDown, Cancel, CursorUp, CursorUp],
    );
    assert_eq!(s.unit().map(|u| u.name.as_str()), Some("Test Scout"));
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(s.gear_slot(), Some(GearSlot::Weapon(0)));
}

#[test]
fn the_pack_fills_from_the_stock_up_to_the_cap() {
    let mut c = ctx();
    let mut s = screen(&c);
    update(&mut s, &mut c, &[CursorRight, Confirm]);
    assert_eq!((s.tab(), s.focus()), (Tab::Pack, Focus::Spare));
    // Elixir, then Potion: two Elixirs and four Potions fill the pack.
    assert_eq!(sounds(&mut s, &mut c, &[Confirm]), ["menu_select"]);
    update(&mut s, &mut c, &[Confirm]);
    // The Elixirs are gone from the stock: the cursor is on the Potions.
    assert_eq!(s.spare(), [(potion(), 6)]);
    update(&mut s, &mut c, &[Confirm, Confirm, Confirm, Confirm]);
    assert_eq!(s.pack_header(&c), "Pack 6/6");
    assert_eq!(s.packed(), [(ItemId::new("elixir"), 2), (potion(), 4)]);
    assert_eq!(s.message(), None);
    let before = s.setup().clone();
    assert_eq!(sounds(&mut s, &mut c, &[Confirm]), ["menu_cancel"]);
    assert_eq!(s.message(), Some("The pack is full (6/6)."));
    assert_eq!(s.setup(), &before);
    assert_eq!(s.setup().stock.count(&potion()), 2);
}

#[test]
fn pack_items_go_back_to_the_stock() {
    let mut c = ctx();
    let mut s = screen(&c);
    // Nothing packed: Right stays on the stock.
    update(&mut s, &mut c, &[CursorRight, Confirm]);
    assert_eq!(
        sounds(&mut s, &mut c, &[CursorRight]),
        [] as [std::string::String; 0]
    );
    assert_eq!(s.focus(), Focus::Spare);
    update(&mut s, &mut c, &[CursorDown, Confirm, Confirm]);
    assert_eq!(s.packed(), [(potion(), 2)]);
    assert_eq!(sounds(&mut s, &mut c, &[CursorRight]), ["menu_move"]);
    assert_eq!(s.focus(), Focus::Pack);
    assert_eq!(sounds(&mut s, &mut c, &[Confirm]), ["menu_select"]);
    assert_eq!(s.packed(), [(potion(), 1)]);
    assert_eq!(s.focus(), Focus::Pack);
    update(&mut s, &mut c, &[CursorLeft]);
    assert_eq!(s.focus(), Focus::Spare);
    update(&mut s, &mut c, &[CursorLeft]);
    assert_eq!(s.focus(), Focus::Pack);
    // The last one out: back to the stock's list.
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(s.packed(), [] as [(trpg_core::ItemId, usize); 0]);
    assert_eq!(s.focus(), Focus::Spare);
    assert_eq!(s.setup().stock.count(&potion()), 6);
    assert_eq!(sounds(&mut s, &mut c, &[Cancel]), ["menu_cancel"]);
    assert_eq!(s.focus(), Focus::Tabs);
}

#[test]
fn the_pack_tab_opens_on_the_pack_when_the_stock_has_no_consumables() {
    let mut c = ctx();
    let mut s = screen(&c);
    s.prep.setup.stock.items.retain(|id, _| *id != potion());
    update(&mut s, &mut c, &[CursorRight, Confirm, Confirm, Confirm]);
    // Both Elixirs packed, none left: the cursor moved to the pack.
    assert_eq!(s.spare(), [] as [(trpg_core::ItemId, u32); 0]);
    assert_eq!(s.focus(), Focus::Pack);
    update(&mut s, &mut c, &[Cancel, Confirm]);
    assert_eq!(s.focus(), Focus::Pack);
    // With neither, the tab can't be opened.
    s.prep.setup.pack.items.clear();
    update(&mut s, &mut c, &[Cancel]);
    assert_eq!(sounds(&mut s, &mut c, &[Confirm]), ["menu_cancel"]);
    assert_eq!(s.focus(), Focus::Tabs);
}

#[test]
fn leaving_asks_first_and_only_when_allowed() {
    let mut c = ctx();
    let mut s = screen(&c);
    assert_eq!(sounds(&mut s, &mut c, &[Cancel]), ["menu_select"]);
    assert!(s.is_asking_leave());
    // Other keys do nothing while it asks.
    assert_eq!(update(&mut s, &mut c, &[CursorRight, CursorDown]), "None");
    assert_eq!((s.tab(), s.focus()), (Tab::Loadouts, Focus::Tabs));
    assert_eq!(sounds(&mut s, &mut c, &[Cancel]), ["menu_cancel"]);
    assert!(!s.is_asking_leave());
    // Leaving pops with nothing.
    assert_eq!(update(&mut s, &mut c, &[Cancel, Confirm]), "Pop");

    let mut s = PreparationsScreen::new(setup(&c), false);
    assert_eq!(s.prep().bench.len(), 1);
    assert_eq!(
        sounds(&mut s, &mut c, &[Cancel]),
        [] as [std::string::String; 0]
    );
    assert!(!s.is_asking_leave());
    assert!(!s.help(&c).contains("leave"));
}

#[test]
fn help_names_the_players_keys() {
    let mut c = ctx();
    let mut s = screen(&c);
    assert_eq!(s.help(&c), "arrows choose · f open · d leave");
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(s.help(&c), "arrows choose · f change gear · d back");
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(s.help(&c), "arrows choose · f stock · d back");
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(s.help(&c), "arrows choose · f take · d back");
    update(
        &mut s,
        &mut c,
        &[Cancel, Cancel, Cancel, CursorRight, Confirm],
    );
    assert_eq!(s.help(&c), "arrows choose · f pack it · d back");
    update(&mut s, &mut c, &[Confirm, CursorRight]);
    assert_eq!(s.help(&c), "arrows choose · f put back · d back");
    update(&mut s, &mut c, &[Cancel, CursorRight]);
    assert_eq!(s.help(&c), "arrows choose · f open · d leave");
    update(&mut s, &mut c, &[Confirm, CursorRight]);
    assert_eq!(
        s.help(&c),
        "Up Down choose · Left Right buy, sell, repair · f confirm · d back"
    );
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(s.help(&c), "Up Down how many · f sell · d back");
    update(&mut s, &mut c, &[Cancel, Cancel, CursorRight, CursorRight]);
    assert_eq!(s.help(&c), "arrows choose · f start the battle · d leave");
    update(&mut s, &mut c, &[Cancel]);
    assert_eq!(s.help(&c), "f yes / d no");
    // The left-handed layout has other keys.
    c.use_layout(LEFT_HANDED);
    assert_eq!(s.help(&c), "j yes / k no");
}

#[test]
fn loadouts_tab_snapshot() {
    let mut h = harness();
    // The lord's third weapon slot, on the Iron Sword he can take.
    h.keys("f f Down Down f Up");
    assert_snapshot!(h.snapshot());
}

#[test]
fn pack_tab_at_the_cap_snapshot() {
    let mut h = harness();
    h.keys("Right f f Down f f f f f f");
    assert_snapshot!(h.snapshot());
}

#[test]
fn fight_tab_and_leave_question_snapshot() {
    let mut h = harness();
    h.keys("Left d");
    assert_snapshot!(h.snapshot());
}

/// Nick (0408): with two archers and only one in the battle, the better
/// bow of the one left out can go to the one who fights.
#[test]
fn a_benched_units_gear_is_traded_through_the_stock() {
    let mut c = ctx();
    let mut s = screen(&c);
    // The scout is listed last, after the deployed units.
    update(&mut s, &mut c, &[Confirm, CursorUp, Confirm]);
    assert_eq!(s.who(), Some(PrepUnit::Benched(0)));
    assert_eq!(s.unit().map(|u| u.name.as_str()), Some("Test Scout"));
    assert_eq!(s.speed_line(&c).as_deref(), Some("AS 1 with Steel Bow"));
    // Her Steel Bow goes back to the stock.
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(stock_texts(&s, &c)[0], PUT_BACK);
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(s.prep().bench[0].loadout.weapon(0), None);
    assert_eq!(s.prep().bench[0].loadout.equipped, None);
    // The archer (two above her, past the mage) takes it in his second
    // slot.
    update(
        &mut s,
        &mut c,
        &[Cancel, CursorUp, CursorUp, Confirm, CursorDown, Confirm],
    );
    assert_eq!(s.unit().map(|u| u.name.as_str()), Some("Test Archer"));
    assert_eq!(
        stock_texts(&s, &c).last().map(String::as_str),
        Some("Steel Bow       Mt 9 Hit 70 Wt 5 Rng2 25/25")
    );
    update(&mut s, &mut c, &[CursorUp, Confirm]);
    let archer = &s.setup().units[2];
    assert_eq!(
        archer.loadout.weapon(1).map(|w| w.def.0.as_str()),
        Some("steel_bow")
    );
    // The scout can take a stock item too, by the same rules.
    update(
        &mut s,
        &mut c,
        &[Cancel, CursorDown, CursorDown, Confirm, Confirm],
    );
    assert_eq!(s.who(), Some(PrepUnit::Benched(0)));
    assert!(stock_texts(&s, &c)[0].ends_with("can't use spears"));
}

#[test]
fn a_benched_unit_snapshot() {
    let mut h = harness();
    // The scout's first slot, its list on the archer's Iron Bow.
    h.keys("f Up f f Up");
    assert_snapshot!(h.snapshot());
}

#[test]
fn down_on_the_tabs_opens_the_tab() {
    let mut c = ctx();
    let mut s = screen(&c);
    assert_eq!(sounds(&mut s, &mut c, &[CursorDown]), ["menu_select"]);
    assert_eq!(s.focus(), Focus::Units);
    update(&mut s, &mut c, &[Cancel, CursorRight, CursorDown]);
    assert_eq!((s.tab(), s.focus()), (Tab::Pack, Focus::Spare));
}

#[test]
fn up_and_down_move_over_the_pack() {
    let mut c = ctx();
    let mut s = screen(&c);
    // An Elixir and a Potion packed, then over to the pack.
    update(&mut s, &mut c, &[CursorRight, Confirm, Confirm]);
    update(&mut s, &mut c, &[CursorDown, Confirm, CursorRight]);
    assert_eq!(s.focus(), Focus::Pack);
    assert_eq!(s.packed().len(), 2);
    // Down to the Potion: Confirm puts that one back.
    assert_eq!(sounds(&mut s, &mut c, &[CursorDown]), ["menu_move"]);
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(s.packed(), [(ItemId::new("elixir"), 1)]);
    assert_eq!(s.setup().stock.count(&potion()), 6);
}

/// Seals (ticket 0603) are neither consumables nor gear: they stay in the
/// stock and are offered nowhere here.
#[test]
fn seals_are_not_offered() {
    let mut c = ctx();
    let mut s = screen(&c);
    let before = (s.spare(), stock_texts(&s, &c));
    s.prep.setup.stock.add(ItemId::new("tier_2_seal"));
    assert_eq!(s.spare(), before.0);
    // The lord's weapon, armour and accessory lists.
    for down in 0..5 {
        let mut t = s.clone();
        update(&mut t, &mut c, &[Confirm, Confirm]);
        for _ in 0..down {
            update(&mut t, &mut c, &[CursorDown]);
        }
        let rows = stock_texts(&t, &c).join("\n");
        assert!(!rows.contains("Seal"), "{rows}");
    }
    assert_eq!(
        s.prep.setup.pack_from_stock(&ItemId::new("tier_2_seal")),
        Err(PrepError::NotConsumable)
    );
}

/// Nick (PR #140): the list also has what the other units hold, and
/// taking one of those swaps the two units' items directly.
#[test]
fn another_units_item_is_swapped_directly() {
    let mut c = ctx();
    let mut s = screen(&c);
    // The archer (third unit), his first slot: an Iron Bow.
    update(&mut s, &mut c, &[Confirm, CursorDown, CursorDown, Confirm]);
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(
        stock_texts(&s, &c),
        [
            PUT_BACK,
            "Steel Spear     can't use spears",
            "Iron Axe        can't use axes",
            "Iron Sword      can't use swords",
            "Steel Bow       25/25  from Test Scout",
        ]
    );
    let last = s.stock_rows(&c).pop().map(|r| r.from);
    let scout = PrepUnit::Benched(0);
    assert_eq!(last, Some(Source::Unit(scout, GearSlot::Weapon(0))));
    update(&mut s, &mut c, &[CursorUp]);
    assert_eq!(s.speed_line(&c).as_deref(), Some("AS 2 → 0 with Steel Bow"));
    assert_eq!(sounds(&mut s, &mut c, &[Confirm]), ["menu_select"]);
    let bow = |u: &Unit| u.loadout.weapon(0).map(|w| w.def.0.clone());
    assert_eq!(bow(&s.setup().units[2]).as_deref(), Some("steel_bow"));
    assert_eq!(bow(&s.prep().bench[0]).as_deref(), Some("iron_bow"));
    assert_eq!(s.setup().stock.weapons.len(), 3, "the stock isn't touched");
    // Now the list offers the Iron Bow back, from the scout.
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(
        stock_texts(&s, &c).last().map(String::as_str),
        Some("Iron Bow        20/20  from Test Scout")
    );
    // The lord's armour: the mage's Linen Robe for his Leather Vest,
    // which the mage (cloth at most) can't wear, so it goes to the stock.
    update(
        &mut s,
        &mut c,
        &[Cancel, Cancel, CursorUp, CursorUp, Confirm],
    );
    update(
        &mut s,
        &mut c,
        &[CursorUp, CursorUp, Confirm, CursorUp, Confirm],
    );
    assert_eq!(
        s.setup().units[0].loadout.armour,
        Some(ItemId::new("linen_robe"))
    );
    let mage = s.setup().units.iter().find(|u| u.name == "Test Mage");
    assert_eq!(mage.map(|u| &u.loadout.armour), Some(&None));
    assert_eq!(s.setup().stock.count(&ItemId::new("leather_vest")), 1);
}

/// The Loadouts tab of `view`.
fn loadouts(view: &PreparationsView) -> &LoadoutsView {
    match &view.body {
        PrepBody::Loadouts(loadouts) => loadouts,
        other => panic!("not the Loadouts tab: {other:?}"),
    }
}

/// The chosen unit's gear in `view`.
fn gear(view: &PreparationsView) -> &GearView {
    loadouts(view).gear.as_ref().expect("a unit is chosen")
}

/// The Pack tab of `view`.
fn pack(view: &PreparationsView) -> &PackView {
    match &view.body {
        PrepBody::Pack(pack) => pack,
        other => panic!("not the Pack tab: {other:?}"),
    }
}

/// A slot row holding `item`.
fn slot(label: &str, item: Option<&str>, wear: &str) -> SlotRowView {
    SlotRowView {
        label: label.to_owned(),
        item: item.map(str::to_owned),
        wear: wear.to_owned(),
    }
}

#[test]
fn the_view_on_the_first_frame_is_the_loadouts_tab_with_the_cursor_on_the_tabs() {
    let c = ctx();
    let view = screen(&c).view(&c);
    assert_eq!(view.heading, "Preparations");
    let tabs: Vec<_> = view
        .tabs
        .iter()
        .map(|t| (t.tab, &*t.label, t.shown))
        .collect();
    assert_eq!(
        tabs,
        [
            (Tab::Loadouts, "Loadouts", true),
            (Tab::Pack, "Pack", false),
            (Tab::Shop, "Shop", false),
            (Tab::Options, "Options", false),
            (Tab::Fight, "Fight!", false),
        ]
    );
    assert!(view.on_tabs);
    assert_eq!((&view.message, &view.leave_question), (&None, &None));
    assert_eq!(view.help, "arrows choose · f open · d leave");

    // The army, the one left out of the battle last; nothing chosen yet.
    let units = &loadouts(&view).units;
    assert_eq!(units.title, "Units");
    let rows: Vec<_> = units.rows.iter().map(|u| (&*u.name, u.deployed)).collect();
    assert_eq!(
        rows,
        [
            ("Test Lord", true),
            ("Test Knight", true),
            ("Test Archer", true),
            ("Test Mage", true),
            ("Test Scout", false),
        ]
    );
    assert_eq!((units.cursor, units.state), (0, ListState::Idle));
    // The first unit's gear shows beside it.
    let gear = gear(&view);
    assert_eq!(gear.slots.title, "Test Lord · Exile");
    assert_eq!(
        gear.slots.rows,
        [
            slot("Weapon", Some("Iron Sword"), "20/20"),
            slot("Weapon", Some("Steel Sword"), "25/25"),
            slot("Weapon", None, ""),
            slot("Armor", Some("Leather Vest"), ""),
            slot("Accessory", None, ""),
        ]
    );
    assert_eq!((gear.slots.cursor, gear.slots.state), (0, ListState::Idle));
    assert_eq!(gear.speed.as_deref(), Some("AS 5 with Iron Sword"));
    assert_eq!(gear.benched_note, None);
    assert_eq!(gear.stock.title, "Stock: weapons");
    assert_eq!((gear.stock.cursor, gear.stock.state), (0, ListState::Idle));
}

#[test]
fn the_view_follows_the_cursor_through_the_loadout_lists() {
    let mut c = ctx();
    let mut s = screen(&c);
    update(&mut s, &mut c, &[Confirm]);
    let view = s.view(&c);
    assert!(!view.on_tabs);
    assert_eq!(loadouts(&view).units.state, ListState::Focused);
    assert_eq!(gear(&view).slots.state, ListState::Idle);
    assert_eq!(view.help, "arrows choose · f change gear · d back");

    // The lord's third slot: the unit is the chosen one.
    update(&mut s, &mut c, &[Confirm, CursorDown, CursorDown]);
    let view = s.view(&c);
    assert_eq!(loadouts(&view).units.state, ListState::Chosen);
    let slots = &gear(&view).slots;
    assert_eq!((slots.cursor, slots.state), (2, ListState::Focused));
    assert_eq!(gear(&view).stock.state, ListState::Idle);
    assert_eq!(
        gear(&view).speed.as_deref(),
        Some("AS 7 with no weapon"),
        "the speed is with the highlighted slot's weapon"
    );

    // Its stock, on the Iron Sword: the slot is the chosen one too, and
    // the speed shows the change.
    update(&mut s, &mut c, &[Confirm, CursorUp]);
    let view = s.view(&c);
    let gear = gear(&view);
    assert_eq!(loadouts(&view).units.state, ListState::Chosen);
    assert_eq!(
        (gear.slots.cursor, gear.slots.state),
        (2, ListState::Chosen)
    );
    assert_eq!(
        (gear.stock.cursor, gear.stock.state),
        (2, ListState::Focused)
    );
    let rows: Vec<_> = gear.stock.rows.iter().collect();
    let row = |name: &str, detail: &str, usable| ItemRowView {
        name: name.to_owned(),
        detail: detail.to_owned(),
        usable,
    };
    assert_eq!(
        rows,
        [
            &row("Steel Spear", "can't use spears", false),
            &row("Iron Axe", "can't use axes", false),
            &row("Iron Sword", "Mt 5 Hit 90 Wt 2 Rng1 20/20", true),
        ]
    );
    assert_eq!(gear.speed.as_deref(), Some("AS 7 → 5 with Iron Sword"));

    // A refused item's reason is the message.
    update(&mut s, &mut c, &[CursorDown, Confirm]);
    let view = s.view(&c);
    assert_eq!(view.message.as_deref(), Some("Test Lord can't use spears."));
    assert_eq!(view.leave_question, None);

    // The armour and accessory slots name their own stock.
    update(&mut s, &mut c, &[Cancel, CursorDown]);
    assert_eq!(gear_title(&s, &c), "Stock: armor");
    update(&mut s, &mut c, &[CursorDown]);
    assert_eq!(gear_title(&s, &c), "Stock: accessories");
}

/// The title of the stock list beside the slots.
fn gear_title(s: &PreparationsScreen, c: &Ctx) -> String {
    gear(&s.view(c)).stock.title.clone()
}

#[test]
fn the_view_of_a_slot_that_holds_something_offers_to_put_it_back() {
    let mut c = ctx();
    let mut s = screen(&c);
    update(&mut s, &mut c, &[Confirm, Confirm, Confirm]);
    let view = s.view(&c);
    let first = &gear(&view).stock.rows[0];
    assert_eq!(
        (&*first.name, &*first.detail, first.usable),
        (PUT_BACK, "", true)
    );
}

#[test]
fn the_view_of_a_benched_unit_says_it_is_not_in_this_battle() {
    let mut c = ctx();
    let mut s = screen(&c);
    // The scout, the last unit, and her first slot's list.
    update(&mut s, &mut c, &[Confirm, CursorUp]);
    let view = s.view(&c);
    let units = &loadouts(&view).units;
    assert_eq!((units.cursor, units.state), (4, ListState::Focused));
    assert!(!units.rows[4].deployed);
    let gear = gear(&view);
    assert_eq!(gear.slots.title, "Test Scout · Archer");
    assert_eq!(
        gear.slots.rows[0],
        slot("Weapon", Some("Steel Bow"), "25/25")
    );
    assert_eq!(gear.speed.as_deref(), Some("AS 1 with Steel Bow"));
    assert_eq!(gear.benched_note.as_deref(), Some("Not in this battle"));
    // What another unit holds says whose it is.
    update(&mut s, &mut c, &[Confirm, Confirm, CursorUp]);
    let view = s.view(&c);
    let last = self::gear(&view).stock.rows.last().cloned();
    let last = last.expect("the archer's bow");
    assert_eq!(
        (&*last.name, &*last.detail, last.usable),
        ("Iron Bow", "20/20  from Test Archer", true)
    );
    assert_eq!(
        self::gear(&view).speed.as_deref(),
        Some("AS 1 → 3 with Iron Bow")
    );
}

#[test]
fn the_view_of_the_pack_tab_counts_both_sides_and_says_when_the_pack_is_full() {
    let mut c = ctx();
    let mut s = screen(&c);
    let row = |name: &str, count, effect: &str| ConsumableRowView {
        name: name.to_owned(),
        count,
        effect: effect.to_owned(),
    };
    update(&mut s, &mut c, &[CursorRight]);
    let view = s.view(&c);
    assert!(view.on_tabs);
    assert!(view.tabs[1].shown && !view.tabs[0].shown);
    let tab = pack(&view);
    assert_eq!(tab.spare.title, "Stock");
    assert_eq!(
        tab.spare.rows,
        [
            row("Elixir", 2, "Restore all HP"),
            row("Potion", 6, "Restore 20 HP")
        ]
    );
    assert_eq!(tab.spare.state, ListState::Idle);
    assert_eq!(tab.pack.title, "Pack 0/6");
    assert_eq!(
        tab.pack.rows,
        [] as [crate::screens::preparations::view::ConsumableRowView; 0]
    );
    assert_eq!(tab.pack.state, ListState::Idle);

    // An Elixir and five Potions fill it; one more is refused.
    update(&mut s, &mut c, &[Confirm, Confirm, CursorDown]);
    update(
        &mut s,
        &mut c,
        &[Confirm, Confirm, Confirm, Confirm, Confirm],
    );
    update(&mut s, &mut c, &[Confirm]);
    let view = s.view(&c);
    let tab = pack(&view);
    assert_eq!(
        tab.spare.rows,
        [
            row("Elixir", 1, "Restore all HP"),
            row("Potion", 1, "Restore 20 HP")
        ]
    );
    assert_eq!((tab.spare.cursor, tab.spare.state), (1, ListState::Focused));
    assert_eq!(tab.pack.title, "Pack 6/6");
    assert_eq!(
        tab.pack.rows,
        [
            row("Elixir", 1, "Restore all HP"),
            row("Potion", 5, "Restore 20 HP")
        ]
    );
    assert_eq!(tab.pack.state, ListState::Idle);
    assert_eq!(view.message.as_deref(), Some("The pack is full (6/6)."));
    assert_eq!(view.help, "arrows choose · f pack it · d back");

    // Over to the pack's second row.
    update(&mut s, &mut c, &[CursorRight, CursorDown]);
    let view = s.view(&c);
    let tab = pack(&view);
    assert_eq!(view.message, None);
    assert_eq!(tab.spare.state, ListState::Idle);
    assert_eq!((tab.pack.cursor, tab.pack.state), (1, ListState::Focused));
}

#[test]
fn the_view_of_the_options_and_fight_tabs_and_the_leave_question() {
    let mut c = ctx();
    let mut s = screen(&c);
    update(&mut s, &mut c, &[CursorLeft, CursorLeft]);
    let view = s.view(&c);
    assert_eq!(view.body, PrepBody::Options);
    assert!(view.tabs[3].shown);

    update(&mut s, &mut c, &[CursorRight]);
    let view = s.view(&c);
    assert_eq!(view.body, PrepBody::Fight("Units 4 · Pack 0/6".to_owned()));
    assert!(view.tabs[4].shown && view.on_tabs);
    assert_eq!(view.leave_question, None);
    assert_eq!(view.help, "arrows choose · f start the battle · d leave");

    update(&mut s, &mut c, &[Cancel]);
    let view = s.view(&c);
    assert_eq!(
        view.leave_question.map(|q| q.text).as_deref(),
        Some("Leave preparations?")
    );
    assert_eq!(view.message, None);
    assert_eq!(view.help, "f yes / d no");
    update(&mut s, &mut c, &[Cancel]);
    assert_eq!(s.view(&c).leave_question, None);
}

/// An army of nobody has no gear to show.
#[test]
fn the_view_of_an_empty_army_has_no_gear() {
    let c = ctx();
    let mut s = screen(&c);
    s.prep
        .setup
        .units
        .retain(|u| u.faction != trpg_core::Faction::Player);
    s.prep.bench.clear();
    let view = s.view(&c);
    assert_eq!(
        loadouts(&view).units.rows,
        [] as [crate::screens::preparations::view::UnitRowView; 0]
    );
    assert_eq!(loadouts(&view).gear, None);
}

/// The running game's Preparations, over the game flow, says the same
/// (ADR-0054: a behaviour test reads the view).
#[test]
fn the_flows_screen_gives_its_view() {
    let mut h = harness();
    h.keys("Right f");
    let screen = h.game().screen::<PreparationsScreen>();
    let view = screen.expect("Preparations is open").view(h.game().ctx());
    assert!(!view.on_tabs);
    assert_eq!(pack(&view).spare.state, ListState::Focused);
}

/// The screen can be found by its type (`Stack::find`), for tests that
/// read its view.
#[test]
fn the_screen_opts_into_being_found_by_type() {
    let c = ctx();
    let s = screen(&c);
    let found = s
        .as_any()
        .and_then(|any| any.downcast_ref::<PreparationsScreen>());
    assert_eq!(found.map(PreparationsScreen::tab), Some(Tab::Loadouts));
}

/// The screen on its Shop tab's lists, on shop tab `tab` (0 Buy, 1 Sell,
/// 2 Repair).
fn in_shop(c: &mut Ctx, s: &mut PreparationsScreen, tab: usize) {
    update(s, c, &[CursorRight, CursorRight, Confirm]);
    assert_eq!((s.tab(), s.focus()), (Tab::Shop, Focus::Shop));
    for _ in 0..tab {
        update(s, c, &[CursorRight]);
    }
}

fn shop_of(view: &PreparationsView) -> &crate::screens::battle::view::ShopScreenView {
    match &view.body {
        PrepBody::Shop(shop) => shop,
        other => panic!("not the Shop tab: {other:?}"),
    }
}

/// Each line as its name, gold and whether it can be chosen.
fn shop_lines(s: &PreparationsScreen, c: &Ctx) -> Vec<(String, String, bool)> {
    let view = s.view(c);
    let lines = shop_of(&view).lines.iter();
    lines
        .map(|l| (l.name.clone(), l.gold.clone(), l.enabled))
        .collect()
}

#[test]
fn the_shop_sells_chapter_fives_basics_and_shows_the_gold() {
    let mut c = ctx();
    let mut s = screen(&c);
    in_shop(&mut c, &mut s, 0);
    let view = s.view(&c);
    let shop = shop_of(&view);
    assert_eq!((&*shop.shop, &*shop.gold), ("Shop", "Gold 3 000"));
    assert_eq!(shop.tabs, ["Buy", "Sell", "Repair"]);
    let names: Vec<_> = shop.lines.iter().map(|l| l.name.as_str()).collect();
    // Iron and Steel, the Potion, the four armours; no Hi-Potion yet.
    assert_eq!(names.len(), 15);
    assert!(names.contains(&"Steel Bow") && names.contains(&"Iron Plate"));
    assert!(!names.contains(&"Hi-Potion"));
    assert_eq!(shop.lines[0].name, "Potion");
    assert_eq!(shop.lines[0].gold, "300");
    assert_eq!(shop.lines[0].detail, "Restore 20 HP");
    assert_eq!(
        view.help,
        "Up Down choose · Left Right buy, sell, repair · f confirm · d back"
    );
    // Cancel goes back to the tabs.
    update(&mut s, &mut c, &[Cancel]);
    assert_eq!(s.focus(), Focus::Tabs);
}

#[test]
fn buying_takes_the_gold_and_the_item_goes_to_the_pack_tabs_stock() {
    let mut c = ctx();
    let mut s = screen(&c);
    in_shop(&mut c, &mut s, 0);
    assert_eq!(sounds(&mut s, &mut c, &[Confirm]), ["menu_select"]);
    assert_eq!(s.setup().gold, 2700);
    assert_eq!(s.setup().stock.count(&potion()), 7);
    let gold = |s: &PreparationsScreen| shop_of(&s.view(&c)).gold.clone();
    assert_eq!(gold(&s), "Gold 2 700");
    // On the Pack tab's stock list.
    update(&mut s, &mut c, &[Cancel, CursorLeft]);
    assert_eq!(s.spare(), [(ItemId::new("elixir"), 2), (potion(), 7)]);
}

#[test]
fn what_the_party_cannot_afford_is_dimmed_and_refused() {
    let mut c = ctx();
    let mut s = screen(&c);
    s.prep.setup.gold = 400;
    in_shop(&mut c, &mut s, 0);
    let lines = shop_lines(&s, &c);
    let line = |name: &str| lines.iter().find(|l| l.0 == name).cloned();
    assert_eq!(line("Potion").map(|l| l.2), Some(true));
    assert_eq!(line("Leather Vest").map(|l| l.2), Some(true));
    assert_eq!(
        line("Iron Sword").map(|l| (l.1, l.2)),
        Some(("460".into(), false))
    );
    // Down to the Iron Sword: refused, nothing changes.
    let at = lines.iter().position(|l| l.0 == "Iron Sword").unwrap();
    let before = s.prep.clone();
    for _ in 0..at {
        update(&mut s, &mut c, &[CursorDown]);
    }
    assert_eq!(s.deal_focus(), at);
    assert_eq!(sounds(&mut s, &mut c, &[Confirm]), ["menu_cancel"]);
    assert_eq!(s.prep, before);
}

#[test]
fn selling_asks_how_many_then_pays_half_price() {
    let mut c = ctx();
    let mut s = screen(&c);
    in_shop(&mut c, &mut s, 1);
    assert_eq!(s.shop_tab(), crate::screens::battle::shop::Tab::Sell);
    let lines = shop_lines(&s, &c);
    // The stock's weapons first, each copy a line with its durability,
    // then its items counted.
    assert_eq!(lines[0].0, "Steel Spear");
    let view = s.view(&c);
    let first = &shop_of(&view).lines[0].detail;
    assert!(first.starts_with("25/25  Mt 10"), "{first}");
    let at = lines.iter().position(|l| l.0 == "Potion ×6").unwrap();
    assert_eq!(lines[at].1, "150");
    for _ in 0..at {
        update(&mut s, &mut c, &[CursorDown]);
    }
    // Up to 4, back down to 3.
    update(&mut s, &mut c, &[Confirm, CursorUp, CursorUp, CursorUp]);
    assert_eq!(s.selling().map(|sale| sale.count), Some(4));
    assert_eq!(sounds(&mut s, &mut c, &[CursorDown]), ["menu_move"]);
    let view = s.view(&c);
    let selling = shop_of(&view).selling.clone().unwrap();
    assert_eq!((selling.count, selling.max), (3, 6));
    assert_eq!(selling.gold, "For 450 gold");
    assert_eq!(view.help, "Up Down how many · f sell · d back");
    assert_eq!(sounds(&mut s, &mut c, &[Confirm]), ["menu_select"]);
    assert_eq!(s.selling(), None);
    assert_eq!(s.setup().gold, 3450);
    assert_eq!(s.setup().stock.count(&potion()), 3);
    // Cancel on the question sells nothing.
    update(&mut s, &mut c, &[Confirm, Cancel]);
    assert_eq!((s.selling(), s.setup().gold), (None, 3450));
    // A weapon copy sells alone: the first line, asked about once.
    let weapons = s.setup().stock.weapons.len();
    while s.deal_focus() > 0 {
        update(&mut s, &mut c, &[CursorUp]);
    }
    update(&mut s, &mut c, &[Confirm, CursorUp]);
    assert_eq!(s.selling().map(|sale| sale.count), Some(1));
    update(&mut s, &mut c, &[Confirm]);
    assert_eq!(s.setup().stock.weapons.len(), weapons - 1);
}

#[test]
fn repairing_a_worn_weapon_costs_its_repair_and_fills_it() {
    let mut c = ctx();
    let mut s = screen(&c);
    // The lord's first weapon and the stock's first, worn.
    let lord = s.prep.setup.units[0].id;
    let worn = s.prep.setup.units[0].loadout.weapons[0].as_mut().unwrap();
    worn.durability_left = 5;
    let worn = worn.clone();
    s.prep.setup.stock.weapons[0].durability_left = 1;
    let items = s.prep.setup.items.clone();
    let def = items.weapon(&worn.def).unwrap().clone();
    let cost = trpg_core::repair_cost(&def, &worn);
    in_shop(&mut c, &mut s, 2);
    let view = s.view(&c);
    let lines = &shop_of(&view).lines;
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].name, "Steel Spear");
    assert!(lines[0].detail.ends_with("Stock"), "{}", lines[0].detail);
    let max = def.durability;
    assert_eq!(lines[1].detail, format!("5/{max} → {max}/{max}  Test Lord"));
    assert_eq!(lines[1].gold, cost.to_string());
    update(&mut s, &mut c, &[CursorDown, Confirm]);
    let held = s.prep.weapon(WeaponAt::Held(PrepUnit::Deployed(lord), 0));
    assert_eq!(held.map(|w| w.durability_left), Some(max));
    assert_eq!(s.setup().gold, 3000 - cost);
    // It is off the list; the highlight stays on a line.
    assert_eq!((shop_lines(&s, &c).len(), s.deal_focus()), (1, 0));
}

#[test]
fn a_battle_with_no_shop_says_so_and_does_not_open_it() {
    let mut c = ctx();
    let mut s = screen(&c);
    s.prep.shop = None;
    update(&mut s, &mut c, &[CursorRight, CursorRight]);
    let view = s.view(&c);
    let shop = shop_of(&view);
    assert_eq!(shop.lines.len(), 0);
    assert_eq!(shop.empty, "No shop before this battle.");
    assert_eq!(sounds(&mut s, &mut c, &[Confirm]), ["menu_cancel"]);
    assert_eq!(s.focus(), Focus::Tabs);
}

#[test]
fn shop_tab_snapshot() {
    let mut h = Harness::with_layout_and(RIGHT_HANDED, |c| {
        // Little gold, so the dearer gear is dimmed.
        let quick = c.game.content.battles.get_mut(QUICK_BATTLE);
        quick.expect("the Quick Battle").solo_gold = 450;
    });
    h.keys("Down f");
    h.keys("Right Right f Down Down");
    assert_snapshot!(h.snapshot());
}

/// Ticket 0443's harness flow: a Potion bought on the Quick Battle's
/// Preparations is packed and fought with, and a win leaves the campaign
/// the gold left after it.
#[test]
fn a_potion_bought_on_preparations_is_in_the_battle_and_its_price_off_the_gold() {
    // The Quick Battle without its enemies: won once a unit acts.
    let mut h = Harness::with_layout_and(RIGHT_HANDED, |c| {
        let quick = c.game.content.battles.get_mut(QUICK_BATTLE);
        let quick = quick.expect("the Quick Battle");
        quick.enemies.clear();
        quick.reinforcements.clear();
    });
    h.keys("Down f");
    // Shop, Buy: the Potion is the first line.
    h.keys("Right Right f f");
    let prep = h
        .game()
        .screen::<PreparationsScreen>()
        .expect("Preparations");
    assert_eq!(prep.setup().gold, 2700);
    assert_eq!(prep.setup().stock.count(&potion()), 7);
    // Pack it (the Pack tab's second row), then Fight!.
    h.keys("d Left f Down f d Left Left f");
    let battle = h.battle().expect("the battle");
    assert_eq!(battle.state().pack().items, [potion()]);
    assert_eq!(battle.state().gold(), 2700);
    let lord = &battle.state().units()[0];
    let wait = trpg_core::Command::Act {
        unit: lord.id,
        dest: lord.pos,
        action: trpg_core::UnitAction::Wait,
    };
    h.battle_mut().expect("the battle").send(&wait);
    let state = h.battle().expect("the battle").state().clone();
    assert_eq!(state.outcome(), Some(trpg_core::Outcome::Victory));
    // The campaign as the win leaves it: the battle's gold, plus the
    // clear gold.
    let campaign = h.flow().and_then(|f| f.campaign().cloned());
    let def = h.game().ctx().game.content.battles[QUICK_BATTLE].clone();
    let mut campaign = campaign.expect("a campaign");
    assert!(campaign.apply_result(&def, &state, 0).is_ok());
    assert_eq!(campaign.gold, 2700 + def.clear_gold);
    assert_eq!(
        campaign.stock.count(&potion()),
        7,
        "the packed one came back"
    );
}
