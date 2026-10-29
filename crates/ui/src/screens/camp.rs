//! The Camp screen (ticket 1003, `docs/design/supports.md`): between
//! battles, the army's support conversations and the camp events, each
//! played with the dialogue player.
//!
//! - **Supports**: the units with a conversation unlocked or seen, in
//!   roster order; Confirm on one lists its partners' conversations, one
//!   row each with its rank (C, B, A). An unlocked one is `NEW`: watching
//!   it views it ([`Campaign::view_support`]), which is what raises the
//!   pair's rank. A seen one can be watched again. A partner who has left
//!   the army keeps the conversations already seen.
//! - **Camp events**: the events available now ([`CampEvent::available`]),
//!   `NEW` until watched once ([`Campaign::see_camp_event`]).
//! - **Break camp**: Confirm leaves. Cancel in the sections moves the
//!   focus there; it never leaves by itself.
//!
//! A section with something new says `NEW` beside it too. The screen
//! keeps its own copy of the campaign and pops with it (a [`Campaign`])
//! when camp breaks, for the game flow to take back.
//!
//! The `camp` pool's tracks rotate while it is open ([`music`]).
//!
//! **The look is a skin** (ADR-0054): this module decides what the screen
//! shows and does, and says it as a [`CampView`]; [`glyph::paint`] paints
//! it. The Playdate's one-bit skin is ticket 1067.

pub mod glyph;
pub mod music;
pub mod view;

use crate::prelude::*;
pub use view::CampView;

use trpg_content::Present;
use trpg_core::{CampEvent, Campaign, CharacterId, SupportPair, SupportRank, SupportTable};

use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::screen::{Ctx, FrameInput, Screen, ScreenResult, Shuyi, Transition};
use crate::screens::DialogueScreen;
use crate::widgets::menu::{Menu, MenuEvent, MenuItem, SuffixKind};
use music::{CampMusic, MusicCue};

/// The music pool camp plays (`assets/audio/audio.ron`).
pub const POOL: &str = "camp";

/// Text key of the `NEW` mark. Like all the screen's text it is in
/// `assets/lang/en/ui.ron` under `camp.` (ADR-0045).
pub const NEW: &str = "camp.new";

/// One of the screen's sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// The support conversations.
    Supports,
    /// The camp events.
    Events,
    /// Leaves camp.
    Leave,
}

impl Section {
    /// Every section, in the screen's order.
    pub const ALL: [Section; 3] = [Section::Supports, Section::Events, Section::Leave];

    /// Its name's text key.
    fn key(self) -> &'static str {
        match self {
            Section::Supports => "camp.section.supports",
            Section::Events => "camp.section.events",
            Section::Leave => "camp.section.leave",
        }
    }
}

/// One support conversation of a unit's list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Talk {
    /// The unit whose list it is in.
    pub unit: CharacterId,
    /// Who the unit talks with.
    pub partner: CharacterId,
    /// The conversation's rank.
    pub rank: SupportRank,
    /// Whether it is unlocked and not yet viewed.
    pub new: bool,
}

/// What a row of the list stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Row {
    /// A unit with supports: Confirm lists its conversations.
    Unit(CharacterId),
    /// A support conversation.
    Talk(Talk),
    /// A camp event, by index into the content's events.
    Event(usize),
}

/// Where the focus is.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Focus {
    /// In the sections.
    Sections,
    /// In the units with supports.
    Units,
    /// In this unit's conversations.
    Talks(CharacterId),
    /// In the camp events.
    Events,
}

/// The Camp screen (see the module docs).
#[derive(Debug, Clone)]
pub struct CampScreen {
    campaign: Campaign,
    sections: Menu<Section>,
    focus: Focus,
    /// The list shown (of the focused section, or the one focus is in).
    list: Menu<Row>,
    /// The music's rotation, once camp has started it.
    music: Option<CampMusic>,
}

impl CampScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "camp";

    /// Camp for `campaign`, the Supports section focused.
    pub fn new(ctx: &Ctx, campaign: Campaign) -> Self {
        let mut screen = Self {
            campaign,
            sections: Menu::new(vec![]),
            focus: Focus::Sections,
            list: Menu::new(vec![]),
            music: None,
        };
        screen.refresh(ctx, 0);
        screen
    }

    /// The campaign as camp has left it so far.
    pub fn campaign(&self) -> &Campaign {
        &self.campaign
    }

    /// Whether camp has anything to list: a support conversation unlocked
    /// or seen, or a camp event available. The game flow skips camp
    /// without one.
    pub fn has_anything(ctx: &Ctx, campaign: &Campaign) -> bool {
        let content = &ctx.game.content;
        let units = campaign.roster.iter().filter_map(|u| u.character.as_ref());
        let mut talks = units.flat_map(|c| talks(campaign, &content.supports, c));
        talks.next().is_some() || !events(campaign, &content.camp).is_empty()
    }

    /// The rows of the list for the section in focus, or the one the
    /// focus is in.
    pub fn rows(&self) -> Vec<Row> {
        self.list.items().iter().map(|i| i.value.clone()).collect()
    }

    /// The section focused.
    pub fn section(&self) -> Section {
        let focus = self.sections.focus();
        self.sections.value(focus).copied().unwrap_or(Section::Leave)
    }

    /// Rebuilds the sections (their `NEW` marks) and the list from the
    /// campaign, the list focused on row `row` (or as near as it has).
    fn refresh(&mut self, ctx: &Ctx, row: usize) {
        let new = ctx.text(NEW).to_owned();
        let section_focus = self.sections.focus();
        let items = Section::ALL.map(|s| {
            let item = MenuItem::new(ctx.text(s.key())).with_value(s);
            if self.news(ctx, s) {
                item.with_suffix(new.clone(), SuffixKind::Warning)
            } else {
                item
            }
        });
        self.sections = Menu::new(items.into()).focused(section_focus);
        let rows = self.list_rows(ctx);
        let last = rows.len().saturating_sub(1);
        let items = rows.into_iter().map(|r| self.item(ctx, r, &new)).collect();
        self.list = Menu::new(items).focused(row.min(last));
    }

    /// Whether section `section` has anything new.
    fn news(&self, ctx: &Ctx, section: Section) -> bool {
        let content = &ctx.game.content;
        match section {
            Section::Supports => {
                let units = self.campaign.roster.iter().filter_map(|u| u.character.as_ref());
                let mut talks = units.flat_map(|c| talks(&self.campaign, &content.supports, c));
                talks.any(|t| t.new)
            }
            Section::Events => {
                let mut available = events(&self.campaign, &content.camp).into_iter();
                available.any(|i| !self.campaign.camp_seen.contains(&content.camp[i].id))
            }
            Section::Leave => false,
        }
    }

    /// The rows of the list shown now.
    fn list_rows(&self, ctx: &Ctx) -> Vec<Row> {
        let shown = match &self.focus {
            Focus::Sections => match self.section() {
                Section::Supports => Focus::Units,
                Section::Events => Focus::Events,
                Section::Leave => return vec![],
            },
            focus => focus.clone(),
        };
        self.list_rows_of(ctx, shown)
    }

    /// The rows of the list `shown` (the units' for the sections).
    fn list_rows_of(&self, ctx: &Ctx, shown: Focus) -> Vec<Row> {
        let content = &ctx.game.content;
        let table = &content.supports;
        match shown {
            Focus::Units | Focus::Sections => {
                let units = self.campaign.roster.iter().filter_map(|u| u.character.clone());
                let with = units.filter(|c| talks(&self.campaign, table, c).next().is_some());
                with.map(Row::Unit).collect()
            }
            Focus::Talks(unit) => talks(&self.campaign, table, &unit).map(Row::Talk).collect(),
            Focus::Events => events(&self.campaign, &content.camp)
                .into_iter()
                .map(Row::Event)
                .collect(),
        }
    }

    /// The menu item of `row`, with `new` beside a new one.
    fn item(&self, ctx: &Ctx, row: Row, new: &str) -> MenuItem<Row> {
        let content = &ctx.game.content;
        let (label, is_new) = match &row {
            Row::Unit(c) => {
                let mut list = talks(&self.campaign, &content.supports, c);
                (self.name(ctx, c), list.any(|t| t.new))
            }
            Row::Talk(t) => (format!("{:?}  {}", t.rank, self.name(ctx, &t.partner)), t.new),
            Row::Event(i) => {
                let event = &content.camp[*i];
                let title = ctx.words().camp_title(event).to_owned();
                (title, !self.campaign.camp_seen.contains(&event.id))
            }
        };
        let item = MenuItem::new(label).with_value(row);
        if is_new {
            item.with_suffix(new.to_owned(), SuffixKind::Warning)
        } else {
            item
        }
    }

    /// The name of `character`: its unit's, if it is in the army, else the
    /// names table's.
    fn name(&self, ctx: &Ctx, character: &CharacterId) -> String {
        let words = ctx.words();
        match self.campaign.member(character) {
            Some(unit) => words.unit(unit).to_owned(),
            None => words.name(&ctx.game.content.names, &character.0).to_owned(),
        }
    }

    /// The bottom help line.
    fn help(&self, ctx: &Ctx) -> String {
        let key = match (&self.focus, self.section()) {
            (Focus::Sections, Section::Leave) => "camp.help.leave",
            (Focus::Sections, _) => "camp.help.sections",
            (Focus::Units, _) => "camp.help.units",
            (Focus::Talks(_) | Focus::Events, _) => "camp.help.watch",
        };
        ctx.text_with(key, &[])
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint`]).
    pub fn view(&self, ctx: &Ctx) -> CampView {
        let (title, empty) = match (&self.focus, self.section()) {
            (Focus::Talks(unit), _) => (
                ctx.text_with("camp.list.talks", &[("name", &self.name(ctx, unit))]),
                "camp.empty.supports",
            ),
            (_, Section::Supports) => (ctx.text("camp.list.supports").to_owned(), "camp.empty.supports"),
            (_, Section::Events) => (ctx.text("camp.list.events").to_owned(), "camp.empty.events"),
            (_, Section::Leave) => (String::new(), "camp.leave_line"),
        };
        // The list's focus shows only while the focus is in it.
        let in_list = self.focus != Focus::Sections;
        let list = (!self.list.items().is_empty()).then(|| {
            let mut list = self.list.view();
            for item in &mut list.items {
                item.focused &= in_list;
            }
            list
        });
        CampView {
            heading: ctx.text("camp.heading").to_owned(),
            sections: self.sections.view(),
            sections_focused: self.focus == Focus::Sections,
            list_title: title,
            list,
            empty: ctx.text(empty).to_owned(),
            help: self.help(ctx),
        }
    }

    /// Starts the music, the first time camp is updated.
    fn start_music(&mut self, ctx: &mut Ctx) {
        if self.music.is_some() {
            return;
        }
        let manifest = &ctx.game.content.audio;
        let Some(pool) = manifest.pools.get(POOL) else {
            return;
        };
        let tracks: Vec<(String, f64)> = pool
            .iter()
            .filter_map(|cue| {
                let track = manifest.music.get(cue)?;
                Some((cue.clone(), f64::from(track.length_ms) / 1000.0))
            })
            .collect();
        let fade_s = f64::from(manifest.music_fade_ms) / 1000.0;
        let first = ctx.pick_music(POOL);
        let first = tracks.iter().position(|(cue, _)| Some(cue) == first.as_ref());
        let started = CampMusic::start(tracks, first.unwrap_or(0), fade_s, ctx.clock_s);
        if let Some((music, cue)) = started {
            self.music = Some(music);
            ask(ctx, &cue);
        }
    }

    /// Plays `scene` for the army as it is, and the pair or the event's
    /// characters, who are there whatever happened since (a partner who
    /// left keeps the conversations seen).
    fn watch(ctx: &Ctx, campaign: &Campaign, scene: &str, with: &[&CharacterId]) -> Transition {
        let Some(scene) = ctx.game.content.dialogue.get(scene) else {
            return Transition::None;
        };
        let army = campaign.roster.iter().filter_map(|u| u.character.clone());
        let present = Present::only(army.chain(with.iter().map(|&c| c.clone())));
        let screen = DialogueScreen::told(ctx, scene, campaign.lead.clone(), &present);
        Transition::Push(Box::new(screen))
    }

    /// Confirm on `row` of the list.
    fn choose(&mut self, ctx: &mut Ctx, row: Row) -> Transition {
        let content = &ctx.game.content;
        match row {
            Row::Unit(c) => {
                self.focus = Focus::Talks(c);
                self.refresh(ctx, 0);
                Transition::None
            }
            Row::Talk(t) => {
                let table: &SupportTable = &content.supports;
                let scene = if t.new {
                    match self.campaign.view_support(&t.unit, &t.partner, table) {
                        Ok(viewed) => viewed.conversation,
                        Err(_) => return Transition::None,
                    }
                } else {
                    let pair = SupportPair::new(t.unit.clone(), t.partner.clone());
                    match table.get(&pair) {
                        Some(def) => def.conversations.get(t.rank).clone(),
                        None => return Transition::None,
                    }
                };
                Self::watch(ctx, &self.campaign, &scene, &[&t.unit, &t.partner])
            }
            Row::Event(i) => {
                let event = &content.camp[i];
                self.campaign.see_camp_event(&event.id);
                let with: Vec<&CharacterId> = event.units.iter().collect();
                Self::watch(ctx, &self.campaign, &event.scene, &with)
            }
        }
    }

    /// One action in the sections.
    fn on_sections(&mut self, ctx: &mut Ctx, action: Action) -> Transition {
        let before = self.sections.focus();
        let event = self.sections.handle_with_sound(action, &mut ctx.audio);
        match event {
            Some(MenuEvent::Chosen(Section::Leave)) => {
                // Camp's track fades out, unless what comes next asks for
                // music of its own this frame.
                ctx.audio.stop_music();
                return Transition::pop_with(self.campaign.clone());
            }
            Some(MenuEvent::Chosen(Section::Supports)) => self.focus = Focus::Units,
            Some(MenuEvent::Chosen(Section::Events)) => self.focus = Focus::Events,
            Some(MenuEvent::Cancelled) => {
                let leave = Section::ALL.len() - 1;
                self.sections = self.sections.clone().focused(leave);
            }
            None => {}
        }
        if self.sections.focus() != before || self.focus != Focus::Sections {
            self.refresh(ctx, 0);
        }
        // An empty list can't take the focus.
        if self.focus != Focus::Sections && self.list.items().is_empty() {
            self.focus = Focus::Sections;
            self.refresh(ctx, 0);
        }
        Transition::None
    }

    /// One action in the list.
    fn on_list(&mut self, ctx: &mut Ctx, action: Action) -> Transition {
        match self.list.handle_with_sound(action, &mut ctx.audio) {
            Some(MenuEvent::Chosen(row)) => self.choose(ctx, row),
            Some(MenuEvent::Cancelled) => {
                // Back to the units, on the one just open; or to the
                // sections.
                let open = match &self.focus {
                    Focus::Talks(unit) => Some(Row::Unit(unit.clone())),
                    _ => None,
                };
                self.focus = if open.is_some() {
                    Focus::Units
                } else {
                    Focus::Sections
                };
                self.refresh(ctx, 0);
                let at = self.rows().iter().position(|r| Some(r) == open.as_ref());
                self.refresh(ctx, at.unwrap_or(0));
                Transition::None
            }
            None => Transition::None,
        }
    }
}

/// Asks for `cue` on `ctx`'s audio.
fn ask(ctx: &mut Ctx, cue: &MusicCue) {
    match cue {
        MusicCue::Play(cue) => ctx.audio.play_music(cue),
        MusicCue::Stop => ctx.audio.stop_music(),
    }
}

/// The support conversations of `unit` in `campaign`, partner by partner
/// in the table's order, lowest rank first: those seen, then the one
/// unlocked (with both in the army, as viewing needs).
fn talks<'a>(
    campaign: &'a Campaign,
    table: &'a SupportTable,
    unit: &'a CharacterId,
) -> impl Iterator<Item = Talk> + 'a {
    let pairs = table.pairs().filter(move |d| d.pair.has(unit));
    pairs.flat_map(move |def| {
        let [a, b] = def.pair.members();
        let partner = if a == unit { b } else { a };
        let state = campaign.supports.state(&def.pair, table).unwrap_or_default();
        let talk = move |rank, new| Talk {
            unit: unit.clone(),
            partner: partner.clone(),
            rank,
            new,
        };
        let seen: Vec<Talk> = state.seen().map(|r| talk(r, false)).collect();
        let both = campaign.member(unit).is_some() && campaign.member(partner).is_some();
        let unlocked = state.unlocked().filter(|_| both).map(|r| talk(r, true));
        seen.into_iter().chain(unlocked)
    })
}

/// The camp events available in `campaign`, by index into `events`.
fn events(campaign: &Campaign, events: &[CampEvent]) -> Vec<usize> {
    let available = |(i, e): (usize, &CampEvent)| e.available(campaign).then_some(i);
    events.iter().enumerate().filter_map(available).collect()
}

impl Screen<Shuyi> for CampScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        self.start_music(ctx);
        if let Some(cue) = self.music.as_mut().and_then(|m| m.tick(ctx.clock_s)) {
            ask(ctx, &cue);
        }
        for &action in &input.actions {
            let t = if self.focus == Focus::Sections {
                self.on_sections(ctx, action)
            } else {
                self.on_list(ctx, action)
            };
            if !matches!(t, Transition::None) {
                return t;
            }
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn core::any::Any> {
        Some(self)
    }

    /// Back from a conversation: the lists again (what was new isn't), and
    /// camp's track again if the conversation played music of its own.
    fn uncovered(&mut self, ctx: &mut Ctx, _result: Option<ScreenResult>) -> Transition {
        let row = self.list.focus();
        self.refresh(ctx, row);
        let playing = ctx.music_clock.as_ref().map(|c| c.cue.as_str());
        if let Some(music) = &mut self.music
            && playing != Some(music.track())
            && let Some(cue) = music.resume(ctx.clock_s)
        {
            ask(ctx, &cue);
        }
        Transition::None
    }
}

#[cfg(test)]
mod tests;
