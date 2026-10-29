//! Battle files (`assets/battles/*.ron`, ticket 0801): everything needed to
//! play one battle. See `assets/battles/README.md` for the format; the
//! loader builds a [`BattleDef`] and checks every rule listed there.
use crate::prelude::*;

use alloc::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use trpg_core::{
    AiBehavior, BattleDef, BattleKind, BattleMap, BattleMusic, BattleNote, CharacterId, ClassTable,
    Defeat, Difficulty, Faction, ItemDef, ItemId, ItemTable, Leftovers, Level, Loot, Objective,
    PlayerSlot, Pos, Reinforcement, Role, TerrainId, TerrainTable, TileRect, Trigger, Turn, Unit,
    UnitId, VillageGoal, default_map_label,
};

use crate::audio::AudioManifest;
use crate::bundle;
use crate::character::{CharacterTable, RawLoadout, character_unit, check_map_labels};
use crate::dialogue::DialogueTable;
use crate::error::ContentError;
use crate::lighting::Lighting;
use crate::map::{MapDef, column_number, line_number};
use crate::ron_loader::parse_ron;
use crate::terrain::TerrainDisplayTable;
use crate::trigger::{check_triggers, off_map};

pub mod tune;

/// Directory of the battle files inside the asset bundle.
pub const BATTLES_DIR: &str = "battles";

/// Extension of a battle file.
const EXTENSION: &str = ".ron";

/// The message for a battle with Preparations that also lists a default
/// pack: there the player brings only what they own (Nick, 0408).
pub const PACK_WITH_PREPARATIONS: &str =
    "default_pack: a battle with Preparations has none; the player packs their own items";

/// The most battle notes a battle may have: with [`MAX_NOTE_CHARS`], they
/// always fit the notes panel and the `Objective` page.
pub const MAX_NOTES: usize = 5;

/// The longest a battle note's text may be, in characters.
pub const MAX_NOTE_CHARS: usize = 120;

/// The content a battle file refers to.
#[derive(Debug, Clone, Copy)]
pub struct BattleRefs<'a> {
    /// Maps by id.
    pub maps: &'a BTreeMap<String, MapDef>,
    /// Terrain rules (which tiles a unit can stand on).
    pub terrain: &'a TerrainTable,
    /// Terrain looks (which terrain is decoration, never on a battle's
    /// map).
    pub terrain_looks: &'a TerrainDisplayTable,
    /// Classes.
    pub classes: &'a ClassTable,
    /// Items.
    pub items: &'a ItemTable,
    /// Named characters and generic templates.
    pub characters: &'a CharacterTable,
    /// Scenes, for the triggers.
    pub dialogue: &'a DialogueTable,
    /// Music cues and pools, for the battle's music.
    pub audio: &'a AudioManifest,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBattle {
    id: String,
    map: String,
    player_slots: Vec<RawSlot>,
    #[serde(default)]
    enemies: Vec<RawEnemy>,
    #[serde(default)]
    reinforcements: Vec<RawReinforcement>,
    #[serde(default)]
    preparations: bool,
    /// Written bare: `pack_cap: 3`.
    #[serde(default, deserialize_with = "bare")]
    pack_cap: Option<usize>,
    #[serde(default)]
    default_pack: Vec<String>,
    #[serde(default)]
    solo_stock: Vec<String>,
    #[serde(default)]
    solo_gold: u32,
    #[serde(default)]
    solo_bench: Vec<String>,
    #[serde(default)]
    clear_gold: u32,
    objective: RawObjective,
    #[serde(default)]
    triggers: Vec<Trigger>,
    #[serde(default)]
    battle_notes: Vec<RawNote>,
    /// The place an escape map's exit leads to, written bare:
    /// `exit_name: "the Brennish lines"`.
    #[serde(default, deserialize_with = "bare")]
    exit_name: Option<String>,
    /// The houses to save (0313), written bare: `village: (minimum: 3,
    /// rewards: [...])`.
    #[serde(default, deserialize_with = "bare")]
    village: Option<RawVillage>,
    difficulty: Difficulty,
    /// What the autobalance may change: `Story` if left out.
    #[serde(default)]
    kind: BattleKind,
    music: BattleMusic,
    seed: u64,
    /// The light the battle is fought in instead of its map's, written
    /// bare: `light: "dusk"`. Look data: read by [`light_of`], never put
    /// in the [`BattleDef`].
    #[serde(default, deserialize_with = "bare")]
    light: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawVillage {
    minimum: u32,
    #[serde(default)]
    rewards: Vec<RawLoot>,
}

/// A reward, as a map file writes a chest's loot.
#[derive(Deserialize)]
enum RawLoot {
    Gold(u32),
    Item(String),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawNote {
    text: String,
    /// Each a unit's `id`, or a character.
    #[serde(default)]
    units: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSlot {
    character: String,
    pos: (i32, i32),
    /// Numbered after the enemies instead of in slot order.
    #[serde(default)]
    after_enemies: bool,
    /// Only green units carry items: listed here, it is an error.
    #[serde(default)]
    items: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEnemy {
    /// A name for battle notes to refer to, written bare: `id: "gate_guard"`.
    #[serde(default, deserialize_with = "bare")]
    id: Option<String>,
    /// Written bare: `template: "brigand"`.
    #[serde(default, deserialize_with = "bare")]
    template: Option<String>,
    /// Written bare: `character: "rook"`.
    #[serde(default, deserialize_with = "bare")]
    character: Option<String>,
    #[serde(default)]
    level: Option<Level>,
    pos: (i32, i32),
    #[serde(default)]
    ai: AiBehavior,
    #[serde(default)]
    loadout: Option<RawLoadout>,
    #[serde(default)]
    boss: bool,
    /// What becomes of it at 0 HP: `Falls` (the default), `Retreats` or
    /// `GoesDown`.
    #[serde(default)]
    defeat: Defeat,
    #[serde(default)]
    name: Option<String>,
    /// `Enemy` (the default) or `Ally`, a green unit.
    #[serde(default = "enemy_faction")]
    faction: Faction,
    /// A green unit's own consumables (0513), by item id.
    #[serde(default)]
    items: Vec<String>,
    /// Burns the houses it ends its action on or next to (0313).
    #[serde(default)]
    raider: bool,
}

fn enemy_faction() -> Faction {
    Faction::Enemy
}

/// Reads an optional field written without `Some(…)`.
pub(crate) fn bare<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawReinforcement {
    turn: Turn,
    unit: RawEnemy,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
enum RawObjective {
    Rout {
        #[serde(default)]
        turn_limit: Option<Turn>,
    },
    DefeatUnit {
        unit: String,
        #[serde(default)]
        turn_limit: Option<Turn>,
        #[serde(default)]
        leftovers: Leftovers,
    },
    Seize {
        pos: (i32, i32),
        #[serde(default)]
        by_lord: bool,
        #[serde(default)]
        turn_limit: Option<Turn>,
    },
    Survive {
        turns: Turn,
    },
    Escape {
        area: TileRect,
    },
}

/// Loads every battle file in the bundle, keyed by id (the file stem).
/// Reports every error of every file.
pub fn load_all(refs: &BattleRefs<'_>) -> Result<BTreeMap<String, BattleDef>, Vec<ContentError>> {
    bundle::load_each(BATTLES_DIR, EXTENSION.trim_start_matches('.'), |file| {
        from_source(file.path, file.stem, file.text, refs)
    })
}

/// The light each battle file of the bundle names instead of its map's
/// (ticket 0438), by battle id (the file stem), checked against
/// `lighting`. A file that doesn't parse names none here: [`load_all`]
/// reports it.
pub fn load_lights(lighting: &Lighting) -> Result<BTreeMap<String, String>, Vec<ContentError>> {
    let mut lights = BTreeMap::new();
    let mut errors = Vec::new();
    for path in bundle::files_in(BATTLES_DIR) {
        let stem = path
            .strip_prefix(BATTLES_DIR)
            .and_then(|p| p.strip_prefix('/'))
            .and_then(|p| p.strip_suffix(EXTENSION));
        let (Some(stem), Some(source)) = (stem, bundle::file(path)) else {
            continue;
        };
        match light_of(&bundle::display_path(path), source, lighting) {
            Ok(Some(light)) => {
                lights.insert(stem.to_owned(), light);
            }
            Ok(None) => {}
            Err(e) => errors.push(e),
        }
    }
    if errors.is_empty() {
        Ok(lights)
    } else {
        Err(errors)
    }
}

/// The light battle file `source` names (`light: "dusk"`), if it names
/// one; an error at the name (attributed to `file`) if it isn't one of
/// `lighting`'s. `None` for a file that doesn't parse.
pub fn light_of(
    file: &str,
    source: &str,
    lighting: &Lighting,
) -> Result<Option<String>, ContentError> {
    let Ok(raw) = parse_ron::<RawBattle>(file, source) else {
        return Ok(None);
    };
    let Some(light) = raw.light else {
        return Ok(None);
    };
    if lighting.get(&light).is_some() {
        return Ok(Some(light));
    }
    let needle = format!("\"{light}\"");
    let at = source.lines().enumerate().find_map(|(i, line)| {
        let byte = line.find(&needle)?;
        Some((line_number(i), column_number(line[..byte].chars().count())))
    });
    let (line, col) = at.unwrap_or((1, 1));
    let message = format!(
        "light: \"{light}\" is not a light; the lights are {}",
        lighting.names()
    );
    Err(ContentError::new(file, message).at(line, Some(col)))
}

/// The enemies the file of battle `battle` gives an `id`, by that `id`,
/// as their unit ids: the names a cinematic's action knows them by
/// (ticket 0827). Only those on the map at the start. None for a battle
/// with no file in the bundle, or one that doesn't parse ([`load_all`]
/// reports that).
pub fn unit_names(battle: &str) -> BTreeMap<String, UnitId> {
    let path = format!("{BATTLES_DIR}/{battle}{EXTENSION}");
    bundle::file(&path).map(unit_names_of).unwrap_or_default()
}

/// The enemies battle file `source` gives an `id`, by it, as their unit
/// ids ([`from_source`] numbers them the same way); none for a file that
/// doesn't parse.
pub fn unit_names_of(source: &str) -> BTreeMap<String, UnitId> {
    let Ok(raw) = parse_ron::<RawBattle>("", source) else {
        return BTreeMap::new();
    };
    let early = raw.player_slots.iter().filter(|s| !s.after_enemies).count();
    let first = BattleDef::first_enemy_id(early);
    let numbered = (first..).zip(&raw.enemies);
    numbered
        .filter_map(|(id, enemy)| Some((enemy.id.clone()?, UnitId(id))))
        .collect()
}

/// Parses and validates battle file `source` (errors attributed to
/// `file`), whose id must be `stem`. Reports every problem found.
pub fn from_source(
    file: &str,
    stem: &str,
    source: &str,
    refs: &BattleRefs<'_>,
) -> Result<BattleDef, Vec<ContentError>> {
    let raw: RawBattle = parse_ron(file, source).map_err(|e| vec![e])?;
    let mut v = Checker::new(file, refs);
    if raw.id != stem {
        v.err(format!(
            "id \"{}\" must match the file name \"{stem}\"",
            raw.id
        ));
    }
    let Some(map) = refs.maps.get(&raw.map).map(|m| m.map.clone()) else {
        v.err(format!("no map \"{}\"", raw.map));
        return Err(v.errors);
    };
    v.no_decoration(&raw.map, &map);
    let slot_ids = slot_ids(&raw.player_slots, raw.enemies.len());
    let early = raw.player_slots.iter().filter(|s| !s.after_enemies).count();
    let mut next_id = BattleDef::first_enemy_id(early);
    let players = v.players(&raw.player_slots, &slot_ids, &map);
    let enemies: Vec<Unit> = raw
        .enemies
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            let unit = v.enemy(&format!("enemy {}", i + 1), e, UnitId(next_id), &map, true);
            next_id += 1;
            unit
        })
        .collect();
    // The slots numbered after the enemies come before the reinforcements.
    let late = raw.player_slots.iter().filter(|s| s.after_enemies).count();
    next_id = next_id.saturating_add(u32::try_from(late).unwrap_or(u32::MAX));
    let reinforcements: Vec<Reinforcement> = raw
        .reinforcements
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            let what = format!("reinforcement {}", i + 1);
            if r.turn == 0 {
                v.err(format!("{what}: turn 0; turns start at 1"));
            }
            let unit = v.enemy(&what, &r.unit, UnitId(next_id), &map, false);
            next_id += 1;
            Some(Reinforcement {
                turn: r.turn,
                unit: unit?,
            })
        })
        .collect();
    let everyone: Vec<Unit> = players
        .iter()
        .chain(&enemies)
        .chain(reinforcements.iter().map(|r| &r.unit))
        .cloned()
        .collect();
    let objective = v.objective(&raw.objective, &map, &everyone);
    let pack_cap = raw.pack_cap.unwrap_or(refs.items.rules.default_pack_cap);
    let default_pack = v.pack(&raw.default_pack, pack_cap, raw.preparations);
    let (solo_stock, solo_bench) = v.solo(&raw.solo_stock, &raw.solo_bench);
    v.music(&raw.music);
    v.errors.extend(check_map_labels(file, &everyone));
    v.errors.extend(check_triggers(
        file,
        &raw.triggers,
        &everyone,
        &map,
        refs.dialogue,
    ));
    let battle_notes = v.notes(&raw.battle_notes, &everyone);
    let exit_name = v.exit_name(raw.exit_name.as_deref(), objective.as_ref());
    let village = v.village(raw.village.as_ref(), &map, &everyone);
    if !v.errors.is_empty() {
        return Err(v.errors);
    }
    Ok(BattleDef {
        id: raw.id,
        map,
        player_slots: player_slots(&raw.player_slots, &slot_ids),
        enemies,
        reinforcements,
        preparations: raw.preparations,
        pack_cap,
        default_pack,
        solo_stock,
        solo_gold: raw.solo_gold,
        solo_bench,
        clear_gold: raw.clear_gold,
        objective: objective.unwrap_or(Objective::Rout { turn_limit: None }),
        triggers: raw.triggers,
        battle_notes,
        exit_name,
        village,
        difficulty: raw.difficulty,
        kind: raw.kind,
        music: raw.music,
        seed: raw.seed,
    })
}

fn pos((x, y): (i32, i32)) -> Pos {
    Pos::new(x, y)
}

/// The player slots of `slots`, with their unit ids.
fn player_slots(slots: &[RawSlot], ids: &[UnitId]) -> Vec<PlayerSlot> {
    slots
        .iter()
        .zip(ids)
        .map(|(s, &id)| PlayerSlot {
            character: CharacterId(s.character.clone()),
            pos: pos(s.pos),
            id,
        })
        .collect()
}

/// The unit id of each slot: the slots numbered in order from 1, skipping
/// those marked `after_enemies`, which follow the `enemies` enemies in
/// order.
fn slot_ids(slots: &[RawSlot], enemies: usize) -> Vec<UnitId> {
    let early = slots.iter().filter(|s| !s.after_enemies).count();
    let (mut next_early, mut next_late) = (0, early + enemies);
    slots
        .iter()
        .map(|s| {
            let next = if s.after_enemies {
                &mut next_late
            } else {
                &mut next_early
            };
            *next += 1;
            BattleDef::slot_id(*next - 1)
        })
        .collect()
}

/// Collects the errors of one battle file.
struct Checker<'a, 'r> {
    file: &'a str,
    refs: &'a BattleRefs<'r>,
    errors: Vec<ContentError>,
    /// Who stands on each starting tile.
    taken: BTreeMap<Pos, String>,
    /// Characters already placed.
    cast: BTreeSet<String>,
    /// The units given an `id`, for the battle notes.
    named: BTreeMap<String, UnitId>,
}

impl<'a, 'r> Checker<'a, 'r> {
    /// A checker of battle file `file`, with no errors yet.
    fn new(file: &'a str, refs: &'a BattleRefs<'r>) -> Self {
        Self {
            file,
            refs,
            errors: Vec::new(),
            taken: BTreeMap::new(),
            cast: BTreeSet::new(),
            named: BTreeMap::new(),
        }
    }

    fn err(&mut self, message: String) {
        self.errors.push(ContentError::new(self.file, message));
    }

    /// The player units the slots would hold, with the characters' own
    /// data (the campaign's roster replaces them in play).
    fn players(&mut self, slots: &[RawSlot], ids: &[UnitId], map: &BattleMap) -> Vec<Unit> {
        let refs = self.refs;
        let mut units = Vec::new();
        for (i, (slot, &unit_id)) in slots.iter().zip(ids).enumerate() {
            let what = format!("player slot {} (\"{}\")", i + 1, slot.character);
            if !slot.items.is_empty() {
                self.err(format!("{what}: only green units carry items"));
            }
            let id = CharacterId(slot.character.clone());
            let Some(def) = refs.characters.characters.get(&id) else {
                self.err(format!("{what}: no character \"{}\"", slot.character));
                continue;
            };
            let at = pos(slot.pos);
            match character_unit(def, unit_id, refs.classes, refs.items, Faction::Player, at) {
                Ok(unit) => {
                    self.place(&what, &unit, map, true);
                    units.push(unit);
                }
                Err(e) => self.err(format!("{what}: {e}")),
            }
        }
        units
    }

    /// The unit of enemy entry `e`, with id `id`. `start`: it is on the map
    /// at the start (so no other unit may share its tile).
    fn enemy(
        &mut self,
        what: &str,
        e: &RawEnemy,
        id: UnitId,
        map: &BattleMap,
        start: bool,
    ) -> Option<Unit> {
        let refs = self.refs;
        let at = pos(e.pos);
        if let Some(name) = &e.id
            && self.named.insert(name.clone(), id).is_some()
        {
            self.err(format!("{what}: id \"{name}\" is already used"));
        }
        let built = match (&e.template, &e.character) {
            (Some(t), None) => {
                let Some(template) = refs.characters.generics.get(t) else {
                    self.err(format!("{what}: no generic template \"{t}\""));
                    return None;
                };
                let mut template = template.clone();
                if let Some(level) = e.level {
                    if level < 1 || level > refs.classes.level_cap {
                        let cap = refs.classes.level_cap;
                        self.err(format!("{what}: level {level} is outside 1..={cap}"));
                    }
                    template.level = level;
                }
                if let Some(loadout) = &e.loadout {
                    template.loadout = loadout.to_def();
                }
                template.unit(id, refs.classes, refs.items, Faction::Enemy, at)
            }
            (None, Some(c)) => {
                let Some(def) = refs.characters.characters.get(&CharacterId(c.clone())) else {
                    self.err(format!("{what}: no character \"{c}\""));
                    return None;
                };
                if e.level.is_some() {
                    self.err(format!(
                        "{what}: a character's level comes from characters.ron"
                    ));
                }
                let mut def = def.clone();
                if let Some(loadout) = &e.loadout {
                    def.loadout = loadout.to_def();
                }
                character_unit(&def, id, refs.classes, refs.items, Faction::Enemy, at)
            }
            _ => {
                self.err(format!("{what}: give a template or a character, not both"));
                return None;
            }
        };
        let mut unit = match built {
            Ok(unit) => unit,
            Err(e) => {
                self.err(format!("{what}: {e}"));
                return None;
            }
        };
        unit.ai = e.ai;
        unit.defeat = e.defeat;
        unit.faction = self.faction(what, e);
        unit.carried = self.carried(what, e);
        if e.raider && unit.faction != Faction::Enemy {
            self.err(format!("{what}: only enemies are raiders"));
        }
        unit.raider = e.raider;
        if e.boss {
            unit.role = Role::Boss;
        }
        if let Some(name) = &e.name {
            unit.name.clone_from(name);
            unit.map_label = default_map_label(name);
        }
        self.place(what, &unit, map, start);
        Some(unit)
    }

    /// Checks no tile of `map` (map file `map_id`) is decoration, the
    /// terrain only the places behind conversations have (ticket 0733).
    fn no_decoration(&mut self, map_id: &str, map: &BattleMap) {
        let looks = self.refs.terrain_looks;
        let used: BTreeSet<&str> = map
            .tiles
            .cells()
            .iter()
            .filter(|&&id| looks.is_decoration(id))
            .filter_map(|&id| looks.get(id).map(|t| t.id.as_str()))
            .collect();
        for id in used {
            self.err(format!(
                "map \"{map_id}\" has \"{id}\", a terrain only for the places behind conversations"
            ));
        }
    }

    /// Checks `unit` stands on a tile of `map` it can stand on, as the only
    /// unit there (`start`), and that its character isn't placed twice.
    fn place(&mut self, what: &str, unit: &Unit, map: &BattleMap, start: bool) {
        let at = unit.pos;
        let class = self.refs.classes.get(&unit.class);
        let tile = map.tiles.get(at).copied();
        match (tile, class) {
            (None, _) => self.err(format!(
                "{what}: ({}, {}) is outside the {}×{} map",
                at.x,
                at.y,
                map.tiles.width(),
                map.tiles.height()
            )),
            (Some(t), Some(class))
                if self
                    .refs
                    .terrain
                    .move_cost(t, class.movement_type)
                    .is_none() =>
            {
                self.err(format!(
                    "{what}: a {} can't stand on ({}, {})",
                    class.name, at.x, at.y
                ));
            }
            _ => {}
        }
        if start {
            if let Some(other) = self.taken.get(&at) {
                let other = other.clone();
                self.err(format!(
                    "{what}: ({}, {}) is already taken by {other}",
                    at.x, at.y
                ));
            } else {
                self.taken.insert(at, what.to_owned());
            }
        }
        if let Some(c) = &unit.character
            && !self.cast.insert(c.0.clone())
        {
            self.err(format!("{what}: character \"{}\" is placed twice", c.0));
        }
    }

    fn objective(
        &mut self,
        raw: &RawObjective,
        map: &BattleMap,
        everyone: &[Unit],
    ) -> Option<Objective> {
        Some(match *raw {
            RawObjective::Rout { turn_limit } => Objective::Rout { turn_limit },
            RawObjective::DefeatUnit {
                ref unit,
                turn_limit,
                leftovers,
            } => {
                let target = everyone.iter().find(|u| {
                    u.faction == Faction::Enemy
                        && u.character.as_ref().is_some_and(|c| &c.0 == unit)
                });
                let Some(target) = target else {
                    self.err(format!(
                        "objective: no enemy of character \"{unit}\" to defeat"
                    ));
                    return None;
                };
                Objective::DefeatUnit {
                    unit: target.id,
                    turn_limit,
                    leftovers,
                }
            }
            RawObjective::Seize {
                pos: at,
                by_lord,
                turn_limit,
            } => {
                let at = pos(at);
                if !map.tiles.in_bounds(at) {
                    self.err(format!(
                        "objective: the seize tile ({}, {}) is outside the map",
                        at.x, at.y
                    ));
                }
                Objective::Seize {
                    pos: at,
                    by_lord,
                    turn_limit,
                }
            }
            RawObjective::Survive { turns } => {
                if turns == 0 {
                    self.err("objective: survive 0 turns".to_owned());
                }
                Objective::Survive { turns }
            }
            RawObjective::Escape { area } => {
                if let Some(message) = off_map(&area, map) {
                    self.err(format!("objective: the exit {message}"));
                }
                Objective::Escape { area }
            }
        })
        .inspect(|o| {
            if o.turn_limit() == Some(0) {
                self.err("objective: turn limit 0".to_owned());
            }
        })
    }

    /// The exit's name, trimmed: one line, and only on an escape map.
    fn exit_name(&mut self, name: Option<&str>, objective: Option<&Objective>) -> Option<String> {
        let name = name?.trim();
        if !matches!(objective, None | Some(Objective::Escape { .. })) {
            self.err("exit_name: only an escape map names its exit".to_owned());
        }
        if name.is_empty() {
            self.err("exit_name: no text".to_owned());
        }
        if name.chars().any(char::is_control) {
            self.err("exit_name: the name must be one line".to_owned());
        }
        Some(name.to_owned())
    }

    /// The village goal of `raw` on `map`: its houses are every `house`
    /// tile; the minimum is 1 to their number; one known reward, never
    /// `Gold(0)`, per house past the minimum. Without one, none of
    /// `everyone` may be a raider.
    fn village(
        &mut self,
        raw: Option<&RawVillage>,
        map: &BattleMap,
        everyone: &[Unit],
    ) -> Option<VillageGoal> {
        let Some(raw) = raw else {
            if everyone.iter().any(|u| u.raider) {
                self.err("raider: only a battle with a `village` has raiders".to_owned());
            }
            return None;
        };
        let looks = self.refs.terrain_looks;
        let house = looks.id_of("house");
        let burnt = looks.id_of("burnt").unwrap_or_else(|| {
            self.err("village: no \"burnt\" terrain for burned houses".to_owned());
            TerrainId(0)
        });
        let houses: Vec<Pos> = map
            .tiles
            .positions()
            .filter(|&p| house.is_some_and(|h| map.tiles.get(p) == Some(&h)))
            .collect();
        let total = u32::try_from(houses.len()).unwrap_or(u32::MAX);
        if raw.minimum == 0 {
            self.err("village: minimum 0; save at least 1 house".to_owned());
        } else if raw.minimum > total {
            self.err(format!(
                "village: minimum {} but the map has {total} houses",
                raw.minimum
            ));
        }
        let extra = total.saturating_sub(raw.minimum);
        let given = u32::try_from(raw.rewards.len()).unwrap_or(u32::MAX);
        if (1..=total).contains(&raw.minimum) && given != extra {
            self.err(format!(
                "village: {given} rewards; give one per house past the minimum ({extra})"
            ));
        }
        let rewards = raw
            .rewards
            .iter()
            .enumerate()
            .map(|(i, r)| match r {
                RawLoot::Gold(n) => {
                    if *n == 0 {
                        self.err(format!("village: reward {} is 0 gold", i + 1));
                    }
                    Loot::Gold(*n)
                }
                RawLoot::Item(item) => {
                    let id = ItemId::new(item);
                    if self.refs.items.get(&id).is_none() {
                        self.err(format!("village: reward {}: no item \"{item}\"", i + 1));
                    }
                    Loot::Item(id)
                }
            })
            .collect();
        Some(VillageGoal {
            houses,
            minimum: raw.minimum,
            rewards,
            burnt,
        })
    }

    /// The battle notes, their units resolved: each is a unit's `id`, or a
    /// character of the battle (one of `everyone`).
    fn notes(&mut self, notes: &[RawNote], everyone: &[Unit]) -> Vec<BattleNote> {
        let is = |u: &Unit, name: &str| u.character.as_ref().is_some_and(|c| c.0 == name);
        let clashes: Vec<String> = self
            .named
            .keys()
            .filter(|name| everyone.iter().any(|u| is(u, name)))
            .cloned()
            .collect();
        for name in clashes {
            self.err(format!("id \"{name}\" is also a character in the battle"));
        }
        if notes.len() > MAX_NOTES {
            self.err(format!(
                "battle_notes: {} notes, more than {MAX_NOTES}",
                notes.len()
            ));
        }
        let mut out = Vec::new();
        for (i, note) in notes.iter().enumerate() {
            let what = format!("battle note {}", i + 1);
            let text = note.text.trim();
            let chars = text.chars().count();
            if text.is_empty() {
                self.err(format!("{what}: no text"));
            } else if chars > MAX_NOTE_CHARS {
                self.err(format!(
                    "{what}: {chars} characters, more than {MAX_NOTE_CHARS}"
                ));
            }
            if text.chars().any(char::is_control) {
                self.err(format!("{what}: the text must be one line"));
            }
            let mut units = Vec::new();
            for name in &note.units {
                let found: Vec<UnitId> = match self.named.get(name) {
                    Some(&id) => vec![id],
                    None => everyone
                        .iter()
                        .filter(|u| is(u, name))
                        .map(|u| u.id)
                        .collect(),
                };
                if found.is_empty() {
                    self.err(format!("{what}: no unit \"{name}\" in the battle"));
                }
                for id in found {
                    if units.contains(&id) {
                        self.err(format!("{what}: \"{name}\" is listed twice"));
                    } else {
                        units.push(id);
                    }
                }
            }
            out.push(BattleNote {
                text: text.to_owned(),
                units,
            });
        }
        out
    }

    /// The music names a music cue, or a pool, of the audio manifest.
    fn music(&mut self, music: &BattleMusic) {
        let audio = self.refs.audio;
        match music {
            BattleMusic::Cue(cue) if !audio.music.contains_key(cue) => {
                self.err(format!("music: no music cue \"{cue}\""));
            }
            BattleMusic::Pool(pool) if !audio.pools.contains_key(pool) => {
                self.err(format!("music: no music pool \"{pool}\""));
            }
            _ => {}
        }
    }

    /// The stock and the bench of the battle played on its own.
    fn solo(&mut self, stock: &[String], bench: &[String]) -> (Vec<ItemId>, Vec<CharacterId>) {
        (self.solo_stock(stock), self.solo_bench(bench))
    }

    /// The default pack's items: known consumables, at most `cap`, and
    /// none in a battle with `preparations`.
    fn pack(&mut self, items: &[String], cap: usize, preparations: bool) -> Vec<ItemId> {
        if preparations && !items.is_empty() {
            self.err(PACK_WITH_PREPARATIONS.to_owned());
        }
        if items.len() > cap {
            self.err(format!(
                "default_pack: {} items, more than the pack cap {cap}",
                items.len()
            ));
        }
        for item in items {
            match self.refs.items.get(&ItemId::new(item)) {
                Some(ItemDef::Consumable(_)) => {}
                Some(_) => self.err(format!("default_pack: \"{item}\" isn't a consumable")),
                None => self.err(format!("default_pack: no item \"{item}\"")),
            }
        }
        items.iter().map(|i| ItemId::new(i)).collect()
    }

    /// Enemy entry `e`'s faction: `Enemy` or `Ally`, and an `Ally` is
    /// neither a boss nor anything but `Falls` at 0 HP.
    fn faction(&mut self, what: &str, e: &RawEnemy) -> Faction {
        match e.faction {
            Faction::Enemy => {}
            Faction::Ally => {
                if e.boss {
                    self.err(format!("{what}: a green unit can't be a boss"));
                }
                if e.defeat != Defeat::Falls {
                    self.err(format!("{what}: only enemies have a `defeat`"));
                }
            }
            other => self.err(format!("{what}: faction {other:?}; use Enemy or Ally")),
        }
        e.faction
    }

    /// Enemy entry `e`'s carried items: known consumables, on a green unit.
    fn carried(&mut self, what: &str, e: &RawEnemy) -> Vec<ItemId> {
        if e.faction != Faction::Ally && !e.items.is_empty() {
            self.err(format!("{what}: only green units carry items"));
        }
        for item in &e.items {
            match self.refs.items.get(&ItemId::new(item)) {
                Some(ItemDef::Consumable(_)) => {}
                Some(_) => self.err(format!("{what}: item \"{item}\" isn't a consumable")),
                None => self.err(format!("{what}: no item \"{item}\"")),
            }
        }
        e.items.iter().map(|i| ItemId::new(i)).collect()
    }

    /// The solo bench's characters: known, and not in the battle.
    fn solo_bench(&mut self, characters: &[String]) -> Vec<CharacterId> {
        for (i, c) in characters.iter().enumerate() {
            if !self
                .refs
                .characters
                .characters
                .contains_key(&CharacterId(c.clone()))
            {
                self.err(format!("solo_bench: no character \"{c}\""));
            } else if self.cast.contains(c) || characters[..i].contains(c) {
                self.err(format!(
                    "solo_bench: \"{c}\" is already in the battle or listed twice"
                ));
            }
        }
        characters.iter().map(|c| CharacterId(c.clone())).collect()
    }

    /// The solo stock's items: any known items.
    fn solo_stock(&mut self, items: &[String]) -> Vec<ItemId> {
        for item in items {
            if self.refs.items.get(&ItemId::new(item)).is_none() {
                self.err(format!("solo_stock: no item \"{item}\""));
            }
        }
        items.iter().map(|i| ItemId::new(i)).collect()
    }
}

#[cfg(test)]
mod tests;
