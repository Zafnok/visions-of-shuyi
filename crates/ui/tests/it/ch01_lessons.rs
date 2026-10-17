//! The lessons (ticket 0457, `assets/data/lessons.ron`) played on the
//! shipped Chapter 1 from New Game: each opens at its moment at the level
//! it belongs to, its items are done (in another order than the list's
//! where they can be), and it closes; the campaign keeps it. The basic
//! controls show only at `Full`, nothing at `None`. Rewind's lesson spends
//! no charge.

use trpg_core::{Phase, Pos, TutorialLevel};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::screens::battle::BattleScreen;
use trpg_ui::screens::battle::mode::{MenuEntry, Mode};

fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

/// The basic-controls lessons (`Full` only).
const BASICS: [&str; 5] = ["cursor", "next_unit", "moving", "attacking", "ending_turn"];

// Chapter 1's party where it starts (`assets/battles/ch01.ron`).
const LEAD: Pos = Pos::new(4, 12);
/// The Sergeant, a Rider: the one who can reach an enemy on turn 1.
const SERGEANT: Pos = Pos::new(6, 11);
const HERETIC: Pos = Pos::new(4, 13);
/// The Brigand on the road, the nearest enemy.
const BRIGAND: Pos = Pos::new(10, 10);

/// The familiarity answer's rows below "Not familiar".
fn downs(level: TutorialLevel) -> &'static str {
    match level {
        TutorialLevel::Full => "",
        TutorialLevel::Short => "Down",
        TutorialLevel::None => "Down Down",
    }
}

/// Skips every scene on screen: Cancel, then Confirm on "Skip scene?".
fn skip_scenes(h: &mut Harness) {
    for _ in 0..8 {
        if h.top_screen() != "dialogue" {
            return;
        }
        h.keys("d f");
    }
    panic!("the scenes never end: {:?}", h.screens());
}

/// Chapter 1 from New Game in Classic at tutorial `level`, with the lessons
/// on (as the game has them) but those of `without`, so a test reaches the
/// one it is about: turn 1, its scenes skipped and its banner closed, the
/// cursor on the lead.
fn chapter_1(level: TutorialLevel, without: &[&str]) -> Harness {
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    h.with_lessons();
    let lessons = &mut h.ctx_mut().game.content.lessons.lessons;
    lessons.retain(|l| !without.contains(&l.id.as_str()));
    h.keys("f f").keys(downs(level)).keys("f Up f");
    skip_scenes(&mut h);
    h.keys("f");
    skip_scenes(&mut h);
    assert_eq!(h.screens(), ["title", "battle"]);
    h.wait(0.1);
    assert_eq!(h.cursor_tile(), Some(LEAD));
    let campaign = h.flow().and_then(|f| f.campaign()).map(|c| c.tutorial);
    assert_eq!(campaign, Some(level));
    h
}

fn battle(h: &Harness) -> &BattleScreen {
    h.battle().unwrap_or_else(|| panic!("no battle"))
}

fn shown(h: &Harness) -> Option<&str> {
    battle(h).shown_lesson()
}

fn done(h: &Harness) -> Vec<String> {
    let campaign = h.flow().and_then(|f| f.campaign());
    let campaign = campaign.unwrap_or_else(|| panic!("no campaign"));
    campaign.lessons_done.iter().cloned().collect()
}

/// Moves the cursor from where the map shows it to `to` with the arrow keys.
fn go(h: &mut Harness, to: Pos) {
    let at = h.cursor_tile().unwrap_or_else(|| panic!("no cursor"));
    let (dx, dy) = (to.x - at.x, to.y - at.y);
    for _ in 0..dx.abs() {
        h.keys(if dx < 0 { "Left" } else { "Right" });
    }
    for _ in 0..dy.abs() {
        h.keys(if dy < 0 { "Up" } else { "Down" });
    }
}

/// The entry the open action menu is on, if one is open.
fn menu_entry(h: &Harness) -> Option<MenuEntry> {
    match battle(h).mode() {
        Mode::ActionMenu { menu, entries, .. } => entries.get(menu.focus()).copied(),
        _ => None,
    }
}

/// In the open action menu, moves to `entry` and picks it.
fn pick(h: &mut Harness, entry: MenuEntry) {
    for _ in 0..10 {
        if menu_entry(h) == Some(entry) {
            h.keys("f").wait(0.1);
            return;
        }
        h.keys("Up");
    }
    panic!("no {entry:?} in the menu: {:?}", battle(h).mode());
}

/// Selects the unit on `at`, keeps it there and makes it wait.
fn wait_unit(h: &mut Harness, at: Pos) {
    go(h, at);
    h.keys("f f").wait(0.5);
    pick(h, MenuEntry::Wait);
    h.wait(0.5);
}

/// Every unit of the player's that hasn't acted waits where it stands.
fn wait_all(h: &mut Harness) {
    while let Some(&at) = battle(h).ready_units().first() {
        wait_unit(h, at);
    }
}

/// Plays on to the player phase of turn `turn`: the enemy's phase at the
/// held speed, its banners closed.
fn to_turn(h: &mut Harness, turn: u32) {
    for _ in 0..200 {
        let state = battle(h).state();
        if state.turn() == turn && state.phase() == Phase::Player && battle(h).banner().is_none() {
            h.wait(0.1);
            return;
        }
        if battle(h).banner().is_some() {
            h.keys("f");
        } else {
            h.hold("f", 0.5);
        }
    }
    panic!("turn {turn} never came");
}

/// Ends the turn: End turn, then yes.
fn end_turn(h: &mut Harness) {
    h.keys("Space f");
}

/// Opens Options from the map menu and goes back to the map.
fn visit_options(h: &mut Harness) {
    h.keys("d Down Down Down f");
    assert_eq!(h.top_screen(), "options");
    h.keys("d");
    assert_eq!(h.top_screen(), "battle");
    // Back in the map menu: closed.
    h.keys("d");
    assert!(matches!(battle(h).mode(), Mode::Idle { .. }));
}

/// The `Full` openers of turn 1, done: the cursor, the keys, the next
/// unit and a unit's details. The cursor ends on the lead.
fn openers(h: &mut Harness) {
    assert_eq!(shown(h), Some("cursor"));
    h.keys("Up Down Left Right");
    assert_eq!(shown(h), Some("keys"));
    h.keys("Right Left");
    visit_options(h);
    assert_eq!(shown(h), Some("next_unit"));
    h.keys("s");
    go(h, LEAD);
    h.keys("e d");
    assert_eq!(done(h), ["cursor", "keys", "next_unit"]);
}

/// Lesson 1, `Full`: opens on turn 1, the four directions in another
/// order close it; until then the turn can't end.
#[test]
fn the_cursor_lesson_opens_on_turn_one_and_closes_on_the_four_directions() {
    let mut h = chapter_1(TutorialLevel::Full, &[]);
    assert_eq!(shown(&h), Some("cursor"));
    assert!(h.snapshot().contains("Moving the cursor"));
    h.keys("Right Left Down");
    assert_eq!(battle(&h).lesson_ticks(), [false, true, true, true]);
    // End turn: refused, and no prompt.
    h.keys("Space");
    assert_eq!(battle(&h).state().phase(), Phase::Player, "the turn waits");
    assert!(matches!(battle(&h).mode(), Mode::Idle { .. }));
    h.keys("Up");
    assert_eq!(done(&h), ["cursor"]);
    // Lesson 6 runs after it at Full.
    assert_eq!(shown(&h), Some("keys"));
}

/// Lesson 6, `Short` (so the first at Short): Options first, then the
/// cursor; the keys in its text are the player's.
#[test]
fn the_keys_lesson_opens_first_at_short_and_closes_after_options() {
    let mut h = chapter_1(TutorialLevel::Short, &[]);
    assert_eq!(shown(&h), Some("keys"));
    let snap = h.snapshot();
    assert!(snap.contains("Played on the keys"), "{snap}");
    assert!(snap.contains("Open Options from the map menu: d"), "{snap}");
    visit_options(&mut h);
    assert_eq!(battle(&h).lesson_ticks(), [false, true]);
    h.keys("Up");
    assert_eq!(done(&h), ["keys"]);
}

/// Lesson 4, `Full`: details first, then the next unit.
#[test]
fn the_next_unit_lesson_closes_on_details_and_next_unit() {
    let mut h = chapter_1(TutorialLevel::Full, &["cursor", "keys"]);
    assert_eq!(shown(&h), Some("next_unit"));
    h.keys("e");
    assert_eq!(battle(&h).lesson_ticks(), [false, true]);
    h.keys("d s");
    assert!(done(&h).contains(&"next_unit".to_owned()));
}

/// Lesson 2, `Full`: select, move, the menu.
#[test]
fn the_moving_lesson_closes_when_a_unit_moves_and_its_menu_opens() {
    let mut h = chapter_1(TutorialLevel::Full, &[]);
    openers(&mut h);
    assert_eq!(shown(&h), Some("moving"));
    h.keys("f");
    assert_eq!(battle(&h).lesson_ticks(), [true, false, false]);
    h.keys("Up f").wait(1.0);
    assert!(matches!(battle(&h).mode(), Mode::ActionMenu { .. }));
    assert!(done(&h).contains(&"moving".to_owned()));
}

/// Lesson 3, `Full`: the first time a selected unit could reach an enemy
/// (the Sergeant, on turn 1): the forecast, then the attack.
#[test]
fn the_attacking_lesson_opens_when_an_enemy_is_in_reach_and_closes_on_an_attack() {
    let mut h = chapter_1(
        TutorialLevel::Full,
        &["cursor", "keys", "next_unit", "moving", "threat_arrows"],
    );
    assert_eq!(shown(&h), None);
    go(&mut h, SERGEANT);
    h.keys("f");
    assert_eq!(shown(&h), Some("attacking"));
    go(&mut h, BRIGAND);
    h.keys("f").wait(1.5);
    assert_eq!(battle(&h).lesson_ticks(), [true, false]);
    h.keys("f").hold("f", 4.0);
    assert!(done(&h).contains(&"attacking".to_owned()));
}

/// Lesson 5, `Full`: when no unit is left to act on turn 1, before
/// auto-end (on here) would end the turn: it waits for the player to.
#[test]
fn the_ending_turn_lesson_opens_when_every_unit_has_acted_and_auto_end_waits() {
    let mut h = chapter_1(TutorialLevel::Full, &[]);
    openers(&mut h);
    // Auto-end on.
    h.keys("Shift+Space");
    assert!(battle(&h).auto_end());
    // The moving lesson on the way; selecting the Sergeant starts the
    // attacking lesson, which can't be done once everyone has waited.
    wait_all(&mut h);
    h.wait(1.0);
    assert_eq!(battle(&h).state().phase(), Phase::Player);
    assert_eq!(shown(&h), Some("ending_turn"));
    assert!(done(&h).contains(&"moving".to_owned()));
    assert!(!done(&h).contains(&"attacking".to_owned()), "put away");
    end_turn(&mut h);
    assert_ne!(battle(&h).state().phase(), Phase::Player);
    assert!(done(&h).contains(&"ending_turn".to_owned()));
}

/// Lesson 7, `Short`: turn 2's player phase; opening and closing the
/// rewind screen does it, and spends no charge.
#[test]
fn the_rewind_lesson_opens_on_turn_two_and_spends_no_charge() {
    let mut h = chapter_1(
        TutorialLevel::Short,
        &["keys", "threat_arrows", "combat_arts", "terrain_magic"],
    );
    end_turn(&mut h);
    to_turn(&mut h, 2);
    let charges = battle(&h).state().rewind_charges();
    assert_eq!(shown(&h), Some("rewind"));
    // End turn: refused, and no prompt.
    h.keys("Space");
    assert_eq!(battle(&h).state().phase(), Phase::Player, "the turn waits");
    assert!(matches!(battle(&h).mode(), Mode::Idle { .. }));
    h.keys("r");
    assert!(battle(&h).rewind().is_some());
    assert_eq!(battle(&h).lesson_ticks(), [true, false]);
    // The box stays up over the rewind screen.
    assert_eq!(shown(&h), Some("rewind"));
    h.keys("d");
    assert!(battle(&h).rewind().is_none());
    assert!(done(&h).contains(&"rewind".to_owned()));
    assert_eq!(battle(&h).state().rewind_charges(), charges);
    assert_eq!(battle(&h).state().turn(), 2);
}

/// Lesson 8, `Short`: the first time the arrows show (the Sergeant on
/// his own tile, whom the road's Brigand reaches); a tile with none, then
/// one with arrows.
#[test]
fn the_threat_arrows_lesson_opens_when_they_show_and_closes_on_both_kinds_of_tile() {
    let mut h = chapter_1(TutorialLevel::Short, &["keys"]);
    assert_eq!(shown(&h), None, "nobody reaches the lead");
    go(&mut h, SERGEANT);
    assert_eq!(shown(&h), Some("threat_arrows"));
    // Selected: his own tile has arrows. Left, out of the Brigand's reach.
    h.keys("f");
    let ticks = battle(&h).lesson_ticks().to_vec();
    assert_eq!(ticks, [true, false]);
    h.keys("Left Left Down");
    assert!(done(&h).contains(&"threat_arrows".to_owned()));
}

/// Lesson 9, `Short`: the first time a unit's menu has Arts it can use
/// (the Sergeant, aimed at the Brigand, back from the forecast to his
/// menu); opening them does it.
#[test]
fn the_combat_arts_lesson_opens_with_usable_arts_and_closes_when_they_open() {
    let mut h = chapter_1(TutorialLevel::Short, &["keys", "threat_arrows"]);
    go(&mut h, SERGEANT);
    h.keys("f");
    go(&mut h, BRIGAND);
    h.keys("f").wait(1.5);
    assert!(matches!(battle(&h).mode(), Mode::Targeting(_)));
    h.keys("d");
    assert!(matches!(battle(&h).mode(), Mode::ActionMenu { .. }));
    assert_eq!(shown(&h), Some("combat_arts"));
    pick(&mut h, MenuEntry::Arts);
    assert!(matches!(battle(&h).mode(), Mode::ArtsMenu { .. }));
    assert!(done(&h).contains(&"combat_arts".to_owned()));
}

/// Lesson 10, `Short`: Chapter 1 can show it on turn 1. The Heretic
/// beside the western forest opens her spells; aiming one at the forest
/// does it.
#[test]
fn the_terrain_magic_lesson_opens_with_a_spell_for_a_tile_and_closes_when_one_is_aimed() {
    let mut h = chapter_1(TutorialLevel::Short, &["keys", "threat_arrows"]);
    go(&mut h, HERETIC);
    h.keys("f");
    go(&mut h, p(3, 10));
    h.keys("f").wait(1.5);
    pick(&mut h, MenuEntry::Magic);
    assert_eq!(shown(&h), Some("terrain_magic"));
    assert!(matches!(battle(&h).mode(), Mode::SpellMenu { .. }));
    h.keys("f").wait(0.1);
    assert!(done(&h).contains(&"terrain_magic".to_owned()));
}

/// At `Short` none of the basic controls opens through turn 1 (selecting,
/// moving, an enemy in reach, everyone done) and the turn ends; at `None`
/// no lesson opens at all, into turn 2.
#[test]
fn short_shows_no_basic_controls_and_none_shows_nothing() {
    for level in [TutorialLevel::Short, TutorialLevel::None] {
        // The lessons that need the right moment have their own tests.
        let mut h = chapter_1(level, &["threat_arrows", "combat_arts", "terrain_magic"]);
        let mut seen = Vec::new();
        let look = |h: &Harness, seen: &mut Vec<String>| seen.extend(shown(h).map(str::to_owned));
        look(&h, &mut seen);
        if level == TutorialLevel::Short {
            h.keys("Right Left");
            visit_options(&mut h);
        }
        go(&mut h, SERGEANT);
        h.keys("f");
        look(&h, &mut seen);
        go(&mut h, BRIGAND);
        look(&h, &mut seen);
        h.keys("d");
        wait_all(&mut h);
        h.wait(1.0);
        look(&h, &mut seen);
        end_turn(&mut h);
        to_turn(&mut h, 2);
        look(&h, &mut seen);
        let basics: Vec<&String> = seen
            .iter()
            .filter(|id| BASICS.contains(&id.as_str()))
            .collect();
        assert!(basics.is_empty(), "{level:?}: {basics:?}");
        if level == TutorialLevel::None {
            assert!(seen.is_empty(), "{seen:?}");
            assert!(done(&h).is_empty());
        } else {
            assert_eq!(seen.first().map(String::as_str), Some("keys"));
            assert_eq!(seen.last().map(String::as_str), Some("rewind"));
        }
    }
}
