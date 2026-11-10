//! Scripted tests of the world map (ticket 1007) through the real game, on
//! the test world map (`assets-test/worldmaps/`): New Game starts at its
//! first chapter. Travel to a town, shop, leave, save, and load it again;
//! travel to a story battle, win it, and find the map with its new places
//! open; win the act's final battle and land on the next act's map.

use insta::assert_snapshot;
use trpg_core::{
    Campaign, CharacterId, Command, ItemId, LEAD_ID, Pos, SaveFile, SavePoint, UnitAction, UnitId,
};
use trpg_ui::ShuyiCtx;
use trpg_ui::harness::Harness;
use trpg_ui::input::{PadKind, RIGHT_HANDED};
use trpg_ui::save::slot_key;
use trpg_ui::screen::SkinSet;
use trpg_ui::screens::{TownScreen, WorldMapScreen};
use trpg_ui::{Ctx, MemoryStorage, Storage};

/// The lead's unit in the test battle (its first slot), and the fort it
/// seizes to win.
const LEAD: UnitId = UnitId(1);
const FORT: Pos = Pos::new(5, 5);

/// A launch with the right-handed layout picked, on `storage`, whose New
/// Game starts the test world map's first chapter with the lead, the test
/// knight and 2 000 gold.
fn launch(storage: Box<dyn Storage>) -> Harness {
    launch_with(storage, |_| {})
}

/// [`launch`], with `edit` made to the game's context first.
fn launch_with(storage: Box<dyn Storage>, edit: impl FnOnce(&mut Ctx)) -> Harness {
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
    edit(&mut ctx);
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

/// Find rebound on the Key bindings screen (ticket 1198): its row is three
/// `Up`s from the first (past the switch and Restore defaults), and the new
/// key in its second slot opens the list on the map.
#[test]
fn a_key_rebound_to_find_opens_it_on_the_map() {
    let mut h = launch(Box::new(MemoryStorage::new()));
    h.keys("Down Down f Up Up Up f");
    assert_eq!(h.screens(), ["title", "options", "key_bindings"]);
    h.keys("Up Up Up Right f v d d");
    assert_eq!(h.screens(), ["title"]);
    h.keys("Up Up");
    new_game(&mut h);
    assert!(map(&h).view(h.game().ctx()).list.is_none());
    h.keys("v");
    let title = h.game().ctx().text("world_map.find.title").to_owned();
    let list = map(&h).view(h.game().ctx()).list;
    assert_eq!(list.map(|l| l.title), Some(title));
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
    h.keys("f");
    let shop = h.game().screen::<TownScreen>();
    let shop = shop.unwrap_or_else(|| panic!("no town shop"));
    assert_eq!(shop.campaign().gold, 1700, "the potion bought");
    h.keys("d");
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
    // Wolf Pass, the side quest: its scene, its battle; back on the map,
    // cleared, the chapter unchanged.
    h.keys("Up f");
    walk(&mut h);
    skip_scene(&mut h);
    assert_eq!(h.top_screen(), "battle");
    h.keys("f f");
    let seize = Command::Act {
        unit: LEAD,
        dest: FORT,
        action: UnitAction::Seize,
    };
    let battle = h.battle_mut().unwrap_or_else(|| panic!("no battle"));
    battle.send(&seize);
    h.keys("f f f");
    assert_eq!(h.screens(), ["title", "game_flow", "world_map"]);
    let quest = campaign(&h);
    let world = quest
        .world
        .clone()
        .unwrap_or_else(|| panic!("not on a map"));
    assert!(world.cleared.contains("pass"));
    assert_eq!(quest.chapter, "test_wm2");
    // On to Old Bridge, the act's final battle: Wolf Pass's road back to
    // Stone Mill, then Stone Mill's road east.
    h.keys("Right Right f");
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

/// A story battle whose next chapter has no world map: winning it asks
/// "Save your progress?" (0869), then starts that chapter, its intro scene
/// first, instead of going back to the map. The save made there is a
/// chapter save, as at the end of a chapter off the map, and loads into
/// that chapter too.
#[test]
fn a_win_whose_next_chapter_has_no_map_starts_it() {
    let next_off_map = |ctx: &mut Ctx| {
        let chapter = ctx.game.content.chapters.get_mut("test_wm1");
        chapter.unwrap_or_else(|| panic!("no test_wm1")).next = Some("test".into());
    };
    let mut h = launch_with(Box::new(MemoryStorage::new()), next_off_map);
    new_game(&mut h);
    h.keys("x Down Down f f");
    walk(&mut h);
    skip_scene(&mut h);
    win(&mut h);
    assert_eq!(h.screens(), ["title", "game_flow", "save_prompt"]);
    // Yes, into the first empty slot.
    h.keys("f f");
    assert_eq!(h.top_screen(), "dialogue");
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    assert_eq!(flow.chapter().map(|c| c.id.as_str()), Some("test"));
    let saved = saved(&h);
    assert_eq!(saved.point, SavePoint::ChapterCleared);
    assert_eq!(saved.campaign.chapter, "test_wm1", "the chapter just won");

    let mut h = launch_with(h.into_storage(), next_off_map);
    h.keys("Down f f");
    assert_eq!(h.top_screen(), "dialogue");
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    assert_eq!(flow.chapter().map(|c| c.id.as_str()), Some("test"));
}

/// [`launch`], with no chapter after Stone Mill's (as Act 1's last is).
fn launch_last(storage: Box<dyn Storage>) -> Harness {
    launch_with(storage, |ctx| {
        let chapter = ctx.game.content.chapters.get_mut("test_wm1");
        chapter.unwrap_or_else(|| panic!("no test_wm1")).next = None;
    })
}

/// From New Game, Stone Mill won with no chapter after it: "Save your
/// progress?".
fn win_the_last(h: &mut Harness) {
    new_game(h);
    h.keys("x Down Down f f");
    walk(h);
    skip_scene(h);
    win(h);
    assert_eq!(h.screens(), ["title", "game_flow", "save_prompt"]);
}

/// The save in the first slot.
fn saved(h: &Harness) -> SaveFile {
    let read = trpg_ui::save::read(&h.game().ctx().storage, &slot_key(1));
    let read = read.unwrap_or_else(|e| panic!("{e:?}"));
    read.unwrap_or_else(|| panic!("nothing saved"))
}

/// The last story battle won (0869): "Save your progress?"; Yes, the
/// slots, saved, "To be continued". The save is the map with Stone Mill
/// cleared and the army as it won, and loads there.
#[test]
fn win_the_last_story_battle_then_save_on_the_map() {
    let mut h = launch_last(Box::new(MemoryStorage::new()));
    win_the_last(&mut h);
    let won = h
        .flow()
        .and_then(|f| f.campaign().cloned())
        .unwrap_or_else(|| panic!("no campaign"));
    h.keys("f");
    assert_eq!(h.screens(), ["title", "game_flow", "save_slots"]);
    h.keys("f");
    assert_eq!(h.screens(), ["title", "game_flow", "to_be_continued"]);
    let saved = saved(&h);
    assert_eq!(saved.point, SavePoint::WorldMap);
    let world = saved.campaign.world.clone();
    let world = world.unwrap_or_else(|| panic!("not on a map"));
    assert!(world.cleared.contains("ford"));
    assert_eq!(world.at, "ford");
    assert_eq!(saved.campaign.roster, won.roster);
    assert_eq!(saved.campaign.stock, won.stock);
    assert_eq!(saved.campaign.gold, won.gold);

    // Load it from the title: the map, Stone Mill cleared, its places open.
    let mut h = launch_last(h.into_storage());
    h.keys("Down f f");
    assert_eq!(h.screens(), ["title", "game_flow", "world_map"]);
    let loaded = campaign(&h);
    assert_eq!(loaded.world, saved.campaign.world);
    let content = &h.game().ctx().game.content;
    let def = &content.worldmaps["test_world"].map;
    assert!(def.is_open("pass", &loaded), "Wolf Pass opened");
}

/// The last story battle won, "Save your progress?" No: "To be continued",
/// nothing saved.
#[test]
fn win_the_last_story_battle_then_decline_the_save() {
    let mut h = launch_last(Box::new(MemoryStorage::new()));
    win_the_last(&mut h);
    h.keys("Down f");
    assert_eq!(h.screens(), ["title", "game_flow", "to_be_continued"]);
    let read = trpg_ui::save::read(&h.game().ctx().storage, &slot_key(1));
    assert!(matches!(read, Ok(None)), "nothing saved");
}

/// Act 1's real map (ticket 1070), from a save at `at` with `cleared` won
/// and the given chapter's roster: the map as a player finds it.
fn act1_from_a_save(at: &str, cleared: &[&str]) -> Harness {
    act1_at_chapter("test_wm1", at, cleared)
}

/// [`act1_from_a_save`], the campaign at chapter `chapter`.
fn act1_at_chapter(chapter: &str, at: &str, cleared: &[&str]) -> Harness {
    let mut storage: Box<dyn Storage> = Box::new(MemoryStorage::new());
    let ctx = Ctx::embedded().unwrap_or_else(|e| panic!("{e}"));
    let content = &ctx.game.content;
    let lead = trpg_core::LeadProfile::new("Mara", trpg_core::LeadGender::Female);
    let mut c = trpg_content::battle_campaign(
        content,
        &content.battles["test"],
        trpg_core::GameMode::Classic,
        lead,
    );
    c.chapter = chapter.into();
    c.world = Some(trpg_core::WorldState {
        map: "act1".into(),
        at: at.into(),
        cleared: cleared.iter().map(|&s| s.to_owned()).collect(),
        ..trpg_core::WorldState::default()
    });
    let file = trpg_core::SaveFile::world_map(c);
    trpg_ui::save::write(storage.as_mut(), &trpg_ui::save::slot_key(1), &file)
        .unwrap_or_else(|e| panic!("{e:?}"));
    let mut h = launch(storage);
    // The title's Load Game, the first slot.
    h.keys("Down f f");
    assert_eq!(h.screens(), ["title", "game_flow", "world_map"]);
    h
}

/// Travel between Chapter 4's open places; what isn't built yet shows and
/// can't be reached. Aldwater Toll Bridge is a place to pass through until
/// 1071 builds its fight (0850); Veyne Hall, Chapter 4's story battle,
/// waits for its chapter.
#[test]
fn act1_the_army_walks_chapter_4s_open_places() {
    let mut h = act1_from_a_save("greywater", &[]);
    let start = campaign(&h);
    let content = &h.game().ctx().game.content;
    let def = &content.worldmaps["act1"].map;
    let open: Vec<&str> = def.open_nodes(&start).map(|n| n.id.as_str()).collect();
    assert_eq!(open, ["greywater", "valley_road", "millhaven", "aldwater"]);
    for shown in ["millhaven_road", "company_ledger", "veyne_hall"] {
        assert!(def.is_shown(shown, &start), "{shown} shows, greyed");
        assert!(!def.is_open(shown, &start), "{shown} isn't open");
    }
    assert_snapshot!("act1_start", h.snapshot());
    // Next place: The Valley Road, then Millhaven.
    h.keys("s f");
    walk(&mut h);
    assert_eq!(at(&h), "valley_road");
    h.keys("s f");
    walk(&mut h);
    assert_eq!(at(&h), "millhaven");
}

/// The stock of each of the first `shops` shops of the town the army is in,
/// opened by Confirm from the map as a save lands on it, and left.
fn town_stock(h: &mut Harness, shops: usize) -> Vec<Vec<String>> {
    let mut stocks = Vec::new();
    for i in 0..shops {
        if map(h).view(h.game().ctx()).list.is_some() {
            h.keys("d");
        }
        h.keys("f");
        for _ in 0..i {
            h.keys("Down");
        }
        h.keys("f");
        assert_eq!(h.top_screen(), "town_shop");
        let shop = h.game().screen::<TownScreen>();
        let shop = shop.unwrap_or_else(|| panic!("no town shop"));
        stocks.push(shop.shop().stock.iter().map(|i| i.0.clone()).collect());
        h.keys("d");
    }
    stocks
}

/// Each town's shops open with their stock, and the stock grows with the
/// story: Saltmere's last stock for Chapter 8 adds the Hi-Potion.
#[test]
fn act1_each_towns_shops_open_with_their_stock() {
    let mut h = act1_from_a_save("millhaven", &[]);
    let stocks = town_stock(&mut h, 3);
    assert!(stocks[0].contains(&"iron_sword".to_owned()));
    assert!(!stocks[0].contains(&"steel_sword".to_owned()), "Chapter 4");
    assert_eq!(stocks[1], ["potion"]);
    assert!(stocks[2].is_empty(), "a Blacksmith sells nothing");

    let mut h = act1_from_a_save("millhaven", &["veyne_hall"]);
    assert!(town_stock(&mut h, 1)[0].contains(&"steel_sword".to_owned()));

    let mut h = act1_from_a_save("saltmere", &["veyne_hall", "salt_road"]);
    let stocks = town_stock(&mut h, 5);
    assert!(stocks[0].contains(&"steel_bow".to_owned()));
    assert_eq!(stocks[1], ["potion"]);

    let mut h = act1_from_a_save("saltmere", &["undercroft", "salt_road"]);
    assert_eq!(town_stock(&mut h, 2)[1], ["potion", "hi_potion"]);

    let mut h = act1_from_a_save("cairnford", &["envoys_house"]);
    assert!(town_stock(&mut h, 3)[0].contains(&"steel_axe".to_owned()));

    let mut h = act1_from_a_save("ossary", &["pilgrims_road"]);
    assert!(town_stock(&mut h, 3)[2].contains(&"linen_robe".to_owned()));
}

/// The cursor follows the roads on Act 1's map at Chapter 4 (1088, Nick's
/// playtest: it "kinda just goes wherever it wants"): Millhaven has no
/// road east; Aldwater's road east goes to Veyne Hall, and Left from there
/// goes back to Aldwater, not to The Captain's Oath.
#[test]
fn act1_the_cursor_goes_where_the_roads_go() {
    let mut h = act1_at_chapter("ch04", "millhaven", &[]);
    let cursor = |h: &Harness| map(h).cursor().to_owned();
    h.keys("Right");
    assert_eq!(cursor(&h), "millhaven", "no road east of Millhaven");
    h.keys("Down");
    assert_eq!(cursor(&h), "valley_road");
    h.keys("Right");
    assert_eq!(cursor(&h), "aldwater");
    h.keys("Down");
    assert_eq!(cursor(&h), "aldwater", "no road south of Aldwater");
    h.keys("Right");
    assert_eq!(cursor(&h), "veyne_hall");
    h.keys("Left");
    assert_eq!(cursor(&h), "aldwater");
    h.keys("Up");
    assert_eq!(cursor(&h), "veyne_hall", "the road east and north");
    h.keys("Up");
    assert_eq!(cursor(&h), "captains_oath");
}

/// Ticket 1066, on the Playdate (its console, inks, skins and buttons):
/// New Game to the map; A on the army's town lists its shops, the Vendor,
/// a potion bought, B leaves; the D-pad and A travel to the crossroads;
/// B's menu (Find and Your army first, they have no button) saves; the
/// next launch loads the same place, gold and stock. Every screen on the
/// way is painted by its Playdate skin.
#[test]
fn on_the_playdate_shop_in_a_town_then_save_on_the_map_and_load_it_again() {
    let playdate = crate::playdate::playdate_skins;
    let mut h = launch_with(Box::new(MemoryStorage::new()), playdate);
    h.use_pad(PadKind::Playdate);
    // The title's wait, New Game, Classic, the familiarity answer; Start.
    h.pad("East East East East DpadUp East");
    assert_eq!(h.screens(), ["title", "game_flow", "world_map"]);
    let ctx = h.game().ctx();
    assert_eq!(ctx.game.skin_set, SkinSet::Playdate);
    // The place's window: at the bottom here, which hides no place.
    let start = h.game().buffer();
    assert!((0..15).any(|y| row_of(start, y).contains("╔═ Millbrook")));
    h.pad("East");
    let list = map(&h).view(h.game().ctx()).list;
    assert_eq!(list.map(|l| l.title).as_deref(), Some("Millbrook"));
    assert!(row_of(h.game().buffer(), 0).contains("╔═ Millbrook"));
    h.pad("DpadDown East");
    assert_eq!(h.top_screen(), "town_shop");
    assert!(row_of(h.game().buffer(), 0).starts_with("╔═ Vendor"));
    h.pad("East");
    let shop = h.game().screen::<TownScreen>();
    let shop = shop.unwrap_or_else(|| panic!("no town shop"));
    assert_eq!(shop.campaign().gold, 1700, "the potion bought");
    h.pad("West");
    assert_eq!(h.top_screen(), "world_map");
    assert_eq!(campaign(&h).gold, 1700);
    // Right follows the road to the crossroads; A travels there.
    h.pad("DpadRight East");
    walk(&mut h);
    assert_eq!(at(&h), "cross");
    h.pad("West");
    let menu = map(&h).view(h.game().ctx()).list;
    let menu = menu.unwrap_or_else(|| panic!("no menu")).menu;
    let labels: Vec<String> = menu
        .items
        .iter()
        .map(|i| h.game().ctx().label(&i.label))
        .collect();
    assert_eq!(labels, ["Find", "Your army", "Save", "Camp", "Options"]);
    h.pad("DpadDown DpadDown East East");
    assert_eq!(h.top_screen(), "world_map");
    let saved = campaign(&h);

    let mut h = launch_with(h.into_storage(), playdate);
    h.use_pad(PadKind::Playdate);
    // The title's wait, Load Game, the first slot.
    h.pad("East DpadDown East");
    assert_eq!(h.top_screen(), "save_slots");
    h.pad("East");
    assert_eq!(h.screens(), ["title", "game_flow", "world_map"]);
    let loaded = campaign(&h);
    assert_eq!(loaded.world, saved.world);
    assert_eq!(loaded.gold, 1700);
    assert_eq!(loaded.stock, saved.stock);
    let loaded_map = h.game().buffer();
    assert!((0..15).any(|y| row_of(loaded_map, y).contains("╔═ Crossroads")));
}

/// Row `y` of `buf`'s glyphs.
fn row_of(buf: &trpg_ui::GlyphBuffer, y: i32) -> String {
    (0..i32::from(buf.width()))
        .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
        .collect()
}
