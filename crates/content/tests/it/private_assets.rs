//! The one test that opts in to the bought art (ADR-0040). It only exists
//! with the `private-assets` feature (its `mod` line in `main.rs` is behind
//! it), which no gate turns on:
//!
//! ```text
//! cargo test -p trpg-content --features private-assets --test it private_assets::
//! ```
//!
//! Run it after changing anything in `assets-private/game/`; the Pages build
//! runs it before it builds the game.

/// Every content file still loads and validates with the private files laid
/// over `assets/`.
#[test]
fn the_content_loads_with_the_private_assets_over_it() {
    if let Err(errors) = trpg_content::load_embedded() {
        panic!("{errors}");
    }
}

/// Every portrait image that comes from the private assets is what
/// `cargo xtask portrait-import` writes (ADR-0043): a 64×64 cut bust, which
/// fills the dialogue frame at 4×, or a 48×48 face.
#[test]
fn bought_portraits_are_cut_busts_or_faces() {
    use trpg_content::bundle::{PRIVATE_DISPLAY_ROOT, display_path};

    let content = match trpg_content::load_embedded() {
        Ok(content) => content,
        Err(errors) => panic!("{errors}"),
    };
    for portrait in content.portraits.values() {
        for expression in &portrait.expressions {
            let shown = display_path(expression.image.path());
            let size = (expression.size.width, expression.size.height);
            assert!(
                !shown.starts_with(PRIVATE_DISPLAY_ROOT) || matches!(size, (64, 64) | (48, 48)),
                "{shown} is {size:?}"
            );
        }
    }
}

/// The bought map sprites (`cargo xtask map-sprite-import`, ADR-0049): the
/// game's tileset is there, paints only units (terrain is ticket 0437's),
/// and each of its pictures is the standing frame of a 48×80 sheet.
#[test]
fn the_bought_map_sprites_are_standing_frames_of_whole_sheets() {
    let content = match trpg_content::load_embedded() {
        Ok(content) => content,
        Err(errors) => panic!("{errors}"),
    };
    let tileset = &content.tilesets["tiny_tales"];
    assert!(tileset.terrain.is_none());
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

/// Every bought file in the build has a credit (ADR-0051), and the credits
/// screen's list has the bought art's.
#[test]
fn every_bought_file_has_a_credit() {
    use trpg_content::bundle::{PRIVATE_DISPLAY_ROOT, display_path, files_under};
    use trpg_content::{CreditGroup, credits};

    let mut bought = files_under("");
    bought.retain(|p| display_path(p).starts_with(PRIVATE_DISPLAY_ROOT));
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
    assert_eq!(art, ["Tiny Tales"]);
}
