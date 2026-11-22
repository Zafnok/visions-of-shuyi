//! The battle's threat display (1291): the danger zone (0405), and the
//! threat arrows (0448) with the searches kept for them ([`ThreatPlanner`],
//! 1251; [`ThreatAreas`], 1259). [`ThreatDisplay`] keeps them; the battle
//! screen says which unit and tile the arrows are for.

use crate::prelude::*;
use core::cell::{RefCell, RefMut};

use trpg_core::{
    BattleState, Faction, Phase, Pos, ThreatAreas, ThreatPlanner, TileSet, Unit, UnitId,
    danger_zone, threats_at,
};

use super::BattleScreen;
use super::mode::Mode;
use crate::map_view::ThreatView;

/// The tiles units hostile to the player could attack this turn
/// ([`danger_zone`] with each unit's attack ranges); empty if it can't be
/// worked out.
pub fn danger_tiles(state: &BattleState) -> TileSet {
    let tiles = &state.map().tiles;
    danger_zone(
        state.map(),
        state.terrain(),
        state.classes(),
        state.units(),
        Faction::Player,
        |u| state.threat_ranges(u),
    )
    .unwrap_or_else(|_| TileSet::new(tiles.width(), tiles.height()))
}

/// Where the enemies stand that could attack unit `unit` in their next
/// phase if it stood on `tile` ([`threats_at`] with each unit's attack
/// ranges), in the battle's order; none if it can't be worked out.
pub fn threat_enemies(state: &BattleState, unit: UnitId, tile: Pos) -> Vec<Pos> {
    let ids = threats_at(
        state.map(),
        state.terrain(),
        state.classes(),
        state.units(),
        unit,
        tile,
        |u| state.threat_ranges(u),
    )
    .unwrap_or_default();
    enemy_tiles(state, ids)
}

/// Where the units `ids` stand, in that order.
fn enemy_tiles(state: &BattleState, ids: Vec<UnitId>) -> Vec<Pos> {
    let at = ids.into_iter().filter_map(|id| state.unit(id));
    at.map(|u| u.pos).collect()
}

/// The answer of [`threat_enemies`] for a unit on a tile, kept so that it
/// isn't worked out again every frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct KeptThreat {
    pub(super) unit: UnitId,
    pub(super) tile: Pos,
    pub(super) enemies: Vec<Pos>,
}

/// The danger zone and the threat arrows of one battle screen.
#[derive(Debug, Clone, Default)]
pub struct ThreatDisplay {
    /// The danger zone, while shown (recomputed after every command).
    danger: Option<TileSet>,
    /// The threat arrows last worked out (0448): the unit and the tile
    /// they were for, and where the enemies stand
    /// ([`threat_enemies`]). Kept until the scene asks about another unit
    /// or tile, and dropped whenever the battle changes.
    pub(super) threats: RefCell<Option<KeptThreat>>,
    /// The enemies' moves searched for the unit the threat arrows were
    /// last worked out for ([`ThreatPlanner`], ticket 1251), so another
    /// tile for it doesn't search them all again. Dropped with `threats`.
    planner: RefCell<Option<ThreatPlanner>>,
    /// Every enemy's threat area with every unit in place
    /// ([`ThreatAreas`], ticket 1259): the threat arrows of any unit on its
    /// own tile without a search. Dropped with `threats`.
    areas: RefCell<Option<ThreatAreas>>,
}

impl ThreatDisplay {
    /// The danger zone, while shown.
    pub fn danger(&self) -> Option<&TileSet> {
        self.danger.as_ref()
    }

    /// Shows or hides the danger zone of `state`.
    pub(super) fn toggle_danger(&mut self, state: &BattleState) {
        self.danger = match self.danger {
            Some(_) => None,
            None => Some(danger_tiles(state)),
        };
    }

    /// The battle is now `state` (a command, a rewind): the danger zone,
    /// if shown, is worked out again, and the kept arrows and searches
    /// are dropped.
    pub(super) fn battle_changed(&mut self, state: &BattleState) {
        if self.danger.is_some() {
            self.danger = Some(danger_tiles(state));
        }
        self.threats.take();
        self.planner.take();
        self.areas.take();
    }

    /// The threat arrows for `unit` on `tile` of `state`, if an enemy
    /// could attack it there. Worked out once for a unit and a tile, then
    /// kept until either changes or the battle does. With a unit
    /// `selected`, its planner is made at once, so the cursor's moves
    /// after cost almost nothing (1251).
    pub(super) fn arrows(
        &self,
        state: &BattleState,
        (unit, tile): (UnitId, Pos),
        selected: bool,
    ) -> Option<ThreatView> {
        if selected {
            self.kept_planner(state, unit);
        }
        let mut kept = self.threats.borrow_mut();
        let enemies = match &*kept {
            Some(k) if (k.unit, k.tile) == (unit, tile) => k.enemies.clone(),
            _ => {
                let enemies = self.planned_threats(state, unit, tile);
                *kept = Some(KeptThreat {
                    unit,
                    tile,
                    enemies: enemies.clone(),
                });
                enemies
            }
        };
        (!enemies.is_empty()).then_some(ThreatView { tile, enemies })
    }

    /// [`threat_enemies`] for `unit` on `tile`: on its own tile by the
    /// battle's threat areas, made now if none are kept for its side;
    /// elsewhere by the planner kept for `unit` ([`kept_planner`]).
    ///
    /// [`kept_planner`]: Self::kept_planner
    fn planned_threats(&self, state: &BattleState, unit: UnitId, tile: Pos) -> Vec<Pos> {
        let (map, terrain, classes, units) =
            (state.map(), state.terrain(), state.classes(), state.units());
        let ranges = |u: &Unit| state.threat_ranges(u);
        if let Some(standing) = state.unit(unit).filter(|u| u.pos == tile) {
            let mut kept = self.areas.borrow_mut();
            let mut ids = kept.as_ref().and_then(|a| a.threats_to(standing));
            if ids.is_none() {
                let faction = standing.faction;
                *kept = ThreatAreas::new(map, terrain, classes, units, faction, ranges).ok();
                ids = kept.as_ref().and_then(|a| a.threats_to(standing));
            }
            return enemy_tiles(state, ids.unwrap_or_default());
        }
        let kept = self.kept_planner(state, unit);
        let Some(planner) = kept.as_ref() else {
            return Vec::new();
        };
        let ids = planner.threats_at(map, terrain, classes, units, tile);
        enemy_tiles(state, ids.unwrap_or_default())
    }

    /// The planner kept for `unit`: made now if the one kept is for
    /// another unit, from the battle's threat areas if they are kept;
    /// `None` if it can't be made.
    fn kept_planner(&self, state: &BattleState, unit: UnitId) -> RefMut<'_, Option<ThreatPlanner>> {
        let (map, terrain, classes, units) =
            (state.map(), state.terrain(), state.classes(), state.units());
        let ranges = |u: &Unit| state.threat_ranges(u);
        let mut kept = self.planner.borrow_mut();
        if kept.as_ref().is_none_or(|p| p.unit() != unit) {
            *kept = match &*self.areas.borrow() {
                Some(areas) => {
                    ThreatPlanner::with_areas(map, terrain, classes, units, unit, ranges, areas)
                }
                None => ThreatPlanner::new(map, terrain, classes, units, unit, ranges),
            }
            .ok();
        }
        kept
    }
}

impl BattleScreen {
    /// The unit of the player's and the tile the threat arrows are for
    /// (`docs/design/turn-structure.md`, *Threat arrows*), in the player's
    /// phase of a battle still running:
    ///
    /// - browsing with the cursor on one of the player's units (one that
    ///   has acted too): that unit where it stands;
    /// - a unit selected and the cursor on a tile it can stop on, or on an
    ///   enemy the path is aimed at: the unit at its path's end;
    /// - the unit walking there, and then its action menu and the lists
    ///   that opens: the same. Not while a target is picked (the forecast).
    pub(super) fn threat_target(&self) -> Option<(UnitId, Pos)> {
        if self.state.phase() != Phase::Player || self.state.outcome().is_some() {
            return None;
        }
        match &self.mode {
            Mode::Idle { .. } => {
                let unit = self.hovered()?;
                (unit.faction == Faction::Player).then_some((unit.id, unit.pos))
            }
            Mode::Selected(sel) => {
                let dest = sel.dest();
                let pointed = self.cursor.pos == dest || sel.target.is_some();
                (pointed && sel.reach.is_stoppable(dest)).then_some((sel.unit, dest))
            }
            Mode::Moving { sel, .. } | Mode::ActionMenu { sel, .. } | Mode::List { sel, .. } => {
                Some((sel.unit, sel.dest()))
            }
            _ => None,
        }
    }

    /// The threat arrows to show: the tile [`threat_target`] gives and
    /// the enemies that could attack its unit there, if any can
    /// ([`ThreatDisplay::arrows`]).
    ///
    /// [`threat_target`]: Self::threat_target
    pub(super) fn threat_view(&self) -> Option<ThreatView> {
        let target = self.threat_target()?;
        let selected = !matches!(self.mode, Mode::Idle { .. });
        self.threat.arrows(&self.state, target, selected)
    }
}
