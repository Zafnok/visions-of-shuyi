//! Plays a cinematic file (`assets/cinematics/`, ticket 0817, ADR-0060): a
//! [`Timeline`] says which shot a time falls in and how far through it,
//! and a [`CinematicPlayer`] draws that shot.
//!
//! **Every frame is drawn from the time alone**: nothing is carried from
//! one frame to the next. So whoever owns the clock can follow the music,
//! jump or loop (the title, 0819), and a test can draw any moment.

use std::collections::BTreeMap;
use std::rc::Rc;

use trpg_content::{CinematicDef, ImageId, Shot, battle_campaign};
use trpg_core::{BattleDef, GameMode, Pos};

use crate::cinema;
use crate::color::UiColor;
use crate::console::{CELL_H_PX, CELL_W_PX};
use crate::glyph_buffer::{Cell, GlyphBuffer, Rect};
use crate::map_view::{MapScene, MapSkin, UnitView};
use crate::screen::{Ctx, IdleMotion};
use crate::screens::title::glyph::paint_logo;
use crate::settings::MapZoom;

/// An area surely big enough for one tile of any skin, to ask its size.
const ROOMY: Rect = Rect::new(0, 0, 64, 64);

/// Where a time falls in a [`Timeline`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShotAt {
    /// Which shot, counted from 0.
    pub index: usize,
    /// How far through the shot: 0 at its start, up to (never reaching) 1
    /// at the next shot's start.
    pub progress: f32,
}

/// When each shot of a cinematic starts, and when the cinematic ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timeline {
    /// Each shot's start, in milliseconds, in order.
    starts_ms: Vec<u32>,
    /// The length, in milliseconds; at least 1.
    length_ms: u32,
}

impl Timeline {
    /// A timeline of shots starting at `starts_ms` (in order, the first at
    /// 0, as a loaded cinematic has them) over `length_ms` (0 counts as 1).
    pub fn new(starts_ms: Vec<u32>, length_ms: u32) -> Self {
        Self {
            starts_ms,
            length_ms: length_ms.max(1),
        }
    }

    /// The timeline of `def`'s shots.
    pub fn of(def: &CinematicDef) -> Self {
        Self::new(def.shots.iter().map(|s| s.at_ms).collect(), def.length_ms)
    }

    /// The length, in milliseconds.
    pub fn length_ms(&self) -> u32 {
        self.length_ms
    }

    /// `t_ms` wrapped at the length: the cinematic loops.
    pub fn wrapped_ms(&self, t_ms: u64) -> u32 {
        u32::try_from(t_ms % u64::from(self.length_ms)).unwrap_or(0)
    }

    /// The shot `t_ms` milliseconds after the start falls in, wrapped at
    /// the length. A shot lasts until the next one starts, the last until
    /// the end; time before the first shot's start is the first shot's.
    pub fn at_ms(&self, t_ms: u64) -> ShotAt {
        let t = self.wrapped_ms(t_ms);
        let index = self.starts_ms.iter().rposition(|&s| s <= t).unwrap_or(0);
        let start = self.starts_ms.get(index).map_or(0, |&s| s.min(t));
        let end = self.starts_ms.get(index + 1).map_or(self.length_ms, |&s| s);
        let span = end.saturating_sub(start).max(1);
        #[allow(clippy::cast_precision_loss)] // a shot is far below 2^24 ms
        let progress = ((t - start) as f32 / span as f32).clamp(0.0, 1.0);
        ShotAt { index, progress }
    }

    /// [`at_ms`](Self::at_ms) for `t` seconds ([`millis`]).
    pub fn at(&self, t: f32) -> ShotAt {
        self.at_ms(millis(t))
    }
}

/// `t` seconds in whole milliseconds. A time below 0, or one that isn't a
/// number, counts as 0.
pub fn millis(t: f32) -> u64 {
    let ms = (f64::from(t) * 1000.0).round();
    // The cast stops at 0 and at the biggest number by itself.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    if ms.is_finite() { ms as u64 } else { 0 }
}

/// `p` (0 to 1; clamped, not a number counts as 0) eased in and out: slow
/// at both ends, fastest in the middle.
pub fn ease_in_out(p: f32) -> f32 {
    let p = if p.is_finite() {
        p.clamp(0.0, 1.0)
    } else {
        0.0
    };
    p * p * (3.0 - 2.0 * p)
}

/// A whole map as the map skin painted it, to pan over.
#[derive(Debug, Clone, PartialEq)]
pub struct MapPicture {
    /// The picture: just big enough for the map.
    pub scene: Rc<GlyphBuffer>,
    /// One tile's size in the picture, in pixels.
    pub tile_px: (i32, i32),
}

impl MapPicture {
    /// All of `scene` (a view of a whole map, from its first tile) painted
    /// by `skin` into a buffer just big enough. `None` if the skin can't
    /// place the scene's first tile (a scene of no tiles).
    pub fn paint(ctx: &Ctx, skin: &dyn MapSkin, scene: &MapScene) -> Option<Self> {
        let tile = skin.tile_px(scene, ROOMY, scene.origin)?;
        let cells = |tiles: i32, tile_px: i32, cell_px: u16| {
            let cell_px = i32::from(cell_px);
            u16::try_from((tiles * tile_px + cell_px - 1) / cell_px).unwrap_or(0)
        };
        let (w, h) = (
            cells(scene.size.0, tile.w, CELL_W_PX),
            cells(scene.size.1, tile.h, CELL_H_PX),
        );
        let c = |u| ctx.palette.get(u);
        let blank = Cell::new(' ', c(UiColor::Text), c(UiColor::Black));
        let mut buf = GlyphBuffer::new(w, h, blank);
        skin.paint(ctx, scene, buf.bounds(), &mut buf);
        Some(Self {
            scene: Rc::new(buf),
            tile_px: (tile.w, tile.h),
        })
    }

    /// The picture's size in its own pixels.
    pub fn size_px(&self) -> (u32, u32) {
        let px = self.scene.pixel_bounds();
        (
            u32::try_from(px.w).unwrap_or(0),
            u32::try_from(px.h).unwrap_or(0),
        )
    }

    /// The picture's pixel at the map point `tiles` (a whole number is the
    /// middle of that tile, [`trpg_content::TilePoint`]).
    #[allow(clippy::cast_precision_loss)] // a tile is a few pixels
    pub fn point_px(&self, tiles: (f32, f32)) -> (f32, f32) {
        (
            (tiles.0 + 0.5) * self.tile_px.0 as f32,
            (tiles.1 + 0.5) * self.tile_px.1 as f32,
        )
    }
}

/// `ctx`'s map skin at the art's own size, whatever zoom the player has
/// the battle map at ([`MapZoom`]): a shot's zoom is then the only one.
pub fn shot_skin(ctx: &Ctx) -> Rc<dyn MapSkin> {
    let skin = &ctx.map_skin;
    skin.at_zoom(MapZoom::X1).unwrap_or_else(|| Rc::clone(skin))
}

/// The whole map of battle `def` with its units where they start: the
/// player's (as a battle played on its own has them, with `ctx`'s lead)
/// and the enemies. No cursor, ranges or reinforcements.
pub fn battle_scene(ctx: &Ctx, def: &BattleDef) -> MapScene {
    let tiles = &def.map.tiles;
    let size = (i32::from(tiles.width()), i32::from(tiles.height()));
    let mut scene = MapScene::new(Pos::new(0, 0), size);
    scene.set_terrain(|pos| tiles.get(pos).copied());
    scene.look = ctx.content.battle_look(def);
    let campaign = battle_campaign(&ctx.content, def, GameMode::Classic, ctx.lead.clone());
    for unit in &campaign.battle_setup(def, &ctx.content.tables()).units {
        scene.push_unit(UnitView::of(unit));
    }
    scene
}

/// A shot with what it needs to be drawn, built once.
#[derive(Debug, Clone, PartialEq)]
enum Built {
    /// A pan over `picture` (`None`: the battle couldn't be painted, and
    /// the shot is blank), the middle of the screen going from the
    /// picture's pixel `from` to `to`.
    MapPan {
        picture: Option<MapPicture>,
        from: (f32, f32),
        to: (f32, f32),
        zoom: u8,
    },
    /// The title's logo.
    Logo,
}

/// What a frame of a cinematic shows, as plain data: for tests of what
/// plays, and for [`CinematicPlayer::draw`] to paint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Frame {
    /// A map behind the whole screen.
    MapPan {
        /// The picture's pixel at the screen's top-left ([`cinema::view`]).
        origin_px: (f32, f32),
        /// Screen pixels per picture pixel.
        zoom: u8,
    },
    /// The logo over the title's backdrop.
    Logo {
        /// The sparks, as milliseconds since the cinematic's start; `None`
        /// when screens show no idle motion.
        sparks: Option<u32>,
    },
    /// Nothing: a shot whose map isn't there.
    Blank,
}

/// Draws a cinematic at any time.
#[derive(Debug, Clone, PartialEq)]
pub struct CinematicPlayer {
    timeline: Timeline,
    shots: Vec<Built>,
    /// The picture behind the logo.
    backdrop: Option<ImageId>,
}

impl CinematicPlayer {
    /// A player of the cinematic `id`, or `None` if the content has none
    /// of that name. Paints each battle its shots pan over once, with
    /// `ctx`'s map skin (at the art's own size, [`shot_skin`]), palette
    /// and lead as they are now. The logo is on
    /// no backdrop until [`with_backdrop`](Self::with_backdrop) gives one.
    pub fn new(ctx: &Ctx, id: &str) -> Option<Self> {
        let def = ctx.content.cinematics.get(id)?;
        let skin = shot_skin(ctx);
        let mut pictures: BTreeMap<&str, Option<MapPicture>> = BTreeMap::new();
        let shots = def.shots.iter().map(|s| match &s.shot {
            Shot::MapPan {
                battle,
                from,
                to,
                zoom,
            } => {
                let picture = pictures.entry(battle).or_insert_with(|| {
                    let def = ctx.content.battles.get(battle)?;
                    MapPicture::paint(ctx, skin.as_ref(), &battle_scene(ctx, def))
                });
                let px = |point: (f32, f32)| picture.as_ref().map_or(point, |p| p.point_px(point));
                Built::MapPan {
                    from: px(from.tiles()),
                    to: px(to.tiles()),
                    zoom: *zoom,
                    picture: picture.clone(),
                }
            }
            Shot::Logo => Built::Logo,
        });
        Some(Self {
            shots: shots.collect(),
            timeline: Timeline::of(def),
            backdrop: None,
        })
    }

    /// The same player with `backdrop` behind the logo: the picture the
    /// title picked for this launch, so both show the same one.
    #[must_use]
    pub fn with_backdrop(self, backdrop: Option<ImageId>) -> Self {
        Self { backdrop, ..self }
    }

    /// When its shots start and how long it is.
    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    /// What is shown `t` seconds after the start (wrapped at the length),
    /// on a screen of `screen` cells.
    pub fn frame(&self, ctx: &Ctx, t: f32, screen: Rect) -> Frame {
        let at = self.timeline.at(t);
        match self.shots.get(at.index) {
            Some(Built::MapPan {
                picture: Some(picture),
                from,
                to,
                zoom,
            }) => {
                let eased = ease_in_out(at.progress);
                let centre = (
                    from.0 + (to.0 - from.0) * eased,
                    from.1 + (to.1 - from.1) * eased,
                );
                let px =
                    |cells: i32, cell: u16| u32::try_from(cells).unwrap_or(0) * u32::from(cell);
                let clip = (px(screen.w, CELL_W_PX), px(screen.h, CELL_H_PX));
                Frame::MapPan {
                    origin_px: cinema::view(picture.size_px(), clip, centre, *zoom),
                    zoom: *zoom,
                }
            }
            Some(Built::Logo) => Frame::Logo {
                sparks: (ctx.idle_motion == IdleMotion::On)
                    .then(|| self.timeline.wrapped_ms(millis(t))),
            },
            Some(Built::MapPan { picture: None, .. }) | None => Frame::Blank,
        }
    }

    /// Draws the cinematic as it is `t` seconds after its start (wrapped
    /// at the length) over the whole of `buf`, at `brightness` (1 = as it
    /// is, 0 = black: [`GlyphBuffer::fade`]). The same `t` always draws
    /// the same frame.
    pub fn draw(&self, ctx: &Ctx, t: f32, brightness: f32, buf: &mut GlyphBuffer) {
        let c = |u| ctx.palette.get(u);
        let (text, black) = (c(UiColor::Text), c(UiColor::Black));
        let picture = match self.shots.get(self.timeline.at(t).index) {
            Some(Built::MapPan { picture, .. }) => picture.as_ref(),
            _ => None,
        };
        match (self.frame(ctx, t, buf.bounds()), picture) {
            (Frame::MapPan { origin_px, zoom }, Some(picture)) => {
                buf.fill_rect(buf.bounds(), Cell::see_through(' ', text));
                buf.set_backdrop(Rc::clone(&picture.scene), buf.bounds(), origin_px, zoom);
            }
            (Frame::Logo { sparks }, _) => paint_logo(ctx, self.backdrop, sparks, buf),
            _ => buf.fill_rect(buf.bounds(), Cell::new(' ', text, black)),
        }
        buf.fade(brightness);
    }
}

#[cfg(test)]
mod tests;
