//! Tests of the one-time tips on the battle screen (ticket 0406): each
//! trigger fires once per profile, tips take the keys until dismissed, and
//! their text names the keys the player has bound.

use crate::input::{LEFT_HANDED, RIGHT_HANDED};
use insta::assert_snapshot;
use shuyi_ui::text::StyledText;
use trpg_content::TipTrigger;
use trpg_core::{Command, Event, Objective, Phase, StatGains, TerrainRules, UnitId};

use super::banner::PHASE_BANNER_S;
use super::layout::{MAP_VIEW, SIDE_PANEL, snap};
use super::testing::{Stacked, battle_with, skirmish, skirmish_charged, through_ai_phases};
use super::testing::{quick, render, step};
use super::triggers::not_plain;
use super::*;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::screen::tests::ctx;
use crate::storage::Storage;
use crate::tips::{TIPS_SEEN_KEY, reset_tips};

/// A context with tips on.
fn tipped() -> Ctx {
    let mut c = ctx();
    c.game.tips_enabled = true;
    c
}

/// One frame of `dt` seconds with `actions`.
fn frame(s: &mut BattleScreen, c: &mut Ctx, actions: &[Action], dt: f32) {
    s.update(c, &FrameInput::new(actions.to_vec(), dt, vec![]));
}

/// Whether the profile has seen the tip `id` (0467).
fn seen(c: &Ctx, id: &str) -> bool {
    let seen = c.storage.read(TIPS_SEEN_KEY).ok().flatten();
    seen.is_some_and(|s| s.lines().any(|l| l == id))
}

/// Whether some row of the rendered screen contains `text`.
fn shows(s: &BattleScreen, c: &Ctx, text: &str) -> bool {
    let buf = render(s, c);
    (0..i32::from(ConsoleSpec::PC.rows)).any(|y| {
        let row: String = (0..i32::from(ConsoleSpec::PC.cols))
            .map(|x| buf.get(x, y).map_or(' ', |cell| cell.glyph))
            .collect();
        row.contains(text)
    })
}

/// Dismisses every tip on screen, returning the triggers it took. After
/// each, the next in its moment comes up at once (1292).
fn dismiss_all(s: &mut BattleScreen, c: &mut Ctx) -> Vec<TipTrigger> {
    let mut seen = Vec::new();
    for _ in 0..10 {
        step(s, c, &[]);
        let Some(t) = s.shown_tip() else {
            return seen;
        };
        seen.push(t);
        step(s, c, &[Action::Confirm]);
    }
    panic!("tips never ran out: {seen:?}");
}

#[test]
fn the_first_battle_shows_the_start_tip_and_confirm_dismisses_it() {
    let mut c = tipped();
    let mut s = quick();
    assert_eq!(s.shown_tip(), None);
    step(&mut s, &mut c, &[]);
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstBattleStart));
    assert!(shows(&s, &c, "Your move"));
    assert_eq!(s.help(&c), "f close");
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.shown_tip(), None);
    assert!(!shows(&s, &c, "Your move"));
    assert_eq!(s.mode(), &Mode::default());
}

/// Ticket 0455: the guide speaks the tip: their name (the names table's
/// `guide`, 0731) is the box's title, the tip's title its first line, and
/// their face is beside the text when the game has their portrait, smiling
/// unless the tip says otherwise.
#[test]
fn the_tip_box_is_the_guides() {
    let mut c = tipped();
    let mut s = quick();
    assert_eq!(s.tip_view(&c), None);
    step(&mut s, &mut c, &[]);
    let view = s.tip_view(&c).unwrap();
    assert_eq!(Some(view.title.as_str()), c.game.content.names.get("guide"));
    assert_eq!(view.title, "Vix");
    assert_eq!(view.heading, "Your move");
    assert_eq!(view.face, None);
    assert_eq!(view.lines.len(), 3);
    assert!(view.lines[0].starts_with("Over to you. Steer the cursor onto"));
    // Her line names no key; the instructions under it do (0475).
    assert_eq!(
        view.how.iter().map(StyledText::plain).collect::<Vec<_>>(),
        [
            "Move the cursor: arrows. Pick up a unit: f",
            "Next unit to act: s. A unit's details: e"
        ]
    );
    assert_eq!(view.close.plain(), "f close");
    // With the guide's portrait (the bought art), the face.
    let mut guide = c.game.content.portraits["test_lord"].clone();
    let smile = guide.expressions[1].clone();
    guide.expressions.push(trpg_content::portrait::Expression {
        name: "smile".into(),
        ..smile
    });
    c.game.content.portraits.insert("guide".into(), guide);
    // A tip naming no expression smiles.
    let tips = c.game.content.tips.tips.iter_mut();
    tips.filter(|t| t.id == "battle_start")
        .for_each(|t| t.face = None);
    let face = s.tip_view(&c).and_then(|v| v.face).unwrap();
    assert_eq!(
        (face.portrait.as_str(), face.expression.as_str()),
        ("guide", "smile")
    );
    // A tip naming another expression shows it.
    let tip = c
        .game
        .content
        .tips
        .tips
        .iter_mut()
        .find(|t| t.id == "battle_start");
    tip.unwrap().face = Some("sad".into());
    let face = s.tip_view(&c).and_then(|v| v.face).unwrap();
    assert_eq!(face.expression, "sad");
}

/// A campaign's tutorial level (0835): tips show at `Full` (and with no
/// campaign, as here elsewhere), not at `Short` or `None`, and a tip held
/// back isn't marked seen.
#[test]
fn only_a_full_tutorial_shows_tips() {
    use trpg_core::TutorialLevel;
    for (level, shown) in [
        (None, true),
        (Some(TutorialLevel::Full), true),
        (Some(TutorialLevel::Short), false),
        (Some(TutorialLevel::None), false),
    ] {
        let mut c = tipped();
        c.game.campaign_tutorial = level;
        let mut s = quick();
        step(&mut s, &mut c, &[]);
        let tip = shown.then_some(TipTrigger::FirstBattleStart);
        assert_eq!(s.shown_tip(), tip, "{level:?}");
        let seen = c.storage.read(TIPS_SEEN_KEY).ok().flatten();
        assert_eq!(seen.is_some(), shown, "{level:?}");
    }
}

#[test]
fn the_start_tip_waits_for_the_first_player_phase_banner() {
    let mut c = tipped();
    let state = quick_battle(&c.game.content).unwrap();
    let events = [Event::PhaseStarted {
        turn: 1,
        phase: Phase::Player,
    }];
    // The objective's banner waits for Confirm (0481); the phase banner
    // after it closes by itself: the tip shows.
    let mut s = BattleScreen::start(state.clone(), &events);
    step(&mut s, &mut c, &[]);
    assert_eq!(s.shown_tip(), None);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert!(s.banner().is_some());
    assert_eq!(s.shown_tip(), None);
    assert!(!shows(&s, &c, "Your move"));
    frame(&mut s, &mut c, &[], PHASE_BANNER_S * 0.9);
    assert_eq!(s.shown_tip(), None);
    frame(&mut s, &mut c, &[], PHASE_BANNER_S * 0.2);
    assert_eq!(s.banner(), None);
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstBattleStart));
    assert!(shows(&s, &c, "Your move"));
    // The Confirm that closes the banner doesn't also close the tip.
    let mut c = tipped();
    let mut s = BattleScreen::start(state, &events);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.shown_tip(), None);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.banner(), None);
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstBattleStart));
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.shown_tip(), None);
    assert_eq!(s.mode(), &Mode::default());
}

#[test]
fn cancel_dismisses_too_and_other_keys_wait() {
    let mut c = tipped();
    let mut s = quick();
    let start = s.cursor().pos;
    step(&mut s, &mut c, &[]);
    step(&mut s, &mut c, &[Action::CursorRight, Action::Confirm]);
    // Neither the cursor move nor the select got through, and the tip is
    // gone: the first action of the frame was swallowed, then the
    // Confirm dismissed it.
    assert_eq!(s.cursor().pos, start);
    assert_eq!(s.shown_tip(), None);
    assert_eq!(s.mode(), &Mode::default());
    // Cancel dismisses a tip as well.
    let mut c = tipped();
    let mut s = quick();
    step(&mut s, &mut c, &[]);
    step(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(s.shown_tip(), None);
    assert!(matches!(s.mode(), Mode::Idle { .. }), "{:?}", s.mode());
}

/// Ticket 0467: a tip is saved as seen when it is on screen, not when its
/// trigger fires; one that never reached the screen shows on the next
/// battle (a Retry is a new battle screen).
#[test]
fn a_tip_counts_as_seen_once_it_is_shown() {
    let mut c = tipped();
    let mut s = quick();
    s.guide.tips.fire(TipTrigger::FirstBattleStart);
    s.guide.tips.absorb(&mut c);
    assert_eq!(
        s.guide.tips.queued().collect::<Vec<_>>(),
        [TipTrigger::FirstBattleStart]
    );
    assert_eq!(c.storage.read(TIPS_SEEN_KEY).unwrap(), None, "queued only");
    // Firing again doesn't queue it twice.
    s.guide.tips.fire(TipTrigger::FirstBattleStart);
    s.guide.tips.absorb(&mut c);
    assert_eq!(s.guide.tips.queued().count(), 1);
    // Left before it showed: the next battle shows it.
    let mut retry = quick();
    assert_eq!(
        dismiss_all(&mut retry, &mut c),
        [TipTrigger::FirstBattleStart]
    );
    // On screen: seen.
    assert_eq!(
        c.storage.read(TIPS_SEEN_KEY).unwrap().as_deref(),
        Some("battle_start")
    );
    let mut s = quick();
    s.guide.tips.show(Some(TipTrigger::FirstLowHp), &mut c);
    let seen = c.storage.read(TIPS_SEEN_KEY).unwrap().unwrap();
    assert_eq!(
        seen,
        "battle_start
low_hp"
    );
}

/// Ticket 0467: a Confirm in the frame a tip comes due, before it has
/// been drawn, doesn't close it unseen; it is swallowed.
#[test]
fn a_tip_not_yet_drawn_takes_the_key_but_stays() {
    let mut c = tipped();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstBattleStart));
    assert_eq!(s.mode(), &Mode::default());
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.shown_tip(), None);
}

#[test]
fn a_tip_shows_once_per_profile() {
    let mut c = tipped();
    let mut s = quick();
    assert_eq!(dismiss_all(&mut s, &mut c), [TipTrigger::FirstBattleStart]);
    assert_eq!(
        c.storage.read(TIPS_SEEN_KEY).unwrap().as_deref(),
        Some("battle_start")
    );
    // Not again on this screen, nor on a new battle with the same storage.
    step(&mut s, &mut c, &[]);
    assert_eq!(s.shown_tip(), None);
    let mut again = quick();
    assert_eq!(dismiss_all(&mut again, &mut c), []);
    // Resetting the tips brings it back.
    reset_tips(&mut c.storage).unwrap();
    let mut fresh = quick();
    assert_eq!(
        dismiss_all(&mut fresh, &mut c),
        [TipTrigger::FirstBattleStart]
    );
}

#[test]
fn tips_are_off_unless_enabled() {
    let mut c = ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[]);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.shown_tip(), None);
    assert!(matches!(s.mode(), Mode::Selected(_)), "{:?}", s.mode());
    assert_eq!(c.storage.read(TIPS_SEEN_KEY).unwrap(), None);
}

#[test]
fn selecting_a_unit_shows_the_selection_tip_then_the_in_range_tip() {
    let mut c = tipped();
    let mut s = quick();
    dismiss_all(&mut s, &mut c);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert!(matches!(s.mode(), Mode::Selected(_)));
    // The brigand at (7, 4) is within the lord's reach; the lord has a
    // skill (0458).
    assert_eq!(
        dismiss_all(&mut s, &mut c),
        [
            TipTrigger::FirstUnitSelected,
            TipTrigger::FirstSkillUnit,
            TipTrigger::FirstEnemyInRange
        ]
    );
    // Selecting again shows nothing.
    step(&mut s, &mut c, &[Action::Cancel]);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(dismiss_all(&mut s, &mut c), []);
}

#[test]
fn no_in_range_tip_when_no_enemy_is_near() {
    let mut c = tipped();
    let quick = quick_battle(&c.game.content).unwrap();
    let mut units = quick.units().to_vec();
    for u in units.iter_mut().filter(|u| u.faction == Faction::Enemy) {
        u.pos = Pos::new(13, 0);
    }
    let rout = Objective::Rout { turn_limit: None };
    let mut s = BattleScreen::new(battle_with(&c, quick.map().clone(), units, rout));
    dismiss_all(&mut s, &mut c);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(
        dismiss_all(&mut s, &mut c),
        [TipTrigger::FirstUnitSelected, TipTrigger::FirstSkillUnit]
    );
}

#[test]
fn the_forecast_shows_its_tip() {
    let mut c = tipped();
    let mut s = BattleScreen::new(skirmish(&c, 100));
    dismiss_all(&mut s, &mut c);
    // Select the lord, step right, open the menu: Attack.
    step(&mut s, &mut c, &[Action::Confirm]);
    dismiss_all(&mut s, &mut c);
    step(&mut s, &mut c, &[Action::CursorRight, Action::Confirm]);
    frame(&mut s, &mut c, &[], 1.0);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert!(matches!(s.mode(), Mode::Targeting(_)), "{:?}", s.mode());
    // The lord has two swords: the swap tip follows (0463).
    let seen = dismiss_all(&mut s, &mut c);
    assert_eq!(
        seen,
        [TipTrigger::FirstForecast, TipTrigger::FirstWeaponSwap]
    );
    // The forecast is still there under the dismissed tip.
    assert!(matches!(s.mode(), Mode::Targeting(_)));
    // A tip waits out the combat that follows.
    step(&mut s, &mut c, &[Action::Confirm]);
    assert!(matches!(s.mode(), Mode::Combat(_)), "{:?}", s.mode());
    s.guide.tips.fire(TipTrigger::FirstLowHp);
    step(&mut s, &mut c, &[]);
    assert_eq!(s.shown_tip(), None);
}

#[test]
fn a_badly_hurt_unit_shows_the_low_hp_tip() {
    let mut c = tipped();
    let quick = quick_battle(&c.game.content).unwrap();
    let mut units = quick.units().to_vec();
    units[1].hp = 1;
    let rout = Objective::Rout { turn_limit: None };
    let mut s = BattleScreen::new(battle_with(&c, quick.map().clone(), units, rout));
    assert_eq!(
        dismiss_all(&mut s, &mut c),
        [TipTrigger::FirstBattleStart, TipTrigger::FirstLowHp]
    );
}

#[test]
fn a_player_level_up_shows_its_tip_but_an_enemys_does_not() {
    let mut c = tipped();
    let mut s = quick();
    dismiss_all(&mut s, &mut c);
    let up = |unit| Event::LeveledUp {
        unit: UnitId(unit),
        level: 2,
        gains: StatGains::default(),
    };
    s.note_level_ups(&[up(4)]);
    assert_eq!(dismiss_all(&mut s, &mut c), []);
    s.note_level_ups(&[up(1)]);
    assert_eq!(dismiss_all(&mut s, &mut c), [TipTrigger::FirstLevelUp]);
}

#[test]
fn resting_on_an_enemy_shows_the_danger_zone_tip() {
    let mut c = tipped();
    let mut s = quick();
    dismiss_all(&mut s, &mut c);
    s.cursor.jump(Pos::new(8, 2));
    assert_eq!(
        dismiss_all(&mut s, &mut c),
        [TipTrigger::DangerZoneAvailable]
    );
    assert!(!shows(&s, &c, "Danger zone"));
}

#[test]
fn the_enemy_phase_tip_shows_over_its_banner_and_holds_it() {
    let mut c = tipped();
    let mut s = quick();
    dismiss_all(&mut s, &mut c);
    step(&mut s, &mut c, &[Action::EndTurn]);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.state().phase(), Phase::Enemy);
    step(&mut s, &mut c, &[]);
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstEnemyPhase));
    assert!(shows(&s, &c, "Enemy phase"));
    // The banner waits while the tip is up, however long the player reads.
    frame(&mut s, &mut c, &[], 30.0);
    assert_eq!(s.state().phase(), Phase::Enemy);
    assert!(s.banner().is_some());
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.shown_tip(), None);
    // Then the banner runs its course, the enemies act and the player
    // phase comes back.
    through_ai_phases(&mut s, &mut c, 30.0);
    assert_eq!(s.state().phase(), Phase::Player);
    assert_eq!(s.state().turn(), 2);
}

#[test]
fn tips_wait_for_the_player_phase_to_settle() {
    let mut c = tipped();
    let mut s = quick();
    dismiss_all(&mut s, &mut c);
    step(&mut s, &mut c, &[Action::EndTurn]);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.state().phase(), Phase::Enemy);
    // A unit found badly hurt while the enemy plays: the tip waits.
    s.guide.tips.fire(TipTrigger::FirstLowHp);
    // (The enemy-phase tip is queued behind it and shows first.)
    let shown = dismiss_all(&mut s, &mut c);
    assert_eq!(shown, [TipTrigger::FirstEnemyPhase]);
    // The player phase's banner comes first; the tip waits for it.
    through_ai_phases(&mut s, &mut c, 30.0);
    assert_eq!(s.state().phase(), Phase::Player);
    assert!(s.banner().is_some());
    assert_eq!(s.shown_tip(), None);
    for _ in 0..4 {
        frame(&mut s, &mut c, &[], 30.0);
    }
    assert!(s.banner().is_none());
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstLowHp));
}

/// Ticket 1292: a tip fired under the rewind screen missed its moment. It
/// doesn't come up when the screen goes; it stays unseen and shows the
/// next time its trigger fires.
#[test]
fn a_tip_fired_under_the_rewind_screen_waits_for_its_trigger() {
    let mut c = tipped();
    let mut s = BattleScreen::new(skirmish_charged(&c, 100, 3));
    dismiss_all(&mut s, &mut c);
    let mut s = Stacked::new(s);
    s.press(&mut c, &[Action::Rewind]);
    assert!(s.rewind().is_some());
    s.battle_mut().guide.tips.fire(TipTrigger::FirstLowHp);
    s.frame(&mut c, &[], 0.0);
    assert_eq!(s.shown_tip(), None);
    // Closed: no tip comes up, and the one fired under it is still unseen.
    s.press(&mut c, &[Action::Cancel]);
    assert!(s.rewind().is_none());
    for _ in 0..3 {
        s.frame(&mut c, &[], 0.1);
        assert_eq!(s.shown_tip(), None);
    }
    assert!(!seen(&c, "low_hp"), "dropped unseen");
    // Its trigger again, on the map: it comes up at once.
    s.battle_mut().guide.tips.fire(TipTrigger::FirstLowHp);
    s.frame(&mut c, &[], 0.0);
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstLowHp));
}

/// A battle like the Quick Battle, with `edit` applied to its units.
fn edited(c: &Ctx, edit: impl FnOnce(&mut Vec<Unit>)) -> BattleScreen {
    let quick = quick_battle(&c.game.content).unwrap();
    let mut units = quick.units().to_vec();
    edit(&mut units);
    let rout = Objective::Rout { turn_limit: None };
    BattleScreen::new(battle_with(c, quick.map().clone(), units, rout))
}

#[test]
fn low_hp_is_half_of_max_hp_or_less_of_a_unit_still_standing() {
    let low = |hp| {
        let mut c = tipped();
        let mut s = edited(&c, |units| {
            units[1].stats.hp = 20;
            units[1].hp = hp;
        });
        dismiss_all(&mut s, &mut c).contains(&TipTrigger::FirstLowHp)
    };
    assert!(low(10));
    assert!(!low(11));
    assert!(!low(0));
    // An enemy's low HP is not a player's worry.
    let mut c = tipped();
    let mut s = edited(&c, |units| units[3].hp = 1);
    assert_eq!(dismiss_all(&mut s, &mut c), [TipTrigger::FirstBattleStart]);
}

#[test]
fn the_start_tip_is_for_the_first_turn_of_the_player_phase_only() {
    let mut c = tipped();
    let mut state = quick_battle(&c.game.content).unwrap();
    state.apply(&Command::EndPhase).unwrap();
    assert_eq!((state.turn(), state.phase()), (1, Phase::Enemy));
    let mut s = BattleScreen::new(state.clone());
    assert_eq!(dismiss_all(&mut s, &mut c), [TipTrigger::FirstEnemyPhase]);
    // Nor does a start tip wait behind it for the next player phase.
    through_ai_phases(&mut s, &mut c, 30.0);
    for _ in 0..5 {
        frame(&mut s, &mut c, &[], 30.0);
    }
    assert_eq!(s.state().phase(), Phase::Player);
    let later = dismiss_all(&mut s, &mut c);
    assert!(!later.contains(&TipTrigger::FirstBattleStart), "{later:?}");
    state.apply(&Command::EndPhase).unwrap();
    assert_eq!((state.turn(), state.phase()), (2, Phase::Player));
    let mut s = BattleScreen::new(state);
    assert_eq!(dismiss_all(&mut s, &mut c), []);
}

#[test]
fn a_tip_names_the_keys_the_player_has_bound() {
    let mut c = tipped();
    c.use_layout(LEFT_HANDED);
    let mut s = quick();
    step(&mut s, &mut c, &[]);
    assert!(shows(&s, &c, "Move the cursor: w a s d. Pick up a unit: j"));
    assert_eq!(s.help(&c), "j close");
}

#[test]
fn a_tip_shows_not_mapped_for_an_action_with_no_key() {
    let mut c = tipped();
    let mut keys = c.layout_bindings(RIGHT_HANDED);
    keys.clear(Action::NextUnit, 0);
    c.set_layout_bindings(RIGHT_HANDED, keys).unwrap();
    let mut s = quick();
    step(&mut s, &mut c, &[]);
    assert!(shows(&s, &c, "Next unit to act: ! not mapped."));
}

#[test]
fn the_start_tip_renders() {
    let mut c = tipped();
    let mut s = quick();
    step(&mut s, &mut c, &[]);
    assert_snapshot!(snap(&render(&s, &c), &c, MAP_VIEW));
}

// ---- Skills, magic and terrain (ticket 0458) -----------------------------------

/// Selecting a unit with a skill shows the skills tip; one without
/// doesn't. Every player class has an active skill, so the unit without
/// one is the lord made a Brigand (a class with none) that learned none.
#[test]
fn selecting_a_unit_with_a_skill_shows_the_skills_tip() {
    for skilled in [true, false] {
        let mut c = tipped();
        let mut s = edited(&c, |units| {
            if !skilled {
                units[0].class = trpg_core::ClassId("brigand".into());
                units[0].class_records.clear();
                units[0].learned_skills.clear();
            }
        });
        let lord = s.state().units()[0].clone();
        assert_eq!(s.has_skill(lord.id), skilled);
        dismiss_all(&mut s, &mut c);
        s.cursor.jump(lord.pos);
        step(&mut s, &mut c, &[Action::Confirm]);
        assert!(matches!(s.mode(), Mode::Selected(_)), "{skilled}");
        let tips = dismiss_all(&mut s, &mut c);
        assert_eq!(
            tips.contains(&TipTrigger::FirstSkillUnit),
            skilled,
            "{tips:?}"
        );
    }
}

/// Selects the player unit `name` and keeps it where it stands: its
/// action menu opens. The tips on the way are dismissed and returned.
fn open_action_menu(s: &mut BattleScreen, c: &mut Ctx, name: &str) -> Vec<TipTrigger> {
    let at = s
        .state()
        .units()
        .iter()
        .find(|u| u.name == name)
        .unwrap()
        .pos;
    dismiss_all(s, c);
    s.cursor.jump(at);
    step(s, c, &[Action::Confirm]);
    let mut tips = dismiss_all(s, c);
    step(s, c, &[Action::Confirm]);
    frame(s, c, &[], 1.0);
    assert!(
        matches!(s.mode(), Mode::ActionMenu { .. }),
        "{:?}",
        s.mode()
    );
    tips.extend(dismiss_all(s, c));
    tips
}

#[test]
fn a_casters_action_menu_shows_the_magic_tip_once() {
    let mut c = tipped();
    let mut s = quick();
    let tips = open_action_menu(&mut s, &mut c, "Test Lord");
    assert!(!tips.contains(&TipTrigger::FirstCaster), "{tips:?}");
    let mut s = quick();
    let tips = open_action_menu(&mut s, &mut c, "Test Mage");
    assert!(tips.contains(&TipTrigger::FirstCaster), "{tips:?}");
    // Shown once per profile.
    let mut s = quick();
    let tips = open_action_menu(&mut s, &mut c, "Test Mage");
    assert!(!tips.contains(&TipTrigger::FirstCaster), "{tips:?}");
}

/// A tile with no unit on it whose terrain is plain ground if not
/// `special`, else not.
fn empty_tile(s: &BattleScreen, special: bool) -> Pos {
    let map = &s.state().map().tiles;
    let w = i32::from(map.width());
    let h = i32::from(map.height());
    (0..h)
        .flat_map(|y| (0..w).map(move |x| Pos::new(x, y)))
        .find(|&p| !s.state().units().iter().any(|u| u.pos == p) && s.special_terrain(p) == special)
        .unwrap_or_else(|| panic!("no tile with special = {special}"))
}

#[test]
fn resting_on_terrain_that_is_not_plain_ground_shows_the_terrain_tip_at_full_only() {
    use trpg_core::TutorialLevel;
    for (level, shown) in [
        (Some(TutorialLevel::Full), true),
        (Some(TutorialLevel::Short), false),
        (Some(TutorialLevel::None), false),
    ] {
        let mut c = tipped();
        c.game.campaign_tutorial = level;
        let mut s = quick();
        dismiss_all(&mut s, &mut c);
        s.cursor.jump(empty_tile(&s, false));
        assert_eq!(dismiss_all(&mut s, &mut c), [], "{level:?}");
        s.cursor.jump(empty_tile(&s, true));
        let tips = dismiss_all(&mut s, &mut c);
        let want = if shown {
            vec![TipTrigger::FirstTerrain]
        } else {
            vec![]
        };
        assert_eq!(tips, want, "{level:?}");
        // Once: the next tile like it shows nothing.
        s.cursor.jump(empty_tile(&s, false));
        step(&mut s, &mut c, &[]);
        s.cursor.jump(empty_tile(&s, true));
        assert_eq!(dismiss_all(&mut s, &mut c), [], "{level:?}");
    }
}

/// Ticket 0466: only at `Full`, once, and not under a skin that can't zoom
/// until one that can is on.
#[test]
fn the_zoom_tip_waits_for_a_skin_that_can_zoom_and_shows_at_full_only() {
    use trpg_core::TutorialLevel;
    for (level, shown) in [
        (Some(TutorialLevel::Full), true),
        (Some(TutorialLevel::Short), false),
        (Some(TutorialLevel::None), false),
    ] {
        let mut c = tipped();
        c.game.campaign_tutorial = level;
        let mut s = quick();
        let zoom = TipTrigger::ZoomAvailable;
        assert!(!dismiss_all(&mut s, &mut c).contains(&zoom), "{level:?}");
        let skin = crate::map_view::skin_named(&c.game.content, "sprite").unwrap();
        c.set_map_skin(skin);
        assert_eq!(
            dismiss_all(&mut s, &mut c).contains(&zoom),
            shown,
            "{level:?}"
        );
        assert!(!dismiss_all(&mut s, &mut c).contains(&zoom), "{level:?}");
    }
}

#[test]
fn plain_ground_is_cost_1_with_no_bonus() {
    let s = quick();
    let state = s.state();
    let map = &state.map().tiles;
    let w = i32::from(map.width());
    let h = i32::from(map.height());
    for p in (0..h).flat_map(|y| (0..w).map(move |x| Pos::new(x, y))) {
        let t = state.terrain().get(*map.get(p).unwrap()).unwrap();
        let plain = t.defense == 0
            && t.avoid == 0
            && t.heal_percent == 0
            && t.move_cost.iter().all(|&c| c == Some(1));
        assert_eq!(s.special_terrain(p), !plain, "{p:?} {}", t.name);
    }
    // Off the map: nothing.
    assert!(!s.special_terrain(Pos::new(-1, 0)));
}

/// Each bonus alone, or a cost other than 1 for one movement type, makes
/// terrain not plain ground.
#[test]
fn any_one_bonus_or_cost_makes_terrain_not_plain() {
    let plain = TerrainRules {
        name: "Plain".into(),
        move_cost: vec![Some(1), Some(1)],
        defense: 0,
        avoid: 0,
        heal_percent: 0,
    };
    assert!(!not_plain(&plain));
    let with = |edit: fn(&mut TerrainRules)| {
        let mut t = plain.clone();
        edit(&mut t);
        not_plain(&t)
    };
    assert!(with(|t| t.defense = 1));
    assert!(with(|t| t.avoid = 10));
    assert!(with(|t| t.heal_percent = 10));
    assert!(with(|t| t.move_cost[1] = Some(2)));
    assert!(with(|t| t.move_cost[0] = None));
}

// ---- Wading (ticket 0483) ------------------------------------------------------

/// The Quick Battle's archer, beside the test map's river at (2, 3), as
/// a unit of `class`.
fn archer_as(c: &Ctx, class: &str) -> BattleScreen {
    let s = edited(c, |units| {
        units[2].class = trpg_core::ClassId(class.to_owned());
    });
    assert_eq!(s.state().units()[2].pos, Pos::new(2, 4));
    s
}

/// A foot or flying unit beside a river could step into it; a mounted or
/// armored one can't.
#[test]
fn only_foot_and_flying_units_have_water_in_reach() {
    let c = ctx();
    let water = c.game.content.terrain.display.id_of(WATER).unwrap();
    for (class, wades) in [
        ("archer", true),
        ("flier", true),
        ("rider", false),
        ("guard", false),
    ] {
        let s = archer_as(&c, class);
        let id = s.state().units()[2].id;
        let sel = Selection::new(s.state(), id).unwrap();
        assert_eq!(
            doable::wades_in_reach(s.state(), &sel, water),
            wades,
            "{class}"
        );
    }
}

/// Selecting the archer shows the Wading tip, once; a mounted unit in
/// its place shows none.
#[test]
fn selecting_a_unit_that_could_wade_shows_the_wading_tip() {
    for (class, shown) in [("archer", true), ("rider", false)] {
        let mut c = tipped();
        let mut s = archer_as(&c, class);
        dismiss_all(&mut s, &mut c);
        step(&mut s, &mut c, &[Action::CursorLeft, Action::CursorUp]);
        step(&mut s, &mut c, &[Action::Confirm]);
        assert!(matches!(s.mode(), Mode::Selected(_)), "{class}");
        let seen = dismiss_all(&mut s, &mut c);
        assert_eq!(
            seen.contains(&TipTrigger::FirstWadeInReach),
            shown,
            "{class}: {seen:?}"
        );
        step(&mut s, &mut c, &[Action::Cancel]);
        step(&mut s, &mut c, &[Action::Confirm]);
        assert!(!dismiss_all(&mut s, &mut c).contains(&TipTrigger::FirstWadeInReach));
    }
}

// ---- Weapon kinds (ticket 0489) ------------------------------------------------

/// Gives `unit` a class of `class` and `weapon`, a `kind`, equipped.
fn arm(unit: &mut Unit, class: &str, kind: trpg_core::WeaponKind, weapon: &str) {
    unit.class = trpg_core::ClassId(class.to_owned());
    unit.weapon_ranks.insert(kind, trpg_core::WeaponRank::D);
    unit.loadout.weapons[0] = Some(trpg_core::WeaponInstance {
        def: trpg_core::ItemId::new(weapon),
        durability_left: 20,
    });
    unit.loadout.equipped = Some(trpg_core::Equipped::Weapon(0));
}

/// The skirmish (the lord at (6, 2), the raider at (7, 1) and a brigand at
/// (8, 2), the archer at (8, 4)) with `edit` applied to its units.
fn kinds(c: &Ctx, edit: impl FnOnce(&mut Vec<Unit>)) -> BattleScreen {
    let quick = quick_battle(&c.game.content).unwrap();
    let mut units = quick.units().to_vec();
    units[0].pos = Pos::new(6, 2);
    units[2].pos = Pos::new(8, 4);
    edit(&mut units);
    let rout = Objective::Rout { turn_limit: None };
    BattleScreen::new(battle_with(c, quick.map().clone(), units, rout))
}

/// The lord armed with an Iron Spear (as a Guard).
fn spear_lord(units: &mut [Unit]) {
    arm(
        &mut units[0],
        "guard",
        trpg_core::WeaponKind::Spear,
        "iron_spear",
    );
}

/// The raider made a Rider (Mounted) with an Iron Sword.
fn rider_raider(units: &mut [Unit]) {
    arm(
        &mut units[5],
        "rider",
        trpg_core::WeaponKind::Sword,
        "iron_sword",
    );
}

/// The lord steps right and attacks: the forecast opens on the raider.
/// Returns the tips that showed on the way, dismissed.
fn attack_raider(s: &mut BattleScreen, c: &mut Ctx) -> Vec<TipTrigger> {
    let mut seen = dismiss_all(s, c);
    step(s, c, &[Action::Confirm]);
    seen.extend(dismiss_all(s, c));
    step(s, c, &[Action::CursorRight, Action::Confirm]);
    frame(s, c, &[], 1.0);
    step(s, c, &[Action::Confirm]);
    let Mode::Targeting(t) = s.mode() else {
        panic!("no forecast: {:?}", s.mode());
    };
    assert_eq!(t.target(), s.state().units()[5].id);
    seen.extend(dismiss_all(s, c));
    seen
}

/// The notes of the forecast on screen.
fn forecast_notes(s: &BattleScreen, c: &Ctx) -> Vec<String> {
    s.view(c).forecast.expect("a forecast").notes
}

/// The forecast `s` shows, as its kinds' traits make it effective.
fn effects(s: &BattleScreen) -> Vec<forecast::KindEffect> {
    let Mode::Targeting(t) = s.mode() else {
        panic!("no forecast: {:?}", s.mode());
    };
    forecast::kind_effects(s.state(), t)
}

/// A spear on a Mounted unit, a bow on a Flying one: effective by its
/// kind. A spear on a foot unit, or a bow on a mounted one, isn't.
#[test]
fn a_kinds_trait_is_effective_on_its_tag_only() {
    use trpg_core::{UnitTag, WeaponKind};
    let c = ctx();
    let aimed = |s: &BattleScreen, unit: usize, target: usize| {
        let state = s.state();
        let mut sel = Selection::new(state, state.units()[unit].id).unwrap();
        sel.target = Some(state.units()[target].id);
        let t = Targeting::aimed(state, sel).expect("in reach");
        forecast::kind_effects(state, &t)
    };
    // The lord's spear on the raider (a Rider, then a Raider).
    let s = kinds(&c, |u| {
        spear_lord(u);
        u[0].pos = Pos::new(7, 2);
        rider_raider(u);
    });
    let spear = forecast::KindEffect {
        kind: WeaponKind::Spear,
        tag: UnitTag::Mounted,
        mult: 2,
    };
    assert_eq!(aimed(&s, 0, 5), [spear]);
    let s = kinds(&c, |u| {
        spear_lord(u);
        u[0].pos = Pos::new(7, 2);
    });
    assert_eq!(aimed(&s, 0, 5), []);
    // The archer's bow on the brigand two tiles up (a Flier, then a Rider).
    let flier = |class: &'static str| {
        kinds(&c, move |u| {
            u[3].class = trpg_core::ClassId(class.to_owned());
        })
    };
    let bow = forecast::KindEffect {
        kind: WeaponKind::Bow,
        tag: UnitTag::Flying,
        mult: 3,
    };
    assert_eq!(aimed(&flier("flier"), 2, 3), [bow]);
    assert_eq!(aimed(&flier("rider"), 2, 3), []);
}

/// Opening the forecast of a spear on a Rider shows the Weapon kinds tip,
/// once; the forecast says why the strike is effective.
#[test]
fn a_spear_on_a_rider_shows_the_weapon_kinds_tip_once() {
    let mut c = tipped();
    let mut s = kinds(&c, |u| {
        spear_lord(u);
        rider_raider(u);
    });
    let seen = attack_raider(&mut s, &mut c);
    assert!(seen.contains(&TipTrigger::FirstEffective), "{seen:?}");
    assert_eq!(forecast_notes(&s, &c), ["Spear ×2 vs Mounted"]);
    // Back out and in again: not twice.
    step(&mut s, &mut c, &[Action::Cancel]);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert!(matches!(s.mode(), Mode::Targeting(_)), "{:?}", s.mode());
    assert_eq!(dismiss_all(&mut s, &mut c), []);
    let tips = &c.game.content.tips;
    let tip = tips.for_trigger(TipTrigger::FirstEffective).unwrap();
    assert_eq!(tip.id, "weapon_kinds");
}

/// A Rider attacking a spear: the counter is effective, so it shows too.
#[test]
fn a_spear_countering_a_rider_shows_the_tip_too() {
    let mut c = tipped();
    let mut s = kinds(&c, |u| {
        arm(
            &mut u[0],
            "rider",
            trpg_core::WeaponKind::Sword,
            "iron_sword",
        );
        arm(
            &mut u[5],
            "guard",
            trpg_core::WeaponKind::Spear,
            "iron_spear",
        );
    });
    let seen = attack_raider(&mut s, &mut c);
    assert!(seen.contains(&TipTrigger::FirstEffective), "{seen:?}");
    assert_eq!(effects(&s).len(), 1);
    assert_eq!(forecast_notes(&s, &c), ["Spear ×2 vs Mounted"]);
}

/// A plain strike: no tip, no note.
#[test]
fn a_plain_strike_has_no_weapon_kinds_tip_or_note() {
    let mut c = tipped();
    let mut s = kinds(&c, |_| {});
    let seen = attack_raider(&mut s, &mut c);
    assert!(!seen.contains(&TipTrigger::FirstEffective), "{seen:?}");
    assert!(seen.contains(&TipTrigger::FirstForecast), "{seen:?}");
    assert_eq!(forecast_notes(&s, &c), [] as [&str; 0]);
}

/// At Short, no tip; the forecast's note still says why.
#[test]
fn at_short_the_note_shows_without_the_tip() {
    let mut c = tipped();
    c.game.campaign_tutorial = Some(trpg_core::TutorialLevel::Short);
    let mut s = kinds(&c, |u| {
        spear_lord(u);
        rider_raider(u);
    });
    assert_eq!(attack_raider(&mut s, &mut c), []);
    assert_eq!(effects(&s).len(), 1);
    assert_eq!(forecast_notes(&s, &c), ["Spear ×2 vs Mounted"]);
}

/// The forecast of the lord's spear on the Rider: the `!` on its strikes
/// and the note that says why.
#[test]
fn effective_spear_forecast_snapshot() {
    let mut c = ctx();
    let mut s = kinds(&c, |u| {
        spear_lord(u);
        rider_raider(u);
    });
    attack_raider(&mut s, &mut c);
    assert_snapshot!(snap(&render(&s, &c), &c, SIDE_PANEL));
}

/// Ticket 1292: two tips due at once come up one at a time. Closing the
/// first brings up the second at once, still in their moment (selecting),
/// with no press on the map between them.
#[test]
fn two_tips_due_at_once_come_up_one_after_the_other() {
    let mut c = tipped();
    let mut s = quick();
    dismiss_all(&mut s, &mut c);
    step(&mut s, &mut c, &[Action::Confirm]);
    step(&mut s, &mut c, &[]);
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstUnitSelected));
    // Confirm twice in one frame: the first closes it, and the second is
    // not the next tip's, which was not on screen yet (0495).
    step(&mut s, &mut c, &[Action::Confirm, Action::Confirm]);
    assert!(matches!(s.mode(), Mode::Selected(_)));
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstSkillUnit));
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstEnemyInRange));
}

/// Ticket 1292: a tip that fires (by an event, here a level-up) while
/// another tip is up waits for it in the same moment, and comes up as soon
/// as that one is closed.
#[test]
fn a_tip_behind_another_comes_up_as_it_closes() {
    let mut c = tipped();
    let mut s = quick();
    step(&mut s, &mut c, &[]);
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstBattleStart));
    s.guide.tips.fire(TipTrigger::FirstLevelUp);
    step(&mut s, &mut c, &[]);
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstBattleStart));
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstLevelUp));
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.shown_tip(), None);
}

/// Tickets 0495 and 1292: a tip that comes due while a screen the player
/// opened from the map menu (here Guides) is over the battle missed its
/// moment. Cancel closes the screen, Cancel again closes the menu, and no
/// tip comes up; still unseen, it shows the next time its trigger fires,
/// and only a press made while it is up closes it.
#[test]
fn a_tip_fired_under_a_screen_never_takes_the_menu_key() {
    let mut c = tipped();
    let mut s = quick();
    dismiss_all(&mut s, &mut c);
    let mut s = Stacked::new(s);
    s.press(&mut c, &[Action::Cancel]);
    assert!(matches!(s.mode(), Mode::MapMenu { .. }), "{:?}", s.mode());
    s.press(
        &mut c,
        &[Action::CursorDown, Action::CursorDown, Action::Confirm],
    );
    assert!(s.guides().is_some());
    s.battle_mut().guide.tips.fire(TipTrigger::FirstLowHp);
    s.frame(&mut c, &[], 0.0);
    assert_eq!(s.shown_tip(), None);
    // Back in the map menu: no tip.
    s.press(&mut c, &[Action::Cancel]);
    assert!(s.guides().is_none());
    s.frame(&mut c, &[], 0.0);
    assert!(matches!(s.mode(), Mode::MapMenu { .. }), "{:?}", s.mode());
    assert_eq!(s.shown_tip(), None);
    // Cancel closes the menu; still no tip, and it is unseen.
    s.press(&mut c, &[Action::Cancel]);
    assert!(matches!(s.mode(), Mode::Idle { .. }), "{:?}", s.mode());
    s.frame(&mut c, &[], 0.0);
    assert_eq!(s.shown_tip(), None);
    assert!(!seen(&c, "low_hp"), "dropped unseen");
    // Its trigger again: it comes up, and a key pressed in the frame it
    // first shows doesn't close it; a fresh Cancel does.
    s.battle_mut().guide.tips.fire(TipTrigger::FirstLowHp);
    s.press(&mut c, &[Action::Cancel]);
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstLowHp));
    s.press(&mut c, &[Action::Cancel]);
    assert_eq!(s.shown_tip(), None);
    assert!(matches!(s.mode(), Mode::Idle { .. }), "{:?}", s.mode());
    assert!(seen(&c, "low_hp"));
}

/// Acceptance (0494): the unseen tips, whose lines the guide may still
/// say, are every tip but those shown; none while tips are off.
#[test]
fn the_unseen_tips_are_those_not_shown_yet_and_none_while_tips_are_off() {
    let mut c = tipped();
    let mut s = quick();
    step(&mut s, &mut c, &[]);
    let shown = s
        .shown_tip()
        .and_then(|t| c.game.content.tips.for_trigger(t));
    let shown = shown.map_or_default(|t| t.id.clone());
    let unseen: Vec<String> = s.guide.tips.unseen(&c).map(|t| t.id.clone()).collect();
    assert_eq!(unseen.len(), c.game.content.tips.tips.len() - 1);
    assert!(!unseen.contains(&shown), "{shown} was shown");
    c.game.tips_enabled = false;
    step(&mut s, &mut c, &[]);
    assert_eq!(s.guide.tips.unseen(&c).count(), 0);
}

/// Ticket 0463: the lord's forecast with its two swords shows the
/// Swapping weapons tip, once; it names the keys that swap.
#[test]
fn a_forecast_with_a_weapon_to_swap_to_shows_the_swap_tip_once() {
    let mut c = tipped();
    let mut s = kinds(&c, |_| {});
    let seen = attack_raider(&mut s, &mut c);
    assert!(seen.contains(&TipTrigger::FirstWeaponSwap), "{seen:?}");
    let tip = c.game.content.tips.for_trigger(TipTrigger::FirstWeaponSwap);
    let tip = tip.unwrap();
    assert_eq!(tip.id, "weapon_swap");
    assert!(tip.how.as_deref().unwrap_or("").contains("{CursorLeft}"));
    // Back out and in again: not twice.
    step(&mut s, &mut c, &[Action::Cancel]);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert!(matches!(s.mode(), Mode::Targeting(_)), "{:?}", s.mode());
    assert_eq!(dismiss_all(&mut s, &mut c), []);
}

/// With one sword, nothing to swap: no tip (the forecast's own tip still
/// shows).
#[test]
fn a_forecast_with_one_weapon_has_no_swap_tip() {
    let mut c = tipped();
    let mut s = kinds(&c, |u| u[0].loadout.weapons[1] = None);
    let seen = attack_raider(&mut s, &mut c);
    assert!(!seen.contains(&TipTrigger::FirstWeaponSwap), "{seen:?}");
    assert!(seen.contains(&TipTrigger::FirstForecast), "{seen:?}");
}

/// Ticket 1291: the enemy phase's tip, which shows over its banner, still
/// waits while something busy is over the map (here the battle's notes).
#[test]
fn the_enemy_phase_tip_waits_for_the_notes() {
    let mut c = ctx();
    c.game.tips_enabled = true;
    let mut s = BattleScreen::new(quick_battle(&c.game.content).unwrap());
    s.guide.tips.fire(TipTrigger::FirstEnemyPhase);
    s.guide.tips.absorb(&mut c);
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstEnemyPhase));
    s.queue.push_front(Queued::Notes);
    assert!(s.notes_open());
    assert_eq!(s.shown_tip(), None);
}

/// The pages and questions of ticket 1312 her boxes step aside for: unit
/// info, the map menu, its unit list and the objective page.
fn pages(s: &BattleScreen, c: &Ctx) -> [(&'static str, Mode); 4] {
    let lord = UnitId(1);
    [
        (
            "info",
            Mode::Info {
                unit: lord,
                back: None,
            },
        ),
        (
            "map menu",
            Mode::MapMenu {
                menu: map_menu::map_menu(&s.state, map_menu::MapEntry::EndTurn),
            },
        ),
        (
            "unit list",
            Mode::UnitList {
                menu: map_menu::unit_list(&s.state, c.words()),
            },
        ),
        ("objective", Mode::Objective),
    ]
}

/// Closes the page open with Cancel, back to the map.
fn close_page(s: &mut BattleScreen, c: &mut Ctx) {
    for _ in 0..4 {
        if !s.mode().covers_guide() {
            return;
        }
        step(s, c, &[Action::Cancel]);
    }
    panic!("the page never closed: {:?}", s.mode());
}

/// Ticket 1312: her tip on screen steps aside while a page is open, takes
/// none of its keys, and is back, the same tip, once it closes.
#[test]
fn a_tip_steps_aside_for_a_page_and_comes_back() {
    for kind in 0..4 {
        let mut c = tipped();
        let mut s = quick();
        step(&mut s, &mut c, &[]);
        assert_eq!(s.shown_tip(), Some(TipTrigger::FirstBattleStart));
        let (name, page) = pages(&s, &c)[kind].clone();
        s.mode = page;
        step(&mut s, &mut c, &[]);
        let view = s.view(&c);
        assert_eq!(view.tip, None, "{name}");
        assert_eq!(view.lesson, None, "{name}");
        assert_eq!(s.shown_tip(), None, "{name}");
        if name == "info" {
            assert!(view.info.is_some(), "the info screen");
        }
        // Its keys reach the page: Cancel closes it, not the tip.
        close_page(&mut s, &mut c);
        assert_eq!(s.mode(), &Mode::default(), "{name}");
        assert_eq!(s.shown_tip(), Some(TipTrigger::FirstBattleStart), "{name}");
        assert_eq!(s.view(&c).tip.unwrap().heading, "Your move", "{name}");
    }
}

/// Ticket 1312: a tip that fires while a page is open waits for it to
/// close.
#[test]
fn a_tip_fired_on_a_page_waits_for_it_to_close() {
    for kind in 0..4 {
        let mut c = tipped();
        let mut s = quick();
        dismiss_all(&mut s, &mut c);
        let (name, page) = pages(&s, &c)[kind].clone();
        s.mode = page;
        step(&mut s, &mut c, &[]);
        s.guide.fire(TipTrigger::FirstLowHp);
        s.guide.tips.absorb(&mut c);
        step(&mut s, &mut c, &[]);
        assert_eq!(s.shown_tip(), None, "{name}");
        assert_eq!(s.view(&c).tip, None, "{name}");
        close_page(&mut s, &mut c);
        assert_eq!(s.shown_tip(), Some(TipTrigger::FirstLowHp), "{name}");
    }
}

/// Ticket 1312: the info screen opened by its key while a tip is queued
/// takes the info screen's keys (the next unit), and the Playdate's view
/// has no tip over it either.
#[test]
fn unit_info_takes_its_keys_with_a_tip_waiting() {
    let mut c = tipped();
    let mut s = quick();
    dismiss_all(&mut s, &mut c);
    step(&mut s, &mut c, &[Action::Info]);
    assert!(matches!(s.mode(), Mode::Info { .. }), "{:?}", s.mode());
    s.guide.fire(TipTrigger::FirstLowHp);
    s.guide.tips.absorb(&mut c);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert!(matches!(s.mode(), Mode::Info { .. }), "{:?}", s.mode());
    c.game.skin_set = SkinSet::Playdate;
    let view = s.view(&c);
    assert!(view.info.is_some());
    assert_eq!(view.tip, None);
    c.game.skin_set = SkinSet::Glyph;
    step(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(s.mode(), &Mode::default());
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstLowHp));
}
