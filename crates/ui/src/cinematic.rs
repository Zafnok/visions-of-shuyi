//! Plays a cinematic file (`assets/cinematics/`, ticket 0817, ADR-0060): a
//! [`Timeline`] says which shot a time falls in and how far through it,
//! and a [`CinematicPlayer`] draws that shot.
//!
//! The timeline, the easing, the pan and the player's skeleton are the
//! engine's (`shuyi_ui::cinematic`, ticket 1113); this game's shot kinds,
//! and how each is drawn ([`ShotPainter`]), are here.
//!
//! **Every frame is drawn from the time alone**: nothing is carried from
//! one frame to the next. So whoever owns the clock can follow the music,
//! jump or loop (the title, 0819), and a test can draw any moment.
//!
//! A map shot may have an action (ticket 0827, ADR-0061): its units walk
//! and strike. [`CinematicPlayer::scene`] is the map scene at a time, with
//! each unit as [`action_state`] has it; the map skin paints it.
//!
//! A combat shot (ticket 0832, ADR-0062) is the battle's own combat
//! scene for an attack the rules work out: [`CinematicPlayer::stage`] is
//! the stage at a time, and the combat scene's skin paints it.

use crate::prelude::*;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;

use shuyi_ui::cinematic::ShotPainter;
use trpg_content::{ImageId, Shot, ShotAction, Side, battle_campaign};
use trpg_core::{BattleDef, GameMode, LEAD_ID, LeadGender, LeadProfile, Pos};

use crate::color::UiColor;
use crate::glyph_buffer::{Cell, GlyphBuffer, Rect};
use crate::map_view::{MapScene, MapSkin, UnitView};
use crate::screen::{Ctx, IdleMotion, Shuyi};
use crate::screens::battle::stage::{self, view::StageView};
use crate::screens::title::glyph::paint_logo;
use crate::settings::MapZoom;

mod action;
pub mod combat;
pub mod faces;
pub mod opening;
pub mod talk;

pub use action::{FALL_MS, FLASH_MS, LUNGE_MS, LUNGE_TILES, action_state, facing_of};
use combat::Duel;
use faces::{Face, draw_faces, first_x};
pub use shuyi_ui::cinematic::{Moment, Pan, ShotAt, Timeline, ease_in_out, millis};
use talk::{TalkBox, talk_at};

/// An area surely big enough for one tile of any skin, to ask its size.
const ROOMY: Rect = Rect::new(0, 0, 64, 64);

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
        let console = ctx.console;
        let (w, h) = (
            cells(scene.size.0, tile.w, console.cell_w),
            cells(scene.size.1, tile.h, console.cell_h),
        );
        let c = |u| ctx.palette.get(u);
        let blank = Cell::new(' ', c(UiColor::Text), c(UiColor::Black));
        let mut buf = GlyphBuffer::with_spec(console.sized(w, h), blank);
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
    let skin = &ctx.game.map_skin;
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
    scene.look = ctx.game.content.battle_look(def);
    let campaign = battle_campaign(
        &ctx.game.content,
        def,
        GameMode::Classic,
        ctx.game.lead.clone(),
    );
    for unit in &campaign.battle_setup(def, &ctx.game.content.tables()).units {
        scene.push_unit(UnitView::of(unit));
    }
    scene
}

/// A battle's map as its shots show it.
#[derive(Debug, Clone, PartialEq)]
struct Stage {
    /// The map with its units where they start: what is painted.
    scene: Rc<MapScene>,
    /// That scene, painted once.
    picture: MapPicture,
}

/// A shot with what it needs to be drawn, built once.
#[derive(Debug, Clone, PartialEq)]
enum Built {
    /// A pan over `stage`'s picture (`None`: the battle couldn't be
    /// painted, and the shot is blank), in the picture's pixels. With an
    /// `action`, the units are as it has them at each moment, and the
    /// scene is painted anew each frame.
    MapPan {
        stage: Option<Stage>,
        pan: Pan,
        action: Option<ShotAction>,
    },
    /// The title's logo.
    Logo,
    /// Text boxes of a scene, one after another.
    Talk { boxes: Vec<TalkBox> },
    /// Faces crossing the screen from `from`.
    Characters {
        faces: Vec<Face>,
        expression: String,
        from: Side,
    },
    /// A fight on the combat stage (`None`: the rules allow it no longer,
    /// and the shot is blank).
    Combat { duel: Option<Duel> },
}

/// What a frame of a cinematic shows, as plain data: for tests of what
/// plays, and for [`CinematicPlayer::draw`] to paint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Frame {
    /// A map behind the whole screen.
    MapPan {
        /// The picture's pixel at the screen's top-left ([`Pan::origin_px`]).
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
    /// A text box of a scene in the dialogue screen's look.
    Talk {
        /// Which of the shot's boxes, counted from 0.
        index: usize,
        /// How many characters of its text are typed out.
        shown: usize,
    },
    /// Faces in frames with their names, side by side.
    Characters {
        /// The column of the first face's frame (off screen below 0 and
        /// from the screen's width on).
        x: i32,
        /// Columns from each face to the one after it; below 0 when they
        /// come from the left.
        step: i32,
    },
    /// The combat stage: a fight playing ([`CinematicPlayer::stage`]).
    Combat {
        /// Seconds into the fight: the time since the shot started, up to
        /// the fight's end, where it holds.
        seconds: f32,
    },
    /// Nothing: a shot whose map or scene isn't there.
    Blank,
}

/// What every shot of a [`CinematicPlayer`] is drawn with.
#[derive(Debug, Clone, PartialEq)]
struct Cast {
    /// The picture behind the logo.
    backdrop: Option<ImageId>,
    /// The lead the talk, characters and combat shots show.
    lead: LeadProfile,
}

impl Built {
    /// The map scene of a map shot `into_ms` milliseconds after it
    /// started ([`CinematicPlayer::scene`]); `None` for another shot.
    fn scene(&self, into_ms: u32) -> Option<MapScene> {
        let Built::MapPan {
            stage: Some(stage),
            action,
            ..
        } = self
        else {
            return None;
        };
        let mut scene = MapScene::clone(&stage.scene);
        if let Some(action) = action {
            let clock_ms = u64::from(action.from_ms) + u64::from(into_ms);
            scene.units = action_state(&scene.units, &action.actions, clock_ms);
        }
        Some(scene)
    }
}

impl ShotPainter for Built {
    type Game = Shuyi;
    type Shared = Cast;
    type Frame = Frame;

    fn frame(&self, ctx: &Ctx, _: &Cast, moment: Moment, screen: Rect) -> Frame {
        match self {
            Built::MapPan {
                stage: Some(Stage { picture, .. }),
                pan,
                ..
            } => Frame::MapPan {
                origin_px: pan.origin_px(ctx.console, picture.size_px(), screen, moment.progress),
                zoom: pan.zoom,
            },
            Built::Logo => Frame::Logo {
                sparks: (ctx.idle_motion == IdleMotion::On).then_some(moment.t_ms),
            },
            Built::Talk { boxes } => {
                #[allow(clippy::cast_precision_loss)] // a shot is far below 2^24 ms
                let shot_s = moment.shot_ms as f32 / 1000.0;
                let lens: Vec<usize> = boxes.iter().map(TalkBox::chars).collect();
                let speed = ctx.settings().text_speed.chars_per_s();
                match talk_at(moment.progress, shot_s, &lens, speed) {
                    Some((index, shown)) => Frame::Talk { index, shown },
                    None => Frame::Blank,
                }
            }
            Built::Characters { faces, from, .. } if !faces.is_empty() => {
                let (x, step) = first_x(moment.progress, faces.len(), *from, screen.w);
                Frame::Characters { x, step }
            }
            Built::Combat { duel: Some(duel) } => {
                #[allow(clippy::cast_precision_loss)] // a shot is far below 2^24 ms
                let into = moment.into_ms as f32 / 1000.0;
                Frame::Combat {
                    seconds: into.min(duel.seconds()),
                }
            }
            Built::MapPan { stage: None, .. }
            | Built::Characters { .. }
            | Built::Combat { duel: None } => Frame::Blank,
        }
    }

    fn paint(&self, ctx: &Ctx, cast: &Cast, moment: Moment, frame: Frame, buf: &mut GlyphBuffer) {
        match (frame, self) {
            (
                Frame::MapPan { origin_px, zoom },
                Built::MapPan {
                    stage: Some(stage),
                    action,
                    ..
                },
            ) => {
                // Units that stand were painted once; units that act are
                // painted as they are now.
                let acted = action.as_ref().and_then(|_| self.scene(moment.into_ms));
                let skin = shot_skin(ctx);
                let now = acted.and_then(|scene| MapPicture::paint(ctx, skin.as_ref(), &scene));
                let picture = now.map_or_else(|| Rc::clone(&stage.picture.scene), |p| p.scene);
                let text = ctx.palette.get(UiColor::Text);
                buf.fill_rect(buf.bounds(), Cell::see_through(' ', text));
                buf.set_backdrop(picture, buf.bounds(), origin_px, zoom);
            }
            (Frame::Logo { sparks }, _) => paint_logo(ctx, cast.backdrop, sparks, buf),
            (Frame::Talk { index, shown }, Built::Talk { boxes }) => {
                if let Some(talk) = boxes.get(index) {
                    talk.draw(ctx, &cast.lead, shown, buf);
                }
            }
            (
                Frame::Characters { x, step },
                Built::Characters {
                    faces, expression, ..
                },
            ) => {
                draw_faces(ctx, buf, faces, expression, (x, step));
            }
            (Frame::Combat { seconds }, Built::Combat { duel: Some(duel) }) => {
                if let Some(view) = duel.view(ctx, &cast.lead, seconds) {
                    stage::glyph::paint(ctx, &view, buf);
                }
            }
            _ => {}
        }
    }
}

/// Draws a cinematic at any time: the engine's player of this game's
/// shots, with the backdrop behind its logo and the lead its shots show.
#[derive(Debug, Clone, PartialEq)]
pub struct CinematicPlayer {
    player: shuyi_ui::cinematic::CinematicPlayer<Built>,
    cast: Cast,
}

impl CinematicPlayer {
    /// A player of the cinematic `id`, or `None` if the content has none
    /// of that name. Paints each battle its shots pan over once, with
    /// `ctx`'s map skin (at the art's own size, [`shot_skin`]), palette
    /// and lead as they are now. (A shot with an action is painted again
    /// each frame, by the skin `ctx` has then: make the player again
    /// after a change of skin.) The logo is on
    /// no backdrop until [`with_backdrop`](Self::with_backdrop) gives one.
    ///
    /// `lead` is who the talk, characters and combat shots show as the
    /// lead, by name and look: the most recent save's
    /// ([`save::latest_lead`](crate::save::latest_lead)). `None`: the
    /// names table's name for the lead, and the male look
    /// (`docs/design/title-screen.md`, *Intro cinematic*). Their scenes,
    /// and the names on the combat stage, are in the player's language as
    /// it is now. Each combat shot's fight is worked out by the rules
    /// once, here.
    pub fn new(ctx: &Ctx, id: &str, lead: Option<LeadProfile>) -> Option<Self> {
        let def = ctx.game.content.cinematics.get(id)?;
        let lead = lead.unwrap_or_else(|| {
            let name = ctx.words().name(&ctx.game.content.names, LEAD_ID);
            LeadProfile::new(name, LeadGender::Male)
        });
        let skin = shot_skin(ctx);
        let mut stages: BTreeMap<&str, Option<Stage>> = BTreeMap::new();
        let player = shuyi_ui::cinematic::CinematicPlayer::of(def, |shot| match shot {
            Shot::MapPan {
                battle,
                from,
                to,
                zoom,
                action,
            } => {
                let stage = stages.entry(battle).or_insert_with(|| {
                    let scene = battle_scene(ctx, ctx.game.content.battles.get(battle)?);
                    let picture = MapPicture::paint(ctx, skin.as_ref(), &scene)?;
                    let scene = Rc::new(scene);
                    Some(Stage { scene, picture })
                });
                let picture = stage.as_ref().map(|s| &s.picture);
                let px = |point: (f32, f32)| picture.map_or(point, |p| p.point_px(point));
                Built::MapPan {
                    pan: Pan {
                        from: px(from.tiles()),
                        to: px(to.tiles()),
                        zoom: *zoom,
                    },
                    stage: stage.clone(),
                    action: action.clone(),
                }
            }
            Shot::Logo => Built::Logo,
            Shot::Talk {
                scene,
                first,
                count,
            } => {
                let scene = ctx.game.content.dialogue.get(scene);
                let boxes = scene.map(|s| talk::boxes(ctx, s, &lead, *first, *count));
                Built::Talk {
                    boxes: boxes.unwrap_or_default(),
                }
            }
            Shot::Characters {
                who,
                expression,
                from,
            } => Built::Characters {
                faces: faces::faces(ctx, who, &lead),
                expression: expression.clone(),
                from: *from,
            },
            Shot::Combat(shot) => Built::Combat {
                duel: Duel::new(ctx, shot, &lead),
            },
        });
        Some(Self {
            player,
            cast: Cast {
                backdrop: None,
                lead,
            },
        })
    }

    /// The same player with `backdrop` behind the logo: the picture the
    /// title picked for this launch, so both show the same one.
    #[must_use]
    pub fn with_backdrop(mut self, backdrop: Option<ImageId>) -> Self {
        self.cast.backdrop = backdrop;
        self
    }

    /// The lead its talk and characters shots show.
    pub fn lead(&self) -> &LeadProfile {
        &self.cast.lead
    }

    /// The text boxes of shot `index`, if it is a talk shot.
    pub fn talk_boxes(&self, index: usize) -> Option<&[TalkBox]> {
        match self.player.shot(index) {
            Some(Built::Talk { boxes }) => Some(boxes),
            _ => None,
        }
    }

    /// The faces of shot `index`, if it is a characters shot.
    pub fn faces(&self, index: usize) -> Option<&[Face]> {
        match self.player.shot(index) {
            Some(Built::Characters { faces, .. }) => Some(faces),
            _ => None,
        }
    }

    /// When its shots start and how long it is.
    pub fn timeline(&self) -> &Timeline {
        self.player.timeline()
    }

    /// What is shown `t` seconds after the start (wrapped at the length),
    /// on a screen of `screen` cells.
    pub fn frame(&self, ctx: &Ctx, t: f32, screen: Rect) -> Frame {
        let frame = self.player.frame(ctx, &self.cast, t, screen);
        frame.unwrap_or(Frame::Blank)
    }

    /// The map scene shown `t` seconds after the start (wrapped at the
    /// length): the battle's whole map and its units, each where the
    /// shot's action has it by then ([`action_state`], on the action's
    /// clock: the shot's `action_from` plus the time since the shot
    /// started), or where it starts in a shot with no action. `None` when
    /// the shot isn't of a map.
    pub fn scene(&self, t: f32) -> Option<MapScene> {
        let (shot, moment) = self.player.at(t)?;
        shot.scene(moment.into_ms)
    }

    /// The combat stage shown `t` seconds after the start (wrapped at the
    /// length): the fight of the shot then, as far into it as the shot
    /// has played (at its end from then on), as a battle shows it with
    /// combat animations on. `None` when the shot isn't a fight.
    pub fn stage(&self, ctx: &Ctx, t: f32) -> Option<StageView> {
        let Frame::Combat { seconds } = self.frame(ctx, t, Rect::new(0, 0, 0, 0)) else {
            return None;
        };
        let Some((Built::Combat { duel: Some(duel) }, _)) = self.player.at(t) else {
            return None;
        };
        duel.view(ctx, &self.cast.lead, seconds)
    }

    /// Draws the cinematic as it is `t` seconds after its start (wrapped
    /// at the length) over the whole of `buf`, at `brightness` (1 = as it
    /// is, 0 = black: [`GlyphBuffer::fade`]). The same `t` always draws
    /// the same frame.
    pub fn draw(&self, ctx: &Ctx, t: f32, brightness: f32, buf: &mut GlyphBuffer) {
        self.player.draw(ctx, &self.cast, t, brightness, buf);
    }
}

#[cfg(test)]
mod combat_tests;
#[cfg(test)]
mod shot_tests;
#[cfg(test)]
mod tests;
