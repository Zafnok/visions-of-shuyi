//! Tests of the world map files: the test world map loads, and each check
//! catches its problem.

use super::*;
use crate::Content;

fn content() -> Content {
    crate::load_embedded().unwrap_or_else(|e| panic!("{e}"))
}

fn refs(c: &Content) -> WorldRefs<'_> {
    WorldRefs {
        battles: &c.battles,
        chapters: &c.chapters,
        dialogue: &c.dialogue,
        characters: &c.characters,
        items: &c.items,
        audio: &c.audio,
    }
}

/// A small map that passes every check, with `nodes` and `roads` added.
fn text(nodes: &str, roads: &str) -> String {
    format!(
        r#"(
            act: 1, levels: (1, 5), music: "world_map", start: "a", final_node: "b",
            size: (20, 10),
            nodes: [
                (id: "a", name: "A", pos: (2, 2), kind: Empty),
                (id: "b", name: "B", pos: (10, 2), kind: StoryBattle("test_wm1")),
                {nodes}
            ],
            roads: [(from: "a", to: "b"), {roads}],
        )"#
    )
}

/// The problems of world map `text` (as `test_world`, the map `test_wm1`
/// names).
fn problems(text: &str) -> Vec<String> {
    let c = content();
    let raw: RawWorldMap = parse_ron("t.ron", text).unwrap_or_else(|e| panic!("{e:?}"));
    let mut out = Vec::new();
    let def = build("test_world", raw, refs(&c), &mut |m| out.push(m));
    check(&def, refs(&c), &mut |m| out.push(m));
    out
}

#[test]
fn the_small_map_has_no_problem() {
    assert_eq!(problems(&text("", "")), Vec::<String>::new());
}

#[test]
fn the_test_world_map_loads_with_its_places_levels_and_roads() {
    let c = content();
    let def = &c.worldmaps["test_world"];
    assert_eq!(def.map.start, "home");
    assert_eq!(def.map.next.as_deref(), Some("test_world_2"));
    assert_eq!(def.name("ford"), "Stone Mill");
    assert_eq!(def.places["ford"], Pos::new(34, 13));
    assert_eq!(def.place_music["city"], "capital");
    assert_eq!(def.music, "world_map");
    // A battle's level is its enemies' average.
    let ford = def.map.node("ford").expect("a place");
    assert_eq!(ford.level, enemy_level(&c.battles["test_world_story"]));
    assert!(ford.level.is_some());
    assert_eq!(def.map.node("home").and_then(|n| n.level), None);
    // A road is its places and its turns.
    assert_eq!(
        def.roads[0],
        [Pos::new(14, 20), Pos::new(14, 16), Pos::new(24, 16)]
    );
    assert_eq!(def.ground_at(Pos::new(0, 0)), "sea");
    assert_eq!(def.ground_at(Pos::new(99, 99)), "plain");
}

#[test]
fn a_maps_picture_is_kept() {
    let c = content();
    let raw: RawWorldMap = parse_ron(
        "t.ron",
        &text("", "").replace(
            "size: (20, 10)",
            r#"size: (20, 10), picture: Some("images/map.png")"#,
        ),
    )
    .unwrap_or_else(|e| panic!("{e:?}"));
    let def = build("test_world", raw, refs(&c), &mut |_| {});
    assert_eq!(def.picture.as_deref(), Some("images/map.png"));
}

#[test]
fn the_average_enemy_level_rounds_to_the_nearest() {
    let c = content();
    let mut battle = c.battles["test"].clone();
    let one = battle.enemies[0].clone();
    battle.enemies = vec![one.clone(), one.clone()];
    battle.enemies[0].level = 3;
    battle.enemies[1].level = 4;
    assert_eq!(enemy_level(&battle), Some(4));
    battle.enemies.clear();
    assert_eq!(enemy_level(&battle), None);
}

#[test]
fn a_road_must_join_two_places_and_run_straight() {
    assert_eq!(
        problems(&text("", r#"(from: "a", to: "z")"#)),
        ["the road from \"a\" to \"z\" doesn't join two places of the map"]
    );
    let bent = text(
        r#"(id: "c", name: "C", pos: (5, 6), kind: Empty),"#,
        r#"(from: "a", to: "c")"#,
    );
    assert_eq!(
        problems(&bent),
        ["the road from \"a\" to \"c\" must run straight across or down between its turns"]
    );
    let turned = text(
        r#"(id: "c", name: "C", pos: (5, 6), kind: Empty),"#,
        r#"(from: "a", to: "c", via: [(2, 6)])"#,
    );
    assert_eq!(problems(&turned), Vec::<String>::new());
}

#[test]
fn every_place_must_be_reachable_from_the_start() {
    let island = text(r#"(id: "c", name: "C", pos: (5, 6), kind: Empty),"#, "");
    assert_eq!(
        problems(&island),
        ["place \"c\" can't be reached from the start by any road"]
    );
}

#[test]
fn the_start_and_the_final_battle_are_checked() {
    let bad = text("", "").replace(r#"start: "a""#, r#"start: "b""#);
    assert_eq!(
        problems(&bad),
        ["start \"b\" must be open from the start and not a battle"]
    );
    let bad = text("", "").replace(r#"final_node: "b""#, r#"final_node: "a""#);
    assert_eq!(
        problems(&bad),
        [
            "final_node \"a\" must be one of the map's story battles, or `NotBuilt` until its chapter exists"
        ]
    );
    // A place whose chapter doesn't exist yet may be named while `NotBuilt`.
    let planned = text(
        r#"(id: "z", name: "Z", pos: (5, 2), kind: Empty, opens_when: [NotBuilt]),"#,
        r#"(from: "a", to: "z"),"#,
    )
    .replace(r#"final_node: "b""#, r#"final_node: "z""#);
    assert_eq!(problems(&planned), Vec::<String>::new());
}

#[test]
fn a_shops_later_stock_and_its_conditions_are_checked() {
    let town = |later: &str| {
        text(
            &format!(
                r#"(id: "t", name: "T", pos: (5, 2), kind: Town(shops: [
                    (kind: Vendor, stock: ["potion"], later: [{later}]),
                ])),"#
            ),
            r#"(from: "a", to: "t"),"#,
        )
    };
    let good = town(r#"(when: [Cleared("b")], stock: ["hi_potion"])"#);
    assert_eq!(problems(&good), Vec::<String>::new());
    let bad = problems(&town(
        r#"(when: [Cleared("nowhere")], stock: ["hi_potion"])"#,
    ));
    assert_eq!(bad, ["place \"t\": Cleared(\"nowhere\") names no place"]);
    let bad = problems(&town(r#"(when: [Alive("nobody")], stock: ["hi_potion"])"#));
    assert_eq!(bad, ["place \"t\": Alive(\"nobody\") names no character"]);
    // A Vendor sells consumables: what it adds later is checked the same.
    let bad = problems(&town(r#"(when: [Cleared("b")], stock: ["iron_sword"])"#));
    assert_eq!(bad.len(), 1, "{bad:?}");
    assert!(bad[0].contains("iron_sword"), "{bad:?}");
}

/// Act 1's real map (1070): what it holds, and that it keeps it as the
/// tickets that make its battles fill them in.
mod act1 {
    use super::*;
    use trpg_core::{NodeCondition, NodeKind, ShopKind};

    fn map() -> WorldMap {
        content().worldmaps["act1"].map.clone()
    }

    /// Items `kind` sells at `town` with everything it adds later.
    fn stock(map: &WorldMap, town: &str, kind: ShopKind, later: bool) -> Vec<String> {
        let Some(NodeKind::Town {
            shops, later: more, ..
        }) = map.node(town).map(|n| &n.kind)
        else {
            panic!("no town {town}");
        };
        let i = shops.iter().position(|s| s.kind == kind);
        let i = i.unwrap_or_else(|| panic!("{town} has no {kind:?}"));
        let grown = more.iter().filter(|g| later && g.shop == i);
        shops[i]
            .stock
            .iter()
            .chain(grown.flat_map(|g| &g.stock))
            .map(|item| item.0.clone())
            .collect()
    }

    #[test]
    fn every_place_of_the_outline_is_on_the_map() {
        let map = map();
        for id in [
            "veyne_hall",
            "aldwater",
            "millhaven_road",
            "millhaven",
            "company_ledger",
            "envoys_house",
            "salt_road",
            "smugglers_cove",
            "saltmere",
            "letters_home",
            "captains_oath",
            "ashfields",
            "burnt_mill",
            "brennish_outriders",
            "cairnford",
            "undercroft",
            "pilgrims_road",
            "ossuary_gate",
            "ossary",
            "other_cart",
            "unwritten",
            "saltmere_docks",
        ] {
            assert!(map.node(id).is_some(), "no place {id}");
        }
        assert_eq!(map.final_node, "saltmere_docks");
    }

    #[test]
    fn a_made_battle_is_never_marked_not_built() {
        for node in &map().nodes {
            let unbuilt = node.opens_when.contains(&NodeCondition::NotBuilt);
            assert!(!(unbuilt && node.kind.is_battle()), "{}", node.id);
        }
    }

    #[test]
    fn each_side_quest_waits_for_its_character() {
        let map = map();
        for (place, who) in [
            ("company_ledger", "sergeant"),
            ("captains_oath", "retainer"),
            ("letters_home", "poacher"),
            ("other_cart", "heretic"),
            ("unwritten", "keeper"),
        ] {
            let node = map.node(place).unwrap_or_else(|| panic!("no {place}"));
            let alive = NodeCondition::Alive(trpg_core::CharacterId(who.into()));
            assert!(node.opens_when.contains(&alive), "{place} waits for {who}");
        }
    }

    #[test]
    fn the_shops_follow_the_tiers() {
        let map = map();
        let steel = |s: &str| s.starts_with("steel_");
        // Chapter 4 has no Steel yet; it reaches Millhaven from Chapter 5.
        let armoury = stock(&map, "millhaven", ShopKind::Armoury, false);
        assert!(!armoury.iter().any(|i| steel(i)), "{armoury:?}");
        let armoury = stock(&map, "millhaven", ShopKind::Armoury, true);
        assert!(armoury.iter().any(|i| steel(i)), "{armoury:?}");
        // The Hi-Potion only from Chapter 8, at Saltmere.
        for kind in [ShopKind::Vendor, ShopKind::Alchemist] {
            assert_eq!(stock(&map, "saltmere", kind, false), ["potion"]);
            assert_eq!(stock(&map, "saltmere", kind, true), ["potion", "hi_potion"]);
        }
        // Saltmere is the biggest shop in Act 1, and nothing sells the Elixir.
        let size = |town| {
            let Some(NodeKind::Town { shops, .. }) = map.node(town).map(|n| &n.kind) else {
                panic!("no town {town}");
            };
            assert!(
                !shops
                    .iter()
                    .any(|s| s.stock.iter().any(|i| i.0 == "elixir")),
                "{town} sells the Elixir"
            );
            shops.iter().map(|s| s.stock.len()).sum::<usize>()
        };
        for town in ["millhaven", "cairnford", "ossary"] {
            assert!(size("saltmere") > size(town), "{town} outsells Saltmere");
        }
    }

    #[test]
    fn the_random_pool_is_the_acts_own_and_its_levels_hold_the_cap() {
        let map = map();
        let rules = map.skirmishes.clone().unwrap_or_else(|| panic!("no pool"));
        assert_eq!(rules.pool.len(), 3);
        assert_eq!(
            (rules.spawn, rules.maximum, rules.level_offset),
            ((0, 2), 3, 1)
        );
        assert_eq!(rules.size, (4, 6));
        assert_eq!(map.levels, (1, 30));
    }
}

#[test]
fn story_battles_must_name_a_chapter_on_this_map() {
    let other = text(
        r#"(id: "c", name: "C", pos: (5, 2), kind: StoryBattle("test")),
           (id: "d", name: "D", pos: (8, 5), kind: StoryBattle("nope")),"#,
        r#"(from: "a", to: "c"), (from: "c", to: "d", via: [(5, 5)])"#,
    );
    assert_eq!(
        problems(&other),
        [
            "place \"c\": chapter \"test\" must say `world_map: Some(\"test_world\")`",
            "place \"d\": no chapter \"nope\"",
        ]
    );
}

#[test]
fn a_side_quest_names_a_battle_and_scenes_that_exist() {
    let quest = text(
        r#"(id: "q", name: "Q", pos: (5, 2),
            kind: SideQuest((battle: "nope", intro_scenes: ["no_scene"])),
            opens_when: [Cleared("zz"), Alive("nobody"), Flag("anything")]),"#,
        r#"(from: "a", to: "q")"#,
    );
    assert_eq!(
        problems(&quest),
        [
            "place \"q\": no battle \"nope\"",
            "place \"q\": no scene \"no_scene\"",
            "place \"q\": Cleared(\"zz\") names no place",
            "place \"q\": Alive(\"nobody\") names no character",
        ]
    );
}

#[test]
fn a_towns_shops_are_checked_as_map_shops_are() {
    let town = text(
        r#"(id: "t", name: "T", pos: (5, 2), kind: Town(shops: [
               (kind: Tailor, stock: ["iron_sword"]),
               (kind: Vendor),
               (kind: Blacksmith, stock: ["potion"]),
               (kind: Basic, stock: ["potion"]),
           ])),
           (id: "u", name: "U", pos: (8, 2), kind: Town(shops: [])),"#,
        r#"(from: "a", to: "t"), (from: "t", to: "u")"#,
    );
    assert_eq!(
        problems(&town),
        [
            "town \"t\": a Tailor trades only cloth and leather armor, not \"iron_sword\"",
            "town \"t\": a Vendor sells nothing",
            "town \"t\": a Blacksmith repairs, it sells nothing",
            "town \"t\": Blacksmith shop can't sell \"potion\"",
            "town \"t\": the basic shop is only on Preparations",
            "town \"u\" has no shop",
        ]
    );
}

#[test]
fn music_size_ground_and_places_are_checked() {
    let bad = text(
        r#"(id: "edge", name: "E", pos: (0, 0), kind: Empty, music: Some("no_cue")),"#,
        r#"(from: "a", to: "edge", via: [(2, 0)])"#,
    )
    .replace(r#"music: "world_map""#, r#"music: "nope""#)
    .replace(
        "size: (20, 10)",
        r#"size: (20, 2), ground: ["~~~~~~~~~~~~~~~~~~~~", "~~x"]"#,
    )
    .replace("levels: (1, 5)", "levels: (6, 5)");
    assert_eq!(
        problems(&bad),
        [
            "ground row 1 isn't 20 characters",
            "ground row 1: 'x' is no ground",
            "place \"a\" at Pos { x: 2, y: 2 } is too near the map's edge",
            "place \"b\" at Pos { x: 10, y: 2 } is too near the map's edge",
            "place \"edge\" at Pos { x: 0, y: 0 } is too near the map's edge",
            "levels (6, 5): the lowest is above the highest",
            "music: no music cue \"nope\"",
            "place \"edge\": no music cue \"no_cue\"",
        ]
    );
    // Equal levels are fine; a map with no rows isn't; a place too near
    // one edge alone is.
    let even = text("", "").replace("levels: (1, 5)", "levels: (5, 5)");
    assert_eq!(problems(&even), Vec::<String>::new());
    let flat = text("", "").replace("size: (20, 10)", "size: (20, 0)");
    assert!(problems(&flat).contains(&"size (20, 0) must be from (1, 1) to (70, 30)".to_owned()));
    let right = text(
        r#"(id: "r", name: "R", pos: (19, 5), kind: Empty),
           (id: "top", name: "T", pos: (5, 0), kind: Empty),"#,
        r#"(from: "a", to: "r", via: [(19, 2)]), (from: "a", to: "top", via: [(5, 2)])"#,
    );
    assert_eq!(
        problems(&right),
        [
            "place \"r\" at Pos { x: 19, y: 5 } is too near the map's edge",
            "place \"top\" at Pos { x: 5, y: 0 } is too near the map's edge",
        ]
    );
    // The limits themselves are fine: one cell across, a place two cells
    // from the right edge.
    let narrow = text("", "").replace("size: (20, 10)", "size: (1, 10)");
    assert!(!problems(&narrow).iter().any(|p| p.starts_with("size")));
    let edge = text(
        r#"(id: "e", name: "E", pos: (18, 2), kind: Empty),"#,
        r#"(from: "b", to: "e")"#,
    );
    assert_eq!(problems(&edge), Vec::<String>::new());
    let low = text("", "").replace("size: (20, 10)", "size: (20, 1)");
    assert!(!problems(&low).iter().any(|p| p.starts_with("size")));
    // A place one cell from the left edge or the top is fine; at the left
    // edge it isn't.
    let near = text(
        r#"(id: "l", name: "L", pos: (1, 5), kind: Empty),
           (id: "t", name: "T", pos: (5, 1), kind: Empty),
           (id: "z", name: "Z", pos: (0, 5), kind: Empty),"#,
        r#"(from: "a", to: "l", via: [(1, 2)]), (from: "a", to: "t", via: [(5, 2)]),
           (from: "l", to: "z")"#,
    );
    assert_eq!(
        problems(&near),
        ["place \"z\" at Pos { x: 0, y: 5 } is too near the map's edge"]
    );
    let big = text("", "").replace("size: (20, 10)", "size: (71, 10)");
    assert_eq!(
        problems(&big),
        ["size (71, 10) must be from (1, 1) to (70, 30)"]
    );
}

#[test]
fn a_place_listed_twice_is_an_error() {
    let twice = text(
        r#"(id: "a", name: "A again", pos: (5, 2), kind: Empty),"#,
        "",
    );
    assert!(problems(&twice).contains(&"place \"a\" is listed twice".to_owned()));
}

#[test]
fn a_next_map_and_a_chapters_world_map_must_exist() {
    let c = content();
    let mut maps = c.worldmaps.clone();
    let mut chapters = c.chapters.clone();
    maps.get_mut("test_world_2").expect("a map").map.next = Some("gone".into());
    let ch = chapters.get_mut("test").expect("a chapter");
    ch.world_map = Some("test_world".into());
    let errors: Vec<String> = check_links(&maps, &chapters)
        .into_iter()
        .map(|e| e.message)
        .collect();
    assert_eq!(
        errors,
        [
            "next: no world map \"gone\"",
            "world_map: \"test_world\" is no world map with this chapter's story battle",
        ]
    );
}

#[test]
fn fixed_skirmishes_validate_battle_scenes_and_music_pool() {
    let fixed = |battle: &str, scenes: &str| {
        format!(
            r#"(id: "fixed", name: "Fixed", pos: (3, 5), kind: FixedSkirmish(battle: (battle: "{battle}", intro_scenes: [{scenes}]), guards_path: true)),"#
        )
    };
    assert_eq!(
        problems(&text(
            &fixed("test_fixed_guard", ""),
            r#"(from: "a", to: "fixed", via: [(2, 5)]),"#
        )),
        Vec::<String>::new()
    );
    assert!(
        problems(&text(&fixed("missing", r#""missing""#), ""))
            .iter()
            .any(|e| e.contains("no battle"))
    );
    assert!(
        problems(&text(&fixed("ch01", ""), ""))
            .iter()
            .any(|e| e.contains("music must be"))
    );
}

#[test]
fn a_chapter_condition_must_name_a_chapter() {
    let c = content();
    let place = |chapter: &str| {
        text(
            &format!(
                r#"(id: "c", name: "C", pos: (5, 5), kind: Empty, opens_when: [Chapter(["{chapter}"])]),"#
            ),
            r#"(from: "a", to: "c", via: [(2, 5)]),"#,
        )
    };
    assert!(from_source("w", "test_world", &place("ch02"), refs(&c)).is_ok());
    let errors =
        from_source("w", "test_world", &place("nope"), refs(&c)).expect_err("no such chapter");
    assert!(
        errors[0].to_string().contains("Chapter names no chapter"),
        "{errors:?}"
    );
}
