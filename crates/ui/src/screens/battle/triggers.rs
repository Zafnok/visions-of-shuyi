//! When the guide's tips and lessons fire (0406, 0457, 1291, 1310): the
//! registry of trigger rows, one per [`TipTrigger`] name the tips and
//! lessons may use, each with the moment it is checked at and its
//! predicate. The [`Guide`](super::guide::Guide) queues what they fire and
//! decides when it shows.
//!
//! A new tip is a `tips.ron` entry, its text and a row in [`ROWS`]; the
//! content crate keeps no list of names. A name the tips or lessons use
//! with no row is reported by [`unknown_triggers`] (a test of the shipped
//! files).

use crate::prelude::*;

use trpg_content::{Content, TipTrigger};
use trpg_core::{CastTarget, Event, Faction, Phase, Pos, TerrainRules, Unit, UnitId};

use super::mode::{Line, MenuEntry, Mode};
use super::{BattleScreen, WATER, art_list, doable, forecast, map_menu};
use crate::screen::Ctx;

/// When a trigger's row is checked, with its predicate.
#[derive(Debug, Clone, Copy)]
pub enum When {
    /// On the battle's state and mode: at each frame's start, to the tips
    /// and the lessons, and at its end, to the lessons alone.
    State(fn(&BattleScreen) -> bool),
    /// On the battle's state with the content, at each frame's start
    /// after the [`State`](Self::State) rows, to the tips and the lessons.
    Content(fn(&BattleScreen, &Ctx) -> bool),
    /// At each frame's start, after the lessons took in what the rows
    /// before fired, so the first frame of a battle sees the lessons it
    /// starts.
    AfterLessons(fn(&BattleScreen, &Ctx) -> bool),
    /// On a command's events, as it applies.
    Events(fn(&BattleScreen, &[Event]) -> bool),
    /// Out of battle: the screen it names fires it
    /// ([`TipScreen::once`](crate::tips::screen::TipScreen::once)).
    Screen,
}

/// One trigger: its name, as the tips and lessons write it, and when it
/// fires.
#[derive(Debug, Clone, Copy)]
pub struct Row {
    /// Its name.
    pub trigger: TipTrigger,
    /// When it is checked, and its predicate.
    pub when: When,
}

const fn row(trigger: TipTrigger, when: When) -> Row {
    Row { trigger, when }
}

/// Every trigger the game knows, in the order each moment fires them.
pub const ROWS: &[Row] = &[
    row(TipTrigger::FirstBattleStart, When::State(battle_start)),
    row(TipTrigger::FirstEnemyPhase, When::State(enemy_phase)),
    row(TipTrigger::FirstLowHp, When::State(low_hp)),
    row(TipTrigger::FirstUnitSelected, When::State(unit_selected)),
    row(TipTrigger::FirstSkillUnit, When::State(skill_unit)),
    row(TipTrigger::FirstEnemyInRange, When::State(enemy_in_range)),
    row(TipTrigger::FirstForecast, When::State(forecast_open)),
    row(TipTrigger::FirstEffective, When::State(effective)),
    row(TipTrigger::FirstWeaponSwap, When::State(weapon_swap)),
    row(TipTrigger::DangerZoneAvailable, When::State(danger_zone)),
    row(TipTrigger::FirstTerrain, When::State(terrain)),
    row(TipTrigger::FirstCaster, When::State(caster)),
    row(TipTrigger::ArtsAvailable, When::State(arts)),
    row(TipTrigger::TileSpellAvailable, When::State(tile_spell)),
    row(TipTrigger::NoUnitLeft, When::State(no_unit_left)),
    row(TipTrigger::FirstUnitActed, When::State(unit_acted)),
    row(TipTrigger::SecondTurn, When::State(second_turn)),
    row(TipTrigger::ThreatArrowsShown, When::State(threat_arrows)),
    row(TipTrigger::FirstWadeInReach, When::Content(wade_in_reach)),
    row(
        TipTrigger::ZoomAvailable,
        When::AfterLessons(zoom_available),
    ),
    row(TipTrigger::FirstLevelUp, When::Events(level_up)),
    row(TipTrigger::FirstLoadGame, When::Screen),
    row(TipTrigger::FirstProfileScreen, When::Screen),
];

/// The names `content`'s tips and lessons use that no row has, in file
/// order (tips, then lessons), each once: they would never fire.
pub fn unknown_triggers(content: &Content) -> Vec<TipTrigger> {
    let known = |t: &TipTrigger| ROWS.iter().any(|r| r.trigger == *t);
    let tips = content.tips.tips.iter().map(|t| t.trigger);
    let lessons = content.lessons.lessons.iter().map(|l| l.trigger);
    let mut unknown = Vec::new();
    for t in tips.chain(lessons) {
        if !known(&t) && !unknown.contains(&t) {
            unknown.push(t);
        }
    }
    unknown
}

impl BattleScreen {
    /// The registry's rows, then the ones added to this battle
    /// ([`add_trigger_row`](Self::add_trigger_row)).
    fn rows(&self) -> impl Iterator<Item = &Row> {
        ROWS.iter().chain(&self.extra_rows)
    }

    /// Adds `row` to this battle's triggers, after the registry's: a
    /// trigger the registry doesn't have (a test's).
    pub fn add_trigger_row(&mut self, row: Row) {
        self.extra_rows.push(row);
    }

    /// Fires the [`When::Events`] rows `events` hold for (a player unit's
    /// level-up fires [`TipTrigger::FirstLevelUp`]).
    pub(super) fn note_level_ups(&mut self, events: &[Event]) {
        let fired: Vec<TipTrigger> = self
            .rows()
            .filter(|r| matches!(r.when, When::Events(check) if check(self, events)))
            .map(|r| r.trigger)
            .collect();
        for trigger in fired {
            self.guide.fire(trigger);
        }
    }

    /// At the frame's start (1292): the queued tips' moment passes if the
    /// kind of mode changed ([`Guide::take_in_mode`]), then the tips and
    /// lessons the battle's state fired are queued.
    ///
    /// [`Guide::take_in_mode`]: super::guide::Guide::take_in_mode
    pub(super) fn take_in_tips(&mut self, ctx: &mut Ctx) {
        self.guide.take_in_mode(&self.mode);
        let mut fired = self.triggers_now();
        fired.extend(self.fired_with(ctx, |w, s, c| match w {
            When::Content(check) => check(s, c),
            _ => false,
        }));
        for trigger in fired {
            self.guide.fire(trigger);
        }
        let battle = self.battle_id.as_deref();
        self.guide
            .absorb_lessons(ctx, battle, &self.state, &self.mode);
        let after = self.fired_with(ctx, |w, s, c| match w {
            When::AfterLessons(check) => check(s, c),
            _ => false,
        });
        for trigger in after {
            self.guide.fire(trigger);
        }
        self.guide.absorb_tips(ctx);
    }

    /// What the frame did may start a lesson auto-end must wait for
    /// (every unit has acted: ending the turn), or leave one that can't be
    /// finished this turn: the triggers the state shows now go to the
    /// lessons alone, which take them in.
    pub(super) fn take_in_lessons(&mut self, ctx: &Ctx) {
        for trigger in self.triggers_now() {
            self.guide.fire_lesson(trigger);
        }
        let battle = self.battle_id.as_deref();
        self.guide
            .absorb_lessons(ctx, battle, &self.state, &self.mode);
    }

    /// The tip and lesson triggers the battle's current state shows: its
    /// [`When::State`] rows that hold, in order.
    pub(super) fn triggers_now(&self) -> Vec<TipTrigger> {
        self.rows()
            .filter(|r| matches!(r.when, When::State(check) if check(self)))
            .map(|r| r.trigger)
            .collect()
    }

    /// The rows whose moment and predicate `holds` with `ctx`.
    fn fired_with(&self, ctx: &Ctx, holds: impl Fn(When, &Self, &Ctx) -> bool) -> Vec<TipTrigger> {
        self.rows()
            .filter(|r| holds(r.when, self, ctx))
            .map(|r| r.trigger)
            .collect()
    }

    /// Whether `unit` has a skill it can use now (a passive or an active).
    pub(super) fn has_skill(&self, unit: UnitId) -> bool {
        let state = &self.state;
        state
            .unit(unit)
            .is_some_and(|u| !u.usable_skills(state.classes(), state.skills()).is_empty())
    }

    /// Whether the terrain at `pos` is not plain ground: it gives a Def,
    /// Avoid or healing bonus, or costs a movement type other than 1 to
    /// enter (more, or impassable).
    pub(super) fn special_terrain(&self, pos: Pos) -> bool {
        let state = &self.state;
        let rules = state
            .map()
            .tiles
            .get(pos)
            .and_then(|&id| state.terrain().get(id));
        rules.is_some_and(not_plain)
    }
}

/// Whether terrain `t` is not plain ground: it gives a Def, Avoid or
/// healing bonus, or costs a movement type other than 1 to enter (more, or
/// impassable).
pub(super) fn not_plain(t: &TerrainRules) -> bool {
    t.defense != 0
        || t.avoid != 0
        || t.heal_percent != 0
        || t.move_cost.iter().any(|&c| c != Some(1))
}

// The predicates, by moment.

/// Turn 1's player phase.
fn battle_start(s: &BattleScreen) -> bool {
    s.state.turn() == 1 && s.state.phase() == Phase::Player
}

/// The enemy phase.
fn enemy_phase(s: &BattleScreen) -> bool {
    s.state.phase() == Phase::Enemy
}

/// A unit of the player's is alive at half its HP or less.
fn low_hp(s: &BattleScreen) -> bool {
    let low = |u: &Unit| u.faction == Faction::Player && u.hp > 0 && u.hp * 2 <= u.stats.hp;
    s.state.units().iter().any(low)
}

/// A unit is selected.
fn unit_selected(s: &BattleScreen) -> bool {
    matches!(s.mode, Mode::Selected(_))
}

/// The selected unit has a skill it can use.
fn skill_unit(s: &BattleScreen) -> bool {
    matches!(&s.mode, Mode::Selected(sel) if s.has_skill(sel.unit))
}

/// The selected unit could reach an enemy.
fn enemy_in_range(s: &BattleScreen) -> bool {
    matches!(&s.mode, Mode::Selected(sel) if doable::reaches_enemy(&s.state, sel))
}

/// The forecast is open.
fn forecast_open(s: &BattleScreen) -> bool {
    matches!(s.mode, Mode::Targeting(_))
}

/// The forecast is open and a strike is effective by its weapon kind.
fn effective(s: &BattleScreen) -> bool {
    matches!(&s.mode, Mode::Targeting(t) if !forecast::kind_effects(&s.state, t).is_empty())
}

/// The forecast is open and its unit can swap weapons there.
fn weapon_swap(s: &BattleScreen) -> bool {
    matches!(&s.mode, Mode::Targeting(t) if t.can_swap())
}

/// Browsing, the cursor on an enemy, with no danger zone shown.
fn danger_zone(s: &BattleScreen) -> bool {
    let on_enemy = s.hovered().is_some_and(|u| u.faction == Faction::Enemy);
    matches!(s.mode, Mode::Idle { .. }) && on_enemy && s.threat.danger().is_none()
}

/// Browsing, the cursor on terrain that is not plain ground.
fn terrain(s: &BattleScreen) -> bool {
    matches!(s.mode, Mode::Idle { .. }) && s.special_terrain(s.cursor.pos)
}

/// The action menu is open with Magic in it.
fn caster(s: &BattleScreen) -> bool {
    let Mode::ActionMenu { menu, .. } = &s.mode else {
        return false;
    };
    menu.items().iter().any(|i| i.value == MenuEntry::Magic)
}

/// The action menu is open with `Arts` its unit can use.
fn arts(s: &BattleScreen) -> bool {
    matches!(&s.mode, Mode::ActionMenu { arts, .. } if art_list::can_use_art(arts))
}

/// A spell list is open with a spell it could cast on a tile.
fn tile_spell(s: &BattleScreen) -> bool {
    let Mode::List { menu, .. } = &s.mode else {
        return false;
    };
    let on_tile = |t: &CastTarget| matches!(t, CastTarget::Tile(_));
    let tile_spell = |line: &Line| match line {
        Line::Spell(c) => c.targets.iter().any(on_tile),
        _ => false,
    };
    menu.items().iter().any(|i| tile_spell(&i.value))
}

/// The player's phase of a battle still going: the lessons' triggers
/// (0457) fire only then.
fn lessons_phase(s: &BattleScreen) -> bool {
    s.state.phase() == Phase::Player && s.state.outcome().is_none()
}

/// Browsing, with every unit of the player's acted.
fn no_unit_left(s: &BattleScreen) -> bool {
    let browsing = matches!(s.mode, Mode::Idle { .. });
    lessons_phase(s) && browsing && map_menu::ready_players(&s.state) == 0
}

/// Browsing on turn 1 once a unit of the player's has acted (1304).
fn unit_acted(s: &BattleScreen) -> bool {
    let browsing = matches!(s.mode, Mode::Idle { .. });
    let acted = |u: &Unit| u.faction == Faction::Player && u.acted;
    lessons_phase(s) && s.state.turn() == 1 && browsing && s.state.units().iter().any(acted)
}

/// Turn 2's player phase.
fn second_turn(s: &BattleScreen) -> bool {
    lessons_phase(s) && s.state.turn() == 2
}

/// The threat arrows show.
fn threat_arrows(s: &BattleScreen) -> bool {
    lessons_phase(s) && s.threat_view().is_some()
}

/// The selected unit could step into a river (0483): `water` is the
/// content's terrain, so it is looked up in `ctx`.
fn wade_in_reach(s: &BattleScreen, ctx: &Ctx) -> bool {
    let Mode::Selected(sel) = &s.mode else {
        return false;
    };
    let water = ctx.game.content.terrain.display.id_of(WATER);
    water.is_some_and(|w| doable::wades_in_reach(&s.state, sel, w))
}

/// Browsing a map whose skin can zoom with no lesson in play but one that
/// waits for the turn to end (0466, 1304).
fn zoom_available(s: &BattleScreen, ctx: &Ctx) -> bool {
    let browsing = matches!(s.mode, Mode::Idle { .. });
    browsing && !s.guide.lessons_busy() && ctx.game.map_skin.zoom().is_some()
}

/// A unit of the player's levels up in `events`.
fn level_up(s: &BattleScreen, events: &[Event]) -> bool {
    events.iter().any(|e| {
        matches!(e, Event::LeveledUp { unit, .. }
            if s.state.unit(*unit).is_some_and(|u| u.faction == Faction::Player))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::tests::ctx;

    /// Ticket 1310: every name the game's tips and lessons use, and the
    /// tests' tips, has a row (a name with none would never fire).
    #[test]
    fn every_trigger_the_tips_and_lessons_name_has_a_row() {
        let c = ctx();
        let content = &c.game.content;
        assert!(!content.tips.tips.is_empty() && !content.lessons.lessons.is_empty());
        assert_eq!(unknown_triggers(content), []);
    }

    /// A tip or a lesson whose name has no row is reported, once.
    #[test]
    fn a_name_with_no_row_fails() {
        let mut content = ctx().game.content;
        let source = r#"(tips: [(id: "a", trigger: NoSuchTrigger, title: "T", text: "x")])"#;
        let tips = trpg_content::tip::from_source("t.ron", source, None).unwrap();
        content.tips.tips.extend(tips.tips);
        let mut lesson = content.lessons.lessons[0].clone();
        lesson.trigger = TipTrigger::new("NoSuchTrigger");
        content.lessons.lessons.push(lesson.clone());
        lesson.trigger = TipTrigger::new("NoLessonTrigger");
        content.lessons.lessons.push(lesson);
        let unknown = ["NoSuchTrigger", "NoLessonTrigger"].map(TipTrigger::new);
        assert_eq!(unknown_triggers(&content), unknown);
    }

    /// One row per name.
    #[test]
    fn each_name_has_one_row() {
        for (i, row) in ROWS.iter().enumerate() {
            let later = ROWS[i + 1..].iter().any(|r| r.trigger == row.trigger);
            assert!(!later, "{} has two rows", row.trigger);
        }
    }
}
