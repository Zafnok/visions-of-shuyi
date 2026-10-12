use super::*;
use crate::image::png_header;

/// An image table of `files`: bundle path, width, height.
fn table(files: &[(&'static str, u32, u32)]) -> ImageTable {
    let pngs: Vec<_> = files
        .iter()
        .map(|&(path, w, h)| (path, png_header(w, h)))
        .collect();
    ImageTable::from_files(pngs.iter().map(|(path, png)| (*path, &png[..]))).unwrap_or_default()
}

fn images() -> ImageTable {
    table(&[
        ("battlers/t.png", 64, 72),
        ("battlers/edge.png", 120, 72),
        ("battlers/wide.png", 121, 72),
        ("battlers/tall.png", 64, 73),
        ("battlers/full.png", 240, 144),
        ("battlers/wider.png", 241, 144),
        ("battlers/taller.png", 240, 145),
        ("battlebacks/fields.png", 336, 248),
        ("battlebacks/hills.png", 336, 248),
        ("battlebacks/narrow.png", 335, 248),
        ("battlebacks/short.png", 336, 247),
        // Not under `battlers/` or `battlebacks/`: neither file can name it.
        ("images/t.png", 64, 72),
        ("images/fields.png", 336, 248),
    ])
}

fn parse(src: &str) -> Result<Battler, Vec<ContentError>> {
    parse_battler("t.ron", "t", src, &images())
}

fn messages<T>(result: Result<T, Vec<ContentError>>) -> Vec<String> {
    result
        .err()
        .unwrap_or_default()
        .iter()
        .map(ToString::to_string)
        .collect()
}

/// A sidecar for `t` with image `image` and `rest` after its facing.
fn sidecar(image: &str, rest: &str) -> String {
    format!(r#"(id: "t", image: "{image}", faces: Right{rest})"#)
}

#[test]
fn a_sidecar_names_its_image_its_facing_and_its_scale() {
    let b = parse(r#"(id: "t", image: "t.png", faces: Left, scale: (5, 4))"#).unwrap();
    assert_eq!(b.id, "t");
    assert_eq!(b.image.path(), "battlers/t.png");
    assert_eq!(
        b.size,
        ImageInfo {
            width: 64,
            height: 72
        }
    );
    assert_eq!(b.faces, Facing::Left);
    assert_eq!(b.scale, (5, 4));
    // 64 × 5/4 and 72 × 5/4; doubled for the scene.
    assert_eq!(b.scaled(1), (80, 90));
    assert_eq!(b.scaled(2), (160, 180));
}

#[test]
fn the_scale_defaults_to_the_pictures_own_size() {
    let b = parse(&sidecar("t.png", "")).unwrap();
    assert_eq!((b.faces, b.scale), (Facing::Right, (1, 1)));
    assert_eq!(b.scaled(1), (64, 72));
    assert_eq!(b.scaled(3), (192, 216));
}

#[test]
fn a_syntax_error_and_an_unknown_field_are_reported_with_their_place() {
    let missing = messages(parse(r#"(id: "t", image: "t.png")"#));
    assert_eq!(missing.len(), 1);
    assert!(missing[0].starts_with("t.ron:1:"), "{missing:?}");
    assert!(missing[0].contains("faces"), "{missing:?}");
    let unknown = messages(parse(&sidecar("t.png", ", frames: 3")));
    assert_eq!(unknown.len(), 1);
    assert!(unknown[0].starts_with("t.ron:1:"), "{unknown:?}");
    assert!(unknown[0].contains("frames"), "{unknown:?}");
}

#[test]
fn the_id_must_be_the_file_stem() {
    assert_eq!(
        messages(parse(r#"(id: "u", image: "t.png", faces: Right)"#)),
        [r#"t.ron: id "u" doesn't match the file name; name the file u.ron"#]
    );
}

#[test]
fn the_image_must_be_a_png_under_the_battlers_directory() {
    assert_eq!(
        messages(parse(&sidecar("none.png", ""))),
        ["t.ron: image: assets/battlers/none.png is not a PNG image in the assets"]
    );
    assert_eq!(
        messages(parse(&sidecar("../images/t.png", ""))),
        ["t.ron: image: assets/battlers/../images/t.png is not a PNG image in the assets"]
    );
}

#[test]
fn the_scale_is_from_one_to_two_times() {
    let scaled = |scale: &str| parse(&sidecar("t.png", &format!(", scale: {scale}")));
    for ok in ["(1, 1)", "(2, 1)", "(3, 2)", "(4, 4)", "(8, 4)"] {
        assert!(scaled(ok).is_ok(), "{ok}");
    }
    for bad in ["(3, 4)", "(9, 4)", "(1, 0)", "(0, 0)", "(0, 1)"] {
        assert_eq!(
            messages(scaled(bad)),
            [format!(
                "t.ron: scale {bad} must be from 1 to 2 times: (1, 1) to (2, 1)"
            )],
            "{bad}"
        );
    }
}

#[test]
fn a_scaled_picture_must_fit_the_scene() {
    let of = |image: &str, scale: &str| parse(&sidecar(image, &format!(", scale: {scale}")));
    assert!(of("full.png", "(1, 1)").is_ok());
    assert!(of("edge.png", "(2, 1)").is_ok());
    let too_big = |image: &str, scale: &str, size: &str| {
        assert_eq!(
            messages(of(image, scale)),
            [format!(
                "t.ron: image: assets/battlers/{image} is {size} px at scale {scale}; \
                 a combat picture must fit 240×144 px"
            )]
        );
    };
    too_big("wider.png", "(1, 1)", "241×144");
    too_big("taller.png", "(1, 1)", "240×145");
    too_big("wide.png", "(2, 1)", "242×144");
    too_big("tall.png", "(2, 1)", "128×146");
}

#[test]
fn every_problem_of_a_sidecar_is_reported_at_once() {
    assert_eq!(
        messages(parse(
            r#"(id: "u", image: "none.png", faces: Right, scale: (1, 2))"#
        )),
        [
            r#"t.ron: id "u" doesn't match the file name; name the file u.ron"#,
            "t.ron: scale (1, 2) must be from 1 to 2 times: (1, 1) to (2, 1)",
            "t.ron: image: assets/battlers/none.png is not a PNG image in the assets",
        ]
    );
}

fn battler(id: &str) -> Battler {
    Battler {
        id: id.to_owned(),
        ..parse(&sidecar("t.png", "")).unwrap()
    }
}

#[test]
fn a_fighter_gets_the_first_battler_named_else_the_default() {
    let mut table = BattlerTable::default();
    assert_eq!(table.pick(["guard"]), None);
    for id in ["guard", "retainer", DEFAULT_BATTLER] {
        table.battlers.insert(id.to_owned(), battler(id));
    }
    let picked = |ids: &[&str]| table.pick(ids.iter().copied()).map(|b| b.id.clone());
    assert_eq!(picked(&["retainer", "guard"]).as_deref(), Some("retainer"));
    assert_eq!(picked(&["red_guard", "guard"]).as_deref(), Some("guard"));
    assert_eq!(picked(&["nobody", "mage"]).as_deref(), Some("default"));
    assert_eq!(picked(&[]).as_deref(), Some("default"));
}

#[test]
fn the_default_battler_must_exist() {
    let mut battlers = BTreeMap::new();
    assert_eq!(
        missing_default(&battlers).map(|e| e.to_string()).as_deref(),
        Some(
            "assets/battlers/default.ron: missing: the picture of a fighter whose class and \
             character have none"
        )
    );
    battlers.insert("guard".to_owned(), battler("guard"));
    assert!(missing_default(&battlers).is_some());
    battlers.insert(DEFAULT_BATTLER.to_owned(), battler(DEFAULT_BATTLER));
    assert_eq!(missing_default(&battlers), None);
}

/// The shipped terrains: `plain`, `mountain`, … (no `swamp`, no `moor`).
fn terrains() -> TerrainDisplayTable {
    crate::TerrainDef::load(None).unwrap_or_default().display
}

fn backdrops(src: &str) -> Result<Backdrops, Vec<ContentError>> {
    parse_backdrops("b.ron", src, &images(), Some(&terrains()))
}

#[test]
fn a_terrain_has_its_backdrop_or_the_default() {
    let b = backdrops(r#"(default: "fields.png", terrain: { "mountain": "hills.png" })"#).unwrap();
    let path = |terrain| b.of(terrain).map(ImageId::path);
    assert_eq!(path("mountain"), Some("battlebacks/hills.png"));
    assert_eq!(path("plain"), Some("battlebacks/fields.png"));
    assert_eq!(path("no_such"), Some("battlebacks/fields.png"));
    // No terrain needs naming.
    let plain = backdrops(r#"(default: "fields.png")"#).unwrap();
    assert_eq!(
        plain.of("mountain").map(ImageId::path),
        Some("battlebacks/fields.png")
    );
    assert_eq!(Backdrops::default().of("plain"), None);
}

#[test]
fn every_problem_of_the_backdrops_is_reported_at_once() {
    let src = r#"(default: "none.png", terrain: { "plain": "narrow.png",
        "mountain": "short.png", "swamp": "hills.png", "moor": "../images/fields.png" })"#;
    assert_eq!(
        messages(backdrops(src)),
        [
            "b.ron: default: assets/battlebacks/none.png is not a PNG image in the assets",
            "b.ron: terrain \"moor\": assets/battlebacks/../images/fields.png is not a PNG image in the assets",
            "b.ron: terrain \"mountain\": assets/battlebacks/short.png is 336×247 px; a backdrop is 336×248 px",
            "b.ron: terrain \"plain\": assets/battlebacks/narrow.png is 335×248 px; a backdrop is 336×248 px",
            "b.ron: terrain \"moor\": no such terrain",
            "b.ron: terrain \"swamp\": no such terrain",
        ]
    );
    // Without the terrain file (it failed to load), ids aren't checked.
    let unchecked = parse_backdrops(
        "b.ron",
        r#"(default: "fields.png", terrain: { "swamp": "hills.png" })"#,
        &images(),
        None,
    );
    assert!(unchecked.is_ok(), "{unchecked:?}");
    assert_eq!(
        messages(backdrops(r#"(default: "fields.png", sky: 1)"#)).len(),
        1
    );
}

#[test]
fn the_shipped_battlers_and_backdrops_load() {
    let images = ImageTable::load().unwrap_or_default();
    let table = load_all(&images, None);
    assert!(table.is_ok(), "{table:?}");
    let table = table.unwrap_or_default();
    assert!(table.battlers.contains_key(DEFAULT_BATTLER));
    assert!(table.backdrops.default.is_some());
    for (id, b) in &table.battlers {
        assert_eq!(&b.id, id);
    }
}
