//! A conversation's moving opening (ticket 0734, ADR-0076): the characters
//! of an opening file on its place, each where its walks have it at a
//! moment. Like a map shot's action (ADR-0061), everything is worked out
//! from the opening's clock alone, and this only says what is in the map
//! scene: the map skins paint it (ADR-0038). The dialogue screen plays it
//! before the line the `@opening` is written before, then darkens the
//! place over [`DARKEN_MS`].

use crate::prelude::*;
use alloc::collections::BTreeSet;

use trpg_content::{Act, CastMember, OpeningDef};
use trpg_core::{ClassId, Faction, LEAD_ID, LeadProfile, default_map_label};

use super::action::{action_state, facing_of};
use crate::map_view::{STANDING_FRAME, UnitEffects, UnitView};
use crate::screen::Ctx;
use crate::screens::battle::walk;

/// How long the place takes to darken, once the opening's characters have
/// stopped, before the talk comes up: half a second
/// (`docs/design/look-and-feel.md`, *Places behind conversations*).
/// *Tunable.*
pub const DARKEN_MS: u32 = 500;

/// How long `def` plays, darkening included, in milliseconds.
pub fn opening_ms(def: &OpeningDef) -> u32 {
    def.length_ms.saturating_add(DARKEN_MS)
}

/// How far the place has darkened `clock_ms` into `def`: 0 while its
/// characters move, then up to 1 over [`DARKEN_MS`].
#[allow(clippy::cast_precision_loss)] // an opening is a few seconds
pub fn darkened(def: &OpeningDef, clock_ms: u64) -> f32 {
    let into = clock_ms.saturating_sub(u64::from(def.length_ms));
    (into as f32 / DARKEN_MS as f32).min(1.0)
}

/// The cast of `def` standing where it starts, as the map scene shows
/// them: by their character, so a skin with pictures draws their map
/// picture (the lead's by `lead`'s look), and the glyph skin their label
/// (the lead's from `lead`'s name); in the neutral colour, as they are on
/// no side here, and without an HP bar or marks.
pub fn cast_units(ctx: &Ctx, def: &OpeningDef, lead: &LeadProfile) -> Vec<UnitView> {
    def.cast.iter().map(|m| cast_unit(ctx, m, lead)).collect()
}

/// One of an opening's cast as [`cast_units`] has them.
fn cast_unit(ctx: &Ctx, member: &CastMember, lead: &LeadProfile) -> UnitView {
    let id = &member.character;
    let def = ctx.content.characters.characters.get(id);
    let label = if id.0 == LEAD_ID {
        // check-text: not a data name (the player's own)
        default_map_label(&lead.name)
    } else {
        let label = def.and_then(|d| d.map_label.clone());
        label.unwrap_or_else(|| {
            default_map_label(ctx.words().name(&ctx.content.names, id.0.as_str()))
        })
    };
    UnitView {
        id: member.unit,
        pos: member.at,
        faction: Faction::Neutral,
        label,
        class: def.map_or_else(|| ClassId(String::new()), |d| d.class.clone()),
        character: Some(id.clone()),
        acted: false,
        hp: (1, 1),
        effects: UnitEffects::default(),
        fade: 0.0,
        highlight: false,
        facing: facing_of(member.facing),
        frame: STANDING_FRAME,
        offset: (0.0, 0.0),
        flash: 0.0,
        marks: false,
    }
}

/// The cast of `def` `clock_ms` milliseconds into it, given how they
/// `start` ([`cast_units`]): each along its walk as a map shot's action
/// has it ([`action_state`]), turned the way it goes, or the way the walk
/// says it keeps facing. Once a walk is over its walker keeps facing the
/// way it last went (or the way the walk said), not the camera; someone
/// who hasn't walked faces the way the cast has them. After the end they
/// all stand where it left them.
pub fn opening_units(start: &[UnitView], def: &OpeningDef, clock_ms: u64) -> Vec<UnitView> {
    let mut units = action_state(start, &def.actions, clock_ms);
    // Each walker's last walk that has started says which way it faces.
    let mut decided = BTreeSet::new();
    for action in def.actions.iter().rev() {
        let Act::Walk {
            unit,
            path,
            ms,
            facing,
        } = &action.act
        else {
            continue;
        };
        let Some(since) = clock_ms.checked_sub(u64::from(action.at_ms)) else {
            continue;
        };
        if !decided.insert(*unit) || since < u64::from(*ms) {
            // A later walk decides; or it is walking, as the action has it.
            continue;
        }
        let last_leg = match path[..] {
            [.., from, to] => Some(walk::facing(from, to)),
            _ => None,
        };
        let way = facing.map(facing_of).or(last_leg);
        let walker = units.iter_mut().find(|u| u.id == *unit);
        if let (Some(walker), Some(way)) = (walker, way) {
            walker.facing = way;
        }
    }
    units
}

#[cfg(test)]
mod tests;
