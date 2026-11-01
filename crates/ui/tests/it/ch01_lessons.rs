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
use trpg_ui::screens::battle::mode::{MenuEntry, Mode, list_entry};
use trpg_ui::screens::battle::rewind::RewindScreen;

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
    let h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    start_chapter_1(h, level, without)
}

/// [`chapter_1`] with the tips on too.
fn chapter_1_with_tips(level: TutorialLevel) -> Harness {
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    h.with_tips();
    start_chapter_1(h, level, &[])
}

fn start_chapter_1(mut h: Harness, level: TutorialLevel, without: &[&str]) -> Harness {
    h.with_lessons();
    let lessons = &mut h.ctx_mut().game.content.lessons.lessons;
    lessons.retain(|l| !without.contains(&l.id.as_str()));
    h.keys("f f").keys(downs(level)).keys("f Up f");
    skip_scenes(&mut h);
    h.keys("f");
    skip_scenes(&mut h);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    h.wait(0.1);
    assert_eq!(h.cursor_tile(), Some(LEAD));
    let campaign = h
        .flow()
        .and_then(|f| f.campaign_now(h.game().ctx()))
        .map(|c| c.tutorial);
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
    let campaign = h.flow().and_then(|f| f.campaign_now(h.game().ctx()));
    let campaign = campaign.unwrap_or_else(|| panic!("no campaign"));
    campaign.lessons_done.iter().cloned().collect()
}

/// Moves the cursor from where the map shows it to `to` with the arrow keys.
fn go(h: &mut Harness, to: Pos) {
    let at = h.cursor_tile();
    let at = at.unwrap_or_else(|| {
        panic!(
            "no cursor: {:?}
{}",
            h.screens(),
            h.snapshot()
        )
    });
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
        Mode::ActionMenu { menu, .. } => menu.value(menu.focus()).copied(),
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
    let rewind = h.screen::<RewindScreen>().expect("the rewind screen");
    assert_eq!(battle(&h).lesson_ticks(), [true, false]);
    // The box stays up over the rewind screen.
    assert!(rewind.view(h.game().ctx()).lesson.is_some());
    h.keys("d");
    assert!(h.screen::<RewindScreen>().is_none());
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
    assert!(matches!(
        battle(&h).mode(),
        Mode::List { menu, .. } if list_entry(menu) == MenuEntry::Arts
    ));
    assert!(done(&h).contains(&"combat_arts".to_owned()));
}

/// Whether End turn is refused: pressed on the map, the prompt doesn't
/// open (it is backed out of if it does).
fn end_turn_refused(h: &mut Harness) -> bool {
    h.keys("Space");
    let refused = matches!(battle(h).mode(), Mode::Idle { .. });
    if !refused {
        h.keys("d");
    }
    refused
}

/// Ticket 0464, Nick's playtest at `Full`: the Sergeant, the one unit who
/// reaches an enemy on turn 1, aims at the Brigand (Attacking opens), backs
/// out to his menu (it has Arts: Combat Arts waits behind Attacking) and
/// attacks. Nobody left can use an art, so Combat Arts doesn't open, and
/// End turn isn't refused for it. It opens on a later turn when a menu
/// with Arts does.
#[test]
fn the_arts_lesson_doesnt_open_after_the_only_unit_with_arts_attacks() {
    let mut h = chapter_1(
        TutorialLevel::Full,
        &[
            "cursor",
            "keys",
            "next_unit",
            "moving",
            "threat_arrows",
            "terrain_magic",
        ],
    );
    go(&mut h, SERGEANT);
    h.keys("f");
    assert_eq!(shown(&h), Some("attacking"));
    go(&mut h, BRIGAND);
    h.keys("f").wait(1.5);
    assert!(matches!(battle(&h).mode(), Mode::Targeting(_)));
    h.keys("d");
    assert!(matches!(battle(&h).mode(), Mode::ActionMenu { .. }));
    pick(&mut h, MenuEntry::Attack);
    h.wait(1.0);
    assert!(matches!(battle(&h).mode(), Mode::Targeting(_)));
    h.keys("f").hold("f", 4.0);
    assert!(done(&h).contains(&"attacking".to_owned()));
    assert_eq!(battle(&h).state().phase(), Phase::Player);
    assert!(!battle(&h).ready_units().is_empty(), "units left to act");
    assert_eq!(shown(&h), None);
    assert!(!battle(&h).lesson_open());
    assert!(!end_turn_refused(&mut h));
    assert!(!done(&h).contains(&"combat_arts".to_owned()));
}

/// Ticket 0464: the Attacking lesson open, the Sergeant waits instead: no
/// unit left reaches an enemy, so it is put away at once, not done, and
/// the turn may end with units still to act.
#[test]
fn the_attacking_lesson_is_put_away_when_nobody_left_can_reach_an_enemy() {
    let mut h = chapter_1(
        TutorialLevel::Full,
        &[
            "cursor",
            "keys",
            "next_unit",
            "moving",
            "threat_arrows",
            "combat_arts",
            "terrain_magic",
        ],
    );
    go(&mut h, SERGEANT);
    h.keys("f");
    assert_eq!(shown(&h), Some("attacking"));
    h.keys("f").wait(0.5);
    pick(&mut h, MenuEntry::Wait);
    h.wait(0.5);
    assert!(!battle(&h).ready_units().is_empty(), "units left to act");
    assert_eq!(shown(&h), None);
    assert!(!battle(&h).lesson_open());
    assert!(!done(&h).contains(&"attacking".to_owned()));
    assert!(!end_turn_refused(&mut h));
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
    assert!(matches!(
        battle(&h).mode(),
        Mode::List { menu, .. } if list_entry(menu) == MenuEntry::Magic
    ));
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
            assert_eq!(done(&h), [] as [std::string::String; 0]);
        } else {
            assert_eq!(seen.first().map(String::as_str), Some("keys"));
            assert_eq!(seen.last().map(String::as_str), Some("rewind"));
        }
    }
}

/// Opens Options from the map menu, moves the `Tutorial` row (three up
/// from the first) by `steps` (`Right` toward None), keeps it and closes
/// the map menu.
fn set_tutorial(h: &mut Harness, steps: &str) {
    h.keys("d Down Down Down f");
    assert_eq!(h.top_screen(), "options");
    h.keys("Up Up Up").keys(steps).keys("d Left Left f");
    assert_eq!(h.top_screen(), "battle");
    h.keys("d").wait(0.1);
    assert!(matches!(battle(h).mode(), Mode::Idle { .. }));
}

fn tip(h: &Harness) -> Option<trpg_content::TipTrigger> {
    battle(h).shown_tip()
}

/// Ticket 0841: set to None mid-battle, with the cursor lesson open,
/// three more waiting and a tip queued behind it, nothing shows again
/// for the rest of the battle, and End turn is never refused.
#[test]
fn the_tutorial_set_to_none_stops_the_lessons_and_tips_already_waiting() {
    let mut h = chapter_1_with_tips(TutorialLevel::Full);
    // The start tip comes first, then the cursor lesson opens.
    assert!(tip(&h).is_some());
    h.keys("f").wait(0.1);
    assert_eq!(shown(&h), Some("cursor"));
    assert!(battle(&h).lesson_open());
    // Selecting the lead queues its tip behind the open lesson.
    h.keys("f").wait(0.1);
    assert_eq!(tip(&h), None);
    h.keys("d").wait(0.1);
    set_tutorial(&mut h, "Right Right");
    let campaign = h
        .flow()
        .and_then(|f| f.campaign_now(h.game().ctx()))
        .map(|c| c.tutorial);
    assert_eq!(campaign, Some(TutorialLevel::None));
    let quiet = |h: &Harness| {
        assert_eq!(shown(h), None);
        assert!(!battle(h).lesson_open());
        assert_eq!(tip(h), None);
        assert_eq!(battle(h).tip_view(h.game().ctx()), None);
    };
    quiet(&h);
    // Closed, not done: it can come back if the level goes back up.
    assert_eq!(done(&h), [] as [std::string::String; 0]);
    go(&mut h, SERGEANT);
    h.keys("f").wait(0.1);
    quiet(&h);
    go(&mut h, BRIGAND);
    quiet(&h);
    h.keys("d").wait(0.1);
    // End turn is not refused: the prompt opens.
    h.keys("Space");
    assert!(
        !matches!(battle(&h).mode(), Mode::Idle { .. }),
        "End turn refused"
    );
    h.keys("f");
    to_turn(&mut h, 2);
    quiet(&h);
    assert_eq!(done(&h), [] as [std::string::String; 0]);
}

/// Full → Short drops the `Full`-only lessons waiting (the cursor one
/// open) and the tips; the `Short` keys lesson comes next.
#[test]
fn the_tutorial_set_to_short_drops_the_full_only_lessons_and_tips() {
    let mut h = chapter_1_with_tips(TutorialLevel::Full);
    h.keys("f").wait(0.1);
    assert_eq!(shown(&h), Some("cursor"));
    h.keys("f").wait(0.1);
    h.keys("d").wait(0.1);
    set_tutorial(&mut h, "Right");
    assert_eq!(tip(&h), None);
    assert_eq!(shown(&h), Some("keys"));
    h.keys("Right Left");
    visit_options(&mut h);
    assert_eq!(done(&h), ["keys"]);
    let mut seen = Vec::new();
    for _ in 0..3 {
        seen.extend(shown(&h).map(str::to_owned));
        assert_eq!(tip(&h), None);
        h.keys("s").wait(0.1);
    }
    let basics: Vec<&String> = seen
        .iter()
        .filter(|id| BASICS.contains(&id.as_str()))
        .collect();
    assert!(basics.is_empty(), "{basics:?}");
}

/// Whether the guide's box at the end of the lessons is up (0465).
fn end_box(h: &Harness) -> bool {
    battle(h).lessons_end_shown(h.game().ctx())
}

/// Backs out to the map from a unit's list or forecast, and has it wait.
fn back_and_wait(h: &mut Harness) {
    for _ in 0..4 {
        if menu_entry(h).is_some() {
            pick(h, MenuEntry::Wait);
            h.wait(0.5);
            return;
        }
        h.keys("d").wait(0.1);
    }
    panic!("no action menu: {:?}", battle(h).mode());
}

/// Every lesson `Short` shows, played on Chapter 1 from turn 1, the last
/// (Rewind) on turn 2.
fn finish_short(h: &mut Harness) {
    assert_eq!(shown(h), Some("keys"));
    h.keys("Right Left");
    visit_options(h);
    go(h, SERGEANT);
    assert_eq!(shown(h), Some("threat_arrows"));
    h.keys("f Left Left Down d").wait(0.1);
    go(h, SERGEANT);
    h.keys("f");
    go(h, BRIGAND);
    h.keys("f").wait(1.5);
    h.keys("d");
    assert_eq!(shown(h), Some("combat_arts"));
    pick(h, MenuEntry::Arts);
    back_and_wait(h);
    go(h, HERETIC);
    h.keys("f");
    go(h, p(3, 10));
    h.keys("f").wait(1.5);
    pick(h, MenuEntry::Magic);
    assert_eq!(shown(h), Some("terrain_magic"));
    h.keys("f").wait(0.1);
    back_and_wait(h);
    assert!(!end_box(h), "Rewind is left");
    wait_all(h);
    h.wait(0.5);
    end_turn(h);
    to_turn(h, 2);
    assert_eq!(shown(h), Some("rewind"));
    h.keys("r d").wait(0.1);
}

/// Closes what stands before turn 1's map after a restart.
fn back_on_turn_1(h: &mut Harness) {
    skip_scenes(h);
    h.wait(0.1);
    while h.top_screen() == "dialogue" || battle(h).banner().is_some() || battle(h).notes_open() {
        if h.top_screen() == "dialogue" {
            skip_scenes(h);
        } else {
            h.keys("f");
        }
        h.wait(0.1);
    }
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert_eq!(battle(h).state().turn(), 1);
    let charges = battle(h).state().rewind_charges();
    assert_eq!(battle(h).history().charges_left(), charges, "none spent");
}

/// Acceptance, `Short`: the box comes as the last lesson closes, says where
/// Guides is with the player's key, starts on "Keep going", takes the keys
/// while it is up; "Keep going" closes it for good and the battle goes on.
#[test]
fn the_box_comes_after_the_last_lesson_and_keep_going_closes_it() {
    let mut h = chapter_1(TutorialLevel::Short, &[]);
    finish_short(&mut h);
    assert!(end_box(&h), "{}", h.snapshot());
    insta::assert_snapshot!(h.snapshot());
    let view = battle(&h).view(h.game().ctx()).lessons_end.unwrap();
    assert_eq!(view.focus, 0);
    assert_eq!(
        view.lines[1],
        "Need a reminder? Guides, in the map menu (d)."
    );
    // It holds the keys: the map doesn't move, Cancel and End turn do nothing.
    let at = h.cursor_tile();
    h.keys("Left d Space");
    assert_eq!(h.cursor_tile(), at);
    assert!(end_box(&h) && matches!(battle(&h).mode(), Mode::Idle { .. }));
    // Round the answers and back to the first.
    h.keys("Up");
    let focus = |h: &Harness| {
        let view = battle(h).view(h.game().ctx()).lessons_end;
        view.map(|v| v.focus)
    };
    assert_eq!(focus(&h), Some(2));
    h.keys("Down");
    assert_eq!(focus(&h), Some(0));
    h.keys("f");
    assert!(!end_box(&h));
    assert_eq!(battle(&h).state().turn(), 2);
    h.keys("Left");
    assert_ne!(h.cursor_tile(), at, "the map is the player's again");
}

/// Acceptance: "Restart the battle" is turn 1 with every charge and the
/// lessons done; the box doesn't come back.
#[test]
fn restart_the_battle_goes_back_to_turn_one_with_the_lessons_done() {
    let mut h = chapter_1(TutorialLevel::Short, &[]);
    finish_short(&mut h);
    h.keys("Down f");
    back_on_turn_1(&mut h);
    assert_eq!(shown(&h), None);
    assert!(!end_box(&h));
    assert!(done(&h).contains(&"keys".to_owned()));
    assert!(done(&h).contains(&"rewind".to_owned()));
}

/// Acceptance: "Restart with the lessons" is turn 1 with every charge and
/// the lessons again, each opening at its moment; finishing them again
/// doesn't ask again (once per campaign). The campaign's saves see it.
#[test]
fn restart_with_the_lessons_shows_them_again_and_does_not_ask_twice() {
    let mut h = chapter_1(TutorialLevel::Short, &[]);
    finish_short(&mut h);
    h.keys("Down Down f");
    back_on_turn_1(&mut h);
    assert!(!done(&h).contains(&"keys".to_owned()));
    assert!(!done(&h).contains(&"rewind".to_owned()));
    finish_short(&mut h);
    assert!(done(&h).contains(&"rewind".to_owned()));
    assert!(!end_box(&h));
}

/// Acceptance, `Full`: the basic controls count too. Without the lessons
/// that wait for their chance, the last is Rewind on turn 2, after Ending
/// the turn.
#[test]
fn at_full_the_box_waits_for_the_basic_controls_too() {
    let mut h = chapter_1(
        TutorialLevel::Full,
        &["threat_arrows", "combat_arts", "terrain_magic"],
    );
    openers(&mut h);
    assert_eq!(shown(&h), Some("moving"));
    h.keys("f f").wait(0.5);
    pick(&mut h, MenuEntry::Wait);
    h.wait(0.5);
    go(&mut h, SERGEANT);
    h.keys("f");
    assert_eq!(shown(&h), Some("attacking"));
    go(&mut h, BRIGAND);
    h.keys("f").wait(1.5);
    h.keys("f").hold("f", 4.0);
    assert!(done(&h).contains(&"attacking".to_owned()));
    assert!(!end_box(&h));
    wait_all(&mut h);
    h.wait(0.5);
    assert_eq!(shown(&h), Some("ending_turn"));
    end_turn(&mut h);
    assert!(!end_box(&h));
    to_turn(&mut h, 2);
    assert!(!end_box(&h), "Rewind is left");
    h.keys("r d").wait(0.1);
    assert!(end_box(&h), "{}", h.snapshot());
}

/// Acceptance: at `None` the box never comes.
#[test]
fn at_none_the_box_never_comes() {
    let mut h = chapter_1(TutorialLevel::None, &[]);
    assert!(!end_box(&h));
    end_turn(&mut h);
    to_turn(&mut h, 2);
    assert!(!end_box(&h));
}
