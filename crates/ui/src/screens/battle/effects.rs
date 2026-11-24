//! What plays over the map as the battle goes on, apart from a combat's
//! [`playback`](super::playback) (1291): the heal and burn numbers over
//! units, the tiles flashing after their terrain changed, the sounds of
//! events no playback plays (0424), and the speeds the walks and the AI's
//! actions play at. [`Effects`] keeps them and ages them every frame.

use crate::prelude::*;

use trpg_core::{BattleState, Event, Pos, StatValue, UnitId};

use super::event_sounds::{self, CueQueue};
use super::mode::Mode;
use super::playback::TIMINGS;
use super::walk;
use crate::screen::Ctx;
use crate::settings::AnimSpeed;

/// How long a tile flashes when its terrain changes, in seconds. *Tunable.*
pub const TERRAIN_FLASH_S: f32 = 0.4;

/// A tile whose terrain just changed (a tile cast, a fire burning out): it
/// flashes for [`TERRAIN_FLASH_S`] seconds.
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainFlash {
    /// The tile.
    pub pos: Pos,
    /// Seconds it has flashed.
    pub t: f32,
}

impl TerrainFlash {
    /// How strong the flash is now: `1` at the change, fading to none.
    pub fn strength(&self) -> f32 {
        (1.0 - self.t / TERRAIN_FLASH_S).clamp(0.0, 1.0)
    }
}

/// What a [`HealPopup`] tells of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupKind {
    /// HP restored: `+10`, in `hp_high`.
    Heal,
    /// HP lost to a fire burning out under the unit: `-5`, in `hp_low`.
    Burn,
}

/// A `+10` shown over a unit that was healed, or a `-5` over one a fire
/// burnt, for [`Timings::heal_popup`](super::playback::Timings::heal_popup)
/// seconds (a burn's start once no banner is on screen).
#[derive(Debug, Clone, PartialEq)]
pub struct HealPopup {
    /// The tile it floats over.
    pub pos: Pos,
    /// HP restored or lost.
    pub amount: StatValue,
    /// Whether the HP was restored or lost.
    pub kind: PopupKind,
    /// Seconds it has been up.
    pub t: f32,
}

impl HealPopup {
    /// What it reads: `+10` for a heal, `-5` for a burn.
    pub fn text(&self) -> String {
        match self.kind {
            PopupKind::Heal => format!("+{}", self.amount),
            PopupKind::Burn => format!("-{}", self.amount),
        }
    }
}

/// The playback effects of one battle screen.
#[derive(Debug, Clone)]
pub struct Effects {
    /// Heal numbers floating over units.
    pub(super) popups: Vec<HealPopup>,
    /// Tiles flashing after their terrain changed.
    pub(super) flashes: Vec<TerrainFlash>,
    /// Sounds of events that no playback or walk plays (0424), waiting
    /// for their moment.
    pub(super) cues: CueQueue,
    /// The unit whose walk was shown since the last command: its move's
    /// steps have been heard.
    pub(super) walked: Option<UnitId>,
    /// Walking speed, in tiles per second, and an AI unit's while Confirm
    /// is held: the map skin's
    /// ([`walk`](shuyi_ui::grid::GridSkin::walk): its `normal` speed, or
    /// `fast` with Fast animations, and `held`), as of the frame's start.
    pub(super) pace: (f32, f32),
    /// How much faster the rest of an AI unit's action plays (the camera's
    /// pan, the mark on the unit): the player's speeds for the AI's phases
    /// ([`Settings::battle_speed`]), as of the frame's start.
    ///
    /// [`Settings::battle_speed`]: crate::settings::Settings::battle_speed
    pub(super) ai_speed: f32,
}

impl Default for Effects {
    fn default() -> Self {
        Self {
            popups: vec![],
            flashes: vec![],
            cues: CueQueue::default(),
            walked: None,
            pace: (walk::WALK_TILES_PER_S, walk::HELD_WALK_TILES_PER_S),
            ai_speed: 1.0,
        }
    }
}

impl Effects {
    /// Takes the walking speed and the AI's speed for the walks and the
    /// actions that start this frame, from the map skin and the player's
    /// settings as they are now.
    pub(super) fn take_speeds(&mut self, ctx: &Ctx) {
        let (skin, settings) = (&ctx.game.map_skin, ctx.settings());
        let walk = skin.walk();
        let normal = match settings.game.anim_speed {
            AnimSpeed::Normal => walk.normal,
            AnimSpeed::Fast => walk.fast,
        };
        self.pace = (normal, walk.held);
        self.ai_speed = settings.game.battle_speed(true);
    }

    /// Shows what a command's `events` did over the map of `state` (as it
    /// left it): a number over each unit healed or burnt, a flash on each
    /// tile whose terrain changed.
    pub(super) fn show_events(&mut self, events: &[Event], state: &BattleState) {
        self.popups.extend(events.iter().filter_map(|e| match *e {
            Event::Healed { target, amount } if amount > 0 => {
                let pos = state.unit(target)?.pos;
                Some(HealPopup {
                    pos,
                    amount,
                    kind: PopupKind::Heal,
                    t: 0.0,
                })
            }
            Event::BurnDamage { pos, amount, .. } if amount > 0 => Some(HealPopup {
                pos,
                amount,
                kind: PopupKind::Burn,
                t: 0.0,
            }),
            _ => None,
        }));
        self.flashes.extend(events.iter().filter_map(|e| match *e {
            Event::TerrainChanged { pos, .. } => Some(TerrainFlash { pos, t: 0.0 }),
            _ => None,
        }));
    }

    /// Ages the heal and burn numbers and the terrain flashes by `dt`
    /// seconds, dropping those whose time is up. A burn number waits while
    /// a `banner` is on screen: the fire burns out as the phase banner
    /// comes up, which would cover it for longer than it lasts.
    pub(super) fn tick(&mut self, dt: f32, banner: bool) {
        for p in &mut self.popups {
            if !(banner && p.kind == PopupKind::Burn) {
                p.t += dt;
            }
        }
        self.popups.retain(|p| p.t < TIMINGS.heal_popup);
        for f in &mut self.flashes {
            f.t += dt;
        }
        self.flashes.retain(|f| f.t < TERRAIN_FLASH_S);
    }

    /// Plays this frame's sounds (0424): `mode`'s combat playback's and
    /// walk's that the coming tick of `dt` seconds reaches (`held`:
    /// Confirm is down), and the queued events' that are due.
    pub(super) fn play_sounds(
        &mut self,
        ctx: &mut Ctx,
        (mode, state): (&Mode, &BattleState),
        dt: f32,
        held: bool,
    ) {
        let mut due = match mode {
            Mode::Combat(playback) => playback.sounds(dt, held),
            _ => Vec::new(),
        };
        if let Some((id, tiles)) = mode.tiles_entered(dt, held) {
            self.walked = Some(id);
            let step = state
                .unit(id)
                .and_then(|u| event_sounds::unit_step_sound(state, u));
            due.extend(step.into_iter().cycle().take(tiles));
        }
        due.extend(self.cues.tick(dt));
        for cue in due {
            ctx.audio.play_sound(cue);
        }
    }
}
