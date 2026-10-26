//! The rewind screen (ticket 0307, `docs/design/death-and-difficulty.md`):
//! pushed over the battle, it lists the battle's past actions, newest
//! first, in the side panel. The map shows the battle as it was just
//! before the highlighted action; confirming (after a "Use 1 of N
//! charges?" prompt) pops with the point picked, and the battle goes back
//! there for one charge ([`RewindEffect`]). A lesson about rewinding stays
//! over the map. [`RewindScreen::view`] says what it shows; the battle
//! skin paints it (ADR-0054).

use crate::prelude::*;
use core::fmt::Display;

use trpg_content::MapLook;
use trpg_core::{
    BattleHistory, BattleState, Command, Event, Phase, Pos, Replayed, Side, UnitAction, UnitId,
};

use super::camera::Camera;
use super::layout::map_area;
use super::view::{RewindScreenView, RewindView};
use super::{glyph, playdate};
use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::map_view::{MapScene, UnitView};
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::screens::Skins;
use crate::tips::LessonView;
use crate::widgets::ListCursor;
use crate::words::Words;

/// The rewind screen's skins (ADR-0083): the PC's list over the side
/// panel, the Playdate's window over the map (ticket 1039).
pub const SKINS: Skins<RewindScreenView> = Skins {
    glyph: glyph::rewind::paint,
    playdate: playdate::rewind::paint,
};

/// One past action.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// What happened, e.g. `Turn 2 · Ana attacked Brigand (hit, 7 dmg)`.
    pub line: String,
    /// The point a rewind to it goes back to (the command's index).
    pub point: usize,
    /// The battle just before it.
    pub before: BattleState,
}

/// What an input does to the battle: the screen pops with any effect but
/// [`None`](Self::None), which the battle carries out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewindEffect {
    /// Nothing more.
    None,
    /// Close the screen, battle unchanged.
    Close,
    /// Rewind to this point (the player confirmed).
    Rewind(usize),
}

/// The battle's map as the player looks at it, for the rewind screen to
/// show another state on: the camera, the view it was kept for (tiles
/// across and down), the cursor's tile and the map's look.
#[derive(Debug, Clone, PartialEq)]
pub struct MapFrame {
    pub(super) camera: Camera,
    pub(super) view: (i32, i32),
    pub(super) cursor: Pos,
    pub(super) look: MapLook,
}

/// The open rewind screen.
#[derive(Debug, Clone, PartialEq)]
pub struct RewindScreen {
    entries: Vec<Entry>,
    cursor: ListCursor,
    confirming: bool,
    charges_left: u8,
    charges: u8,
    /// The battle as it is: the map shows it while nothing is listed.
    now: BattleState,
    frame: MapFrame,
    /// The lesson about rewinding on screen as it opened, if any.
    lesson: Option<LessonView>,
}

impl RewindScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "rewind";

    /// The screen for `history`, which led to `now`, focused on the newest
    /// action, its map seen through `frame`.
    pub fn new(
        history: &BattleHistory,
        now: &BattleState,
        frame: MapFrame,
        words: Words<'_>,
    ) -> Self {
        let mut entries: Vec<Entry> = history
            .replay()
            .into_iter()
            .enumerate()
            .map(|(point, r)| Entry {
                line: describe(&r, words),
                point,
                before: r.before,
            })
            .collect();
        entries.reverse();
        Self {
            cursor: ListCursor::new(entries.len()),
            entries,
            confirming: false,
            charges_left: history.charges_left(),
            charges: now.rewind_charges(),
            now: now.clone(),
            frame,
            lesson: None,
        }
    }

    /// This screen with `lesson` over its map.
    #[must_use]
    pub fn with_lesson(mut self, lesson: Option<LessonView>) -> Self {
        self.lesson = lesson;
        self
    }

    /// The past actions, newest first.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The highlighted entry, if there are any.
    pub fn focused(&self) -> Option<&Entry> {
        self.entries.get(self.cursor.focus())
    }

    /// Whether the "Use 1 of N charges?" prompt is up.
    pub fn is_confirming(&self) -> bool {
        self.confirming
    }

    /// Whether Confirm can rewind: an action to go back to and a charge.
    pub fn can_rewind(&self) -> bool {
        self.charges_left > 0 && !self.entries.is_empty()
    }

    /// Handles one action: up/down choose (not wrapping), and so do the
    /// crank's notches (ticket 1035: Rewind back is down, further back;
    /// Rewind forward is up, towards the present), Confirm asks and then
    /// rewinds, Cancel backs out of the prompt or closes, Rewind closes.
    pub fn step(&mut self, action: Action) -> RewindEffect {
        self.step_sounding(action).0
    }

    /// [`step`](Self::step), with the menu sound it makes, if any.
    fn step_sounding(&mut self, action: Action) -> (RewindEffect, Option<MenuSound>) {
        use MenuSound::{Cancel, Move, Select};
        match action {
            Action::Cancel if self.confirming => {
                self.confirming = false;
                (RewindEffect::None, Some(Cancel))
            }
            Action::Cancel | Action::Rewind => (RewindEffect::Close, Some(Cancel)),
            Action::Confirm if self.confirming => match self.focused() {
                Some(e) => (RewindEffect::Rewind(e.point), Some(Select)),
                None => (RewindEffect::None, None),
            },
            Action::Confirm if self.can_rewind() => {
                self.confirming = true;
                (RewindEffect::None, Some(Select))
            }
            Action::CursorDown | Action::RewindBack | Action::CursorUp | Action::RewindForward
                if !self.confirming =>
            {
                let back = matches!(action, Action::CursorDown | Action::RewindBack);
                let moved = self.cursor.step(back, false, |_| true);
                (RewindEffect::None, moved.then_some(Move))
            }
            _ => (RewindEffect::None, None),
        }
    }

    /// The list: title, charges, the past actions, the highlighted entry
    /// and the question while it is asked.
    pub fn list_view(&self, words: Words<'_>) -> RewindView {
        let spent = self.charges_left == 0;
        let charges = charges_text(words, self.charges_left, self.charges);
        let prompt = self.confirming.then(|| {
            let key = if self.charges_left == 1 {
                "battle.rewind.use.one"
            } else {
                "battle.rewind.use.many"
            };
            words.text_with(key, &[("left", &self.charges_left)])
        });
        RewindView {
            title: words.text("battle.rewind.title").to_owned(),
            charges,
            spent,
            entries: self.entries.iter().map(|e| e.line.clone()).collect(),
            focus: self.cursor.focus(),
            nothing: self
                .entries
                .is_empty()
                .then(|| words.text("battle.rewind.nothing").to_owned()),
            prompt,
        }
    }

    /// What the map shows (ADR-0038): the battle as it was just before the
    /// highlighted action (as it is now with nothing listed), with no
    /// flashes, ranges, cursor or path.
    pub fn scene(&self, ctx: &Ctx) -> MapScene {
        let size = ctx.game.map_skin.view_tiles(map_area(ctx));
        let shown = self.focused().map_or(&self.now, |e| &e.before);
        let frame = &self.frame;
        let camera = super::fit_camera(frame.camera, frame.view, frame.cursor, shown, size);
        let mut scene = super::terrain_scene(shown, camera.origin, size);
        scene.clock_ms = super::clock_ms(ctx);
        scene.look.clone_from(&frame.look);
        for unit in shown.units() {
            scene.push_unit(UnitView::of(unit));
        }
        super::animate(&mut scene, None);
        scene
    }

    /// The help line: what the keys do on the list or at the prompt.
    pub fn help(&self, ctx: &Ctx) -> String {
        let key = if self.confirming {
            "battle.rewind.help.confirm"
        } else if self.can_rewind() {
            "battle.rewind.help.pick"
        } else if self.entries.is_empty() {
            "battle.rewind.help.empty"
        } else {
            "battle.rewind.help.browse"
        };
        ctx.text_with(key, &[])
    }

    /// The whole screen (ADR-0054): the map, the list, the lesson, the
    /// help and the debug hint.
    pub fn view(&self, ctx: &Ctx) -> RewindScreenView {
        RewindScreenView {
            scene: self.scene(ctx),
            list: self.list_view(ctx.words()),
            lesson: self.lesson.clone(),
            help: self.help(ctx),
            debug_hint: crate::screens::debug_hint(ctx),
        }
    }
}

impl Screen<Shuyi> for RewindScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    /// Steps the list for each action, with its menu sound; pops with the
    /// first effect, for the battle to carry out.
    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        for &action in &input.actions {
            let (effect, sound) = self.step_sounding(action);
            if let Some(sound) = sound {
                ctx.audio.menu(sound);
            }
            if effect != RewindEffect::None {
                return Transition::pop_with(effect);
            }
        }
        Transition::None
    }

    /// Paints the whole screen, its map too: the battle under it isn't
    /// drawn.
    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        SKINS.paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }
}

/// The charges line: `left` of `charges` charges left, or that none are.
pub fn charges_text(words: Words<'_>, left: u8, charges: u8) -> String {
    if left == 0 {
        return words.text("battle.rewind.no_charges").to_owned();
    }
    let key = if charges == 1 {
        "battle.rewind.charges.one"
    } else {
        "battle.rewind.charges.many"
    };
    let counts = [("left", &left as &dyn Display), ("charges", &charges)];
    words.text_with(key, &counts)
}

/// A unit's name in `state`, or `?` (never shown: a command only names
/// units on the map just before it).
fn name<'a>(state: &'a BattleState, words: Words<'a>, id: UnitId) -> &'a str {
    state.unit(id).map_or("?", |u| words.unit(u))
}

/// How a combat went for the attacker: `hit, 7 dmg`, `2 hits, 14 dmg` or
/// `missed`, plus `defeated` if the defender fell.
fn combat_summary(words: Words<'_>, events: &[Event]) -> Option<String> {
    let (outcome, defender_hp) = events.iter().find_map(|e| match e {
        Event::CombatResolved { outcome, .. } => Some((outcome, outcome.defender_hp)),
        _ => None,
    })?;
    let hits: Vec<_> = outcome
        .strikes
        .iter()
        .filter(|s| s.by == Side::Attacker && s.hit && !s.healed)
        .collect();
    let dmg: i32 = hits.iter().map(|s| s.damage).sum();
    let text = match hits.len() {
        0 => words.text("battle.rewind.missed").to_owned(),
        1 => words.text_with("battle.rewind.hit", &[("dmg", &dmg)]),
        n => words.text_with("battle.rewind.hits", &[("n", &n), ("dmg", &dmg)]),
    };
    Some(if defender_hp == 0 {
        words.text_with("battle.rewind.defeated", &[("result", &text)])
    } else {
        text
    })
}

/// A readable line for a past command, e.g. `Turn 2 · Ana attacked Brigand
/// (hit, 7 dmg)`.
pub fn describe(r: &Replayed, words: Words<'_>) -> String {
    let s = &r.before;
    let name = |s, id| name(s, words, id);
    let did = |key, who: &str| words.text_with(key, &[("who", &who)]);
    let did_to = |key, who: &str, what: &str, target: &str| {
        words.text_with(key, &[("who", &who), ("what", &what), ("target", &target)])
    };
    let what = match &r.command {
        Command::Act { unit, action, .. } => {
            let who = name(s, *unit);
            // A fight's result after what started it: `… (hit, 7 dmg)`.
            let fought = |what: String| match combat_summary(words, &r.events) {
                Some(result) => words.text_with(
                    "battle.rewind.with_result",
                    &[("what", &what), ("result", &result)],
                ),
                None => what,
            };
            match action {
                UnitAction::Wait => did("battle.rewind.waited", who),
                UnitAction::Attack { target, .. } => {
                    fought(did_to("battle.rewind.attacked", who, "", name(s, *target)))
                }
                UnitAction::UseItem {
                    pack_index: index,
                    target,
                }
                | UnitAction::UseOwnItem { index, target } => {
                    let items = match action {
                        UnitAction::UseItem { .. } => Some(&s.pack().items),
                        _ => s.unit(*unit).map(|u| &u.carried),
                    };
                    let item = items
                        .and_then(|items| items.get(*index))
                        .filter(|id| s.items().get(id).is_some())
                        .map_or(words.text("battle.rewind.an_item"), |id| {
                            words.item(id, s.items())
                        });
                    if target == unit {
                        did_to("battle.rewind.used", who, item, "")
                    } else {
                        did_to("battle.rewind.used_on", who, item, name(s, *target))
                    }
                }
                UnitAction::Seize => did("battle.rewind.seized", who),
                UnitAction::Escape => did("battle.rewind.escaped", who),
                UnitAction::Shop { .. } => did("battle.rewind.shopped", who),
                UnitAction::Open => did("battle.rewind.opened", who),
                UnitAction::Evacuate => did("battle.rewind.evacuated", who),
                UnitAction::Cast { spell, target, .. } => {
                    let spell = s
                        .spells()
                        .get(spell)
                        .map_or(words.text("battle.rewind.a_spell"), |d| words.spell(d));
                    match target {
                        trpg_core::CastTarget::Unit(t) => {
                            fought(did_to("battle.rewind.cast_on", who, spell, name(s, *t)))
                        }
                        trpg_core::CastTarget::Tile(_) => {
                            did_to("battle.rewind.cast", who, spell, "")
                        }
                    }
                }
                UnitAction::UseSkill { skill, .. } => {
                    let skill = s
                        .skills()
                        .get(skill)
                        .map_or(words.text("battle.rewind.a_skill"), |d| words.skill(d));
                    did_to("battle.rewind.used", who, skill, "")
                }
            }
        }
        Command::Equip { unit, .. } => did("battle.rewind.equipped", name(s, *unit)),
        Command::MoveAfter { unit, to } => {
            let who = name(s, *unit);
            if to.is_some() {
                did("battle.rewind.moved_after", who)
            } else {
                did("battle.rewind.stayed_after", who)
            }
        }
        Command::Talk { unit, target, .. } => {
            did_to("battle.rewind.talked", name(s, *unit), "", name(s, *target))
        }
        Command::EndPhase => words.text(phase_ended(s.phase())).to_owned(),
    };
    words.text_with(
        "battle.rewind.line",
        &[("turn", &s.turn()), ("what", &what)],
    )
}

/// The key of the line for the end of `phase` (`Player phase ended`).
const fn phase_ended(phase: Phase) -> &'static str {
    match phase {
        Phase::Player => "battle.rewind.phase_ended.player",
        Phase::Enemy => "battle.rewind.phase_ended.enemy",
        // Ally and neutral units (FE calls it the ally phase).
        Phase::Other => "battle.rewind.phase_ended.other",
    }
}

#[cfg(test)]
mod tests;
