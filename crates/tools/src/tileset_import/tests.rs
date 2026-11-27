//! Tests of the importer, on a made-up Tiled sheet: never a bought one.

use std::collections::BTreeSet;
use std::path::PathBuf;

use trpg_content::tileset::{Layer, TilesetRefs, parse_tileset};

use super::*;
use crate::map_sprite_import::{For, Source};

/// A Tiled tileset file like the bought ones: 4 × 2 tiles of 16 px, two
/// terrains with tiles and one without, a tile whose corners mix two
/// terrains, an animated tile and a tile with no terrain.
const TSX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!-- made up -->
<tileset name="Made &amp; up" tilewidth="16" tileheight="16" tilecount="8" columns="4">
 <image source="Images/made.png" trans="ff01fe" width="64" height="32"/>
 <terraintypes>
  <terrain name="Mud" tile="0"/>
  <terrain name="Deep &quot;Mud&quot;" tile="1"/>
  <terrain name="Empty" tile="-1"/>
 </terraintypes>
 <tile id="0" terrain=",,,0"/>
 <tile id="1" terrain="0,0,0,0"/>
 <tile id="2" terrain="0,0,0,0">
  <animation>
   <frame tileid="2" duration="150"/>
  </animation>
 </tile>
 <tile id="3" terrain="0,,,"/>
 <tile id="4" terrain="1,1,,"/>
 <tile id="5" terrain="0,1,0,0"/>
 <tile id="6"/>
</tileset>
"#;

/// The colour of tile `id` of the made-up image.
fn color(id: u32) -> [u8; 4] {
    [u8::try_from(10 * id + 10).unwrap(), 100, 200, 255]
}

/// The made-up image, 64 × 32: each tile one colour, but tile 0's first
/// pixel the clear colour, tile 1's second pixel see-through, and tile 7
/// all the clear colour.
fn made_image() -> Vec<u8> {
    let mut rgba = Vec::new();
    for y in 0..32u32 {
        for x in 0..64u32 {
            let id = (y / 16) * 4 + x / 16;
            let pixel = match (id, x % 16, y % 16) {
                (0, 0, 0) | (7, ..) => [255, 1, 254, 255],
                (1, 1, 0) => [9, 9, 9, 0],
                _ => color(id),
            };
            rgba.extend(pixel);
        }
    }
    rgba
}

/// A folder, removed when the test is over.
struct Temp(PathBuf);

impl Temp {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("xtask-ti-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    /// Writes `bytes` at `path` under the folder.
    fn write(&self, path: &str, bytes: &[u8]) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    /// The made-up sheet as Tiled sheet `set/Made` of the packs in `dir`
    /// of the folder.
    fn with_sheet(self, dir: &str) -> Self {
        self.write(&format!("{dir}/{TILED_DIR}/set/Made.tsx"), TSX.as_bytes());
        let png = encode_png(64, 32, &made_image()).unwrap();
        self.write(&format!("{dir}/{TILED_DIR}/set/Images/made.png"), &png);
        self
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Our terrains, in `terrain.ron`'s order.
fn terrains() -> Vec<String> {
    let content = trpg_content::load_embedded().unwrap();
    let display = &content.terrain.display;
    display.terrains.iter().map(|t| t.id.clone()).collect()
}

/// A mapping of the made-up sheet: every terrain tile 1, but water tile
/// 6; two looks; `layers` and `looks` as given.
fn mapping_text(layers: &str, looks: &str) -> String {
    let mut terrain = String::new();
    for id in terrains() {
        let tile = if id == "water" { 6 } else { 1 };
        let _ = write!(terrain, "\"{id}\": (\"set/Made\", {tile}), ");
    }
    format!("(terrain: {{ {terrain} }}, layers: {{ {layers} }}, looks: {{ {looks} }})")
}

const LAYERS: &str = r##"
    "mud": Corners(of: ["plain"], sheet: "set/Made", terrain: "Mud"),
    "edge": Corners(of: ["plain", "road"], sheet: "set/Made", terrain: "Mud",
                    pick: 1, full: false, tint: "fire"),
    "span": Tiles(of: ["bridge"], tile: ("set/Made", 6), beside: ["water"],
                  sides: { ".#.#": ("set/Made", 1), "#...": ("set/Made", 6) }),
    "fort": Tiles(of: ["fort"], tile: ("set/Made", 3)),
"##;

const LOOKS: &str = r#"
    "outdoor": (layers: ["mud", "span", "fort"]),
    "indoor": (terrain: { "water": ("set/Made", 1) }, layers: ["edge"]),
"#;

fn mapping(layers: &str, looks: &str) -> Mapping {
    ron::from_str(&mapping_text(layers, looks)).unwrap()
}

/// The palette of the tests: only `fire`.
fn fire(name: &str) -> Option<[u8; 3]> {
    (name == "fire").then_some([200, 100, 0])
}

#[test]
fn tags_are_read_with_their_attributes() {
    let tags = tags(TSX);
    let names: Vec<&str> = tags.iter().map(|(name, _)| name.as_str()).collect();
    // The declaration, the comment and closing tags aren't elements.
    assert_eq!(
        names,
        [
            "tileset",
            "image",
            "terraintypes",
            "terrain",
            "terrain",
            "terrain",
            "tile",
            "tile",
            "tile",
            "animation",
            "frame",
            "tile",
            "tile",
            "tile",
            "tile"
        ]
    );
    let attribute = |tag: usize, key: &str| tags[tag].1.get(key).map(String::as_str);
    assert_eq!(attribute(0, "name"), Some("Made & up"));
    assert_eq!(attribute(0, "columns"), Some("4"));
    assert_eq!(attribute(1, "source"), Some("Images/made.png"));
    assert_eq!(attribute(4, "name"), Some("Deep \"Mud\""));
    assert_eq!(attribute(6, "terrain"), Some(",,,0"));
    // A self-closing tag and an open one read the same.
    assert_eq!(attribute(8, "terrain"), Some("0,0,0,0"));
    assert_eq!(attribute(14, "id"), Some("6"));
    assert_eq!(tags[2].1.len(), 0);
    // No element, a bare one, and an attribute cut short.
    assert_eq!(
        super::tags("no tags"),
        [] as [(
            std::string::String,
            std::collections::BTreeMap<std::string::String, std::string::String>
        ); 0]
    );
    assert_eq!(super::tags("<a/>")[0], ("a".to_owned(), BTreeMap::new()));
    assert!(super::tags("<a b=\"c>")[0].1.is_empty());
    assert_eq!(unescape("&lt;a&gt; &apos;b&apos; &amp;lt;"), "<a> 'b' &lt;");
}

#[test]
fn a_tsx_gives_its_image_its_terrains_and_its_tiles_corners() {
    let tsx = parse_tsx(TSX).unwrap();
    assert_eq!(tsx.image, "Images/made.png");
    assert_eq!(tsx.clear, Some([255, 1, 254]));
    assert_eq!((tsx.tile, tsx.columns), ((16, 16), 4));
    assert_eq!(tsx.terrains, ["Mud", "Deep \"Mud\"", "Empty"]);
    let (mud, deep) = (Some(0), Some(1));
    assert_eq!(
        tsx.corners,
        [
            (0, [None, None, None, mud]),
            (1, [mud; 4]),
            (2, [mud; 4]),
            (3, [mud, None, None, None]),
            (4, [deep, deep, None, None]),
            (5, [mud, deep, mud, mud]),
        ]
    );
    // No `trans`: no clear colour.
    let plain = TSX.replace(" trans=\"ff01fe\"", "");
    assert_eq!(parse_tsx(&plain).unwrap().clear, None);
}

#[test]
fn a_tsx_that_cant_be_read_says_why() {
    let error = |from: &str, to: &str| parse_tsx(&TSX.replace(from, to)).unwrap_err();
    assert_eq!(parse_tsx("<map/>").unwrap_err(), "no <tileset>");
    assert_eq!(error(" columns=\"4\"", ""), "<tileset> has no columns");
    assert_eq!(
        error("columns=\"4\"", "columns=\"0\""),
        "<tileset> has no columns"
    );
    assert_eq!(
        error("tilewidth=\"16\"", "tilewidth=\"x\""),
        "<tileset> has no tilewidth"
    );
    assert_eq!(
        error("tileheight=\"16\" ", ""),
        "<tileset> has no tileheight"
    );
    assert_eq!(error("<image", "<picture"), "no <image>");
    assert_eq!(error("source=", "src="), "<image> has no source");
    assert_eq!(
        error("ff01fe", "ff01fg"),
        "trans \"ff01fg\" is not a colour"
    );
    for bad in ["0,0,0", "0,0,0,0,0", "a,,,"] {
        let message = format!("a <tile> with terrain \"{bad}\" is not read");
        assert_eq!(error("0,,,", bad), message);
    }
    let no_id = error("<tile id=\"3\" terrain", "<tile terrain");
    assert_eq!(no_id, "a <tile> with terrain \"0,,,\" is not read");
}

#[test]
fn colours_and_corners_are_read_as_written() {
    assert_eq!(hex("ff01fe"), Some([255, 1, 254]));
    assert_eq!(hex("#0a0B0c"), Some([10, 11, 12]));
    for bad in ["", "ff01f", "ff01fe0", "ff01fg", "ff01fé", "ééé"] {
        assert_eq!(hex(bad), None, "{bad}");
    }
    assert_eq!(corner_terrains(",,,0"), Some([None, None, None, Some(0)]));
    assert_eq!(
        corner_terrains("12, ,3,"),
        Some([Some(12), None, Some(3), None])
    );
    assert_eq!(corner_terrains(",,,"), Some([None; 4]));
    for bad in ["", ",,", ",,,,", "-1,,,"] {
        assert_eq!(corner_terrains(bad), None, "{bad}");
    }
}

#[test]
fn a_sheets_tiles_are_cut_with_the_clear_colour_as_nothing() {
    let temp = Temp::new("sheet").with_sheet("lib");
    let dir = temp.0.join("lib");
    let sheet = Sheet::read(&dir, "set/Made").unwrap();
    assert_eq!(sheet.size, (64, 32));
    assert_eq!(sheet.rgba.len(), 64 * 32 * 4);
    // Tile 0: its first pixel is the clear colour, the rest its colour.
    let tile = sheet.tile(0).unwrap();
    assert_eq!(tile.len(), 16 * 16 * 4);
    assert_eq!(tile[..4], [0; 4]);
    assert_eq!(tile[4..8], color(0));
    assert_eq!(tile[16 * 16 * 4 - 4..], color(0));
    // Tile 1: a see-through pixel is nothing, whatever its colour.
    let tile = sheet.tile(1).unwrap();
    assert_eq!((&tile[..4], &tile[4..8]), (&color(1)[..], &[0; 4][..]));
    // Tile 5 is the second of the second row; tile 7 is all clear.
    assert!(sheet.tile(5).unwrap().chunks(4).all(|p| p == color(5)));
    assert!(sheet.tile(7).unwrap().iter().all(|&v| v == 0));
    assert_eq!(
        sheet.tile(8).unwrap_err(),
        "tile 8 is outside the 64×32 px image"
    );
    // An image narrower than the file says: the tile past it isn't there.
    let mut narrow = sheet.clone();
    narrow.size = (63, 32);
    assert!(narrow.tile(3).is_err() && narrow.tile(2).is_ok());
    narrow.size = (64, 31);
    assert!(narrow.tile(4).is_err() && narrow.tile(3).is_ok());
}

#[test]
fn a_sheet_that_cant_be_read_says_which_file() {
    let temp = Temp::new("bad-sheet").with_sheet("lib");
    let dir = temp.0.join("lib");
    let err = Sheet::read(&dir, "set/None").unwrap_err();
    assert!(
        err.starts_with("reading ") && err.contains("None.tsx"),
        "{err}"
    );
    temp.write("lib/tiny-tales/tilesets/set/Bad.tsx", b"<map/>");
    let err = Sheet::read(&dir, "set/Bad").unwrap_err();
    assert!(err.ends_with("Bad.tsx: no <tileset>"), "{err}");
    temp.write(
        "lib/tiny-tales/tilesets/set/Big.tsx",
        TSX.replace("tilewidth=\"16\"", "tilewidth=\"8\"")
            .as_bytes(),
    );
    let err = Sheet::read(&dir, "set/Big").unwrap_err();
    assert!(
        err.ends_with("Big.tsx: its tiles are 8×16 px, not 16×16"),
        "{err}"
    );
    temp.write(
        "lib/tiny-tales/tilesets/set/Lost.tsx",
        TSX.replace("made.png", "lost.png").as_bytes(),
    );
    let err = Sheet::read(&dir, "set/Lost").unwrap_err();
    assert!(
        err.starts_with("reading ") && err.contains("lost.png"),
        "{err}"
    );
    temp.write("lib/tiny-tales/tilesets/set/Images/text.png", b"not a png");
    temp.write(
        "lib/tiny-tales/tilesets/set/Text.tsx",
        TSX.replace("made.png", "text.png").as_bytes(),
    );
    let err = Sheet::read(&dir, "set/Text").unwrap_err();
    assert!(err.contains("text.png: "), "{err}");
}

#[test]
fn a_terrains_tiles_are_found_by_the_mix_of_their_corners() {
    let sheet = Sheet {
        tsx: parse_tsx(TSX).unwrap(),
        size: (64, 32),
        rgba: made_image(),
    };
    let mud = sheet.corners("Mud", 0).unwrap();
    let mut expect = [None; 16];
    expect[0b0001] = Some(0);
    expect[0b1000] = Some(3);
    expect[0b1111] = Some(1);
    assert_eq!(mud, expect);
    // The second listed, where there is one; else the last.
    expect[0b1111] = Some(2);
    assert_eq!(sheet.corners("Mud", 1).unwrap(), expect);
    assert_eq!(sheet.corners("Mud", 7).unwrap(), expect);
    // A tile mixing two terrains is neither's.
    let mut expect = [None; 16];
    expect[0b1100] = Some(4);
    assert_eq!(sheet.corners("Deep \"Mud\"", 0).unwrap(), expect);
    assert_eq!(sheet.corners("Empty", 0).unwrap(), [None; 16]);
    assert_eq!(
        sheet.corners("Sand", 0).unwrap_err(),
        "no terrain \"Sand\" (it has: Mud, Deep \"Mud\", Empty)"
    );
}

#[test]
fn a_tint_recolours_each_pixel_as_bright_as_it_was() {
    assert_eq!(brightness([0, 0, 0]), 0);
    assert_eq!(brightness([255, 255, 255]), 255_000);
    assert_eq!(brightness([10, 20, 30]), 2990 + 11_740 + 3420);
    // Mid grey in orange: the orange, 100/118.5 as bright.
    let mut pixels = vec![
        100, 100, 100, 255, 7, 8, 9, 0, 255, 255, 255, 128, 0, 0, 0, 255,
    ];
    tint(&mut pixels, [200, 100, 0]);
    assert_eq!(pixels[..4], [168, 84, 0, 255]);
    // A clear pixel is left alone.
    assert_eq!(pixels[4..8], [7, 8, 9, 0]);
    // White: as bright as the channel goes, its opacity kept.
    assert_eq!(pixels[8..12], [255, 215, 0, 128]);
    assert_eq!(pixels[12..], [0, 0, 0, 255]);
    // Black has no brightness to scale by: black.
    let mut pixels = vec![100, 100, 100, 255];
    tint(&mut pixels, [0, 0, 0]);
    assert_eq!(pixels, [0, 0, 0, 255]);
}

#[test]
fn an_import_packs_each_tile_once_and_writes_the_looks() {
    let temp = Temp::new("import").with_sheet("lib");
    let terrains = terrains();
    let map = mapping(LAYERS, LOOKS);
    let imported = import(&temp.0.join("lib"), &map, &terrains, &fire).unwrap();
    // Six pictures: tile 1 (every terrain's), tile 6 (water's), tiles 0
    // and 3 (the mud's corners), then 0 and 3 again, tinted.
    let (width, height, rgba) = &imported.image;
    assert_eq!((*width, *height), (256, 16));
    assert_eq!(rgba.len(), 256 * 16 * 4);
    let px = |cell: usize, x: usize, y: usize| {
        let at = (y * 256 + cell * 16 + x) * 4;
        [rgba[at], rgba[at + 1], rgba[at + 2], rgba[at + 3]]
    };
    assert_eq!((px(0, 0, 0), px(0, 1, 0)), (color(1), [0; 4]));
    assert_eq!((px(1, 0, 0), px(1, 15, 15)), (color(6), color(6)));
    assert_eq!((px(2, 0, 0), px(2, 1, 0)), ([0; 4], color(0)));
    assert_eq!(px(3, 5, 9), color(3));
    let mut tinted = color(0);
    tint(&mut tinted, [200, 100, 0]);
    assert_eq!((px(4, 0, 0), px(4, 1, 0)), ([0; 4], tinted));
    assert_ne!(px(5, 5, 9), color(3));
    assert_eq!(px(6, 0, 0), [0; 4]);
    // The look of a map that names none: every terrain its tile, in
    // terrain.ron's order.
    let own = &imported.own;
    let ids: Vec<&String> = own.terrain.iter().map(|(id, _)| id).collect();
    assert_eq!(ids, terrains.iter().collect::<Vec<_>>());
    let at = |look: &LookText, id: &str| look.terrain.iter().find(|(t, _)| t == id).unwrap().1;
    assert_eq!((at(own, "plain"), at(own, "water")), ((0, 0), (1, 0)));
    let names = |names: &[&str]| names.iter().map(|&n| n.to_owned()).collect::<Vec<_>>();
    assert_eq!(
        own.layers,
        [
            LayerText::Corners {
                of: names(&["plain"]),
                tiles: vec![(1, (2, 0)), (8, (3, 0)), (15, (0, 0))],
                place: Place::default(),
            },
            LayerText::Tiles {
                of: names(&["bridge"]),
                at: (1, 0),
                beside: names(&["water"]),
                sides: vec![(5, (0, 0)), (8, (1, 0))],
            },
            LayerText::Tiles {
                of: names(&["fort"]),
                at: (3, 0),
                beside: vec![],
                sides: vec![],
            },
        ]
    );
    // The other look: its own tile for water, and the tinted layer
    // without the tile of all four corners.
    assert_eq!(imported.others.len(), 1);
    let (name, indoor) = &imported.others[0];
    assert_eq!(name, "indoor");
    assert_eq!((at(indoor, "plain"), at(indoor, "water")), ((0, 0), (0, 0)));
    assert_eq!(
        indoor.layers,
        [LayerText::Corners {
            of: names(&["plain", "road"]),
            tiles: vec![(1, (4, 0)), (8, (5, 0))],
            place: Place::default(),
        }]
    );
}

#[test]
fn more_tiles_than_a_row_holds_go_on_the_next_row() {
    let temp = Temp::new("rows").with_sheet("lib");
    let dir = temp.0.join("lib");
    let mut packer = Packer {
        dir: &dir,
        strips: &BTreeMap::new(),
        recolours: &BTreeMap::new(),
        colors: &fire,
        sheets: BTreeMap::new(),
        cells: Vec::new(),
        pixels: Vec::new(),
        slots: Vec::new(),
        taken: BTreeSet::new(),
        layouts: Vec::new(),
    };
    // No picture yet: one empty row.
    assert_eq!(packer.image().0, 256);
    assert_eq!(packer.image().1, 16);
    // Seventeen different pictures: the same tiles, tinted or not, and
    // made different by name.
    for i in 0..17u32 {
        packer
            .cells
            .push((format!("filler {i}"), Cut::Tile(0), String::new()));
        packer
            .pixels
            .push(vec![u8::try_from(i).unwrap(); 16 * 16 * 4]);
        packer.slots.push(i);
        packer.taken.insert(i);
    }
    assert_eq!(packer.cell("set/Made", 5, "").unwrap(), (1, 1));
    assert_eq!(packer.cell("set/Made", 5, "").unwrap(), (1, 1));
    assert_eq!(packer.cell("set/Made", 5, "fire").unwrap(), (2, 1));
    assert_eq!(packer.sheets.len(), 1);
    let (width, height, rgba) = packer.image();
    assert_eq!((width, height), (256, 32));
    let px = |x: usize, y: usize| rgba[(y * 256 + x) * 4];
    assert_eq!((px(0, 0), px(15, 15), px(16, 0)), (0, 0, 1));
    assert_eq!((px(255, 15), px(0, 16), px(15, 31)), (15, 16, 16));
    assert_eq!(px(16, 16), color(5)[0]);
    assert_eq!(px(48, 16), 0);
    // What can't be packed says why.
    assert_eq!(
        packer.cell("set/Made", 5, "ice").unwrap_err(),
        "no palette colour \"ice\""
    );
    assert_eq!(
        packer.cell("set/Made", 9, "").unwrap_err(),
        "set/Made: tile 9 is outside the 64×32 px image"
    );
    assert!(
        packer
            .cell("set/None", 0, "")
            .unwrap_err()
            .starts_with("reading ")
    );
}

#[test]
fn a_mapping_is_checked_before_any_file_is_read() {
    let terrains = terrains();
    let check_of = |layers: &str, looks: &str| check(&mapping(layers, looks), &terrains);
    assert_eq!(check_of(LAYERS, LOOKS), Ok(()));
    let err = |layers: &str, looks: &str| check_of(layers, looks).unwrap_err();
    let swapped = |from: &str, to: &str| err(&LAYERS.replace(from, to), LOOKS);
    assert_eq!(
        swapped("of: [\"plain\"]", "of: [\"lava\"]"),
        "layer \"mud\": terrain \"lava\" is not in terrain.ron"
    );
    assert_eq!(
        swapped("of: [\"bridge\"]", "of: [\"moat\"]"),
        "layer \"span\": terrain \"moat\" is not in terrain.ron"
    );
    assert_eq!(
        swapped("beside: [\"water\"]", "beside: [\"lake\"]"),
        "layer \"span\": terrain \"lake\" is not in terrain.ron"
    );
    assert_eq!(
        swapped("\"#...\"", "\"#..\""),
        "layer \"span\": sides \"#..\" must be four of `#` and `.`"
    );
    let looks = |from: &str, to: &str| err(LAYERS, &LOOKS.replace(from, to));
    assert_eq!(
        looks("\"outdoor\"", "\"outside\""),
        "the mapping has no look \"outdoor\""
    );
    assert_eq!(
        looks("\"indoor\"", "\"cave\""),
        "look \"cave\" is not a look a map can name"
    );
    assert_eq!(
        looks("{ \"water\":", "{ \"moat\":"),
        "look \"indoor\": terrain \"moat\" is not in terrain.ron"
    );
    assert_eq!(
        looks("[\"edge\"]", "[\"edges\"]"),
        "look \"indoor\": no layer \"edges\""
    );
    // A terrain with no tile, in the mapping or in a look.
    let mut map = mapping(LAYERS, LOOKS);
    map.terrain.remove("water");
    assert_eq!(
        check(&map, &terrains).unwrap_err(),
        "look \"outdoor\": terrain \"water\" has no tile"
    );
    map.terrain
        .insert("lava".to_owned(), ("set/Made".to_owned(), 1));
    assert_eq!(
        check(&map, &terrains).unwrap_err(),
        "terrain: terrain \"lava\" is not in terrain.ron"
    );
    // Nothing is read: no folder is needed to be told.
    let nowhere = Path::new("no such folder");
    let err = import(nowhere, &map, &terrains, &fire).unwrap_err();
    assert_eq!(err, "terrain: terrain \"lava\" is not in terrain.ron");
}

#[test]
fn what_the_bought_files_lack_is_named_with_its_layer() {
    let temp = Temp::new("lacks").with_sheet("lib");
    let dir = temp.0.join("lib");
    let terrains = terrains();
    let err = |layers: &str| import(&dir, &mapping(layers, LOOKS), &terrains, &fire).unwrap_err();
    let swapped = |from: &str, to: &str| err(&LAYERS.replace(from, to));
    assert_eq!(
        swapped("terrain: \"Mud\"),", "terrain: \"Sand\"),"),
        "layer \"mud\": set/Made: no terrain \"Sand\" (it has: Mud, Deep \"Mud\", Empty)"
    );
    assert_eq!(
        swapped("terrain: \"Mud\"),", "terrain: \"Empty\"),"),
        "layer \"mud\": set/Made: no tile has corners of \"Empty\""
    );
    assert_eq!(
        swapped("tint: \"fire\"", "tint: \"smoke\""),
        "layer \"edge\": no palette colour \"smoke\""
    );
    assert_eq!(
        swapped("tile: (\"set/Made\", 6)", "tile: (\"set/Made\", 8)"),
        "layer \"span\": set/Made: tile 8 is outside the 64×32 px image"
    );
    assert_eq!(
        swapped("\".#.#\": (\"set/Made\", 1)", "\".#.#\": (\"set/Made\", 9)"),
        "layer \"span\": set/Made: tile 9 is outside the 64×32 px image"
    );
    let lost = swapped(
        "sheet: \"set/Made\", terrain: \"Mud\"),",
        "sheet: \"set/Lost\", terrain: \"Mud\"),",
    );
    assert!(lost.starts_with("layer \"mud\": reading "), "{lost}");
    // A terrain's own tile that isn't there.
    let mut map = mapping(LAYERS, LOOKS);
    map.terrain
        .insert("plain".to_owned(), ("set/Made".to_owned(), 8));
    assert_eq!(
        import(&dir, &map, &terrains, &fire).unwrap_err(),
        "look \"outdoor\": set/Made: tile 8 is outside the 64×32 px image"
    );
}

const UNITS: [MapSprite; 4] = [
    MapSprite {
        who: For::Class("exile"),
        name: "a",
        source: Source::Copy("heroes/A/map_sprite.png"),
    },
    // A rider cut from the battle pack, and its picture on foot (0499).
    MapSprite {
        who: For::Class("brigand"),
        name: "horse",
        source: Source::Cut("Lance/Lance_Red1.png"),
    },
    MapSprite {
        who: For::OnFoot("brigand"),
        name: "a",
        source: Source::Copy("heroes/A/map_sprite.png"),
    },
    MapSprite {
        who: For::Fallback,
        name: "a",
        source: Source::Copy("heroes/A/map_sprite.png"),
    },
];

#[test]
fn the_tileset_file_is_one_the_game_reads() {
    let temp = Temp::new("file").with_sheet("lib");
    let map = mapping(LAYERS, LOOKS);
    let imported = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap();
    let text = tileset(&imported, &UNITS);
    assert!(text.starts_with(
        "// Written by `cargo xtask tileset-import` (ticket 0437). Do not edit: change\n\
         // assets-src/tilesets/tiny_tales.ron (the terrain) or the table in\n\
         // crates/xtask/src/map_sprite_import.rs (the units) and run it again.\n\
         (\n    id: \"tiny_tales\",\n    image: \"tilesets/tiny_tales.png\",\n    \
         tile_px: (16, 16),\n    terrain: {\n        \"plain\": (0, 0),\n"
    ));
    assert!(text.contains("    looks: {\n        \"indoor\": (\n"));
    assert!(text.ends_with(
        "        fallback: (image: \"units/a.png\", frame: (1, 0), walk: true),\n        \
         on_foot: {\n            \
         \"brigand\": (image: \"units/a.png\", frame: (1, 0), walk: true),\n        },\n    ),\n)\n"
    ));
    // The game's loader takes it, with the packed image and the unit's
    // sheet in the bundle.
    let (width, height, rgba) = &imported.image;
    let png = encode_png(*width, *height, rgba).unwrap();
    let header = |w: u32, h: u32| {
        let mut sheet = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        sheet.extend(w.to_be_bytes());
        sheet.extend(h.to_be_bytes());
        sheet
    };
    let (sheet, horse) = (header(48, 80), header(72, 96));
    let files = [
        ("tilesets/tiny_tales.png", png.as_slice()),
        ("units/a.png", sheet.as_slice()),
        ("units/horse.png", horse.as_slice()),
    ];
    let content = trpg_content::load_embedded().unwrap();
    let refs = TilesetRefs {
        images: &trpg_content::image::from_files(files).unwrap(),
        terrain: &content.terrain.display,
        classes: &content.classes,
        characters: &content.characters,
    };
    let read = parse_tileset("tiny_tales.ron", "tiny_tales", &text, &refs).unwrap();
    let terrain = read.terrain.clone().unwrap();
    assert_eq!(terrain.tile_px, (16, 16));
    assert_eq!(terrain.look.layers.len(), 3);
    assert_eq!(terrain.looks["indoor"].layers.len(), 1);
    assert_eq!(
        terrain.look.tiles.len(),
        content.terrain.display.terrains.len()
    );
    // The rider's 24 × 24 standing frame, and the class's picture on foot.
    let brigand = trpg_core::ClassId("brigand".to_owned());
    let rect = read.classes[&brigand].rect;
    assert_eq!((rect.x, rect.y, rect.w, rect.h), (24, 0, 24, 24));
    assert_eq!(read.on_foot_picture(&brigand), Some(read.fallback));
}

#[test]
fn the_list_says_which_bought_tiles_each_look_uses() {
    // Each line with its columns' padding squeezed to one space.
    let lines = |text: String| -> Vec<String> {
        let squeezed = |line: &str| line.split_whitespace().collect::<Vec<_>>().join(" ");
        text.lines().map(squeezed).collect()
    };
    let text = list(&mapping(LAYERS, LOOKS));
    // Columns line up: a terrain's tile starts at the same place as a
    // layer's words.
    assert!(
        text.contains("\n  water            set/Made tile 1\n"),
        "{text}"
    );
    assert!(
        text.contains("\n  layer mud        between tiles of plain: "),
        "{text}"
    );
    let lines = lines(text);
    let has = |line: &str| lines.iter().any(|l| l == line);
    assert_eq!(lines[0], "look indoor");
    // A look's own tile where it has one, else the mapping's.
    let water: Vec<&String> = lines.iter().filter(|l| l.starts_with("water ")).collect();
    assert_eq!(water, ["water set/Made tile 1", "water set/Made tile 6"]);
    assert!(has("plain set/Made tile 1"));
    assert!(has(
        "layer edge between tiles of plain, road: set/Made \"Mud\", recoloured fire"
    ));
    assert!(has("look outdoor"));
    assert!(has("layer mud between tiles of plain: set/Made \"Mud\""));
    assert!(has("layer span on bridge: set/Made tile 6"));
    // A layer a look lists that the mapping lacks.
    let looks = LOOKS.replace("[\"edge\"]", "[\"edges\"]");
    let text = list(&mapping(LAYERS, &looks));
    assert!(
        text.contains("  layer edges      (no such layer)\n"),
        "{text}"
    );
}

#[test]
fn the_games_mapping_gives_every_look_every_terrain() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let map = read_mapping(&root).unwrap();
    assert_eq!(check(&map, &terrains()), Ok(()));
    let looks: Vec<&str> = map.looks.keys().map(String::as_str).collect();
    assert_eq!(looks, ["indoor", "outdoor", "rogue_adventure"]);
    // Every layer is used by a look.
    for name in map.layers.keys() {
        let used = map.looks.values().any(|look| look.layers.contains(name));
        assert!(used, "layer {name} is in no look");
    }
}

#[test]
fn run_writes_the_image_and_the_tileset_into_the_private_checkout() {
    let temp = Temp::new("run").with_sheet(LIBRARIES_DIR);
    let root = &temp.0;
    // Without the checkout's game folder, or the mapping: nothing.
    let err = run(root, &UNITS).unwrap_err();
    assert!(
        err.contains("cargo shuyi private-assets --library"),
        "{err}"
    );
    fs::create_dir_all(root.join(GAME_DIR)).unwrap();
    let err = run(root, &UNITS).unwrap_err();
    assert!(
        err.starts_with("reading ") && err.contains("tiny_tales.ron"),
        "{err}"
    );
    temp.write(MAPPING_PATH, b"(terrain: {})");
    let err = run(root, &UNITS).unwrap_err();
    assert!(
        err.starts_with("assets-src/tilesets/tiny_tales.ron: "),
        "{err}"
    );
    assert_eq!(fs::read_dir(root.join(GAME_DIR)).unwrap().count(), 0);
    // With them: both files.
    temp.write(MAPPING_PATH, mapping_text(LAYERS, LOOKS).as_bytes());
    let summary = run(root, &UNITS).unwrap();
    assert_eq!(
        summary,
        "tileset-import: wrote assets-private/game/tilesets/tiny_tales.png (256×16) and \
         assets-private/game/tilesets/tiny_tales.ron (0 moving tiles)"
    );
    let game = root.join(GAME_DIR);
    let png = fs::read(game.join("tilesets/tiny_tales.png")).unwrap();
    let palette = trpg_content::palette::load().unwrap();
    let colors = |name: &str| palette.get(name);
    let map = mapping(LAYERS, LOOKS);
    let imported = import(&root.join(LIBRARIES_DIR), &map, &terrains(), &colors).unwrap();
    assert_eq!(decode_png(&png).unwrap(), imported.image);
    let text = fs::read_to_string(game.join("tilesets/tiny_tales.ron")).unwrap();
    assert_eq!(text, tileset(&imported, &UNITS));
    // Without the library: nothing, though the game folder is there.
    let bare = Temp::new("run-bare");
    fs::create_dir_all(bare.0.join(GAME_DIR)).unwrap();
    let err = run(&bare.0, &UNITS).unwrap_err();
    assert!(err.contains("is missing"), "{err}");
    // A file where the tilesets folder goes fails the write.
    let blocked = Temp::new("run-blocked").with_sheet(LIBRARIES_DIR);
    blocked.write(MAPPING_PATH, mapping_text(LAYERS, LOOKS).as_bytes());
    blocked.write("assets-private/game/tilesets", b"x");
    let err = run(&blocked.0, &UNITS).unwrap_err();
    assert!(err.starts_with("creating "), "{err}");
}

/// A made-up sheet of `columns` × 3 tiles: each pixel is its tile's
/// number, then where it is in the tile.
fn block_sheet(columns: u32) -> Sheet {
    let mut rgba = Vec::new();
    for y in 0..48u32 {
        for x in 0..columns * 16 {
            let id = (y / 16) * columns + x / 16;
            let [id, x, y] = [id, x % 16, y % 16].map(|v| u8::try_from(v).unwrap());
            rgba.extend([id, x, y, 255]);
        }
    }
    Sheet {
        tsx: parse_tsx(&TSX.replace("columns=\"4\"", &format!("columns=\"{columns}\""))).unwrap(),
        size: (columns * 16, 48),
        rgba,
    }
}

#[test]
fn a_blocks_tile_is_joined_from_the_quarters_for_where_the_terrain_ends() {
    let sheet = block_sheet(4);
    // The block from tile 1: tiles 1 2 3 / 5 6 7 / 9 10 11. Each quarter
    // of the joined tile is the same quarter of the tile named: top-left,
    // top-right, bottom-left, bottom-right.
    let quarters = |ends: u8| {
        let tile = joined(&sheet, 1, ends).unwrap();
        assert_eq!(tile.len(), 16 * 16 * 4);
        let mut out = Vec::new();
        for (left, top) in [(0, 0), (8, 0), (0, 8), (8, 8)] {
            let ids: BTreeSet<u8> = (top..top + 8)
                .flat_map(|y| (left..left + 8).map(move |x| (x, y)))
                .map(|(x, y)| {
                    let at = (y * 16 + x) * 4;
                    let px = &tile[at..at + 4];
                    assert_eq!(
                        px[1..],
                        [u8::try_from(x).unwrap(), u8::try_from(y).unwrap(), 255]
                    );
                    px[0]
                })
                .collect();
            assert_eq!(ids.len(), 1, "{ends:04b}: a quarter of one tile");
            out.push(*ids.first().unwrap());
        }
        out
    };
    // Ending nowhere: the middle; everywhere: the corners.
    assert_eq!(quarters(0b0000), [6, 6, 6, 6]);
    assert_eq!(quarters(0b1111), [1, 3, 9, 11]);
    // Up, right, down, left.
    assert_eq!(quarters(0b1000), [2, 2, 6, 6]);
    assert_eq!(quarters(0b0100), [6, 7, 6, 7]);
    assert_eq!(quarters(0b0010), [6, 6, 10, 10]);
    assert_eq!(quarters(0b0001), [5, 6, 5, 6]);
    // A strip across, and the end of one.
    assert_eq!(quarters(0b1010), [2, 2, 10, 10]);
    assert_eq!(quarters(0b1110), [2, 3, 10, 11]);
    // A block that runs off the sheet, at the side or the bottom.
    assert_eq!(
        joined(&sheet, 2, 0).unwrap_err(),
        "the block from tile 2 runs past the image's right edge"
    );
    assert!(joined(&sheet, 1, 0).is_ok() && joined(&sheet, 4, 0).is_ok());
    assert!(joined(&sheet, 5, 0).is_ok());
    // Five tiles across: a block fits from the third, not the fourth.
    let wide = block_sheet(5);
    assert!(joined(&wide, 2, 0).is_ok());
    assert!(joined(&wide, 3, 0).is_err());
    assert_eq!(
        joined(&sheet, 5, 0b0010).unwrap_err(),
        "tile 14 is outside the 64×48 px image"
    );
}

#[test]
fn a_block_layer_is_a_picture_on_each_tile_for_each_way_its_terrain_ends() {
    let temp = Temp::new("block").with_sheet("lib");
    let sheet = block_sheet(4);
    let png = encode_png(64, 48, &sheet.rgba).unwrap();
    temp.write(
        "lib/tiny-tales/tilesets/set/Big.tsx",
        TSX.replace("made.png", "big.png").as_bytes(),
    );
    temp.write("lib/tiny-tales/tilesets/set/Images/big.png", &png);
    let layers = r#""wood": Block(of: ["forest", "thicket"], sheet: "set/Big", block: 1),
                    "fire": Block(of: ["burning"], sheet: "set/Big", block: 1, tint: "fire"),"#;
    let looks = r#""outdoor": (layers: ["wood", "fire"])"#;
    let map = mapping(layers, looks);
    let terrains = terrains();
    assert_eq!(check(&map, &terrains), Ok(()));
    let imported = import(&temp.0.join("lib"), &map, &terrains, &fire).unwrap();
    // Tile 1 for every terrain and 6 for water, then sixteen joined
    // tiles, then sixteen more in the fire colour.
    assert_eq!(imported.image.1, 48);
    let LayerText::Tiles {
        of,
        at,
        beside,
        sides,
    } = &imported.own.layers[0]
    else {
        panic!("{:?}", imported.own.layers);
    };
    assert_eq!(of, &["forest", "thicket"]);
    assert_eq!(*at, (2, 0));
    // Every other terrain ends it.
    let others: Vec<&String> = terrains
        .iter()
        .filter(|id| *id != "forest" && *id != "thicket")
        .collect();
    assert_eq!(beside.iter().collect::<Vec<_>>(), others);
    let mixes: Vec<u8> = sides.iter().map(|&(mix, _)| mix).collect();
    assert_eq!(mixes, (1..16).collect::<Vec<u8>>());
    assert_eq!(sides[14], (15, (1, 1)));
    let LayerText::Tiles { at, sides, .. } = &imported.own.layers[1] else {
        panic!("{:?}", imported.own.layers);
    };
    assert_eq!((*at, sides[14].1), ((2, 1), (1, 2)));
    // The joined tile of a wood alone is the block's corners.
    let (width, _, rgba) = &imported.image;
    let px = |x: u32, y: u32| rgba[usize::try_from((y * width + x) * 4).unwrap()];
    assert_eq!(
        [px(16, 16), px(31, 16), px(16, 31), px(31, 31)],
        [1, 3, 9, 11]
    );
    // The list names the block.
    let text = list(&map);
    assert!(
        text.contains("layer wood       on forest, thicket: set/Big block from tile 1\n"),
        "{text}"
    );
    // A terrain that isn't ours.
    let bad = mapping(&layers.replace("\"thicket\"", "\"moss\""), looks);
    assert_eq!(
        check(&bad, &terrains).unwrap_err(),
        "layer \"wood\": terrain \"moss\" is not in terrain.ron"
    );
}

/// [`TSX`] with these loops (ticket 1315): the water's tile 6 between 5
/// and itself (written from 5), the fort's tile 3 with 4, tile 1 with
/// itself (one picture: no motion) and tile 0 between 5 and 6 (never
/// itself).
fn animated_tsx() -> String {
    let anim = |id: u32, rest: &str, frames: &[(u32, u32)]| {
        let mut written = String::new();
        for (tile, ms) in frames {
            let _ = write!(written, "<frame tileid=\"{tile}\" duration=\"{ms}\"/>");
        }
        let frames = written;
        format!("<tile id=\"{id}\"{rest}><animation>{frames}</animation></tile>")
    };
    TSX.replace("<tile id=\"6\"/>", &anim(6, "", &[(5, 300), (6, 100)]))
        .replace(
            "<tile id=\"3\" terrain=\"0,,,\"/>",
            &anim(3, " terrain=\"0,,,\"", &[(3, 100), (4, 120)]),
        )
        .replace(
            "<tile id=\"1\" terrain=\"0,0,0,0\"/>",
            &anim(1, " terrain=\"0,0,0,0\"", &[(1, 100), (1, 100)]),
        )
        .replace(
            "<tile id=\"0\" terrain=\",,,0\"/>",
            &anim(0, " terrain=\",,,0\"", &[(5, 100), (6, 100)]),
        )
}

#[test]
fn a_tsx_gives_each_tiles_animation() {
    assert_eq!(
        parse_tsx(TSX).unwrap().animations,
        BTreeMap::from([(2, vec![(2, 150)])])
    );
    let tsx = parse_tsx(&animated_tsx()).unwrap();
    assert_eq!(tsx.animations[&6], [(5, 300), (6, 100)]);
    assert_eq!(tsx.animations.len(), 5);
    let stray = TSX.replace(
        "<tile id=\"0\"",
        "<frame tileid=\"1\" duration=\"5\"/><tile id=\"0\"",
    );
    assert_eq!(parse_tsx(&stray).unwrap_err(), "a <frame> outside a <tile>");
    let bad = TSX.replace("duration=\"150\"", "duration=\"soon\"");
    assert_eq!(
        parse_tsx(&bad).unwrap_err(),
        "tile 2: a <frame> is not read"
    );
}

#[test]
fn an_import_packs_the_frames_of_each_tile_that_moves() {
    let temp = Temp::new("frames").with_sheet("lib");
    temp.write(
        "lib/tiny-tales/tilesets/set/Made.tsx",
        animated_tsx().as_bytes(),
    );
    let map = mapping(LAYERS, LOOKS);
    let imported = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap();
    // The looks' six cells as before (tile 1, 6, 0, 3, then 0 and 3 in
    // fire), then the frames: tile 5, tile 4 and tile 4 in fire.
    let (width, height, rgba) = &imported.image;
    assert_eq!((*width, *height), (256, 16));
    let px = |cell: usize| {
        let at = (5 * 256 + cell * 16 + 5) * 4;
        [rgba[at], rgba[at + 1], rgba[at + 2], rgba[at + 3]]
    };
    assert_eq!((px(6), px(7)), (color(5), color(4)));
    let mut tinted = color(4);
    tint(&mut tinted, [200, 100, 0]);
    assert_eq!(px(8), tinted);
    let todo = || Some(MINIMAL_TODO.to_owned());
    assert_eq!(
        imported.moving,
        [
            // Turned to start on the water's own tile.
            FramesText {
                at: (1, 0),
                frames: vec![((1, 0), 100), ((6, 0), 300)],
                minimal_todo: todo(),
            },
            FramesText {
                at: (3, 0),
                frames: vec![((3, 0), 100), ((7, 0), 120)],
                minimal_todo: todo(),
            },
            FramesText {
                at: (5, 0),
                frames: vec![((5, 0), 100), ((8, 0), 120)],
                minimal_todo: todo(),
            },
        ]
    );
    // The game's loader takes them: each moving cell is drawn by a row.
    let text = tileset(&imported, &UNITS);
    assert!(
        text.contains(
            "    frames: [\n        (at: (1, 0), frames: [((1, 0), 100), ((6, 0), 300)],\n"
        )
    );
    let png = encode_png(*width, *height, rgba).unwrap();
    let mut sheet = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    sheet.extend(48u32.to_be_bytes());
    sheet.extend(80u32.to_be_bytes());
    let mut horse = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    horse.extend(72u32.to_be_bytes());
    horse.extend(96u32.to_be_bytes());
    let files = [
        ("tilesets/tiny_tales.png", png.as_slice()),
        ("units/a.png", sheet.as_slice()),
        ("units/horse.png", horse.as_slice()),
    ];
    let content = trpg_content::load_embedded().unwrap();
    let refs = TilesetRefs {
        images: &trpg_content::image::from_files(files).unwrap(),
        terrain: &content.terrain.display,
        classes: &content.classes,
        characters: &content.characters,
    };
    let read = parse_tileset("tiny_tales.ron", "tiny_tales", &text, &refs).unwrap();
    let ats: Vec<(u32, u32)> = read.frames.iter().map(|t| t.at).collect();
    assert_eq!(ats, [(1, 0), (3, 0), (5, 0)]);
    let report = trpg_content::motion::check(trpg_content::tileset::moving_rows([&read]));
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    assert_eq!(report.todo.len(), 3);
}

/// A plain sheet (ticket 1318) of 6 × 6 tiles: each tile one colour by
/// its number ([`plain_color`]), as `pack/plain.png` of the packs in `lib`.
const PLAIN: &str = "pack/plain.png";

/// The colour of tile `id` of the plain sheet.
fn plain_color(id: u32) -> [u8; 4] {
    [u8::try_from(7 * id).unwrap(), 50, 50, 255]
}

/// Writes the plain sheet.
fn with_plain(temp: &Temp) {
    let mut rgba = Vec::new();
    for y in 0..96u32 {
        for x in 0..96u32 {
            rgba.extend(plain_color((y / 16) * 6 + x / 16));
        }
    }
    temp.write(&format!("lib/{PLAIN}"), &encode_png(96, 96, &rgba).unwrap());
}

/// The colour of the packed image's cell `at`, at its pixel (5, 5).
fn cell_color(image: &(u32, u32, Vec<u8>), (column, row): At) -> [u8; 4] {
    let (width, _, rgba) = image;
    let (x, y) = (column * 16 + 5, row * 16 + 5);
    let at = usize::try_from((y * width + x) * 4).unwrap();
    rgba[at..at + 4].try_into().unwrap()
}

/// A mapping of the made-up sheets with these `layers` and `looks`, and
/// the plain sheet moving by `strip` (unless it is empty).
fn plain_mapping(layers: &str, looks: &str, strip: &str) -> Mapping {
    let text = mapping_text(layers, looks);
    let text = if strip.is_empty() {
        text
    } else {
        let open = text.trim_end_matches(')');
        format!("{open}, moving: {{ \"{PLAIN}\": {strip} }})")
    };
    ron::from_str(&text).unwrap()
}

#[test]
fn a_plain_sheets_tiles_are_cut_by_their_place() {
    let temp = Temp::new("plain");
    with_plain(&temp);
    let dir = temp.0.join("lib");
    let sheet = Sheet::read(&dir, PLAIN).unwrap();
    assert_eq!((sheet.size, sheet.tsx.columns), ((96, 96), 6));
    assert!(
        sheet
            .tile(7)
            .unwrap()
            .chunks(4)
            .all(|p| p == plain_color(7))
    );
    assert_eq!(
        sheet.tile(36).unwrap_err(),
        "tile 36 is outside the 96×96 px image"
    );
    // Nothing to read, an image too small for a tile, not an image.
    let err = Sheet::read(&dir, "pack/none.png").unwrap_err();
    assert!(
        err.starts_with("reading ") && err.contains("none.png"),
        "{err}"
    );
    let small = encode_png(8, 32, &[0; 8 * 32 * 4]).unwrap();
    temp.write("lib/pack/small.png", &small);
    let err = Sheet::read(&dir, "pack/small.png").unwrap_err();
    assert!(
        err.ends_with("a 8×32 px image holds no 16×16 tile"),
        "{err}"
    );
    // One tile is enough.
    let one = encode_png(16, 16, &[7; 16 * 16 * 4]).unwrap();
    temp.write("lib/pack/one.png", &one);
    let sheet = Sheet::read(&dir, "pack/one.png").unwrap();
    assert_eq!((sheet.size, sheet.tsx.columns), ((16, 16), 1));
    temp.write("lib/pack/text.png", b"not a png");
    let err = Sheet::read(&dir, "pack/text.png").unwrap_err();
    assert!(err.contains("text.png: "), "{err}");
}

#[test]
fn a_pair_is_its_upper_tile_over_its_lower_one() {
    let temp = Temp::new("pair");
    // Tile 0 all one colour; tile 1 see-through on its left half, a
    // fifth opaque at its top-right pixel; tile 2 a fifth opaque all over.
    let mut rgba = Vec::new();
    for y in 0..16u32 {
        for x in 0..48u32 {
            rgba.extend(match (x, y) {
                (0..16, _) => [100, 0, 0, 255],
                (16..24, _) => [9, 9, 9, 0],
                (31, 0) | (32.., _) => [0, 0, 200, 51],
                _ => [0, 0, 200, 255],
            });
        }
    }
    temp.write("lib/pack/pair.png", &encode_png(48, 16, &rgba).unwrap());
    let sheet = Sheet::read(&temp.0.join("lib"), "pack/pair.png").unwrap();
    let pixels = paired(&sheet, 0, 1).unwrap();
    let px = |x: usize, y: usize| pixels[(y * 16 + x) * 4..(y * 16 + x) * 4 + 4].to_vec();
    assert_eq!(
        (px(0, 0), px(7, 15)),
        (vec![100, 0, 0, 255], vec![100, 0, 0, 255])
    );
    assert_eq!(px(8, 15), [0, 0, 200, 255]);
    // A fifth of the way from the lower colour to the upper one.
    assert_eq!(px(15, 0), [80, 0, 40, 255]);
    // Over nothing, a pixel is what it is: the clear half stays clear,
    // and a part see-through one keeps its colour and its alpha.
    assert_eq!(paired(&sheet, 1, 1).unwrap()[..4], [0; 4]);
    assert_eq!(paired(&sheet, 1, 2).unwrap()[..4], [0, 0, 200, 51]);
    assert!(paired(&sheet, 0, 3).is_err());
}

#[test]
fn a_cuts_frames_are_the_same_picture_further_along() {
    assert_eq!(Cut::Tile(4).along(2), Cut::Tile(6));
    let pair = Cut::Pair { under: 14, over: 0 };
    assert_eq!(pair.along(1), Cut::Pair { under: 15, over: 1 });
    let block = Cut::Joined { block: 3, ends: 5 };
    assert_eq!(block.along(3), Cut::Joined { block: 6, ends: 5 });
    // The column furthest right, in a sheet six tiles across: a frame
    // runs past the edge from there, whichever tile has the lower number.
    assert_eq!(Cut::Tile(10).right_column(6), 4);
    assert_eq!(pair.right_column(6), 2);
    assert_eq!(Cut::Pair { under: 12, over: 2 }.right_column(6), 2);
    assert_eq!(Cut::Pair { under: 2, over: 12 }.right_column(6), 2);
    assert_eq!(Cut::Pair { under: 6, over: 6 }.right_column(6), 0);
    assert_eq!(block.right_column(6), 5);
}

#[test]
fn a_patch_layer_takes_each_mix_from_the_usual_layout() {
    let temp = Temp::new("patch").with_sheet("lib");
    with_plain(&temp);
    let layers = format!(
        "{LAYERS} \"ground\": Patch(of: [\"plain\"], sheet: \"{PLAIN}\", patch: 0, inside: 3),
         \"dry\": Patch(of: [\"road\"], sheet: \"{PLAIN}\", patch: 0, inside: 3, full: false),"
    );
    let looks = r#""outdoor": (layers: ["ground"]), "indoor": (layers: ["dry"]),"#;
    let map = plain_mapping(&layers, looks, "");
    let imported = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap();
    let LayerText::Corners { of, tiles, .. } = &imported.own.layers[0] else {
        panic!("{:?}", imported.own.layers);
    };
    assert_eq!(of, &["plain"]);
    // Each mix (up-left, up-right, down-left, down-right) and the tile
    // it shows: the patch's (0, 1, 2, 6, 7, 8, 12, 13, 14), the inside
    // corners' (3, 5, 15, 17), and on the diagonals the patch's outside
    // corner drawn last, which covers the other (the tiles are solid).
    let expected = [
        ([0, 0, 0, 1], 0),
        ([0, 0, 1, 1], 1),
        ([0, 0, 1, 0], 2),
        ([0, 1, 0, 1], 6),
        ([1, 1, 1, 1], 7),
        ([1, 0, 1, 0], 8),
        ([0, 1, 0, 0], 12),
        ([1, 1, 0, 0], 13),
        ([1, 0, 0, 0], 14),
        ([1, 1, 1, 0], 3),
        ([1, 1, 0, 1], 5),
        ([1, 0, 1, 1], 15),
        ([0, 1, 1, 1], 17),
        ([1, 0, 0, 1], 0),
        ([0, 1, 1, 0], 2),
    ];
    assert_eq!(tiles.len(), 15);
    for (corners, tile) in expected {
        let mix = trpg_content::tileset::mix(corners.map(|c| c == 1));
        let at = tiles.iter().find(|(m, _)| *m == mix).unwrap().1;
        assert_eq!(
            cell_color(&imported.image, at),
            plain_color(tile),
            "{corners:?}"
        );
    }
    // Without the full tile: the other fourteen.
    let (_, indoor) = &imported.others[0];
    let LayerText::Corners { tiles, .. } = &indoor.layers[0] else {
        panic!("{:?}", indoor.layers);
    };
    assert_eq!(tiles.len(), 14);
    assert!(tiles.iter().all(|(mix, _)| *mix != 15));
    // A patch past the image's right edge.
    let layers = format!(
        "{LAYERS} \"ground\": Patch(of: [\"plain\"], sheet: \"{PLAIN}\", patch: 4, inside: 3),"
    );
    let map = plain_mapping(&layers, r#""outdoor": (layers: ["ground"]),"#, "");
    let err = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap_err();
    assert!(
        err.ends_with("the patch from tile 4 runs past the image's right edge"),
        "{err}"
    );
}

#[test]
fn a_plain_sheets_tiles_move_along_its_strip() {
    let temp = Temp::new("strip").with_sheet("lib");
    with_plain(&temp);
    // A patch on the sheet's left half: each frame three tiles on.
    let layers = format!(
        "{LAYERS} \"river\": Patch(of: [\"water\"], sheet: \"{PLAIN}\", patch: 0, inside: 18, full: false),"
    );
    let looks =
        format!(r#""outdoor": (terrain: {{ "water": ("{PLAIN}", 7) }}, layers: ["river"]),"#);
    let map = plain_mapping(&layers, &looks, "(frames: 2, step: 3, ms: 160)");
    let imported = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap();
    // The water's own tile and the river's fourteen pictures move; the
    // made-up Tiled sheet's tiles don't.
    assert_eq!(imported.moving.len(), 15);
    for moving in &imported.moving {
        let [(first, 160), (second, 160)] = moving.frames[..] else {
            panic!("{moving:?}");
        };
        assert_eq!(first, moving.at);
        let [red, ..] = cell_color(&imported.image, first);
        let [red_after, ..] = cell_color(&imported.image, second);
        // Three tiles on: seven more of red for each.
        assert_eq!(red_after, red + 21, "{moving:?}");
        assert_eq!(moving.minimal_todo.as_deref(), Some(MINIMAL_TODO));
    }
    let own = &imported.own.terrain;
    let water = own.iter().find(|(id, _)| id == "water").unwrap().1;
    assert!(imported.moving.iter().any(|m| m.at == water));
    // Frames that run past the right edge: the last frame is
    // (frames - 1) × step tiles on, from the picture's rightmost column.
    // Five on from the water's own tile (column 1) is past it; four on
    // from it fits, but not from the patch's right column (2).
    let cases = [
        ("(frames: 2, step: 5, ms: 160)", "Tile(7)"),
        ("(frames: 3, step: 2, ms: 160)", "Tile(2)"),
    ];
    for (strip, cut) in cases {
        let map = plain_mapping(&layers, &looks, strip);
        let err = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap_err();
        let past = format!("{PLAIN}: the frames of {cut} run past the image's right edge");
        assert_eq!(err, past, "{strip}");
    }
    // A step of whole rows runs down the column (1384): the water's own
    // tile (7) shows 19, then 31; a fourth frame (43) is below the image.
    let looks_down = format!(r#""outdoor": (terrain: {{ "water": ("{PLAIN}", 7) }}, layers: []),"#);
    let map = plain_mapping("", &looks_down, "(frames: 3, step: 12, ms: 150)");
    let imported = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap();
    let [moving] = &imported.moving[..] else {
        panic!("{:?}", imported.moving);
    };
    let colors: Vec<_> = (moving.frames.iter())
        .map(|&(at, ms)| (cell_color(&imported.image, at), ms))
        .collect();
    let expected = [7, 19, 31].map(|id| (plain_color(id), 150));
    assert_eq!(colors, expected);
    let map = plain_mapping("", &looks_down, "(frames: 4, step: 12, ms: 150)");
    let err = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap_err();
    assert!(
        err.contains("tile 43 is outside the 96×96 px image (frame 3"),
        "{err}"
    );
    // A loop whose frames are all one picture doesn't move.
    let mut flat = Vec::new();
    for _ in 0..16 * 64 {
        flat.extend([40, 60, 80, 255]);
    }
    temp.write("lib/pack/flat.png", &encode_png(64, 16, &flat).unwrap());
    // Its frames are packed after the patch's fifteen pictures, on the
    // image's second row: each is compared with the frame it is.
    let looks = r#""outdoor": (terrain: { "water": ("pack/flat.png", 0) }, layers: ["ground"]),"#;
    let ground =
        format!("\"ground\": Patch(of: [\"plain\"], sheet: \"{PLAIN}\", patch: 0, inside: 3),");
    let text = mapping_text(&ground, looks);
    let open = text.trim_end_matches(')');
    let text = format!("{open}, moving: {{ \"pack/flat.png\": (frames: 4, step: 1, ms: 9) }})");
    let map: Mapping = ron::from_str(&text).unwrap();
    let imported = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap();
    assert_eq!(imported.moving, []);
    assert_eq!(imported.image.1, 32);
}

#[test]
fn a_strip_and_a_patch_are_checked_with_the_mapping() {
    let terrains = terrains();
    let looks = r#""outdoor": (layers: []),"#;
    let checked = |strip: &str| check(&plain_mapping("", looks, strip), &terrains);
    assert_eq!(checked("(frames: 2, step: 1, ms: 1)"), Ok(()));
    let bad = [
        "(frames: 1, step: 1, ms: 1)",
        "(frames: 2, step: 0, ms: 1)",
        "(frames: 2, step: 1, ms: 0)",
    ];
    for strip in bad {
        assert_eq!(
            checked(strip).unwrap_err(),
            format!("moving \"{PLAIN}\": it needs 2 frames or more, a step and a time")
        );
    }
    let text = mapping_text("", looks);
    let open = text.trim_end_matches(')');
    let text = format!("{open}, moving: {{ \"set/Made\": (frames: 2, step: 1, ms: 1) }})");
    let map: Mapping = ron::from_str(&text).unwrap();
    assert_eq!(
        check(&map, &terrains).unwrap_err(),
        "moving \"set/Made\": only a plain sheet (….png) moves this way"
    );
    // A patch's terrains are ours.
    let layers = format!("\"g\": Patch(of: [\"lava\"], sheet: \"{PLAIN}\", patch: 0, inside: 3),");
    let map = plain_mapping(&layers, r#""outdoor": (layers: ["g"]),"#, "");
    assert_eq!(
        check(&map, &terrains).unwrap_err(),
        "layer \"g\": terrain \"lava\" is not in terrain.ron"
    );
    // The list names its sheet and patch.
    let text = list(&map);
    let line = format!("between tiles of lava: {PLAIN} patch from tile 0");
    assert!(text.contains(&line), "{text}");
}

/// A kit whose cells are numbered by what they are: outside corners 1–4,
/// edges across 5–6, edges down 7–8, inside corners 9–12.
const KIT: Kit = Kit {
    outer: [1, 2, 3, 4],
    across: [5, 6],
    down: [7, 8],
    inside: [9, 10, 11, 12],
};

/// Whether the tile at `(x, y)` of `rows` is in the layer (`#`); a place
/// off `rows` is not.
fn is_in(rows: &[&str], x: i32, y: i32) -> bool {
    let row = usize::try_from(y).ok().and_then(|y| rows.get(y));
    let at = usize::try_from(x).ok();
    row.zip(at).and_then(|(r, x)| r.as_bytes().get(x)) == Some(&b'#')
}

/// The cell of quarter `(qx, qy)` (0 left or up, 1 right or down) of the
/// tile at `(x, y)` of `rows`, as 0096's mockups chose it: by the tile's
/// neighbour across, the one up or down, and the diagonal between them.
fn by_tile(rows: &[&str], (x, y): (i32, i32), (qx, qy): (usize, usize)) -> Option<u32> {
    let is = |x, y| is_in(rows, x, y);
    if !is(x, y) {
        return None;
    }
    let (dx, dy) = (if qx == 0 { -1 } else { 1 }, if qy == 0 { -1 } else { 1 });
    let quarter = qy * 2 + qx;
    match (is(x + dx, y), is(x, y + dy), is(x + dx, y + dy)) {
        (false, false, _) => Some(KIT.outer[quarter]),
        (true, false, _) => Some(KIT.across[qy]),
        (false, true, _) => Some(KIT.down[qx]),
        (true, true, false) => Some(KIT.inside[quarter]),
        (true, true, true) => None,
    }
}

#[test]
fn a_quarters_picture_is_each_tiles_quarter_by_its_neighbours() {
    // Some mixes: (up-left, up-right, down-left, down-right) in the
    // layer, and the cell of each of the picture's four quarters.
    let expected: [([u8; 4], [Option<u32>; 4]); 4] = [
        // A lone tile down-right: its up-left quarter, an outside corner.
        ([0, 0, 0, 1], [None, None, None, Some(1)]),
        // Two below: the top edge.
        ([0, 0, 1, 1], [None, None, Some(5), Some(5)]),
        // Three: the one down-right's quarter is an inside corner, the
        // others edges.
        ([0, 1, 1, 1], [None, Some(7), Some(5), Some(9)]),
        // Two diagonal: two outside corners.
        ([1, 0, 0, 1], [Some(4), None, None, Some(1)]),
    ];
    for (places, cells) in expected {
        assert_eq!(KIT.quarters(places.map(|p| p == 1)), cells, "{places:?}");
    }
    assert_eq!(KIT.quarters([false; 4]), [None; 4]);
    assert_eq!(KIT.quarters([true; 4]), [None; 4]);
    // On a plus, an L (their inside corners), a ring, an X and a blob, every
    // quarter of every tile is the mockups' choice: the picture of the
    // point at that quarter's corner shows it.
    let shapes: [&[&str]; 5] = [
        &[".....", "..#..", ".###.", "..#..", "....."],
        &[".....", ".#...", ".#...", ".###.", "....."],
        &[".....", ".###.", ".#.#.", ".###.", "....."],
        &[".....", ".#.#.", "..#..", ".#.#.", "....."],
        &[".....", ".##..", ".###.", "..##.", "....."],
    ];
    let mut mixes = BTreeSet::new();
    for rows in shapes {
        for (x, y) in (0..5).flat_map(|y| (0..5).map(move |x| (x, y))) {
            for (qx, qy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                // The point at the quarter's corner, and the tiles round it.
                let (px, py) = (x + i32::from(qx == 1), y + i32::from(qy == 1));
                let round = [(px - 1, py - 1), (px, py - 1), (px - 1, py), (px, py)];
                let places = round.map(|(x, y)| is_in(rows, x, y));
                mixes.insert(trpg_content::tileset::mix(places));
                // The tile's place round the point is opposite its quarter.
                let shown = KIT.quarters(places)[(qy * 2 + qx) ^ 3];
                let wanted = by_tile(rows, (x, y), (qx, qy));
                assert_eq!(shown, wanted, "{rows:?} ({x}, {y}) quarter ({qx}, {qy})");
            }
        }
    }
    assert_eq!(mixes.len(), 16, "the shapes have every mix: {mixes:?}");
}

#[test]
fn a_quartered_picture_takes_the_quarter_of_each_cell_nearest_its_middle() {
    let temp = Temp::new("quarters");
    // One tile, each quarter its own colour (up-left 0, up-right 1,
    // down-left 2, down-right 3), its green the row in the quarter.
    let shade = |q: u8, row: u8| [40 * q + 10, 20 + row, 30, 255];
    let mut rgba = Vec::new();
    for y in 0..16u8 {
        for x in 0..16u8 {
            rgba.extend(shade(2 * u8::from(y >= 8) + u8::from(x >= 8), y % 8));
        }
    }
    temp.write("lib/pack/q.png", &encode_png(16, 16, &rgba).unwrap());
    let sheet = Sheet::read(&temp.0.join("lib"), "pack/q.png").unwrap();
    let px = |p: &[u8], x: usize, y: usize| p[(y * 16 + x) * 4..(y * 16 + x) * 4 + 4].to_vec();
    let all = quartered(&sheet, [Some(0); 4]).unwrap();
    // Each quarter of the picture is the cell's opposite quarter, row by
    // row.
    assert_eq!(px(&all, 0, 0), shade(3, 0));
    assert_eq!(px(&all, 3, 5), shade(3, 5));
    assert_eq!(px(&all, 15, 0), shade(2, 0));
    assert_eq!(px(&all, 7, 15), shade(1, 7));
    assert_eq!(px(&all, 8, 8), shade(0, 0));
    assert_eq!(px(&all, 9, 12), shade(0, 4));
    // A quarter with no cell is clear.
    let some = quartered(&sheet, [None, Some(0), None, None]).unwrap();
    assert_eq!(px(&some, 3, 3), [0; 4]);
    assert_eq!(px(&some, 12, 3), shade(2, 3));
    assert_eq!(px(&some, 12, 12), [0; 4]);
    assert!(quartered(&sheet, [None, None, None, Some(1)]).is_err());
    // As a cut: its frames are each cell further along.
    let cut = Cut::Quarters([Some(1), None, Some(8), Some(3)]);
    assert_eq!(
        cut.along(2),
        Cut::Quarters([Some(3), None, Some(10), Some(5)])
    );
    assert_eq!(cut.right_column(6), 3);
    assert_eq!(Cut::Quarters([None; 4]).right_column(6), 0);
}

#[test]
fn a_recolour_changes_only_the_colours_it_names() {
    let mut pixels = vec![
        0, 0, 0, 255, // black: brown
        29, 43, 83, 128, // half see-through blue: brown, as see-through
        1, 2, 3, 255, // another colour: kept
        0, 0, 0, 0, // nothing: kept
    ];
    recolour(
        &mut pixels,
        &[((0, 0, 0), (64, 51, 37)), ((29, 43, 83), (106, 84, 61))],
    );
    assert_eq!(
        pixels,
        [64, 51, 37, 255, 106, 84, 61, 128, 1, 2, 3, 255, 0, 0, 0, 0]
    );
}

/// A quarters layer of `of` on the plain sheet, its cells numbered as
/// [`KIT`]'s.
fn rim(of: &str) -> String {
    format!(
        "\"rim\": Quarters(of: [{of}], sheet: \"{PLAIN}\", kit: (outer: (1, 2, 3, 4), \
         across: (5, 6), down: (7, 8), inside: (9, 10, 11, 12))),"
    )
}

#[test]
fn a_quarters_layer_is_packed_recoloured_and_listed() {
    let temp = Temp::new("quarters-layer").with_sheet("lib");
    with_plain(&temp);
    let layers = format!("{LAYERS} {}", rim("\"mountain\", \"peak\""));
    let text = mapping_text(&layers, r#""outdoor": (layers: ["rim"]),"#);
    let open = text.trim_end_matches(')');
    // Tile 4's colour (an up-left quarter's outside corner) recoloured.
    let [r, g, b, _] = plain_color(4);
    let text = format!("{open}, recolour: {{ \"{PLAIN}\": [(({r}, {g}, {b}), (1, 2, 3))] }})");
    let map: Mapping = ron::from_str(&text).unwrap();
    let imported = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap();
    let LayerText::Corners { of, tiles, .. } = &imported.own.layers[0] else {
        panic!("{:?}", imported.own.layers);
    };
    assert_eq!(of, &["mountain", "peak"]);
    // Every mix but all four (the tiles' own picture shows there).
    let mixes: Vec<u8> = tiles.iter().map(|(m, _)| *m).collect();
    assert_eq!(mixes, (1..15).collect::<Vec<_>>());
    // The up-left quarter of a point with only its up-left tile in the
    // layer: that tile's down-right outside corner, tile 4, recoloured.
    let lone = tiles.iter().find(|(m, _)| *m == 8).unwrap().1;
    assert_eq!(cell_color(&imported.image, lone), [1, 2, 3, 255]);
    // Two below (its lower half): the top edge, tile 5, as it is.
    let top = tiles.iter().find(|(m, _)| *m == 3).unwrap().1;
    let (width, _, rgba) = &imported.image;
    let at = usize::try_from(((top.1 * 16 + 12) * width + top.0 * 16 + 5) * 4).unwrap();
    assert_eq!(rgba[at..at + 4], plain_color(5));
    // The list names the kit and says it is recoloured.
    let listed = format!("between tiles of mountain, peak: {PLAIN} quarters, recoloured");
    assert!(list(&map).contains(&listed), "{}", list(&map));
    // A terrain of the layer must be ours.
    let map = plain_mapping(&rim("\"lava\""), r#""outdoor": (layers: ["rim"]),"#, "");
    assert_eq!(
        check(&map, &terrains()).unwrap_err(),
        "layer \"rim\": terrain \"lava\" is not in terrain.ron"
    );
}

/// The face of [`KIT`]'s layer: its left end 13, its middle 14, its right
/// end 15.
const FACE: Face = Face {
    left: 13,
    middle: 14,
    right: 15,
};

#[test]
fn a_face_hangs_below_the_layer_rounded_at_its_ends() {
    // Some mixes: (up-left, up-right, down-left, down-right) in the
    // layer, and the face on the down-left tile's right half and the
    // down-right's left half.
    let expected: [([u8; 4], [Option<u32>; 2]); 5] = [
        // The bottom edge: the face's middle on both.
        ([1, 1, 0, 0], [Some(14), Some(14)]),
        // The layer ends to the right of the tile above, or to the left.
        ([1, 0, 0, 0], [Some(15), None]),
        ([0, 1, 0, 0], [None, Some(13)]),
        // An inside corner: the layer goes on right of the tile above.
        ([1, 1, 0, 1], [Some(14), None]),
        // Diagonal: the tile below the one in the layer gets its end.
        ([1, 0, 0, 1], [Some(15), None]),
    ];
    for (places, face) in expected {
        assert_eq!(FACE.under(places.map(|p| p == 1)), face, "{places:?}");
    }
    assert_eq!(FACE.under([false; 4]), [None; 2]);
    assert_eq!(FACE.under([true; 4]), [None; 2]);
    // On every shape, each half of each tile has the face 0096 chose: on
    // a tile below one of the layer that isn't in it, the end where the
    // layer ends that way beside the tile above, else the middle.
    let shapes: [&[&str]; 5] = [
        &[".....", "..#..", ".###.", "..#..", "....."],
        &[".....", ".#...", ".#...", ".###.", "....."],
        &[".....", ".###.", ".#.#.", ".###.", "....."],
        &[".....", ".#.#.", "..#..", ".#.#.", "....."],
        &[".....", ".##..", ".###.", "..##.", "....."],
    ];
    for rows in shapes {
        let is = |x, y| is_in(rows, x, y);
        for (x, y) in (0..5).flat_map(|y| (0..5).map(move |x| (x, y))) {
            let hangs = is(x, y - 1) && !is(x, y);
            let end = |dx, end| hangs.then_some(if is(x + dx, y - 1) { 14 } else { end });
            // Its left half is the point at its top-left's (the tile is
            // down-right of it), its right half the top-right's.
            let point = |px: i32| {
                let round = [(px - 1, y - 1), (px, y - 1), (px - 1, y), (px, y)];
                FACE.under(round.map(|(x, y)| is(x, y)))
            };
            assert_eq!(point(x)[1], end(-1, 13), "{rows:?} ({x}, {y}) left");
            assert_eq!(point(x + 1)[0], end(1, 15), "{rows:?} ({x}, {y}) right");
        }
    }
}

#[test]
fn a_faced_picture_hangs_the_faces_halves_under_the_rim_and_its_shadow_is_its_shape() {
    let temp = Temp::new("faced");
    // Three cells, each pixel's red its cell and column, its green its row.
    let shade = |cell: u8, x: u8, y: u8| [40 * cell + x, 10 * y, 30, 255];
    let mut rgba = Vec::new();
    for y in 0..16u8 {
        for x in 0..48u8 {
            rgba.extend(shade(x / 16, x % 16, y));
        }
    }
    temp.write("lib/pack/f.png", &encode_png(48, 16, &rgba).unwrap());
    let sheet = Sheet::read(&temp.0.join("lib"), "pack/f.png").unwrap();
    let px = |p: &[u8], x: usize, y: usize| p[(y * 16 + x) * 4..(y * 16 + x) * 4 + 4].to_vec();
    let top = [Some(0), Some(0), None, None];
    let picture = faced(&sheet, top, [Some(1), Some(2)], None).unwrap();
    // A tile wide, half a tile taller than one.
    assert_eq!(picture.len(), 16 * 24 * 4);
    // Its top half the rim's quarters, as a quartered picture's.
    let rim = quartered(&sheet, top).unwrap();
    assert_eq!(picture[..16 * 8 * 4], rim[..16 * 8 * 4]);
    // From its middle down, the down-left tile's right half of the face
    // cell, the down-right's left half, row by row.
    assert_eq!(px(&picture, 2, 8), shade(1, 10, 0));
    assert_eq!(px(&picture, 7, 23), shade(1, 15, 15));
    assert_eq!(px(&picture, 12, 10), shade(2, 4, 2));
    assert_eq!(px(&picture, 8, 16), shade(2, 0, 8));
    // Its shadow: the same shape in the shadow's colour, nothing elsewhere.
    let colour = [12, 18, 8, 70];
    let one = [Some(0), None, None, None];
    let shadow = faced(&sheet, one, [Some(1), None], Some(colour)).unwrap();
    assert_eq!(px(&shadow, 3, 2), colour);
    assert_eq!(px(&shadow, 3, 20), colour);
    assert_eq!(px(&shadow, 12, 2), [0; 4]);
    assert_eq!(px(&shadow, 12, 20), [0; 4]);
    assert!(faced(&sheet, one, [None, Some(3)], None).is_err());
    // As a cut: two cells down, its frames each cell further along.
    let cut = Cut::Faced {
        quarters: one,
        face: [None, Some(2)],
        shadow: None,
    };
    let downs = (cut.down(), Cut::Quarters(one).down(), Cut::Tile(0).down());
    assert_eq!(downs, (2, 1, 1));
    let along = Cut::Faced {
        quarters: [Some(1), None, None, None],
        face: [None, Some(3)],
        shadow: None,
    };
    assert_eq!(cut.along(1), along);
    assert_eq!(cut.right_column(6), 2);
}

/// [`rim`] with [`FACE`] below it, skipping walls, and its shadow.
fn faced_rim() -> String {
    let rim = rim("\"mountain\", \"peak\"").replace(
        "))),",
        ")), face: (left: 13, middle: 14, right: 15), skip: [\"wall\"]),",
    );
    format!("{rim} \"shade\": Shadow(of: \"rim\", colour: (12, 18, 8, 70), offset: (1, 3)),")
}

/// The made-up sheets imported with [`faced_rim`]'s layers, the shadow,
/// the rim and the fort, in a folder named for `name`.
fn faced_import(name: &str) -> (Temp, Mapping, Imported) {
    let temp = Temp::new(name).with_sheet("lib");
    with_plain(&temp);
    let layers = format!("{LAYERS} {}", faced_rim());
    let look = r#""outdoor": (layers: ["shade", "rim", "fort"]),"#;
    let map = plain_mapping(&layers, look, "");
    let imported = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap();
    (temp, map, imported)
}

#[test]
fn a_faced_layer_and_its_shadow_are_packed_taller_than_a_tile() {
    let (_temp, map, imported) = faced_import("faced-layer");
    let [
        LayerText::Corners {
            tiles: shade,
            place: shade_place,
            ..
        },
        LayerText::Corners {
            of,
            tiles: rim,
            place,
        },
        LayerText::Tiles { at: fort, .. },
    ] = imported.own.layers.as_slice()
    else {
        panic!("{:?}", imported.own.layers);
    };
    assert_eq!(of, &["mountain", "peak"]);
    let tall = Place {
        size: Some((16, 24)),
        offset: (0, 0),
        skip: vec!["wall".to_owned()],
    };
    assert_eq!(place, &tall);
    // The shadow: the same pieces, moved a pixel right and three down.
    let moved = Place {
        offset: (1, 3),
        ..tall
    };
    assert_eq!(shade_place, &moved);
    // Every mix but all four has a piece, in both.
    let mixes = |tiles: &[(u8, At)]| tiles.iter().map(|(m, _)| *m).collect::<Vec<_>>();
    assert_eq!(mixes(rim), (1..15).collect::<Vec<_>>());
    assert_eq!(mixes(shade), mixes(rim));
    // Each piece has the cell below its own to itself: no picture starts
    // there, and none is packed twice.
    let pieces: Vec<At> = rim.iter().chain(shade).map(|(_, at)| *at).collect();
    let mut starts: BTreeSet<At> = imported.own.terrain.iter().map(|(_, at)| *at).collect();
    starts.insert(*fort);
    for at in &pieces {
        assert!(starts.insert(*at), "{at:?} packed twice");
    }
    for (column, row) in &pieces {
        assert!(!starts.contains(&(*column, row + 1)), "({column}, {row})");
    }
    // The face's lower half, by the layer's ends; the shadow's colour.
    let (width, _, rgba) = &imported.image;
    let pixel = |(column, row): At, x: u32, y: u32| {
        let at = usize::try_from(((row * 16 + y) * width + column * 16 + x) * 4).unwrap();
        rgba[at..at + 4].to_vec()
    };
    let of_mix = |tiles: &[(u8, At)], mix: u8| tiles.iter().find(|(m, _)| *m == mix).unwrap().1;
    let bottom = of_mix(rim, 12);
    assert_eq!(pixel(bottom, 3, 20), plain_color(14));
    assert_eq!(pixel(bottom, 12, 20), plain_color(14));
    assert_eq!(pixel(of_mix(rim, 8), 3, 20), plain_color(15));
    assert_eq!(pixel(of_mix(rim, 8), 12, 20), [0; 4]);
    assert_eq!(pixel(of_mix(rim, 4), 12, 20), plain_color(13));
    assert_eq!(pixel(of_mix(shade, 12), 3, 20), [12, 18, 8, 70]);
    assert_eq!(pixel(of_mix(shade, 12), 3, 5), [12, 18, 8, 70]);
    // The list names the face and the shadow.
    let listed = list(&map);
    assert!(
        listed.contains(", a face below from tiles 13, 14, 15"),
        "{listed}"
    );
    assert!(
        listed.contains("the shadow of layer rim, moved (1, 3)"),
        "{listed}"
    );
}

#[test]
fn the_game_reads_a_faced_layer_at_its_size_moved_and_skipping_walls() {
    let (_temp, _, imported) = faced_import("faced-read");
    let (width, height, rgba) = &imported.image;
    let png = encode_png(*width, *height, rgba).unwrap();
    let content = trpg_content::load_embedded().unwrap();
    let header = |w: u32, h: u32| {
        let mut sheet = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        sheet.extend(w.to_be_bytes());
        sheet.extend(h.to_be_bytes());
        sheet
    };
    let (sheet, horse) = (header(48, 80), header(72, 96));
    let files = [
        ("tilesets/tiny_tales.png", png.as_slice()),
        ("units/a.png", sheet.as_slice()),
        ("units/horse.png", horse.as_slice()),
    ];
    let refs = TilesetRefs {
        images: &trpg_content::image::from_files(files).unwrap(),
        terrain: &content.terrain.display,
        classes: &content.classes,
        characters: &content.characters,
    };
    let text = tileset(&imported, &UNITS);
    let read = parse_tileset("tiny_tales.ron", "tiny_tales", &text, &refs).unwrap();
    let look = &read.terrain.unwrap().look;
    let [Layer::Corners(shadow), Layer::Corners(face), _] = look.layers.as_slice() else {
        panic!("{:?}", look.layers);
    };
    let wall = content.terrain.display.id_of("wall").unwrap();
    assert_eq!((shadow.offset, face.offset), ((1, 3), (0, 0)));
    assert_eq!(face.skip, BTreeSet::from([wall]));
    assert_eq!(shadow.skip, face.skip);
    assert_eq!(face.tiles[12].map(|r| (r.w, r.h)), Some((16, 24)));
}

#[test]
fn a_shadow_is_of_a_faced_layer_and_what_it_skips_is_ours() {
    let look = r#""outdoor": (layers: ["shade"]),"#;
    let problem = |layers: &str| check(&plain_mapping(layers, look, ""), &terrains()).unwrap_err();
    let shadow = |of: &str| {
        format!("\"shade\": Shadow(of: \"{of}\", colour: (0, 0, 0, 9), offset: (0, 1)),")
    };
    let wanted = |of: &str| {
        format!(
            "layer \"shade\": a shadow is of a `Quarters` layer with a face, and \"{of}\" isn't one"
        )
    };
    let plain_rim = rim("\"mountain\"");
    assert_eq!(
        problem(&format!("{plain_rim} {}", shadow("rim"))),
        wanted("rim")
    );
    assert_eq!(
        problem(&format!("{LAYERS} {}", shadow("fort"))),
        wanted("fort")
    );
    assert_eq!(problem(&shadow("none")), wanted("none"));
    let lava = faced_rim().replace("\"wall\"", "\"lava\"");
    assert_eq!(
        problem(&lava),
        "layer \"rim\": terrain \"lava\" is not in terrain.ron"
    );
}

/// What a tile of a fort's rows is: `#` a wall, `:` a road (a gate).
fn wall_at(rows: &[&str], x: i32, y: i32) -> (bool, bool) {
    let row = usize::try_from(y).ok().and_then(|y| rows.get(y));
    let at = usize::try_from(x).ok();
    let c = row.zip(at).and_then(|(r, x)| r.as_bytes().get(x));
    (c == Some(&b'#'), c == Some(&b':'))
}

#[test]
fn a_walls_top_and_face_go_by_its_neighbours_on_a_ring_with_a_gate() {
    // A keep like Harl's (ticket 1360): a gate in its west wall and one in
    // its south wall.
    let ring: &[&str] = &[
        ".......", ".#####.", ".:...#.", ".#...#.", ".##:##.", ".......",
    ];
    // Its tops: `A` across, `V` up and down, `P` a pillar; and its faces,
    // `a` under a wall running across, `p` under a pillar, as 0096's
    // mockups drew them: on the tile below each wall that doesn't run up
    // and down, unless that tile is a wall or a gate.
    let tops: &[&str] = &[
        ".......", ".AAAAP.", ".....V.", ".P...V.", ".PA.AP.", ".......",
    ];
    let faces: &[&str] = &[
        ".......", ".......", "..aaa..", ".......", ".......", ".pa.ap.",
    ];
    let at = |rows: &[&str], x: i32, y: i32| {
        let (x, y) = (usize::try_from(x).unwrap(), usize::try_from(y).unwrap());
        rows[y].as_bytes()[x]
    };
    let wall = |x, y| wall_at(ring, x, y).0;
    for (x, y) in (0..6).flat_map(|y| (0..7).map(move |x| (x, y))) {
        let sides = [
            wall(x, y - 1),
            wall(x + 1, y),
            wall(x, y + 1),
            wall(x - 1, y),
        ];
        let top = wall(x, y).then(|| match WallTop::of(sides) {
            WallTop::Across => b'A',
            WallTop::UpDown => b'V',
            WallTop::Pillar => b'P',
        });
        assert_eq!(top.unwrap_or(b'.'), at(tops, x, y), "top ({x}, {y})");
        // The face on this tile hangs from the point at the top-left of
        // the tile above it (that tile down-right of the point), unless
        // this tile is skipped.
        let round = [(x - 1, y - 2), (x, y - 2), (x - 1, y - 1), (x, y - 1)];
        let (is_wall, is_road) = wall_at(ring, x, y);
        let hung = face(
            round.map(|(x, y)| wall(x, y)),
            u32::from(b'a'),
            u32::from(b'p'),
        );
        let shown = hung.filter(|_| !is_wall && !is_road);
        let shown = shown.map_or(b'.', |c| u8::try_from(c).unwrap());
        assert_eq!(shown, at(faces, x, y), "face ({x}, {y})");
    }
    // A wall alone is a pillar, but a point sees too little to tell it
    // from the end of a wall running across: it hangs that face.
    assert_eq!(WallTop::of([false; 4]), WallTop::Pillar);
    assert_eq!(face([false, false, false, true], 1, 2), Some(1));
    assert_eq!(face([true, true, true, false], 1, 2), None);
}

#[test]
fn a_mirrored_tile_is_its_left_half_twice_and_a_stack_is_two_tiles_tall() {
    let temp = Temp::new("walls");
    // Two cells, each pixel's red its cell and column, its green its row.
    let shade = |cell: u8, x: u8, y: u8| [40 * cell + x, 10 * y, 30, 255];
    let mut rgba = Vec::new();
    for y in 0..16u8 {
        for x in 0..32u8 {
            rgba.extend(shade(x / 16, x % 16, y));
        }
    }
    temp.write("lib/pack/w.png", &encode_png(32, 16, &rgba).unwrap());
    let sheet = Sheet::read(&temp.0.join("lib"), "pack/w.png").unwrap();
    let px = |p: &[u8], x: usize, y: usize| p[(y * 16 + x) * 4..(y * 16 + x) * 4 + 4].to_vec();
    let mirror = mirrored(&sheet, 1).unwrap();
    assert_eq!(mirror.len(), 16 * 16 * 4);
    assert_eq!(px(&mirror, 2, 3), shade(1, 2, 3));
    assert_eq!(px(&mirror, 13, 3), shade(1, 2, 3));
    assert_eq!(px(&mirror, 8, 15), shade(1, 7, 15));
    assert_eq!(px(&mirror, 7, 15), shade(1, 7, 15));
    let stack = stacked(&sheet, [1, 0]).unwrap();
    assert_eq!(stack.len(), 16 * 32 * 4);
    assert_eq!(px(&stack, 4, 5), shade(1, 4, 5));
    assert_eq!(px(&stack, 4, 21), shade(0, 4, 5));
    assert!(stacked(&sheet, [0, 2]).is_err());
    assert!(mirrored(&sheet, 2).is_err());
    assert_eq!(blank(), vec![0; 16 * 16 * 4]);
    // As cuts: a stack takes two cells down; frames are further along.
    let downs = (
        Cut::Stacked([0, 1]).down(),
        Cut::Mirrored(0).down(),
        Cut::Blank.down(),
    );
    assert_eq!(downs, (2, 1, 1));
    assert_eq!(Cut::Stacked([3, 9]).along(1), Cut::Stacked([4, 10]));
    assert_eq!(Cut::Mirrored(3).along(2), Cut::Mirrored(5));
    assert_eq!(Cut::Blank.along(2), Cut::Blank);
    assert_eq!(Cut::Stacked([3, 9]).right_column(6), 3);
    // Either cell may be the one furthest right.
    assert_eq!(Cut::Stacked([0, 11]).right_column(6), 5);
    assert_eq!(Cut::Stacked([11, 0]).right_column(6), 5);
    assert_eq!(Cut::Mirrored(10).right_column(6), 4);
    assert_eq!(Cut::Blank.right_column(6), 0);
}

/// The keep's layers (ticket 1360) on the plain sheet: tops from tiles 1
/// (across), 2 (whose left half runs up and down) and 3 (a pillar),
/// faces 4 (across) and 5 (a pillar), a tower of 6 over 7, and a fort's
/// stone 8 by a floor.
const KEEP: &str = r#"
    "top": Walls(of: ["wall"], sheet: "pack/plain.png", across: 1, side: 2, pillar: 3),
    "face": Faces(of: ["wall"], sheet: "pack/plain.png", across: 4, pillar: 5,
                  skip: ["wall", "road"]),
    "tower": Standing(of: ["fort"], sheet: "pack/plain.png", cells: (6, 7),
                      skip: ["wall", "floor"]),
    "stone": Tiles(of: ["fort"], beside: ["floor"], near: ("pack/plain.png", 8)),
"#;

/// The made-up sheets imported with [`KEEP`]'s layers, in a folder named
/// for `name`.
fn keep_import(name: &str) -> (Temp, Mapping, Imported) {
    let temp = Temp::new(name).with_sheet("lib");
    with_plain(&temp);
    let look = r#""outdoor": (layers: ["face", "top", "tower", "stone"]),"#;
    let map = plain_mapping(KEEP, look, "");
    let imported = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap();
    (temp, map, imported)
}

#[test]
fn the_keeps_layers_are_packed_as_tiles_and_as_pictures_from_points() {
    let (_temp, map, imported) = keep_import("keep");
    let [
        LayerText::Corners {
            of: face_of,
            tiles: faces,
            place: face_place,
        },
        LayerText::Tiles {
            of: top_of,
            at: pillar,
            beside,
            sides: tops,
        },
        LayerText::Corners {
            tiles: tower,
            place: tower_place,
            ..
        },
        LayerText::Tiles {
            at: nothing,
            beside: by,
            sides: stone,
            ..
        },
    ] = imported.own.layers.as_slice()
    else {
        panic!("{:?}", imported.own.layers);
    };
    let colour = |at: At| cell_color(&imported.image, at);
    let names = |n: &[&str]| n.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    // The tops: a pillar unless the sides say across or up and down.
    assert_eq!((top_of, beside), (&names(&["wall"]), &names(&["wall"])));
    assert_eq!(colour(*pillar), plain_color(3));
    let mixes = |tiles: &[(u8, At)]| tiles.iter().map(|(m, _)| *m).collect::<Vec<_>>();
    assert_eq!(mixes(tops), [1, 4, 5, 10]);
    assert_eq!(colour(tops[0].1), plain_color(1));
    assert_eq!(tops[0].1, tops[2].1);
    assert_eq!(colour(tops[3].1), plain_color(2));
    // The faces: by every point with a wall down-right of it, a tile down
    // from it, a pillar's where the wall goes on up.
    assert_eq!(face_of, &names(&["wall"]));
    let downright: Vec<u8> = (1..16).filter(|m| m & 1 == 1).collect();
    assert_eq!(mixes(faces), downright);
    for (mix, at) in faces {
        let wanted = if mix & 4 == 4 { 5 } else { 4 };
        assert_eq!(colour(*at), plain_color(wanted), "{mix}");
    }
    let hung = Place {
        size: None,
        offset: (8, 24),
        skip: names(&["wall", "road"]),
    };
    assert_eq!(face_place, &hung);
    // The tower: two tiles tall, standing on its tile, packed in two cells.
    assert_eq!(mixes(tower), downright);
    let standing = Place {
        size: Some((16, 32)),
        offset: (8, -8),
        skip: names(&["wall", "floor"]),
    };
    assert_eq!(tower_place, &standing);
    let (column, row) = tower[0].1;
    assert_eq!(colour((column, row)), plain_color(6));
    assert_eq!(colour((column, row + 1)), plain_color(7));
    // The stone: by any floor; nothing elsewhere.
    assert_eq!(by, &names(&["floor"]));
    assert_eq!(colour(*nothing), [0; 4]);
    assert_eq!(mixes(stone), (1..16).collect::<Vec<_>>());
    assert!(stone.iter().all(|(_, at)| colour(*at) == plain_color(8)));
    // The list names them.
    let listed = list(&map);
    for line in [
        "wall tops on wall: pack/plain.png",
        "wall faces below wall: pack/plain.png",
        "standing on fort: pack/plain.png tiles 6 over 7",
        "on fort: nothing, by them pack/plain.png tile 8",
    ] {
        assert!(listed.contains(line), "{line}\n{listed}");
    }
}

#[test]
fn the_game_reads_the_keeps_faces_and_tower_at_their_size_and_place() {
    let (_temp, _, imported) = keep_import("keep-read");
    let (width, height, rgba) = &imported.image;
    let png = encode_png(*width, *height, rgba).unwrap();
    let content = trpg_content::load_embedded().unwrap();
    let header = |w: u32, h: u32| {
        let mut sheet = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        sheet.extend(w.to_be_bytes());
        sheet.extend(h.to_be_bytes());
        sheet
    };
    let (sheet, horse) = (header(48, 80), header(72, 96));
    let files = [
        ("tilesets/tiny_tales.png", png.as_slice()),
        ("units/a.png", sheet.as_slice()),
        ("units/horse.png", horse.as_slice()),
    ];
    let refs = TilesetRefs {
        images: &trpg_content::image::from_files(files).unwrap(),
        terrain: &content.terrain.display,
        classes: &content.classes,
        characters: &content.characters,
    };
    let text = tileset(&imported, &UNITS);
    let read = parse_tileset("tiny_tales.ron", "tiny_tales", &text, &refs).unwrap();
    let look = &read.terrain.unwrap().look;
    let [
        Layer::Corners(face),
        Layer::Tiles(_),
        Layer::Corners(tower),
        Layer::Tiles(_),
    ] = look.layers.as_slice()
    else {
        panic!("{:?}", look.layers);
    };
    let id = |name| content.terrain.display.id_of(name).unwrap();
    assert_eq!((face.offset, tower.offset), ((8, 24), (8, -8)));
    assert_eq!(face.skip, BTreeSet::from([id("wall"), id("road")]));
    assert_eq!(tower.skip, BTreeSet::from([id("wall"), id("floor")]));
    assert_eq!(face.tiles[1].map(|r| (r.w, r.h)), Some((16, 16)));
    assert_eq!(tower.tiles[1].map(|r| (r.w, r.h)), Some((16, 32)));
    assert_eq!(tower.tiles[2], None);
}

#[test]
fn the_keeps_layers_name_only_our_terrains() {
    let look = r#""outdoor": (layers: ["top", "face", "tower", "stone"]),"#;
    let problem = |from: &str, to: &str| {
        let layers = KEEP.replacen(from, to, 1);
        check(&plain_mapping(&layers, look, ""), &terrains()).unwrap_err()
    };
    let lava = |layer: &str| format!("layer \"{layer}\": terrain \"lava\" is not in terrain.ron");
    assert_eq!(
        problem("[\"wall\"], sheet", "[\"lava\"], sheet"),
        lava("top")
    );
    assert_eq!(problem("\"wall\", \"road\"", "\"lava\""), lava("face"));
    assert_eq!(problem("\"wall\", \"floor\"", "\"lava\""), lava("tower"));
    assert_eq!(problem("[\"floor\"]", "[\"lava\"]"), lava("stone"));
}

/// The colour of frame `k` of the made-up tree sheet.
fn tree_color(k: u32) -> [u8; 4] {
    [u8::try_from(40 + 20 * k).unwrap(), 120, 30, 255]
}

/// Writes a made-up tree sheet, `lib/trees/t.png`: eight frames of 64 ×
/// 20 px, four across, each one colour; and `lib/trees/narrow.png`, 32 px
/// wide, narrower than a frame.
fn with_tree_sheet(temp: &Temp) {
    let mut rgba = Vec::new();
    for y in 0..40u32 {
        for x in 0..256u32 {
            rgba.extend(tree_color((y / 20) * 4 + x / 64));
        }
    }
    temp.write("lib/trees/t.png", &encode_png(256, 40, &rgba).unwrap());
    let narrow = vec![0; 32 * 40 * 4];
    temp.write(
        "lib/trees/narrow.png",
        &encode_png(32, 40, &narrow).unwrap(),
    );
}

/// The look `indoor` with `frames` frames of the made-up sheet `sheet` as
/// trees on the forest.
fn tree_looks(sheet: &str, frames: u32) -> String {
    format!(
        r#""outdoor": (layers: ["mud", "span", "fort"]),
           "indoor": (layers: ["edge"], trees: (of: ["forest"], sheets: ["trees/{sheet}"],
                      frame: (64, 20), frames: {frames}, ms: 160, foot: (32, 18))),"#
    )
}

#[test]
fn a_halved_picture_keeps_its_edge_crisp_and_its_shadow_soft() {
    let (o, c) = ([200, 100, 0, 255], [0, 0, 0, 0]);
    let shade = [0, 0, 0, 100];
    // Four blocks of two by two: all solid, two solid (the edge), one
    // solid, and a shadow's half.
    #[rustfmt::skip]
    let rows: [[[u8; 4]; 8]; 2] = [
        [o, o, o, c, o, c, shade, shade],
        [o, o, o, c, c, c, c, c],
    ];
    let pixels: Vec<u8> = rows.iter().flatten().flatten().copied().collect();
    let half = halved(&pixels, (8, 2));
    let at = |i: usize| <[u8; 4]>::try_from(&half[i * 4..i * 4 + 4]).unwrap();
    assert_eq!(half.len(), 4 * 4);
    assert_eq!(at(0), o);
    assert_eq!(at(1), o);
    // One solid pixel of four: its colour, a quarter as opaque.
    assert_eq!(at(2), [200, 100, 0, 63]);
    assert_eq!(at(3), [0, 0, 0, 50]);
    // One solid pixel below a shadow's: the two averaged, as clear as the
    // block on the whole.
    let low: Vec<u8> = [c, c, o, shade].into_iter().flatten().collect();
    assert_eq!(halved(&low, (2, 2)), [100, 50, 0, 88]);
    // A block nearly clear (one pixel at alpha 3) is clear.
    let faint: Vec<u8> = [[90, 90, 90, 3], c, c, c].into_iter().flatten().collect();
    assert_eq!(halved(&faint, (2, 2)), [0, 0, 0, 0]);
}

#[test]
fn a_looks_trees_are_packed_whole_and_halved_below_the_tiles() {
    let temp = Temp::new("trees").with_sheet("lib");
    with_tree_sheet(&temp);
    let map = mapping(LAYERS, &tree_looks("t.png", 8));
    let imported = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap();
    let (_, indoor) = &imported.others[0];
    let trees = indoor.trees.as_ref().expect("the look's trees");
    assert_eq!(trees.of, ["forest"]);
    let (big, half) = (&trees.big[0], &trees.half[0]);
    assert_eq!((big.size, big.foot), ((64, 20), (32, 18)));
    assert_eq!((half.size, half.foot), ((32, 10), (16, 9)));
    // Below every tile packed: four frames four cells wide fill a row of
    // the image, then the next four two cells down (a frame is 20 px
    // tall); then the eight halves, two cells wide, fill one row.
    let row = big.frames[0].0.1;
    assert!(row > 0);
    let cells = |kind: &tileset_ron::TreeKindText| -> Vec<At> {
        kind.frames.iter().map(|&(at, _)| at).collect()
    };
    let shelf =
        |top: u32, step: u32, n: u32| -> Vec<At> { (0..n).map(|k| (k * step, top)).collect() };
    let mut whole = shelf(row, 4, 4);
    whole.extend(shelf(row + 2, 4, 4));
    assert_eq!(cells(big), whole);
    assert_eq!(cells(half), shelf(row + 4, 2, 8));
    assert!(big.frames.iter().all(|&(_, ms)| ms == 160));
    // Each frame its picture, whole and halved.
    for (k, (&(whole, _), &(halved, _))) in (0..).zip(big.frames.iter().zip(&half.frames)) {
        assert_eq!(
            cell_color(&imported.image, whole),
            tree_color(k),
            "frame {k}"
        );
        assert_eq!(
            cell_color(&imported.image, halved),
            tree_color(k),
            "frame {k}"
        );
    }
    // The game reads the trees from the tileset file.
    let text = tileset(&imported, &UNITS);
    let (width, height, rgba) = &imported.image;
    let png = encode_png(*width, *height, rgba).unwrap();
    let header = |w: u32, h: u32| {
        let mut sheet = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        sheet.extend(w.to_be_bytes());
        sheet.extend(h.to_be_bytes());
        sheet
    };
    let (sheet, horse) = (header(48, 80), header(72, 96));
    let files = [
        ("tilesets/tiny_tales.png", png.as_slice()),
        ("units/a.png", sheet.as_slice()),
        ("units/horse.png", horse.as_slice()),
    ];
    let content = trpg_content::load_embedded().unwrap();
    let refs = TilesetRefs {
        images: &trpg_content::image::from_files(files).unwrap(),
        terrain: &content.terrain.display,
        classes: &content.classes,
        characters: &content.characters,
    };
    let read = parse_tileset("tiny_tales.ron", "tiny_tales", &text, &refs).unwrap();
    let trees = read.trees("indoor").expect("the game reads the trees");
    assert_eq!(
        (trees.big[0].frames.len(), trees.half[0].frames.len()),
        (8, 8)
    );
    let rows = trpg_content::tileset::moving_rows([&read]);
    let report = trpg_content::motion::check(rows);
    assert!(report.problems.is_empty(), "{:?}", report.problems);
    assert!(
        report.todo.iter().any(|t| t.contains("a one-bit tree")),
        "{:?}",
        report.todo
    );
}

#[test]
fn trees_whose_frames_dont_fit_their_sheet_are_refused() {
    let temp = Temp::new("tree-fit").with_sheet("lib");
    with_tree_sheet(&temp);
    let dir = temp.0.join("lib");
    let refused = |sheet: &str, frames: u32| {
        let map = mapping(LAYERS, &tree_looks(sheet, frames));
        import(&dir, &map, &terrains(), &fire).unwrap_err()
    };
    // Nine frames of 20 px need three rows of four: 60 px of a 40 px sheet.
    assert_eq!(
        refused("t.png", 9),
        "look \"indoor\": trees: trees/t.png: 9 frames of 64×20 px don't fit the 256×40 px image"
    );
    // A frame of an odd side doesn't halve; one of no side, or wider than
    // the packed image, can't be packed.
    for (w, h) in [(63, 20), (64, 21), (0, 20), (64, 0), (512, 20)] {
        let looks =
            tree_looks("t.png", 8).replace("frame: (64, 20)", &format!("frame: ({w}, {h})"));
        let map = mapping(LAYERS, &looks);
        assert_eq!(
            import(&dir, &map, &terrains(), &fire).unwrap_err(),
            format!("look \"indoor\": trees: a frame of {w}×{h} px doesn't halve or doesn't fit")
        );
    }
    // A frame as wide as the packed image fits.
    let widest = tree_looks("t.png", 2).replace("frame: (64, 20)", "frame: (256, 20)");
    assert!(import(&dir, &mapping(LAYERS, &widest), &terrains(), &fire).is_ok());
    // A sheet narrower than a frame holds none.
    assert_eq!(
        refused("narrow.png", 2),
        "look \"indoor\": trees: trees/narrow.png: 2 frames of 64×20 px don't fit the 32×40 px \
         image"
    );
}

#[test]
fn a_looks_trees_are_checked_with_the_mapping() {
    let looks = |trees: &str| {
        format!("\"outdoor\": (layers: []), \"indoor\": (layers: [], trees: {trees}),")
    };
    let check_of = |trees: &str| check(&mapping(LAYERS, &looks(trees)), &terrains());
    let tree = |of: &str, frames: u32| {
        format!(
            "(of: [\"{of}\"], sheets: [\"trees/t.png\"], frame: (32, 20), frames: {frames}, \
             ms: 160, foot: (16, 18))"
        )
    };
    assert_eq!(check_of(&tree("forest", 4)), Ok(()));
    assert!(check_of(&tree("woods", 4)).unwrap_err().contains("woods"));
    assert_eq!(
        check_of(&tree("forest", 1)),
        Err("look \"indoor\": trees need a sheet, 2 frames or more and a time".to_owned())
    );
}

/// A strip of 4 frames of 4 × 4 px side by side (ticket 0460), as
/// `pack/flames.png`: frame `f` all [`flame`]`(f)`, but frame 1's
/// top-left pixel see-through.
fn with_flames(temp: &Temp) {
    let mut rgba = Vec::new();
    for y in 0..4u32 {
        for x in 0..16u32 {
            let f = x / 4;
            rgba.extend(if (f, x % 4, y) == (1, 0, 0) {
                [0; 4]
            } else {
                flame(f)
            });
        }
    }
    temp.write("lib/pack/flames.png", &encode_png(16, 4, &rgba).unwrap());
}

/// The colour of frame `f` of the strip.
fn flame(f: u32) -> [u8; 4] {
    [u8::try_from(40 * f + 40).unwrap(), 60, 0, 255]
}

/// Two flames on a burning tile: one in the top-left corner from frame 0,
/// one in the bottom-right corner from frame 2.
const FLAMES: &str = r#"
    "fire": Sprites(of: ["burning"], sheet: "pack/flames.png", frame: (4, 4), frames: 4,
                    ms: 100, at: [(0, 0, 0), (12, 12, 2)]),
"#;

/// The colour of the packed image's pixel `(x, y)` of cell `at`.
fn pixel_of(image: &(u32, u32, Vec<u8>), (column, row): At, (x, y): (u32, u32)) -> [u8; 4] {
    let (width, _, rgba) = image;
    let at = usize::try_from(((row * 16 + y) * width + column * 16 + x) * 4).unwrap();
    rgba[at..at + 4].try_into().unwrap()
}

#[test]
fn sprites_lie_on_their_tile_each_on_its_own_frame_and_move() {
    let temp = Temp::new("sprites").with_sheet("lib");
    with_flames(&temp);
    let look = r#""outdoor": (layers: ["fire"]),"#;
    let map = plain_mapping(FLAMES, look, "");
    let imported = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap();
    let [
        LayerText::Tiles {
            of,
            at,
            beside,
            sides,
        },
    ] = imported.own.layers.as_slice()
    else {
        panic!("{:?}", imported.own.layers);
    };
    assert_eq!(
        (of.as_slice(), beside.is_empty()),
        (["burning".to_owned()].as_slice(), true)
    );
    assert_eq!(sides, &[]);
    let px = |cell, xy| pixel_of(&imported.image, cell, xy);
    // Frame 0: the first flame on frame 0, the second on frame 2, nothing
    // between them.
    assert_eq!((px(*at, (0, 0)), px(*at, (3, 3))), (flame(0), flame(0)));
    assert_eq!((px(*at, (12, 12)), px(*at, (15, 15))), (flame(2), flame(2)));
    assert_eq!((px(*at, (4, 4)), px(*at, (11, 11))), ([0; 4], [0; 4]));
    // It moves through the strip's 4 frames, 100 ms each, each flame a
    // frame further on, round the loop; a see-through pixel shows nothing.
    let [moving] = imported.moving.as_slice() else {
        panic!("{:?}", imported.moving);
    };
    assert_eq!(moving.at, *at);
    assert_eq!(moving.frames[0].0, *at);
    assert_eq!(
        moving.frames.iter().map(|f| f.1).collect::<Vec<_>>(),
        [100; 4]
    );
    for (k, &(cell, _)) in (0..).zip(&moving.frames) {
        let (first, second) = (k, (k + 2) % 4);
        assert_eq!(px(cell, (3, 3)), flame(first), "frame {k}");
        assert_eq!(px(cell, (15, 15)), flame(second), "frame {k}");
        let corner = if first == 1 { [0; 4] } else { flame(first) };
        assert_eq!(px(cell, (0, 0)), corner, "frame {k}");
    }
    // Each frame is a cell of its own.
    let cells: BTreeSet<At> = moving.frames.iter().map(|f| f.0).collect();
    assert_eq!(cells.len(), 4);
    assert!(list(&map).contains("moving on burning: pack/flames.png, 2 sprites of 4×4 px"));
    let cut = Cut::Sprites {
        layout: 3,
        frame: 1,
    };
    assert_eq!(
        cut.along(2),
        Cut::Sprites {
            layout: 3,
            frame: 3
        }
    );
    assert_eq!((cut.right_column(6), cut.down()), (0, 1));
}

#[test]
fn a_sprite_is_checked_whole_inside_its_tile_and_its_strip() {
    let sprites = |at: &str, frames: u32, sheet: &str| {
        format!(
            "\"fire\": Sprites(of: [\"burning\"], sheet: \"{sheet}\", frame: (4, 4), \
             frames: {frames}, ms: 100, at: [{at}]),"
        )
    };
    let check_of = |layers: &str| check(&mapping(layers, "\"outdoor\": (layers: [])"), &terrains());
    assert_eq!(check_of(&sprites("(12, 12, 3)", 4, "p/f.png")), Ok(()));
    for (at, frames, sheet, problem) in [
        (
            "(13, 0, 0)",
            4,
            "p/f.png",
            "a 4×4 px sprite at (13, 0) spills out of the 16×16 tile",
        ),
        (
            "(0, 13, 0)",
            4,
            "p/f.png",
            "a 4×4 px sprite at (0, 13) spills out of the 16×16 tile",
        ),
        (
            "(0, 0, 4)",
            4,
            "p/f.png",
            "the sprite at (0, 0) starts on frame 4 of 4",
        ),
        (
            "(0, 0, 0)",
            1,
            "p/f.png",
            "sprites need 2 frames or more, a size and a time",
        ),
        ("", 4, "p/f.png", "no sprite on the tile"),
        (
            "(0, 0, 0)",
            4,
            "set/Made",
            "sprites come from a plain sheet (….png)",
        ),
    ] {
        assert_eq!(
            check_of(&sprites(at, frames, sheet)),
            Err(format!("layer \"fire\": {problem}")),
            "{at}"
        );
    }
    // A strip with fewer frames than the layer says, found when imported.
    let temp = Temp::new("sprites-short").with_sheet("lib");
    with_flames(&temp);
    let map = plain_mapping(
        &sprites("(0, 0, 0)", 5, "pack/flames.png"),
        "\"outdoor\": (layers: [\"fire\"]),",
        "",
    );
    let err = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap_err();
    assert_eq!(
        err,
        "pack/flames.png: frame 4 of 4×4 px is outside the 16×4 px image"
    );
}

#[test]
fn a_standing_picture_may_be_moved_across_onto_the_tiles_beside() {
    let temp = Temp::new("shift").with_sheet("lib");
    with_plain(&temp);
    let layers = r#"
        "left": Standing(of: ["throne"], sheet: "pack/plain.png", cells: (0, 6), shift: -1),
        "right": Standing(of: ["throne"], sheet: "pack/plain.png", cells: (2, 8), shift: 1),
    "#;
    let map = plain_mapping(layers, r#""outdoor": (layers: ["left", "right"]),"#, "");
    let imported = import(&temp.0.join("lib"), &map, &terrains(), &fire).unwrap();
    let offsets: Vec<(i32, i32)> = imported
        .own
        .layers
        .iter()
        .map(|layer| match layer {
            LayerText::Corners { place, .. } => place.offset,
            LayerText::Tiles { .. } => panic!("{layer:?}"),
        })
        .collect();
    // A tile left of the tile above (8 - 16), a tile right of it.
    assert_eq!(offsets, [(-8, -8), (24, -8)]);
    let listed = list(&map);
    assert!(
        listed.contains("tiles 0 over 6, moved -1 tiles across"),
        "{listed}"
    );
    assert!(
        listed.contains("tiles 2 over 8, moved 1 tiles across"),
        "{listed}"
    );
}
