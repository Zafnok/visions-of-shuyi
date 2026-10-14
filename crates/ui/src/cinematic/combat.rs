//! The combat shot (ticket 0832, ADR-0062): the battle's own combat scene
//! for one attack. The rules work the combat out
//! ([`trpg_content::cinematic::fight`]); its events make the same [`Playback`] a
//! battle makes, and [`stage::view_for`] says what the stage shows at a
//! moment of it, as it does in a battle with combat animations on. A
//! [`Duel`] keeps the playback at its start and plays a copy to the
//! moment asked: nothing is carried from one frame to the next
//! (ADR-0060 §2).

use trpg_content::CombatShot;
use trpg_core::{Event, LeadProfile};

use crate::screen::Ctx;
use crate::screens::battle::playback::{Playback, SceneCue, TIMINGS};
use crate::screens::battle::stage::{self, view::StageView};

/// A combat shot's fight, played back.
#[derive(Debug, Clone, PartialEq)]
pub struct Duel {
    /// The fight's playback, at its start.
    playback: Playback,
    /// The string id of the terrain the defender stands on, if known: the
    /// backdrop's.
    ground: Option<String>,
    /// The light of the battle's map.
    light: String,
}

impl Duel {
    /// The fight of `shot` with `lead` as the lead (its name; the
    /// player's language as it is now), or `None` if the rules allow it
    /// no longer (the content check refuses a file whose fight they don't
    /// allow) or its battle isn't there. The scenes the attack may set off
    /// are left out: a cinematic plays only its own shots.
    pub fn new(ctx: &Ctx, shot: &CombatShot, lead: &LeadProfile) -> Option<Self> {
        let def = ctx.content.battles.get(&shot.battle)?;
        let fight = trpg_content::cinematic::fight(&ctx.content, shot, lead.clone()).ok()?;
        let events: Vec<Event> = fight
            .events
            .into_iter()
            .filter(|e| SceneCue::of(e).is_none())
            .collect();
        let (before, fallen) = (&fight.before, fight.after.fallen());
        let playback = Playback::new(ctx.words(), &events, before, fallen, TIMINGS)?;
        let ground = playback
            .bouts()
            .first()
            .and_then(|bout| fight.after.map().tiles.get(bout.defender.pos))
            .and_then(|id| ctx.content.terrain.display.get(*id))
            .map(|terrain| terrain.id.clone());
        Some(Self {
            playback,
            ground,
            light: ctx.content.battle_look(def).light,
        })
    }

    /// How long the fight lasts at normal speed, in seconds: as long as
    /// it plays in a battle.
    pub fn seconds(&self) -> f32 {
        self.playback.total()
    }

    /// The stage `seconds` into the fight (at its end from then on), with
    /// `lead`'s picture for the lead.
    pub fn view(&self, ctx: &Ctx, lead: &LeadProfile, seconds: f32) -> Option<StageView> {
        let mut now = self.playback.clone();
        now.tick(seconds, false);
        stage::view_for(ctx, lead, &now, self.ground.as_deref(), &self.light)
    }
}
