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
    /// The pictures a map may place, in the order written: each its name,
    /// its top-left cell and its size in pixels (ticket 1378).
    pub props: Vec<(String, At, (u32, u32))>,
    /// The trees standing on its forest, if it has any (ticket 1356).
    pub trees: Option<TreesText>,
}

/// A look's trees to write (ticket 1356).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TreesText {
    /// The terrains they stand on.
    pub of: Vec<String>,
    /// The kinds of big tree.
    pub big: Vec<TreeKindText>,
    /// The kinds of half-size tree.
    pub half: Vec<TreeKindText>,
    /// What is still to make for the `Minimal` tier's still, if anything.
    pub minimal_todo: Option<String>,
}

/// A kind of tree to write: its frames' size, its foot and its frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeKindText {
    /// A frame's size in pixels.
    pub size: (u32, u32),
    /// The pixel of a frame on the middle of its trunk tile's bottom edge.
    pub foot: (u32, u32),
    /// Each frame's top-left cell and its time in ms.
    pub frames: Vec<(At, u32)>,
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
        /// Its pictures' size, offset and skipped terrains.
        place: Place,
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

/// Where a corner layer's pictures go (ticket 1361), as the format's
/// `size`, `offset` and `skip` write it: the default writes none of them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Place {
    /// Each picture's size in pixels; `None`, a tile's.
    pub size: Option<(u32, u32)>,
    /// Pixels across and down each picture is moved by.
    pub offset: (i32, i32),
    /// The terrains whose tiles get no part of a picture.
    pub skip: Vec<String>,
}

impl Place {
    /// The fields to write after a corner layer's `tiles`, each starting
    /// with `, `; nothing for the default.
    fn fields(&self) -> String {
        let mut out = String::new();
        if let Some((w, h)) = self.size {
            let _ = write!(out, ", size: ({w}, {h})");
        }
        if self.offset != (0, 0) {
            let (x, y) = self.offset;
            let _ = write!(out, ", offset: ({x}, {y})");
        }
        if !self.skip.is_empty() {
            let _ = write!(out, ", skip: {}", list(&self.skip));
        }
        out
    }
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
        LayerText::Corners { of, tiles, place } => {
            let (of, tiles) = (list(of), mixes("corners", tiles, &inner));
            let place = place.fields();
            format!("{indent}Corners(of: {of}, tiles: [\n{tiles}{indent}]{place}),\n")
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
    if !look.props.is_empty() {
        let _ = writeln!(out, "{indent}props: {{");
        for (name, (column, row), (w, h)) in &look.props {
            let _ = writeln!(
                out,
                "{indent}    \"{name}\": (at: ({column}, {row}), size: ({w}, {h})),"
            );
        }
        let _ = writeln!(out, "{indent}}},");
    }
    if let Some(trees) = &look.trees {
        out.push_str(&trees_field(trees, indent));
    }
    out
}

/// The `trees` field of a look, each line indented by `indent`, every
/// lesser tier showing a tree's first frame.
fn trees_field(trees: &TreesText, indent: &str) -> String {
    let kinds = |kinds: &[TreeKindText]| {
        let mut out = String::new();
        for TreeKindText {
            size: (w, h),
            foot: (fx, fy),
            frames,
        } in kinds
        {
            let shown: Vec<String> = frames
                .iter()
                .map(|((c, r), ms)| format!("(({c}, {r}), {ms})"))
                .collect();
            let _ = writeln!(
                out,
                "{indent}        (size: ({w}, {h}), foot: ({fx}, {fy}), frames: [{}]),",
                shown.join(", ")
            );
        }
        out
    };
    let todo = trees
        .minimal_todo
        .as_ref()
        .map_or_default(|t| format!(", todo: \"{t}\""));
    let stills = format!("{{Reduced: (show: FirstFrame), Minimal: (show: FirstFrame{todo})}}");
    format!(
        "{indent}trees: (\n{indent}    of: {},\n{indent}    big: [\n{}{indent}    ],\n\
         {indent}    half: [\n{}{indent}    ],\n{indent}    motion: (stills: {stills}),\n\
         {indent}),\n",
        list(&trees.of),
        kinds(&trees.big),
        kinds(&trees.half),
    )
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

/// A moving cell to write (ticket 1315): every row that draws `at`
/// loops through `frames`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FramesText {
    /// The cell, and its first frame.
    pub at: At,
    /// Each frame's cell and its time in ms, the first `at` itself.
    pub frames: Vec<(At, u32)>,
    /// What is still to make for the `Minimal` tier's still (a one-bit
    /// tile), if anything: the check lists it (ADR-0131).
    pub minimal_todo: Option<String>,
}

/// The `frames` field naming each of `moving`, every lesser tier showing
/// the first frame; nothing for none.
pub fn frames(moving: &[FramesText]) -> String {
    if moving.is_empty() {
        return String::new();
    }
    let mut out = String::from("    frames: [\n");
    for FramesText {
        at: (column, row),
        frames,
        minimal_todo,
    } in moving
    {
        let shown: Vec<String> = frames
            .iter()
            .map(|((c, r), ms)| format!("(({c}, {r}), {ms})"))
            .collect();
        let todo = minimal_todo
            .as_ref()
            .map_or_default(|t| format!(", todo: \"{t}\""));
        let stills = format!("{{Reduced: (show: FirstFrame), Minimal: (show: FirstFrame{todo})}}");
        // Writing to a `String` can't fail.
        let _ = writeln!(
            out,
            "        (at: ({column}, {row}), frames: [{}],\n         motion: (stills: {stills})),",
            shown.join(", ")
        );
    }
    out.push_str("    ],\n");
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
                    place: Place::default(),
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
            props: vec![],
            trees: None,
        }
    }

    #[test]
    fn a_looks_trees_are_written_after_its_layers_each_still_its_first_frame() {
        let kind = |size, foot, frames: Vec<(At, u32)>| TreeKindText { size, foot, frames };
        let with_trees = LookText {
            terrain: vec![],
            layers: vec![],
            props: vec![],
            trees: Some(TreesText {
                of: names(&["forest"]),
                big: vec![kind((64, 80), (32, 74), vec![((0, 9), 160), ((4, 9), 160)])],
                half: vec![kind(
                    (32, 40),
                    (16, 37),
                    vec![((0, 14), 160), ((2, 14), 160)],
                )],
                minimal_todo: Some("a one-bit tree".to_owned()),
            }),
        };
        assert_eq!(
            look(&with_trees, ""),
            "terrain: {\n},\nlayers: [\n],\ntrees: (\n    of: [\"forest\"],\n    big: [\n        \
             (size: (64, 80), foot: (32, 74), frames: [((0, 9), 160), ((4, 9), 160)]),\n    ],\n    \
             half: [\n        \
             (size: (32, 40), foot: (16, 37), frames: [((0, 14), 160), ((2, 14), 160)]),\n    ],\n    \
             motion: (stills: {Reduced: (show: FirstFrame), Minimal: (show: FirstFrame, \
             todo: \"a one-bit tree\")}),\n),\n"
        );
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
            props: vec![],
            trees: None,
        };
        assert_eq!(
            look(&bare, ""),
            "terrain: {\n    \"plain\": (7, 9),\n},\nlayers: [\n],\n"
        );
        assert_eq!(list(&[]), "[]");
        assert_eq!(list(&names(&["a", "b"])), "[\"a\", \"b\"]");
    }

    #[test]
    fn a_corner_layer_writes_where_its_pictures_go_only_if_it_says() {
        let corners = |place| LayerText::Corners {
            of: names(&["mountain"]),
            tiles: vec![(8, (0, 2))],
            place,
        };
        let tall = Place {
            size: Some((16, 24)),
            offset: (1, -3),
            skip: names(&["wall", "fort"]),
        };
        assert_eq!(
            layer(&corners(tall), ""),
            "Corners(of: [\"mountain\"], tiles: [\n    (corners: \"#...\", at: (0, 2)),\n], \
             size: (16, 24), offset: (1, -3), skip: [\"wall\", \"fort\"]),\n"
        );
        let moved = Place {
            offset: (0, 1),
            ..Place::default()
        };
        assert!(layer(&corners(moved), "").ends_with("], offset: (0, 1)),\n"));
        let skipping = Place {
            skip: names(&["wall"]),
            ..Place::default()
        };
        assert!(layer(&corners(skipping), "").ends_with("], skip: [\"wall\"]),\n"));
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
    fn moving_cells_are_written_with_their_stills_or_not_at_all() {
        assert_eq!(frames(&[]), "");
        let water = FramesText {
            at: (3, 1),
            frames: vec![((3, 1), 600), ((4, 1), 200)],
            minimal_todo: None,
        };
        let coast = FramesText {
            at: (5, 0),
            frames: vec![((5, 0), 100), ((6, 0), 100)],
            minimal_todo: Some("a one-bit tile".to_owned()),
        };
        assert_eq!(
            frames(&[water, coast]),
            [
                "    frames: [",
                "        (at: (3, 1), frames: [((3, 1), 600), ((4, 1), 200)],",
                "         motion: (stills: {Reduced: (show: FirstFrame), Minimal: (show: FirstFrame)})),",
                "        (at: (5, 0), frames: [((5, 0), 100), ((6, 0), 100)],",
                "         motion: (stills: {Reduced: (show: FirstFrame), \
                 Minimal: (show: FirstFrame, todo: \"a one-bit tile\")})),",
                "    ],",
                "",
            ]
            .join("\n")
        );
    }

    #[test]
    fn other_looks_are_written_by_name_or_not_at_all() {
        assert_eq!(looks(&[]), "");
        let bare = LookText {
            terrain: vec![("plain".to_owned(), (0, 0))],
            layers: vec![],
            props: vec![],
            trees: None,
        };
        assert_eq!(
            looks(&[("indoor".to_owned(), bare)]),
            "    looks: {\n        \"indoor\": (\n            terrain: {\n                \
             \"plain\": (0, 0),\n            },\n            layers: [\n            ],\n        \
             ),\n    },\n"
        );
    }
}
