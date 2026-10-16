//! The action of a map shot (ticket 0827, ADR-0061): a list of timed walks
//! and strikes its units act out. **Authored pictures, not a battle**:
//! nothing here runs the combat rules. The checks below only make sure
//! the list can be drawn (the units exist, a walk starts where its unit
//! is, a unit that fell does nothing more). The format is described in
//! `assets/cinematics/README.md`.
//!
//! A conversation's opening (ticket 0734, `assets/openings/`) is a list of
//! the same walks, by characters it places on a place's map: the same
//! checks run on that map and that cast ([`Stage`]).
use crate::prelude::*;

use alloc::collections::BTreeMap;

use serde::Deserialize;
use serde::de::value::SeqAccessDeserializer;
use serde::de::{SeqAccess, Visitor};
use trpg_core::{
    BattleDef, BattleMap, CharacterId, ClassDef, MovementTypeId, Pos, TerrainTable, UnitId,
};

use super::{Refs, millis, seconds};
use crate::battle::{self, bare};

/// The action of one shot: a list, and where on its clock the shot starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShotAction {
    /// The action clock's value when the shot starts, in milliseconds. It
    /// then runs with the cinematic's time.
    pub from_ms: u32,
    /// What happens, in order of time; never empty.
    pub actions: Vec<ActionDef>,
}

/// One thing a unit does, and when.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionDef {
    /// When it starts, in milliseconds on the action clock.
    pub at_ms: u32,
    /// What is done.
    pub act: Act,
}

/// What a unit does in a shot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Act {
    /// The unit walks along a path at an even speed and stays at its end.
    Walk {
        /// Who walks: its unit id in the battle.
        unit: UnitId,
        /// Every tile of the walk, the one it starts on first, each next
        /// to the one before it; at least two.
        path: Vec<Pos>,
        /// How long the walk takes, in milliseconds; never 0.
        ms: u32,
        /// The way the walker keeps facing all along (stepping back, or
        /// to the side); `None`: the way it goes (ticket 0734).
        facing: Option<Heading>,
    },
    /// The unit lunges at another, which is left with a part of its HP;
    /// with none, it falls.
    Strike {
        /// Who strikes.
        unit: UnitId,
        /// Who is struck.
        target: UnitId,
        /// The HP the target has left, in percent of its full HP: 0 to 100.
        hp_left: u8,
    },
}

/// Which way someone is turned, as a file writes it (ticket 0734): `Down`
/// (towards the camera), `Left`, `Right` or `Up`. A look only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Deserialize)]
pub enum Heading {
    /// Towards the camera.
    #[default]
    Down,
    /// To the left.
    Left,
    /// To the right.
    Right,
    /// Away from the camera.
    Up,
}

/// One entry of an action list, as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawAction {
    at: f32,
    #[serde(rename = "do")]
    act: RawAct,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
enum RawAct {
    Walk {
        unit: String,
        path: Vec<(i32, i32)>,
        secs: f32,
        /// Written bare: `facing: Left`.
        #[serde(default, deserialize_with = "bare")]
        facing: Option<Heading>,
    },
    Strike {
        unit: String,
        target: String,
        hp_left: i32,
    },
}

/// A shot's `action`: a list written in place, or the name of one in the
/// file's `actions` table.
pub(super) enum RawActionRef {
    Named(String),
    List(Vec<RawAction>),
}

impl<'de> Deserialize<'de> for RawActionRef {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Either;
        impl<'de> Visitor<'de> for Either {
            type Value = RawActionRef;

            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_str("a list of actions, or the name of one in `actions`")
            }

            fn visit_str<E>(self, name: &str) -> Result<Self::Value, E> {
                Ok(RawActionRef::Named(name.to_owned()))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
                Vec::deserialize(SeqAccessDeserializer::new(seq)).map(RawActionRef::List)
            }
        }
        d.deserialize_any(Either)
    }
}

/// A unit on a battle's map when the battle starts, or a character an
/// opening places on its map.
pub(super) struct Actor<'a> {
    pub(super) id: UnitId,
    pub(super) pos: Pos,
    /// Its class, if the class table has it.
    pub(super) class: Option<&'a ClassDef>,
    /// The movement type it walks by and what it is in a message (its
    /// class's name); `None`: it may walk anywhere on the map.
    pub(super) walks: Option<(MovementTypeId, String)>,
}

impl<'a> Actor<'a> {
    /// A unit `id` of `class` (if the class table has it) on `pos`,
    /// walking by its class's movement type.
    pub(super) fn of_class(id: UnitId, pos: Pos, class: Option<&'a ClassDef>) -> Self {
        Self {
            id,
            pos,
            class,
            walks: class.map(|c| (c.movement_type, c.name.clone())),
        }
    }
}

/// Where an action list is acted out: a map and who is on it.
pub(super) struct Stage<'a> {
    /// The map walked on.
    pub(super) map: &'a BattleMap,
    /// Who is on it, by the names entries call them.
    pub(super) cast: BTreeMap<String, Actor<'a>>,
    /// Where someone not in the cast isn't, for a message: `on the map of
    /// battle "ch01" at its start`.
    pub(super) nobody: String,
    /// Whether strikes may be acted out (an opening has walks only).
    pub(super) strikes: bool,
}

impl<'a> Stage<'a> {
    /// `battle`'s map, with its units where they start.
    pub(super) fn battle(battle: &'a BattleDef, refs: Refs<'a>) -> Self {
        Self {
            map: &battle.map,
            cast: cast(battle, refs),
            nobody: format!("on the map of battle \"{}\" at its start", battle.id),
            strikes: true,
        }
    }
}

/// The units on `battle`'s map at its start, by the names a cinematic
/// knows them by: a player slot's character, an enemy's `id` in the
/// battle file, and an enemy that is a character by that character.
pub(super) fn cast<'a>(battle: &BattleDef, refs: Refs<'a>) -> BTreeMap<String, Actor<'a>> {
    let classes = refs.classes;
    let mut cast = BTreeMap::new();
    for slot in &battle.player_slots {
        let def = refs.characters.characters.get(&slot.character);
        let class = def.and_then(|d| classes.get(&d.class));
        let actor = Actor::of_class(slot.id, slot.pos, class);
        cast.insert(slot.character.0.clone(), actor);
    }
    let named = battle::unit_names(&battle.id);
    for unit in &battle.enemies {
        let ids = named.iter();
        let own = ids.filter(|&(_, &id)| id == unit.id).map(|(name, _)| name);
        let character = unit.character.as_ref().map(|CharacterId(c)| c);
        for name in own.chain(character) {
            let actor = Actor::of_class(unit.id, unit.pos, classes.get(&unit.class));
            cast.insert(name.clone(), actor);
        }
    }
    cast
}

/// Where a unit is as an action list is read, entry by entry.
struct Standing {
    pos: Pos,
    /// When its last walk ends, in milliseconds.
    free_at_ms: u64,
    /// The percent of its HP it has left.
    hp_left: i32,
    /// The entry (counted from 1) whose strike felled it.
    fell_at: Option<usize>,
}

/// Reads one action list on one stage.
struct Checker<'a, 'r> {
    stage: &'a Stage<'r>,
    terrain: &'a TerrainTable,
    state: BTreeMap<UnitId, Standing>,
    problems: Vec<String>,
}

impl<'a> Checker<'a, '_> {
    /// The unit called `name`, as it stands now, and how it walks; a
    /// problem if the stage has none, or if it has fallen.
    fn unit(
        &mut self,
        n: usize,
        name: &str,
    ) -> Option<(UnitId, Option<&'a (MovementTypeId, String)>)> {
        let stage = self.stage;
        let Some(actor) = stage.cast.get(name) else {
            self.problems
                .push(format!("action {n}: no unit \"{name}\" {}", stage.nobody));
            return None;
        };
        let (id, walks) = (actor.id, actor.walks.as_ref());
        let standing = self.state.entry(id).or_insert(Standing {
            pos: actor.pos,
            free_at_ms: 0,
            hp_left: 100,
            fell_at: None,
        });
        if let Some(fell) = standing.fell_at {
            self.problems.push(format!(
                "action {n}: \"{name}\" fell at action {fell} and can do nothing more"
            ));
            return None;
        }
        Some((id, walks))
    }

    /// A problem if the unit `id` (called `name`) is in the middle of a
    /// walk at `at_ms`.
    fn not_walking(&mut self, n: usize, name: &str, id: UnitId, at_ms: u64) {
        let free = self.state.get(&id).map_or(0, |s| s.free_at_ms);
        if at_ms < free {
            #[allow(clippy::cast_precision_loss)] // a few thousand seconds at most
            let until = free as f64 / 1000.0;
            self.problems.push(format!(
                "action {n}: \"{name}\" is still walking then (until {until} seconds)"
            ));
        }
    }

    /// Checks a walk and returns its path tile by tile (empty if the path
    /// as written can't be walked).
    fn walk(&mut self, n: usize, at_ms: u64, name: &str, corners: &[(i32, i32)]) -> Vec<Pos> {
        let Some((id, walks)) = self.unit(n, name) else {
            return Vec::new();
        };
        self.not_walking(n, name, id, at_ms);
        let corners: Vec<Pos> = corners.iter().map(|&(x, y)| Pos::new(x, y)).collect();
        let (Some(&first), Some(&last), true) =
            (corners.first(), corners.last(), corners.len() > 1)
        else {
            self.problems
                .push(format!("action {n}: path: a walk needs at least two tiles"));
            return Vec::new();
        };
        let was = self.state.get(&id).map_or(first, |s| s.pos);
        if let Some(standing) = self.state.get_mut(&id) {
            standing.pos = last;
        }
        if first != was {
            self.problems.push(format!(
                "action {n}: path starts at ({}, {}), but \"{name}\" is at ({}, {}) then",
                first.x, first.y, was.x, was.y
            ));
        }
        let tiles = &self.stage.map.tiles;
        let (width, height) = (tiles.width(), tiles.height());
        // Off the map or crooked, there are no tiles to walk.
        let mut walkable = true;
        for at in corners.iter().filter(|&&at| tiles.get(at).is_none()) {
            walkable = false;
            self.problems.push(format!(
                "action {n}: path: ({}, {}) is not on the map ({width}×{height} tiles)",
                at.x, at.y
            ));
        }
        for pair in corners.windows(2) {
            let (from, to) = (pair[0], pair[1]);
            if (from.x != to.x) == (from.y != to.y) {
                walkable = false;
                self.problems.push(format!(
                    "action {n}: path: ({}, {}) to ({}, {}) is not a straight run along a row or a column",
                    from.x, from.y, to.x, to.y
                ));
            }
        }
        if !walkable {
            return Vec::new();
        }
        let path = tile_by_tile(&corners, i32::from(width.max(height)));
        for at in &path {
            let cost = (tiles.get(*at).copied().zip(walks))
                .map(|(terrain, (movement, _))| self.terrain.move_cost(terrain, *movement));
            if let (Some(None), Some((_, what))) = (cost, walks) {
                self.problems.push(format!(
                    "action {n}: path: a {what} can't walk on ({}, {})",
                    at.x, at.y
                ));
            }
        }
        path
    }

    /// Checks a strike and returns who strikes whom.
    fn strike(
        &mut self,
        n: usize,
        at_ms: u64,
        (name, target): (&str, &str),
        hp_left: i32,
    ) -> Option<(UnitId, UnitId)> {
        let unit = self.unit(n, name);
        let struck = self.unit(n, target);
        if let Some((id, _)) = unit {
            self.not_walking(n, name, id, at_ms);
        }
        let in_range = (0..=100).contains(&hp_left);
        if !in_range {
            self.problems
                .push(format!("action {n}: hp_left {hp_left}; it is 0 to 100"));
        }
        let ((unit, _), (struck, _)) = (unit?, struck?);
        if unit == struck {
            self.problems
                .push(format!("action {n}: \"{name}\" can't strike itself"));
            return None;
        }
        let standing = self.state.get_mut(&struck)?;
        if in_range && hp_left > standing.hp_left {
            self.problems.push(format!(
                "action {n}: hp_left {hp_left} is more than the {} \"{target}\" has left; HP never rises",
                standing.hp_left
            ));
        } else if in_range {
            standing.hp_left = hp_left;
            standing.fell_at = (hp_left == 0).then_some(n);
        }
        Some((unit, struck))
    }
}

/// Every tile of a walk through `corners`, each a straight run from the
/// one before it: the first corner, then tile after tile. No run is taken
/// further than `longest` tiles (a map's longer side), so that corners
/// that aren't on the map can never make a path without end.
fn tile_by_tile(corners: &[Pos], longest: i32) -> Vec<Pos> {
    let mut path: Vec<Pos> = corners.first().copied().into_iter().collect();
    for pair in corners.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let (dx, dy) = (to.x.saturating_sub(from.x), to.y.saturating_sub(from.y));
        let run = dx.saturating_abs().max(dy.saturating_abs()).min(longest);
        let tile = |i: i32| Pos::new(from.x + dx.signum() * i, from.y + dy.signum() * i);
        path.extend((1..=run).map(tile));
    }
    path
}

/// Checks the action list `raw` on `stage` (whose tiles a movement type
/// can enter: `terrain`) and returns it as the game keeps it, with every
/// problem in plain words:
///
/// - no entries; an `at` that isn't a time, or is before the entry before
///   it;
/// - a unit or a target that isn't in the stage's cast (on the battle's
///   map at its start), or that a strike before left with no HP;
/// - a strike on a stage that has none (an opening);
/// - a walk that takes no time, has fewer than two tiles, doesn't start
///   where its unit is then, starts before the unit's walk before it has
///   ended, leaves the map, turns anywhere but along a row or a column,
///   or crosses a tile the unit's movement type can't enter;
/// - a strike by a unit in the middle of a walk, or at itself; an
///   `hp_left` that isn't 0 to 100, or is more than the target has left.
pub(super) fn check(
    raw: &[RawAction],
    stage: &Stage,
    terrain: &TerrainTable,
) -> (Vec<ActionDef>, Vec<String>) {
    let mut v = Checker {
        stage,
        terrain,
        state: BTreeMap::new(),
        problems: Vec::new(),
    };
    if raw.is_empty() {
        v.problems.push("an action needs at least one entry".into());
    }
    let mut actions: Vec<ActionDef> = Vec::new();
    // When the entry before started.
    let mut before = 0;
    for (i, RawAction { at, act }) in raw.iter().enumerate() {
        let n = i + 1;
        let at_ms = millis(*at);
        match at_ms {
            None => v.problems.push(format!("action {n}: at {at} is not a time")),
            Some(ms) if ms < before => v.problems.push(format!(
                "action {n}: at {at} is before the action before it ({} seconds); actions are in order of time",
                seconds(before)
            )),
            _ => {}
        }
        let at_ms = at_ms.unwrap_or(before).max(before);
        before = at_ms;
        let act = match act {
            RawAct::Walk {
                unit,
                path,
                secs,
                facing,
            } => {
                let ms = millis(*secs).filter(|&ms| ms > 0);
                if ms.is_none() {
                    v.problems.push(format!(
                        "action {n}: secs {secs}; a walk takes more than 0 seconds"
                    ));
                }
                let path = v.walk(n, u64::from(at_ms), unit, path);
                let ms = ms.unwrap_or(1);
                let id = stage.cast.get(unit).map(|actor| actor.id);
                if let Some(standing) = id.and_then(|id| v.state.get_mut(&id)) {
                    standing.free_at_ms = u64::from(at_ms) + u64::from(ms);
                }
                id.filter(|_| path.len() >= 2).map(|unit| Act::Walk {
                    unit,
                    path,
                    ms,
                    facing: *facing,
                })
            }
            RawAct::Strike { .. } if !stage.strikes => {
                v.problems
                    .push(format!("action {n}: an opening has walks only, no strikes"));
                None
            }
            RawAct::Strike {
                unit,
                target,
                hp_left,
            } => v
                .strike(n, u64::from(at_ms), (unit, target), *hp_left)
                .map(|(unit, target)| Act::Strike {
                    unit,
                    target,
                    hp_left: u8::try_from(*hp_left).unwrap_or(0),
                }),
        };
        if let Some(act) = act {
            actions.push(ActionDef { at_ms, act });
        }
    }
    (actions, v.problems)
}
