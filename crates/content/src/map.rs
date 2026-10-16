//! Battle map files (`assets/maps/*.map`): a RON header, a `---` line, then
//! one character per tile. The format is documented in
//! `assets/maps/README.md`.
//!
//! The format is the engine's (`shuyi_assets::grid_map`, ticket 1116):
//! this module gives it the game's terrains, tile looks and size limit,
//! and the game's tile features, shops and chests.
//!
//! A header may also say how the map looks ([`MapLook`]). That is kept
//! beside the map the rules see, never in it (ADR-0038).
use crate::prelude::*;

use alloc::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use shuyi_assets::grid_map::{self, GridMap, Legend, MapFeature, MapFormat};
use trpg_core::{
    BattleMap, ItemId, ItemTable, Loot, Shop, ShopKind, TerrainId, TerrainTable, TileFeature,
};

pub use shuyi_assets::grid_map::{DEFAULT_TILES, MapLook, SEPARATOR};
pub(crate) use shuyi_assets::grid_map::{column_number, line_number};

use crate::bundle;
use crate::error::ContentError;
use crate::lighting::Lighting;
use crate::terrain::TerrainDisplayTable;

/// Directory of map files inside the asset bundle.
pub const MAPS_DIR: &str = "maps";
/// File extension of map files.
pub const MAP_EXTENSION: &str = ".map";
/// Largest allowed width and height, in tiles.
pub const MAX_MAP_SIZE: usize = 64;
/// The sets of terrain pictures a map may ask for in its header's `look`
/// (ADR-0052). A tileset names its pictures for each; one that lacks a
/// look paints the map with its own. The first is the look of a map that
/// names none.
pub const TILE_LOOKS: [&str; 2] = [DEFAULT_TILES, "indoor"];

/// What the game's maps may be.
const FORMAT: MapFormat<'static> = MapFormat {
    looks: &TILE_LOOKS,
    max_size: MAX_MAP_SIZE,
};

/// A map's legend: tile character → terrain string id, plus the resolved
/// [`TerrainId`]s.
pub type MapLegend = Legend<TerrainId>;

/// A parsed map together with the legend it was written with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapDef {
    /// The map rules see.
    pub map: BattleMap,
    /// The legend from the file (for printing it back).
    pub legend: MapLegend,
    /// How the map looks.
    pub look: MapLook,
}

/// A tile feature as written in a map header: the game's features in the
/// engine's format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum FeatureDef {
    Shop { kind: ShopKind, stock: Vec<String> },
    Chest(LootDef),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum LootDef {
    Gold(u32),
    Item(String),
}

impl FeatureDef {
    fn to_core(&self) -> TileFeature {
        match self {
            FeatureDef::Shop { kind, stock } => TileFeature::Shop(Shop {
                kind: *kind,
                stock: stock.iter().map(|i| ItemId::new(i)).collect(),
            }),
            FeatureDef::Chest(LootDef::Gold(n)) => TileFeature::Chest(Loot::Gold(*n)),
            FeatureDef::Chest(LootDef::Item(i)) => TileFeature::Chest(Loot::Item(ItemId::new(i))),
        }
    }

    fn from_core(feature: &TileFeature) -> Self {
        match feature {
            TileFeature::Shop(shop) => FeatureDef::Shop {
                kind: shop.kind,
                stock: shop.stock.iter().map(|i| i.0.clone()).collect(),
            },
            TileFeature::Chest(Loot::Gold(n)) => FeatureDef::Chest(LootDef::Gold(*n)),
            TileFeature::Chest(Loot::Item(i)) => FeatureDef::Chest(LootDef::Item(i.0.clone())),
        }
    }
}

impl MapFeature for FeatureDef {
    /// Problems visible without the item table: a shop's list must be empty
    /// for a blacksmith and not empty otherwise; a chest can't hold 0 gold.
    fn problem(&self) -> Option<String> {
        let problem = match self {
            FeatureDef::Shop { kind, stock } => match (kind, stock.is_empty()) {
                (ShopKind::Blacksmith, false) => {
                    "a blacksmith sells nothing; its stock must be empty"
                }
                (ShopKind::Armoury | ShopKind::Vendor, true) => "the shop sells nothing",
                _ => return None,
            },
            FeatureDef::Chest(LootDef::Gold(0)) => "the chest holds 0 gold",
            FeatureDef::Chest(_) => return None,
        };
        Some(problem.to_owned())
    }
}

/// Loads every `*.map` file in the bundle, keyed by file stem
/// (`maps/test_small.map` → `"test_small"`). Reports every error of every
/// file.
pub fn load_all(
    display: &TerrainDisplayTable,
) -> Result<BTreeMap<String, MapDef>, Vec<ContentError>> {
    let mut maps = BTreeMap::new();
    let mut errors = Vec::new();
    for path in bundle::files_in(MAPS_DIR) {
        let Some(stem) = path
            .strip_prefix(MAPS_DIR)
            .and_then(|p| p.strip_prefix('/'))
            .and_then(|p| p.strip_suffix(MAP_EXTENSION))
        else {
            continue;
        };
        let file = bundle::display_path(path);
        let result = bundle::file(path)
            .ok_or_else(|| vec![ContentError::new(&file, "file is not valid UTF-8")])
            .and_then(|source| parse_map(&file, source, display));
        match result {
            Ok(def) => {
                maps.insert(stem.to_owned(), def);
            }
            Err(e) => errors.extend(e),
        }
    }
    if errors.is_empty() {
        Ok(maps)
    } else {
        Err(errors)
    }
}

/// Every map of `maps` whose light isn't one of `lighting`'s, each
/// reported where its file names it.
pub fn check_lights(maps: &BTreeMap<String, MapDef>, lighting: &Lighting) -> Vec<ContentError> {
    let unknown = maps
        .iter()
        .filter(|(_, def)| lighting.get(&def.look.light).is_none());
    unknown
        .map(|(id, def)| {
            let path = format!("{MAPS_DIR}/{id}{MAP_EXTENSION}");
            let source = bundle::file(&path).unwrap_or_default();
            light_error(
                &bundle::display_path(&path),
                source,
                &def.look.light,
                lighting,
            )
        })
        .collect()
}

/// The error for map file `file` (its text is `source`) naming `light`,
/// which isn't one of `lighting`'s: at the name in the header, or at the
/// file's start if it isn't found.
pub fn light_error(file: &str, source: &str, light: &str, lighting: &Lighting) -> ContentError {
    let header = grid_map::header_lines(source);
    let (line, col) = grid_map::header_position(&header, &format!("\"{light}\"")).unwrap_or((1, 1));
    let message = format!(
        "look: light \"{light}\" is not a light; the lights are {}",
        lighting.names()
    );
    ContentError::new(file, message).at(line, Some(col))
}

/// Parses map `source` (errors attributed to `file`), resolving legend
/// terrain ids in `display`. Reports every problem with its line and column.
pub fn parse_map(
    file: &str,
    source: &str,
    display: &TerrainDisplayTable,
) -> Result<MapDef, Vec<ContentError>> {
    let parsed: GridMap<TerrainId, FeatureDef> =
        grid_map::parse(file, source, &FORMAT, |id| display.id_of(id))?;
    let features = parsed
        .features
        .iter()
        .map(|(&pos, def)| (pos, def.to_core()))
        .collect();
    Ok(MapDef {
        map: BattleMap {
            features,
            ..BattleMap::new(parsed.name, parsed.tiles)
        },
        legend: parsed.legend,
        look: parsed.look,
    })
}

/// Prints `map` in `.map` format using `legend`, with `look` if it isn't
/// the look of a map that names none. `None` if a tile's terrain has no
/// character in the legend.
pub fn print_map(map: &BattleMap, legend: &MapLegend, look: &MapLook) -> Option<String> {
    grid_map::print(&GridMap {
        name: map.name.clone(),
        tiles: map.tiles.clone(),
        features: map
            .features
            .iter()
            .map(|(&p, f)| (p, FeatureDef::from_core(f)))
            .collect(),
        legend: legend.clone(),
        look: look.clone(),
    })
}

/// Checks every map's features against the items and terrain: shop and
/// chest items exist, each shop sells only what its kind may
/// ([`ShopKind::stocks`]), and every feature sits on a tile some movement
/// type can enter.
pub fn check_features(
    maps: &BTreeMap<String, MapDef>,
    items: &ItemTable,
    terrain: &TerrainTable,
) -> Vec<ContentError> {
    let mut errors = Vec::new();
    for (stem, def) in maps {
        let file = bundle::display_path(&format!("{MAPS_DIR}/{stem}{MAP_EXTENSION}"));
        for (pos, feature) in &def.map.features {
            let at = format!("feature at ({}, {})", pos.x, pos.y);
            let mut error =
                |msg: String| errors.push(ContentError::new(&file, format!("{at}: {msg}")));
            let passable = def
                .map
                .tiles
                .get(*pos)
                .and_then(|&t| terrain.get(t))
                .is_some_and(|t| t.move_cost.iter().any(Option::is_some));
            if !passable {
                error("the tile can't be entered".to_owned());
            }
            let ids: Vec<&ItemId> = match feature {
                TileFeature::Shop(shop) => shop.stock.iter().collect(),
                TileFeature::Chest(Loot::Item(i)) => vec![i],
                TileFeature::Chest(Loot::Gold(_)) => vec![],
            };
            for id in ids {
                match (items.get(id), feature) {
                    (None, _) => error(format!("unknown item \"{}\"", id.0)),
                    (Some(item), TileFeature::Shop(shop)) if !shop.kind.stocks(item) => {
                        error(format!("{:?} shop can't sell \"{}\"", shop.kind, id.0));
                    }
                    _ => {}
                }
            }
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use trpg_core::Pos;

    use super::*;
    use crate::terrain::TerrainDisplay;

    fn display() -> TerrainDisplayTable {
        let t = |id: &str| TerrainDisplay {
            id: id.to_owned(),
            glyphs: ['.', '.'],
            fg: "grass".into(),
            bg: "black".into(),
            decoration: false,
        };
        TerrainDisplayTable {
            terrains: vec![t("plain"), t("forest"), t("water"), t("wall")],
        }
    }

    const HEADER: &str = "(\n  name: \"Test\",\n  legend: { '.': \"plain\", 'T': \"forest\", '~': \"water\" },\n)\n---\n";

    fn parse(src: &str) -> Result<MapDef, Vec<ContentError>> {
        parse_map("m.map", src, &display())
    }

    fn errors(src: &str) -> Vec<String> {
        parse(src)
            .err()
            .unwrap_or_default()
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn an_unknown_light_is_reported_where_it_is_written() {
        let lighting = crate::lighting::load().unwrap();
        let source = "(\n  name: \"M\",\n  legend: {'.': \"plain\"},\n  look: (light: \"midnight\"),\n)\n---\n.\n";
        let error = light_error("m.map", source, "midnight", &lighting);
        assert_eq!(
            error.to_string(),
            "m.map:4:17: look: light \"midnight\" is not a light; the lights are \"dawn\", \"dusk\", \"night\", \"noon\""
        );
        // A name the header doesn't have (it was set some other way): the
        // file's start. One below the tiles' line isn't the header's.
        let below = "(name: \"M\")\n---\n\"midnight\"\n";
        let error = light_error("m.map", below, "midnight", &lighting);
        assert_eq!((error.line, error.column), (Some(1), Some(1)));
        // Every shipped map names a light that exists.
        let terrain = crate::TerrainDef::load(None).unwrap_or_default();
        let maps = load_all(&terrain.display).unwrap();
        assert_eq!(check_lights(&maps, &lighting), []);
        let mut odd = maps;
        for def in odd.values_mut() {
            def.look.light = "midnight".to_owned();
        }
        let errors = check_lights(&odd, &lighting);
        assert_eq!(errors.len(), odd.len());
        assert!(errors.iter().all(|e| e.file.contains("/maps/")));
    }

    #[test]
    fn embedded_maps_load() {
        let terrain = crate::terrain::TerrainDef::load(None).unwrap_or_default();
        let maps = load_all(&terrain.display);
        assert!(maps.is_ok(), "{maps:?}");
        let maps = maps.unwrap_or_default();
        let small = maps.get("test_small");
        assert!(small.is_some(), "{:?}", maps.keys());
        // test_small uses every legend entry.
        if let Some(def) = small {
            for (c, &t) in &def.legend.terrains {
                assert!(def.map.tiles.cells().contains(&t), "'{c}' unused");
            }
        }
    }

    const FEATURES: &str = "(
  name: \"F\",
  legend: { '.': \"plain\", '#': \"wall\" },
  features: {
    (0, 0): Shop(kind: Armoury, stock: [\"sword\", \"vest\"]),
    (1, 0): Shop(kind: Blacksmith, stock: []),
    (0, 1): Chest(Gold(50)),
    (1, 1): Chest(Item(\"potion\")),
  },
)
---
..#
..#
";

    fn feature_items() -> ItemTable {
        let mut items = ItemTable::default();
        let weapon = trpg_core::WeaponDef {
            name: "Sword".into(),
            kind: trpg_core::WeaponKind::Sword,
            rank: trpg_core::WeaponRank::E,
            might: 1,
            hit: 90,
            crit: 0,
            weight: 1,
            min_range: 1,
            max_range: 1,
            damage_type: trpg_core::DamageType::Physical,
            durability: 20,
            effective: vec![],
            arts: vec![],
            price: 100,
        };
        items
            .items
            .insert(ItemId::new("sword"), trpg_core::ItemDef::Weapon(weapon));
        items.items.insert(
            ItemId::new("vest"),
            trpg_core::ItemDef::Armour(trpg_core::ArmourDef {
                name: "Vest".into(),
                weight_class: trpg_core::ArmourWeight::Light,
                bonus: trpg_core::Stats::default(),
                weight: 0,
                price: 10,
            }),
        );
        items.items.insert(
            ItemId::new("potion"),
            trpg_core::ItemDef::Consumable(trpg_core::ConsumableDef {
                name: "Potion".into(),
                effect: trpg_core::ConsumableEffect::Heal(10),
                price: 10,
            }),
        );
        items
    }

    fn feature_terrain() -> TerrainTable {
        let rules = |name: &str, cost| trpg_core::TerrainRules {
            name: name.into(),
            move_cost: vec![cost, None],
            defense: 0,
            avoid: 0,
            heal_percent: 0,
        };
        TerrainTable {
            movement_types: vec!["foot".into(), "flying".into()],
            terrains: vec![
                rules("Plain", Some(1)),
                rules("Forest", Some(2)),
                rules("Water", None),
                rules("Wall", None),
            ],
        }
    }

    fn check(src: &str) -> Vec<String> {
        let def = parse(src).unwrap_or_else(|e| panic!("{e:?}"));
        let maps = BTreeMap::from([("f".to_owned(), def)]);
        check_features(&maps, &feature_items(), &feature_terrain())
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn parses_features() {
        let def = parse(FEATURES).unwrap_or_else(|e| panic!("{e:?}"));
        let m = &def.map;
        assert_eq!(m.features.len(), 4);
        assert_eq!(
            m.shop(Pos::new(0, 0)),
            Some(&Shop {
                kind: ShopKind::Armoury,
                stock: vec![ItemId::new("sword"), ItemId::new("vest")],
            })
        );
        assert_eq!(
            m.shop(Pos::new(1, 0)).map(|s| s.kind),
            Some(ShopKind::Blacksmith)
        );
        assert_eq!(m.chest(Pos::new(0, 1)), Some(&Loot::Gold(50)));
        assert_eq!(
            m.chest(Pos::new(1, 1)),
            Some(&Loot::Item(ItemId::new("potion")))
        );
        assert!(check(FEATURES).is_empty());
    }

    #[test]
    fn maps_without_features_have_none() {
        let def = parse(&format!(
            "{HEADER}..
"
        ))
        .ok();
        assert_eq!(def.map(|d| d.map.features.len()), Some(0));
    }

    #[test]
    fn features_round_trip() {
        let def = parse(FEATURES).ok();
        let printed = def
            .as_ref()
            .and_then(|d| print_map(&d.map, &d.legend, &d.look));
        assert!(
            printed
                .as_deref()
                .is_some_and(|p| p.contains("Chest(Gold(50))"))
        );
        let again = printed.as_deref().and_then(|p| parse(p).ok());
        assert!(def.is_some());
        assert_eq!(again, def);
    }

    #[test]
    fn feature_shape_errors() {
        let src = FEATURES
            .replace("(0, 0)", "(5, 0)")
            .replace("Blacksmith, stock: []", "Blacksmith, stock: [\"sword\"]")
            .replace("Gold(50)", "Gold(0)");
        assert_eq!(
            errors(&src),
            [
                "m.map: feature at (0, 1): the chest holds 0 gold",
                "m.map: feature at (1, 0): a blacksmith sells nothing; its stock must be empty",
                "m.map: feature at (5, 0): outside the map",
            ]
        );
        let empty = FEATURES.replace("[\"sword\", \"vest\"]", "[]");
        assert_eq!(
            errors(&empty),
            ["m.map: feature at (0, 0): the shop sells nothing"]
        );
        let vendor = empty.replace("Armoury", "Vendor");
        assert_eq!(
            errors(&vendor),
            ["m.map: feature at (0, 0): the shop sells nothing"]
        );
    }

    #[test]
    fn feature_content_errors() {
        let src = FEATURES
            .replace("\"vest\"]", "\"potion\", \"ghost\"]")
            .replace("Item(\"potion\")", "Item(\"relic\")")
            .replace("(1, 0): Shop", "(2, 0): Shop");
        assert_eq!(
            check(&src),
            [
                "assets/maps/f.map: feature at (0, 0): Armoury shop can't sell \"potion\"",
                "assets/maps/f.map: feature at (0, 0): unknown item \"ghost\"",
                "assets/maps/f.map: feature at (1, 1): unknown item \"relic\"",
                "assets/maps/f.map: feature at (2, 0): the tile can't be entered",
            ]
        );
        let vendor = FEATURES.replace("Armoury", "Vendor");
        assert_eq!(
            check(&vendor),
            [
                "assets/maps/f.map: feature at (0, 0): Vendor shop can't sell \"sword\"",
                "assets/maps/f.map: feature at (0, 0): Vendor shop can't sell \"vest\"",
            ]
        );
        // A tile only fliers can enter still counts as passable.
        let mut terrain = feature_terrain();
        terrain.terrains[3].move_cost = vec![None, Some(1)];
        let def = parse(&FEATURES.replace("(1, 0): Shop", "(2, 0): Shop"))
            .unwrap_or_else(|e| panic!("{e:?}"));
        let maps = BTreeMap::from([("f".to_owned(), def)]);
        assert!(check_features(&maps, &feature_items(), &terrain).is_empty());
    }

    #[test]
    fn embedded_map_features_are_valid() {
        let content = crate::load_embedded();
        assert!(content.is_ok(), "{content:?}");
        let small = content.ok().and_then(|c| c.maps.get("test_small").cloned());
        assert_eq!(small.map(|d| d.map.features.len()), Some(5));
    }

    /// Every map the game has (`assets/` and the test files) prints back
    /// to itself: parse, print, parse gives the same map.
    #[test]
    fn every_map_round_trips() {
        let terrain = crate::TerrainDef::load(None).unwrap_or_default();
        let maps = load_all(&terrain.display).unwrap();
        assert!(maps.contains_key("ch01") && maps.contains_key("test_small"));
        for (id, def) in &maps {
            let printed = print_map(&def.map, &def.legend, &def.look);
            let printed = printed.unwrap_or_else(|| panic!("{id} doesn't print"));
            let again = parse_map(id, &printed, &terrain.display);
            assert_eq!(again.as_ref(), Ok(def), "{id}:\n{printed}");
            let twice = again
                .ok()
                .and_then(|d| print_map(&d.map, &d.legend, &d.look));
            assert_eq!(twice, Some(printed), "{id}");
        }
    }
}
