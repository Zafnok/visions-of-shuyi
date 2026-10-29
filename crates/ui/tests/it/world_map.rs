//! Scripted tests of the world map (ticket 1007) through the real game, on
//! the test world map (`assets-test/worldmaps/`): New Game starts at its
//! first chapter. Travel to a town, shop, leave, save, and load it again;
//! travel to a story battle, win it, and find the map with its new places
//! open; win the act's final battle and land on the next act's map.

use insta::assert_snapshot;
use trpg_core::{Campaign, CharacterId, Command, ItemId, LEAD_ID, Pos, UnitAction, UnitId};
use trpg_ui::ShuyiCtx;
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::screens::WorldMapScreen;
use trpg_ui::{Ctx, MemoryStorage, Storage};

/// The lead's unit in the test battle (its first slot), and the fort it
/// seizes to win.
const LEAD: UnitId = UnitId(1);
const FORT: Pos = Pos::new(5, 5);

/// A launch with the right-handed layout picked, on `storage`, whose New
/// Game starts the test world map's first chapter with the lead, the test
/// knight and 2 000 gold.
fn launch(storage: Box<dyn Storage>) -> Harness {
    let mut storage = storage;
    if let Err(e) = storage.write("layout", RIGHT_HANDED.name()) {
        panic!("saving the layout: {e}");
    }
    let mut ctx = Ctx::embedded()
        .unwrap_or_else(|e| panic!("{e}"))
        .with_storage(storage);
    ctx.game.content.new_game = trpg_content::NewGameDef {
        first_chapter: "test_wm1".into(),
        roster: vec![
            CharacterId(LEAD_ID.into()),
            CharacterId("test_knight".into()),
        ],
        gold: 2000,
        stock: Vec::new(),
    };
    Harness::from_game(trpg_ui::game::start(ctx))
}

/// From the title: New Game in Classic with the default lead, to the map.
fn new_game(h: &mut Harness) {
    h.keys("f f f Up f");
    assert_eq!(h.screens(), ["title", "game_flow", "world_map"]);
}

/// The world map on the stack.
fn map(h: &Harness) -> &WorldMapScreen {
    h.game()
        .screen::<WorldMapScreen>()
        .unwrap_or_else(|| panic!("no world map"))
}

/// The campaign as the world map has it.
fn campaign(h: &Harness) -> Campaign {
    map(h).campaign().clone()
}

/// Where the army is.
fn at(h: &Harness) -> String {
    let world = campaign(h).world.unwrap_or_else(|| panic!("not on a map"));
    world.at
}

/// Lets the army walk to where it was sent (and a battle there start).
fn walk(h: &mut Harness) {
    h.wait(2.0);
    if let Some(map) = h.game().screen::<WorldMapScreen>() {
        assert!(!map.walking());
    }
}

/// Skips the scene on screen: Cancel, then Confirm on "Skip scene?".
fn skip_scene(h: &mut Harness) {
    assert_eq!(h.top_screen(), "dialogue");
    h.keys("d f");
}

/// From the battle's start (its notes and banner), wins the test battle,
/// through its results and victory scene.
fn win(h: &mut Harness) {
    assert_eq!(h.top_screen(), "battle");
    h.keys("f f");
    let seize = Command::Act {
        unit: LEAD,
        dest: FORT,
        action: UnitAction::Seize,
    };
    let battle = h.battle_mut().unwrap_or_else(|| panic!("no battle"));
    battle.send(&seize);
    h.keys("f");
    assert_eq!(h.top_screen(), "results");
    h.keys("f f");
    skip_scene(h);
}

/// Travel to a town, open its Vendor, buy a potion, leave; travel on and
/// save on the map; the next launch loads the same place, gold and stock.
#[test]
fn shop_in_a_town_then_save_on_the_map_and_load_it_again() {
    let mut h = launch(Box::new(MemoryStorage::new()));
    new_game(&mut h);
    // The army starts in Millbrook: Confirm opens it, its two shops listed.
    assert_eq!(at(&h), "home");
    assert_snapshot!("world_map_start", h.snapshot());
    h.keys("f");
    assert_snapshot!("world_map_town_list", h.snapshot());
    h.keys("Down f");
    assert_eq!(h.top_screen(), "town_shop");
    h.keys("f d");
    assert_eq!(h.top_screen(), "world_map");
    let potion = ItemId::new("potion");
    assert_eq!(campaign(&h).gold, 1700);
    assert_eq!(campaign(&h).stock.count(&potion), 1);
    // Next place (in the map's order): the lighthouse, then the
    // crossroads; travel there.
    h.keys("s s f");
    walk(&mut h);
    assert_eq!(at(&h), "cross");
    // The menu: Save, into the first empty slot.
    h.keys("d");
    assert_snapshot!("world_map_menu", h.snapshot());
    h.keys("f f");
    assert_eq!(h.top_screen(), "world_map");
    let saved = campaign(&h);

    let mut h = launch(h.into_storage());
    h.keys("Down f");
    assert_eq!(h.screens(), ["title", "game_flow", "save_slots"]);
    h.keys("f");
    assert_eq!(h.screens(), ["title", "game_flow", "world_map"]);
    let loaded = campaign(&h);
    assert_eq!(loaded.world, saved.world);
    assert_eq!(loaded.gold, 1700);
    assert_eq!(loaded.stock, saved.stock);
    assert_eq!(loaded.chapter, "test_wm1");
}

/// Travel to the story battle, win it: back on the map, the place cleared
/// and the places that waited for it open; win the act's final battle:
/// the next act's map, the old one gone.
#[test]
fn win_a_story_battle_then_the_acts_final_one() {
    let mut h = launch(Box::new(MemoryStorage::new()));
    new_game(&mut h);
    // Find → Story battles: the cursor on Stone Mill; travel there.
    h.keys("x Down Down f f");
    walk(&mut h);
    // Battles are forced: its chapter's intro scene, then the battle.
    skip_scene(&mut h);
    win(&mut h);
    assert_eq!(h.screens(), ["title", "game_flow", "world_map"]);
    let won = campaign(&h);
    let world = won.world.clone().unwrap_or_else(|| panic!("not on a map"));
    assert!(world.cleared.contains("ford"));
    assert_eq!(world.at, "ford");
    assert_eq!(won.chapter, "test_wm2");
    let content = &h.game().ctx().game.content;
    let def = &content.worldmaps["test_world"].map;
    for place in ["pass", "chapel", "bridge"] {
        assert!(def.is_open(place, &won), "{place} opened");
    }
    assert_snapshot!("world_map_after_a_win", h.snapshot());
    // On to Old Bridge, the act's final battle.
    h.keys("Right f");
    walk(&mut h);
    skip_scene(&mut h);
    win(&mut h);
    assert_eq!(h.screens(), ["title", "game_flow", "world_map"]);
    let world = campaign(&h).world.unwrap_or_else(|| panic!("not on a map"));
    assert_eq!(
        (world.map.as_str(), world.at.as_str()),
        ("test_world_2", "port")
    );
    assert!(world.cleared.is_empty());
}
