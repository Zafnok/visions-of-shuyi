//! The world map (ticket 1007): from Chapter 4 on, each act is a map of
//! places joined by roads, and the army moves between them.
//!
//! # Rules
//!
//! Sources: `docs/design/world-structure.md` (the map, its places, saving
//! and camp, acts), Nick's answers on 1007 (`docs/design/look-and-feel.md`,
//! *The world map*).
//!
//! - **A map** ([`WorldMap`], from `assets/worldmaps/`) is one act: its
//!   places ([`WorldNode`]), the roads between them, where the army starts,
//!   its final story battle and the act's map after it. Where the places
//!   are drawn, and how, is the map file's look, not the rules.
//! - **Places** ([`NodeKind`]): a **story battle** (a chapter's battle,
//!   whose chapter lives on this map), a **side quest** (a battle with its
//!   own scenes), a **town** (its shops) and an **empty** place to pass
//!   through. Fixed skirmishes name authored battles; saved random groups overlay cleared battle places (1008).
//! - **Open places** ([`WorldMap::is_open`]): a place can be reached once
//!   every one of its conditions ([`NodeCondition`]) holds; a story battle
//!   also waits for its chapter to be the campaign's
//!   [`chapter`](Campaign::chapter). A cleared place stays open. A side
//!   quest whose character died before it was played closes again: missed
//!   for good.
//! - **Shown places** ([`WorldMap::is_shown`], Nick, 1007): the open places,
//!   and the places not open yet one road away from an open one, which
//!   show what they are but can't be travelled to, unless the map marks
//!   them [`hidden`](WorldNode::hidden): those stay out of sight until they
//!   open (`world-structure.md`: a side quest is "discovered" when it is
//!   shown).
//! - **Open roads**: a road is open when both its places are; a road from an
//!   open place to a shown one that isn't open yet is shown, not open.
//! - **Travel** ([`WorldCommand::TravelTo`]): the army goes to an open
//!   place along open roads, the fewest roads first. It never passes
//!   *through* a guarding battle not yet won: on the way, an unwon battle stops the
//!   army there (*Claude's starting rule*), and a route around one is taken
//!   when there is one. Non-guarding fixed fights and random groups may be passed.
//! - **Battles are forced** (Nick, 1007): arriving at a battle not yet won
//!   starts it ([`WorldEvent::BattleAhead`]). Nothing else opens by itself
//!   in the rules; whether a town opens on arrival is the player's setting.
//! - **Entering** ([`WorldCommand::Enter`]) the army's place: a town opens
//!   ([`WorldEvent::Entered`]); a battle not yet won starts.
//! - **Towns** ([`Campaign::trade`]): a town's shops, whose stock grows with
//!   the story ([`WorldMap::town_shops`]), trade with the
//!   party's stock and gold, as between chapters ([`ShopSession`]); its
//!   Blacksmith also repairs the weapons the army holds (*Claude's starting
//!   rule*: a town shop sells only from the stock, since outside
//!   Preparations nothing moves gear there).
//! - **Winning a battle** ([`Campaign::clear_node`]) marks its place
//!   cleared, which opens what waited for it. Winning the map's **final**
//!   story battle moves the army to the next act's map, at its start, with
//!   nothing cleared: the old map can't be reached again.
//!
//! State changes only through [`Campaign::world_command`] and
//! [`Campaign::clear_node`], which return what happened as
//! [`WorldEvent`]s (ADR-0004).

use crate::prelude::*;

use alloc::collections::{BTreeMap, BTreeSet, VecDeque};

mod skirmish;
pub use skirmish::{RandomSkirmish, SkirmishRules};

use serde::{Deserialize, Serialize};

use crate::campaign::Campaign;
use crate::item::{ItemId, ItemTable};
use crate::prep::StockItem;
use crate::shop::{self, Gold, Shop, ShopError, ShopSession};
use crate::unit::CharacterId;

/// When a place opens: every condition of it must hold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeCondition {
    /// This place of the map is cleared (its battle won).
    Cleared(String),
    /// This story flag is set.
    Flag(String),
    /// This character is in the army (a side quest's character, alive and
    /// recruited).
    Alive(CharacterId),
    /// The campaign is in one of these chapters, by id (a skirmish that
    /// shows up only then, ticket 0510). Once cleared the place stays open,
    /// as every cleared place does.
    Chapter(Vec<String>),
    /// Never holds: the place's battle isn't made yet, and the ticket that
    /// makes it removes this (1070: a map can name what it will hold).
    NotBuilt,
}

/// More for one of a town's shops to sell once `when` holds: the stock
/// growing with the story (`world-structure.md`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShopGrowth {
    /// The shop, by index in the town's list.
    pub shop: usize,
    /// When it comes on sale: every condition must hold.
    pub when: Vec<NodeCondition>,
    /// What comes on sale.
    pub stock: Vec<ItemId>,
}

/// A side quest's battle and its scenes.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SideQuest {
    /// The battle, by id (`assets/battles/`).
    pub battle: String,
    /// Scenes played on arrival, before the battle.
    #[serde(default)]
    pub intro_scenes: Vec<String>,
    /// Scenes played after a victory.
    #[serde(default)]
    pub victory_scenes: Vec<String>,
}

/// What a place is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    /// A chapter's story battle: the chapter, by id, whose battle it is.
    StoryBattle(String),
    /// A side quest (paralogue).
    SideQuest(SideQuest),
    /// A hand-placed battle cleared once; only guards stop travel past it.
    FixedSkirmish {
        /// Battle and optional scenes.
        battle: SideQuest,
        /// Whether it blocks travel through its place until won.
        guards_path: bool,
    },
    /// A town and its shops.
    Town {
        /// Its shops, in the order the town lists them, as they start.
        shops: Vec<Shop>,
        /// What they add later ([`WorldMap::town_shops`]).
        #[serde(default)]
        later: Vec<ShopGrowth>,
        /// Whether it is a capital (its music, `audio.md`).
        #[serde(default)]
        capital: bool,
    },
    /// A place to pass through.
    Empty,
}

impl NodeKind {
    /// Whether it is a battle.
    pub fn is_battle(&self) -> bool {
        matches!(
            self,
            Self::StoryBattle(_) | Self::SideQuest(_) | Self::FixedSkirmish { .. }
        )
    }
}

/// One place of a world map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldNode {
    /// Its id, unique on its map.
    pub id: String,
    /// What it is.
    pub kind: NodeKind,
    /// A battle's enemy level (the average of its enemies'), for its level
    /// marker; `None` for a place that isn't a battle.
    #[serde(default)]
    pub level: Option<u32>,
    /// When it opens; empty: open from the start.
    #[serde(default)]
    pub opens_when: Vec<NodeCondition>,
    /// Whether it stays out of sight until it opens, even one road from
    /// an open place (a secret).
    #[serde(default)]
    pub hidden: bool,
}

/// One act's world map: its places and roads (see the module docs).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WorldMap {
    /// Its id (the file stem).
    pub id: String,
    /// The act it is, from 1.
    pub act: u32,
    /// The act's enemy levels, lowest and highest (random skirmishes are
    /// capped against it, 1008).
    pub levels: (u32, u32),
    /// Where the army starts.
    pub start: String,
    /// The act's final story battle: winning it moves on to [`next`](Self::next).
    pub final_node: String,
    /// The next act's map, if any.
    pub next: Option<String>,
    /// The places.
    pub nodes: Vec<WorldNode>,
    /// Random-skirmish settings, absent before this act has a pool.
    pub skirmishes: Option<SkirmishRules>,
    /// The roads, each between two places.
    pub roads: Vec<(String, String)>,
}

/// Where the army is on its act's map; saved with the campaign.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WorldState {
    /// The act's map, by id.
    pub map: String,
    /// The place the army is at.
    pub at: String,
    /// The places whose battle is won.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub cleared: BTreeSet<String>,
    /// Random groups by place; the underlying place remains cleared.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub skirmishes: BTreeMap<String, RandomSkirmish>,
}

/// What the player asks of the army on the world map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldCommand {
    /// Go to this place.
    TravelTo(String),
    /// Enter the place the army is at.
    Enter,
}

/// What happened on the world map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldEvent {
    /// The army went along these places, the first where it was and the
    /// last where it is now.
    Travelled(Vec<String>),
    /// The army is at a battle not yet won: it starts.
    BattleAhead(String),
    /// The army entered this town.
    Entered(String),
    /// This place's battle is won.
    Cleared(String),
    /// A random group appeared on a cleared place.
    SkirmishSpawned {
        /// Its place.
        node: String,
        /// The saved group.
        group: RandomSkirmish,
    },
    /// Existing groups were lowered to the latest cap, or left when no valid level remains.
    SkirmishesCapped(u32),
    /// This place opened.
    Opened(String),
    /// The act's map gave way to the next one, by id.
    NewAct(String),
}

/// Why a world-map command can't be done; nothing changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldError {
    /// The campaign isn't on this map.
    NotOnMap,
    /// The map has no such place.
    NoSuchNode,
    /// The place isn't open.
    Locked,
    /// No open road leads there.
    NoRoad,
    /// The army is already there.
    AlreadyThere,
    /// The army's place has nothing to enter.
    NothingHere,
}

/// A deal in a town's shop ([`Campaign::trade`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TownTrade {
    /// Buy one of this item into the stock.
    Buy(ItemId),
    /// Sell one of this from the stock.
    Sell(StockItem),
    /// Repair the stock's weapon copy at this index.
    RepairStock(usize),
    /// Repair the weapon in slot `slot` of the roster's unit `unit` (by
    /// index).
    RepairHeld {
        /// The unit's index in the roster.
        unit: usize,
        /// The weapon slot.
        slot: usize,
    },
}

impl WorldMap {
    /// Place `id`.
    pub fn node(&self, id: &str) -> Option<&WorldNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Whether place `id` is open for `campaign` (see the module docs).
    pub fn is_open(&self, id: &str, campaign: &Campaign) -> bool {
        let Some(node) = self.node(id) else {
            return false;
        };
        if self.is_cleared(id, campaign) {
            return true;
        }
        if let NodeKind::StoryBattle(chapter) = &node.kind
            && *chapter != campaign.chapter
        {
            return false;
        }
        node.opens_when.iter().all(|c| self.holds(c, campaign))
    }

    /// Whether place `id` is cleared in `campaign` (on this map).
    pub fn is_cleared(&self, id: &str, campaign: &Campaign) -> bool {
        campaign
            .world
            .as_ref()
            .is_some_and(|w| w.map == self.id && w.cleared.contains(id))
    }

    /// Whether `condition` holds for `campaign`.
    fn holds(&self, condition: &NodeCondition, campaign: &Campaign) -> bool {
        match condition {
            NodeCondition::Cleared(id) => self.is_cleared(id, campaign),
            NodeCondition::Flag(flag) => campaign.flags.get(flag) == Some(&true),
            NodeCondition::Alive(character) => campaign.member(character).is_some(),
            NodeCondition::Chapter(chapters) => chapters.contains(&campaign.chapter),
            NodeCondition::NotBuilt => false,
        }
    }

    /// Town `id`'s shops as they are now for `campaign`: what they start
    /// with and what has come on sale since. Empty for anything but a town.
    pub fn town_shops(&self, id: &str, campaign: &Campaign) -> Vec<Shop> {
        let Some(NodeKind::Town { shops, later, .. }) = self.node(id).map(|n| &n.kind) else {
            return Vec::new();
        };
        let mut shops = shops.clone();
        for growth in later {
            if growth.when.iter().all(|c| self.holds(c, campaign))
                && let Some(shop) = shops.get_mut(growth.shop)
            {
                shop.stock.extend(growth.stock.iter().cloned());
            }
        }
        shops
    }

    /// The open places, in the map's order.
    pub fn open_nodes<'a>(&'a self, campaign: &'a Campaign) -> impl Iterator<Item = &'a WorldNode> {
        self.nodes.iter().filter(|n| self.is_open(&n.id, campaign))
    }

    /// Whether place `id` is shown (see the module docs): open, or one
    /// road from an open place and not [`hidden`](WorldNode::hidden).
    pub fn is_shown(&self, id: &str, campaign: &Campaign) -> bool {
        if self.is_open(id, campaign) {
            return true;
        }
        let Some(node) = self.node(id).filter(|n| !n.hidden) else {
            return false;
        };
        self.roads.iter().any(|(a, b)| {
            (a == &node.id && self.is_open(b, campaign))
                || (b == &node.id && self.is_open(a, campaign))
        })
    }

    /// The shown places, in the map's order.
    pub fn shown_nodes<'a>(
        &'a self,
        campaign: &'a Campaign,
    ) -> impl Iterator<Item = &'a WorldNode> {
        self.nodes.iter().filter(|n| self.is_shown(&n.id, campaign))
    }

    /// The roads shown but not open: from an open place to a shown one that
    /// isn't open yet, in the map's order.
    pub fn locked_roads<'a>(
        &'a self,
        campaign: &'a Campaign,
    ) -> impl Iterator<Item = &'a (String, String)> {
        self.roads.iter().filter(|(a, b)| {
            let (open_a, open_b) = (self.is_open(a, campaign), self.is_open(b, campaign));
            (open_a && !open_b && self.is_shown(b, campaign))
                || (open_b && !open_a && self.is_shown(a, campaign))
        })
    }

    /// The open roads, in the map's order.
    pub fn open_roads<'a>(
        &'a self,
        campaign: &'a Campaign,
    ) -> impl Iterator<Item = &'a (String, String)> {
        self.roads
            .iter()
            .filter(|(a, b)| self.is_open(a, campaign) && self.is_open(b, campaign))
    }

    /// Whether place `id` holds a battle `campaign` hasn't won.
    pub fn is_unwon_battle(&self, id: &str, campaign: &Campaign) -> bool {
        self.random_at(id, campaign).is_some()
            || self
                .node(id)
                .is_some_and(|n| n.kind.is_battle() && !self.is_cleared(id, campaign))
    }

    /// A random group at this place on this map.
    pub fn random_at<'a>(&self, id: &str, campaign: &'a Campaign) -> Option<&'a RandomSkirmish> {
        campaign
            .world
            .as_ref()
            .filter(|w| w.map == self.id)?
            .skirmishes
            .get(id)
    }

    /// Whether an unwon battle stops travel through this place.
    pub fn blocks_path(&self, id: &str, campaign: &Campaign) -> bool {
        !self.is_cleared(id, campaign)
            && self.node(id).is_some_and(|n| {
                n.kind.is_battle()
                    && !matches!(
                        n.kind,
                        NodeKind::FixedSkirmish {
                            guards_path: false,
                            ..
                        }
                    )
            })
    }

    /// The places next to `id` along open roads.
    fn neighbours<'a>(
        &'a self,
        id: &'a str,
        campaign: &'a Campaign,
    ) -> impl Iterator<Item = &'a str> {
        self.open_roads(campaign).filter_map(move |(a, b)| {
            if a == id {
                Some(b.as_str())
            } else if b == id {
                Some(a.as_str())
            } else {
                None
            }
        })
    }

    /// The fewest-roads route from `from` to `to` over open roads, both
    /// ends included; `through` says which places on the way may be passed
    /// through (`to` always may be reached). `None` if there is none.
    fn search(
        &self,
        from: &str,
        to: &str,
        campaign: &Campaign,
        through: impl Fn(&str) -> bool,
    ) -> Option<Vec<String>> {
        let mut came_from: Vec<(String, Option<String>)> = vec![(from.to_owned(), None)];
        let mut queue = VecDeque::from([from.to_owned()]);
        while let Some(here) = queue.pop_front() {
            if here == to {
                let mut route = vec![here.clone()];
                let mut at = here;
                while let Some((_, Some(prev))) = came_from.iter().find(|(n, _)| *n == at) {
                    route.push(prev.clone());
                    at = prev.clone();
                }
                route.reverse();
                return Some(route);
            }
            if here != from && !through(&here) {
                continue;
            }
            for next in self.neighbours(&here, campaign) {
                if !came_from.iter().any(|(n, _)| n == next) {
                    came_from.push((next.to_owned(), Some(here.clone())));
                    queue.push_back(next.to_owned());
                }
            }
        }
        None
    }

    /// The route the army takes from where it is to `to` (see the module
    /// docs): around unwon battles if it can, else to the first unwon
    /// battle on the way. Both ends included.
    pub fn route(&self, from: &str, to: &str, campaign: &Campaign) -> Option<Vec<String>> {
        let passable = |id: &str| !self.blocks_path(id, campaign);
        if let Some(route) = self.search(from, to, campaign, passable) {
            return Some(route);
        }
        let mut route = self.search(from, to, campaign, |_| true)?;
        if let Some(stop) = route
            .iter()
            .skip(1)
            .position(|id| self.blocks_path(id, campaign))
        {
            route.truncate(stop + 2);
        }
        Some(route)
    }
}

impl Campaign {
    /// Puts the army on `map`, at its start, if it isn't on it already
    /// (the first world-map chapter of an act begins).
    pub fn enter_world(&mut self, map: &WorldMap) {
        if self.world.as_ref().is_none_or(|w| w.map != map.id) {
            self.world = Some(WorldState {
                map: map.id.clone(),
                at: map.start.clone(),
                cleared: BTreeSet::new(),
                skirmishes: BTreeMap::new(),
            });
        }
    }

    /// Carries out `command` on `map`, the campaign's act's map.
    pub fn world_command(
        &mut self,
        map: &WorldMap,
        command: &WorldCommand,
    ) -> Result<Vec<WorldEvent>, WorldError> {
        let world = self.world.as_ref().ok_or(WorldError::NotOnMap)?;
        if world.map != map.id {
            return Err(WorldError::NotOnMap);
        }
        let at = world.at.clone();
        match command {
            WorldCommand::TravelTo(to) => {
                if map.node(to).is_none() {
                    return Err(WorldError::NoSuchNode);
                }
                if !map.is_open(to, self) {
                    return Err(WorldError::Locked);
                }
                if *to == at {
                    return Err(WorldError::AlreadyThere);
                }
                let route = map.route(&at, to, self).ok_or(WorldError::NoRoad)?;
                let end = route.last().cloned().unwrap_or(at);
                if let Some(world) = &mut self.world {
                    world.at.clone_from(&end);
                }
                let mut events = vec![WorldEvent::Travelled(route)];
                if map.is_unwon_battle(&end, self) {
                    events.push(WorldEvent::BattleAhead(end));
                }
                Ok(events)
            }
            WorldCommand::Enter => match map.node(&at).map(|n| &n.kind) {
                Some(NodeKind::Town { .. }) => Ok(vec![WorldEvent::Entered(at)]),
                Some(_) if map.is_unwon_battle(&at, self) => Ok(vec![WorldEvent::BattleAhead(at)]),
                _ => Err(WorldError::NothingHere),
            },
        }
    }

    /// Makes `trade` in a town's `shop`: returns the gold paid or got. On
    /// `Err` nothing changes.
    pub fn trade(
        &mut self,
        shop: &Shop,
        items: &ItemTable,
        trade: &TownTrade,
    ) -> Result<Gold, ShopError> {
        if let TownTrade::RepairHeld { unit, slot } = trade {
            let copy = self
                .roster
                .get_mut(*unit)
                .and_then(|u| u.loadout.weapons.get_mut(*slot))
                .and_then(Option::as_mut)
                .ok_or(ShopError::NoItem)?;
            return shop::repair(shop, items, &mut self.gold, copy);
        }
        let mut session = ShopSession::new(shop, items, &mut self.gold, &mut self.stock);
        match trade {
            TownTrade::Buy(item) => session.buy(item),
            TownTrade::Sell(StockItem::Item(item)) => session.sell_item(item),
            TownTrade::Sell(StockItem::Weapon(index)) => session.sell_weapon(*index),
            TownTrade::RepairStock(index) => session.repair_weapon(*index),
            TownTrade::RepairHeld { .. } => Err(ShopError::NoItem),
        }
    }

    /// Place `node`'s battle on `map` is won: it is cleared, and what
    /// waited for it opens. The `chapter` the campaign goes on with is set
    /// first, so its story battle opens too. Winning the map's final story
    /// battle moves the army onto `next`, the next act's map, if there is
    /// one. Returns what happened; nothing if the campaign isn't on `map`.
    pub fn clear_node(
        &mut self,
        map: &WorldMap,
        node: &str,
        chapter: Option<&str>,
        next: Option<&WorldMap>,
    ) -> Vec<WorldEvent> {
        if self.world.as_ref().is_none_or(|w| w.map != map.id) {
            return Vec::new();
        }
        if map.node(node).is_none_or(|n| !n.kind.is_battle()) {
            return Vec::new();
        }
        if let Some(world) = &mut self.world {
            if world.skirmishes.remove(node).is_some() {
                return vec![WorldEvent::Cleared(node.to_owned())];
            }
            if world.cleared.contains(node) {
                return Vec::new();
            }
        }
        let before: BTreeSet<String> = map.open_nodes(self).map(|n| n.id.clone()).collect();
        if let Some(world) = &mut self.world {
            world.cleared.insert(node.to_owned());
        }
        if let Some(chapter) = chapter {
            chapter.clone_into(&mut self.chapter);
        }
        let mut events = vec![WorldEvent::Cleared(node.to_owned())];
        if node == map.final_node
            && let Some(next) = next
        {
            self.world = None;
            self.enter_world(next);
            events.push(WorldEvent::NewAct(next.id.clone()));
            return events;
        }
        events.extend(
            map.open_nodes(self)
                .filter(|n| !before.contains(&n.id))
                .map(|n| WorldEvent::Opened(n.id.clone())),
        );
        if let Some(n) = map.node(node)
            && matches!(n.kind, NodeKind::StoryBattle(_))
            && let Some(level) = n.level
        {
            events.extend(self.spawn_skirmishes(map, level));
        }
        events
    }
}

#[cfg(test)]
mod tests;
