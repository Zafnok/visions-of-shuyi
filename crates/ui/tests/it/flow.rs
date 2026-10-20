//! Scripted tests of the game flow (ticket 0801) through the real game:
//! New Game → mode → familiarity → lead → the test chapter (`assets-test/chapters/test.ron`:
//! an intro scene, a battle where the lead seizes the fort at (5, 5)
//! within 3 turns, the results (0810), a victory scene) → "Save your
//! progress?" (declined here; `save.rs` saves) → "To be continued" →
//! title. Battles are won and lost with scripted commands.

use insta::assert_snapshot;
use trpg_content::TipTrigger;
use trpg_core::{
    BattleState, Command, GameMode, LeadGender, LeadProfile, Outcome, Pos, TutorialLevel,
    UnitAction, UnitId,
};
use trpg_ui::ShuyiCtx;
use trpg_ui::audio::{AudioRequest, pick_from_pool};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::widgets::menu::LabelText as _;

/// The lead's unit in the test battle (its first slot).
const LEAD: UnitId = UnitId(1);
/// The seize tile.
const FORT: Pos = Pos::new(5, 5);

fn title() -> Harness {
    Harness::with_layout(RIGHT_HANDED)
}

/// From the title: New Game, the mode (`Down` for Casual), "Not familiar",
/// then the lead screen's `keys`, which must end on Start.
fn new_game(casual: bool, lead_keys: &str) -> Harness {
    let mut h = title();
    h.keys("f");
    assert_eq!(h.screens(), ["title", "game_flow", "mode_select"]);
    h.keys(if casual { "Down f" } else { "f" });
    assert_eq!(h.top_screen(), "familiarity");
    h.keys("f");
    assert_eq!(h.top_screen(), "lead_select");
    h.keys(lead_keys);
    h
}

/// Skips the scene on screen: Cancel, then Confirm on "Skip scene?".
fn skip_scene(h: &mut Harness) {
    assert_eq!(h.top_screen(), "dialogue");
    h.keys("d f");
}

/// New Game in Classic with the default lead, through the intro scene to
/// the battle, its notes still up.
fn to_notes() -> Harness {
    let mut h = new_game(false, "Up f");
    skip_scene(&mut h);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert!(notes_open(&h));
    h
}

/// [`to_notes`], and the battle notes closed, then the `PLAYER PHASE`
/// banner after them.
fn to_battle() -> Harness {
    let mut h = to_notes();
    h.keys("f");
    assert!(!notes_open(&h));
    assert!(h.snapshot().contains("PLAYER PHASE"));
    assert!(h.snapshot().contains("Turn 1/3"));
    h.keys("f");
    assert!(!h.snapshot().contains("PLAYER PHASE"));
    h
}

/// Whether the battle on screen shows its notes box.
fn notes_open(h: &Harness) -> bool {
    let battle = h.battle();
    battle.is_some_and(trpg_ui::screens::battle::BattleScreen::notes_open)
}

/// Sends `cmd` to the battle on screen.
fn send(h: &mut Harness, cmd: &Command) {
    let battle = h.battle_mut();
    battle.unwrap_or_else(|| panic!("no battle")).send(cmd);
}

/// The battle on screen.
fn battle(h: &Harness) -> BattleState {
    let battle = h.battle();
    battle
        .unwrap_or_else(|| panic!("no battle"))
        .state()
        .clone()
}

/// Rewind charges left in the battle on screen.
fn charges(h: &Harness) -> u8 {
    let battle = h.battle();
    battle.map_or(0, |b| b.history().charges_left())
}

fn seize(h: &mut Harness) {
    send(
        h,
        &Command::Act {
            unit: LEAD,
            dest: FORT,
            action: UnitAction::Seize,
        },
    );
}

/// Presses Confirm until the top screen is `name` (banners close one per
/// press).
fn confirm_until(h: &mut Harness, name: &str) {
    for _ in 0..20 {
        if h.top_screen() == name {
            return;
        }
        h.keys("f");
    }
    panic!("never reached {name}: {:?}", h.screens());
}

/// From the won battle's `VICTORY` banner, through the results (a press
/// fills the bars, a press goes on), to the victory scene.
fn past_results(h: &mut Harness) {
    h.keys("f");
    assert_eq!(h.screens(), ["title", "game_flow", "results"]);
    h.keys("f f");
}

/// Answers "Save your progress?" with No.
fn decline_save(h: &mut Harness) {
    assert_eq!(h.screens(), ["title", "game_flow", "save_prompt"]);
    h.keys("Down f");
}

/// Acceptance: New Game on the test chapter → skip the scenes → win with
/// scripted commands → the results → the victory scene → the save prompt
/// → "To be
/// continued" → title.
#[test]
fn new_game_plays_the_test_chapter_to_the_end() {
    let mut h = to_battle();
    let chapter = h.flow().and_then(|f| f.chapter()).map(|c| c.id.clone());
    assert_eq!(chapter.as_deref(), Some("test"));
    assert_eq!(charges(&h), 3, "a Normal map");
    seize(&mut h);
    assert_eq!(battle(&h).outcome(), Some(Outcome::Victory));
    // The VICTORY banner, the results, then the victory scene.
    past_results(&mut h);
    assert_eq!(h.screens(), ["title", "game_flow", "dialogue"]);
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    let campaign = flow.campaign().unwrap_or_else(|| panic!("no campaign"));
    assert_eq!(campaign.gold, 500, "the battle's clear gold");
    // The two unused potions went to the stock.
    assert_eq!(campaign.stock.count(&trpg_core::ItemId::new("potion")), 2);
    let rewards = flow.rewards().unwrap_or_else(|| panic!("no rewards"));
    assert_eq!(rewards.unused_charges, 3);
    assert_eq!(rewards.bonus_exp, 21);
    skip_scene(&mut h);
    decline_save(&mut h);
    assert_eq!(h.screens(), ["title", "game_flow", "to_be_continued"]);
    assert_snapshot!(h.snapshot());
    h.keys("f");
    assert_eq!(h.screens(), ["title"]);
}

/// Acceptance: New Game → the female lead, her name typed → the intro shows her name and
/// pronouns (0708 tokens).
#[test]
fn the_intro_speaks_of_the_lead_the_player_made() {
    // Female; the name box: "Ellery" deleted, "Ma" typed, Enter; Start.
    let mut h = new_game(
        true,
        "Right Down f Backspace Backspace Backspace Backspace Backspace Backspace",
    );
    h.type_text("Ma");
    h.keys("Enter Down f");
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    let campaign = flow.campaign().unwrap_or_else(|| panic!("no campaign"));
    assert_eq!(campaign.mode, GameMode::Casual);
    assert_eq!(campaign.lead, LeadProfile::new("Ma", LeadGender::Female));
    assert_eq!(campaign.roster[0].name, "Ma");
    assert_eq!(h.game().ctx().game.lead, campaign.lead);
    assert_eq!(h.top_screen(), "dialogue");
    // First box: the knight names her. Second: narration with her
    // pronoun.
    h.keys("f");
    assert!(shows(&h, "Ma! The fort is just ahead."), "{}", h.snapshot());
    h.keys("f f");
    assert!(
        shows(&h, "Ma looks over the field. She can see the fort"),
        "{}",
        h.snapshot()
    );
    assert_snapshot!(h.snapshot());
}

/// Acceptance (0810): winning the test chapter shows the clear gold and the
/// unused-rewind EXP bonus, each deployed unit's bar filling by it, before
/// the victory scene.
#[test]
fn a_won_battle_shows_its_gold_and_the_rewind_bonus() {
    let mut h = to_battle();
    seize(&mut h);
    h.keys("f");
    assert_eq!(h.screens(), ["title", "game_flow", "results"]);
    assert!(shows(&h, "Gold for clearing the map"), "{}", h.snapshot());
    assert!(shows(&h, "+500 (now 500)"), "{}", h.snapshot());
    assert!(shows(&h, "3 of 3"));
    assert!(shows(&h, "Bonus EXP for each unit                     +21"));
    // Both deployed units, before the bars move.
    let bar = |name: &str, class: &str, bar: &str, exp: u32| {
        format!("{name:<14}{class:<9}Lv 1  EXP {bar:░<20} {exp:>2}")
    };
    assert!(
        shows(&h, &bar("Ellery", "Exile", "", 0)),
        "{}",
        h.snapshot()
    );
    assert!(shows(&h, &bar("Test Knight", "Guard", "", 0)));
    // They fill by themselves, and then wait.
    h.wait(3.0);
    assert_eq!(h.screens(), ["title", "game_flow", "results"]);
    assert!(
        shows(&h, &bar("Ellery", "Exile", "████", 21)),
        "{}",
        h.snapshot()
    );
    assert!(shows(&h, &bar("Test Knight", "Guard", "████", 21)));
    assert_snapshot!(h.snapshot());
    // What the screen showed is what the campaign got.
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    let campaign = flow.campaign().unwrap_or_else(|| panic!("no campaign"));
    assert_eq!(campaign.gold, 500);
    let exp: Vec<u32> = campaign.roster.iter().map(|u| u.exp).collect();
    assert_eq!(exp[..2], [21, 21]);
    // Nobody levelled: one press goes on to the victory scene.
    h.keys("d");
    assert_eq!(h.screens(), ["title", "game_flow", "dialogue"]);
}

/// Acceptance (0810): with a rewind charge used the bonus is smaller, and
/// with none left there is no EXP line at all.
#[test]
fn used_rewind_charges_shrink_the_bonus_to_nothing() {
    let wait_at = |h: &mut Harness, x: i32| {
        send(
            h,
            &Command::Act {
                unit: LEAD,
                dest: Pos::new(x, 5),
                action: UnitAction::Wait,
            },
        );
    };
    let mut h = to_battle();
    wait_at(&mut h, 4);
    h.keys("r f f");
    assert_eq!(charges(&h), 2);
    seize(&mut h);
    h.keys("f");
    assert!(shows(&h, "2 of 3"), "{}", h.snapshot());
    assert!(shows(&h, "Bonus EXP for each unit                     +14"));
    // Every charge used.
    let mut h = to_battle();
    for _ in 0..3 {
        wait_at(&mut h, 4);
        h.keys("r f f");
    }
    assert_eq!(charges(&h), 0);
    seize(&mut h);
    h.keys("f");
    assert_eq!(h.screens(), ["title", "game_flow", "results"]);
    assert!(shows(&h, "+500 (now 500)"), "{}", h.snapshot());
    assert!(shows(&h, "0 of 3"));
    assert!(!shows(&h, "Bonus EXP"));
    assert!(!shows(&h, "EXP"));
    h.keys("f");
    assert_eq!(h.screens(), ["title", "game_flow", "dialogue"]);
}

/// Whether some row of the screen contains `text`.
fn shows(h: &Harness, text: &str) -> bool {
    let buf = h.game().buffer();
    (0..32).any(|y| {
        (0..100)
            .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
            .collect::<String>()
            .contains(text)
    })
}

/// Acceptance: a defeat → Game Over → Retry starts the battle again,
/// identical to its start, with every rewind charge back.
#[test]
fn retry_after_a_defeat_restarts_the_battle() {
    let mut h = to_battle();
    let start = battle(&h);
    // A move, rewound (a charge spent), then the turn limit runs out.
    send(
        &mut h,
        &Command::Act {
            unit: LEAD,
            dest: Pos::new(4, 5),
            action: UnitAction::Wait,
        },
    );
    h.keys("r f f");
    assert_eq!(charges(&h), 2);
    for _ in 0..6 {
        send(&mut h, &Command::EndPhase);
    }
    assert_eq!(battle(&h).outcome(), Some(Outcome::Defeat));
    confirm_until(&mut h, "game_over");
    assert_eq!(h.screens(), ["title", "game_flow", "battle", "game_over"]);
    // Past its opening animation (0830).
    h.wait(2.1);
    assert_snapshot!(h.snapshot());
    // Retry (below Rewind): the battle from its start, its notes first.
    h.keys("Down f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert_eq!(battle(&h), start);
    assert_eq!(charges(&h), 3);
    assert!(notes_open(&h));
    h.keys("f");
    assert!(shows(&h, "PLAYER PHASE"), "{}", h.snapshot());
    assert!(shows(&h, "Turn 1/3"));
    // Lose again, and go back to the title.
    for _ in 0..6 {
        send(&mut h, &Command::EndPhase);
    }
    confirm_until(&mut h, "game_over");
    // The first press ends the animation; the rest is the menu's.
    h.keys("f Down Down f");
    assert_eq!(h.screens(), ["title"]);
}

/// A lost battle, on the first frame of Game Over (two rewind charges
/// left).
fn lost() -> Harness {
    let mut h = to_battle();
    for _ in 0..6 {
        send(&mut h, &Command::EndPhase);
    }
    assert_eq!(battle(&h).outcome(), Some(Outcome::Defeat));
    confirm_until(&mut h, "game_over");
    h
}

/// Game Over as it is this frame.
fn game_over(h: &Harness) -> trpg_ui::screens::game_over::GameOverView {
    let screen = h.screen::<trpg_ui::screens::GameOverScreen>();
    let screen = screen.unwrap_or_else(|| panic!("no game over"));
    screen.view(h.game().ctx())
}

/// Ticket 0830: Game Over opens with a two-second animation over the lost
/// battle, and the menu is there only at its end. Snapshots: the first
/// frame (the battle, as it was lost), the middle (black, the logo
/// fading in) and the last (the logo, the words, the menu).
#[test]
fn game_over_opens_with_its_animation() {
    let mut h = lost();
    let first = game_over(&h);
    assert!(first.progress < 16, "{}", first.progress);
    assert_eq!((first.menu, first.help), (None, None));
    assert_snapshot!("game_over_first_frame", h.snapshot());
    h.wait(1.0);
    let middle = game_over(&h);
    assert!(
        (120..=136).contains(&middle.progress),
        "{}",
        middle.progress
    );
    assert_eq!((middle.menu, middle.help), (None, None));
    assert_snapshot!("game_over_middle_frame", h.snapshot());
    // Still not there a tenth of a second before the end.
    h.wait(0.85);
    let late = game_over(&h);
    assert!(late.progress < 255, "{}", late.progress);
    assert_eq!(late.menu, None);
    h.wait(0.2);
    let last = game_over(&h);
    assert_eq!(last.progress, 255);
    let menu = last.menu.unwrap_or_else(|| panic!("no menu"));
    let labels: Vec<_> = menu.items.iter().map(|i| i.label.as_str()).collect();
    assert_eq!(labels, ["Rewind", "Retry Battle", "Title"]);
    assert_eq!(menu.focused().map(|i| i.label.as_str()), Some("Rewind"));
    assert!(last.help.is_some());
    assert_snapshot!("game_over_last_frame", h.snapshot());
    // Waiting picked nothing, and the menu works as before: Retry.
    assert_eq!(h.screens(), ["title", "game_flow", "battle", "game_over"]);
    h.keys("Down f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert_eq!(charges(&h), 3);
}

/// Ticket 0830: a press during the animation jumps to its end and does
/// nothing else: `Rewind`, under the cursor, is not picked.
#[test]
fn a_press_ends_game_overs_animation_and_picks_nothing() {
    for skip in ["f", "Down", "d"] {
        let mut h = lost();
        h.wait(0.5);
        assert_eq!(game_over(&h).menu, None);
        h.clear_audio();
        h.keys(skip);
        assert_eq!(
            h.screens(),
            ["title", "game_flow", "battle", "game_over"],
            "{skip}"
        );
        let view = game_over(&h);
        assert_eq!(view.progress, 255, "{skip}");
        let menu = view.menu.unwrap_or_else(|| panic!("no menu"));
        assert_eq!(menu.focused().map(|i| i.label.as_str()), Some("Rewind"));
        assert_eq!(h.sounds(), [""; 0], "{skip}");
        // The next press is the menu's: the rewind screen, on the battle.
        h.keys("f");
        let rewinding = ["title", "game_flow", "battle", "rewind"];
        assert_eq!(h.screens(), rewinding, "{skip}");
        assert_eq!(battle(&h).outcome(), Some(Outcome::Defeat));
        // Backed out of, Game Over is as it was left: no animation again.
        h.keys("d");
        assert_eq!(
            h.screens(),
            ["title", "game_flow", "battle", "game_over"],
            "{skip}"
        );
        assert_eq!(game_over(&h).progress, 255, "{skip}");
    }
}

/// Acceptance: map menu → `Restart Battle` → confirm: the same as Retry.
#[test]
fn restart_battle_from_the_map_menu() {
    let mut h = to_battle();
    let start = battle(&h);
    send(
        &mut h,
        &Command::Act {
            unit: LEAD,
            dest: Pos::new(4, 5),
            action: UnitAction::Wait,
        },
    );
    h.keys("r f f");
    assert_eq!(charges(&h), 2);
    send(
        &mut h,
        &Command::Act {
            unit: UnitId(2),
            dest: Pos::new(4, 6),
            action: UnitAction::Wait,
        },
    );
    // The map menu: Units, Objective, Guides, Options, Suspend, Restart Battle.
    h.keys("d Down Down Down Down Down f");
    assert!(
        shows(&h, "Restart the battle from turn 1?"),
        "{}",
        h.snapshot()
    );
    assert_snapshot!(h.snapshot());
    // Back out to the menu, then confirm.
    h.keys("d");
    assert!(!shows(&h, "Restart the battle"));
    h.keys("f f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert_eq!(battle(&h), start);
    assert_eq!(charges(&h), 3);
    assert!(notes_open(&h));
    h.keys("f");
    assert!(shows(&h, "PLAYER PHASE"), "{}", h.snapshot());
    assert!(shows(&h, "Turn 1/3"));
}

/// Acceptance (0411): the test battle's two notes show in a box when the
/// battle starts, before any phase banner; only Confirm closes it; the map
/// menu's `Objective` page lists them again.
#[test]
fn battle_notes_show_at_the_start_and_on_the_objective_page() {
    let mut h = to_notes();
    let start = battle(&h);
    assert_eq!(start.battle_notes().len(), 2);
    assert!(shows(&h, "BATTLE NOTES"), "{}", h.snapshot());
    assert!(shows(&h, "• Brigand: guards the road, but the fort is"));
    assert!(shows(&h, "  nearer than the fight."));
    assert!(shows(&h, "• Seize the fort within 3 turns: send your lead"));
    assert!(shows(&h, "f close"));
    assert_snapshot!(h.snapshot());
    // No key but Confirm does anything: no map menu, no cursor move, no
    // end of turn.
    let cursor = |h: &Harness| h.battle().map(|b| b.cursor().pos);
    let at = cursor(&h);
    h.keys("d Right e Space Space r");
    assert!(notes_open(&h));
    assert_eq!(cursor(&h), at);
    assert_eq!(battle(&h), start);
    assert!(!shows(&h, "Objective"));
    h.keys("f");
    assert!(!notes_open(&h));
    assert!(!shows(&h, "BATTLE NOTES"));
    assert!(!shows(&h, "Brigand: guards"));
    assert_eq!(battle(&h), start);
    // Then turn 1's `PLAYER PHASE` banner (0435), then the map.
    assert!(shows(&h, "PLAYER PHASE"), "{}", h.snapshot());
    assert!(shows(&h, "Turn 1/3"));
    h.keys("f");
    assert!(!shows(&h, "PLAYER PHASE"));
    assert_eq!(battle(&h), start);
    // The map menu's Objective page: the objective, then the notes.
    h.keys("d Down f");
    assert!(shows(&h, "Seize the Fort"), "{}", h.snapshot());
    assert!(shows(&h, "Turn 1/3"));
    assert!(shows(&h, "Battle notes"));
    assert!(shows(&h, "• Brigand: guards the road, but the fort is"));
    assert!(shows(&h, "• Seize the fort within 3 turns: send your lead"));
    assert!(!shows(&h, "BATTLE NOTES"));
    assert_snapshot!("objective_page_lists_the_battle_notes", h.snapshot());
    h.keys("d d");
    assert!(!shows(&h, "Battle notes"));
}

/// Acceptance (0411): the first phase banner comes after the notes, not
/// over them.
#[test]
fn the_phase_banner_waits_for_the_battle_notes() {
    let banner = |h: &Harness| {
        let battle = h.battle();
        battle.is_some_and(|b| b.banner().is_some())
    };
    let mut h = to_notes();
    assert!(!banner(&h));
    // The enemy phase begins behind the box: its banner waits.
    send(&mut h, &Command::EndPhase);
    h.keys("Left");
    assert!(notes_open(&h));
    assert!(!banner(&h));
    assert!(!shows(&h, "PHASE"));
    // The banners come in order: turn 1's player phase, then the enemy's.
    h.keys("f");
    assert!(!notes_open(&h));
    assert!(banner(&h));
    assert!(shows(&h, "PLAYER PHASE"), "{}", h.snapshot());
    assert!(!shows(&h, "ENEMY PHASE"));
    h.keys("f");
    assert!(banner(&h));
    assert!(shows(&h, "ENEMY PHASE"), "{}", h.snapshot());
}

/// Acceptance (0411): a battle without notes (the Quick Battle) has no
/// notes box, and nothing more on its `Objective` page.
#[test]
fn no_notes_box_for_a_battle_without_notes() {
    let mut h = title();
    // Quick Battle, then Preparations: Left wraps to `Fight!`.
    h.keys("Down f Left f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert!(battle(&h).battle_notes().is_empty());
    assert!(!notes_open(&h));
    assert!(!shows(&h, "BATTLE NOTES"));
    // It opens on the `PLAYER PHASE` banner.
    assert!(shows(&h, "PLAYER PHASE"), "{}", h.snapshot());
    h.keys("f");
    // The Objective page: only the objective and the turn.
    h.keys("d Down f");
    assert!(shows(&h, "Rout the enemy"), "{}", h.snapshot());
    assert!(!shows(&h, "Battle notes"));
}

/// The music the run asked for, in order: cue names, `-` for a stop.
fn music(h: &Harness) -> Vec<String> {
    h.audio_requests()
        .iter()
        .filter(|r| !matches!(r, AudioRequest::PlaySound { .. }))
        .map(|r| r.cue().unwrap_or("-").to_owned())
        .collect()
}

/// The `n`-th random music pick of a run, from `pool`, with the harness's
/// music seed.
fn pool_pick(h: &mut Harness, pool: &str, n: u64) -> String {
    let ctx = h.ctx_mut();
    pick_from_pool(&ctx.game.content.audio, pool, ctx.music_seed ^ n)
        .unwrap_or_else(|| panic!("no {pool} pool"))
        .to_owned()
}

/// The `n`-th random music pick of a run from the `skirmish` pool (the
/// test battle's `Pool("skirmish")`).
fn skirmish_pick(h: &mut Harness, n: u64) -> String {
    pool_pick(h, "skirmish", n)
}

/// Presses Confirm until the banner `text` shows; no music is asked for
/// on the way.
fn confirm_until_banner(h: &mut Harness, text: &str) {
    let before = music(h);
    for _ in 0..20 {
        if shows(h, text) {
            return;
        }
        assert_eq!(music(h), before);
        h.keys("f");
    }
    panic!("no {text} banner: {}", h.snapshot());
}

/// Ticket 0807: a story battle's cue starts with the battle (the intro
/// scene still has the title's music) and stays through turns and a
/// rewind. Ticket 0809: the `VICTORY` banner asks for one sting of the
/// `victory` pool, once; nothing else is asked for through the results,
/// the victory scene and the save prompt. "To be continued" stops the
/// music; then the title's.
#[test]
fn the_battles_music_plays_from_its_start_to_the_end_of_the_chapter() {
    let mut h = title();
    if let Some(test) = h.ctx_mut().game.content.battles.get_mut("test") {
        test.music = trpg_core::BattleMusic::Cue("battle_easy".into());
    }
    h.keys("f f f Up f");
    assert_eq!(h.top_screen(), "dialogue");
    assert_eq!(music(&h), ["title"]);
    skip_scene(&mut h);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert_eq!(music(&h), ["title", "battle_easy"]);
    // The battle notes closed, and the banner after them. A move, rewound;
    // then a whole turn: enemy phase, player phase.
    h.keys("f f");
    send(
        &mut h,
        &Command::Act {
            unit: LEAD,
            dest: Pos::new(4, 5),
            action: UnitAction::Wait,
        },
    );
    h.keys("r f f");
    assert_eq!(charges(&h), 2);
    send(&mut h, &Command::EndPhase);
    send(&mut h, &Command::EndPhase);
    h.wait(1.0);
    assert_eq!(battle(&h).turn(), 2);
    seize(&mut h);
    // Past the phase banner: the sting comes with the VICTORY banner.
    confirm_until_banner(&mut h, "VICTORY");
    let sting = pool_pick(&mut h, "victory", 0);
    assert_eq!(music(&h), ["title", "battle_easy", sting.as_str()]);
    h.wait(2.0);
    confirm_until(&mut h, "dialogue");
    assert_eq!(h.screens(), ["title", "game_flow", "dialogue"]);
    assert_eq!(music(&h), ["title", "battle_easy", sting.as_str()]);
    skip_scene(&mut h);
    decline_save(&mut h);
    assert_eq!(h.screens(), ["title", "game_flow", "to_be_continued"]);
    assert_eq!(music(&h), ["title", "battle_easy", sting.as_str(), "-"]);
    h.keys("f").wait(0.1);
    assert_eq!(h.screens(), ["title"]);
    assert_eq!(
        music(&h),
        ["title", "battle_easy", sting.as_str(), "-", "title"]
    );
}

/// Ticket 0809: the `DEFEAT` banner asks for `defeat`, once, and the Game
/// Over screen asks for nothing, so it plays on (0023). Ticket 0807:
/// `Retry Battle` starts the battle's music again, a pool picking afresh;
/// `Title` brings the title's.
#[test]
fn defeat_plays_through_game_over_and_retry_plays_the_battles_music_again() {
    let mut h = to_battle();
    let first = skirmish_pick(&mut h, 0);
    assert_eq!(music(&h), ["title", first.as_str()]);
    for _ in 0..6 {
        send(&mut h, &Command::EndPhase);
    }
    confirm_until_banner(&mut h, "DEFEAT");
    assert_eq!(music(&h), ["title", first.as_str(), "defeat"]);
    confirm_until(&mut h, "game_over");
    h.wait(2.0);
    assert_eq!(music(&h), ["title", first.as_str(), "defeat"]);
    h.clear_audio();
    h.keys("Down f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert_eq!(music(&h), [skirmish_pick(&mut h, 1)]);
    for _ in 0..6 {
        send(&mut h, &Command::EndPhase);
    }
    confirm_until(&mut h, "game_over");
    // The first press ends Game Over's opening animation (0830).
    h.keys("f Down Down f").wait(0.1);
    assert_eq!(h.screens(), ["title"]);
    assert_eq!(
        music(&h),
        [skirmish_pick(&mut h, 1).as_str(), "defeat", "title"]
    );
}

/// Ticket 0807: `Restart Battle` asks for the battle's music again without
/// a stop: a pool picks again, a cue (already playing) just carries on.
#[test]
fn restart_battle_asks_for_the_battles_music_again() {
    let mut h = to_battle();
    h.clear_audio();
    h.keys("d Down Down Down Down Down f f f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert_eq!(battle(&h).turn(), 1);
    assert_eq!(music(&h), [skirmish_pick(&mut h, 1)]);

    let mut h = title();
    if let Some(test) = h.ctx_mut().game.content.battles.get_mut("test") {
        test.music = trpg_core::BattleMusic::Cue("battle_easy".into());
    }
    h.keys("f f f Up f");
    skip_scene(&mut h);
    // The battle notes closed.
    h.keys("f");
    h.wait(2.0).clear_audio();
    h.keys("d Down Down Down Down Down f f f").wait(2.0);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert_eq!(music(&h), ["battle_easy"]);
    assert!(h.music_commands().is_empty(), "{:?}", h.music_commands());
}

/// Acceptance (0835): New Game with each familiarity answer goes mode →
/// familiarity → lead, keeps the answer's tutorial level in the campaign,
/// and with tips on shows the battle's first-turn tip only at `Full`.
#[test]
fn each_familiarity_answer_sets_the_level_and_gates_the_tips() {
    for (downs, level, tip) in [
        ("", TutorialLevel::Full, Some(TipTrigger::FirstBattleStart)),
        ("Down", TutorialLevel::Short, None),
        ("Down Down", TutorialLevel::None, None),
    ] {
        let mut h = title();
        h.with_tips();
        h.keys("f");
        assert_eq!(h.screens(), ["title", "game_flow", "mode_select"]);
        h.keys("f");
        assert_eq!(h.screens(), ["title", "game_flow", "familiarity"]);
        h.keys(downs).keys("f");
        assert_eq!(h.screens(), ["title", "game_flow", "lead_select"]);
        h.keys("Up f");
        skip_scene(&mut h);
        let campaign = h.flow().and_then(|f| f.campaign_now(h.game().ctx()));
        let campaign = campaign.unwrap_or_else(|| panic!("no campaign"));
        assert_eq!(campaign.tutorial, level);
        // The battle notes closed, then past the turn's banner.
        h.keys("f").wait(2.0);
        let screen = h.battle();
        let shown = screen.and_then(trpg_ui::screens::battle::BattleScreen::shown_tip);
        assert_eq!(shown, tip, "{level:?}");
    }
}

/// Ticket 0467 (c): a tip queued but not yet shown when the battle is
/// lost (the start tip, behind the battle notes, then the banners) shows
/// in the retried battle: it counted as seen only once on screen.
#[test]
fn a_tip_not_shown_before_a_retry_shows_after_it() {
    let mut h = title();
    h.with_tips();
    h.keys("f f f Up f");
    skip_scene(&mut h);
    assert!(notes_open(&h));
    let shown = |h: &Harness| {
        let screen = h.battle();
        screen.and_then(trpg_ui::screens::battle::BattleScreen::shown_tip)
    };
    assert_eq!(shown(&h), None, "the notes go first");
    for _ in 0..6 {
        send(&mut h, &Command::EndPhase);
    }
    assert_eq!(battle(&h).outcome(), Some(Outcome::Defeat));
    for _ in 0..20 {
        if h.top_screen() == "game_over" {
            break;
        }
        assert_eq!(shown(&h), None, "the banners go first");
        h.keys("f");
    }
    assert_eq!(h.top_screen(), "game_over");
    // Retry: the notes, the banner, then the tip.
    h.wait(2.1).keys("Down f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    h.keys("f f").wait(0.1);
    assert_eq!(shown(&h), Some(TipTrigger::FirstBattleStart));
}

/// Cancel on the lead screen goes back to the familiarity screen; on it,
/// to the mode; on the mode, to the title.
#[test]
fn back_out_of_new_game() {
    let mut h = new_game(false, "d");
    assert_eq!(h.screens(), ["title", "game_flow", "familiarity"]);
    h.keys("d");
    assert_eq!(h.screens(), ["title", "game_flow", "mode_select"]);
    h.keys("d");
    assert_eq!(h.screens(), ["title"]);
}

/// The debug Quick Battle runs through the flow: a won battle with no
/// scenes goes to "To be continued".
#[test]
fn quick_battle_runs_through_the_flow() {
    let mut h = title();
    // Preparations first; Left wraps to `Fight!`.
    h.keys("Down f");
    assert_eq!(h.screens(), ["title", "game_flow", "preparations"]);
    h.keys("Left f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    let campaign = flow.campaign().unwrap_or_else(|| panic!("no campaign"));
    assert_eq!(campaign.chapter, "quick");
    assert_eq!(campaign.mode, GameMode::Classic);
    // Its four slots' characters and the scout left out of the battle.
    assert_eq!(campaign.roster.len(), 5);
    assert_eq!(battle(&h).units().len(), 8);
}

/// Playtime counts from the start of the campaign.
#[test]
fn playtime_counts_while_playing() {
    let mut h = to_battle();
    h.wait(2.0);
    let played = h
        .flow()
        .and_then(|f| f.campaign_now(h.game().ctx()))
        .map_or(0, |c| c.playtime_s);
    assert_eq!(played, 2);
}

/// A chapter with a `next` goes on to it after its victory scenes, with
/// the army it won with.
#[test]
fn the_next_chapter_follows_a_victory() {
    let mut ctx = trpg_ui::Ctx::embedded()
        .unwrap_or_else(|e| panic!("{e}"))
        .with_layout(RIGHT_HANDED);
    ctx.game.content.new_game = trpg_content::NewGameDef::test_chapter();
    if let Some(test) = ctx.game.content.chapters.get_mut("test") {
        test.next = Some("quick".into());
    }
    let mut h = Harness::from_game(trpg_ui::game::start(ctx));
    // New Game, Classic, Not familiar, Start.
    h.keys("f f f Up f");
    skip_scene(&mut h);
    // The battle notes, turn 1's banner, then the VICTORY banner.
    seize(&mut h);
    h.keys("f f");
    past_results(&mut h);
    skip_scene(&mut h);
    decline_save(&mut h);
    // The Quick Battle has Preparations, with the army's own stock: the
    // two Potions left over from the test battle.
    assert_eq!(h.screens(), ["title", "game_flow", "preparations"]);
    let prep = h.screen::<trpg_ui::screens::PreparationsScreen>();
    let prep = prep.unwrap_or_else(|| panic!("no preparations"));
    assert_eq!(prep.spare(), [(trpg_core::ItemId::new("potion"), 2)]);
    // A story battle's Preparations can't be left.
    h.keys("d");
    assert!(!shows(&h, "Leave preparations?"), "{}", h.snapshot());
    h.keys("Left f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    assert_eq!(flow.chapter().map(|c| c.id.as_str()), Some("quick"));
    let campaign = flow.campaign().unwrap_or_else(|| panic!("no campaign"));
    assert_eq!(campaign.chapter, "quick");
    assert_eq!(campaign.gold, 500);
    // The Quick Battle's slots are the test lord, knight, archer and mage: of
    // this army only the knight has one.
    let players: Vec<String> = battle(&h)
        .units()
        .iter()
        .filter(|u| u.faction == trpg_core::Faction::Player)
        .map(|u| u.name.clone())
        .collect();
    assert_eq!(players, ["Test Knight"]);
}

/// On a controller, choosing the name opens the letter grid instead of
/// the typing box (Nick: type on a keyboard, a grid on a controller).
#[test]
fn a_controller_spells_the_name_on_the_letter_grid() {
    let mut h = title();
    // New Game, Classic, Not familiar, down to the name, choose it.
    h.pad("South South South DpadDown South");
    assert_eq!(h.top_screen(), "lead_select");
    assert!(shows(&h, "A  B  C  D"), "{}", h.snapshot());
    assert!(!shows(&h, "Type a name"));
    // East (Cancel) deletes the "y"; "B" is right of "A"; Done is up and
    // round to the left of the bottom row.
    h.pad("East DpadRight South DpadUp DpadLeft DpadLeft South");
    assert!(!shows(&h, "A  B  C  D"), "{}", h.snapshot());
    assert!(shows(&h, "EllerB Veyne"), "{}", h.snapshot());
    // The keyboard still types: the same name row opens the typing box.
    h.keys("f");
    assert!(shows(&h, "Type a name on your keyboard."));
}
