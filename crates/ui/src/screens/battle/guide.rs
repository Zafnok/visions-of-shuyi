//! The battle's guide (ticket 1291): her one-time [`tips`](super::tips)
//! (0406), her checklist [`lessons`](super::lessons) (0456), the box at
//! the end of the lessons (0465) and her voice (0476, 0494, 0496, 1296).
//! The battle screen tells her the [`Moment`] (what is on screen over the
//! map), fires the triggers (`triggers.rs`) and notes what
//! the player did; she decides which box shows and keeps her lines loaded.

use crate::prelude::*;

use trpg_content::{DoneWhen, LineId, TipTrigger};
use trpg_core::{BattleState, CastTarget, Command, Event, Phase, Pos};

use super::art_list;
use super::doable::Doable;
use super::forecast_of;
use super::lessons::{EndAnswer, LessonState};
use super::mode::{MenuEntry, Mode, list_entry};
use super::tips::TipState;
use crate::audio::MenuSound;
use crate::input::Action;
use crate::screen::Ctx;
use crate::tips::voice::GuideVoice;
use crate::tips::{LessonView, LessonsEndView, TipView};

/// What is over the battle's map this frame, besides the guide's boxes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Over {
    /// Nothing: the map is the player's.
    #[default]
    Nothing,
    /// A banner (a phase's, the objective's, the outcome's).
    Banner,
    /// A combat's playback or an AI action, an EXP bar or level-up page,
    /// a scene waiting to play next, or the battle's notes in their box.
    /// A banner waits behind each of them.
    Busy,
}

/// The battle as the guide's boxes wait for it this frame (1291): a box
/// shows only once the map is the player's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Moment {
    /// What is over the map.
    pub over: Over,
    /// The battle's phase.
    pub phase: Phase,
    /// Whether the battle is decided (won or lost).
    pub decided: bool,
    /// Whether the player is browsing the map ([`Mode::Idle`]) with
    /// nothing waiting in the battle's queue (notes, banners, scenes).
    pub browsing_clear: bool,
}

/// What the mode shows that the lessons watch for (0456, 0457): a unit
/// selected, the attack forecast open, an action menu or arts menu open, a
/// spell aimed at a tile, or none of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Marks {
    None,
    Selected,
    Forecast,
    ActionMenu,
    Arts,
    TileAimed,
}

impl Marks {
    pub(super) fn of(mode: &Mode) -> Self {
        match mode {
            Mode::Selected(_) => Self::Selected,
            _ if forecast_of(mode).is_some() => Self::Forecast,
            Mode::ActionMenu { .. } => Self::ActionMenu,
            Mode::List { menu, .. } if list_entry(menu) == MenuEntry::Arts => Self::Arts,
            Mode::CastTarget(t) if matches!(t.target(), CastTarget::Tile(_)) => Self::TileAimed,
            _ => Self::None,
        }
    }

    /// What it coming up shows the player did.
    fn done(self) -> Option<DoneWhen> {
        match self {
            Self::None => None,
            Self::Selected => Some(DoneWhen::UnitSelected),
            Self::Forecast => Some(DoneWhen::ForecastOpened),
            Self::ActionMenu => Some(DoneWhen::ActionMenuOpened),
            Self::Arts => Some(DoneWhen::ArtsOpened),
            Self::TileAimed => Some(DoneWhen::TileAimed),
        }
    }
}

/// What a lesson just done holds back (0465, 0495, 1292).
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct AfterLesson {
    /// Whether the box at the end of the lessons waits for a press on the
    /// map: a lesson was just done, so a key meant for it never answers
    /// the box.
    end_waits: bool,
    /// Whether a lesson was just done with another in line: it goes
    /// straight to that one, never to a tip between them, until it opens.
    next_first: bool,
}

/// What a key given to the guide's boxes did ([`Guide::take_key`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Took {
    /// No box of hers is up: the key is the battle's.
    No,
    /// Her box on screen took it.
    Yes,
    /// The box at the end of the lessons was answered with a restart.
    Restart,
}

/// The guide on one battle screen.
#[derive(Debug, Clone, Default)]
pub struct Guide {
    /// One-time tips waiting to show (0406).
    pub(super) tips: TipState,
    /// The kind of mode the queued tips were in their moment for, and
    /// whether it plays back a command: when it changes, they are dropped
    /// (1292), unless it was a playback.
    tip_mode: Option<(core::mem::Discriminant<Mode>, bool)>,
    /// What a lesson just done holds back.
    after_lesson: AfterLesson,
    /// Checklist lessons waiting or open (0456).
    pub(super) lessons: LessonState,
    /// Her voice for the box on screen (0476), and the lines kept loaded
    /// with hers: the coming scenes' (1296).
    voice: GuideVoice,
    /// The first lines of the scene this frame pushes, if it does: still
    /// kept should her set change as it goes (1296).
    pushed_lines: Vec<LineId>,
    /// The answer the box at the end of the lessons is on (0465).
    end_focus: usize,
    /// What the units left to act could do, for the lessons (0464): worked
    /// out when one asks, started anew whenever the battle changes.
    doable: Doable,
    /// What the player did since the lessons were last ticked
    /// ([`tick_lessons`](Self::tick_lessons)).
    noticed: Vec<DoneWhen>,
}

impl Guide {
    /// Notes that `trigger` happened, for the tips and the lessons.
    pub(super) fn fire(&mut self, trigger: TipTrigger) {
        self.tips.fire(trigger);
        self.lessons.fire(trigger);
    }

    /// Notes that `trigger` happened, for the lessons only.
    pub(super) fn fire_lesson(&mut self, trigger: TipTrigger) {
        self.lessons.fire(trigger);
    }

    /// Whether a lesson is in play but one that waits for the turn to end
    /// ([`LessonState::busy`]).
    pub(super) fn lessons_busy(&self) -> bool {
        self.lessons.busy()
    }

    /// Whether a lesson is in play: auto-end waits for it.
    pub(super) fn lesson_in_play(&self) -> bool {
        self.lessons.in_play()
    }

    /// Whether an open lesson holds the turn (0456).
    pub(super) fn holds_turn(&self) -> bool {
        self.lessons.holds_turn()
    }

    /// The battle changed (a command, a rewind): what the units left could
    /// do is worked out anew.
    pub(super) fn battle_changed(&mut self) {
        self.doable = Doable::default();
    }

    /// The tip on screen, if any: the first queued whose moment has come
    /// (`due_tip`), unless an open lesson is on screen, or the next lesson
    /// after one just done: tips wait for it (0456).
    pub(super) fn shown_tip(&self, m: Moment) -> Option<TipTrigger> {
        let lesson_up = self.lesson_first() && self.lesson_moment(m);
        self.due_tip(m).filter(|_| !lesson_up)
    }

    /// The first queued tip whose moment has come, if any. Tips wait out a
    /// combat's playback, a scene queued, a screen over the battle (a
    /// scene playing, the rewind screen) and the battle notes; the
    /// enemy-phase tip shows as that phase begins (over its banner), the
    /// others in a player phase without a banner. Only a tip still queued
    /// is in its moment ([`take_in_mode`](Self::take_in_mode), 1292).
    fn due_tip(&self, m: Moment) -> Option<TipTrigger> {
        if m.over == Over::Busy || self.tips.covered() {
            return None;
        }
        let clear = m.phase == Phase::Player && m.over == Over::Nothing;
        self.tips
            .queued()
            .find(|&t| t == TipTrigger::FirstEnemyPhase || clear)
    }

    /// At the frame's start: the queued tips' moment passes if the kind of
    /// `mode` changed, and those not on screen are dropped (1292: they
    /// show the next time their trigger fires). A playback (a combat, an
    /// AI action) ends the moment of none: a level-up's tip, fired as its
    /// combat starts, shows once the map is back.
    pub(super) fn take_in_mode(&mut self, mode: &Mode) {
        let playback = matches!(mode, Mode::Combat(_) | Mode::AiAction(_));
        let now = (core::mem::discriminant(mode), playback);
        if self.tip_mode != Some(now) {
            if !self.tip_mode.is_some_and(|(_, was)| was) {
                self.tips.go_stale();
            }
            self.tip_mode = Some(now);
        }
    }

    /// Queues the tips fired since the last time.
    pub(super) fn absorb_tips(&mut self, ctx: &mut Ctx) {
        self.tips.absorb(ctx);
    }

    /// The id of the line she says in the box on screen, the tip's or
    /// else the lesson's: its `text`, never its items or `how` (0476).
    fn guide_line(&self, m: Moment, ctx: &Ctx) -> Option<LineId> {
        let content = &ctx.game.content;
        let text = match self.shown_tip(m) {
            Some(trigger) => &content.tips.for_trigger(trigger)?.text,
            None => &content.lessons.get(self.shown_lesson(m)?)?.text,
        };
        Some(trpg_content::voice::guide_line(text))
    }

    /// The ids of her lines battle `battle` may still show (0494, 0496),
    /// each once: the queued tips', the waiting lessons', the tips not
    /// seen yet and the lessons that may yet start here. The order
    /// doesn't matter: every one is kept loaded.
    pub(super) fn coming_guide_lines(&self, ctx: &Ctx, battle: Option<&str>) -> Vec<LineId> {
        let content = &ctx.game.content;
        let queued = self
            .tips
            .queued()
            .filter_map(|t| content.tips.for_trigger(t));
        let waiting = self
            .lessons
            .waiting()
            .filter_map(|id| content.lessons.get(id));
        let to_start = content.lessons.lessons.iter().filter(|l| {
            LessonState::shows(ctx, l.level)
                && l.starts_in(battle)
                && !ctx.game.campaign_lessons.contains(&l.id)
        });
        let texts = queued
            .map(|t| &t.text)
            .chain(waiting.map(|l| &l.text))
            .chain(self.tips.unseen(ctx).map(|t| &t.text))
            .chain(to_start.map(|l| &l.text));
        let mut lines: Vec<LineId> = Vec::new();
        for text in texts {
            let line = trpg_content::voice::guide_line(text);
            if !lines.contains(&line) {
                lines.push(line);
            }
        }
        lines
    }

    /// The battle pushes a scene this frame whose first lines are `lines`:
    /// they stay in her set as it goes (1296).
    pub(super) fn pushing_scene(&mut self, lines: Vec<LineId>) {
        self.pushed_lines = lines;
    }

    /// Whether this frame's end builds the set of lines kept loaded: when
    /// her voice says it is due ([`GuideVoice::keep_due`]) and the battle
    /// `stays` on top, or the screen going over it is a scene it plays,
    /// whose lines must stay in the set (1296). Any other screen over it
    /// leaves the set as it was until the battle is back on top.
    pub(super) fn keep_due(&self, stays: bool, m: Moment, ctx: &Ctx) -> bool {
        let line = stays.then(|| self.guide_line(m, ctx)).flatten();
        let due = self.voice.keep_due(line.as_ref(), ctx);
        Self::set_due(stays, !self.pushed_lines.is_empty(), due)
    }

    /// Whether the set is built: `due`, and the battle `stays` on top or
    /// is `pushing_scene`.
    pub(super) fn set_due(stays: bool, pushing_scene: bool, due: bool) -> bool {
        (stays || pushing_scene) && due
    }

    /// The frame's end (0467, 0476, 1296): still on screen (`stays`), her
    /// tip on screen, if any, is seen, and she says what its box or the
    /// lesson's says. `scenes`, when [`keep_due`](Self::keep_due): the
    /// first lines of the scenes the battle may still play, kept loaded
    /// with the scene it pushes now and her own lines.
    pub(super) fn end_frame(
        &mut self,
        stays: bool,
        m: Moment,
        battle: Option<&str>,
        scenes: Option<Vec<LineId>>,
        ctx: &mut Ctx,
    ) {
        self.tips
            .show(stays.then(|| self.shown_tip(m)).flatten(), ctx);
        let line = stays.then(|| self.guide_line(m, ctx)).flatten();
        let pushed = core::mem::take(&mut self.pushed_lines);
        let coming = scenes.map(|scenes| {
            let mut lines = pushed;
            for line in scenes
                .into_iter()
                .chain(self.coming_guide_lines(ctx, battle))
            {
                if !lines.contains(&line) {
                    lines.push(line);
                }
            }
            lines
        });
        self.voice.show(line, coming, ctx);
    }

    /// A screen over the battle (a scene, the rewind screen, …): no tip or
    /// lesson shows until it goes, and her set is sent again once back.
    pub(super) fn covered(&mut self) {
        self.tips.set_covered(true);
        self.voice.covered();
    }

    /// On top again.
    pub(super) fn uncovered(&mut self) {
        self.tips.set_covered(false);
    }

    /// The battle leaves the stack (0496): her lines are let go.
    pub(super) fn leave(&mut self, ctx: &mut Ctx) {
        self.voice.leave(ctx);
    }

    /// The box of the tip on screen ([`Self::shown_tip`]), if any.
    pub(super) fn tip_view(&self, m: Moment, ctx: &Ctx) -> Option<TipView> {
        if !crate::tips::shown_in(ctx) {
            return None;
        }
        let tip = ctx.game.content.tips.for_trigger(self.shown_tip(m)?)?;
        Some(TipView::new(ctx, tip))
    }

    /// Whether a lesson may be on screen now: the player's phase of a
    /// battle still on, with no combat or AI action playing, no EXP or
    /// level up, scene, banner or battle notes over the map, and no screen
    /// over the battle. Lessons wait as tips do; the rewind screen shows
    /// one that needs it ([`rewind_lesson`](Self::rewind_lesson)).
    fn lesson_moment(&self, m: Moment) -> bool {
        m.over == Over::Nothing && !self.tips.covered() && m.phase == Phase::Player && !m.decided
    }

    /// The id of the lesson on screen, if any: the one in play, once its
    /// moment has come. One not yet open also waits for a tip on screen,
    /// so it comes after the tip of the same moment.
    pub(super) fn shown_lesson(&self, m: Moment) -> Option<&str> {
        let id = self.lessons.current()?;
        let clear = self.lesson_first() || self.due_tip(m).is_none();
        (self.lesson_moment(m) && clear).then_some(id)
    }

    /// Whether the lesson in play goes before any tip: it is open, or it
    /// comes straight after one just done.
    fn lesson_first(&self) -> bool {
        self.lessons.is_open() || (self.after_lesson.next_first && self.lessons.current().is_some())
    }

    /// The box of the lesson on screen ([`Self::shown_lesson`]), if any,
    /// which never hides the tile `clear_of` (the cursor's).
    pub(super) fn lesson_view(&self, m: Moment, ctx: &Ctx, clear_of: Pos) -> Option<LessonView> {
        let lesson = ctx.game.content.lessons.get(self.shown_lesson(m)?)?;
        if !LessonState::shows(ctx, lesson.level) {
            return None;
        }
        let ticked = self.lessons.ticked();
        Some(LessonView::new(ctx, lesson, ticked, clear_of))
    }

    /// The box of the lesson on screen for the rewind screen, if the
    /// lesson is about rewinding: it stays over the map there.
    pub(super) fn rewind_lesson(&self, m: Moment, ctx: &Ctx, clear_of: Pos) -> Option<LessonView> {
        self.lessons
            .needs_rewind()
            .then(|| self.lesson_view(m, ctx, clear_of))
            .flatten()
    }

    /// Whether the box at the end of the lessons is on screen (0465): they
    /// are all done, and the map is back to browsing with nothing else over
    /// it, so it never cuts into what the last lesson had the player doing.
    pub(super) fn lessons_end_shown(&self, m: Moment, ctx: &Ctx, battle: Option<&str>) -> bool {
        m.browsing_clear
            && !self.after_lesson.end_waits
            && self.lesson_moment(m)
            && self.due_tip(m).is_none()
            && LessonState::end_due(ctx, battle)
    }

    /// The box at the end of the lessons, if it is on screen.
    pub(super) fn lessons_end_view(
        &self,
        m: Moment,
        ctx: &Ctx,
        battle: Option<&str>,
        clear_of: Pos,
    ) -> Option<LessonsEndView> {
        self.lessons_end_shown(m, ctx, battle)
            .then(|| LessonsEndView::new(ctx, self.end_focus, clear_of))
    }

    /// Gives `action` to her box on screen, if any: a tip takes the keys
    /// until it is closed (by Confirm or Cancel, once it has been drawn,
    /// 0467), and the box at the end of the lessons every key while it is
    /// up.
    pub(super) fn take_key(
        &mut self,
        m: Moment,
        ctx: &mut Ctx,
        action: Action,
        battle: Option<&str>,
    ) -> Took {
        if let Some(tip) = self.shown_tip(m) {
            if matches!(action, Action::Confirm | Action::Cancel) && self.tips.on_screen(tip) {
                self.tips.dismiss(tip);
            }
            return Took::Yes;
        }
        if self.lessons_end_shown(m, ctx, battle) {
            return self.lessons_end_key(ctx, action);
        }
        Took::No
    }

    /// Gives `action` to the box at the end of the lessons. Cancel does
    /// nothing: the box asks for an answer.
    fn lessons_end_key(&mut self, ctx: &mut Ctx, action: Action) -> Took {
        let last = EndAnswer::ALL.len() - 1;
        match action {
            Action::CursorUp | Action::CursorDown => {
                self.end_focus = if action == Action::CursorUp {
                    self.end_focus.checked_sub(1).unwrap_or(last)
                } else if self.end_focus == last {
                    0
                } else {
                    self.end_focus + 1
                };
                ctx.audio.menu(MenuSound::Move);
            }
            Action::Confirm => {
                ctx.audio.menu(MenuSound::Select);
                LessonState::end_answered(ctx);
                let answer = EndAnswer::ALL[self.end_focus];
                if answer == EndAnswer::RestartWithLessons {
                    self.lessons.show_again(ctx);
                }
                if answer != EndAnswer::KeepGoing {
                    return Took::Restart;
                }
            }
            _ => {}
        }
        Took::Yes
    }

    /// A press on the map: the box at the end of the lessons may come up
    /// (0495).
    pub(super) fn pressed_on_map(&mut self) {
        self.after_lesson.end_waits = false;
    }

    /// Queues the lessons the fired triggers start in battle `battle`
    /// (`state` as it stands, `mode` the player's), then
    /// [settles](Self::settle_lessons) them.
    pub(super) fn absorb_lessons(
        &mut self,
        ctx: &Ctx,
        battle: Option<&str>,
        state: &BattleState,
        mode: &Mode,
    ) {
        self.lessons.absorb(ctx, battle, state.turn());
        self.settle_lessons(state, mode, ctx.game.map_skin.zoom().is_some());
    }

    /// In the player's phase of a battle still on, takes out of play the
    /// lesson in front while no unit left to act can do an item it has
    /// left ([`LessonState::settle`], 0464): one open comes back in a
    /// later turn, one not open yet with its trigger. Zooming takes a
    /// look that can zoom (`can_zoom`); ending the turn is taught on turn 1
    /// only (1304, Nick: the player "already knows" after it). Swapping
    /// weapons opens only in a forecast that swaps, so what she says is
    /// about the unit on screen (1306); once open it stays while a unit
    /// left could swap.
    pub(super) fn settle_lessons(&mut self, state: &BattleState, mode: &Mode, can_zoom: bool) {
        if state.phase() != Phase::Player || state.outcome().is_some() {
            return;
        }
        let doable = &mut self.doable;
        let swap_now = matches!(mode, Mode::Targeting(t) if t.can_swap());
        self.lessons
            .settle_when(state.turn(), |done, open| match done {
                DoneWhen::ZoomedIn | DoneWhen::ZoomedOut => can_zoom,
                DoneWhen::TurnEnded => state.turn() == 1,
                DoneWhen::WeaponSwapped if !open => swap_now,
                _ => doable.can(state, done),
            });
    }

    /// As the frame ends: the lesson on screen, if it can still be done,
    /// is open from now on.
    pub(super) fn mark_shown(&mut self, m: Moment) {
        if self.shown_lesson(m).is_some() {
            self.lessons.mark_open();
            self.after_lesson.next_first = false;
        }
    }

    /// Notes that the player did `done`, for the lesson in play
    /// ([`tick_lessons`](Self::tick_lessons) ticks it).
    pub(super) fn notice(&mut self, done: DoneWhen) {
        self.noticed.push(done);
    }

    /// Notes that the battle used `action`, unless in the AI's phase.
    pub(super) fn notice_pressed(&mut self, action: Action, ai_phase: bool) {
        if !ai_phase {
            self.notice(DoneWhen::Pressed(action));
        }
    }

    /// Notes what the player did that changed the mode from what `was`
    /// marks to `mode`: selected a unit, picked where it goes, opened the
    /// forecast.
    pub(super) fn notice_mode(&mut self, was: Marks, mode: &Mode) {
        let now = Marks::of(mode);
        let gone = matches!(
            mode,
            Mode::Moving { .. }
                | Mode::ActionMenu { .. }
                | Mode::Targeting(_)
                | Mode::CastTarget(_)
        );
        if was == Marks::Selected && gone {
            self.notice(DoneWhen::UnitMoved);
        }
        // Attacked from or backed out of: either counts (1304).
        if was == Marks::Forecast && now != Marks::Forecast {
            self.notice(DoneWhen::ForecastClosed);
        }
        if let Some(done) = now.done().filter(|_| now != was) {
            self.notice(done);
        }
    }

    /// Notes where a selected unit's path points, for the threat arrows'
    /// lesson (0457): `threat` is whether the arrows show at its tile
    /// (`None` while the path points at no tile they are worked out for).
    pub(super) fn notice_threat(&mut self, threat: Option<bool>) {
        match threat {
            Some(true) => self.notice(DoneWhen::ThreatPointed),
            Some(false) => self.notice(DoneWhen::SafePointed),
            None => {}
        }
    }

    /// Notes what the player did that player command `cmd` with `events`
    /// shows: a unit of theirs attacked (in the player's phase every combat
    /// a command fights is the player's attack), moved after its action,
    /// or they ended their phase.
    pub(super) fn notice_events(&mut self, cmd: &Command, events: &[Event]) {
        match cmd {
            Command::EndPhase => self.notice(DoneWhen::TurnEnded),
            Command::MoveAfter { .. } => self.notice(DoneWhen::UnitMoved),
            _ => {}
        }
        if events
            .iter()
            .any(|e| matches!(e, Event::CombatResolved { .. }))
        {
            self.notice(DoneWhen::Attacked);
            if art_list::attacks_with_art(cmd) {
                self.notice(DoneWhen::ArtAttacked);
            }
        }
    }

    /// Ticks the lesson in play's items for what was noticed since last
    /// time; one that is done closes, and the campaign keeps it.
    /// A lesson done makes the box at the end of the lessons wait for a
    /// press on the map (0495), so a key meant for the lesson never answers
    /// it.
    pub(super) fn tick_lessons(&mut self, ctx: &mut Ctx) {
        let was_open = self.lessons.is_open();
        for done in core::mem::take(&mut self.noticed) {
            self.lessons.tick(ctx, done);
        }
        if was_open && !self.lessons.is_open() {
            self.after_lesson.end_waits = true;
            self.after_lesson.next_first = self.lessons.current().is_some();
        }
    }
}
