use std::sync::LazyLock;

use proptest::prelude::*;
use trpg_content::battle::from_source;
use trpg_content::map::parse_map;
use trpg_content::worldmap::{self, WorldRefs};

use super::*;

static CONTENT: LazyLock<Content> =
    LazyLock::new(|| trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}")));

fn world() -> &'static WorldMapDef {
    CONTENT
        .worldmaps
        .get("test_world")
        .expect("the test world map")
}

fn request() -> Request {
    Request {
        map: "test_world".to_owned(),
        at: (10, 10),
        level: 35,
        units: Units::Mix,
        chapters: Vec::new(),
        kind: Kind::Fixed,
        tier: "Normal".to_owned(),
        seed: 7,
        name: "Skirmish".to_owned(),
    }
}

/// The content with `made`'s map in it, as `cargo xtask skirmish` has it.
fn with_map(made: &Generated) -> Content {
    let mut content = (*CONTENT).clone();
    let display = &content.terrain.display;
    let map = parse_map(&made.id, &made.map_text, display)
        .unwrap_or_else(|e| panic!("{e:?}\n{}", made.map_text));
    content.maps.insert(made.id.clone(), map);
    content
}

fn load(made: &Generated) -> trpg_core::BattleDef {
    let content = with_map(made);
    from_source(
        "generated",
        &made.id,
        &made.battle_text,
        &content.battle_refs(),
    )
    .unwrap_or_else(|e| panic!("{e:?}\n{}", made.battle_text))
}

fn generated(request: &Request) -> Generated {
    generate(&CONTENT, world(), request).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn a_mixed_lv_35_battle_passes_validation_with_enemies_around_its_level() {
    let def = load(&generated(&request()));
    assert_eq!(def.enemies.len(), ENEMIES as usize);
    assert!(def.enemies.iter().all(|u| u.level.abs_diff(35) <= 1));
    let tiers = def
        .enemies
        .iter()
        .map(|u| CONTENT.classes.get(&u.class).map(|c| c.tier));
    assert!(tiers.into_iter().all(|t| t == Some(tier_at(35))));
    let army = def.bots_army.expect("the bots' army");
    assert_eq!(army.level, 35);
}

#[test]
fn only_mages_means_only_mages() {
    let request = Request {
        units: Units::Only("mage".to_owned()),
        chapters: vec!["ch02".to_owned()],
        level: 40,
        ..request()
    };
    let made = generated(&request);
    let def = load(&made);
    assert!(def.enemies.iter().all(|u| u.class.0 == "mage"));
    assert_eq!(made.army.chapter, "ch02");
}

#[test]
fn a_weapon_type_picks_the_classes_that_fight_with_it() {
    let names = templates(&CONTENT, &Units::Only("Axe".to_owned()), 1).expect("axe classes");
    assert!(names.contains(&"skirmish_raider".to_owned()));
    assert!(!names.contains(&"skirmish_mage".to_owned()));
    assert!(templates(&CONTENT, &Units::Only("Iron Rider".to_owned()), 1).is_ok());
    assert!(templates(&CONTENT, &Units::Only("nobody".to_owned()), 1).is_err());
}

#[test]
fn every_template_can_fight_and_is_never_a_lords_class() {
    for (id, generic) in &CONTENT.characters.generics {
        if !id.starts_with(TEMPLATES) {
            continue;
        }
        let class = CONTENT.classes.get(&generic.class).expect("its class");
        assert!(!class.lord_only, "{id}");
        assert!(
            !generic.loadout.weapons.is_empty() || casts(&CONTENT, class),
            "{id} has no way to attack"
        );
    }
}

#[test]
fn the_same_request_and_seed_make_the_same_files_and_another_seed_doesnt() {
    assert_eq!(generated(&request()), generated(&request()));
    let other = Request {
        seed: 8,
        ..request()
    };
    assert_ne!(
        generated(&request()).battle_text,
        generated(&other).battle_text
    );
}

#[test]
fn the_map_takes_its_ground_from_the_world_map_around_the_cell() {
    // 17,5 is deep in the test world's forest; 37,6 on open plain.
    let forest = generated(&Request {
        at: (17, 5),
        ..request()
    });
    let plain = generated(&Request {
        at: (37, 6),
        ..request()
    });
    let trees = |m: &Generated| {
        m.map_text
            .split("---")
            .nth(1)
            .map_or(0, |t| t.matches('T').count())
    };
    assert!(trees(&forest) > 10, "{}", forest.map_text);
    assert_eq!(trees(&plain), 0, "{}", plain.map_text);
}

#[test]
fn a_cell_off_the_map_or_at_sea_or_a_wrong_level_or_chapter_is_refused() {
    let refused = |r: Request| generate(&CONTENT, world(), &r).is_err();
    assert!(refused(Request {
        at: (500, 3),
        ..request()
    }));
    assert!(refused(Request {
        at: (2, 28),
        ..request()
    }));
    assert!(refused(Request {
        level: 0,
        ..request()
    }));
    assert!(refused(Request {
        level: 100,
        ..request()
    }));
    assert!(refused(Request {
        chapters: vec!["nope".to_owned()],
        ..request()
    }));
}

/// `source` as world map `test_world`, with `made`'s battle in the tables.
fn world_of(source: &str, made: &Generated) -> Result<WorldMapDef, String> {
    let def = load(made);
    let mut battles = CONTENT.battles.clone();
    battles.insert(made.id.clone(), def);
    let refs = WorldRefs {
        battles: &battles,
        chapters: &CONTENT.chapters,
        dialogue: &CONTENT.dialogue,
        characters: &CONTENT.characters,
        items: &CONTENT.items,
        audio: &CONTENT.audio,
    };
    worldmap::from_source("test_world", "test_world", source, refs).map_err(|e| format!("{e:?}"))
}

fn world_source() -> &'static str {
    trpg_content::bundle::file("worldmaps/test_world.ron").expect("the test world map's text")
}

#[test]
fn the_place_joins_the_map_by_a_road_and_passes_its_checks() {
    let made = generated(&request());
    let text = add_place(world_source(), world(), &request(), &made.id).expect("added");
    let def = world_of(&text, &made).unwrap_or_else(|e| panic!("{e}"));
    let node = def.map.node("skirmish_10_10").expect("the place");
    assert!(node.opens_when.is_empty() && !node.hidden);
    assert!(node.level.is_some_and(|l| l.abs_diff(35) <= 1));
    assert!(def.map.roads.iter().any(|(_, to)| to == "skirmish_10_10"));
}

#[test]
fn a_place_for_one_chapter_opens_only_then_and_is_out_of_sight_otherwise() {
    let request = Request {
        chapters: vec!["ch02".to_owned()],
        ..request()
    };
    let made = generated(&request);
    let text = add_place(world_source(), world(), &request, &made.id).expect("added");
    let def = world_of(&text, &made).unwrap_or_else(|e| panic!("{e}"));
    let lead = trpg_core::LeadProfile::new("Lead", trpg_core::LeadGender::Male);
    let mut campaign = trpg_core::Campaign::new_game(
        trpg_core::GameMode::Classic,
        lead,
        "ch02".to_owned(),
        Vec::new(),
        0,
        trpg_core::Stock::default(),
    );
    campaign.enter_world(&def.map);
    assert!(def.map.is_open("skirmish_10_10", &campaign));
    campaign.chapter = "ch03".to_owned();
    assert!(!def.map.is_shown("skirmish_10_10", &campaign));
}

#[test]
fn the_road_never_starts_at_a_place_that_is_not_built() {
    let mut world = world().clone();
    let nearest = world
        .places
        .iter()
        .min_by_key(|(_, p)| (p.x - 10).abs() + (p.y - 10).abs());
    let nearest = nearest.map(|(id, _)| id.clone()).expect("a place");
    let node = world
        .map
        .nodes
        .iter_mut()
        .find(|n| n.id == nearest)
        .expect("its node");
    node.opens_when.push(NodeCondition::NotBuilt);
    let made = generated(&request());
    let text = add_place(world_source(), &world, &request(), &made.id).expect("added");
    assert!(
        !text.contains(&format!("(from: \"{nearest}\", to: \"skirmish_10_10\"")),
        "{text}"
    );
    assert!(text.contains("to: \"skirmish_10_10\""), "{text}");
}

#[test]
fn the_road_starts_at_the_nearest_place_and_turns_only_when_it_must() {
    let road = |at| {
        let request = Request { at, ..request() };
        let made = generated(&request);
        let text = add_place(world_source(), world(), &request, &made.id).expect("added");
        let to = format!("to: \"skirmish_{}_{}\"", at.0, at.1);
        let line = text.lines().find(|l| l.contains(&to)).expect("its road");
        line.trim().to_owned()
    };
    assert_eq!(
        road((10, 10)),
        "(from: \"camp\", to: \"skirmish_10_10\", via: [(8, 10)]),"
    );
    assert_eq!(
        road((50, 18)),
        "(from: \"guard\", to: \"skirmish_50_18\", via: [(54, 18)]),"
    );
    // Straight down from Wolf Pass, straight across from Old Camp.
    assert_eq!(road((30, 8)), "(from: \"pass\", to: \"skirmish_30_8\"),");
    assert_eq!(
        road((60, 7)),
        "(from: \"training\", to: \"skirmish_60_7\"),"
    );
}

#[test]
fn the_cell_must_leave_room_for_the_place_s_plate_and_marker() {
    let at = |at| generate(&CONTENT, world(), &Request { at, ..request() });
    assert!(at((68, 10)).is_ok());
    assert!(at((69, 10)).is_err());
    assert!(at((0, 10)).is_err());
    assert!(at((10, 0)).is_err());
    assert!(at((10, 29)).is_err() || world().ground_at(Pos::new(10, 29)) == "sea");
}

#[test]
fn the_region_is_every_cell_within_reach_of_the_place() {
    let mut rows = vec![".".repeat(20); 20];
    rows[4].replace_range(5..6, "f");
    rows[10].replace_range(11..12, "^");
    let world = WorldMapDef {
        size: (20, 20),
        ground: rows,
        ..WorldMapDef::default()
    };
    let cells = region(&world, Pos::new(8, 7));
    assert_eq!(cells.len(), 49);
    assert_eq!(cells[0], "forest");
    assert_eq!(cells[48], "mountain");
    assert_eq!(cells.iter().filter(|g| **g == "plain").count(), 47);
}

#[test]
fn the_map_leaves_the_sides_and_the_middle_row_clear() {
    let text = map_text(&["forest"], "w", 1, 2, &mut SimRng::new(1));
    let rows: Vec<&str> = text.split("---\n").nth(1).expect("tiles").lines().collect();
    assert_eq!(rows.len(), 10);
    for (y, row) in rows.iter().enumerate() {
        let want = if y == 5 {
            "..............".to_owned()
        } else {
            format!("....{}....", "T".repeat(6))
        };
        assert_eq!(*row, want, "row {y}");
    }
}

#[test]
fn each_ground_becomes_its_tile() {
    let tiles: String = [
        "plain", "forest", "thicket", "mountain", "peak", "water", "sea",
    ]
    .into_iter()
    .map(tile_of)
    .collect();
    assert_eq!(tiles, ".TT^^~~");
}

#[test]
fn the_army_is_the_earliest_chapter_s_or_the_story_s_last() {
    let chapter = |list: &[&str]| {
        let chapters: Vec<String> = list.iter().map(|c| (*c).to_owned()).collect();
        army_chapter(&CONTENT, &chapters)
    };
    assert_eq!(chapter(&["ch02", "ch01"]), Ok("ch01".to_owned()));
    let last = story_chapters(&CONTENT).last().map(|c| c.id.clone());
    assert_eq!(chapter(&[]).ok(), last);
    assert_eq!(
        chapter(&["nope"]),
        Err("no chapter \"nope\" in the story".to_owned())
    );
}

#[test]
fn a_taken_cell_is_refused() {
    let at = world().places.get("camp").expect("a place");
    let request = Request {
        at: (at.x, at.y),
        ..request()
    };
    let made = generated(&request);
    assert!(add_place(world_source(), world(), &request, &made.id).is_err());
}

#[test]
fn a_random_battle_fits_the_pool_and_joins_it() {
    let request = Request {
        kind: Kind::Random,
        tier: "Easy".to_owned(),
        ..request()
    };
    let made = generated(&request);
    let def = load(&made);
    assert!(trpg_content::battle::pool_source_fits(
        &made.battle_text,
        &def,
        ENEMIES
    ));
    let source = "(\n    spawn: (0, 2), maximum: 3, level_offset: 1, size: (4, 6),\n    \
                  pool: [\"a\"],\n    rewards: [None],\n)\n";
    let text = add_to_pool(source, &made.id).expect("added");
    assert!(text.contains(&format!("\"{}\"", made.id)));
    let _: trpg_core::SkirmishRules = ron::from_str(&text).unwrap_or_else(|e| panic!("{e}"));
}

proptest! {
    #[test]
    fn generated_battles_always_pass_content_validation(
        seed in any::<u64>(),
        level in 1u32..=99,
        x in 9i32..60,
        y in 1i32..12,
        mix in any::<bool>(),
    ) {
        let units = if mix { Units::Mix } else { Units::Only("sword".to_owned()) };
        let request = Request { seed, level, at: (x, y), units, ..request() };
        // Open sea is refused; anything else must load.
        if let Ok(made) = generate(&CONTENT, world(), &request) {
            let def = load(&made);
            prop_assert_eq!(def.enemies.len(), ENEMIES as usize);
            prop_assert!(def.player_slots.len() >= 6);
        }
    }
}
