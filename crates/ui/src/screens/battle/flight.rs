//! The leftovers leaving (ticket 0845): a battle won by defeating one unit
//! whose objective says the rest flee ([`Leftovers::Flee`]) ends with the
//! enemies still on the map running ([`Event::Fled`]). Before the victory
//! banner, each walks toward its nearest map edge at the map's walking
//! pace and fades out, all of them together, in one short moment
//! ([`FLEE_S`]); Confirm skips it.
//!
//! A look only (ADR-0038): the rules have already moved them off the map
//! ([`BattleState::fled`]). Until the moment is on screen (Harl's fall
//! still playing back, his EXP bar) they stand where they were.
//!
//! [`Leftovers::Flee`]: trpg_core::Leftovers::Flee
//! [`Event::Fled`]: trpg_core::Event::Fled

use crate::prelude::*;
use trpg_core::{BattleState, Pos, Unit, UnitId};

use super::walk;
use crate::map_view::scene::UnitView;

/// How long the leftovers take to leave, in seconds: under 2 s together
/// (ticket 0845). *Tunable.*
pub const FLEE_S: f32 = 1.5;

/// The enemies that fled, each with its way out, and how far the moment
/// has gone.
#[derive(Debug, Clone, PartialEq)]
pub struct Flight {
    units: Vec<(Unit, Vec<Pos>)>,
    t: f32,
}

impl Flight {
    /// The flight of `units`, those of `state`'s fled; `None` if none.
    pub fn new(state: &BattleState, units: &[UnitId]) -> Option<Self> {
        let tiles = &state.map().tiles;
        let size = (i32::from(tiles.width()), i32::from(tiles.height()));
        let units: Vec<_> = state
            .fled()
            .iter()
            .filter(|u| units.contains(&u.id))
            .map(|u| {
                let mut u = u.clone();
                // Running, not done: drawn as on the map in their phase.
                u.acted = false;
                let way = way_out(u.pos, size);
                (u, way)
            })
            .collect();
        (!units.is_empty()).then_some(Self { units, t: 0.0 })
    }

    /// Plays `dt` more seconds of it.
    pub fn tick(&mut self, dt: f32) {
        self.t += dt;
    }

    /// Whether everyone is gone.
    pub fn done(&self) -> bool {
        self.t >= FLEE_S
    }

    /// The units as drawn now, walking at `pace` tiles a second: on their
    /// way out, fading.
    pub fn views(&self, pace: f32) -> Vec<UnitView> {
        let fade = (self.t / FLEE_S).clamp(0.0, 1.0);
        let tiles = self.t * pace;
        self.units
            .iter()
            .map(|(unit, way)| {
                let view = UnitView::of(unit).fading(fade);
                let Some(gait) = walk::gait(way, tiles, pace) else {
                    // At the edge (or not started): standing at its end.
                    let at = if self.t > 0.0 {
                        way.last()
                    } else {
                        way.first()
                    };
                    return view.at(at.copied().unwrap_or(unit.pos));
                };
                let at = way[walk::steps(way.len(), tiles)];
                UnitView {
                    facing: gait.facing,
                    frame: gait.frame,
                    offset: gait.offset,
                    ..view.at(at)
                }
            })
            .collect()
    }
}

/// The tiles from `pos` straight to the nearest edge of a map of `size`
/// tiles: left, right, up, down when two are as near.
fn way_out(pos: Pos, (w, h): (i32, i32)) -> Vec<Pos> {
    let ways = [
        (pos.x, (-1, 0)),
        (w - 1 - pos.x, (1, 0)),
        (pos.y, (0, -1)),
        (h - 1 - pos.y, (0, 1)),
    ];
    let (len, (dx, dy)) = ways
        .into_iter()
        .min_by_key(|&(len, _)| len)
        .unwrap_or((0, (0, 0)));
    (0..=len.max(0))
        .map(|k| Pos::new(pos.x + k * dx, pos.y + k * dy))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_way_out_is_straight_to_the_nearest_edge() {
        let size = (10, 8);
        let way = way_out(Pos::new(7, 3), size);
        assert_eq!(way, [(7, 3), (8, 3), (9, 3)].map(|(x, y)| Pos::new(x, y)));
        assert_eq!(way_out(Pos::new(2, 6), size).last(), Some(&Pos::new(2, 7)));
        // On the edge already: nowhere to walk.
        assert_eq!(way_out(Pos::new(0, 4), size), [Pos::new(0, 4)]);
        // As near left as up: left.
        assert_eq!(way_out(Pos::new(1, 1), size).last(), Some(&Pos::new(0, 1)));
    }
}
