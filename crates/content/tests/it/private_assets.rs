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
//! playtest page's build (`playtest-itch.yml`) runs it with the game's
//! content before it builds the game.

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
/// frame of a 48×80 sheet, but the mounted classes': the standing frame of
/// a 72×96 sheet of `MattWalkden`'s riders, cut to 24×24 frames (0499),
/// each with the knight on foot named apart.
#[test]
fn the_bought_map_sprites_are_standing_frames_of_whole_sheets() {
    let content = match trpg_content::load_embedded() {
        Ok(content) => content,
        Err(errors) => panic!("{errors}"),
    };
    let tileset = &content.tilesets["tiny_tales"];
    let pictures = tileset.classes.values().chain(tileset.characters.values());
    let pictures = pictures.chain(tileset.on_foot.values());
    let mut riders = 0;
    for picture in pictures.chain([&tileset.fallback]) {
        let path = picture.image.path();
        let size = content
            .images
            .info(picture.image)
            .map(|i| (i.width, i.height));
        let rect = picture.rect;
        let rect = (rect.x, rect.y, rect.w, rect.h);
        if size == Some((72, 96)) {
            riders += 1;
            assert_eq!(rect, (24, 0, 24, 24), "{path}");
        } else {
            assert_eq!(size, Some((48, 80)), "{path}");
            assert_eq!(rect, (16, 0, 16, 20), "{path}");
        }
    }
    assert!(tileset.classes.len() >= 10);
    // Every mounted class: a rider, and the knight on foot.
    assert_eq!(riders, tileset.on_foot.len());
    for (class, on_foot) in &tileset.on_foot {
        assert!(
            content.classes.get(class).is_some_and(|c| c.tags.mounted),
            "{class:?}"
        );
        assert_eq!(on_foot.image.path(), "units/knight_m1.png");
        let rider = tileset.classes[class].image.path();
        assert!(rider == "units/lance_cavalier.png" || rider == "units/mounted_archer.png");
    }
    assert!(
        tileset
            .on_foot
            .contains_key(&trpg_core::ClassId("rider".into()))
    );
}

/// The named characters with art of their own beyond Chapter 1 (ticket
/// 0858): each speaks with a bought face, and the map and the combat scene
/// pick their own picture, not their class's. A class default or a missing
/// portrait fails here, though either would still fill the frame.
#[test]
fn named_characters_resolve_their_own_face_map_sprite_and_battler() {
    use trpg_content::bundle::{display_path, is_private};
    use trpg_content::portrait::REQUIRED_EXPRESSIONS;
    use trpg_core::{CharacterId, ClassId};

    let content = match trpg_content::load_embedded() {
        Ok(content) => content,
        Err(errors) => panic!("{errors}"),
    };
    let tileset = &content.tilesets["tiny_tales"];
    // (character, its class, its map sheet)
    for (id, class, sheet) in [("defector", "swordsman", "units/thief.png")] {
        let portrait = content.portraits.get(id);
        let portrait = portrait.unwrap_or_else(|| panic!("{id} has no portrait"));
        for name in REQUIRED_EXPRESSIONS {
            let found = portrait.expressions.iter().find(|e| e.name == name);
            let found = found.unwrap_or_else(|| panic!("{id} has no {name} face"));
            let shown = display_path(found.image.path());
            assert!(is_private(&shown), "{id} {name}: {shown}");
        }

        let character = CharacterId(id.into());
        let class = ClassId(class.into());
        let picture = tileset.unit_picture([&character], &class);
        assert_eq!(picture.image.path(), sheet, "{id}'s map sprite");
        assert_ne!(Some(picture), tileset.classes.get(&class).copied(), "{id}");

        let battler = content.battlers.pick([id, class.0.as_str()]);
        let battler = battler.unwrap_or_else(|| panic!("{id} has no battler"));
        assert_eq!(battler.id, id, "{id}'s combat picture");
        let shown = display_path(battler.image.path());
        assert!(is_private(&shown), "{id}'s battler: {shown}");
    }
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

/// The road keeps its width onto a bridge (ticket 0461): the road's corner
/// layer counts a bridge as road, so the four tiles round a point on the
/// seam, some road and some bridge, are all in the layer and pick the
/// inside mix (`####`, which needs no edge picture), never an edge that
/// narrows the road. A look whose roads are whole tiles (Rogue Adventure,
/// 1318) has no road corner layer to narrow them.
#[test]
fn the_road_runs_full_width_onto_a_bridge() {
    use trpg_content::tileset::{Layer, mix};

    let content = match trpg_content::load_embedded() {
        Ok(content) => content,
        Err(errors) => panic!("{errors}"),
    };
    let display = &content.terrain.display;
    let road = display.id_of("road").expect("no road");
    let bridge = display.id_of("bridge").expect("no bridge");
    let terrain = content.tilesets["tiny_tales"]
        .terrain
        .as_ref()
        .expect("no terrain tiles");
    for (name, look) in &terrain.looks {
        // The road's own layer is the one whose tiles include a bridge.
        let layer = look.layers.iter().find_map(|layer| match layer {
            Layer::Corners(c) if c.of.contains(&road) && c.of.contains(&bridge) => Some(c),
            _ => None,
        });
        let Some(layer) = layer else {
            // Whole tiles: no corner layer has the road at all.
            let joined = look.layers.iter().any(|layer| match layer {
                Layer::Corners(c) => c.of.contains(&road),
                Layer::Tiles(_) => false,
            });
            assert!(!joined, "look {name}: the road doesn't join a bridge");
            continue;
        };
        // Road on one side of a bridge, bridge on the other.
        for tiles in [[road, bridge, road, bridge], [road, road, bridge, bridge]] {
            let picked = mix(tiles.map(|t| layer.of.contains(&t)));
            assert_eq!(
                picked, 15,
                "look {name}: a road beside a bridge is not whole"
            );
        }
    }
}

/// Chapter 1's map is drawn with Rogue Adventure (ticket 1318, 0096): its
/// look gives the plain, the forest, the road, the water and the bridge
/// each a tile of their own, none the outdoor look's, and its water moves.
#[test]
fn chapter_1s_ground_and_river_are_rogue_adventures() {
    use trpg_content::motion::GraphicsTier;

    let content = match trpg_content::load_embedded() {
        Ok(content) => content,
        Err(errors) => panic!("{errors}"),
    };
    assert_eq!(content.maps["ch01"].look.tiles, "rogue_adventure");
    let tileset = &content.tilesets["tiny_tales"];
    let rogue = tileset.look("rogue_adventure").expect("no look");
    let outdoor = tileset.look("outdoor").expect("no look");
    let display = &content.terrain.display;
    let tile = |look: &trpg_content::tileset::Look, id: &str| {
        look.tiles[&display.id_of(id).expect("no terrain")]
    };
    let ground = ["plain", "forest", "road", "water"].map(|id| tile(rogue, id));
    for (id, rect) in ["plain", "forest", "road", "water"].into_iter().zip(ground) {
        assert_ne!(rect, tile(outdoor, id), "{id} is Tiny Tales'");
    }
    // Four different pictures: the forest has a floor of its own.
    let different: std::collections::BTreeSet<_> = ground.iter().map(|r| (r.x, r.y)).collect();
    assert_eq!(different.len(), 4);
    // Water under the bridge; the bridge's planks over it.
    assert_eq!(tile(rogue, "bridge"), tile(rogue, "water"));
    // The river moves, and stands still on a lesser tier.
    let water = tile(rogue, "water");
    let frames = &tileset.frames;
    assert!(
        (1..2000)
            .step_by(40)
            .any(|t| frames.picture(water, GraphicsTier::Full, t) != water)
    );
    assert_eq!(frames.picture(water, GraphicsTier::Reduced, 500), water);
}

/// Chapter 1's hills and peaks are one raised area (ticket 1359): a rim
/// joined at corners round both, with a picture for every mix but all
/// four (the grass of the hill's top), over the peaks' own rock, the
/// cliff's face below it and its shadow under both (1361); Tiny Tales'
/// hills and peaks are gone from the look.
#[test]
fn chapter_1s_hills_and_peaks_are_one_raised_area_with_a_rim() {
    use std::collections::BTreeSet;
    use trpg_content::tileset::Layer;

    let content = match trpg_content::load_embedded() {
        Ok(content) => content,
        Err(errors) => panic!("{errors}"),
    };
    let tileset = &content.tilesets["tiny_tales"];
    let rogue = tileset.look("rogue_adventure").expect("no look");
    let outdoor = tileset.look("outdoor").expect("no look");
    let id = |name: &str| content.terrain.display.id_of(name).expect("no terrain");
    let (mountain, peak) = (id("mountain"), id("peak"));
    let corners_of = |look: &trpg_content::tileset::Look, of: &BTreeSet<_>| {
        look.layers
            .iter()
            .position(|l| matches!(l, Layer::Corners(c) if c.of == *of))
    };
    let raised = BTreeSet::from([mountain, peak]);
    // Its shadow first (ticket 1361), then the rim with the cliff's face
    // below: pictures half a tile taller than a tile, never on a wall.
    let shadow = corners_of(rogue, &raised).expect("no rim round hills and peaks");
    let rim = shadow + 1;
    let (Layer::Corners(shade), Some(Layer::Corners(layer))) =
        (&rogue.layers[shadow], rogue.layers.get(rim))
    else {
        panic!("no shadow under the rim: {:?}", &rogue.layers[shadow..]);
    };
    assert_eq!(layer.of, raised);
    assert_eq!((shade.offset, layer.offset), ((1, 3), (0, 0)));
    assert_eq!(layer.skip, BTreeSet::from([id("wall")]));
    assert_eq!(shade.skip, layer.skip);
    for pieces in [layer, shade] {
        let drawn: Vec<usize> = (0..16).filter(|&m| pieces.tiles[m].is_some()).collect();
        assert_eq!(drawn, (1..15).collect::<Vec<_>>());
        let sizes: BTreeSet<_> = pieces.tiles.iter().flatten().map(|r| (r.w, r.h)).collect();
        assert_eq!(sizes, BTreeSet::from([(16, 24)]));
    }
    // The peaks' rock, under the rim, isn't the outdoor look's.
    let rock = corners_of(rogue, &BTreeSet::from([peak])).expect("no rock on the peaks");
    assert!(rock < rim);
    assert_ne!(
        rogue.layers[rock],
        outdoor.layers[corners_of(outdoor, &BTreeSet::from([peak])).expect("outdoor peaks")]
    );
    // Tiny Tales' hills are only outdoors.
    assert!(corners_of(rogue, &BTreeSet::from([mountain])).is_none());
    assert!(corners_of(outdoor, &BTreeSet::from([mountain])).is_some());
}

/// Harl's fort is the stone keep and Harrowby a round tower (ticket 1360,
/// 0096): drawn last, the face hung on the tile below each wall (never on
/// a wall or a gate), the wall's top by its walls beside it, the tower two
/// tiles tall standing on its fort, the keep's fort a stone by its floor;
/// the floor paved, and Tiny Tales' floor and fort gone from the look.
#[test]
fn chapter_1s_fort_is_the_stone_keep_and_harrowby_a_tower() {
    use std::collections::BTreeSet;
    use trpg_content::tileset::Layer;

    let content = match trpg_content::load_embedded() {
        Ok(content) => content,
        Err(errors) => panic!("{errors}"),
    };
    let tileset = &content.tilesets["tiny_tales"];
    let rogue = tileset.look("rogue_adventure").expect("no look");
    let outdoor = tileset.look("outdoor").expect("no look");
    let id = |name: &str| content.terrain.display.id_of(name).expect("no terrain");
    let (wall, fort, floor) = (id("wall"), id("fort"), id("floor"));
    let keep = rogue.layers.len() - 4;
    let [
        Layer::Corners(face),
        Layer::Tiles(top),
        Layer::Corners(tower),
        Layer::Tiles(stone),
    ] = &rogue.layers[keep..]
    else {
        panic!("the keep isn't drawn last: {:?}", &rogue.layers[keep..]);
    };
    assert_eq!(
        (&face.of, &top.of, &top.beside),
        (&BTreeSet::from([wall]), &face.of, &face.of)
    );
    let gates = BTreeSet::from([wall, id("road"), id("gate"), id("door")]);
    assert_eq!((face.offset, &face.skip), ((8, 24), &gates));
    // Across, up and down, and a pillar: three pictures.
    let at = |r: &trpg_content::tileset::ImageRect| (r.x, r.y, r.w, r.h);
    let tops: BTreeSet<_> = top
        .sides
        .iter()
        .flatten()
        .chain([&top.tile])
        .map(at)
        .collect();
    assert_eq!(tops.len(), 3);
    assert_eq!(tower.of, BTreeSet::from([fort]));
    assert_eq!(
        (tower.offset, &tower.skip),
        ((8, -8), &BTreeSet::from([wall, floor]))
    );
    let tall: BTreeSet<_> = tower.tiles.iter().flatten().map(|r| (r.w, r.h)).collect();
    assert_eq!(tall, BTreeSet::from([(16, 32)]));
    // The stone covers the tower's foot on a fort by a floor.
    assert_eq!(
        (&stone.of, &stone.beside),
        (&tower.of, &BTreeSet::from([floor]))
    );
    assert!(stone.sides[1..].iter().all(Option::is_some));
    assert_eq!(stone.sides[0], None);
    // The paving isn't Tiny Tales'; no layer of the look is its floor's
    // rim or its fort.
    assert_ne!(rogue.tiles[&floor], outdoor.tiles[&floor]);
    let tiny_tales_floor = outdoor.layers.iter().find(
        |l| matches!(l, Layer::Corners(c) if c.of.contains(&floor) && !c.of.contains(&id("water"))),
    );
    let tiny_tales_fort = outdoor
        .layers
        .iter()
        .find(|l| matches!(l, Layer::Tiles(t) if t.of == BTreeSet::from([fort])));
    for layer in [tiny_tales_floor, tiny_tales_fort] {
        let layer = layer.expect("outdoors has it");
        assert!(!rogue.layers.contains(layer), "{layer:?}");
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
            "Shironejiya battle pictures (the materials in this game may not be reused)",
            "Classes and Mobs pixel-art TRPG pieces",
            "Fantasy Battle Pack",
            // Chapter 1's ground and river (ticket 1318).
            "Rogue Adventure World (modified: a wall mirrored)",
            // Chapter 1's hills (ticket 1359).
            "Lonesome Forest (modified: stones recolored)",
            // The title's campfire (ticket 0922).
            "Idylwild's Chromatic Campfires",
        ]
    );
}

/// The bought backdrops (`cargo xtask battler-import`, ticket 0438): a
/// fight at dusk or at night is in front of the bundle's own picture of
/// the place, not of the day picture shaded.
#[test]
fn the_bought_backdrops_have_their_own_dusk_and_night() {
    let content = match trpg_content::load_embedded() {
        Ok(content) => content,
        Err(errors) => panic!("{errors}"),
    };
    let backdrops = &content.battlers.backdrops;
    for light in ["dusk", "night"] {
        for terrain in [None, Some("plain"), Some("mountain"), Some("peak")] {
            let found = backdrops.in_light(terrain, light);
            let Some((image, true)) = found else {
                panic!("{light} on {terrain:?}: {found:?}");
            };
            let path = image.path();
            assert!(path.ends_with(&format!("_{light}.png")), "{path}");
        }
    }
}

/// Chapter 3's speaking enemies (ticket 0861): the prior of Greywater and
/// Ned Tolley have their own generator faces, with every expression their
/// scenes use, and the prior his own combat picture, not his class's.
#[test]
fn chapter_three_speakers_have_their_own_faces_and_the_prior_his_picture() {
    use trpg_content::bundle::{display_path, is_private};

    let content = match trpg_content::load_embedded() {
        Ok(content) => content,
        Err(errors) => panic!("{errors}"),
    };
    for id in ["greywater_prior", "hound_sergeant"] {
        let portrait = content.portraits.get(id);
        let Some(portrait) = portrait else {
            panic!("no portrait {id}")
        };
        for name in ["neutral", "angry", "sad", "surprised"] {
            let found = portrait.expressions.iter().find(|e| e.name == name);
            let Some(expression) = found else {
                panic!("{id} has no {name}")
            };
            let shown = display_path(expression.image.path());
            assert!(
                is_private(&shown) && shown.contains(&format!("portraits/{id}/")),
                "{id} {name}: {shown}"
            );
        }
    }
    let prior = content.battlers.pick(["greywater_prior", "mage"]);
    let Some(prior) = prior else {
        panic!("no battler for the prior")
    };
    assert_eq!(prior.id, "greywater_prior");
    let shown = display_path(prior.image.path());
    assert!(is_private(&shown), "{shown}");
}
