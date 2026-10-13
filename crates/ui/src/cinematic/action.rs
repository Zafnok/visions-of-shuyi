//! What the units of a map shot are doing at a moment of its action
//! (ticket 0827, ADR-0061): where each has walked to, who is lunging, who
//! was just struck and who has fallen. **Authored pictures, not a
//! battle**: nothing here runs the combat rules, and everything is worked
//! out from the action clock alone, so a shot can be cut into anywhere.
//!
//! This only says what is in the map scene; the map skins paint it
//! (ADR-0038).

use trpg_content::{Act, ActionDef};
use trpg_core::{Pos, StatValue, UnitId};

use crate::map_view::{Facing, STANDING_FRAME, UnitView};
use crate::screens::battle::walk;

/// How long a strike's lunge takes, there and back, in milliseconds. The
/// blow lands half way through. *Tunable.*
pub const LUNGE_MS: u32 = 400;

/// How far a striking unit leans toward its target at the moment the blow
/// lands, in tiles: a few pixels. *Tunable.*
pub const LUNGE_TILES: f32 = 0.25;

/// How long a struck unit flashes, in milliseconds. *Tunable.*
pub const FLASH_MS: u32 = 200;

/// How long a unit left with no HP takes to fade out, in milliseconds.
/// *Tunable.*
pub const FALL_MS: u32 = 500;

/// The units of a shot `clock_ms` milliseconds into `actions`, given how
/// they `start` (a battle's units where they stand as it begins):
///
/// - a unit whose walk has started is along its path, at an even speed
///   over the walk's time, turned the way it goes and stepping; once the
///   time is up it stands on the path's last tile;
/// - a unit that struck less than [`LUNGE_MS`] ago leans toward its
///   target, furthest ([`LUNGE_TILES`]) half way through, and faces it;
/// - from that half-way moment its target has the HP the strike left it
///   (a percentage of its full HP, rounded up, never more than it had)
///   and flashes for [`FLASH_MS`]; left with none, it fades out over
///   [`FALL_MS`] and is then not in the list.
///
/// An action by or at a unit `start` doesn't have changes nothing.
pub fn action_state(start: &[UnitView], actions: &[ActionDef], clock_ms: u64) -> Vec<UnitView> {
    let mut units = start.to_vec();
    // Each action that has started, with how long ago.
    let started = || {
        let since = |at_ms: u32| clock_ms.checked_sub(u64::from(at_ms));
        actions
            .iter()
            .filter_map(move |a| Some((since(a.at_ms)?, &a.act)))
    };
    for (since, act) in started() {
        if let Act::Walk { unit, path, ms } = act
            && let Some(unit) = units.iter_mut().find(|u| u.id == *unit)
        {
            walk_on(unit, path, *ms, since);
        }
    }
    // Everyone is where they walked to: a lunge goes toward its target.
    let places: Vec<(UnitId, Pos)> = units.iter().map(|u| (u.id, u.pos)).collect();
    let mut gone = Vec::new();
    for (since, act) in started() {
        let Act::Strike {
            unit,
            target,
            hp_left,
        } = act
        else {
            continue;
        };
        let place = |id: &UnitId| places.iter().find(|(unit, _)| unit == id);
        let (Some(_), Some(&(_, place))) = (place(unit), place(target)) else {
            continue;
        };
        for striker in units.iter_mut().filter(|u| u.id == *unit) {
            if since < u64::from(LUNGE_MS) {
                lunge(striker, place, since);
            }
        }
        let landed = since.checked_sub(u64::from(LUNGE_MS / 2));
        let struck = units.iter_mut().find(|u| u.id == *target);
        if let (Some(struck), Some(since)) = (struck, landed)
            && strike(struck, *hp_left, since)
        {
            gone.push(struck.id);
        }
    }
    units.retain(|u| !gone.contains(&u.id));
    units
}

/// `ms` (a time since something happened) as a part of `of`, from 0 to 1.
#[allow(clippy::cast_precision_loss)] // times of a shot: far below 2^24 ms
fn part(ms: u64, of: u32) -> f32 {
    (ms as f32 / of.max(1) as f32).min(1.0)
}

/// Puts `unit` where a walk along `path` over `ms` milliseconds has it
/// `since` milliseconds after its start.
fn walk_on(unit: &mut UnitView, path: &[Pos], ms: u32, since: u64) {
    let Some(&end) = path.last() else {
        return;
    };
    #[allow(clippy::cast_precision_loss)] // a path of a map's few tiles
    let legs = path.len().saturating_sub(1) as f32;
    let tiles = part(since, ms) * legs;
    #[allow(clippy::cast_precision_loss)] // as `part`
    let pace = legs * 1000.0 / ms.max(1) as f32;
    // No gait once it has arrived: it stands on the last tile.
    let gait = walk::gait(path, tiles, pace).filter(|_| since < u64::from(ms));
    let on = path.get(walk::steps(path.len(), tiles));
    unit.pos = gait.and(on).copied().unwrap_or(end);
    unit.facing = gait.map_or(Facing::Down, |g| g.facing);
    unit.frame = gait.map_or(STANDING_FRAME, |g| g.frame);
    unit.offset = gait.map_or((0.0, 0.0), |g| g.offset);
}

/// Leans `unit` toward the tile `target`, `since` milliseconds into its
/// lunge: out for the first half, back for the second. A unit in the
/// middle of a walk, or on its target's tile, doesn't lean.
fn lunge(unit: &mut UnitView, target: Pos, since: u64) {
    let (dx, dy) = (target.x - unit.pos.x, target.y - unit.pos.y);
    let far = dx.abs().max(dy.abs());
    if far == 0 || unit.between_tiles() {
        return;
    }
    let reach = LUNGE_TILES * (1.0 - (2.0 * part(since, LUNGE_MS) - 1.0).abs());
    #[allow(clippy::cast_precision_loss)] // tiles of a map
    let toward = |d: i32| d as f32 / far as f32 * reach;
    unit.offset = (toward(dx), toward(dy));
    unit.facing = walk::facing(unit.pos, target);
}

/// Shows `unit` as a blow that left it `hp_left` percent of its full HP
/// does `since` milliseconds after it landed. Returns whether the unit is
/// gone: it had no HP left and has faded out.
fn strike(unit: &mut UnitView, hp_left: u8, since: u64) -> bool {
    let (hp, full) = unit.hp;
    let left = (i64::from(full) * i64::from(hp_left) + 99).div_euclid(100);
    unit.hp.0 = hp.min(StatValue::try_from(left).unwrap_or(hp));
    unit.flash = 1.0 - part(since, FLASH_MS);
    if hp_left > 0 {
        return false;
    }
    unit.fade = part(since, FALL_MS);
    since >= u64::from(FALL_MS)
}

#[cfg(test)]
mod tests;
