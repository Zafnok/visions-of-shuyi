//! The terrain part of a tileset file as text (ADR-0052): what both the
//! generator of the public test tileset (`test_auto`) and the importer of
//! the bought tiles (`tileset_import`) write, so the two files are shaped
//! alike. `assets/tilesets/README.md` documents the format.

use std::fmt::Write as _;

use trpg_content::tileset::mix_name;

/// A tile of the tileset's image: `(column, row)`.
pub type At = (u32, u32);

/// A terrain look to write.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LookText {
    /// Each terrain's own tile, in the order written.
    pub terrain: Vec<(String, At)>,
    /// The layers painted over them, the first undermost.
    pub layers: Vec<LayerText>,
}

/// A layer to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerText {
    /// Pictures between tiles: one for each mix of corners listed.
    Corners {
        /// The terrains in the layer.
        of: Vec<String>,
        /// Each mix of corners that has a picture, and its tile.
        tiles: Vec<(u8, At)>,
    },
    /// A picture on each tile.
    Tiles {
        /// The terrains that get the picture.
        of: Vec<String>,
        /// The picture.
        at: At,
        /// The terrains a neighbour is looked at for.
        beside: Vec<String>,
        /// Each mix of sides with a picture of its own, and its tile.
        sides: Vec<(u8, At)>,
    },
}

/// `names` as a RON list of strings.
pub fn list(names: &[String]) -> String {
    let quoted: Vec<String> = names.iter().map(|n| format!("\"{n}\"")).collect();
    format!("[{}]", quoted.join(", "))
}

/// The lines of the pictures `mixes` of a layer, each a `kind` (`corners`
/// or `sides`), indented by `indent`.
fn mixes(kind: &str, mixes: &[(u8, At)], indent: &str) -> String {
    let mut out = String::new();
    for &(mix, (column, row)) in mixes {
        let name = mix_name(mix);
        // Writing to a `String` can't fail.
        let _ = writeln!(out, "{indent}({kind}: \"{name}\", at: ({column}, {row})),");
    }
    out
}

/// `layer` as written, each line indented by `indent`.
fn layer(layer: &LayerText, indent: &str) -> String {
    let inner = format!("{indent}    ");
    match layer {
        LayerText::Corners { of, tiles } => {
            let (of, tiles) = (list(of), mixes("corners", tiles, &inner));
            format!("{indent}Corners(of: {of}, tiles: [\n{tiles}{indent}]),\n")
        }
        LayerText::Tiles {
            of,
            at: (column, row),
            beside,
            sides,
        } => {
            let mut out = format!("{indent}Tiles(of: {}, at: ({column}, {row})", list(of));
            if beside.is_empty() && sides.is_empty() {
                out.push_str("),\n");
            } else {
                let (beside, sides) = (list(beside), mixes("sides", sides, &inner));
                let _ = writeln!(out, ", beside: {beside}, sides: [\n{sides}{indent}]),");
            }
            out
        }
    }
}

/// The `terrain` and `layers` fields of `look`, each line indented by
/// `indent`.
pub fn look(look: &LookText, indent: &str) -> String {
    let mut out = format!("{indent}terrain: {{\n");
    for (id, (column, row)) in &look.terrain {
        let _ = writeln!(out, "{indent}    \"{id}\": ({column}, {row}),");
    }
    let _ = writeln!(out, "{indent}}},\n{indent}layers: [");
    for l in &look.layers {
        out.push_str(&layer(l, &format!("{indent}    ")));
    }
    let _ = writeln!(out, "{indent}],");
    out
}

/// The `looks` field naming each of `named`; nothing for none.
pub fn looks(named: &[(String, LookText)]) -> String {
    if named.is_empty() {
        return String::new();
    }
    let mut out = String::from("    looks: {\n");
    for (name, text) in named {
        let _ = writeln!(out, "        \"{name}\": (");
        out.push_str(&look(text, "            "));
        out.push_str("        ),\n");
    }
    out.push_str("    },\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(names: &[&str]) -> Vec<String> {
        names.iter().map(|&n| n.to_owned()).collect()
    }

    fn sample() -> LookText {
        LookText {
            terrain: vec![("plain".to_owned(), (0, 0)), ("water".to_owned(), (3, 1))],
            layers: vec![
                LayerText::Corners {
                    of: names(&["water", "sea"]),
                    tiles: vec![(1, (4, 2)), (12, (5, 2))],
                },
                LayerText::Tiles {
                    of: names(&["fort"]),
                    at: (0, 4),
                    beside: vec![],
                    sides: vec![],
                },
                LayerText::Tiles {
                    of: names(&["bridge"]),
                    at: (1, 4),
                    beside: names(&["water"]),
                    sides: vec![(5, (2, 4))],
                },
            ],
        }
    }

    #[test]
    fn a_look_is_written_as_its_tiles_then_its_layers() {
        assert_eq!(
            look(&sample(), "    "),
            "    terrain: {\n        \"plain\": (0, 0),\n        \"water\": (3, 1),\n    },\n    \
             layers: [\n        Corners(of: [\"water\", \"sea\"], tiles: [\n            \
             (corners: \"...#\", at: (4, 2)),\n            (corners: \"##..\", at: (5, 2)),\n        \
             ]),\n        Tiles(of: [\"fort\"], at: (0, 4)),\n        \
             Tiles(of: [\"bridge\"], at: (1, 4), beside: [\"water\"], sides: [\n            \
             (sides: \".#.#\", at: (2, 4)),\n        ]),\n    ],\n"
        );
        // No layers: an empty list.
        let bare = LookText {
            terrain: vec![("plain".to_owned(), (7, 9))],
            layers: vec![],
        };
        assert_eq!(
            look(&bare, ""),
            "terrain: {\n    \"plain\": (7, 9),\n},\nlayers: [\n],\n"
        );
        assert_eq!(list(&[]), "[]");
        assert_eq!(list(&names(&["a", "b"])), "[\"a\", \"b\"]");
    }

    #[test]
    fn a_tile_layer_that_looks_at_neighbours_names_them_even_without_pictures() {
        let beside = LayerText::Tiles {
            of: names(&["bridge"]),
            at: (1, 4),
            beside: names(&["water"]),
            sides: vec![],
        };
        assert_eq!(
            layer(&beside, ""),
            "Tiles(of: [\"bridge\"], at: (1, 4), beside: [\"water\"], sides: [\n]),\n"
        );
    }

    #[test]
    fn other_looks_are_written_by_name_or_not_at_all() {
        assert_eq!(looks(&[]), "");
        let bare = LookText {
            terrain: vec![("plain".to_owned(), (0, 0))],
            layers: vec![],
        };
        assert_eq!(
            looks(&[("indoor".to_owned(), bare)]),
            "    looks: {\n        \"indoor\": (\n            terrain: {\n                \
             \"plain\": (0, 0),\n            },\n            layers: [\n            ],\n        \
             ),\n    },\n"
        );
    }
}
