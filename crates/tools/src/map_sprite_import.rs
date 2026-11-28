//! `cargo xtask map-sprite-import`: copies the bought map sprites the game
//! uses into `assets-private/game/units/` under stable names (ticket 0436,
//! ADR-0049), and cuts the mounted ones from `MattWalkden`'s *Fantasy Battle
//! Pack* (ticket 0499). The tileset that names them,
//! `assets-private/game/tilesets/tiny_tales.ron`, is written by
//! `cargo xtask tileset-import` (ADR-0052), with [`units`]: run it after
//! this.
//!
//! A bought map sprite is one 48×80 sheet per character or class: 3
//! columns (walking frames) × 4 rows (facing down, left, right, up) of
//! 16×20 frames. The tileset names each one's standing, front-facing
//! frame, column 1 of row 0. Which sprite each class and character gets is
//! the table [`SPRITES`] (`docs/design/look-and-feel.md` has it in words).
//!
//! A rider is bigger than a Tiny Tales sprite, so it is cut ([`cut`]) into
//! the same 3 × 4 layout of [`RIDER_FRAME`] frames, which the tileset names
//! with `frame_px`. The rider on foot keeps a Tiny Tales knight: the
//! tileset names it in its `on_foot` table, by class.
//!
//! The output is bought art (ADR-0040): it goes to the private assets
//! checkout, never into this repository.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use shuyi_cli::codec::{decode_png, encode_png};
use trpg_content::image::png_size;

/// Shown for `--help` and after a bad argument.
pub const USAGE: &str = "usage: cargo xtask map-sprite-import [--list]\n\n\
Copies each bought map sprite the game uses from\n\
assets-private/library/tiny-tales/characters/ to assets-private/game/units/<name>.png,\n\
and cuts the riders from assets-private/library/mattwalkden/Sprite Sheets/.\n\
--list  print which sprite each class and character gets, and copy nothing\n\n\
Run `cargo shuyi private-assets --library` first. Afterwards run\n\
`cargo xtask tileset-import`, which writes the tileset that names them; commit and\n\
push in assets-private/; then run `cargo xtask assets-pin --add`.";

/// The sorted bought characters, relative to the repo root.
pub const LIBRARY_DIR: &str = "assets-private/library/tiny-tales/characters";
/// `MattWalkden`'s unit sheets, relative to the repo root.
pub const BATTLE_PACK_DIR: &str = "assets-private/library/mattwalkden/Sprite Sheets";
/// The private assets the game is built with, relative to the repo root.
pub const GAME_DIR: &str = "assets-private/game";
/// The tileset's id (`trpg_ui::map_view::GAME_TILESET`).
pub const TILESET_ID: &str = "tiny_tales";
/// A bought sheet's size in pixels.
pub const SHEET: (u32, u32) = (48, 80);
/// A frame's size in pixels.
pub const FRAME: (u32, u32) = (16, 20);
/// The standing, front-facing frame.
pub const STANDING: (u32, u32) = (1, 0);
/// A `MattWalkden` unit sheet's size in pixels: 8 × 14 cells.
pub const BATTLE_SHEET: (u32, u32) = (256, 448);
/// A cell of a `MattWalkden` sheet, in pixels.
pub const BATTLE_CELL: u32 = 32;
/// A cut rider's frame: the part of a cell kept, in pixels.
pub const RIDER_FRAME: (u32, u32) = (24, 24);
/// Where in its cell a rider's frame starts: centred across, the horse's
/// hooves on its bottom row (the cell's row 23).
pub const RIDER_IN_CELL: (u32, u32) = (4, 0);
/// The `MattWalkden` sheet's row of move frames for each of the game's
/// facings (down, left, right, up), and whether it is mirrored: the pack
/// faces right, down and up, and left is right mirrored.
pub const RIDER_ROWS: [(u32, bool); 4] = [(6, false), (1, true), (1, false), (11, false)];

/// Who a sprite is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum For {
    /// A character id (or the lead's `lead_m` / `lead_f`).
    Character(&'static str),
    /// A class id.
    Class(&'static str),
    /// A class id, for its riders **on foot** (0083; the skin picks it once
    /// ticket 0334 is done).
    OnFoot(&'static str),
    /// Any unit with neither.
    Fallback,
}

/// One bought sprite the game uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapSprite {
    /// Who it is for.
    pub who: For,
    /// Its file name in `game/units/`, without `.png`.
    pub name: &'static str,
    /// The bought sheet.
    pub source: Source,
}

/// Where a sprite comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A Tiny Tales map sprite sheet, relative to [`LIBRARY_DIR`], as it is.
    Copy(&'static str),
    /// A `MattWalkden` unit sheet, relative to [`BATTLE_PACK_DIR`], [`cut`].
    Cut(&'static str),
}

impl Source {
    /// The sheet's path from the repo root.
    fn path(self) -> String {
        match self {
            Source::Copy(path) => format!("{LIBRARY_DIR}/{path}"),
            Source::Cut(path) => format!("{BATTLE_PACK_DIR}/{path}"),
        }
    }

    /// Its frame's size, if not [`FRAME`].
    fn frame_px(self) -> Option<(u32, u32)> {
        matches!(self, Source::Cut(_)).then_some(RIDER_FRAME)
    }
}

const fn sprite(who: For, name: &'static str, source: &'static str) -> MapSprite {
    let source = Source::Copy(source);
    MapSprite { who, name, source }
}

/// A rider of mounted class `class`, cut from `MattWalkden`'s `source`.
const fn rider(class: &'static str, name: &'static str, source: &'static str) -> MapSprite {
    let (who, source) = (For::Class(class), Source::Cut(source));
    MapSprite { who, name, source }
}

/// The Tiny Tales knight on foot, the picture of a rider of `class` off
/// the horse.
const fn on_foot(class: &'static str) -> MapSprite {
    sprite(
        For::OnFoot(class),
        "knight_m1",
        "map-sprites-only/human-knights/Knight_M1_A.png",
    )
}

/// The lance rider, `MattWalkden`'s red lance cavalier.
const LANCE: &str = "LanceCavalier/LanceCavalier_ShortHair_Red1.png";
/// The bow rider, `MattWalkden`'s red mounted archer.
const BOW: &str = "MountedArcher/MountedArcher_Red1.png";

/// Every class and named character of the Quick Battle and Chapter 1, and
/// the bought sprite each is drawn as. Two rows may share a sprite.
pub const SPRITES: [MapSprite; 27] = [
    // The lead, by gender; the Exile class (the placeholder lord) as the
    // male lead.
    sprite(
        For::Character("lead_m"),
        "fighter_male",
        "heroes/FighterMale/map_sprite.png",
    ),
    sprite(
        For::Character("lead_f"),
        "fighter_female",
        "heroes/FighterFemale/map_sprite.png",
    ),
    sprite(
        For::Class("exile"),
        "fighter_male",
        "heroes/FighterMale/map_sprite.png",
    ),
    // Joss (ticket 0858): the Thief hero, as his face.
    sprite(
        For::Character("defector"),
        "thief",
        "heroes/Thief/map_sprite.png",
    ),
    sprite(For::Class("mage"), "witch", "heroes/Witch/map_sprite.png"),
    sprite(
        For::Class("archer"),
        "archer",
        "heroes/Archer/map_sprite.png",
    ),
    sprite(
        For::Class("guard"),
        "church_knight",
        "battler-classes/faith-and-evil/Human_Church_6__Church_Knight/map_sprite.png",
    ),
    sprite(
        For::Class("cleric"),
        "church_cleric",
        "battler-classes/faith-and-evil/Human_Church_1__Church_Cleric/map_sprite.png",
    ),
    // The mounted classes: MattWalkden's lance rider for those with a
    // spear, its bow rider for those with a bow (ticket 0499).
    rider("rider", "lance_cavalier", LANCE),
    rider("lancer", "lance_cavalier", LANCE),
    rider("high_lancer", "lance_cavalier", LANCE),
    rider("iron_rider", "lance_cavalier", LANCE),
    rider("juggernaut", "lance_cavalier", LANCE),
    rider("outrider", "mounted_archer", BOW),
    rider("windrunner", "mounted_archer", BOW),
    // Off the horse (0083), each is the Tiny Tales knight on foot.
    on_foot("rider"),
    on_foot("lancer"),
    on_foot("high_lancer"),
    on_foot("iron_rider"),
    on_foot("juggernaut"),
    on_foot("outrider"),
    on_foot("windrunner"),
    sprite(
        For::Class("brigand"),
        "warrior_m1",
        "map-sprites-only/human-advanced/Warrior_M1.png",
    ),
    sprite(
        For::Class("raider"),
        "fighter_m1",
        "map-sprites-only/human-advanced/Fighter_M1.png",
    ),
    sprite(
        For::Class("fire_elemental"),
        "fire_elemental",
        "battler-classes/elemental-forces/Fire_Elemental/map_sprite.png",
    ),
    sprite(
        For::Class("frost_elemental"),
        "ice_elemental",
        "battler-classes/elemental-forces/Ice_Elemental/map_sprite.png",
    ),
    sprite(
        For::Fallback,
        "adventurer_m1",
        "map-sprites-only/human-advanced/Adventurer_M1.png",
    ),
];

/// The bundle path of sprite `name`.
fn bundle_path(name: &str) -> String {
    format!("units/{name}.png")
}

/// The `unit_px` and `units` fields of the tileset file naming `sprites`,
/// each unit its standing frame (`tileset-import` writes the file).
pub fn units(sprites: &[MapSprite]) -> String {
    let entry = |s: &MapSprite| {
        let (path, (column, row)) = (bundle_path(s.name), STANDING);
        let size = s.source.frame_px();
        let size = size.map_or(String::new(), |(w, h)| format!(", frame_px: ({w}, {h})"));
        format!("(image: \"{path}\", frame: ({column}, {row}){size}, walk: true)")
    };
    let table = |pick: fn(For) -> Option<&'static str>| {
        let mut rows = String::new();
        for s in sprites {
            if let Some(id) = pick(s.who) {
                // Writing to a `String` can't fail.
                let _ = writeln!(rows, "            \"{id}\": {},", entry(s));
            }
        }
        rows
    };
    let characters = table(|who| match who {
        For::Character(id) => Some(id),
        _ => None,
    });
    let classes = table(|who| match who {
        For::Class(id) => Some(id),
        _ => None,
    });
    let on_foot = table(|who| match who {
        For::OnFoot(id) => Some(id),
        _ => None,
    });
    let fallback = sprites.iter().find(|s| s.who == For::Fallback);
    let fallback = fallback.map_or_default(entry);
    format!(
        "    unit_px: ({}, {}),\n    units: (\n        \
         characters: {{\n{characters}        }},\n        classes: {{\n{classes}        }},\n        \
         fallback: {fallback},\n        on_foot: {{\n{on_foot}        }},\n    ),\n",
        FRAME.0, FRAME.1
    )
}

/// Which sprite each class and character gets, one per line.
pub fn list(sprites: &[MapSprite]) -> String {
    let mut out = String::new();
    for s in sprites {
        let who = match s.who {
            For::Character(id) => format!("character {id}"),
            For::Class(id) => format!("class {id}"),
            For::OnFoot(id) => format!("class {id} on foot"),
            For::Fallback => "any other unit".to_owned(),
        };
        let (path, from) = (bundle_path(s.name), s.source.path());
        let cut = if matches!(s.source, Source::Cut(_)) {
            ", cut"
        } else {
            ""
        };
        let _ = writeln!(out, "{who:<24} {path:<28} {from}{cut}");
    }
    out
}

/// A `MattWalkden` unit sheet's PNG `bytes` cut into a walking sheet of the
/// game's layout (3 walking frames × facing down, left, right, up) of
/// [`RIDER_FRAME`] frames: the pack's first three move frames of each
/// facing ([`RIDER_ROWS`]), the right-facing ones mirrored for left. The
/// half-transparent drop shadow is dropped (the map skin draws its own
/// outline round a picture's shape). An error if a frame's figure reaches
/// outside the part kept.
pub fn cut(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let (width, height, pixels) = decode_png(bytes)?;
    if (width, height) != BATTLE_SHEET {
        let (bw, bh) = BATTLE_SHEET;
        return Err(format!(
            "is {width}×{height} px, not a {bw}×{bh} px unit sheet"
        ));
    }
    let (fw, fh) = RIDER_FRAME;
    let (across, down) = (SHEET.0 / FRAME.0, SHEET.1 / FRAME.1);
    let out_w = fw * across;
    let mut out = vec![0u8; (out_w * fh * down * 4) as usize];
    let at = |x: u32, y: u32| ((y * width + x) * 4) as usize;
    for (facing, (row, mirrored)) in (0..).zip(RIDER_ROWS) {
        for column in 0..across {
            let (cx, cy) = (column * BATTLE_CELL, row * BATTLE_CELL);
            for y in 0..BATTLE_CELL {
                for x in 0..BATTLE_CELL {
                    let src = at(cx + x, cy + y);
                    // A pixel of the figure: solid (the shadow is half).
                    if pixels[src + 3] != u8::MAX {
                        continue;
                    }
                    // Mirrored, the cell's column `x` lands on `31 - x`.
                    let x = if mirrored { BATTLE_CELL - 1 - x } else { x };
                    let (ox, oy) = RIDER_IN_CELL;
                    let (fx, fy) = (x.wrapping_sub(ox), y.wrapping_sub(oy));
                    if fx >= fw || fy >= fh {
                        return Err(format!(
                            "the figure in cell ({column}, {row}) reaches outside the {fw}×{fh} px \
                             frame at ({x}, {y})"
                        ));
                    }
                    let dst = (((facing * fh + fy) * out_w + column * fw + fx) * 4) as usize;
                    out[dst..dst + 4].copy_from_slice(&pixels[src..src + 4]);
                }
            }
        }
    }
    encode_png(out_w, fh * down, &out)
}

/// Runs the command under repo root `root`: copies every sprite of
/// `sprites`. Every source is checked before any file is written.
pub fn run(root: &Path, sprites: &[MapSprite]) -> Result<String, String> {
    let library = root.join(LIBRARY_DIR);
    if !library.is_dir() || !root.join(GAME_DIR).is_dir() {
        return Err(format!(
            "{LIBRARY_DIR}/ or {GAME_DIR}/ is missing: run `cargo shuyi private-assets --library` first"
        ));
    }
    let mut files: Vec<(&str, Vec<u8>)> = Vec::new();
    for s in sprites {
        let path = root.join(s.source.path());
        let read = fs::read(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
        let bytes = match s.source {
            Source::Copy(_) if png_size(&read) != Some(SHEET) => {
                return Err(format!(
                    "{} is not a {}×{} px map sprite sheet",
                    path.display(),
                    SHEET.0,
                    SHEET.1
                ));
            }
            Source::Copy(_) => read,
            Source::Cut(_) => cut(&read).map_err(|e| format!("{}: {e}", path.display()))?,
        };
        match files.iter().find(|(name, _)| *name == s.name) {
            Some((_, other)) if *other != bytes => {
                return Err(format!("two different sprites are named {}", s.name));
            }
            Some(_) => {}
            None => files.push((s.name, bytes)),
        }
    }
    let game = root.join(GAME_DIR);
    let write = |path: std::path::PathBuf, bytes: &[u8]| {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
        }
        fs::write(&path, bytes).map_err(|e| format!("writing {}: {e}", path.display()))
    };
    for (name, bytes) in &files {
        write(game.join(bundle_path(name)), bytes)?;
    }
    Ok(format!(
        "map-sprite-import: wrote {} sprites to {GAME_DIR}/units/; now run \
         `cargo xtask tileset-import`, which writes the tileset that names them",
        files.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The start of a `w × h` PNG file: enough for `png_size`.
    fn png_header(w: u32, h: u32) -> Vec<u8> {
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend(w.to_be_bytes());
        png.extend(h.to_be_bytes());
        png
    }

    const TWO: [MapSprite; 4] = [
        sprite(For::Character("lead_f"), "b", "heroes/B/map_sprite.png"),
        sprite(For::Class("exile"), "a", "heroes/A/map_sprite.png"),
        sprite(For::Class("mage"), "b", "heroes/B/map_sprite.png"),
        sprite(For::Fallback, "a", "heroes/A/map_sprite.png"),
    ];

    /// A repo root with a library holding sheets `A` and `B`.
    fn root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("xtask-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for (hero, tail) in [("A", 1u8), ("B", 2)] {
            let dir = root.join(LIBRARY_DIR).join("heroes").join(hero);
            fs::create_dir_all(&dir).unwrap();
            let mut png = png_header(48, 80);
            png.push(tail);
            fs::write(dir.join("map_sprite.png"), png).unwrap();
        }
        fs::create_dir_all(root.join(GAME_DIR)).unwrap();
        root
    }

    #[test]
    fn the_units_table_names_each_units_standing_frame() {
        assert_eq!(
            units(&TWO),
            "    unit_px: (16, 20),\n    units: (\n        \
             characters: {\n            \
             \"lead_f\": (image: \"units/b.png\", frame: (1, 0), walk: true),\n        },\n        \
             classes: {\n            \
             \"exile\": (image: \"units/a.png\", frame: (1, 0), walk: true),\n            \
             \"mage\": (image: \"units/b.png\", frame: (1, 0), walk: true),\n        },\n        \
             fallback: (image: \"units/a.png\", frame: (1, 0), walk: true),\n        \
             on_foot: {\n        },\n    ),\n"
        );
        assert_eq!(TILESET_ID, "tiny_tales");
    }

    /// Ticket 0499: a cut rider says its frame's size; a class's rider on
    /// foot goes in `on_foot`.
    #[test]
    fn a_rider_has_its_frame_size_and_its_picture_on_foot() {
        let sprites = [
            rider("rider", "horse", "Lance/Lance_Red1.png"),
            on_foot("rider"),
            sprite(For::Fallback, "a", "heroes/A/map_sprite.png"),
        ];
        assert_eq!(
            units(&sprites),
            "    unit_px: (16, 20),\n    units: (\n        characters: {\n        },\n        \
             classes: {\n            \
             \"rider\": (image: \"units/horse.png\", frame: (1, 0), frame_px: (24, 24), \
             walk: true),\n        },\n        \
             fallback: (image: \"units/a.png\", frame: (1, 0), walk: true),\n        \
             on_foot: {\n            \
             \"rider\": (image: \"units/knight_m1.png\", frame: (1, 0), walk: true),\n        \
             },\n    ),\n"
        );
        let list = list(&sprites);
        assert!(list.starts_with(
            "class rider              units/horse.png              \
             assets-private/library/mattwalkden/Sprite Sheets/Lance/Lance_Red1.png, cut\n"
        ));
        assert!(list.contains("class rider on foot      units/knight_m1.png"));
    }

    /// The pixels of a 256 × 448 `MattWalkden` sheet: clear, with a solid
    /// pixel of colour `(column, row, 7, 255)` at `(x, y)` in each cell of
    /// the move rows the cutter reads, and a half-transparent shadow
    /// pixel under it.
    fn battle_sheet(x: u32, y: u32) -> Vec<u8> {
        let (width, height) = BATTLE_SHEET;
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        for (row, _) in RIDER_ROWS {
            for column in 0..3u8 {
                let (px, py) = (u32::from(column) * BATTLE_CELL + x, row * BATTLE_CELL + y);
                let at = ((py * width + px) * 4) as usize;
                let row = u8::try_from(row).unwrap();
                pixels[at..at + 4].copy_from_slice(&[column, row, 7, 255]);
                let shadow = (((py + 1) * width + px) * 4) as usize;
                pixels[shadow..shadow + 4].copy_from_slice(&[0, 0, 0, 128]);
            }
        }
        pixels
    }

    #[test]
    fn a_rider_is_cut_into_the_games_layout_mirrored_for_left_without_its_shadow() {
        let (w, h) = BATTLE_SHEET;
        // A pixel 5 px from the cell's left, on row 20.
        let sheet = encode_png(w, h, &battle_sheet(5, 20)).unwrap();
        let (cw, ch, out) = decode_png(&cut(&sheet).unwrap()).unwrap();
        assert_eq!((cw, ch), (72, 96));
        let solid: Vec<(u32, u32, [u8; 4])> = (0..ch)
            .flat_map(|y| (0..cw).map(move |x| (x, y)))
            .filter_map(|(x, y)| {
                let i = ((y * cw + x) * 4) as usize;
                let p = [out[i], out[i + 1], out[i + 2], out[i + 3]];
                (p[3] != 0).then_some((x, y, p))
            })
            .collect();
        // One pixel a frame, nothing else (no shadow): down from row 6,
        // left from row 1 mirrored (cell x 26, frame x 22), right from row
        // 1, up from row 11; frame x 1 is cell x 5.
        let mut want = Vec::new();
        let rows: [(u8, u32); 4] = [(6, 1), (1, 22), (1, 1), (11, 1)];
        for (facing, (row, fx)) in (0u32..).zip(rows) {
            for column in 0..3u8 {
                let (x, y) = (u32::from(column) * 24 + fx, facing * 24 + 20);
                want.push((x, y, [column, row, 7, 255]));
            }
        }
        want.sort_by_key(|(x, y, _)| (*y, *x));
        assert_eq!(solid, want);
        // The frame's edges: x 4 to 27 and y 0 to 23 of the cell.
        for (x, y) in [(4, 0), (27, 23)] {
            let sheet = encode_png(w, h, &battle_sheet(x, y)).unwrap();
            assert!(cut(&sheet).is_ok(), "({x}, {y})");
        }
        // A figure outside it is an error; so is a sheet of another size.
        for (x, y, at) in [(3, 0, "(3, 0)"), (28, 5, "(28, 5)"), (10, 24, "(10, 24)")] {
            let sheet = encode_png(w, h, &battle_sheet(x, y)).unwrap();
            let err = cut(&sheet).unwrap_err();
            assert!(err.contains("reaches outside the 24×24 px frame"), "{err}");
            assert!(err.ends_with(&format!("at {at}")), "{err}");
        }
        let small = encode_png(48, 80, &vec![0; 48 * 80 * 4]).unwrap();
        assert_eq!(
            cut(&small).unwrap_err(),
            "is 48×80 px, not a 256×448 px unit sheet"
        );
    }

    #[test]
    fn run_cuts_a_rider_from_the_battle_pack() {
        let root = root("map-sprite-import-cut");
        let dir = root.join(BATTLE_PACK_DIR).join("Lance");
        fs::create_dir_all(&dir).unwrap();
        let (w, h) = BATTLE_SHEET;
        let sheet = encode_png(w, h, &battle_sheet(8, 8)).unwrap();
        fs::write(dir.join("Lance_Red1.png"), &sheet).unwrap();
        let sprites = [
            rider("rider", "horse", "Lance/Lance_Red1.png"),
            rider("lancer", "horse", "Lance/Lance_Red1.png"),
            TWO[3],
        ];
        run(&root, &sprites).unwrap();
        let game = root.join(GAME_DIR);
        let written = fs::read(game.join("units/horse.png")).unwrap();
        assert_eq!(written, cut(&sheet).unwrap());
        assert_eq!(png_size(&written), Some((72, 96)));
        // A sheet the cutter refuses names its file.
        fs::write(dir.join("Lance_Red1.png"), png_header(48, 80)).unwrap();
        let err = run(&root, &sprites).unwrap_err();
        assert!(err.contains("Lance_Red1.png: "), "{err}");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_games_table_names_real_ids_once_each_and_one_fallback() {
        let content = trpg_content::load_embedded().unwrap();
        let mut seen = Vec::new();
        for s in SPRITES {
            assert!(!seen.contains(&s.who), "{:?} twice", s.who);
            seen.push(s.who);
            match s.who {
                For::Class(id) | For::OnFoot(id) => {
                    let class = trpg_core::ClassId(id.to_owned());
                    let def = content.classes.get(&class);
                    assert!(def.is_some(), "class {id}");
                    // A rider is cut from the battle pack, and every class
                    // cut from it is mounted and has its picture on foot.
                    let mounted = def.is_some_and(|d| d.tags.mounted);
                    if matches!(s.source, Source::Cut(_)) || matches!(s.who, For::OnFoot(_)) {
                        assert!(mounted, "class {id} isn't mounted");
                    }
                    if mounted {
                        let pair = [For::Class(id), For::OnFoot(id)];
                        for who in pair {
                            assert!(SPRITES.iter().any(|o| o.who == who), "{who:?}");
                        }
                        let horse = SPRITES.iter().find(|o| o.who == For::Class(id));
                        assert!(matches!(horse.unwrap().source, Source::Cut(_)), "{id}");
                    }
                }
                For::Character(id) => {
                    let character = trpg_core::CharacterId(id.to_owned());
                    let known = trpg_content::tileset::LEAD_PICTURES.contains(&id)
                        || content.characters.characters.contains_key(&character);
                    assert!(known, "{id}");
                }
                For::Fallback => {}
            }
            let valid = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_';
            assert!(s.name.chars().all(valid), "{}", s.name);
            // One name, one source.
            let same = SPRITES.iter().filter(|o| o.name == s.name);
            assert!(same.clone().all(|o| o.source == s.source), "{}", s.name);
        }
        assert!(seen.contains(&For::Fallback));
        // Every class of the Quick Battle has a sprite of its own.
        let state = trpg_ui::screens::battle::quick_battle(&content).unwrap();
        for unit in state.units() {
            let id = unit.class.0.as_str();
            let has = |who: &For| matches!(who, For::Class(class) if *class == id);
            assert!(seen.iter().any(has), "class {id}");
        }
    }

    #[test]
    fn the_list_says_who_gets_which_sprite() {
        let text = list(&TWO);
        assert_eq!(text.lines().count(), 4);
        assert!(text.starts_with("character lead_f         units/b.png"));
        assert!(text.contains("class exile              units/a.png"));
        assert!(text.contains("any other unit           units/a.png"));
        assert!(text.ends_with("heroes/A/map_sprite.png\n"));
    }

    #[test]
    fn run_copies_each_sprite_once_and_leaves_the_tileset_to_its_importer() {
        let root = root("map-sprite-import");
        let summary = run(&root, &TWO).unwrap();
        assert_eq!(
            summary,
            "map-sprite-import: wrote 2 sprites to assets-private/game/units/; now run \
             `cargo xtask tileset-import`, which writes the tileset that names them"
        );
        let game = root.join(GAME_DIR);
        let read = |path: &str| fs::read(game.join(path)).unwrap();
        assert_eq!(read("units/a.png").last(), Some(&1));
        assert_eq!(read("units/b.png").last(), Some(&2));
        assert!(!game.join("tilesets").exists());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_missing_checkout_or_a_bad_sheet_writes_nothing() {
        let bare = std::env::temp_dir().join(format!("xtask-msi-bare-{}", std::process::id()));
        let err = run(&bare, &TWO).unwrap_err();
        assert!(
            err.contains("cargo shuyi private-assets --library"),
            "{err}"
        );
        let root = root("map-sprite-import-bad");
        let game = root.join(GAME_DIR);
        // A sheet that isn't there.
        let missing = [sprite(For::Fallback, "c", "heroes/C/map_sprite.png")];
        let err = run(&root, &missing).unwrap_err();
        assert!(err.starts_with("reading "), "{err}");
        // A file of another size.
        let sheet = root.join(LIBRARY_DIR).join("heroes/B/map_sprite.png");
        fs::write(&sheet, png_header(80, 80)).unwrap();
        let err = run(&root, &TWO).unwrap_err();
        assert!(err.ends_with("is not a 48×80 px map sprite sheet"), "{err}");
        // One name for two different sheets.
        fs::write(&sheet, [png_header(48, 80), vec![9]].concat()).unwrap();
        let clash = [
            TWO[1],
            sprite(For::Fallback, "a", "heroes/B/map_sprite.png"),
        ];
        let err = run(&root, &clash).unwrap_err();
        assert_eq!(err, "two different sprites are named a");
        assert_eq!(fs::read_dir(&game).unwrap().count(), 0);
        // Without game/, even with the library.
        fs::remove_dir_all(&game).unwrap();
        assert!(run(&root, &TWO).is_err());
        // A file where the units folder goes fails the write.
        fs::create_dir_all(&game).unwrap();
        fs::write(game.join("units"), "x").unwrap();
        let err = run(&root, &[TWO[1], TWO[3]]).unwrap_err();
        assert!(err.starts_with("creating "), "{err}");
        fs::remove_dir_all(&root).unwrap();
    }
}
