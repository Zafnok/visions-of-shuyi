//! World map files (`assets/worldmaps/*.ron`, ticket 1007, ADR-0099): one
//! act's places and roads (the rules, [`trpg_core::WorldMap`]), and how
//! they are drawn (where each place is, the roads' turns, the ground, the
//! act's music). See `assets/worldmaps/README.md`.
use crate::prelude::*;

use alloc::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::Deserialize;
use trpg_core::{
    BattleDef, CharacterId, Faction, ItemId, ItemTable, NodeCondition, NodeKind, Pos, Shop,
    ShopKind, SideQuest, WorldMap, WorldNode,
};

use crate::audio::AudioManifest;
use crate::bundle;
use crate::chapter::ChapterDef;
use crate::character::CharacterTable;
use crate::dialogue::DialogueTable;
use crate::error::ContentError;
use crate::map::check_shop;
use crate::ron_loader::parse_ron;

/// Directory of the world map files inside the asset bundle.
pub const WORLDMAPS_DIR: &str = "worldmaps";

/// The most cells a world map may be across and down: what the screen
/// shows of it, the whole map at once (no scrolling yet).
pub const MAX_SIZE: (i32, i32) = (70, 30);

/// What each character of a map's `ground` is (the glyph look's ground;
/// the skin decides how each looks).
pub const GROUND: [(char, &str); 9] = [
    (' ', "plain"),
    ('.', "plain"),
    (',', "plain"),
    ('f', "forest"),
    ('t', "thicket"),
    ('^', "mountain"),
    ('A', "peak"),
    ('w', "water"),
    ('~', "sea"),
];

/// One act's world map: its rules and its look.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorldMapDef {
    /// The places and roads (the rules).
    pub map: WorldMap,
    /// Each place's name, by place id.
    pub names: BTreeMap<String, String>,
    /// The music cue the map plays (`audio.md`, *The world map*).
    pub music: String,
    /// The music cue of a place, by place id, played while the army is
    /// there.
    pub place_music: BTreeMap<String, String>,
    /// Where each place is drawn, by place id, in cells of the map.
    pub places: BTreeMap<String, Pos>,
    /// Each road's cells where it turns, from one place to the other: the
    /// road runs straight from one to the next. In the order of the map's
    /// roads.
    pub roads: Vec<Vec<Pos>>,
    /// The map's size in cells, across and down.
    pub size: (i32, i32),
    /// The ground, a row of characters ([`GROUND`]) a line; none: plain.
    pub ground: Vec<String>,
    /// The picture the picture look draws under the places (an image
    /// path in the bundle), if the map has one.
    pub picture: Option<String>,
}

impl WorldMapDef {
    /// The name of place `id` (its id if it has none).
    pub fn name<'a>(&'a self, id: &'a str) -> &'a str {
        self.names.get(id).map_or(id, String::as_str)
    }

    /// What the ground is at `pos` ([`GROUND`]'s names): `plain` where the
    /// map says nothing.
    pub fn ground_at(&self, pos: Pos) -> &'static str {
        let ch = usize::try_from(pos.y)
            .ok()
            .and_then(|y| self.ground.get(y))
            .and_then(|row| row.chars().nth(usize::try_from(pos.x).ok()?));
        ch.and_then(|c| GROUND.iter().find(|(g, _)| *g == c))
            .map_or("plain", |(_, name)| name)
    }
}

/// A condition as written.
#[derive(Debug, Clone, Deserialize)]
enum RawCondition {
    Cleared(String),
    Flag(String),
    Alive(String),
}

/// A shop as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawShop {
    kind: ShopKind,
    #[serde(default)]
    stock: Vec<String>,
}

/// A place's kind as written.
#[derive(Debug, Clone, Deserialize)]
enum RawKind {
    StoryBattle(String),
    SideQuest(SideQuest),
    Town {
        shops: Vec<RawShop>,
        #[serde(default)]
        capital: bool,
    },
    Empty,
}

/// A place as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawNode {
    id: String,
    name: String,
    pos: (i32, i32),
    kind: RawKind,
    #[serde(default)]
    music: Option<String>,
    #[serde(default)]
    opens_when: Vec<RawCondition>,
}

/// A road as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRoad {
    from: String,
    to: String,
    #[serde(default)]
    via: Vec<(i32, i32)>,
}

/// A world map file as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWorldMap {
    act: u32,
    levels: (u32, u32),
    music: String,
    start: String,
    final_node: String,
    #[serde(default)]
    next: Option<String>,
    size: (i32, i32),
    #[serde(default)]
    ground: Vec<String>,
    #[serde(default)]
    picture: Option<String>,
    nodes: Vec<RawNode>,
    #[serde(default)]
    roads: Vec<RawRoad>,
}

/// What a world map is checked against.
#[derive(Clone, Copy)]
pub struct WorldRefs<'a> {
    /// Battles by id.
    pub battles: &'a BTreeMap<String, BattleDef>,
    /// Chapters by id.
    pub chapters: &'a BTreeMap<String, ChapterDef>,
    /// Dialogue scenes.
    pub dialogue: &'a DialogueTable,
    /// Characters.
    pub characters: &'a CharacterTable,
    /// Items.
    pub items: &'a ItemTable,
    /// Music cues.
    pub audio: &'a AudioManifest,
}

/// The average level of `battle`'s enemies, rounded to the nearest; `None`
/// with no enemies.
pub fn enemy_level(battle: &BattleDef) -> Option<u32> {
    let levels: Vec<u32> = battle
        .enemies
        .iter()
        .filter(|u| u.faction == Faction::Enemy)
        .map(|u| u.level)
        .collect();
    let count = u32::try_from(levels.len()).ok().filter(|n| *n > 0)?;
    let sum: u32 = levels.iter().sum();
    Some((sum + count / 2) / count)
}

/// The battle of a place's kind, by id.
fn battle_of<'a>(kind: &'a RawKind, chapters: &'a BTreeMap<String, ChapterDef>) -> Option<&'a str> {
    match kind {
        RawKind::StoryBattle(chapter) => chapters.get(chapter).map(|c| c.battle.as_str()),
        RawKind::SideQuest(quest) => Some(&quest.battle),
        RawKind::Town { .. } | RawKind::Empty => None,
    }
}

/// Builds world map `id` from `raw`, with every problem of it (`err`).
fn build(id: &str, raw: RawWorldMap, refs: WorldRefs<'_>, err: &mut dyn FnMut(String)) -> WorldMapDef {
    let mut def = WorldMapDef {
        music: raw.music,
        size: raw.size,
        ground: raw.ground,
        picture: raw.picture,
        ..WorldMapDef::default()
    };
    let mut nodes = Vec::new();
    for node in raw.nodes {
        let level = battle_of(&node.kind, refs.chapters)
            .and_then(|b| refs.battles.get(b))
            .and_then(enemy_level);
        let kind = match node.kind {
            RawKind::StoryBattle(chapter) => NodeKind::StoryBattle(chapter),
            RawKind::SideQuest(quest) => NodeKind::SideQuest(quest),
            RawKind::Town { shops, capital } => NodeKind::Town {
                shops: shops
                    .into_iter()
                    .map(|s| Shop {
                        kind: s.kind,
                        stock: s.stock.into_iter().map(ItemId).collect(),
                    })
                    .collect(),
                capital,
            },
            RawKind::Empty => NodeKind::Empty,
        };
        let opens_when = node
            .opens_when
            .into_iter()
            .map(|c| match c {
                RawCondition::Cleared(id) => NodeCondition::Cleared(id),
                RawCondition::Flag(flag) => NodeCondition::Flag(flag),
                RawCondition::Alive(who) => NodeCondition::Alive(CharacterId(who)),
            })
            .collect();
        if def.places.insert(node.id.clone(), Pos::new(node.pos.0, node.pos.1)).is_some() {
            err(format!("place \"{}\" is listed twice", node.id));
        }
        def.names.insert(node.id.clone(), node.name);
        if let Some(music) = node.music {
            def.place_music.insert(node.id.clone(), music);
        }
        nodes.push(WorldNode {
            id: node.id,
            kind,
            level,
            opens_when,
        });
    }
    let mut roads = Vec::new();
    for road in raw.roads {
        let (Some(from), Some(to)) = (def.places.get(&road.from), def.places.get(&road.to)) else {
            err(format!(
                "the road from \"{}\" to \"{}\" doesn't join two places of the map",
                road.from, road.to
            ));
            continue;
        };
        let mut cells = vec![*from];
        cells.extend(road.via.iter().map(|&(x, y)| Pos::new(x, y)));
        cells.push(*to);
        def.roads.push(cells);
        roads.push((road.from, road.to));
    }
    def.map = WorldMap {
        id: id.to_owned(),
        act: raw.act,
        levels: raw.levels,
        start: raw.start,
        final_node: raw.final_node,
        next: raw.next,
        nodes,
        roads,
    };
    def
}

/// Every problem with world map `def` (a [`build`]'s), against `refs`.
fn check(def: &WorldMapDef, refs: WorldRefs<'_>, err: &mut dyn FnMut(String)) {
    let map = &def.map;
    check_places(def, refs, err);
    check_look(def, err);
    match map.node(&map.start) {
        None => err(format!("start: no place \"{}\"", map.start)),
        Some(n) if !n.opens_when.is_empty() || n.kind.is_battle() => {
            err(format!("start \"{}\" must be open from the start and not a battle", n.id));
        }
        Some(_) => {}
    }
    match map.node(&map.final_node).map(|n| &n.kind) {
        Some(NodeKind::StoryBattle(_)) => {}
        _ => err(format!(
            "final_node \"{}\" must be one of the map's story battles",
            map.final_node
        )),
    }
    if map.levels.0 > map.levels.1 {
        err(format!("levels {:?}: the lowest is above the highest", map.levels));
    }
    if !refs.audio.music.contains_key(&def.music) {
        err(format!("music: no music cue \"{}\"", def.music));
    }
    for (place, cue) in &def.place_music {
        if !refs.audio.music.contains_key(cue) {
            err(format!("place \"{place}\": no music cue \"{cue}\""));
        }
    }
    for id in unreachable(map) {
        err(format!("place \"{id}\" can't be reached from the start by any road"));
    }
}

/// Every problem with the places of `def`: their kinds' references, their
/// conditions, their shops.
fn check_places(def: &WorldMapDef, refs: WorldRefs<'_>, err: &mut dyn FnMut(String)) {
    let map = &def.map;
    let ids: BTreeSet<&str> = map.nodes.iter().map(|n| n.id.as_str()).collect();
    for node in &map.nodes {
        let id = &node.id;
        match &node.kind {
            NodeKind::StoryBattle(chapter) => match refs.chapters.get(chapter) {
                None => err(format!("place \"{id}\": no chapter \"{chapter}\"")),
                Some(c) if c.world_map.as_deref() != Some(map.id.as_str()) => err(format!(
                    "place \"{id}\": chapter \"{chapter}\" must say `world_map: Some(\"{}\")`",
                    map.id
                )),
                Some(_) => {}
            },
            NodeKind::SideQuest(quest) => {
                if !refs.battles.contains_key(&quest.battle) {
                    err(format!("place \"{id}\": no battle \"{}\"", quest.battle));
                }
                for scene in quest.intro_scenes.iter().chain(&quest.victory_scenes) {
                    if refs.dialogue.get(scene).is_none() {
                        err(format!("place \"{id}\": no scene \"{scene}\""));
                    }
                }
            }
            NodeKind::Town { shops, .. } => check_town(id, shops, refs.items, err),
            NodeKind::Empty => {}
        }
        for condition in &node.opens_when {
            match condition {
                NodeCondition::Cleared(other) if !ids.contains(other.as_str()) => {
                    err(format!("place \"{id}\": Cleared(\"{other}\") names no place"));
                }
                NodeCondition::Alive(who) if !refs.characters.characters.contains_key(who) => {
                    err(format!("place \"{id}\": Alive(\"{}\") names no character", who.0));
                }
                _ => {}
            }
        }
    }
    let stories = map
        .nodes
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::StoryBattle(c) => Some(c),
            _ => None,
        });
    let mut seen = BTreeSet::new();
    for chapter in stories {
        if !seen.insert(chapter) {
            err(format!("chapter \"{chapter}\" has two story battles"));
        }
    }
}

/// Every problem with town `id`'s `shops`.
fn check_town(id: &str, shops: &[Shop], items: &ItemTable, err: &mut dyn FnMut(String)) {
    if shops.is_empty() {
        err(format!("town \"{id}\" has no shop"));
    }
    for shop in shops {
        match (shop.kind, shop.stock.is_empty()) {
            (ShopKind::Basic, _) => {
                err(format!("town \"{id}\": the basic shop is only on Preparations"));
            }
            (ShopKind::Blacksmith, false) => {
                err(format!("town \"{id}\": a Blacksmith repairs, it sells nothing"));
            }
            (ShopKind::Blacksmith, true) => {}
            (_, true) => err(format!("town \"{id}\": a {:?} sells nothing", shop.kind)),
            (_, false) => {}
        }
        for problem in check_shop(shop, items) {
            err(format!("town \"{id}\": {problem}"));
        }
    }
}

/// Every problem with how `def` is drawn: its size, its ground, where its
/// places are and the turns of its roads.
fn check_look(def: &WorldMapDef, err: &mut dyn FnMut(String)) {
    let (w, h) = def.size;
    if w < 1 || h < 1 || w > MAX_SIZE.0 || h > MAX_SIZE.1 {
        err(format!("size {:?} must be from (1, 1) to {MAX_SIZE:?}", def.size));
    }
    if !def.ground.is_empty() && def.ground.len() != usize::try_from(h).unwrap_or(0) {
        err(format!("ground has {} rows, not {h}", def.ground.len()));
    }
    for (y, row) in def.ground.iter().enumerate() {
        if row.chars().count() != usize::try_from(w).unwrap_or(0) {
            err(format!("ground row {y} isn't {w} characters"));
        }
        if let Some(c) = row.chars().find(|c| !GROUND.iter().any(|(g, _)| g == c)) {
            err(format!("ground row {y}: '{c}' is no ground"));
        }
    }
    // A place's plate is three cells wide and its level marker sits
    // above it.
    for (id, pos) in &def.places {
        if pos.x < 1 || pos.x > w - 2 || pos.y < 1 || pos.y >= h {
            err(format!("place \"{id}\" at {pos:?} is too near the map's edge"));
        }
    }
    for (cells, (from, to)) in def.roads.iter().zip(&def.map.roads) {
        let bent = cells.windows(2).any(|p| p[0].x != p[1].x && p[0].y != p[1].y);
        if bent {
            err(format!(
                "the road from \"{from}\" to \"{to}\" must run straight across or down between its turns"
            ));
        }
    }
}

/// The places no road leads to from the start, once every condition holds.
fn unreachable(map: &WorldMap) -> Vec<String> {
    let mut seen = BTreeSet::from([map.start.as_str()]);
    let mut queue = VecDeque::from([map.start.as_str()]);
    while let Some(here) = queue.pop_front() {
        for (a, b) in &map.roads {
            let next = if a == here {
                b
            } else if b == here {
                a
            } else {
                continue;
            };
            if seen.insert(next.as_str()) {
                queue.push_back(next);
            }
        }
    }
    map.nodes
        .iter()
        .filter(|n| !seen.contains(n.id.as_str()) && map.node(&map.start).is_some())
        .map(|n| n.id.clone())
        .collect()
}

/// Loads every world map in the bundle, keyed by id (the file stem),
/// checking each against `refs` and the maps against each other (`next`
/// names a map, a chapter's `world_map` names one with its story battle).
/// Reports every error of every file.
pub fn load_all(refs: WorldRefs<'_>) -> Result<BTreeMap<String, WorldMapDef>, Vec<ContentError>> {
    let mut errors = Vec::new();
    let loaded = bundle::load_each(WORLDMAPS_DIR, "ron", |file| {
        Ok(match parse_ron::<RawWorldMap>(file.path, file.text) {
            Ok(raw) => {
                let mut problems = Vec::new();
                let def = build(file.stem, raw, refs, &mut |m| problems.push(m));
                check(&def, refs, &mut |m| problems.push(m));
                errors.extend(problems.into_iter().map(|m| ContentError::new(file.path, m)));
                Some(def)
            }
            Err(e) => {
                errors.push(e);
                None
            }
        })
    });
    let maps = bundle::parsed(loaded, &mut errors);
    errors.extend(check_links(&maps, refs.chapters));
    if errors.is_empty() {
        Ok(maps)
    } else {
        Err(errors)
    }
}

/// Every problem between the maps and the chapters: a `next` naming no
/// map, a chapter's `world_map` naming no map or one without its story
/// battle.
fn check_links(
    maps: &BTreeMap<String, WorldMapDef>,
    chapters: &BTreeMap<String, ChapterDef>,
) -> Vec<ContentError> {
    let mut errors = Vec::new();
    for (id, def) in maps {
        if let Some(next) = &def.map.next
            && !maps.contains_key(next)
        {
            let file = format!("{WORLDMAPS_DIR}/{id}.ron");
            errors.push(ContentError::new(file, format!("next: no world map \"{next}\"")));
        }
    }
    for chapter in chapters.values() {
        let Some(map) = &chapter.world_map else {
            continue;
        };
        let has_it = maps.get(map).is_some_and(|m| {
            m.map
                .nodes
                .iter()
                .any(|n| n.kind == NodeKind::StoryBattle(chapter.id.clone()))
        });
        if !has_it {
            let file = format!("{}/{}.ron", crate::chapter::CHAPTERS_DIR, chapter.id);
            errors.push(ContentError::new(
                file,
                format!("world_map: \"{map}\" is no world map with this chapter's story battle"),
            ));
        }
    }
    errors
}

#[cfg(test)]
mod tests;
