//! The world map screen (ticket 1007, `docs/design/world-structure.md`,
//! `docs/design/look-and-feel.md` *The world map*): the act's places and
//! open roads, the army, a cursor that jumps from place to place, the
//! place under it in the panel, and the map's menu.
//!
//! - **The cursor** is always on a shown place ([`WorldMap::is_shown`]:
//!   open, or one road from an open one, greyed). The cursor's keys jump to
//!   the nearest place that way; Previous and Next unit go through the
//!   places in the map's order; Your army jumps to the army.
//! - **Travel**: Confirm on another place sends the army there along open
//!   roads ([`WorldCommand::TravelTo`]); it walks one place every
//!   [`STEP_S`] (Confirm finishes the walk at once). Arriving at a battle
//!   not yet won starts it (Nick: battles are forced): the screen pops with
//!   a [`WorldBattle`] and the game flow plays it. Arriving at a town opens
//!   it when the player's setting says so ([`ShuyiSettings::towns`]);
//!   Confirm on the army's own town opens it too.
//! - **A town** with one shop opens its shop screen ([`TownScreen`]); one
//!   with several lists them first (*Claude's starting rule*, as Sacred
//!   Stones' towns list their Armory and Vendor).
//! - **Find** lists the army and the kinds of places on the map; the
//!   highlighted kind's nearest place is where the cursor goes, and
//!   Previous and Next unit go through that kind until Find is closed.
//! - **The menu** (Cancel): `Save` (the slot picker, a world-map save),
//!   `Camp` (the Camp screen) and `Options`. The Camp and town screens work
//!   on a copy of the campaign, taken back when they close.
//! - **Music** (`audio.md`, *The world map*): the map's cue plays to its
//!   end, fades out, the map is quiet [`QUIET_S`], then it starts again. A
//!   town's own cue plays while the town is open (its list of shops or its
//!   shop, Nick: "while interacting with a town").
//!
//! **The look is a skin** (ADR-0054): this module decides what the screen
//! shows and does, and says it as a [`WorldMapView`]; [`glyph::paint`]
//! paints it, or [`picture::paint`] when the player's map look is pictures
//! and the map has a picture.
//!
//! [`ShuyiSettings::towns`]: crate::ShuyiSettings::towns

pub mod glyph;
pub mod picture;
pub mod view;

use crate::prelude::*;
use core::any::Any;

use trpg_content::WorldMapDef;
use trpg_core::{Campaign, LEAD_ID, NodeKind, Pos, WorldCommand, WorldEvent, WorldMap};
pub use view::{ListView, PanelView, PlaceMark, PlaceView, WorldMapView};

use super::camp::music::{CampMusic, MusicCue};
use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::screen::{Ctx, FrameInput, Screen, ScreenResult, Shuyi, Transition};
use crate::screens::town::TownScreen;
use crate::screens::{CampScreen, SlotPickerScreen};
use crate::settings::{MapLook, TownsOpen};
use crate::widgets::{Menu, MenuEvent, MenuItem};

/// How long the map is quiet between two rounds of its track, after the
/// fade (`audio.md`: 30 s, Nick "or maybe longer", TUNABLE).
pub const QUIET_S: f64 = 30.0;

/// How long the army takes from one place to the next, in seconds
/// (*Claude's starting value*, TUNABLE).
pub const STEP_S: f64 = 0.2;

/// What the world map pops with: the battle the army arrived at, and the
/// campaign as the map leaves it.
#[derive(Debug, Clone)]
pub struct WorldBattle {
    /// The campaign, the army at the battle's place.
    pub campaign: Campaign,
    /// The battle's place.
    pub node: String,
}

/// An entry of the map's menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEntry {
    /// The slot picker, to save here.
    Save,
    /// The Camp screen.
    Camp,
    /// The Options screen.
    Options,
}

impl MenuEntry {
    /// Every entry, in the menu's order.
    pub const ALL: [MenuEntry; 3] = [MenuEntry::Save, MenuEntry::Camp, MenuEntry::Options];

    /// Its label's text key.
    fn key(self) -> &'static str {
        match self {
            MenuEntry::Save => "world_map.menu.save",
            MenuEntry::Camp => "world_map.menu.camp",
            MenuEntry::Options => "world_map.menu.options",
        }
    }
}

/// A row of the Find list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindKind {
    /// The army.
    Army,
    /// Towns and capitals.
    Towns,
    /// Story battles.
    StoryBattles,
    /// Fixed skirmishes.
    FixedSkirmishes,
    /// Random groups.
    RandomSkirmishes,
    /// Side quests.
    SideQuests,
}

impl FindKind {
    /// Every row, in the list's order.
    pub const ALL: [FindKind; 6] = [
        FindKind::Army,
        FindKind::Towns,
        FindKind::StoryBattles,
        FindKind::SideQuests,
        FindKind::FixedSkirmishes,
        FindKind::RandomSkirmishes,
    ];

    /// Its label's text key.
    fn key(self) -> &'static str {
        match self {
            FindKind::Army => "world_map.find.army",
            FindKind::Towns => "world_map.find.towns",
            FindKind::StoryBattles => "world_map.find.story_battles",
            FindKind::SideQuests => "world_map.find.side_quests",
            FindKind::FixedSkirmishes => "world_map.find.fixed_skirmishes",
            FindKind::RandomSkirmishes => "world_map.find.random_skirmishes",
        }
    }

    /// Whether a place of `kind` is of this row (never for the army's).
    fn holds(self, kind: &NodeKind) -> bool {
        matches!(
            (self, kind),
            (FindKind::Towns, NodeKind::Town { .. })
                | (FindKind::StoryBattles, NodeKind::StoryBattle(_))
                | (FindKind::SideQuests, NodeKind::SideQuest(_))
                | (FindKind::FixedSkirmishes, NodeKind::FixedSkirmish { .. })
        )
    }
}

/// What is open over the map.
#[derive(Debug, Clone)]
enum Open {
    /// Nothing: the cursor moves.
    Nothing,
    /// The menu.
    Menu(Menu<MenuEntry>),
    /// The Find list.
    Find(Menu<FindKind>),
    /// A town's shops, by index, and the town.
    Town(String, Menu<usize>),
}

/// The army on its way.
#[derive(Debug, Clone)]
struct Walk {
    /// The places it goes through, the first where it started.
    route: Vec<String>,
    /// The place it is at now.
    step: usize,
    /// When it moves on.
    next_at: f64,
    /// The battle at the end of the way, if any.
    battle: Option<String>,
}

/// The world map screen. See the module docs.
#[derive(Debug, Clone)]
pub struct WorldMapScreen {
    campaign: Campaign,
    /// The place under the cursor.
    cursor: String,
    /// When the cursor last moved (its pulse starts bright).
    moved_at: f64,
    walk: Option<Walk>,
    open: Open,
    /// The map's track, as it rotates; `None` until it starts (or while a
    /// place's own cue plays).
    music: Option<CampMusic>,
    /// The place's own cue playing, if any.
    place_cue: Option<String>,
    /// [`Ctx::clock_s`] when the campaign began, for a save's playtime.
    started_at: f64,
}

impl WorldMapScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "world_map";

    /// The world map of `campaign` (on it, [`Campaign::world`]), the cursor
    /// on the army; the campaign began at clock time `started_at`.
    pub fn new(campaign: Campaign, started_at: f64) -> Self {
        let cursor = campaign
            .world
            .as_ref()
            .map(|w| w.at.clone())
            .unwrap_or_default();
        Self {
            campaign,
            cursor,
            moved_at: 0.0,
            walk: None,
            open: Open::Nothing,
            music: None,
            place_cue: None,
            started_at,
        }
    }

    /// The campaign as the map has it.
    pub fn campaign(&self) -> &Campaign {
        &self.campaign
    }

    /// The place under the cursor.
    pub fn cursor(&self) -> &str {
        &self.cursor
    }

    /// Whether the army is on its way.
    pub fn walking(&self) -> bool {
        self.walk.is_some()
    }

    /// The map's file, if the campaign is on one the content has.
    fn def<'a>(&self, ctx: &'a Ctx) -> Option<&'a WorldMapDef> {
        let id = &self.campaign.world.as_ref()?.map;
        ctx.game.content.worldmaps.get(id)
    }

    /// The place the army is at (or passing, on its way).
    fn army_at(&self) -> &str {
        if let Some(walk) = &self.walk {
            return &walk.route[walk.step];
        }
        self.campaign.world.as_ref().map_or("", |w| w.at.as_str())
    }

    /// The shown places of `map`, in its order.
    fn open_places<'a>(&'a self, map: &'a WorldMap) -> Vec<&'a str> {
        map.shown_nodes(&self.campaign)
            .map(|n| n.id.as_str())
            .collect()
    }

    /// Puts the cursor on place `id`.
    fn point(&mut self, ctx: &mut Ctx, id: &str) {
        if self.cursor != id {
            id.clone_into(&mut self.cursor);
            self.moved_at = ctx.clock_s;
            ctx.audio.menu(MenuSound::Move);
        }
    }

    /// Moves the cursor to the nearest open place in direction `(dx, dy)`:
    /// one ahead of it within a 60° cone or so, nearest first, in pixels
    /// (a cell is twice as tall as it is wide).
    fn step_cursor(&mut self, ctx: &mut Ctx, (dx, dy): (i32, i32)) {
        let Some(def) = self.def(ctx) else {
            return;
        };
        let Some(&from) = def.places.get(&self.cursor) else {
            return;
        };
        let score = |to: Pos| {
            let (x, y) = ((to.x - from.x) * 8, (to.y - from.y) * 16);
            let along = x * dx + y * dy;
            let across = (x * dy).abs() + (y * dx).abs();
            (along > 0 && across <= along * 2).then_some(along + across * 2)
        };
        let best = self
            .open_places(&def.map)
            .into_iter()
            .filter_map(|id| Some((score(*def.places.get(id)?)?, id)))
            .min_by_key(|(s, _)| *s)
            .map(|(_, id)| id.to_owned());
        match best {
            Some(id) => self.point(ctx, &id),
            None => ctx.audio.menu(MenuSound::Denied),
        }
    }

    /// The places Previous and Next unit go through: those of `kind`, or
    /// every open place.
    fn cycle(&self, ctx: &Ctx, kind: Option<FindKind>) -> Vec<String> {
        let Some(def) = self.def(ctx) else {
            return Vec::new();
        };
        def.map
            .shown_nodes(&self.campaign)
            .filter(|n| {
                kind.is_none_or(|k| {
                    let random = def.map.random_at(&n.id, &self.campaign).is_some();
                    if random {
                        k == FindKind::RandomSkirmishes
                    } else {
                        k.holds(&n.kind)
                    }
                })
            })
            .map(|n| n.id.clone())
            .collect()
    }

    /// Previous (`forward` false) or Next unit: the next place of `kind`.
    fn next_place(&mut self, ctx: &mut Ctx, kind: Option<FindKind>, forward: bool) {
        let places = self.cycle(ctx, kind);
        if places.is_empty() {
            ctx.audio.menu(MenuSound::Denied);
            return;
        }
        let at = places.iter().position(|p| *p == self.cursor);
        let next = match (at, forward) {
            (Some(i), true) => (i + 1) % places.len(),
            (Some(i), false) => (i + places.len() - 1) % places.len(),
            (None, _) => 0,
        };
        let id = places[next].clone();
        self.point(ctx, &id);
    }

    /// The cursor to the place of `kind` nearest the army (the army's
    /// place for [`FindKind::Army`]).
    fn find(&mut self, ctx: &mut Ctx, kind: FindKind) {
        let army = self.army_at().to_owned();
        if kind == FindKind::Army {
            self.point(ctx, &army);
            return;
        }
        let Some(def) = self.def(ctx) else {
            return;
        };
        let from = def.places.get(&army).copied().unwrap_or_default();
        let distance =
            |id: &String| distance(from, def.places.get(id).copied().unwrap_or_default());
        if let Some(id) = self.cycle(ctx, Some(kind)).into_iter().min_by_key(distance) {
            self.point(ctx, &id);
        }
    }

    /// Confirm on the map: travel to the place under the cursor, or enter
    /// the army's own.
    fn confirm(&mut self, ctx: &mut Ctx) -> Transition {
        let Some(def) = self.def(ctx) else {
            return Transition::None;
        };
        let map = def.map.clone();
        let here = self.army_at().to_owned();
        if self.cursor == here {
            return self.enter(ctx, &map);
        }
        let travel = WorldCommand::TravelTo(self.cursor.clone());
        match self.campaign.world_command(&map, &travel) {
            Ok(events) => {
                let mut walk = Walk {
                    route: Vec::new(),
                    step: 0,
                    next_at: ctx.clock_s + STEP_S,
                    battle: None,
                };
                for event in events {
                    match event {
                        WorldEvent::Travelled(route) => walk.route = route,
                        WorldEvent::BattleAhead(node) => walk.battle = Some(node),
                        _ => {}
                    }
                }
                ctx.audio.menu(MenuSound::Select);
                self.walk = Some(walk);
            }
            Err(_) => ctx.audio.menu(MenuSound::Denied),
        }
        Transition::None
    }

    /// Enters the army's place: a town opens; a battle not yet won starts.
    fn enter(&mut self, ctx: &mut Ctx, map: &WorldMap) -> Transition {
        let Ok(events) = self.campaign.world_command(map, &WorldCommand::Enter) else {
            ctx.audio.menu(MenuSound::Denied);
            return Transition::None;
        };
        match events.into_iter().next() {
            Some(WorldEvent::Entered(town)) => self.open_town(ctx, &town),
            Some(WorldEvent::BattleAhead(node)) => self.fight(node),
            _ => Transition::None,
        }
    }

    /// Pops with the battle at `node`.
    fn fight(&self, node: String) -> Transition {
        Transition::pop_with(WorldBattle {
            campaign: self.campaign.clone(),
            node,
        })
    }

    /// Opens town `town`: its shop, or the list of its shops.
    fn open_town(&mut self, ctx: &mut Ctx, town: &str) -> Transition {
        let Some(def) = self.def(ctx) else {
            return Transition::None;
        };
        let shops = def.map.town_shops(town, &self.campaign);
        if shops.is_empty() {
            return Transition::None;
        }
        let name = ctx.words().place_name(def, town).to_owned();
        ctx.audio.menu(MenuSound::Select);
        if let [shop] = shops.as_slice() {
            self.town_music(ctx, town);
            let screen = TownScreen::new(self.campaign.clone(), shop.clone(), name);
            return Transition::Push(Box::new(screen));
        }
        let items = shops
            .iter()
            .enumerate()
            .map(|(i, s)| {
                MenuItem::new(ctx.text(super::battle::shop::kind_key(s.kind))).with_value(i)
            })
            .collect();
        self.open = Open::Town(town.to_owned(), Menu::new(items));
        Transition::None
    }

    /// The campaign as a save made now holds it.
    fn campaign_now(&self, ctx: &Ctx) -> Campaign {
        let mut campaign = self.campaign.clone();
        crate::flow::FlowScreen::take_in(&mut campaign, ctx, self.started_at);
        campaign
    }

    /// One action on the map.
    fn on_map(&mut self, ctx: &mut Ctx, action: Action) -> Transition {
        match action {
            Action::CursorUp => self.step_cursor(ctx, (0, -1)),
            Action::CursorDown => self.step_cursor(ctx, (0, 1)),
            Action::CursorLeft => self.step_cursor(ctx, (-1, 0)),
            Action::CursorRight => self.step_cursor(ctx, (1, 0)),
            Action::PrevUnit => self.next_place(ctx, None, false),
            Action::NextUnit => self.next_place(ctx, None, true),
            Action::WorldArmy => self.find(ctx, FindKind::Army),
            Action::WorldFind => {
                let items = FindKind::ALL
                    .map(|k| MenuItem::new(ctx.text(k.key())).with_value(k))
                    .into();
                self.open = Open::Find(Menu::new(items));
                self.find(ctx, FindKind::Army);
                ctx.audio.menu(MenuSound::Select);
            }
            Action::Confirm => return self.confirm(ctx),
            Action::Cancel => {
                let items = MenuEntry::ALL
                    .map(|e| MenuItem::new(ctx.text(e.key())).with_value(e))
                    .into();
                self.open = Open::Menu(Menu::new(items));
                ctx.audio.menu(MenuSound::Select);
            }
            _ => {}
        }
        Transition::None
    }

    /// One action in the map's menu.
    fn on_menu(&mut self, ctx: &mut Ctx, action: Action) -> Transition {
        let Open::Menu(menu) = &mut self.open else {
            return Transition::None;
        };
        match menu.handle_with_sound(action, &mut ctx.audio) {
            Some(MenuEvent::Chosen(entry)) => {
                self.open = Open::Nothing;
                let screen: Box<dyn Screen<Shuyi>> = match entry {
                    MenuEntry::Save => {
                        Box::new(SlotPickerScreen::save_world(ctx, self.campaign_now(ctx)))
                    }
                    MenuEntry::Camp => Box::new(CampScreen::new(ctx, self.campaign.clone())),
                    MenuEntry::Options => Box::new(super::options::screen()),
                };
                Transition::Push(screen)
            }
            Some(MenuEvent::Cancelled) => {
                self.open = Open::Nothing;
                Transition::None
            }
            None => Transition::None,
        }
    }

    /// One action in the Find list: the highlighted kind's nearest place
    /// gets the cursor; Previous and Next unit go through that kind.
    fn on_find(&mut self, ctx: &mut Ctx, action: Action) -> Transition {
        let Open::Find(menu) = &mut self.open else {
            return Transition::None;
        };
        let kind = menu.value(menu.focus()).copied();
        match action {
            Action::PrevUnit | Action::NextUnit => {
                let kind = kind.filter(|k| *k != FindKind::Army);
                if kind.is_some() {
                    self.next_place(ctx, kind, action == Action::NextUnit);
                } else {
                    ctx.audio.menu(MenuSound::Denied);
                }
                return Transition::None;
            }
            Action::WorldFind => {
                self.open = Open::Nothing;
                ctx.audio.menu(MenuSound::Cancel);
                return Transition::None;
            }
            _ => {}
        }
        match menu.handle_with_sound(action, &mut ctx.audio) {
            Some(MenuEvent::Chosen(_) | MenuEvent::Cancelled) => self.open = Open::Nothing,
            None => {
                let now = menu.value(menu.focus()).copied();
                if let Some(k) = now.filter(|k| Some(*k) != kind) {
                    self.find(ctx, k);
                }
            }
        }
        Transition::None
    }

    /// One action in a town's list of shops.
    fn on_town(&mut self, ctx: &mut Ctx, action: Action) -> Transition {
        let Open::Town(town, menu) = &mut self.open else {
            return Transition::None;
        };
        let town = town.clone();
        match menu.handle_with_sound(action, &mut ctx.audio) {
            Some(MenuEvent::Chosen(i)) => {
                self.town_music(ctx, &town);
                let Some(def) = self.def(ctx) else {
                    return Transition::None;
                };
                let shop = def.map.town_shops(&town, &self.campaign).get(i).cloned();
                let name = ctx.words().place_name(def, &town).to_owned();
                match shop {
                    Some(shop) => Transition::Push(Box::new(TownScreen::new(
                        self.campaign.clone(),
                        shop,
                        name,
                    ))),
                    None => Transition::None,
                }
            }
            Some(MenuEvent::Cancelled) => {
                self.open = Open::Nothing;
                Transition::None
            }
            None => Transition::None,
        }
    }

    /// The army on its way: one place every [`STEP_S`], or to the end at
    /// once on Confirm. At the end, a battle starts, or a town opens if
    /// the player's setting says so.
    fn walk_on(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Option<Transition> {
        let walk = self.walk.as_mut()?;
        let last = walk.route.len().saturating_sub(1);
        if input.actions.contains(&Action::Confirm) {
            walk.step = last;
        }
        for _ in walk.step..last {
            if ctx.clock_s >= walk.next_at {
                walk.step += 1;
                walk.next_at += STEP_S;
            }
        }
        if walk.step < last {
            return Some(Transition::None);
        }
        let walk = self.walk.take()?;
        let end = walk.route.last().cloned().unwrap_or_default();
        self.cursor.clone_from(&end);
        self.moved_at = ctx.clock_s;
        if let Some(node) = walk.battle {
            return Some(self.fight(node));
        }
        let town = self
            .def(ctx)
            .and_then(|d| d.map.node(&end))
            .is_some_and(|n| matches!(n.kind, NodeKind::Town { .. }));
        if town && ctx.settings().game.towns == TownsOpen::OnArrival {
            return Some(self.open_town(ctx, &end));
        }
        Some(Transition::None)
    }

    /// Town `town`'s own music, if it has one, while it is open; the map's
    /// track starts again when it closes.
    fn town_music(&mut self, ctx: &mut Ctx, town: &str) {
        let cue = self.def(ctx).and_then(|d| d.place_music.get(town)).cloned();
        if let Some(cue) = cue
            && self.place_cue.as_ref() != Some(&cue)
        {
            ctx.audio.play_music(&cue);
            self.place_cue = Some(cue);
            self.music = None;
        }
    }

    /// The music: an open town's own cue, else the map's track, rotating
    /// with its quiet.
    fn tick_music(&mut self, ctx: &mut Ctx) {
        if let Open::Town(town, _) = &self.open {
            let town = town.clone();
            self.town_music(ctx, &town);
            if self.place_cue.is_some() {
                return;
            }
        }
        let Some(def) = self.def(ctx) else {
            return;
        };
        let map_cue = def.music.clone();
        self.place_cue = None;
        if let Some(cue) = self.music.as_mut().and_then(|m| m.tick(ctx.clock_s)) {
            ask(ctx, &cue);
            return;
        }
        if self.music.is_none() {
            let manifest = &ctx.game.content.audio;
            let length = manifest
                .music
                .get(&map_cue)
                .map(|t| f64::from(t.length_ms) / 1000.0);
            let fade_s = f64::from(manifest.music_fade_ms) / 1000.0;
            let tracks = length.map(|l| vec![(map_cue, l)]).unwrap_or_default();
            if let Some((music, cue)) = CampMusic::start(tracks, 0, fade_s, ctx.clock_s) {
                self.music = Some(music.with_quiet(QUIET_S));
                ask(ctx, &cue);
            }
        }
    }

    /// What the panel says about place `id`.
    fn panel(&self, ctx: &Ctx, def: &WorldMapDef, id: &str) -> PanelView {
        let map = &def.map;
        let words = ctx.words();
        let chapter = ctx.game.content.chapters.get(&self.campaign.chapter);
        let chapter = chapter.map_or("", |c| words.chapter_title(c));
        let act = ctx.text_with(
            "world_map.act",
            &[("act", &map.act.to_string()), ("chapter", &chapter)],
        );
        let node = map.node(id);
        let cleared = map.is_cleared(id, &self.campaign);
        let random = map.random_at(id, &self.campaign);
        let kind_key = match node.map(|n| &n.kind) {
            _ if random.is_some() => "world_map.kind.random_skirmish",
            Some(NodeKind::FixedSkirmish { .. }) => "world_map.kind.fixed_skirmish",
            Some(NodeKind::StoryBattle(_)) => "world_map.kind.story_battle",
            Some(NodeKind::SideQuest(_)) => "world_map.kind.side_quest",
            Some(NodeKind::Town { capital: true, .. }) => "world_map.kind.capital",
            Some(NodeKind::Town { .. }) => "world_map.kind.town",
            _ => "world_map.kind.empty",
        };
        let mut lines = Vec::new();
        if !map.is_open(id, &self.campaign) {
            lines.push(ctx.text("world_map.not_open").to_owned());
        }
        if let Some(n) = node {
            match &n.kind {
                k if k.is_battle() && cleared && random.is_none() => {
                    lines.push(ctx.text("world_map.won").to_owned());
                }
                k if k.is_battle() => {
                    if let Some(level) = random.map(|g| g.level).or(n.level) {
                        let level = level.to_string();
                        lines.push(ctx.text_with("world_map.enemy_level", &[("level", &level)]));
                    }
                }
                NodeKind::Town { shops, .. } => {
                    let key = super::battle::shop::kind_key;
                    lines.extend(shops.iter().map(|s| ctx.text(key(s.kind)).to_owned()));
                }
                _ => {}
            }
        }
        if map.blocks_path(id, &self.campaign)
            && matches!(node.map(|n| &n.kind), Some(NodeKind::FixedSkirmish { .. }))
        {
            lines.push(ctx.text("world_map.guards_path").to_owned());
        }
        if id == self.army_at() {
            lines.push(ctx.text("world_map.army_here").to_owned());
        }
        PanelView {
            title: words.place_name(def, id).to_owned(),
            act,
            kind: ctx.text(kind_key).to_owned(),
            lines,
        }
    }

    /// The help line for what is open.
    fn help(&self, ctx: &Ctx) -> String {
        let key = match &self.open {
            Open::Nothing if self.cursor == self.army_at() => "world_map.help.here",
            Open::Nothing => "world_map.help.map",
            Open::Menu(_) | Open::Town(..) => "world_map.help.menu",
            Open::Find(_) => "world_map.help.find",
        };
        ctx.text_with(key, &[])
    }

    /// The shown places but the army's, as the view has them.
    fn places(&self, ctx: &Ctx, def: &WorldMapDef) -> Vec<PlaceView> {
        let map = &def.map;
        let at = |id: &str| def.places.get(id).copied().unwrap_or_default();
        map.shown_nodes(&self.campaign)
            .filter(|n| n.id != self.army_at())
            .map(|n| {
                let cleared = map.is_cleared(&n.id, &self.campaign);
                let open = map.is_open(&n.id, &self.campaign);
                let random = map.random_at(&n.id, &self.campaign);
                let mark = match &n.kind {
                    _ if random.is_some() => PlaceMark::RandomSkirmish,
                    NodeKind::FixedSkirmish { .. } if !cleared => PlaceMark::FixedSkirmish,
                    _ if n.kind.is_battle() && cleared => PlaceMark::Cleared,
                    NodeKind::StoryBattle(_) => PlaceMark::StoryBattle,
                    NodeKind::SideQuest(_) => PlaceMark::SideQuest,
                    NodeKind::Town { capital: true, .. } => PlaceMark::Capital,
                    NodeKind::Town { .. } => PlaceMark::Town,
                    NodeKind::Empty | NodeKind::FixedSkirmish { .. } => PlaceMark::Empty,
                };
                let level = random
                    .map(|g| g.level)
                    .or(n.level)
                    .filter(|_| open && (random.is_some() || (n.kind.is_battle() && !cleared)));
                let level = level.map(|l| {
                    let l = l.to_string();
                    ctx.text_with("world_map.level", &[("level", &l)])
                });
                PlaceView {
                    pos: at(&n.id),
                    mark,
                    level,
                    locked: !open,
                }
            })
            .collect()
    }

    /// The open roads and the roads shown but not open, as the view has
    /// them.
    fn roads(&self, def: &WorldMapDef) -> (Vec<Vec<Pos>>, Vec<Vec<Pos>>) {
        let map = &def.map;
        let roads = def
            .roads
            .iter()
            .zip(&map.roads)
            .filter(|(_, (a, b))| map.is_open(a, &self.campaign) && map.is_open(b, &self.campaign))
            .map(|(cells, _)| cells.clone())
            .collect();
        let locked: Vec<&(String, String)> = map.locked_roads(&self.campaign).collect();
        let locked_roads = def
            .roads
            .iter()
            .zip(&map.roads)
            .filter(|(_, road)| locked.contains(road))
            .map(|(cells, _)| cells.clone())
            .collect();
        (roads, locked_roads)
    }

    /// The screen as it is now, as plain data for a skin to paint.
    pub fn view(&self, ctx: &Ctx) -> WorldMapView {
        let Some(def) = self.def(ctx) else {
            return WorldMapView {
                map: String::new(),
                places: Vec::new(),
                roads: Vec::new(),
                locked_roads: Vec::new(),
                army: Pos::default(),
                army_label: String::new(),
                cursor: Pos::default(),
                cursor_on_army: false,
                cursor_brightness: 1.0,
                panel: PanelView {
                    title: String::new(),
                    act: String::new(),
                    kind: String::new(),
                    lines: Vec::new(),
                },
                list: None,
                help: String::new(),
            };
        };
        let map = &def.map;
        let at = |id: &str| def.places.get(id).copied().unwrap_or_default();
        let places = self.places(ctx, def);
        let (roads, locked_roads) = self.roads(def);
        let lead = self
            .campaign
            .roster
            .iter()
            .find(|u| u.character.as_ref().is_some_and(|c| c.0 == LEAD_ID));
        let list = match &self.open {
            Open::Nothing => None,
            Open::Menu(menu) => Some(ListView {
                title: ctx.text("world_map.menu.title").to_owned(),
                menu: menu.view(),
            }),
            Open::Find(menu) => Some(ListView {
                title: ctx.text("world_map.find.title").to_owned(),
                menu: menu.view(),
            }),
            Open::Town(town, menu) => Some(ListView {
                title: ctx.words().place_name(def, town).to_owned(),
                menu: menu.view(),
            }),
        };
        #[expect(clippy::cast_possible_truncation, reason = "a fraction of a second")]
        let since = (ctx.clock_s - self.moved_at).max(0.0) as f32;
        WorldMapView {
            map: map.id.clone(),
            places,
            roads,
            locked_roads,
            army: at(self.army_at()),
            army_label: lead.map_or_else(String::new, |u| u.map_label.clone()),
            cursor: at(&self.cursor),
            cursor_on_army: self.cursor == self.army_at(),
            cursor_brightness: pulse(since),
            panel: self.panel(ctx, def, &self.cursor),
            list,
            help: self.help(ctx),
        }
    }
}

/// How far apart cells `a` and `b` are on screen, across plus down, in
/// pixels (a cell is 8 wide and 16 tall).
fn distance(a: Pos, b: Pos) -> i32 {
    ((b.x - a.x) * 8).abs() + ((b.y - a.y) * 16).abs()
}

/// The cursor's brightness `t` seconds into its pulse: full, down to half
/// halfway through a second, back up (as the battle map's cursor).
fn pulse(t: f32) -> f32 {
    let mut cursor = shuyi_ui::grid::cursor::Cursor::new(Pos::default());
    cursor.tick(t);
    cursor.brightness()
}

/// Asks for `cue` on `ctx`'s audio.
fn ask(ctx: &mut Ctx, cue: &MusicCue) {
    match cue {
        MusicCue::Play(cue) => ctx.audio.play_music(cue),
        MusicCue::Stop => ctx.audio.stop_music(),
    }
}

/// Paints `view`: with pictures when the player's map look is pictures and
/// the map has a picture, else with glyphs.
fn paint(ctx: &Ctx, view: &WorldMapView, buf: &mut GlyphBuffer) {
    let pictures = ctx.settings().game.map_look == MapLook::Pictures;
    let def = ctx.game.content.worldmaps.get(&view.map);
    match def.and_then(|d| d.picture.as_deref()) {
        Some(picture) if pictures && ctx.game.content.images.id(picture).is_some() => {
            picture::paint(ctx, view, picture, buf);
        }
        _ => glyph::paint(ctx, view, buf),
    }
}

impl Screen<Shuyi> for WorldMapScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn actions(&self) -> Option<Vec<crate::input::Action>> {
        Some(crate::screens::actions::WORLD_MAP.to_vec())
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        self.tick_music(ctx);
        if let Some(t) = self.walk_on(ctx, input) {
            return t;
        }
        for &action in &input.actions {
            let t = match &self.open {
                Open::Nothing => self.on_map(ctx, action),
                Open::Menu(_) => self.on_menu(ctx, action),
                Open::Find(_) => self.on_find(ctx, action),
                Open::Town(..) => self.on_town(ctx, action),
            };
            if !matches!(t, Transition::None) {
                return t;
            }
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    /// Back from camp or a town: their copy of the campaign is the map's
    /// now. Back from anything: the map's music again.
    fn uncovered(&mut self, _ctx: &mut Ctx, result: Option<ScreenResult>) -> Transition {
        if let Some(campaign) = result.and_then(ScreenResult::take::<Campaign>) {
            self.campaign = campaign;
        }
        self.open = Open::Nothing;
        self.music = None;
        self.place_cue = None;
        Transition::None
    }
}

#[cfg(test)]
mod tests;
