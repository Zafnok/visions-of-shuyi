//! The one test that opts in to the bought art (ADR-0040). It only exists
//! with the `private-assets` feature (its `mod` line in `main.rs` is behind
//! it), which no gate turns on:
//!
//! ```text
//! cargo test -p trpg-content --features private-assets --test it private_assets::
//! ```
//!
//! Run it after changing anything in `assets-private/game/` or
//! `assets-private/test/` (the bought art of the test files, ADR-0057); the
//! Pages build runs it before it builds the game.

/// Every content file still loads and validates with the private files laid
/// over `assets/`.
#[test]
fn the_content_loads_with_the_private_assets_over_it() {
    if let Err(errors) = trpg_content::load_embedded() {
        panic!("{errors}");
    }
}

/// A build that ships has the private files and no test files (ADR-0057):
/// the content loads that way too, and none of it is a test file's.
#[test]
fn the_content_loads_with_the_private_assets_and_without_the_test_files() {
    use trpg_content::bundle;

    let content = match bundle::story_only(trpg_content::load_embedded) {
        Ok(content) => content,
        Err(errors) => panic!("{errors}"),
    };
    assert!(content.tilesets.contains_key("tiny_tales"));
    let test = |id: &&String| id.starts_with("test");
    assert_eq!(content.portraits.keys().find(test), None);
    assert_eq!(content.tilesets.keys().find(test), None);
    for path in bundle::story_only(|| bundle::files_under("")) {
        let parts: Vec<&str> = path.split(['/', '.']).collect();
        assert!(!parts.iter().any(|part| part.starts_with("test")), "{path}");
    }
    // With them, the bought test portraits replace the placeholders.
    let shown = bundle::display_path("portraits/test_lord.ron");
    assert_eq!(shown, "assets-private/test/portraits/test_lord.ron");
}

/// Every portrait image that comes from the private assets is what
/// `cargo xtask portrait-import` writes (ADR-0043): a 64×64 cut bust, which
/// fills the dialogue frame at 4×, or a 48×48 face.
#[test]
fn bought_portraits_are_cut_busts_or_faces() {
    use trpg_content::bundle::{display_path, is_private};

    let content = match trpg_content::load_embedded() {
        Ok(content) => content,
        Err(errors) => panic!("{errors}"),
    };
    for portrait in content.portraits.values() {
        for expression in &portrait.expressions {
            let shown = display_path(expression.image.path());
            let size = (expression.size.width, expression.size.height);
            assert!(
                !is_private(&shown) || matches!(size, (64, 64) | (48, 48)),
                "{shown} is {size:?}"
            );
        }
    }
}

/// The bought map sprites (`cargo xtask map-sprite-import`, ADR-0049): the
/// game's tileset is there, and each of its unit pictures is the standing
/// frame of a 48×80 sheet.
#[test]
fn the_bought_map_sprites_are_standing_frames_of_whole_sheets() {
    let content = match trpg_content::load_embedded() {
        Ok(content) => content,
        Err(errors) => panic!("{errors}"),
    };
    let tileset = &content.tilesets["tiny_tales"];
    let pictures = tileset.classes.values().chain(tileset.characters.values());
    for picture in pictures.chain([&tileset.fallback]) {
        let path = picture.image.path();
        let size = content
            .images
            .info(picture.image)
            .map(|i| (i.width, i.height));
        assert_eq!(size, Some((48, 80)), "{path}");
        let rect = picture.rect;
        assert_eq!((rect.x, rect.y, rect.w, rect.h), (16, 0, 16, 20), "{path}");
    }
    assert!(tileset.classes.len() >= 10);
}

/// The bought terrain tiles (`cargo xtask tileset-import`, ADR-0052): the
/// game's tileset paints the ground with 16×16 tiles of its packed image,
/// in layers, and has every look a map can name: its own (a map outdoors)
/// and each other one by name.
#[test]
fn the_bought_tiles_paint_every_look_a_map_can_name() {
    use trpg_content::map::{DEFAULT_TILES, TILE_LOOKS};

    let content = match trpg_content::load_embedded() {
        Ok(content) => content,
        Err(errors) => panic!("{errors}"),
    };
    let tileset = &content.tilesets["tiny_tales"];
    let terrain = tileset.terrain.as_ref().expect("no terrain tiles");
    assert_eq!(terrain.tile_px, (16, 16));
    assert_eq!(terrain.image.path(), "tilesets/tiny_tales.png");
    let terrains = content.terrain.display.terrains.len();
    for name in TILE_LOOKS {
        let look = tileset.look(name).expect("no look");
        let named = terrain.looks.contains_key(name);
        assert_eq!(named, name != DEFAULT_TILES, "look {name}");
        assert_eq!(look.tiles.len(), terrains, "look {name}");
        assert!(look.layers.len() >= 10, "look {name}");
    }
}

/// Every bought file in the build has a credit (ADR-0051), and the credits
/// screen's list has the bought art's.
#[test]
fn every_bought_file_has_a_credit() {
    use trpg_content::bundle::{display_path, files_under, is_private};
    use trpg_content::{CreditGroup, credits};

    let mut bought = files_under("");
    bought.retain(|p| is_private(&display_path(p)));
    assert!(bought.len() > 10, "{bought:?}");
    let file = credits::load_file().unwrap_or_default();
    let uncredited = credits::uncredited(&file, &bought);
    assert!(
        uncredited.is_empty(),
        "no entry of assets/data/credits.ron covers: {uncredited:?}"
    );
    let content = match trpg_content::load_embedded() {
        Ok(content) => content,
        Err(errors) => panic!("{errors}"),
    };
    let art: Vec<&str> = content
        .credits
        .in_group(CreditGroup::Art)
        .map(|e| e.title.as_str())
        .collect();
    assert_eq!(
        art,
        [
            "Tiny Tales",
            "Shironejiya battle pictures (the materials in this game may not be reused)"
        ]
    );
}
