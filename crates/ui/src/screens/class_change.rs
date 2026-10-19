//! The class-choice screen (ticket 0603; `docs/design/progression.md`,
//! *Promotion* and *Reclass*): a unit promotes or changes class **between
//! battles**, spending a seal from the party's stock (Nick: no seal works
//! in a battle).
//!
//! One column per class the unit can go to, side by side: the class's
//! name, tier (and, for a reclass, the class level saved there or `new`),
//! Mov and movement type, tags, weapon kinds with the unit's rank before
//! and after, the active skill it gives, and every stat `current → new`
//! with the gain highlighted. A stat at its hard ceiling is marked `MAX`
//! (shown out of battle only: `progression.md`, *Showing a maxed stat*).
//! Every number is what [`promote`] or [`reclass`] does to a copy of the
//! unit. A class the change is refused for (no seal in stock, class not
//! mastered) is dimmed, with the reason, and can't be chosen.
//!
//! Left and right pick the column (the columns scroll when there are more
//! than [`COLUMNS`]), Confirm chooses it and asks `Promote … ?`, Confirm
//! again makes the change, and its result plays as the level-up overlay
//! ([`Progress`]): the stat bonus of a promotion, spells learned, items
//! sent to the stock (on an empty screen: the pages sit where they do over
//! the battle map). Then the screen closes. Cancel backs out of the
//! question, or closes the screen with nothing changed.
//!
//! Until the between-battle menus exist (Preparations, 0408; camp), the
//! screen is reached from the debug menu with a test unit
//! ([`ClassChangeScreen::demo`]).
//!
//! The screen says what it shows as a [`ClassChangeView`]
//! ([`ClassChangeScreen::view`]); [`glyph::paint`] draws that view.

pub mod glyph;
pub mod view;

use crate::prelude::*;
pub use view::{
    ClassChangeBody, ClassChangeView, ColumnView, ColumnsView, QuestionView, SealView, StatRowView,
};

use trpg_core::progression::class_change::ChangeTables;
use trpg_core::{
    CharacterId, ClassChangeError, ClassDef, ClassId, ClassRecord, Event, Faction, ItemDef, Pos,
    SealKind, StatKind, StatValue, Stock, Unit, UnitId, promote, promotion_targets, reclass,
    reclass_targets,
};

use super::battle::info::{class_level, stat_name, tag_names};
use super::battle::progress::{self, PROGRESS_TIMINGS, Progress, ProgressTables};
use super::battle::skills::kind_name;
use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::widgets::ListCursor;
use crate::widgets::help::{SEPARATOR, key_name};
use crate::words::Words;

/// Columns shown at once.
pub const COLUMNS: usize = 4;

/// The key of the mark after a stat at its hard ceiling (`MAX`).
pub const MAX_MARK: &str = "class_change.max";

/// What kind of class change the screen makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    /// A promotion, with the seal of the new class's tier.
    Promote,
    /// A full class change, with a Reclass Seal.
    Reclass,
}

impl ChangeKind {
    /// The screen's title.
    const fn title(self) -> &'static str {
        match self {
            ChangeKind::Promote => "class_change.title.promote",
            ChangeKind::Reclass => "class_change.title.reclass",
        }
    }

    /// The seal a change into `class` uses up.
    const fn seal(self, class: &ClassDef) -> SealKind {
        match self {
            ChangeKind::Promote => SealKind::Tier(class.tier),
            ChangeKind::Reclass => SealKind::Reclass,
        }
    }
}

/// One class the unit could change to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassOption {
    /// The class.
    pub class: ClassId,
    /// The unit after the change, or why the change is refused.
    pub after: Result<Unit, ClassChangeError>,
}

/// Where the screen is.
#[derive(Debug, Clone, PartialEq)]
enum Stage {
    /// Picking a column.
    Choosing,
    /// `Promote … ?`
    Confirming,
    /// The change was made: its pages play.
    Result(Progress),
    /// Nothing left to show: the screen closes.
    Done,
}

/// The class-choice screen. See the module docs.
#[derive(Debug, Clone)]
pub struct ClassChangeScreen {
    kind: ChangeKind,
    unit: Unit,
    stock: Stock,
    options: Vec<ClassOption>,
    /// The column the cursor is on, and the columns shown.
    cursor: ListCursor,
    stage: Stage,
}

/// The tables a class change reads, from the game's content.
fn tables(ctx: &Ctx) -> ChangeTables<'_> {
    ChangeTables {
        classes: &ctx.game.content.classes,
        items: &ctx.game.content.items,
        spells: &ctx.game.content.spells,
    }
}

/// Makes the `kind` of change of `unit` into `target`.
fn change(
    kind: ChangeKind,
    unit: &mut Unit,
    target: &ClassId,
    stock: &mut Stock,
    tables: &ChangeTables<'_>,
) -> Result<Vec<Event>, ClassChangeError> {
    match kind {
        ChangeKind::Promote => promote(unit, target, stock, tables),
        ChangeKind::Reclass => reclass(unit, target, stock, tables),
    }
}

/// Stat `kind`'s row, `before` the change and `after` it, marked
/// [`at_max`](StatRowView::at_max) if it ends at its hard `ceiling` (or
/// past it). A skin writes it as `Str   9 → 12  +3`, or `Def  49 MAX`
/// ([`glyph::stat_line`]).
pub fn stat_row(
    words: Words<'_>,
    kind: StatKind,
    before: StatValue,
    after: StatValue,
    ceiling: StatValue,
) -> StatRowView {
    StatRowView {
        name: stat_name(words, kind).to_owned(),
        before,
        after,
        at_max: after >= ceiling,
    }
}

/// The Mov row: `Mov  4 → 6` if it changes, else `Mov  5`.
pub fn mov_line(words: Words<'_>, before: StatValue, after: StatValue) -> String {
    let name = stat_name(words, StatKind::Mov);
    if before == after {
        format!("{name:<4}{before:>3}")
    } else {
        format!("{name:<4}{before:>3} → {after}")
    }
}

/// The weapon rows of `class` for `unit`: each kind the class wields with
/// the unit's rank now and on entering the class, `Spear    D → C`, or
/// just the rank if it stays.
pub fn weapon_lines(words: Words<'_>, unit: &Unit, class: &ClassDef) -> Vec<String> {
    class
        .weapons
        .iter()
        .map(|w| {
            let now = unit.rank(w.kind);
            let then = now.max(w.start);
            let kind = kind_name(words, w.kind);
            if now == then {
                format!("{kind:<9}{now:?}")
            } else {
                format!("{kind:<9}{now:?} → {then:?}")
            }
        })
        .collect()
}

impl ClassChangeScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "class_change";

    /// The screen for `unit` and the party's `stock`: the classes a
    /// promotion ([`promotion_targets`]) or a Reclass Seal
    /// ([`reclass_targets`]) offers it, each with the unit as the change
    /// would leave it.
    pub fn new(ctx: &Ctx, kind: ChangeKind, unit: Unit, stock: Stock) -> Self {
        let tables = tables(ctx);
        let targets = match kind {
            ChangeKind::Promote => promotion_targets(&unit, tables.classes),
            ChangeKind::Reclass => reclass_targets(&unit, tables.classes),
        };
        let options: Vec<ClassOption> = targets
            .into_iter()
            .map(|class| {
                let (mut after, mut stock) = (unit.clone(), stock.clone());
                let result = change(kind, &mut after, &class.id, &mut stock, &tables);
                ClassOption {
                    class: class.id.clone(),
                    after: result.map(|_| after),
                }
            })
            .collect();
        Self {
            kind,
            unit,
            stock,
            cursor: ListCursor::new(options.len()),
            options,
            stage: Stage::Choosing,
        }
    }

    /// The screen on a test unit, for the debug menu: the placeholder
    /// knight (a Guard) at level 12, its Guard mastered and a Rider record
    /// saved at class level 6, Def one short of its hard ceiling (so a
    /// `MAX` shows), and a stock with one of every seal. `None` if the
    /// content has no such character.
    pub fn demo(ctx: &Ctx, kind: ChangeKind) -> Option<Self> {
        let content = &ctx.game.content;
        let classes = &content.classes;
        let def = content
            .characters
            .characters
            .get(&CharacterId("test_knight".into()))?;
        let mut unit = trpg_content::character_unit(
            def,
            UnitId(1),
            classes,
            &content.items,
            Faction::Player,
            Pos::new(0, 0),
        )
        .ok()?;
        unit.level = 12;
        unit.exp = 40;
        unit.stats.hp += 8;
        unit.stats.str += 4;
        unit.stats.dex += 3;
        unit.stats.spd += 3;
        unit.stats.def = classes.hard_ceilings.def - 1;
        unit.hp = unit.stats.hp;
        let mastered = ClassRecord {
            class_level: classes.class_level_cap,
            class_points: 0,
        };
        unit.class_records.insert(unit.class.clone(), mastered);
        let saved = ClassRecord {
            class_level: 6,
            class_points: 0,
        };
        unit.class_records.insert(ClassId("rider".into()), saved);
        let mut stock = Stock::default();
        for (id, item) in &content.items.items {
            if matches!(item, ItemDef::Seal(_)) {
                stock.add(id.clone());
            }
        }
        Some(Self::new(ctx, kind, unit, stock))
    }

    /// The unit, as it is now (changed once a change was confirmed).
    pub fn unit(&self) -> &Unit {
        &self.unit
    }

    /// The party's stock, as it is now.
    pub fn stock(&self) -> &Stock {
        &self.stock
    }

    /// The classes offered, in column order.
    pub fn options(&self) -> &[ClassOption] {
        &self.options
    }

    /// The column the cursor is on.
    pub fn focus(&self) -> usize {
        self.cursor.focus()
    }

    /// Whether the `Promote … ?` question is up.
    pub fn is_confirming(&self) -> bool {
        self.stage == Stage::Confirming
    }

    /// The result's pages, once a change was made.
    pub fn progress(&self) -> Option<&Progress> {
        match &self.stage {
            Stage::Result(p) => Some(p),
            _ => None,
        }
    }

    /// The option the cursor is on.
    fn focused(&self) -> Option<&ClassOption> {
        self.options.get(self.cursor.focus())
    }

    /// Moves the cursor one column (wrapping) and scrolls it into view.
    /// Returns whether it moved.
    fn move_focus(&mut self, forward: bool) -> bool {
        let moved = self.cursor.step(forward, true, |_| true);
        // The window of `COLUMNS` columns moves just far enough to hold it.
        self.cursor.window(COLUMNS);
        moved
    }

    /// Makes the change into the focused class and starts its pages.
    fn apply(&mut self, ctx: &Ctx) {
        let Some(target) = self.focused().map(|o| o.class.clone()) else {
            self.stage = Stage::Choosing;
            return;
        };
        let before = self.unit.clone();
        let made = change(
            self.kind,
            &mut self.unit,
            &target,
            &mut self.stock,
            &tables(ctx),
        );
        self.stage = match made {
            Ok(events) => {
                let content = &ctx.game.content;
                let names = ProgressTables {
                    classes: &content.classes,
                    skills: &content.skills,
                    spells: &content.spells,
                    items: &content.items,
                    words: ctx.words(),
                };
                Progress::with_tables(&events, &[before], &names, PROGRESS_TIMINGS)
                    .map_or(Stage::Done, Stage::Result)
            }
            // Only an option whose preview worked can be confirmed.
            Err(_) => Stage::Choosing,
        };
    }

    /// One action while picking a column. Returns whether to close.
    fn step_choosing(&mut self, ctx: &mut Ctx, action: Action) -> bool {
        match action {
            Action::CursorLeft | Action::CursorRight => {
                if self.move_focus(action == Action::CursorRight) {
                    ctx.audio.menu(MenuSound::Move);
                }
            }
            Action::Confirm => {
                if self.focused().is_some_and(|o| o.after.is_ok()) {
                    self.stage = Stage::Confirming;
                    ctx.audio.menu(MenuSound::Select);
                } else {
                    ctx.audio.menu(MenuSound::Denied);
                }
            }
            Action::Cancel => {
                ctx.audio.menu(MenuSound::Cancel);
                return true;
            }
            _ => {}
        }
        false
    }

    /// The name of the seal a change into `class` uses and how many the
    /// stock holds: the seal's item name, or the kind if no item is one.
    fn seal_of(&self, ctx: &Ctx, class: &ClassDef) -> (String, u32) {
        let kind = self.kind.seal(class);
        let items = &ctx.game.content.items;
        let held: u32 = self
            .stock
            .items
            .iter()
            .filter(|(id, _)| items.seal(id).is_some_and(|s| s.kind == kind))
            .map(|(_, n)| n)
            .sum();
        let words = ctx.words();
        let name = items
            .items
            .iter()
            .find_map(|(id, d)| match d {
                ItemDef::Seal(s) if s.kind == kind => Some(words.item(id, items).to_owned()),
                _ => None,
            })
            .unwrap_or_else(|| match kind {
                SealKind::Tier(tier) => ctx.text_with("class_change.seal.tier", &[("tier", &tier)]),
                SealKind::Reclass => ctx.text("class_change.seal.reclass").to_owned(),
            });
        (name, held)
    }

    /// Why the change into `class` is refused, in the player's words.
    fn reason(&self, ctx: &Ctx, class: &ClassDef, error: &ClassChangeError) -> String {
        match error {
            ClassChangeError::NoSeal(_) => {
                let seal = self.seal_of(ctx, class).0;
                ctx.text_with("class_change.reason.no_seal", &[("seal", &seal)])
            }
            ClassChangeError::NotMastered(_) => {
                ctx.text("class_change.reason.not_mastered").to_owned()
            }
            other => other.to_string(),
        }
    }

    /// The question asked before the change, and what it costs.
    fn question(&self, ctx: &Ctx, class: &ClassDef) -> [String; 2] {
        let words = ctx.words();
        let (unit, class_name) = (words.unit(&self.unit), words.class(class));
        let key = match self.kind {
            ChangeKind::Promote => "class_change.question.promote",
            ChangeKind::Reclass => "class_change.question.reclass",
        };
        let ask = ctx.text_with(key, &[("unit", &unit), ("class", &class_name)]);
        let (seal, held) = self.seal_of(ctx, class);
        let cost = ctx.text_with("class_change.cost", &[("seal", &seal), ("held", &held)]);
        [ask, cost]
    }

    /// The help line for the stage.
    pub fn help(&self, ctx: &Ctx) -> String {
        let km = ctx.help_keys();
        match &self.stage {
            Stage::Choosing => {
                let pick = format!(
                    "{}/{}",
                    key_name(km, Action::CursorLeft),
                    key_name(km, Action::CursorRight)
                );
                let many = (self.options.len() > 1)
                    .then(|| ctx.text_with("class_change.hint.class", &[("keys", &pick)]));
                let can = self.focused().is_some_and(|o| o.after.is_ok());
                let choose = can.then(|| ctx.text_with("class_change.hint.choose", &[]));
                let back = Some(ctx.text_with("class_change.hint.back", &[]));
                let hints: Vec<String> = [many, choose, back].into_iter().flatten().collect();
                hints.join(SEPARATOR)
            }
            Stage::Confirming => ctx.text_with("class_change.help.confirming", &[]),
            Stage::Result(p) => progress::help(ctx, p),
            Stage::Done => String::new(),
        }
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint`]): the columns (with the question while it is
    /// open), or the change's pages, and the help line.
    pub fn view(&self, ctx: &Ctx) -> ClassChangeView {
        let body = match &self.stage {
            Stage::Choosing => ClassChangeBody::Columns(Box::new(self.columns_view(ctx, false))),
            Stage::Confirming => ClassChangeBody::Columns(Box::new(self.columns_view(ctx, true))),
            Stage::Result(pages) => {
                let faces = (&ctx.game.content.portraits, &ctx.game.lead);
                ClassChangeBody::Result(progress::view(ctx.words(), faces, pages).map(Box::new))
            }
            Stage::Done => ClassChangeBody::Done,
        };
        ClassChangeView {
            body,
            help: self.help(ctx),
        }
    }

    /// The title, the unit's line, the seals in stock for the focused
    /// class, the columns on show and the marks for those scrolled away;
    /// with the question if `asking`.
    fn columns_view(&self, ctx: &Ctx, asking: bool) -> ColumnsView {
        let words = ctx.words();
        let classes = &ctx.game.content.classes;
        let class = words.class_of(&self.unit.class, classes);
        let focused = self.focused().and_then(|o| classes.get(&o.class));
        let seal = focused.map(|class| {
            let (name, held) = self.seal_of(ctx, class);
            SealView { name, held }
        });
        let question = focused.filter(|_| asking).map(|class| {
            let [ask, cost] = self.question(ctx, class);
            let keys = ctx.text_with("class_change.yes_no", &[]);
            QuestionView { ask, cost, keys }
        });
        let first = self.cursor.first_shown();
        let shown = self.options.iter().enumerate().skip(first);
        // Every option is a class of the content: the targets come from it.
        let columns = shown
            .take(COLUMNS)
            .filter_map(|(index, option)| {
                let class = classes.get(&option.class)?;
                Some(self.column_view(ctx, class, option, index == self.cursor.focus()))
            })
            .collect();
        let more = |key| ctx.text(key).to_owned();
        let some = !self.options.is_empty();
        ColumnsView {
            title: ctx.text(self.kind.title()).to_owned(),
            unit: words.unit(&self.unit).to_owned(),
            class_level: class_level(words, class, self.unit.level),
            seal,
            none: (!some).then(|| ctx.text("class_change.none").to_owned()),
            columns,
            more_left: (first > 0).then(|| more("class_change.more_left")),
            more_right: (first + COLUMNS < self.options.len())
                .then(|| more("class_change.more_right")),
            weapons_heading: ctx.text("class_change.weapons").to_owned(),
            active_heading: ctx.text("class_change.active").to_owned(),
            max_mark: ctx.text(MAX_MARK).to_owned(),
            question,
        }
    }

    /// The column of `class`, the class of `option`.
    fn column_view(
        &self,
        ctx: &Ctx,
        class: &ClassDef,
        option: &ClassOption,
        focused: bool,
    ) -> ColumnView {
        let content = &ctx.game.content;
        let words = ctx.words();
        let record = match (self.kind, self.unit.class_records.get(&class.id)) {
            (ChangeKind::Promote, _) => None,
            (ChangeKind::Reclass, Some(r)) => {
                Some(ctx.text_with("class_change.class_level", &[("level", &r.class_level)]))
            }
            (ChangeKind::Reclass, None) => Some(ctx.text("class_change.new").to_owned()),
        };
        let movement = content
            .terrain
            .rules
            .movement_types
            .get(usize::from(class.movement_type.0))
            .map_or("", String::as_str);
        let active = class.active.as_ref().map(|id| {
            let skill = content.skills.get(id);
            skill.map_or(id.0.as_str(), |s| words.skill(s)).to_owned()
        });
        // A refused class changes nothing: its rows show the stats as
        // they are.
        let after = option.after.as_ref().unwrap_or(&self.unit);
        let stats = StatKind::GROWABLE.into_iter().map(|kind| {
            let (before, then) = (self.unit.stats.get(kind), after.stats.get(kind));
            let ceiling = content.classes.hard_ceilings.get(kind);
            stat_row(words, kind, before, then, ceiling)
        });
        let refused = option.after.as_ref().err();
        ColumnView {
            focused,
            offered: refused.is_none(),
            name: words.class(class).to_owned(),
            tier: ctx.text_with("class_change.tier", &[("tier", &class.tier)]),
            record,
            mov: mov_line(words, self.unit.stats.mov, class.move_points),
            movement: movement.to_owned(),
            tags: tag_names(words, class.tags)
                .into_iter()
                .map(str::to_owned)
                .collect(),
            weapons: weapon_lines(words, &self.unit, class),
            active,
            stats: stats.collect(),
            refused: refused.map(|error| self.reason(ctx, class, error)),
        }
    }
}

impl Screen<Shuyi> for ClassChangeScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        for &action in &input.actions {
            match &mut self.stage {
                Stage::Choosing => {
                    if self.step_choosing(ctx, action) {
                        return Transition::Pop;
                    }
                }
                Stage::Confirming => match action {
                    Action::Confirm => {
                        ctx.audio.menu(MenuSound::Select);
                        self.apply(ctx);
                    }
                    Action::Cancel => {
                        ctx.audio.menu(MenuSound::Cancel);
                        self.stage = Stage::Choosing;
                    }
                    _ => {}
                },
                // Confirm or Cancel finishes or closes a page, as in battle.
                Stage::Result(pages) => {
                    if matches!(action, Action::Confirm | Action::Cancel) {
                        pages.confirm();
                    }
                }
                Stage::Done => {}
            }
        }
        if let Stage::Result(pages) = &mut self.stage {
            pages.tick(input.dt, input.is_held(Action::Confirm));
            if pages.done() {
                self.stage = Stage::Done;
            }
        }
        if self.stage == Stage::Done {
            Transition::Pop
        } else {
            Transition::None
        }
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
