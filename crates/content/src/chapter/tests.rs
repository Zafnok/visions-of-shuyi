//! Chapter and New Game file tests, against the embedded content.

use trpg_core::{LeadGender, Outcome};

use super::*;

fn content() -> Content {
    crate::load_embedded().unwrap_or_else(|e| panic!("{e}"))
}

fn chapter(id: &str) -> ChapterDef {
    ChapterDef {
        id: id.into(),
        title: "T".into(),
        number: None,
        opening_scenes: Vec::new(),
        intro_scenes: vec!["test".into()],
        battle: "test".into(),
        victory_scenes: vec!["test_victory".into()],
        next: None,
        world_map: None,
    }
}

fn messages(errors: Vec<ContentError>) -> Vec<String> {
    errors.into_iter().map(|e| e.message).collect()
}

#[test]
fn the_test_chapters_and_new_game_load() {
    let c = content();
    let test = &c.chapters["test"];
    assert_eq!(test.battle, "test");
    assert_eq!(test.intro_scenes, ["test_intro"]);
    assert_eq!(test.victory_scenes, ["test_victory"]);
    assert_eq!(test.next, None);
    assert_eq!(
        c.chapters["quick"].intro_scenes,
        [] as [std::string::String; 0]
    );
    // The tests' New Game is the test chapter.
    let test = NewGameDef::test_chapter();
    assert_eq!(test.first_chapter, "test");
    let roster: Vec<&str> = test.roster.iter().map(|r| r.0.as_str()).collect();
    assert_eq!(roster, [LEAD_ID, "test_knight"]);
    assert_eq!((test.gold, test.stock.len()), (0, 0));
}

/// The game's chapters and New Game, wired as its outline has them (a test
/// of the content, ADR-0118).
#[test]
fn the_games_chapters_and_new_game_load() {
    if crate::bundle::skip_without_the_game() {
        return;
    }
    let c = content();
    // New Game starts Chapter 1 with its six (assets-private/story/design/chapter-1.md).
    let ch01 = &c.chapters["ch01"];
    assert_eq!(ch01.battle, "ch01");
    assert_eq!(ch01.intro_scenes, ["ch01_intro", "ch01_prebattle"]);
    assert_eq!(ch01.victory_scenes, ["ch01_victory", "ch01_tbc"]);
    assert_eq!(ch01.next.as_deref(), Some("ch02"));
    // Chapter 2 (0848).
    let ch02 = &c.chapters["ch02"];
    assert_eq!((ch02.battle.as_str(), ch02.number), ("ch02", Some(2)));
    assert_eq!(ch02.intro_scenes, ["ch02_intro", "ch02_prebattle"]);
    assert_eq!(
        ch02.victory_scenes,
        ["ch02_victory", "ch02_join", "ch02_tbc"]
    );
    assert_eq!(ch02.next.as_deref(), Some("ch03"));
    // Chapter 3 (0849).
    let ch03 = &c.chapters["ch03"];
    assert_eq!((ch03.battle.as_str(), ch03.number), ("ch03", Some(3)));
    assert_eq!(ch03.intro_scenes, ["ch03_intro", "ch03_prebattle"]);
    assert_eq!(
        ch03.victory_scenes,
        [
            "ch03_register",
            "ch03_telling",
            "ch03_confession",
            "ch03_tbc"
        ]
    );
    assert_eq!(ch03.next.as_deref(), Some("ch04"));
    // Chapter 4 (0850): the first on the Act 1 world map, with an opening
    // scene before its place on the map.
    let ch04 = &c.chapters["ch04"];
    assert_eq!(ch04.world_map.as_deref(), Some("act1"));
    assert_eq!(ch04.opening_scenes, ["ch04_intro"]);
    assert_eq!(ch04.intro_scenes, ["ch04_eve", "ch04_gate"]);
    assert_eq!(ch04.battle, "ch04");
    assert_eq!(ch04.victory_scenes, ["ch04_victory", "ch04_tbc"]);
    assert_eq!(ch04.next.as_deref(), Some("ch05"));
    // Chapter 5 (0851): on the world map too.
    let ch05 = &c.chapters["ch05"];
    assert_eq!(ch05.world_map.as_deref(), Some("act1"));
    assert_eq!(ch05.opening_scenes, ["ch05_intro"]);
    assert_eq!(
        ch05.intro_scenes,
        ["ch05_envoy", "ch05_eve", "ch05_prebattle"]
    );
    assert_eq!(ch05.battle, "ch05");
    assert_eq!(
        ch05.victory_scenes,
        ["ch05_victory", "ch05_quay", "ch05_tide", "ch05_tbc"]
    );
    assert_eq!(ch05.next, None);
    assert_eq!(c.new_game.first_chapter, "ch01");
    assert_eq!(c.new_game.roster.len(), 6);
    assert_eq!(c.new_game.roster[0].0, LEAD_ID);
    assert_eq!((c.new_game.gold, c.new_game.stock.len()), (0, 0));
}

#[test]
fn chapter_errors() {
    let c = content();
    let check =
        |def: &ChapterDef| messages(check_chapter("c.ron", "t", def, &c.battles, &c.dialogue));
    assert_eq!(check(&chapter("t")), Vec::<String>::new());
    assert_eq!(
        check(&chapter("x")),
        ["id \"x\" must match the file name \"t\""]
    );
    let bad = ChapterDef {
        battle: "nope".into(),
        intro_scenes: vec!["a".into()],
        victory_scenes: vec!["b".into()],
        number: Some(0),
        ..chapter("t")
    };
    assert_eq!(
        check(&bad),
        [
            "chapter numbers start at 1",
            "no battle \"nope\"",
            "no dialogue scene \"a\"",
            "no dialogue scene \"b\""
        ]
    );
    // Opening scenes (0850): checked like the others, and only on a
    // world-map chapter.
    let opening = ChapterDef {
        opening_scenes: vec!["o".into()],
        ..chapter("t")
    };
    assert_eq!(
        check(&opening),
        [
            "no dialogue scene \"o\"",
            "opening_scenes: only a world-map chapter has them"
        ]
    );
    let on_a_map = ChapterDef {
        opening_scenes: vec!["test".into()],
        world_map: Some("test_world".into()),
        ..chapter("t")
    };
    assert_eq!(check(&on_a_map), Vec::<String>::new());
}

#[test]
fn next_must_be_a_chapter() {
    let mut chapters = BTreeMap::from([
        ("a".to_owned(), chapter("a")),
        ("b".to_owned(), chapter("b")),
    ]);
    assert_eq!(check_next(&chapters), [] as [shuyi_assets::ContentError; 0]);
    chapters.get_mut("a").unwrap().next = Some("b".into());
    assert_eq!(check_next(&chapters), [] as [shuyi_assets::ContentError; 0]);
    chapters.get_mut("b").unwrap().next = Some("z".into());
    let e = check_next(&chapters);
    assert_eq!(messages(e.clone()), ["next: no chapter \"z\""]);
    assert_eq!(e[0].file, "assets/chapters/b.ron");
}

#[test]
fn a_chapter_file_parses() {
    let def: ChapterDef = parse_ron(
        "c.ron",
        r#"(id: "t", title: "Chapter 1: Ash", battle: "b", next: Some("u"))"#,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(def.title, "Chapter 1: Ash");
    assert!(def.intro_scenes.is_empty() && def.victory_scenes.is_empty());
    assert_eq!(def.next.as_deref(), Some("u"));
}

#[test]
fn new_game_errors() {
    let c = content();
    let load =
        |source: &str| new_game_from_source("n.ron", source, &c.chapters, &c.characters, &c.items);
    let ok =
        r#"(first_chapter: "test", roster: ["lead"], gold: 9, stock: ["potion", "iron_sword"])"#;
    let def = load(ok).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(def.gold, 9);
    let bad =
        r#"(first_chapter: "nope", roster: ["test_knight", "x", "test_knight"], stock: ["y"])"#;
    assert_eq!(
        messages(load(bad).err().unwrap_or_default()),
        [
            "first_chapter: no chapter \"nope\"",
            "roster: no character \"x\"",
            "roster: \"test_knight\" is listed twice",
            "roster: the lead (\"lead\") must be in it",
            "stock: no item \"y\"",
        ]
    );
    assert_eq!(load("(").err().map(|e| e.len()), Some(1));
}

#[test]
fn a_new_campaign_starts_as_the_new_game_file_says() {
    let mut c = content();
    c.new_game = NewGameDef::test_chapter();
    c.new_game.gold = 30;
    c.new_game.stock = vec![
        ItemId::new("potion"),
        ItemId::new("iron_sword"),
        ItemId::new("nope"),
    ];
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let game = new_campaign(&c, GameMode::Casual, lead.clone());
    assert_eq!(game.mode, GameMode::Casual);
    assert_eq!(game.lead, lead);
    assert_eq!(game.chapter, "test");
    assert_eq!(game.gold, 30);
    assert_eq!(game.stock.count(&ItemId::new("potion")), 1);
    assert_eq!(game.stock.weapons.len(), 1);
    assert_eq!(game.stock.weapons[0].def, ItemId::new("iron_sword"));
    let names: Vec<_> = game.roster.iter().map(|u| u.name.as_str()).collect();
    assert_eq!(names, ["Mara", "Test Knight"]);
    assert!(game.roster.iter().all(|u| u.faction == Faction::Player));
    // Unknown characters are skipped.
    assert_eq!(roster_units(&c, &[CharacterId("x".into())]), []);
}

#[test]
fn a_battle_campaign_has_the_battles_characters() {
    let c = content();
    let quick = &c.battles["quick"];
    let lead = LeadProfile::new("Ellery", LeadGender::Male);
    let game = battle_campaign(&c, quick, GameMode::Classic, lead);
    let names: Vec<_> = game.roster.iter().map(|u| u.name.as_str()).collect();
    // Its slots' characters, then its bench.
    assert_eq!(
        names,
        [
            "Test Lord",
            "Test Knight",
            "Test Archer",
            "Test Mage",
            "Test Scout"
        ]
    );
    // Its own gold and stock, to try Preparations and its shop with.
    assert_eq!(game.gold, 3000);
    // The Quick Battle's own stock, to try Preparations with.
    assert_eq!(game.stock, stock_of(&c, &quick.solo_stock));
    assert_eq!(game.stock.count(&ItemId::new("potion")), 6);
    assert_eq!(game.stock.weapons.len(), 3);
    // A battle without one starts with an empty stock.
    let test = &c.battles["test"];
    let lead = LeadProfile::new("Ellery", LeadGender::Male);
    let game = battle_campaign(&c, test, GameMode::Classic, lead);
    assert_eq!((game.gold, game.stock), (0, Stock::default()));
}

#[test]
fn the_tables_are_the_contents() {
    let c = content();
    let t = c.tables();
    assert_eq!(*t.terrain, c.terrain.rules);
    assert_eq!(*t.classes, c.classes);
    assert_eq!(*t.items, c.items);
    assert_eq!(*t.spells, c.spells);
    assert_eq!(*t.skills, c.skills);
    assert_eq!(*t.arts, c.arts);
}

/// Every battle file plays: its setup builds with its own characters,
/// and every chapter's battle can be won or lost (it starts undecided).
#[test]
fn every_battle_starts_undecided() {
    let c = content();
    let tables = c.tables();
    for (id, def) in &c.battles {
        let lead = LeadProfile::new("Ellery", LeadGender::Male);
        let game = battle_campaign(&c, def, GameMode::Classic, lead);
        let setup = game.battle_setup(def, &tables);
        assert_eq!(
            setup.units.len(),
            def.player_slots.len() + def.enemies.len(),
            "{id}"
        );
        let (state, _) = trpg_core::BattleState::new(setup);
        assert_ne!(state.outcome(), Some(Outcome::Victory), "{id}");
        assert_ne!(state.outcome(), Some(Outcome::Defeat), "{id}");
    }
}
