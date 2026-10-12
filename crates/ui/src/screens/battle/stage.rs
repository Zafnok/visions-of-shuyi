//! The combat stage (ticket 0413, ADR-0056): while a combat plays back
//! with combat animations on, the map makes way for the two fighters'
//! full-body pictures in front of a backdrop, moved by the game. This
//! module says what the stage shows ([`view()`]); [`motion`] is how the
//! pictures move and [`glyph`] paints it (ADR-0054).

use trpg_content::{Battler, Light};
use trpg_core::{Affinity, Faction, Side, SideForecast, Strike};

use self::motion::{Blow, Pose};
use self::view::{FighterView, Numbers, StageView};
use super::playback::{Beat, Bout, Fighter, Playback, result_text};
use crate::screen::Ctx;

pub mod glyph;
pub mod motion;
pub mod view;

/// What `strike` did to its target.
fn blow(strike: &Strike) -> Blow {
    match (strike.hit, strike.healed, strike.crit) {
        (false, _, _) => Blow::Miss,
        (true, true, _) => Blow::Heal,
        (true, false, true) => Blow::Critical,
        (true, false, false) => Blow::Hit,
    }
}

/// The index in [`StageView::fighters`] of the fighter on `side`.
fn index(side: Side) -> usize {
    match side {
        Side::Attacker => 0,
        Side::Defender => 1,
    }
}

/// Which fighter stands on the left: the player's own, else the attacker.
fn left(bout: &Bout) -> usize {
    let players = |f: &Fighter| f.faction == Faction::Player;
    usize::from(players(&bout.defender) && !players(&bout.attacker))
}

/// The picture of `fighter`: its character's (the lead's by gender
/// first), else its class's, else the default one.
fn picture(ctx: &Ctx, fighter: &Fighter) -> Option<Battler> {
    let character = fighter.character.as_ref().map(|c| c.0.as_str());
    let by_gender = character.map(|c| ctx.lead.portrait_for(c));
    let ids = [by_gender, character, Some(fighter.class.0.as_str())];
    ctx.content
        .battlers
        .pick(ids.into_iter().flatten())
        .cloned()
}

fn numbers(n: &SideForecast) -> Numbers {
    Numbers {
        damage: n.damage,
        heals: n.affinity == Some(Affinity::Absorb),
        hit: n.hit,
        crit: n.crit,
    }
}

/// The two fighters' poses (attacker, defender), the striker and whether
/// the stage flashes, at `beat`, `local` seconds into a step `len` long.
fn poses(bout: &Bout, beat: Beat, local: f32, len: f32) -> ([Pose; 2], Option<usize>, bool) {
    let mut poses = [Pose::default(); 2];
    match beat {
        Beat::Flash { strike, .. } => {
            let Some(s) = bout.strikes.get(strike) else {
                return (poses, None, false);
            };
            let by = index(s.by);
            let critical = blow(s) == Blow::Critical;
            poses[by].lunge = motion::run_up(local / len.max(f32::EPSILON), critical);
            (poses, Some(by), false)
        }
        Beat::Result { strike, .. } => {
            let Some(s) = bout.strikes.get(strike) else {
                return (poses, None, false);
            };
            let by = index(s.by);
            poses[by].lunge = motion::go_back(local);
            poses[1 - by] = motion::struck(local, blow(s));
            (poses, Some(by), motion::veil(local, blow(s)))
        }
        _ => (poses, None, false),
    }
}

/// What the stage shows for playback `pb` now, or `None` if it has no
/// combat to show. `ground` is the string id of the terrain the defender
/// stands on, if known, and `light` the name of the light the map is in:
/// they pick the backdrop, the light's own picture if the backdrops have
/// one, else noon's shaded by the light.
pub fn view(ctx: &Ctx, pb: &Playback, ground: Option<&str>, light: &str) -> Option<StageView> {
    let (bout, (step, local)) = (pb.bout()?, pb.now()?);
    let words = ctx.words();
    let (poses, striker, veil) = poses(bout, step.beat, local, step.len);
    let lit = match step.beat {
        Beat::Flash { .. } => {
            // Whole half-periods since the flash began: on, off, on, …
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let phase = (local / pb.blink().max(f32::EPSILON)) as u32;
            striker.filter(|_| phase.is_multiple_of(2))
        }
        _ => None,
    };
    let result = match step.beat {
        Beat::Result { strike, .. } | Beat::Gap { strike, .. } => bout.strikes.get(strike),
        _ => None,
    };
    let fighter = |i: usize, f: &Fighter, pose: Pose| FighterView {
        // check-text: not a data name (the playback's own, already in the
        // player's language)
        name: f.name.clone(),
        faction: f.faction,
        hp: pb.hp(f.unit).unwrap_or(f.start),
        max: f.max,
        lit: lit == Some(i),
        numbers: bout.numbers[i].as_ref().map(numbers),
        picture: picture(ctx, f),
        pose: Pose {
            fade: pb.fade(f.unit).unwrap_or(0.0),
            ..pose
        },
    };
    let backdrop = ctx.content.battlers.backdrops.in_light(ground, light);
    let shade = match backdrop {
        Some((_, false)) => ctx.content.lighting.get(light).unwrap_or_default(),
        _ => Light::PLAIN,
    };
    Some(StageView {
        backdrop: backdrop.map(|(image, _)| image),
        shade,
        banner: pb.banner().map(str::to_owned),
        fighters: [
            fighter(0, &bout.attacker, poses[0]),
            fighter(1, &bout.defender, poses[1]),
        ],
        left: left(bout),
        melee: bout.melee(),
        striker,
        result: result.map(|s| (1 - index(s.by), result_text(words, s))),
        veil,
    })
}

#[cfg(test)]
mod tests;
