//! EXP bar and level-up screen (ticket 0602): after a command's combat
//! playback (or at once, for a heal or a tile cast), each player unit that
//! gained something gets its pages in turn: the EXP bar filling from its old
//! EXP to its new ([`Event::ExpGained`], wrapping at 100 for each level),
//! then one level-up page per [`Event::LeveledUp`] (portrait, `LEVEL UP!`,
//! `Lv 4 → 5` and each stat revealed one row at a time), then its class
//! progress ([`Event::ClassLeveledUp`], [`Event::ClassMastered`] and a
//! `Learned: <name>` line per [`Event::SkillLearned`] and
//! [`Event::SpellLearned`]). Everything shown comes from the events; this
//! only times them and says what the page shows ([`view`]); the battle
//! skin paints it (ADR-0054).
//!
//! A class change (ticket 0603; between battles, on the class-choice
//! screen) uses the same pages: a promotion ([`Event::Promoted`]) shows the
//! level-up panel with `PROMOTED!`, `Guard → Iron Rider` and its stat
//! bonus; a reclass ([`Event::Reclassed`]) shows the class box with
//! `Guard → Mage`. Spells learned and items sent to the stock
//! ([`Event::ItemStowed`]) are listed in the class box.
//!
//! Holding Confirm plays a page [`ProgressTimings::fast`] times as fast;
//! pressing it finishes the page's animation (reveals every stat), and on a
//! finished page that waits for the player, closes it. Cancel does the same
//! (Nick, PR #87: "cancel can do the same as confirm on these screens").

use crate::prelude::*;
use alloc::collections::BTreeMap;

use trpg_content::Portrait;
use trpg_core::lead::LeadProfile;
use trpg_core::progression::EXP_PER_LEVEL;
use trpg_core::{
    BattleState, ClassId, ClassLevel, ClassTable, Event, Faction, ItemTable, Level, SkillTable,
    SpellTable, StatGains, StatKind, StatValue, Unit, UnitId,
};

use super::info::stat_name;
use super::view::{ClassProgressView, ExpView, GainsView, ProgressView, StatGainView};
use crate::screen::Ctx;
use crate::tips::view::FaceView;
use crate::words::Words;

/// How long each page's parts take, in seconds. *Tunable.*
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressTimings {
    /// The EXP bar filling from the old EXP to the new.
    pub exp_fill: f32,
    /// The filled EXP bar stays up before the next page.
    pub exp_hold: f32,
    /// The level-up page before its first stat row.
    pub level_intro: f32,
    /// Between two stat rows appearing.
    pub stat_reveal: f32,
    /// A class-level notice (no mastery, nothing learned) stays up.
    pub class_hold: f32,
    /// Speed-up while Confirm is held.
    pub fast: f32,
}

/// The game's timings (ticket 0602: the bar fills over ~0.6 s, one stat
/// every ~0.12 s; ×4 while Confirm is held, like the combat playback).
pub const PROGRESS_TIMINGS: ProgressTimings = ProgressTimings {
    exp_fill: 0.6,
    exp_hold: 0.5,
    level_intro: 0.3,
    stat_reveal: 0.12,
    class_hold: 1.2,
    fast: 4.0,
};

/// The EXP bar of one unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpPage {
    /// The unit.
    pub unit: UnitId,
    /// Its name.
    pub name: String,
    /// Its EXP before the command (towards its next level).
    pub from: u32,
    /// EXP gained (all of the command's awards to it).
    pub gained: u32,
}

/// One level up of one unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelUpPage {
    /// The unit.
    pub unit: UnitId,
    /// Its name.
    pub name: String,
    /// Its character's portrait id, if it is a named character.
    pub character: Option<String>,
    /// The new level.
    pub level: Level,
    /// Each growable stat before this level up ([`StatKind::GROWABLE`]
    /// order).
    pub before: [StatValue; 7],
    /// What each stat gained.
    pub gains: StatGains,
}

/// The stat bonus of one promotion (ticket 0603), shown like a level up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionPage {
    /// The unit.
    pub unit: UnitId,
    /// Its name.
    pub name: String,
    /// Its character's portrait id, if it is a named character.
    pub character: Option<String>,
    /// The name of the class it left.
    pub from: String,
    /// The name of its new class.
    pub to: String,
    /// Each growable stat before the promotion ([`StatKind::GROWABLE`]
    /// order).
    pub before: [StatValue; 7],
    /// The promotion bonus of each stat.
    pub gains: StatGains,
}

/// A unit's class progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassPage {
    /// The unit.
    pub unit: UnitId,
    /// Its name.
    pub name: String,
    /// The class's name (`Guard → Mage` for a reclass).
    pub class: String,
    /// The class level before and after, if it rose.
    pub class_levels: Option<(ClassLevel, ClassLevel)>,
    /// Whether the class was mastered.
    pub mastered: bool,
    /// Names of the skills and spells learned, in event order.
    pub learned: Vec<String>,
    /// Whether the unit changed class with a Reclass Seal.
    pub changed: bool,
    /// Names of the items a class change sent to the stock, in event order.
    pub stowed: Vec<String>,
}

/// One page of the sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Page {
    /// The EXP bar.
    Exp(ExpPage),
    /// A level up.
    LevelUp(LevelUpPage),
    /// A promotion's stat bonus.
    Promotion(PromotionPage),
    /// Class progress.
    Class(ClassPage),
}

impl Page {
    /// How long the page's animation lasts at normal speed.
    pub fn len(&self, t: &ProgressTimings) -> f32 {
        match self {
            Page::Exp(_) => t.exp_fill + t.exp_hold,
            // Seven stats: small, exact.
            #[allow(clippy::cast_precision_loss)]
            Page::LevelUp(_) | Page::Promotion(_) => {
                t.level_intro + t.stat_reveal * StatKind::GROWABLE.len() as f32
            }
            Page::Class(c) if c.waits() => 0.0,
            Page::Class(_) => t.class_hold,
        }
    }

    /// Whether the page stays up, once played, until Confirm closes it
    /// (otherwise it closes by itself).
    pub fn waits(&self) -> bool {
        match self {
            Page::Exp(_) => false,
            Page::LevelUp(_) | Page::Promotion(_) => true,
            Page::Class(c) => c.waits(),
        }
    }
}

impl ClassPage {
    /// A mastery, something learned or a class change is worth stopping
    /// for.
    fn waits(&self) -> bool {
        self.mastered || !self.learned.is_empty() || self.changed || !self.stowed.is_empty()
    }
}

/// The content tables that name what a unit gained.
#[derive(Debug, Clone, Copy)]
pub struct ProgressTables<'a> {
    /// Classes.
    pub classes: &'a ClassTable,
    /// Skills.
    pub skills: &'a SkillTable,
    /// Spells.
    pub spells: &'a SpellTable,
    /// Items.
    pub items: &'a ItemTable,
    /// The language the names are shown in.
    pub words: Words<'a>,
}

impl<'a> ProgressTables<'a> {
    /// The tables of the battle `state`.
    pub fn of(state: &'a BattleState, words: Words<'a>) -> Self {
        Self {
            classes: state.classes(),
            skills: state.skills(),
            spells: state.spells(),
            items: state.items(),
            words,
        }
    }

    /// The name of class `id` (its id if it is missing).
    fn class_name(&self, id: &ClassId) -> String {
        let class = self.classes.get(id);
        class
            .map_or(id.0.as_str(), |c| self.words.class(c))
            .to_owned()
    }
}

/// What a unit gained in one command, gathered from its events.
#[derive(Default)]
struct Gathered {
    exp: Option<u32>,
    levels: Vec<(Level, StatGains)>,
    class: Option<ClassId>,
    class_levels: Option<(ClassLevel, ClassLevel)>,
    mastered: bool,
    learned: Vec<String>,
    /// A promotion: the classes' names and the bonus.
    promoted: Option<(String, String, StatGains)>,
    /// A reclass: the classes' names.
    reclassed: Option<(String, String)>,
    stowed: Vec<String>,
}

impl Gathered {
    /// Adds `event` (one of this unit's); `tables` names what it gained.
    fn add(&mut self, event: &Event, tables: &ProgressTables<'_>) {
        match event {
            Event::ExpGained { amount, .. } => {
                self.exp = Some(self.exp.unwrap_or(0).saturating_add(*amount));
            }
            Event::LeveledUp { level, gains, .. } => self.levels.push((*level, *gains)),
            Event::ClassLeveledUp {
                class, class_level, ..
            } => {
                self.class = Some(class.clone());
                let from = self
                    .class_levels
                    .map_or(class_level.saturating_sub(1), |(from, _)| from);
                self.class_levels = Some((from, *class_level));
            }
            Event::ClassMastered { class, .. } => {
                self.class = Some(class.clone());
                self.mastered = true;
            }
            Event::SkillLearned { skill, .. } => {
                let skill = tables.skills.get(skill);
                let name = skill.map(|s| tables.words.skill(s).to_owned());
                self.learned.extend(name);
            }
            Event::SpellLearned { spell, .. } => {
                let spell = tables.spells.get(spell);
                let name = spell.map(|s| tables.words.spell(s).to_owned());
                self.learned.extend(name);
            }
            Event::Promoted {
                from, to, gains, ..
            } => {
                self.class = Some(to.clone());
                self.promoted = Some((tables.class_name(from), tables.class_name(to), *gains));
            }
            Event::Reclassed { from, to, .. } => {
                self.class = Some(to.clone());
                self.reclassed = Some((tables.class_name(from), tables.class_name(to)));
            }
            Event::ItemStowed { item, .. } => {
                let known = tables.items.get(item).is_some();
                let name = known.then(|| tables.words.item(item, tables.items).to_owned());
                self.stowed.extend(name);
            }
            _ => {}
        }
    }

    /// `unit`'s pages (as it was before the command): its EXP bar, a page
    /// per level up (each starting from the stats the last one left), a
    /// promotion's bonus, and its class progress.
    fn pages(self, unit: &Unit, tables: &ProgressTables<'_>) -> Vec<Page> {
        let mut pages = Vec::new();
        let (id, name) = (unit.id, tables.words.unit(unit).to_owned());
        if let Some(gained) = self.exp {
            pages.push(Page::Exp(ExpPage {
                unit: id,
                name: name.clone(),
                from: unit.exp,
                gained,
            }));
        }
        let mut running = StatKind::GROWABLE.map(|k| unit.stats.get(k));
        for (level, gains) in self.levels {
            pages.push(Page::LevelUp(LevelUpPage {
                unit: id,
                name: name.clone(),
                character: unit.character.as_ref().map(|c| c.0.clone()),
                level,
                before: running,
                gains,
            }));
            for (v, g) in running.iter_mut().zip(gains.0) {
                *v = v.saturating_add(g);
            }
        }
        if let Some((from, to, gains)) = self.promoted {
            pages.push(Page::Promotion(PromotionPage {
                unit: id,
                name: name.clone(),
                character: unit.character.as_ref().map(|c| c.0.clone()),
                from,
                to,
                before: running,
                gains,
            }));
        }
        let changed = self.reclassed.is_some();
        let progressed = self.class_levels.is_some() || self.mastered || changed;
        if progressed || !self.learned.is_empty() || !self.stowed.is_empty() {
            let class = match self.reclassed {
                Some((from, to)) => format!("{from} → {to}"),
                None => tables.class_name(&self.class.unwrap_or_else(|| unit.class.clone())),
            };
            pages.push(Page::Class(ClassPage {
                unit: id,
                name,
                class,
                class_levels: self.class_levels,
                mastered: self.mastered,
                learned: self.learned,
                changed,
                stowed: self.stowed,
            }));
        }
        pages
    }
}

/// The unit an EXP, level, class or learning event is about.
fn progress_unit(event: &Event) -> Option<UnitId> {
    match event {
        Event::ExpGained { unit, .. }
        | Event::LeveledUp { unit, .. }
        | Event::ClassLeveledUp { unit, .. }
        | Event::ClassMastered { unit, .. }
        | Event::SkillLearned { unit, .. }
        | Event::SpellLearned { unit, .. }
        | Event::Promoted { unit, .. }
        | Event::Reclassed { unit, .. }
        | Event::ItemStowed { unit, .. } => Some(*unit),
        _ => None,
    }
}

/// The sequence of pages after one command, and where it is.
#[derive(Debug, Clone, PartialEq)]
pub struct Progress {
    pages: Vec<Page>,
    page: usize,
    t: f32,
    timings: ProgressTimings,
}

impl Progress {
    /// The pages for `events` (one applied command), unit by unit in the
    /// order they first gained something, or `None` if no player unit
    /// gained anything. `before` is every unit before the command (for the
    /// old EXP and stats); `state` is the battle after it (for class, skill
    /// and spell names).
    pub fn new(
        events: &[Event],
        before: &[Unit],
        (state, words): (&BattleState, Words<'_>),
        timings: ProgressTimings,
    ) -> Option<Self> {
        let tables = ProgressTables::of(state, words);
        Self::with_tables(events, before, &tables, timings)
    }

    /// [`Progress::new`] with the content `tables` instead of a battle's:
    /// for events made outside a battle (a class change).
    pub fn with_tables(
        events: &[Event],
        before: &[Unit],
        tables: &ProgressTables<'_>,
        timings: ProgressTimings,
    ) -> Option<Self> {
        let mut gathered: Vec<(&Unit, Gathered)> = Vec::new();
        for event in events {
            let Some(id) = progress_unit(event) else {
                continue;
            };
            if !gathered.iter().any(|(u, _)| u.id == id) {
                let player = |u: &&Unit| u.id == id && u.faction == Faction::Player;
                let Some(unit) = before.iter().find(player) else {
                    continue;
                };
                gathered.push((unit, Gathered::default()));
            }
            if let Some((_, g)) = gathered.iter_mut().find(|(u, _)| u.id == id) {
                g.add(event, tables);
            }
        }
        let pages: Vec<Page> = gathered
            .into_iter()
            .flat_map(|(unit, g)| g.pages(unit, tables))
            .collect();
        if pages.is_empty() {
            return None;
        }
        Some(Self {
            pages,
            page: 0,
            t: 0.0,
            timings,
        })
    }

    /// The same sequence without its EXP bars, for the results screen
    /// (0810), which shows every unit's bar itself; `None` if nothing is
    /// left.
    pub fn without_exp_bars(mut self) -> Option<Self> {
        self.pages.retain(|p| !matches!(p, Page::Exp(_)));
        (!self.pages.is_empty()).then_some(self)
    }

    /// Every page, in order.
    pub fn pages(&self) -> &[Page] {
        &self.pages
    }

    /// The page on screen, if any is left.
    pub fn page(&self) -> Option<&Page> {
        self.pages.get(self.page)
    }

    /// Which page is on screen (`pages().len()` once done).
    pub fn index(&self) -> usize {
        self.page
    }

    /// Seconds into the page on screen.
    pub fn time(&self) -> f32 {
        self.t
    }

    /// Whether every page has closed.
    pub fn done(&self) -> bool {
        self.page >= self.pages.len()
    }

    /// Whether the page on screen has played to its end.
    pub fn page_played(&self) -> bool {
        self.page().is_none_or(|p| self.t >= p.len(&self.timings))
    }

    /// Moves to the next page.
    fn next(&mut self) {
        self.page += 1;
        self.t = 0.0;
    }

    /// Advances the clock by `dt` seconds ([`ProgressTimings::fast`] times
    /// faster while Confirm is held); a page that doesn't wait closes once
    /// played. A bad `dt` counts as 0.
    pub fn tick(&mut self, dt: f32, confirm_held: bool) {
        let Some(page) = self.page() else {
            return;
        };
        let len = page.len(&self.timings);
        let waits = page.waits();
        let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
        let speed = if confirm_held { self.timings.fast } else { 1.0 };
        self.t = (self.t + dt * speed).min(len);
        if !waits && self.t >= len {
            self.next();
        }
    }

    /// Confirm was pressed: finishes the page's animation, or closes a
    /// played page.
    pub fn confirm(&mut self) {
        let Some(page) = self.page() else {
            return;
        };
        let len = page.len(&self.timings);
        if self.t < len {
            self.t = len;
        } else {
            self.next();
        }
    }

    /// The EXP page's shown EXP, counted from its old EXP (so `from +
    /// gained` when full; the bar shows it modulo [`EXP_PER_LEVEL`]).
    pub fn exp_shown(&self) -> Option<u32> {
        let Some(Page::Exp(p)) = self.page() else {
            return None;
        };
        let frac = (self.t / self.timings.exp_fill.max(f32::EPSILON)).clamp(0.0, 1.0);
        // EXP awards are small (at most a few levels): exact in f32.
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let moved = ((p.gained as f32 * frac) + 1e-3).floor() as u32;
        Some(p.from.saturating_add(moved.min(p.gained)))
    }

    /// How many of the level-up (or promotion) page's stat rows are shown.
    pub fn stats_shown(&self) -> usize {
        let Some(Page::LevelUp(_) | Page::Promotion(_)) = self.page() else {
            return 0;
        };
        let t = self.t - self.timings.level_intro;
        if t < 0.0 {
            return 0;
        }
        // Whole reveal periods since the intro, plus the first row.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = (t / self.timings.stat_reveal.max(f32::EPSILON) + 1e-3) as usize;
        (n + 1).min(StatKind::GROWABLE.len())
    }
}

/// The help line of an EXP bar, level-up or class page: skip and fast while
/// it plays, then continue.
pub fn help(ctx: &Ctx, p: &Progress) -> String {
    playing_help(ctx, p.page_played())
}

/// The help line of something that plays by itself (an EXP bar, the
/// results): skip and fast until it has `played`, then continue.
pub(crate) fn playing_help(ctx: &Ctx, played: bool) -> String {
    ctx.text_with(
        if played {
            "battle.help.continue"
        } else {
            "battle.help.playing"
        },
        &[],
    )
}

/// The expression shown on a level up, else the portrait's first.
// check-text: not player text (expression ids)
const LEVEL_UP_EXPRESSIONS: [&str; 2] = ["happy", "neutral"];

/// The text of stat `kind`'s row: `Str   6 → 7  +1` if it grew, else
/// `Def   5`.
pub fn stat_row(words: Words<'_>, kind: StatKind, before: StatValue, gain: StatValue) -> String {
    let (row, gain) = stat_row_parts(stat_name(words, kind), before, gain);
    row + &gain.unwrap_or_default()
}

/// [`stat_row`] of the stat called `name`, in two parts: the row up to the
/// gain (`Str   6 → 7  `), and the gain (`+1`) if it grew.
pub fn stat_row_parts(name: &str, before: StatValue, gain: StatValue) -> (String, Option<String>) {
    if gain > 0 {
        let row = format!("{name:<4}{before:>3} → {:<3} ", before + gain);
        (row, Some(format!("+{gain}")))
    } else {
        (format!("{name:<4}{before:>3}"), None)
    }
}

/// The class line: `Exile  CL 3 → 4`, or just the class's name.
pub fn class_line(words: Words<'_>, page: &ClassPage) -> String {
    match page.class_levels {
        Some((from, to)) => words.text_with(
            "battle.progress.class_levels",
            &[("class", &page.class), ("from", &from), ("to", &to)],
        ),
        None => page.class.clone(),
    }
}

/// The portrait of the character `id`: the lead's id stands for the
/// portrait of the gender the player picked.
fn portrait_of<'a>(
    portraits: &'a BTreeMap<String, Portrait>,
    lead: &LeadProfile,
    id: Option<&str>,
) -> Option<(&'a str, &'a Portrait)> {
    portraits
        .get_key_value(lead.portrait_for(id?))
        .map(|(id, p)| (id.as_str(), p))
}

/// The face a level-up page shows of the character `id`: its portrait's
/// level-up expression, else its first; `None` without a portrait.
fn face_of(
    (portraits, lead): (&BTreeMap<String, Portrait>, &LeadProfile),
    id: Option<&str>,
) -> Option<FaceView> {
    let (id, art) = portrait_of(portraits, lead, id)?;
    let expression = LEVEL_UP_EXPRESSIONS
        .iter()
        .copied()
        .find(|e| art.expression(e).is_some())
        // check-text: not a data name (an expression's)
        .or_else(|| art.expressions.first().map(|e| e.name.as_str()))?;
    Some(FaceView {
        portrait: id.to_owned(),
        expression: expression.to_owned(),
    })
}

/// What the page on screen shows, as far as it has played; `None` once
/// every page has closed. `portraits` and `lead` find the unit's face.
pub fn view(
    words: Words<'_>,
    faces: (&BTreeMap<String, Portrait>, &LeadProfile),
    progress: &Progress,
) -> Option<ProgressView> {
    let gains =
        |character: Option<&str>, banner: &str, name: &str, line: String, stats| GainsView {
            face: face_of(faces, character),
            placeholder: words.text("battle.portrait").to_owned(),
            banner: words.text(banner).to_owned(),
            name: name.to_owned(),
            line,
            stats,
        };
    let shown = progress.stats_shown();
    Some(match progress.page()? {
        Page::Exp(p) => {
            let exp = progress.exp_shown().unwrap_or(p.from);
            ProgressView::Exp(ExpView {
                // check-text: not a data name (the page's, named when it was made)
                name: p.name.clone(),
                exp_name: words.text("battle.exp").to_owned(),
                exp: exp % EXP_PER_LEVEL,
                max: EXP_PER_LEVEL,
            })
        }
        Page::LevelUp(p) => {
            let line = words.text_with(
                "battle.progress.levels",
                &[("from", &p.level.saturating_sub(1)), ("to", &p.level)],
            );
            let stats = stat_gains(words, (&p.before, &p.gains), shown);
            let banner = "battle.progress.level_up";
            // check-text: not a data name (the page's, named when it was made)
            ProgressView::Gains(gains(p.character.as_deref(), banner, &p.name, line, stats))
        }
        Page::Promotion(p) => {
            let line = format!("{} → {}", p.from, p.to);
            let stats = stat_gains(words, (&p.before, &p.gains), shown);
            let banner = "battle.progress.promoted";
            // check-text: not a data name (the page's, named when it was made)
            ProgressView::Gains(gains(p.character.as_deref(), banner, &p.name, line, stats))
        }
        Page::Class(p) => {
            let line = |key, name: &String| words.text_with(key, &[("name", name)]);
            let learned = p.learned.iter().map(|s| line("battle.progress.learned", s));
            let stowed = p.stowed.iter().map(|s| line("battle.progress.stowed", s));
            ProgressView::Class(ClassProgressView {
                // check-text: not a data name (the page's, named when it was made)
                name: p.name.clone(),
                line: class_line(words, p),
                mastered: p
                    .mastered
                    .then(|| words.text("battle.progress.mastered").to_owned()),
                lines: learned.chain(stowed).collect(),
            })
        }
    })
}

/// The first `shown` stat rows of the stats `before` with their `gains`.
fn stat_gains(
    words: Words<'_>,
    (before, gains): (&[StatValue; 7], &StatGains),
    shown: usize,
) -> Vec<StatGainView> {
    StatKind::GROWABLE
        .iter()
        .enumerate()
        .take(shown)
        .map(|(i, kind)| StatGainView {
            name: stat_name(words, *kind).to_owned(),
            before: before[i],
            gain: gains.0[i],
        })
        .collect()
}

#[cfg(test)]
mod tests;
