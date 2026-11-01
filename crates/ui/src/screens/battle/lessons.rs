//! The battle's checklist lessons (0456, `docs/design/tutorial.md`, *The
//! lessons*): the lessons waiting, the one in play and its ticks. One
//! lesson at a time: a lesson its trigger starts waits behind the one in
//! play, and opens once it is done and the moment is right, if a unit left
//! can still do what it asks ([`LessonState::settle`], 0464).
//!
//! The battle screen decides when a lesson shows and when an item's thing
//! was done ([`DoneWhen`]); this keeps the list. Ticks live here, not in
//! the battle's state, so a rewind never un-ticks an item, and a finished
//! lesson is kept with the campaign
//! ([`Shuyi::campaign_lessons`](crate::Shuyi::campaign_lessons)), so it
//! never opens again.

use crate::prelude::*;
use alloc::collections::VecDeque;

use trpg_content::{DoneWhen, Lesson, LessonLevel, TipTrigger};
use trpg_core::TutorialLevel;

use crate::screen::{Ctx, Lessons};

/// What the campaign keeps with its lessons once the player has answered
/// the box at the end of them (0465): never a lesson's id. It stays through
/// "Restart with the lessons", so the box shows once per campaign.
const END_ANSWERED: &str = "lessons_end_answered";

/// An answer to the box at the end of the lessons (0465), in the box's order:
/// the cursor starts on the first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndAnswer {
    /// The battle goes on.
    KeepGoing,
    /// Back to turn 1; the lessons stay done.
    Restart,
    /// Back to turn 1; this battle's lessons show again.
    RestartWithLessons,
}

impl EndAnswer {
    /// Every answer, in the box's order.
    pub const ALL: [Self; 3] = [Self::KeepGoing, Self::Restart, Self::RestartWithLessons];

    /// Text key of its label.
    pub fn text_key(self) -> &'static str {
        match self {
            Self::KeepGoing => "battle.lessons_end.keep_going",
            Self::Restart => "battle.lessons_end.restart",
            Self::RestartWithLessons => "battle.lessons_end.restart_with_lessons",
        }
    }
}

/// A lesson waiting or in play.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Waiting {
    /// Its id.
    id: String,
    /// The tutorial levels it shows at.
    level: LessonLevel,
    /// What ticks each item, in order.
    items: Vec<DoneWhen>,
    /// Each item ticked yet, in order.
    ticked: Vec<bool>,
}

impl Waiting {
    fn new(lesson: &Lesson) -> Self {
        Self {
            id: lesson.id.clone(),
            level: lesson.level,
            items: lesson.items.iter().map(|i| i.done_when).collect(),
            ticked: vec![false; lesson.items.len()],
        }
    }
}

/// The lessons of one battle screen.
#[derive(Debug, Clone, Default)]
pub struct LessonState {
    /// Triggers that fired since the last [`absorb`](Self::absorb).
    fired: Vec<TipTrigger>,
    /// Lessons waiting, the one in play in front.
    queue: VecDeque<Waiting>,
    /// Whether the front lesson has been on screen: from then on it is
    /// open, its items tick, and the turn waits for it.
    open: bool,
    /// The lessons put away unfinished ([`put_away`](Self::put_away)) and
    /// the turn they were put away in: they don't come back that turn.
    away: Vec<(String, u32)>,
    /// The lessons finished on this screen, for "Restart with the lessons"
    /// ([`show_again`](Self::show_again), 0465).
    finished: Vec<String>,
}

impl LessonState {
    /// Notes that `trigger` happened.
    pub fn fire(&mut self, trigger: TipTrigger) {
        if !self.fired.contains(&trigger) {
            self.fired.push(trigger);
        }
    }

    /// Whether a lesson of `level` shows in `ctx`: lessons are on and the
    /// campaign's tutorial level shows it (a battle with no campaign
    /// counts as [`TutorialLevel::Full`]).
    pub fn shows(ctx: &Ctx, level: LessonLevel) -> bool {
        ctx.game.lessons == Lessons::On
            && level.shows_at(ctx.game.campaign_tutorial.unwrap_or(TutorialLevel::Full))
    }

    /// Whether the campaign has finished every lesson its tutorial level
    /// shows in battle `battle` (at least one), and hasn't answered the box
    /// at the end of them yet (0465). A battle with no campaign has no such
    /// box.
    pub fn end_due(ctx: &Ctx, battle: Option<&str>) -> bool {
        let done = &ctx.game.campaign_lessons;
        if ctx.game.campaign_tutorial.is_none() || done.contains(END_ANSWERED) {
            return false;
        }
        let mut shown = ctx
            .game
            .content
            .lessons
            .lessons
            .iter()
            .filter(|l| Self::shows(ctx, l.level) && l.starts_in(battle))
            .peekable();
        shown.peek().is_some() && shown.all(|l| done.contains(&l.id))
    }

    /// The campaign keeps that the box at the end of the lessons was answered.
    pub fn end_answered(ctx: &mut Ctx) {
        ctx.game.campaign_lessons.insert(END_ANSWERED.to_owned());
    }

    /// The lessons finished on this screen aren't done any more: they show
    /// again when the battle starts again.
    pub fn show_again(&self, ctx: &mut Ctx) {
        for id in &self.finished {
            ctx.game.campaign_lessons.remove(id);
        }
    }

    /// Queues the lessons the fired triggers start in battle `battle`:
    /// each one that [`shows`](Self::shows) and isn't done or waiting
    /// already, nor put away in turn `turn`; the others are forgotten.
    /// First drops the lessons waiting that no longer show, the one in
    /// play too, unticked and not done (the tutorial level may change
    /// mid-battle in Options, 0841).
    pub fn absorb(&mut self, ctx: &Ctx, battle: Option<&str>, turn: u32) {
        let fired = core::mem::take(&mut self.fired);
        let front = self.current().map(str::to_owned);
        self.queue.retain(|w| Self::shows(ctx, w.level));
        if self.current() != front.as_deref() {
            self.open = false;
        }
        if ctx.game.lessons == Lessons::Off {
            return;
        }
        self.away.retain(|(_, t)| *t == turn);
        for trigger in fired {
            for lesson in ctx.game.content.lessons.for_trigger(trigger) {
                let due = Self::shows(ctx, lesson.level)
                    && lesson.starts_in(battle)
                    && !ctx.game.campaign_lessons.contains(&lesson.id)
                    && !self.queue.iter().any(|w| w.id == lesson.id)
                    && !self.away.iter().any(|(id, _)| *id == lesson.id);
                if due {
                    self.queue.push_back(Waiting::new(lesson));
                }
            }
        }
    }

    /// The id of the lesson in play (open, or next to open), if any.
    pub fn current(&self) -> Option<&str> {
        self.queue.front().map(|w| w.id.as_str())
    }

    /// Whether the lesson in play has been on screen: its items tick, and
    /// the turn waits for it.
    pub fn is_open(&self) -> bool {
        self.open && !self.queue.is_empty()
    }

    /// Notes that the lesson in play is on screen: it is open.
    pub fn mark_open(&mut self) {
        self.open = !self.queue.is_empty();
    }

    /// Whether the open lesson keeps the turn from ending: it is open and
    /// none of its items is ending the turn.
    pub fn holds_turn(&self) -> bool {
        self.is_open() && !self.has(DoneWhen::TurnEnded)
    }

    /// Whether one of the lesson in play's items is done by `done`.
    pub fn has(&self, done: DoneWhen) -> bool {
        self.queue.front().is_some_and(|w| w.items.contains(&done))
    }

    /// Whether the lesson in play needs the rewind screen (an item is
    /// done by opening it or pressing Rewind): it stays on screen there.
    pub fn needs_rewind(&self) -> bool {
        self.has(DoneWhen::RewindOpened)
            || self.has(DoneWhen::Pressed(trpg_content::Action::Rewind))
    }

    /// Whether a lesson is in play, open or waiting for its moment.
    pub fn in_play(&self) -> bool {
        !self.queue.is_empty()
    }

    /// Takes the lesson in front out of play, unfinished, while it has an
    /// item left that no unit left to act in turn `turn` can do (`can`
    /// says what can be done, [`Doable`](super::doable::Doable), 0464), so
    /// a lesson never holds a turn it can't be finished in. One open is put
    /// away: it comes back from an empty list the next time its trigger
    /// fires in a later turn. One not open yet doesn't open: it waits for
    /// its trigger to fire again, this turn or a later one. Returns whether
    /// one was taken out.
    pub fn settle(&mut self, turn: u32, mut can: impl FnMut(DoneWhen) -> bool) -> bool {
        let mut any = false;
        while let Some(front) = self.queue.front() {
            let stuck = front
                .items
                .iter()
                .zip(&front.ticked)
                .any(|(&d, &t)| !t && !can(d));
            if !stuck {
                break;
            }
            if self.open {
                self.away.push((front.id.clone(), turn));
            }
            self.queue.pop_front();
            self.open = false;
            any = true;
        }
        any
    }

    /// Each item of the lesson in play, ticked yet, in order.
    pub fn ticked(&self) -> &[bool] {
        self.queue.front().map_or(&[], |w| &w.ticked)
    }

    /// Ticks the open lesson's item done by `done`, if it has one. When
    /// that was its last, the lesson closes: the campaign keeps it as done
    /// and the next waiting comes into play. Returns whether an item was
    /// ticked.
    pub fn tick(&mut self, ctx: &mut Ctx, done: DoneWhen) -> bool {
        if !self.is_open() {
            return false;
        }
        let Some(front) = self.queue.front_mut() else {
            return false;
        };
        let Some(i) = front.items.iter().position(|&d| d == done) else {
            return false;
        };
        if core::mem::replace(&mut front.ticked[i], true) {
            return false;
        }
        if front.ticked.iter().all(|&t| t) {
            ctx.game.campaign_lessons.insert(front.id.clone());
            self.finished.push(front.id.clone());
            self.queue.pop_front();
            self.open = false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use trpg_content::{Action, LessonItem, LessonLevel};

    use super::*;
    use crate::screen::tests::ctx;

    const UP: DoneWhen = DoneWhen::Pressed(Action::CursorUp);

    /// A lesson `id` of `level` that `trigger` starts, done by `items`.
    fn lesson(id: &str, level: LessonLevel, trigger: TipTrigger, items: &[DoneWhen]) -> Lesson {
        Lesson {
            id: id.into(),
            level,
            trigger,
            battles: vec![],
            title: "T".into(),
            text: "x".into(),
            items: items
                .iter()
                .map(|&done_when| LessonItem {
                    label: "x".into(),
                    done_when,
                })
                .collect(),
            face: None,
        }
    }

    /// A context with lessons on and `lessons` as the only ones.
    fn with(lessons: Vec<Lesson>) -> Ctx {
        let mut c = ctx();
        c.game.lessons = Lessons::On;
        c.game.content.lessons.lessons = lessons;
        c
    }

    fn start(c: &Ctx) -> LessonState {
        let mut s = LessonState::default();
        s.fire(TipTrigger::FirstBattleStart);
        s.absorb(c, None, 1);
        s
    }

    #[test]
    fn a_trigger_queues_its_lessons_once_in_file_order() {
        let first = TipTrigger::FirstBattleStart;
        let c = with(vec![
            lesson("a", LessonLevel::Full, first, &[UP]),
            lesson("b", LessonLevel::Short, first, &[UP]),
            lesson("c", LessonLevel::Full, TipTrigger::FirstLowHp, &[UP]),
        ]);
        let mut s = start(&c);
        assert_eq!(s.current(), Some("a"));
        // Fired again (every frame of turn 1): nothing more.
        s.fire(first);
        s.fire(first);
        s.absorb(&c, None, 1);
        let ids: Vec<&str> = s.queue.iter().map(|w| w.id.as_str()).collect();
        assert_eq!(ids, ["a", "b"]);
        // Not open until shown: nothing ticks, the turn may end.
        assert!(!s.is_open() && !s.holds_turn());
    }

    #[test]
    fn levels_battles_and_the_switch_gate_the_queue() {
        let first = TipTrigger::FirstBattleStart;
        let full = lesson("full", LessonLevel::Full, first, &[UP]);
        let short = lesson("short", LessonLevel::Short, first, &[UP]);
        let mut only_test = lesson("test", LessonLevel::Full, first, &[UP]);
        only_test.battles = vec!["test".into()];
        let mut c = with(vec![full, short, only_test]);
        let queued = |c: &Ctx, battle| {
            let mut s = LessonState::default();
            s.fire(first);
            s.absorb(c, battle, 1);
            s.queue.into_iter().map(|w| w.id).collect::<Vec<_>>()
        };
        // No campaign: Full.
        assert_eq!(queued(&c, None), ["full", "short"]);
        assert_eq!(queued(&c, Some("test")), ["full", "short", "test"]);
        c.game.campaign_tutorial = Some(TutorialLevel::Full);
        assert_eq!(queued(&c, Some("ch01")), ["full", "short"]);
        c.game.campaign_tutorial = Some(TutorialLevel::Short);
        assert_eq!(queued(&c, Some("test")), ["short"]);
        c.game.campaign_tutorial = Some(TutorialLevel::None);
        assert_eq!(queued(&c, Some("test")), [] as [std::string::String; 0]);
        // Done in this campaign: never again.
        c.game.campaign_tutorial = Some(TutorialLevel::Full);
        c.game.campaign_lessons.insert("full".into());
        assert_eq!(queued(&c, None), ["short"]);
        c.game.lessons = Lessons::Off;
        assert_eq!(queued(&c, None), [] as [std::string::String; 0]);
    }

    #[test]
    fn a_lower_level_drops_the_lessons_it_does_not_show_open_or_waiting() {
        let first = TipTrigger::FirstBattleStart;
        let mut c = with(vec![
            lesson("a", LessonLevel::Full, first, &[UP, DoneWhen::Attacked]),
            lesson("b", LessonLevel::Short, first, &[UP]),
            lesson("c", LessonLevel::Full, first, &[UP]),
        ]);
        c.game.campaign_tutorial = Some(TutorialLevel::Full);
        let mut s = start(&c);
        s.mark_open();
        assert!(s.tick(&mut c, UP));
        assert!(s.holds_turn());
        // Short: the open Full lesson closes unticked and not done.
        c.game.campaign_tutorial = Some(TutorialLevel::Short);
        s.absorb(&c, None, 1);
        assert_eq!(s.current(), Some("b"));
        assert!(!s.is_open() && !s.holds_turn());
        assert_eq!(s.ticked(), [false]);
        assert!(c.game.campaign_lessons.is_empty());
        // A lesson still shown stays open.
        s.mark_open();
        s.absorb(&c, None, 1);
        assert!(s.is_open());
        // None: nothing left, nothing holds the turn.
        c.game.campaign_tutorial = Some(TutorialLevel::None);
        s.absorb(&c, None, 1);
        assert!(!s.in_play() && !s.holds_turn());
        // Back up to Full: it comes back with its trigger.
        c.game.campaign_tutorial = Some(TutorialLevel::Full);
        s.fire(first);
        s.absorb(&c, None, 1);
        assert_eq!(s.current(), Some("a"));
        assert_eq!(s.ticked(), [false, false]);
    }

    #[test]
    fn ticking_closes_the_lesson_and_the_next_comes_into_play() {
        let first = TipTrigger::FirstBattleStart;
        let down = DoneWhen::Pressed(Action::CursorDown);
        let mut c = with(vec![
            lesson("a", LessonLevel::Full, first, &[UP, down]),
            lesson("b", LessonLevel::Full, first, &[DoneWhen::TurnEnded]),
        ]);
        let mut s = start(&c);
        // Not open yet: nothing ticks.
        assert!(!s.tick(&mut c, UP));
        s.mark_open();
        assert!(s.is_open() && s.holds_turn());
        assert!(s.tick(&mut c, UP));
        // Again, or something it doesn't ask for: nothing.
        assert!(!s.tick(&mut c, UP));
        assert!(!s.tick(&mut c, DoneWhen::Attacked));
        assert_eq!(s.ticked(), [true, false]);
        assert!(s.tick(&mut c, down));
        // Done: kept with the campaign, and the next waits to be shown.
        assert!(c.game.campaign_lessons.contains("a"));
        assert_eq!(s.current(), Some("b"));
        assert!(!s.is_open() && s.ticked() == [false]);
        // A lesson with a TurnEnded item lets the turn end.
        s.mark_open();
        assert!(s.is_open() && !s.holds_turn());
        assert!(s.tick(&mut c, DoneWhen::TurnEnded));
        assert_eq!(s.current(), None);
        assert!(!s.is_open() && !s.holds_turn());
        s.mark_open();
        assert!(!s.is_open());
    }

    /// Ticket 0465: the end of the lessons is due when every lesson the
    /// level shows is done, in a campaign, until it is answered; a level
    /// with none to show (None), or lessons off, never has it. Going up a
    /// level holds it back until the new lessons are done too. A lesson
    /// only for another battle doesn't count.
    #[test]
    fn the_end_is_due_once_every_lesson_shown_is_done_until_it_is_answered() {
        let first = TipTrigger::FirstBattleStart;
        let mut elsewhere = lesson("elsewhere", LessonLevel::Short, first, &[UP]);
        elsewhere.battles = vec!["other".into()];
        let mut c = with(vec![
            lesson("basic", LessonLevel::Full, first, &[UP]),
            lesson("short", LessonLevel::Short, first, &[UP]),
            elsewhere,
        ]);
        let due = |c: &Ctx| LessonState::end_due(c, Some("here"));
        // No campaign: never.
        c.game.campaign_lessons.insert("basic".into());
        c.game.campaign_lessons.insert("short".into());
        assert!(!due(&c));
        c.game.campaign_lessons.clear();
        c.game.campaign_tutorial = Some(TutorialLevel::Short);
        assert!(!due(&c), "nothing done yet");
        c.game.campaign_lessons.insert("short".into());
        assert!(due(&c));
        assert!(
            !LessonState::end_due(&c, Some("other")),
            "one is left there"
        );
        c.game.lessons = Lessons::Off;
        assert!(!due(&c));
        c.game.lessons = Lessons::On;
        c.game.campaign_tutorial = Some(TutorialLevel::Full);
        assert!(!due(&c), "a basic lesson is left");
        c.game.campaign_lessons.insert("basic".into());
        assert!(due(&c));
        c.game.campaign_tutorial = Some(TutorialLevel::None);
        assert!(!due(&c), "None shows nothing");
        c.game.campaign_tutorial = Some(TutorialLevel::Full);
        LessonState::end_answered(&mut c);
        assert!(!due(&c), "once per campaign");
    }

    /// Ticket 0465: "Restart with the lessons" takes back what this screen
    /// finished, not what an earlier battle did, and not the answer.
    #[test]
    fn showing_again_takes_back_only_the_lessons_of_this_screen() {
        let first = TipTrigger::FirstBattleStart;
        let mut c = with(vec![
            lesson("before", LessonLevel::Full, TipTrigger::FirstLowHp, &[UP]),
            lesson("here", LessonLevel::Full, first, &[UP]),
        ]);
        c.game.campaign_tutorial = Some(TutorialLevel::Full);
        c.game.campaign_lessons.insert("before".into());
        let mut s = start(&c);
        s.mark_open();
        assert!(s.tick(&mut c, UP));
        LessonState::end_answered(&mut c);
        s.show_again(&mut c);
        assert!(c.game.campaign_lessons.contains("before"));
        assert!(!c.game.campaign_lessons.contains("here"));
        assert!(c.game.campaign_lessons.contains(END_ANSWERED));
        // They come back with their trigger; the box doesn't.
        let s = start(&c);
        assert_eq!(s.current(), Some("here"));
        c.game.campaign_lessons.insert("here".into());
        assert!(!LessonState::end_due(&c, None));
    }

    /// No unit left: nothing that takes one can be done.
    fn nobody(d: DoneWhen) -> bool {
        !d.needs_unit()
    }

    #[test]
    fn a_lesson_that_needs_a_unit_is_put_away_for_the_turn_and_comes_back() {
        let first = TipTrigger::FirstBattleStart;
        let mut c = with(vec![
            lesson("unit", LessonLevel::Full, first, &[UP, DoneWhen::Attacked]),
            lesson("keys", LessonLevel::Full, first, &[UP]),
            lesson("arts", LessonLevel::Full, first, &[DoneWhen::ArtsOpened]),
        ]);
        let mut s = start(&c);
        s.mark_open();
        assert!(s.tick(&mut c, UP));
        // No unit left: the attack can't be done; "keys" stops the sweep.
        assert!(s.settle(1, nobody));
        assert_eq!(s.current(), Some("keys"));
        assert!(!s.is_open());
        assert!(!s.settle(1, nobody));
        s.mark_open();
        assert!(s.tick(&mut c, UP));
        // "arts" never opened: it doesn't, and isn't put away.
        assert!(s.settle(1, nobody));
        assert!(!s.in_play());
        // "unit" is not done, and not again this turn; "arts" waits for
        // its trigger.
        assert!(!c.game.campaign_lessons.contains("unit"));
        s.fire(first);
        s.absorb(&c, None, 1);
        assert_eq!(s.current(), Some("arts"));
        assert!(s.settle(1, nobody));
        assert!(!s.in_play());
        // A lesson that wasn't put away still comes this turn.
        c.game.content.lessons.lessons.push(lesson(
            "later",
            LessonLevel::Full,
            TipTrigger::FirstLowHp,
            &[UP],
        ));
        s.fire(TipTrigger::FirstLowHp);
        s.absorb(&c, None, 1);
        assert_eq!(s.current(), Some("later"));
        s.mark_open();
        assert!(s.tick(&mut c, UP));
        // A later turn: back, from an empty list.
        s.fire(first);
        s.absorb(&c, None, 2);
        assert_eq!(s.current(), Some("unit"));
        assert_eq!(s.ticked(), [false, false]);
    }

    /// Ticket 0464, for each kind of item a unit does: a lesson waiting
    /// behind another doesn't open when its turn comes if nobody left can
    /// do it, and comes back with its trigger that same turn; one open is
    /// put away for the turn the moment nobody can, ticks and all. What
    /// can still be done, or is ticked already, keeps it.
    #[test]
    fn a_lesson_nobody_left_can_do_doesnt_open_or_is_put_away() {
        let first = TipTrigger::FirstBattleStart;
        for done in [
            DoneWhen::Attacked,
            DoneWhen::ForecastOpened,
            DoneWhen::ArtsOpened,
            DoneWhen::TileAimed,
            DoneWhen::UnitMoved,
            DoneWhen::ThreatPointed,
        ] {
            let all = |_: DoneWhen| true;
            let not_it = |d: DoneWhen| d != done;
            let mut c = with(vec![
                lesson("before", LessonLevel::Full, first, &[UP]),
                lesson("it", LessonLevel::Full, first, &[UP, done]),
            ]);
            // Waiting: "before" is in play, and it can be done.
            let mut s = start(&c);
            assert!(!s.settle(1, not_it));
            s.mark_open();
            assert!(s.tick(&mut c, UP));
            // Its turn: nobody can, so it doesn't open.
            assert_eq!(s.current(), Some("it"));
            assert!(s.settle(1, not_it), "{done:?}");
            assert!(!s.in_play() && !s.holds_turn());
            // Its trigger again, this turn, once someone can: it opens.
            s.fire(first);
            s.absorb(&c, None, 1);
            assert_eq!(s.current(), Some("it"));
            assert!(!s.settle(1, all));
            s.mark_open();
            assert!(s.tick(&mut c, UP));
            assert!(s.holds_turn());
            // Open, and nobody left can: put away, unticked and not done.
            assert!(s.settle(1, not_it), "{done:?}");
            assert!(!s.in_play() && !s.holds_turn());
            assert!(!c.game.campaign_lessons.contains("it"));
            s.fire(first);
            s.absorb(&c, None, 1);
            assert!(!s.in_play(), "{done:?}: not again this turn");
            s.fire(first);
            s.absorb(&c, None, 2);
            assert_eq!(s.current(), Some("it"));
            assert_eq!(s.ticked(), [false, false]);
            // Ticked already: the rest can still be done.
            s.mark_open();
            assert!(s.tick(&mut c, done));
            assert!(!s.settle(2, not_it));
            assert!(s.is_open());
        }
    }

    /// A lesson whose items take no unit never leaves for what the units
    /// can't do.
    #[test]
    fn a_lesson_that_takes_no_unit_stays() {
        let first = TipTrigger::FirstBattleStart;
        let items = [
            UP,
            DoneWhen::CursorMoved,
            DoneWhen::RewindOpened,
            DoneWhen::RewindClosed,
            DoneWhen::OptionsOpened,
            DoneWhen::TurnEnded,
        ];
        let c = with(vec![lesson("keys", LessonLevel::Full, first, &items)]);
        let mut s = start(&c);
        assert!(!s.settle(1, nobody));
        s.mark_open();
        assert!(!s.settle(1, nobody));
        assert_eq!(s.current(), Some("keys"));
    }

    #[test]
    fn a_lesson_needs_the_rewind_screen_when_an_item_is_about_it() {
        let first = TipTrigger::FirstBattleStart;
        let rewind = DoneWhen::Pressed(Action::Rewind);
        for (items, needs) in [
            (&[UP][..], false),
            (&[UP, DoneWhen::RewindOpened][..], true),
            (&[rewind][..], true),
        ] {
            let c = with(vec![lesson("a", LessonLevel::Full, first, items)]);
            assert_eq!(start(&c).needs_rewind(), needs, "{items:?}");
        }
        assert!(!LessonState::default().needs_rewind());
    }

    /// Every condition a lesson can ask for.
    const ALL: [DoneWhen; 6] = [
        UP,
        DoneWhen::UnitSelected,
        DoneWhen::UnitMoved,
        DoneWhen::ForecastOpened,
        DoneWhen::Attacked,
        DoneWhen::RewindOpened,
    ];

    proptest! {
        /// Ticking a lesson's items in any order, with repeats and
        /// things it doesn't ask for in between, closes it exactly when
        /// the last of them is ticked.
        #[test]
        fn any_order_closes_the_lesson_exactly_when_all_are_ticked(
            n in 1usize..=6,
            order in prop::collection::vec(0usize..6, 0..30),
        ) {
            let items = &ALL[..n];
            let mut c = with(vec![lesson(
                "a", LessonLevel::Full, TipTrigger::FirstBattleStart, items,
            )]);
            let mut s = start(&c);
            s.mark_open();
            let mut done = std::collections::BTreeSet::new();
            for i in order {
                let ticked = s.tick(&mut c, ALL[i]);
                let fresh = i < n && done.len() < n && done.insert(i);
                prop_assert_eq!(ticked, fresh);
                prop_assert_eq!(s.current().is_none(), done.len() == n);
                prop_assert_eq!(c.game.campaign_lessons.contains("a"), done.len() == n);
            }
        }
    }
}
