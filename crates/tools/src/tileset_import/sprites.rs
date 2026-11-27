//! A `Sprites` layer (ticket 0460): small moving pictures on each tile of
//! a terrain at their own pixel size, Minifantasy's 8-pixel flames on a
//! burning wood. Each packed frame is a see-through tile with every sprite
//! drawn on it, each sprite on its own frame of the strip then; the frames
//! are a moving cell of the tileset like any other (ticket 1315), so the
//! skin shows them by the clock and stands them still with idle motion off.

use super::{Cut, LayerMap, LayerText, Packer, Sheet, TILE, draw_over, is_plain};

/// How a [`super::LayerMap::Sprites`] lays its sprites on a tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// A frame's size in pixels.
    pub frame: (u32, u32),
    /// How many frames the strip has.
    pub frames: u32,
    /// How long each shows, in ms.
    pub ms: u32,
    /// Each sprite: its top-left pixel in the tile and the frame it shows
    /// first, so sprites side by side don't flicker in step.
    pub at: Vec<(u32, u32, u32)>,
}

impl Layout {
    /// What is wrong with it that shows without the strip, if anything: too
    /// few frames, no time, no sprite, a sprite past the tile's edge (0452:
    /// nothing spills onto the next tile) or starting on a frame it lacks.
    pub fn problem(&self) -> Option<String> {
        let (w, h) = self.frame;
        if self.frames < 2 || self.ms == 0 || w == 0 || h == 0 {
            return Some("sprites need 2 frames or more, a size and a time".to_owned());
        }
        if self.at.is_empty() {
            return Some("no sprite on the tile".to_owned());
        }
        self.at.iter().find_map(|&(x, y, first)| {
            if x + w > TILE || y + h > TILE {
                Some(format!(
                    "a {w}×{h} px sprite at ({x}, {y}) spills out of the {TILE}×{TILE} tile"
                ))
            } else if first >= self.frames {
                Some(format!(
                    "the sprite at ({x}, {y}) starts on frame {first} of {}",
                    self.frames
                ))
            } else {
                None
            }
        })
    }
}

/// The tile of frame `k` of `layout`: each sprite's frame then from the
/// strip `sheet` (frames side by side, row by row from the top-left), over
/// a see-through tile. RGBA8, row by row.
pub fn compose(sheet: &Sheet, layout: &Layout, k: u32) -> Result<Vec<u8>, String> {
    let (fw, fh) = layout.frame;
    let (width, height) = sheet.size;
    let across = width / fw.max(1);
    let side = usize::try_from(TILE).unwrap_or(0);
    let mut out = vec![0; side * side * 4];
    for &(x, y, first) in &layout.at {
        let f = (first + k) % layout.frames.max(1);
        let (left, top) = ((f % across.max(1)) * fw, (f / across.max(1)) * fh);
        if across == 0 || top + fh > height {
            return Err(format!(
                "frame {f} of {fw}×{fh} px is outside the {width}×{height} px image"
            ));
        }
        for dy in 0..fh {
            for dx in 0..fw {
                let from = usize::try_from(((top + dy) * width + left + dx) * 4).unwrap_or(0);
                let to = usize::try_from(((y + dy) * TILE + x + dx) * 4).unwrap_or(0);
                let mut drawn = [0; 4];
                drawn.copy_from_slice(&sheet.rgba[from..from + 4]);
                if let Ok(pixel) = <&mut [u8; 4]>::try_from(&mut out[to..to + 4]) {
                    draw_over(pixel, drawn);
                }
            }
        }
    }
    Ok(out)
}

/// `layer`'s strip and layout, if it is a [`LayerMap::Sprites`].
fn of(layer: &LayerMap) -> Option<(&str, Layout)> {
    let LayerMap::Sprites {
        sheet,
        frame,
        frames,
        ms,
        at,
        ..
    } = layer
    else {
        return None;
    };
    let layout = Layout {
        frame: *frame,
        frames: *frames,
        ms: *ms,
        at: at.clone(),
    };
    Some((sheet, layout))
}

/// What is wrong with sprites layer `layer` that shows without its strip:
/// a sheet that isn't a plain one, or [`Layout::problem`].
pub(super) fn check(layer: &LayerMap) -> Result<(), String> {
    let (sheet, layout) = of(layer).ok_or("not a sprites layer")?;
    if !is_plain(sheet) {
        return Err("sprites come from a plain sheet (….png)".to_owned());
    }
    layout.problem().map_or(Ok(()), Err)
}

/// Sprites layer `layer` as the tileset file has it: a tile layer whose
/// picture is the sprites' first frame, packed with the rest by
/// [`Packer::sprite_frames`].
pub(super) fn layer(packer: &mut Packer, layer: &LayerMap) -> Result<LayerText, String> {
    let (LayerMap::Sprites { of, .. }, Some((sheet, layout))) = (layer, of(layer)) else {
        return Err("not a sprites layer".to_owned());
    };
    let layout = packer.layout(layout);
    let at = packer.packed(sheet, Cut::Sprites { layout, frame: 0 }, "")?;
    Ok(LayerText::Tiles {
        of: of.clone(),
        at,
        beside: Vec::new(),
        sides: Vec::new(),
    })
}

/// How the list describes sprites layer `layer`.
pub(super) fn describe(layer: &LayerMap) -> String {
    match (layer, of(layer)) {
        (LayerMap::Sprites { of: on, .. }, Some((sheet, layout))) => {
            let (w, h) = layout.frame;
            let n = layout.at.len();
            format!(
                "moving on {}: {sheet}, {n} sprites of {w}×{h} px",
                on.join(", ")
            )
        }
        _ => "(not sprites)".to_owned(),
    }
}
