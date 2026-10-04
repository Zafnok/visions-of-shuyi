//! Tests of the importer, on a made-up Tiled sheet: never a bought one.

use std::path::PathBuf;

use trpg_content::ImageTable;
use trpg_content::tileset::{TilesetRefs, parse_tileset};

use super::*;
use crate::map_sprite_import::For;

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

    /// The made-up sheet as `set/Made` under `dir` of the folder.
    fn with_sheet(self, dir: &str) -> Self {
        self.write(&format!("{dir}/set/Made.tsx"), TSX.as_bytes());
        let png = encode_png(64, 32, &made_image()).unwrap();
        self.write(&format!("{dir}/set/Images/made.png"), &png);
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
    assert!(super::tags("no tags").is_empty());
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
    temp.write("lib/set/Bad.tsx", b"<map/>");
    let err = Sheet::read(&dir, "set/Bad").unwrap_err();
    assert!(err.ends_with("Bad.tsx: no <tileset>"), "{err}");
    temp.write(
        "lib/set/Big.tsx",
        TSX.replace("tilewidth=\"16\"", "tilewidth=\"8\"")
            .as_bytes(),
    );
    let err = Sheet::read(&dir, "set/Big").unwrap_err();
    assert!(
        err.ends_with("Big.tsx: its tiles are 8×16 px, not 16×16"),
        "{err}"
    );
    temp.write(
        "lib/set/Lost.tsx",
        TSX.replace("made.png", "lost.png").as_bytes(),
    );
    let err = Sheet::read(&dir, "set/Lost").unwrap_err();
    assert!(
        err.starts_with("reading ") && err.contains("lost.png"),
        "{err}"
    );
    temp.write("lib/set/Images/text.png", b"not a png");
    temp.write(
        "lib/set/Text.tsx",
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
        }]
    );
}

#[test]
fn more_tiles_than_a_row_holds_go_on_the_next_row() {
    let temp = Temp::new("rows").with_sheet("lib");
    let dir = temp.0.join("lib");
    let mut packer = Packer {
        dir: &dir,
        colors: &fire,
        sheets: BTreeMap::new(),
        cells: Vec::new(),
        pixels: Vec::new(),
    };
    // No picture yet: one empty row.
    assert_eq!(packer.image().0, 256);
    assert_eq!(packer.image().1, 16);
    // Seventeen different pictures: the same tiles, tinted or not, and
    // made different by name.
    for i in 0..17u32 {
        packer.cells.push((format!("filler {i}"), 0, String::new()));
        packer
            .pixels
            .push(vec![u8::try_from(i).unwrap(); 16 * 16 * 4]);
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

const UNITS: [MapSprite; 2] = [
    MapSprite {
        who: For::Class("exile"),
        name: "a",
        source: "heroes/A/map_sprite.png",
    },
    MapSprite {
        who: For::Fallback,
        name: "a",
        source: "heroes/A/map_sprite.png",
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
        "        fallback: (image: \"units/a.png\", frame: (1, 0), walk: true),\n    ),\n)\n"
    ));
    // The game's loader takes it, with the packed image and the unit's
    // sheet in the bundle.
    let (width, height, rgba) = &imported.image;
    let png = encode_png(*width, *height, rgba).unwrap();
    let mut sheet = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    sheet.extend(48u32.to_be_bytes());
    sheet.extend(80u32.to_be_bytes());
    let files = [
        ("tilesets/tiny_tales.png", png.as_slice()),
        ("units/a.png", sheet.as_slice()),
    ];
    let content = trpg_content::load_embedded().unwrap();
    let refs = TilesetRefs {
        images: &ImageTable::from_files(files).unwrap(),
        terrain: &content.terrain.display,
        classes: &content.classes,
        characters: &content.characters,
    };
    let read = parse_tileset("tiny_tales.ron", "tiny_tales", &text, &refs).unwrap();
    let terrain = read.terrain.unwrap();
    assert_eq!(terrain.tile_px, (16, 16));
    assert_eq!(terrain.look.layers.len(), 3);
    assert_eq!(terrain.looks["indoor"].layers.len(), 1);
    assert_eq!(
        terrain.look.tiles.len(),
        content.terrain.display.terrains.len()
    );
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
    assert_eq!(looks, ["indoor", "outdoor"]);
    // Every layer is used by a look.
    for name in map.layers.keys() {
        let used = map.looks.values().any(|look| look.layers.contains(name));
        assert!(used, "layer {name} is in no look");
    }
}

#[test]
fn run_writes_the_image_and_the_tileset_into_the_private_checkout() {
    let temp = Temp::new("run").with_sheet(LIBRARY_DIR);
    let root = &temp.0;
    // Without the checkout's game folder, or the mapping: nothing.
    let err = run(root, &UNITS).unwrap_err();
    assert!(
        err.contains("cargo xtask private-assets --library"),
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
         assets-private/game/tilesets/tiny_tales.ron"
    );
    let game = root.join(GAME_DIR);
    let png = fs::read(game.join("tilesets/tiny_tales.png")).unwrap();
    let palette = PaletteDef::load().unwrap();
    let colors = |name: &str| palette.get(name);
    let map = mapping(LAYERS, LOOKS);
    let imported = import(&root.join(LIBRARY_DIR), &map, &terrains(), &colors).unwrap();
    assert_eq!(decode_png(&png).unwrap(), imported.image);
    let text = fs::read_to_string(game.join("tilesets/tiny_tales.ron")).unwrap();
    assert_eq!(text, tileset(&imported, &UNITS));
    // Without the library: nothing, though the game folder is there.
    let bare = Temp::new("run-bare");
    fs::create_dir_all(bare.0.join(GAME_DIR)).unwrap();
    let err = run(&bare.0, &UNITS).unwrap_err();
    assert!(err.contains("is missing"), "{err}");
    // A file where the tilesets folder goes fails the write.
    let blocked = Temp::new("run-blocked").with_sheet(LIBRARY_DIR);
    blocked.write(MAPPING_PATH, mapping_text(LAYERS, LOOKS).as_bytes());
    blocked.write("assets-private/game/tilesets", b"x");
    let err = run(&blocked.0, &UNITS).unwrap_err();
    assert!(err.starts_with("creating "), "{err}");
}
