//! The test files (ADR-0057): the characters, chapters, battles and
//! pictures of the tests and the debug tools are in `assets-test/`, which
//! only a debug build or a `test-fixtures` build embeds. These tests load
//! the content the way a build that ships has it
//! (`bundle::story_only`) and the way a test build has it.

use trpg_content::{Content, bundle, load_embedded};

/// The content of a build that ships: without the test files.
fn shipped() -> Content {
    bundle::story_only(load_embedded).unwrap_or_else(|errors| panic!("{errors}"))
}

/// Whether `id` is a test file's: `test`, or starting `test_`.
fn is_test(id: &str) -> bool {
    id == "test" || id.starts_with("test_")
}

/// A build without the test files loads (the story's files never need
/// them) and has no test character, generic, speaker, name, chapter,
/// battle, map, scene, portrait, tileset, image or language pack, and no
/// Quick Battle.
#[test]
fn a_build_without_the_test_files_has_no_test_content() {
    let content = shipped();
    let characters = &content.characters;
    let groups: [(&str, Vec<&str>); 13] = [
        (
            "character",
            characters.characters.keys().map(|c| c.0.as_str()).collect(),
        ),
        (
            "generic",
            characters.generics.keys().map(String::as_str).collect(),
        ),
        (
            "speaker",
            characters.speakers.iter().map(|c| c.0.as_str()).collect(),
        ),
        (
            "name",
            content.names.names.keys().map(String::as_str).collect(),
        ),
        (
            "support",
            content
                .supports
                .pairs()
                .flat_map(|def| def.pair.members())
                .map(|c| c.0.as_str())
                .collect(),
        ),
        (
            "chapter",
            content.chapters.keys().map(String::as_str).collect(),
        ),
        (
            "battle",
            content.battles.keys().map(String::as_str).collect(),
        ),
        ("map", content.maps.keys().map(String::as_str).collect()),
        (
            "scene",
            content.dialogue.scenes.keys().map(String::as_str).collect(),
        ),
        (
            "portrait",
            content.portraits.keys().map(String::as_str).collect(),
        ),
        (
            "tileset",
            content.tilesets.keys().map(String::as_str).collect(),
        ),
        (
            "image",
            content
                .images
                .images
                .keys()
                .flat_map(|path| path.split(['/', '.']))
                .collect(),
        ),
        (
            "language pack",
            content
                .lang
                .codes()
                .map(trpg_content::LangCode::as_str)
                .collect(),
        ),
    ];
    for (what, ids) in &groups {
        let test: Vec<&&str> = ids.iter().filter(|id| is_test(id)).collect();
        assert!(test.is_empty(), "{what}: {test:?}");
    }
    assert!(!content.chapters.contains_key("quick"));
    assert!(!content.battles.contains_key("quick"));
    // Nor any other file of theirs.
    for path in bundle::story_only(|| bundle::files_under("")) {
        let parts: Vec<&str> = path.split(['/', '.']).collect();
        assert!(!parts.iter().any(|part| is_test(part)), "{path}");
        assert!(!bundle::display_path(path).starts_with("assets-test"));
    }
    // The story is all there.
    assert!(
        content
            .chapters
            .contains_key(&content.new_game.first_chapter)
    );
    assert!(characters.characters.len() >= 5, "{characters:?}");
    assert!(characters.generics.len() >= 5);
    assert!(content.names.names.len() > 50);
}

/// A build with the test files has the story's content unchanged, and the
/// test files' beside it: what the tests and the debug Quick Battle play.
#[test]
fn the_test_files_add_to_the_story_and_change_none_of_it() {
    let shipped = shipped();
    let content = load_embedded().unwrap_or_else(|errors| panic!("{errors}"));
    const { assert!(bundle::TEST_FILES) };
    for chapter in ["test", "quick"] {
        assert!(content.chapters.contains_key(chapter), "{chapter}");
        assert!(content.battles.contains_key(chapter), "{chapter}");
    }
    assert!(content.maps.contains_key("test_small"));
    assert!(content.tilesets.contains_key("test"));
    assert!(content.portraits.contains_key("test_lord"));
    assert!(content.dialogue.get("test").is_some());
    let codes: Vec<&str> = content
        .lang
        .codes()
        .map(trpg_content::LangCode::as_str)
        .collect();
    assert_eq!(codes, ["test"]);
    assert_eq!(content.supports.pairs().count(), 2);
    assert_eq!(content.supports.rules, shipped.supports.rules);
    assert_eq!(content.new_game, shipped.new_game);

    let characters = &content.characters;
    let test_characters = [
        "test_archer",
        "test_knight",
        "test_lord",
        "test_mage",
        "test_rogue",
        "test_scout",
    ];
    let added: Vec<&str> = characters
        .characters
        .keys()
        .filter(|id| !shipped.characters.characters.contains_key(id))
        .map(|id| id.0.as_str())
        .collect();
    assert_eq!(added, test_characters);
    let added: Vec<&str> = characters
        .generics
        .keys()
        .filter(|id| !shipped.characters.generics.contains_key(*id))
        .map(String::as_str)
        .collect();
    assert_eq!(
        added,
        ["test_brigand", "test_frost_elemental", "test_raider"]
    );
    assert_eq!(characters.speakers, shipped.characters.speakers);
    let added: Vec<&str> = content
        .names
        .names
        .keys()
        .filter(|id| shipped.names.get(id).is_none())
        .map(String::as_str)
        .collect();
    assert_eq!(added, test_characters);

    // Everything the story has is the same with the test files there.
    for (id, def) in &shipped.characters.characters {
        assert_eq!(characters.characters.get(id), Some(def));
    }
    for (id, template) in &shipped.characters.generics {
        assert_eq!(characters.generics.get(id), Some(template));
    }
    for (id, name) in &shipped.names.names {
        assert_eq!(content.names.get(id), Some(name.as_str()));
    }
    for (id, chapter) in &shipped.chapters {
        assert_eq!(content.chapters.get(id), Some(chapter));
    }
    for (id, battle) in &shipped.battles {
        assert_eq!(content.battles.get(id), Some(battle));
    }
    for (id, map) in &shipped.maps {
        assert_eq!(content.maps.get(id), Some(map));
    }
    for (id, scene) in &shipped.dialogue.scenes {
        assert_eq!(content.dialogue.scenes.get(id), Some(scene));
    }
}
