//! Movement: where a unit can go, the path it takes, and what it can attack.
//! Powers the blue move overlay, the red attack overlay, the path arrow, the
//! enemy danger zone and the AI.
//!
//! # Rules
//!
//! - **Budget.** A unit has its Mov (`stats.mov`, set by its class, changed
//!   by its timed effects: [`Unit::move_points`]) in move points per move. A
//!   negative Mov counts as 0.
//! - **Step cost.** Entering a tile costs the terrain's `move_cost` for the
//!   mover's class movement type (`docs/design/terrain.md`). `None` means the
//!   tile can't be entered. The start tile costs nothing. Only the 4 grid
//!   directions are steps.
//! - **Units.** A unit can move *through* tiles holding units that are not
//!   hostile to it ([`Faction::is_hostile_to`]) but never *into* a tile
//!   holding a hostile unit. It can't *end* its move on a tile holding another
//!   unit. So [`Reach`] has two sets: *passable* tiles (reached within budget,
//!   with their cheapest cost) and *stoppable* tiles (passable and empty,
//!   plus the start tile, always).
//! - **Blocked tiles.** A tile of the map's
//!   [`blocked`](BattleMap::blocked) set (a downed unit lies there) can't
//!   be entered by anyone, as if impassable.
//! - **Stopping tiles** (Hold the Ford, `combat-arts.md`): a tile next to
//!   a unit hostile to the mover that holds a stance which
//!   [stops enemies](crate::skill::TimedMods::stops_enemies) ends the move
//!   of a mover stepping onto it: it can stop there (if the tile is empty)
//!   but goes no further. Fliers are stopped too. A mover that starts on
//!   such a tile may move away freely (only stepping *onto* one stops it).
//!   An occupied stopping tile isn't passable: the mover could neither stop
//!   there nor go on.
//! - **No Canto, split movement, other zones of control or flying over
//!   units** (`docs/design/turn-structure.md`: FE standard only). Movement
//!   skills are separate, later rules.
//! - **Determinism.** The engine's Dijkstra ([`shuyi_core::path`]), with the
//!   priority key `(cost, y, x)`; neighbours are visited in
//!   [`Dir::ALL`](crate::Dir::ALL) order and a tile's recorded predecessor
//!   only changes for a strictly cheaper cost. So among equally cheap paths
//!   the one found first (smallest key) wins, every time.
//! - **Attack tiles** ([`attack_tiles`]): every tile at Manhattan distance
//!   `min..=max` from any stoppable tile, except the stoppable tiles
//!   themselves (the red overlay drawn around the blue one, as in Fire
//!   Emblem). Walls and other impassable tiles are included: range ignores
//!   terrain.
//! - **Threat** ([`threat_area`]): stoppable tiles plus attack tiles for each
//!   of the unit's weapon ranges. A unit with no ranges (no usable weapon)
//!   threatens nothing. The **danger zone** ([`danger_zone`]) is the union of
//!   the threat areas of every unit hostile to a faction.
//! - **Threats to a tile** ([`threats_at`]): the units hostile to a unit
//!   that could attack it next phase if it stood on a given tile: the tile is
//!   in their threat area, worked out with the unit standing there. A unit
//!   whose AI never moves ([`AiBehavior::Stationary`]) only counts the tiles
//!   its ranges reach from where it stands.
//! - Every unit in the `units` slice occupies its tile; callers pass the
//!   living units of the battle.

use crate::prelude::*;

use core::fmt;

use shuyi_core::path;

use crate::ai::AiBehavior;
use crate::class::{ClassId, ClassTable};
use crate::geom::{Dir, Pos};
use crate::map::BattleMap;
use crate::terrain::{MovementTypeId, TerrainTable};
use crate::unit::{Faction, Unit, UnitId};

pub use shuyi_core::grid::TileSet;

/// A weapon range: `(min, max)` Manhattan distance, inclusive.
pub type AttackRange = (u32, u32);

/// Why movement could not be worked out for a unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveError {
    /// No unit with this id is in the `units` slice.
    UnknownUnit(UnitId),
    /// The unit's class is not in the class table.
    UnknownClass(ClassId),
    /// The unit stands outside the map.
    OffMap(UnitId),
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveError::UnknownUnit(id) => write!(f, "no unit with id {}", id.0),
            MoveError::UnknownClass(c) => write!(f, "unknown class \"{}\"", c.0),
            MoveError::OffMap(id) => write!(f, "unit {} is outside the map", id.0),
        }
    }
}

impl core::error::Error for MoveError {}

/// Why a drawn path is not a legal move. `index` is the position in the path
/// of the offending tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    /// The mover itself is invalid.
    Mover(MoveError),
    /// The path has no tiles.
    Empty,
    /// The path does not start on the mover's tile.
    WrongStart,
    /// A step is not to a 4-way neighbour.
    NotAdjacent {
        /// Index of the tile stepped to.
        index: usize,
    },
    /// A step enters a tile the mover can't enter (or leaves the map, or
    /// enters a blocked tile).
    Impassable {
        /// Index of the tile stepped to.
        index: usize,
    },
    /// A step enters a tile holding a hostile unit.
    Hostile {
        /// Index of the tile stepped to.
        index: usize,
    },
    /// A step leaves a stopping tile the path entered (Hold the Ford).
    Stopped {
        /// Index of the stopping tile.
        index: usize,
    },
    /// The path costs more than the mover's Mov.
    OverBudget {
        /// Total cost of the path.
        cost: u32,
        /// The mover's move points.
        budget: u32,
    },
}

impl From<MoveError> for PathError {
    fn from(e: MoveError) -> Self {
        PathError::Mover(e)
    }
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::Mover(e) => e.fmt(f),
            PathError::Empty => f.write_str("the path is empty"),
            PathError::WrongStart => f.write_str("the path doesn't start at the unit"),
            PathError::NotAdjacent { index } => {
                write!(f, "step {index} isn't to a neighbouring tile")
            }
            PathError::Impassable { index } => write!(f, "step {index} is impassable"),
            PathError::Hostile { index } => write!(f, "step {index} enters an enemy's tile"),
            PathError::Stopped { index } => write!(f, "the move must stop at step {index}"),
            PathError::OverBudget { cost, budget } => {
                write!(f, "the path costs {cost} but the unit has {budget} move")
            }
        }
    }
}

impl core::error::Error for PathError {}

/// Where one unit can move this turn. See the module docs for the rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reach {
    /// The passable tiles a move can go on from, with their costs and paths.
    walk: path::Reach,
    /// The stopping tiles reached: each with its cost and the tile of
    /// `walk` it is entered from, in tile order.
    stops: Vec<(Pos, u32, Pos)>,
    stoppable: TileSet,
}

impl Reach {
    /// The mover's tile.
    pub fn origin(&self) -> Pos {
        self.walk.origin()
    }

    /// The mover's move points.
    pub fn budget(&self) -> u32 {
        self.walk.budget()
    }

    /// The stopping tile `pos`, if one is reached: its cost and the tile
    /// it is entered from.
    fn stop(&self, pos: Pos) -> Option<(u32, Pos)> {
        self.stops
            .iter()
            .find(|&&(at, ..)| at == pos)
            .map(|&(_, cost, from)| (cost, from))
    }

    /// The cheapest cost to reach `pos`, or `None` if it is not passable.
    pub fn cost(&self, pos: Pos) -> Option<u32> {
        self.walk
            .cost(pos)
            .or_else(|| self.stop(pos).map(|(cost, _)| cost))
    }

    /// Whether the unit can reach `pos` (possibly without stopping there).
    pub fn is_passable(&self, pos: Pos) -> bool {
        self.walk.contains(pos) || self.stop(pos).is_some()
    }

    /// Whether the unit can end its move on `pos`.
    pub fn is_stoppable(&self, pos: Pos) -> bool {
        self.stoppable.contains(pos)
    }

    /// Every passable tile.
    pub fn passable(&self) -> TileSet {
        let mut tiles = self.walk.tiles();
        for &(pos, ..) in &self.stops {
            tiles.insert(pos);
        }
        tiles
    }

    /// Every tile the unit can end its move on (always includes the origin).
    pub fn stoppable(&self) -> &TileSet {
        &self.stoppable
    }

    /// The cheapest path from the origin to `dest`, both included, or `None`
    /// if `dest` is not passable. Deterministic (see the module docs).
    pub fn path_to(&self, dest: Pos) -> Option<Vec<Pos>> {
        if let Some((_, from)) = self.stop(dest) {
            let mut path = self.walk.path_to(from)?;
            path.push(dest);
            return Some(path);
        }
        self.walk.path_to(dest)
    }
}

/// The unit `id`, its movement type and move points.
fn mover_info<'a>(
    map: &BattleMap,
    classes: &ClassTable,
    units: &'a [Unit],
    id: UnitId,
) -> Result<(&'a Unit, MovementTypeId, u32), MoveError> {
    let unit = units
        .iter()
        .find(|u| u.id == id)
        .ok_or(MoveError::UnknownUnit(id))?;
    let class = classes
        .get(&unit.class)
        .ok_or_else(|| MoveError::UnknownClass(unit.class.clone()))?;
    if !map.tiles.in_bounds(unit.pos) {
        return Err(MoveError::OffMap(id));
    }
    let budget = u32::try_from(unit.move_points()).unwrap_or(0);
    Ok((unit, class.movement_type, budget))
}

/// Whether a unit hostile to `faction` stands on `pos`.
fn hostile_at(units: &[Unit], faction: Faction, pos: Pos) -> bool {
    units
        .iter()
        .any(|u| u.pos == pos && u.faction.is_hostile_to(faction))
}

/// The stopping tiles for a unit of `faction` (see the module docs): every
/// tile next to a hostile unit holding a stance that stops enemies.
fn stopping_tiles<'a>(
    units: impl IntoIterator<Item = &'a Unit>,
    faction: Faction,
    width: u16,
    height: u16,
) -> TileSet {
    let mut stops = TileSet::new(width, height);
    let holders = units.into_iter().filter(|u| {
        u.faction.is_hostile_to(faction) && u.effects.iter().any(|e| e.mods.stops_enemies)
    });
    for holder in holders {
        for dir in Dir::ALL {
            stops.insert(holder.pos.step(dir));
        }
    }
    stops
}

#[cfg(feature = "search-count")]
std::thread_local! {
    static SEARCHES: core::cell::Cell<u32> = const { core::cell::Cell::new(0) };
}

/// How many times [`reachable`] has run on this thread. **For tests only**
/// (the `search-count` feature): the UI's tests count the searches a menu
/// makes (1247).
#[cfg(feature = "search-count")]
pub fn searches() -> u32 {
    SEARCHES.with(core::cell::Cell::get)
}

/// Where unit `mover` can move: Dijkstra from its tile with its Mov as
/// budget, the stopping tiles entered but never left.
pub fn reachable(
    map: &BattleMap,
    terrain: &TerrainTable,
    classes: &ClassTable,
    units: &[Unit],
    mover: UnitId,
) -> Result<Reach, MoveError> {
    search(map, terrain, classes, units, mover, None)
}

/// [`reachable`] with unit `absent`, if any, off the map: what a copy of
/// `units` without it would give, without the copy.
fn search(
    map: &BattleMap,
    terrain: &TerrainTable,
    classes: &ClassTable,
    units: &[Unit],
    mover: UnitId,
    absent: Option<UnitId>,
) -> Result<Reach, MoveError> {
    #[cfg(feature = "search-count")]
    SEARCHES.with(|n| n.set(n.get() + 1));
    let (unit, movement_type, budget) = mover_info(map, classes, units, mover)?;
    let tiles = &map.tiles;
    let (width, height) = (tiles.width(), tiles.height());
    let present = |u: &&Unit| Some(u.id) != absent;
    let mut occupied = TileSet::new(width, height);
    // Tiles no step may enter: hostile units' and blocked ones.
    let mut closed = TileSet::new(width, height);
    for other in units.iter().filter(present).filter(|u| u.id != mover) {
        occupied.insert(other.pos);
        if other.faction.is_hostile_to(unit.faction) {
            closed.insert(other.pos);
        }
    }
    for &pos in &map.blocked {
        closed.insert(pos);
    }
    let step_cost = |pos: Pos| {
        if closed.contains(pos) {
            return None;
        }
        let step = tiles
            .get(pos)
            .and_then(|&t| terrain.move_cost(t, movement_type))?;
        Some(u32::from(step))
    };
    let stopping = stopping_tiles(units.iter().filter(present), unit.faction, width, height);

    // The walk goes on from every tile but the stopping ones; each of
    // those is then entered from its cheapest neighbour of the walk (the
    // first in `Dir::ALL` order on a tie), and ends the move.
    let walk = path::reachable(width, height, unit.pos, budget, |pos| {
        if stopping.contains(pos) {
            return None;
        }
        step_cost(pos)
    });
    let mut stops = Vec::new();
    for pos in stopping.iter() {
        if pos == unit.pos || occupied.contains(pos) {
            continue;
        }
        let Some(step) = step_cost(pos) else {
            continue;
        };
        let entry = Dir::ALL
            .into_iter()
            .filter_map(|dir| {
                let from = pos.step(dir);
                Some((walk.cost(from)?.checked_add(step)?, from))
            })
            .filter(|&(cost, _)| cost <= budget)
            .min_by_key(|&(cost, _)| cost);
        if let Some((cost, from)) = entry {
            stops.push((pos, cost, from));
        }
    }

    let mut stoppable = TileSet::new(width, height);
    for pos in walk.tiles().iter() {
        if pos == unit.pos || !occupied.contains(pos) {
            stoppable.insert(pos);
        }
    }
    for &(pos, ..) in &stops {
        stoppable.insert(pos);
    }
    Ok(Reach {
        walk,
        stops,
        stoppable,
    })
}

/// The cost of moving unit `mover` along `path` (its own tile first), or why
/// the path is illegal: every step must be to a 4-way neighbour the mover can
/// enter, not holding a hostile unit, never leaving a stopping tile it
/// entered (see the module docs), and the total must be within its Mov.
/// Where the path ends is not checked (the UI's arrow may point at any
/// passable tile). The UI's "arrow follows the cursor" path uses this.
pub fn path_cost(
    map: &BattleMap,
    terrain: &TerrainTable,
    classes: &ClassTable,
    units: &[Unit],
    mover: UnitId,
    path: &[Pos],
) -> Result<u32, PathError> {
    let (unit, movement_type, budget) = mover_info(map, classes, units, mover)?;
    let Some(&first) = path.first() else {
        return Err(PathError::Empty);
    };
    if first != unit.pos {
        return Err(PathError::WrongStart);
    }
    let (width, height) = (map.tiles.width(), map.tiles.height());
    let stopping = stopping_tiles(units, unit.faction, width, height);
    let mut total: u32 = 0;
    for (i, pair) in path.windows(2).enumerate() {
        let (from, to) = (pair[0], pair[1]);
        let index = i + 1;
        if i > 0 && stopping.contains(from) {
            return Err(PathError::Stopped { index: i });
        }
        if Pos::manhattan(from, to) != 1 {
            return Err(PathError::NotAdjacent { index });
        }
        if hostile_at(units, unit.faction, to) {
            return Err(PathError::Hostile { index });
        }
        let step = Some(to)
            .filter(|to| !map.blocked.contains(to))
            .and_then(|to| map.tiles.get(to))
            .and_then(|&t| terrain.move_cost(t, movement_type))
            .ok_or(PathError::Impassable { index })?;
        total = total.saturating_add(u32::from(step));
    }
    if total > budget {
        return Err(PathError::OverBudget {
            cost: total,
            budget,
        });
    }
    Ok(total)
}

/// Every tile at Manhattan distance `min..=max` from a stoppable tile of
/// `reach`, except the stoppable tiles themselves. Empty if `min > max`.
pub fn attack_tiles(reach: &Reach, min: u32, max: u32) -> TileSet {
    let stoppable = reach.stoppable();
    let mut out = TileSet::new(stoppable.width(), stoppable.height());
    let (width, height) = (i64::from(stoppable.width()), i64::from(stoppable.height()));
    let max = i64::from(max);
    for from in stoppable.iter() {
        let (fx, fy) = (i64::from(from.x), i64::from(from.y));
        // The diamond of radius `max`, clipped to the map's rows and columns.
        for y in (fy - max).max(0)..(fy + max + 1).min(height) {
            let rest = max - (y - fy).abs();
            for x in (fx - rest).max(0)..(fx + rest + 1).min(width) {
                // Inside the map, so both fit.
                let (Ok(x), Ok(y)) = (i32::try_from(x), i32::try_from(y)) else {
                    continue;
                };
                let to = Pos::new(x, y);
                if Pos::manhattan(from, to) >= min && !stoppable.contains(to) {
                    out.insert(to);
                }
            }
        }
    }
    out
}

/// The tiles of a `width` × `height` map within Manhattan `radius` of
/// `center`, except `center` itself: where an area skill used from
/// `center` reaches (ticket 0482). The skill's effect and its preview on
/// the map both use it.
pub fn area_tiles(width: u16, height: u16, center: Pos, radius: u32) -> TileSet {
    let mut out = TileSet::new(width, height);
    let (cols, rows) = (i64::from(width), i64::from(height));
    let (cx, cy, reach) = (i64::from(center.x), i64::from(center.y), i64::from(radius));
    // The diamond, clipped to the map's rows and columns.
    for y in (cy - reach).max(0)..(cy + reach + 1).min(rows) {
        let rest = reach - (y - cy).abs();
        for x in (cx - rest).max(0)..(cx + rest + 1).min(cols) {
            // Inside the map, so both fit.
            let (Ok(x), Ok(y)) = (i32::try_from(x), i32::try_from(y)) else {
                continue;
            };
            let to = Pos::new(x, y);
            if to != center {
                out.insert(to);
            }
        }
    }
    out
}

/// The tiles unit `unit` threatens this turn with attacks of the given
/// ranges (in a battle: [`Unit::threat_ranges`], its weapons, castable
/// attack spells, arts and actives): its stoppable tiles plus their attack tiles. Empty if
/// `ranges` is empty.
pub fn threat_area(
    map: &BattleMap,
    terrain: &TerrainTable,
    classes: &ClassTable,
    units: &[Unit],
    unit: UnitId,
    ranges: &[AttackRange],
) -> Result<TileSet, MoveError> {
    let reach = reachable(map, terrain, classes, units, unit)?;
    Ok(area_of(&reach, ranges))
}

/// The threat area of `reach` with attacks of `ranges` ([`threat_area`]).
fn area_of(reach: &Reach, ranges: &[AttackRange]) -> TileSet {
    let stoppable = reach.stoppable();
    if ranges.is_empty() {
        return TileSet::new(stoppable.width(), stoppable.height());
    }
    let mut set = stoppable.clone();
    for &(min, max) in ranges {
        set.union_with(&attack_tiles(reach, min, max));
    }
    set
}

/// The union of the threat areas of every unit hostile to `faction`, with
/// each unit's attack ranges given by `ranges` (in a battle:
/// [`Unit::threat_ranges`], the ranges of what it can attack with now).
pub fn danger_zone(
    map: &BattleMap,
    terrain: &TerrainTable,
    classes: &ClassTable,
    units: &[Unit],
    faction: Faction,
    ranges: impl Fn(&Unit) -> Vec<AttackRange>,
) -> Result<TileSet, MoveError> {
    let mut zone = TileSet::new(map.tiles.width(), map.tiles.height());
    for u in units.iter().filter(|u| u.faction.is_hostile_to(faction)) {
        zone.union_with(&threat_area(
            map,
            terrain,
            classes,
            units,
            u.id,
            &ranges(u),
        )?);
    }
    Ok(zone)
}

/// The units hostile to unit `unit` that could attack it in their next
/// phase if it stood on `at` (`docs/design/turn-structure.md`, *Threat
/// arrows*), in the order of `units`: `at` is in their [`threat_area`],
/// worked out with `unit` on `at` instead of its own tile. Whether their AI
/// would choose to doesn't matter, except that one that never moves
/// ([`AiBehavior::Stationary`]) threatens only what its ranges reach from
/// its own tile. `ranges` gives each unit's attack ranges, as for
/// [`danger_zone`].
pub fn threats_at(
    map: &BattleMap,
    terrain: &TerrainTable,
    classes: &ClassTable,
    units: &[Unit],
    unit: UnitId,
    at: Pos,
    ranges: impl Fn(&Unit) -> Vec<AttackRange>,
) -> Result<Vec<UnitId>, MoveError> {
    let Some(target) = units.iter().find(|u| u.id == unit) else {
        return Err(MoveError::UnknownUnit(unit));
    };
    let faction = target.faction;
    let mut moved = units.to_vec();
    for u in moved.iter_mut().filter(|u| u.id == unit) {
        u.pos = at;
    }
    let mut out = Vec::new();
    for u in moved.iter().filter(|u| u.faction.is_hostile_to(faction)) {
        let ranges = ranges(u);
        let reaches = if u.ai == AiBehavior::Stationary {
            let distance = Pos::manhattan(u.pos, at);
            ranges
                .iter()
                .any(|&(min, max)| (min..=max).contains(&distance))
        } else {
            threat_area(map, terrain, classes, &moved, u.id, &ranges)?.contains(at)
        };
        if reaches {
            out.push(u.id);
        }
    }
    Ok(out)
}

/// [`threats_at`] for one unit on any tile, with each hostile unit's moves
/// searched once (ticket 1251). Made for a battle as it stands; asked with
/// that same battle, it gives `threats_at`'s answer for every tile.
///
/// Each hostile unit's moves are searched with the unit off the map. The
/// unit standing on a tile only takes moves away from it (that tile, which
/// no step may enter, and the tiles next to it if the unit holds a stance
/// that stops enemies), so its threat then is a part of that first one. A
/// tile outside it is never threatened; a tile its moves never reach,
/// without such a stance, changes nothing; a tile in range of a tile whose
/// path keeps clear of the unit is threatened. Only when none of these
/// settles it is the hostile unit searched again with the unit on the tile.
#[derive(Debug, Clone)]
pub struct ThreatPlanner {
    unit: UnitId,
    threats: Vec<Threat>,
}

/// A unit hostile to the planner's unit, and how it gets about.
#[derive(Debug, Clone)]
struct Threat {
    id: UnitId,
    ranges: Vec<AttackRange>,
    kind: ThreatKind,
}

#[derive(Debug, Clone)]
enum ThreatKind {
    /// It never moves ([`AiBehavior::Stationary`]): its tile.
    Stationary(Pos),
    /// Its moves with the planner's unit off the map, and their threat
    /// area. `stops`: the planner's unit stops it next to itself.
    Moves {
        reach: Reach,
        area: TileSet,
        stops: bool,
    },
}

impl ThreatPlanner {
    /// Searches the moves of every unit hostile to unit `unit` that may
    /// move, with `unit` off the map. `ranges` gives each unit's attack
    /// ranges, as for [`threats_at`]. Fails as `threats_at` would.
    pub fn new(
        map: &BattleMap,
        terrain: &TerrainTable,
        classes: &ClassTable,
        units: &[Unit],
        unit: UnitId,
        ranges: impl Fn(&Unit) -> Vec<AttackRange>,
    ) -> Result<Self, MoveError> {
        Self::made(map, terrain, classes, units, unit, ranges, None)
    }

    /// [`new`](Self::new), starting from the battle's threat areas
    /// `areas`, made for this same battle (ticket 1259): a hostile unit
    /// whose kept moves never come next to `unit` (nor within two tiles of
    /// it, if it holds a stance that stops enemies) moves just the same
    /// with `unit` off the map, as its search never tries the tiles that
    /// change, so it isn't searched again.
    pub fn with_areas(
        map: &BattleMap,
        terrain: &TerrainTable,
        classes: &ClassTable,
        units: &[Unit],
        unit: UnitId,
        ranges: impl Fn(&Unit) -> Vec<AttackRange>,
        areas: &ThreatAreas,
    ) -> Result<Self, MoveError> {
        Self::made(map, terrain, classes, units, unit, ranges, Some(areas))
    }

    fn made(
        map: &BattleMap,
        terrain: &TerrainTable,
        classes: &ClassTable,
        units: &[Unit],
        unit: UnitId,
        ranges: impl Fn(&Unit) -> Vec<AttackRange>,
        areas: Option<&ThreatAreas>,
    ) -> Result<Self, MoveError> {
        let Some(target) = units.iter().find(|u| u.id == unit) else {
            return Err(MoveError::UnknownUnit(unit));
        };
        let stops = target.effects.iter().any(|e| e.mods.stops_enemies);
        let mut threats = Vec::new();
        let hostile = |u: &&Unit| u.faction.is_hostile_to(target.faction);
        for u in units.iter().filter(hostile) {
            let ranges = ranges(u);
            let kept = areas.and_then(|a| a.moves_clear_of(u.id, target.pos, stops));
            let kind = if u.ai == AiBehavior::Stationary {
                ThreatKind::Stationary(u.pos)
            } else if let Some((reach, kept_ranges, area)) = kept {
                ThreatKind::Moves {
                    area: if *kept_ranges == ranges {
                        area.clone()
                    } else {
                        area_of(reach, &ranges)
                    },
                    reach: reach.clone(),
                    stops,
                }
            } else {
                let reach = search(map, terrain, classes, units, u.id, Some(unit))?;
                ThreatKind::Moves {
                    area: area_of(&reach, &ranges),
                    reach,
                    stops,
                }
            };
            threats.push(Threat {
                id: u.id,
                ranges,
                kind,
            });
        }
        Ok(Self { unit, threats })
    }

    /// The unit it answers for.
    pub fn unit(&self) -> UnitId {
        self.unit
    }

    /// [`threats_at`]'s answer for the planner's unit on `at`, in the
    /// battle it was made for (the same `map`, `terrain`, `classes` and
    /// `units`). Searches a hostile unit's moves again only when the
    /// searches kept don't settle it.
    pub fn threats_at(
        &self,
        map: &BattleMap,
        terrain: &TerrainTable,
        classes: &ClassTable,
        units: &[Unit],
        at: Pos,
    ) -> Result<Vec<UnitId>, MoveError> {
        let mut moved: Option<Vec<Unit>> = None;
        let mut out = Vec::new();
        for threat in &self.threats {
            let reaches = if let Some(reaches) = threat.settles(at) {
                reaches
            } else {
                let moved = moved.get_or_insert_with(|| {
                    let mut moved = units.to_vec();
                    for u in moved.iter_mut().filter(|u| u.id == self.unit) {
                        u.pos = at;
                    }
                    moved
                });
                threat_area(map, terrain, classes, moved, threat.id, &threat.ranges)?.contains(at)
            };
            if reaches {
                out.push(threat.id);
            }
        }
        Ok(out)
    }
}

/// [`threats_at`] for every unit of one faction on its own tile, with each
/// hostile unit's moves searched once for the battle (ticket 1259). A unit
/// on its own tile is where the battle has it, so its threats are the
/// hostile units whose threat area, worked out with every unit in place,
/// holds that tile: the same areas [`danger_zone`] unions. Made for a
/// battle as it stands, it answers for that battle only.
///
/// Each hostile unit is kept as a [`ThreatPlanner`] keeps it, its moves
/// searched with every unit in place.
#[derive(Debug, Clone)]
pub struct ThreatAreas {
    faction: Faction,
    threats: Vec<Threat>,
}

impl ThreatAreas {
    /// Searches the moves of every unit hostile to `faction` that may
    /// move. `ranges` gives each unit's attack ranges, as for
    /// [`threats_at`]. Fails as `threats_at` would.
    pub fn new(
        map: &BattleMap,
        terrain: &TerrainTable,
        classes: &ClassTable,
        units: &[Unit],
        faction: Faction,
        ranges: impl Fn(&Unit) -> Vec<AttackRange>,
    ) -> Result<Self, MoveError> {
        let mut threats = Vec::new();
        for u in units.iter().filter(|u| u.faction.is_hostile_to(faction)) {
            let ranges = ranges(u);
            let kind = if u.ai == AiBehavior::Stationary {
                ThreatKind::Stationary(u.pos)
            } else {
                let reach = reachable(map, terrain, classes, units, u.id)?;
                ThreatKind::Moves {
                    area: area_of(&reach, &ranges),
                    reach,
                    stops: false,
                }
            };
            threats.push(Threat {
                id: u.id,
                ranges,
                kind,
            });
        }
        Ok(Self { faction, threats })
    }

    /// [`threats_at`]'s answer for `unit` on its own tile, in the battle
    /// these were made for; `None` if it isn't of their faction.
    pub fn threats_to(&self, unit: &Unit) -> Option<Vec<UnitId>> {
        if unit.faction != self.faction {
            return None;
        }
        let reaches = |threat: &&Threat| match &threat.kind {
            ThreatKind::Stationary(pos) => threat.in_range(*pos, unit.pos),
            ThreatKind::Moves { area, .. } => area.contains(unit.pos),
        };
        let threats = self.threats.iter().filter(reaches);
        Some(threats.map(|t| t.id).collect())
    }

    /// Hostile unit `id`'s kept moves, its ranges and threat area, if its
    /// search with a unit off `pos` is the same: none of its passable
    /// tiles is next to `pos`, nor, with `stops` (the unit there stops
    /// enemies next to it), within two tiles of it.
    fn moves_clear_of(
        &self,
        id: UnitId,
        pos: Pos,
        stops: bool,
    ) -> Option<(&Reach, &Vec<AttackRange>, &TileSet)> {
        let threat = self.threats.iter().find(|t| t.id == id)?;
        let ThreatKind::Moves { reach, area, .. } = &threat.kind else {
            return None;
        };
        // The tiles next to `pos`, and with `stops` the tiles next to them.
        let around = |p: Pos| Dir::ALL.into_iter().map(move |dir| p.step(dir));
        let mut near: Vec<Pos> = around(pos).collect();
        if stops {
            let next = near.clone();
            near.extend(next.into_iter().flat_map(around));
        }
        let clear = !near.into_iter().any(|t| reach.is_passable(t));
        clear.then_some((reach, &threat.ranges, area))
    }
}

impl Threat {
    /// Whether one of its ranges reaches `at` from `from`.
    fn in_range(&self, from: Pos, at: Pos) -> bool {
        let distance = Pos::manhattan(from, at);
        self.ranges
            .iter()
            .any(|&(min, max)| (min..=max).contains(&distance))
    }

    /// Whether it threatens the planner's unit on `at`, when that is
    /// settled without searching its moves again (see [`ThreatPlanner`]).
    fn settles(&self, at: Pos) -> Option<bool> {
        let in_range = |from: Pos| self.in_range(from, at);
        let (reach, area, stops) = match &self.kind {
            ThreatKind::Stationary(pos) => return Some(in_range(*pos)),
            ThreatKind::Moves { reach, area, stops } => (reach, area, *stops),
        };
        if !area.contains(at) {
            return Some(false);
        }
        if !stops && !reach.is_passable(at) {
            return Some(true);
        }
        // A path clear of the unit: never onto its tile and, if it stops
        // enemies, never on from a tile next to it. The first tile is the
        // hostile unit's own, which it may always leave.
        let clear = |path: &[Pos]| {
            let walked = path.get(1..).unwrap_or_default();
            let passed = walked.split_last().map_or(&[][..], |(_, before)| before);
            let next_to = |&pos: &Pos| Pos::manhattan(pos, at) == 1;
            let stopped = stops && passed.iter().any(next_to);
            !walked.contains(&at) && !stopped
        };
        let witness = reach
            .stoppable()
            .iter()
            .filter(|&from| in_range(from))
            .any(|from| reach.path_to(from).is_some_and(|path| clear(&path)));
        witness.then_some(true)
    }
}

#[cfg(test)]
mod tests;
