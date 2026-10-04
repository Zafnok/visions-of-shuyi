use trpg_core::{Command, LeadGender, LeadProfile, Pos, StatKind, UnitAction, UnitId};

use super::*;
use crate::game::Game;
use crate::harness::Harness;
use crate::input::Action::Cancel;
use crate::screen::tests::ctx;
use crate::screens::battle::progress::stat_row;

/// Playtime counts the seconds since the campaign began, carrying on
/// from what it had (a loaded campaign, 0802).
#[test]
fn playtime_carries_on_from_the_campaigns() {
    let mut c = ctx();
    c.clock_s = 10.0;
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let mut campaign = trpg_content::new_campaign(&c.content, GameMode::Casual, lead.clone());
    campaign.playtime_s = 100;
    let mut flow = FlowScreen::new_game();
    flow.begin(&mut c, campaign);
    assert_eq!(c.lead, lead);
    c.clock_s = 12.5;
    flow.count_playtime(&c);
    assert_eq!(flow.campaign().map(|c| c.playtime_s), Some(102));
}

fn update(flow: &mut FlowScreen, c: &mut Ctx, actions: &[crate::input::Action]) {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    flow.update(c, &input);
}

/// Nick (0408): after a defeat, `Retry Battle` goes back to Preparations
/// with the loadouts and pack as the player left them.
#[test]
fn retry_goes_back_to_preparations_as_they_were_left() {
    use crate::input::Action::{Confirm, CursorLeft, CursorRight};
    let mut c = ctx();
    let mut flow = FlowScreen::quick_battle(&mut c).unwrap_or_else(|| panic!("no quick battle"));
    assert_eq!(flow.name(), "preparations");
    assert!(flow.battle().is_none());
    // Pack an Elixir, then Fight!.
    update(&mut flow, &mut c, &[CursorRight, Confirm, Confirm]);
    update(&mut flow, &mut c, &[Cancel, CursorRight, Confirm]);
    assert_eq!(flow.name(), "battle");
    assert!(flow.preparations().is_none());
    let elixir = [trpg_core::ItemId::new("elixir")];
    let pack = |f: &FlowScreen| f.battle().map(|b| b.state().pack().items.clone());
    assert_eq!(pack(&flow).as_deref(), Some(&elixir[..]));
    // A defeat's Game Over, then Retry.
    flow.stage = Stage::GameOver(GameOverScreen::new());
    update(&mut flow, &mut c, &[Confirm]);
    assert_eq!(flow.name(), "preparations");
    let prep = flow.preparations().unwrap_or_else(|| panic!("no prep"));
    assert_eq!(prep.setup().pack.items, elixir);
    assert_eq!(prep.setup().stock.count(&elixir[0]), 1);
    // Fight! again: the same battle.
    update(&mut flow, &mut c, &[CursorLeft, Confirm]);
    assert_eq!(pack(&flow).as_deref(), Some(&elixir[..]));
}

/// A battle without Preparations starts straight away, with its default
/// pack.
#[test]
fn a_battle_without_preparations_starts_at_once() {
    let mut c = ctx();
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let mut campaign = trpg_content::new_campaign(&c.content, GameMode::Casual, lead);
    campaign.chapter = "test".into();
    let mut flow = FlowScreen::new_game();
    flow.begin(&mut c, campaign);
    // Skip the intro scene: Cancel, then Confirm.
    update(&mut flow, &mut c, &[Cancel]);
    update(&mut flow, &mut c, &[crate::input::Action::Confirm]);
    assert_eq!(flow.name(), "battle");
    let pack = flow.battle().map(|b| b.state().pack().items.len());
    assert_eq!(pack, Some(2));
}

/// The flow gives a battle the look its map's file names (ADR-0052): when
/// it starts it, when it restarts it, and when a suspended battle is
/// continued.
#[test]
fn a_battle_has_the_look_its_maps_file_names() {
    use crate::input::Action::{Confirm, CursorLeft};
    let look = |flow: &FlowScreen, c: &Ctx| flow.battle().map(|b| b.scene(c).look.tiles);
    // The Quick Battle opens on Preparations: Fight!.
    let fight = |flow: &mut FlowScreen, c: &mut Ctx| {
        assert_eq!(flow.name(), "preparations");
        update(flow, c, &[CursorLeft, Confirm]);
    };
    // The Quick Battle's map names no look: a map outdoors.
    let mut c = ctx();
    let mut flow = FlowScreen::quick_battle(&mut c).unwrap();
    fight(&mut flow, &mut c);
    assert_eq!(look(&flow, &c).as_deref(), Some("outdoor"));
    // With one: started.
    let mut c = ctx();
    let map = c.content.maps.get_mut("test_small").unwrap();
    map.look.tiles = "indoor".to_owned();
    let mut flow = FlowScreen::quick_battle(&mut c).unwrap();
    fight(&mut flow, &mut c);
    assert_eq!(look(&flow, &c).as_deref(), Some("indoor"));
    // Restarted: Preparations again, then the same look.
    flow.restart(&mut c);
    fight(&mut flow, &mut c);
    assert_eq!(look(&flow, &c).as_deref(), Some("indoor"));
    // Suspended, then continued.
    let stage = std::mem::replace(&mut flow.stage, Stage::ToBeContinued(ToBeContinuedScreen));
    let Stage::Battle(battle) = stage else {
        panic!("no battle");
    };
    assert!(flow.suspend(&mut c, battle));
    let continued = FlowScreen::resume(&mut c).unwrap();
    assert_eq!(look(&continued, &c).as_deref(), Some("indoor"));
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

/// Acceptance (0810): a level up caused by the rewind bonus shows 0602's
/// level-up screen after the results, with the numbers the roster got, and
/// waits for its own press before the victory scene.
#[test]
fn a_level_up_from_the_rewind_bonus_shows_the_level_up_screen() {
    let mut c = ctx();
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let mut campaign = trpg_content::new_campaign(&c.content, GameMode::Classic, lead);
    // Three unused charges (21 EXP) level her.
    campaign.roster[0].exp = 85;
    let before = campaign.roster[0].clone();
    let mut flow = FlowScreen::new_game();
    flow.begin(&mut c, campaign);
    let mut h = Harness::from_game(Game::new(c, Box::new(flow)));
    // Skip the intro, close the battle notes and turn 1's banner; the lead
    // seizes the fort.
    h.keys("d f f f");
    assert_eq!(h.top_screen(), "battle");
    let seize = Command::Act {
        unit: UnitId(1),
        dest: Pos::new(5, 5),
        action: UnitAction::Seize,
    };
    let battle = h.flow_mut().and_then(FlowScreen::battle_mut);
    battle.unwrap_or_else(|| panic!("no battle")).send(&seize);
    h.keys("f");
    assert_eq!(h.top_screen(), "results");
    let stage = h.flow().map(FlowScreen::stage);
    assert!(matches!(stage, Some(Stage::Results(_))));
    let row = "Mara          Exile    Lv 1  EXP █████████████████░░░ 85";
    assert!(shows(&h, row), "{}", h.snapshot());
    // A press fills the bars: she is level 2, and marked.
    h.keys("f");
    let row = "Mara          Exile    Lv 2  EXP █░░░░░░░░░░░░░░░░░░░  6  LEVEL UP";
    assert!(shows(&h, row), "{}", h.snapshot());
    // The next shows her level-up page; it stays until its own press.
    h.keys("f");
    assert_eq!(h.top_screen(), "results");
    assert!(shows(&h, "LEVEL UP!"), "{}", h.snapshot());
    assert!(shows(&h, "Lv 1 → 2"));
    h.wait(10.0);
    assert_eq!(h.top_screen(), "results");
    let campaign = h.flow().and_then(FlowScreen::campaign);
    let after = campaign.map(|c| c.roster[0].clone());
    let after = after.unwrap_or_else(|| panic!("no campaign"));
    assert_eq!((after.level, after.exp), (2, 6));
    // The page's numbers are the roster's.
    for kind in StatKind::GROWABLE {
        let was = before.stats.get(kind);
        let row = stat_row(kind, was, after.stats.get(kind) - was);
        assert!(shows(&h, &row), "{row}: {}", h.snapshot());
    }
    h.keys("d");
    assert_eq!(h.top_screen(), "dialogue");
}

/// The test chapter in `mode`, won after the knight fell in its battle:
/// the first text of its victory scene, and whether the knight is still in
/// the army then.
fn victory_line_after_the_knight_fell(mode: GameMode) -> (Option<String>, bool) {
    let knight = trpg_core::CharacterId("test_knight".into());
    let mut c = ctx();
    // The brigand (unit 3) stands next to the knight and never misses.
    let battle = c.content.battles.get_mut("test");
    let brigand = &mut battle.unwrap_or_else(|| panic!("no test battle")).enemies[0];
    brigand.pos = Pos::new(4, 6);
    for kind in [StatKind::Hp, StatKind::Str, StatKind::Dex, StatKind::Def] {
        brigand.stats.set(kind, 60);
    }
    brigand.hp = 60;
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let mut campaign = trpg_content::new_campaign(&c.content, mode, lead);
    assert_eq!(campaign.roster[1].character.as_ref(), Some(&knight));
    campaign.roster[1].stats.hp = 1;
    let mut flow = FlowScreen::new_game();
    flow.begin(&mut c, campaign);
    let mut h = Harness::from_game(Game::new(c, Box::new(flow)));
    // Skip the intro, close the battle notes and turn 1's banner.
    h.keys("d f f f");
    assert_eq!(h.top_screen(), "battle");
    let act = |h: &mut Harness, unit, dest, action| {
        let battle = h.flow_mut().and_then(FlowScreen::battle_mut);
        let battle = battle.unwrap_or_else(|| panic!("no battle"));
        battle.send(&Command::Act { unit, dest, action });
    };
    // The knight strikes the brigand and falls to its counter.
    let attack = UnitAction::Attack {
        target: UnitId(3),
        slot: 0,
        active: None,
        art: None,
    };
    act(&mut h, UnitId(2), Pos::new(3, 6), attack);
    let battle = h.flow().and_then(FlowScreen::battle);
    let fallen = battle.map(|b| b.state().fallen().len());
    assert_eq!(fallen, Some(1), "the knight fell");
    h.wait(30.0);
    // The lead seizes the fort.
    act(&mut h, UnitId(1), Pos::new(5, 5), UnitAction::Seize);
    for _ in 0..10 {
        if h.top_screen() == "dialogue" {
            break;
        }
        h.keys("f");
    }
    assert_eq!(h.top_screen(), "dialogue");
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    let Stage::Scene(scene) = flow.stage() else {
        panic!("no scene");
    };
    assert_eq!(scene.player().scene_id(), "test_victory");
    let text = scene.player().current().text.map(String::from);
    let campaign = flow.campaign().unwrap_or_else(|| panic!("no campaign"));
    (text, campaign.member(&knight).is_some())
}

/// Acceptance (0715): a companion who died in the battle (Classic) has
/// left the army by the victory scene, which plays the lines for when
/// they are gone; one who retreated (Casual) is still in it, and speaks.
#[test]
fn the_victory_scene_plays_for_the_army_as_the_battle_left_it() {
    let (text, in_army) = victory_line_after_the_knight_fell(GameMode::Classic);
    assert!(!in_army);
    assert_eq!(
        text.as_deref(),
        Some("The fort is taken. Nobody is left to tell Mara well done.")
    );
    let (text, in_army) = victory_line_after_the_knight_fell(GameMode::Casual);
    assert!(in_army);
    assert_eq!(text.as_deref(), Some("The fort is ours, Mara. Well done."));
}

/// A scene with nothing to say for the army as it is isn't played: the
/// flow goes on to the next one, or to what follows them.
#[test]
fn a_scene_with_nothing_to_say_for_the_army_is_passed_over() {
    let mut c = ctx();
    let src = "@scene only_knight\n@if test_knight\n> The knight nods.\n@endif\n@end\n";
    let scenes = trpg_content::dialogue::from_sources(&[("t.dlg", src)], None, None, None, None)
        .unwrap_or_else(|e| panic!("{e:?}"));
    c.content.dialogue.scenes.extend(scenes.scenes);
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let start = |c: &mut Ctx, with_knight: bool, scenes: &[&str]| {
        let mut campaign = trpg_content::new_campaign(&c.content, GameMode::Classic, lead.clone());
        if !with_knight {
            campaign.roster.truncate(1);
        }
        let mut flow = FlowScreen::new_game();
        flow.adopt(c, campaign);
        flow.scenes = scenes.iter().map(|s| (*s).to_owned()).collect();
        flow.then = Then::NextChapter;
        flow.next_scene(c);
        flow
    };
    let playing = |flow: &FlowScreen| match flow.stage() {
        Stage::Scene(scene) => Some(scene.player().scene_id().to_owned()),
        _ => None,
    };
    // With the knight: played.
    let flow = start(&mut c, true, &["only_knight", "test_victory"]);
    assert_eq!(playing(&flow).as_deref(), Some("only_knight"));
    assert_eq!(flow.scenes.len(), 1);
    // Without: the next scene plays instead.
    let flow = start(&mut c, false, &["only_knight", "test_victory"]);
    assert_eq!(playing(&flow).as_deref(), Some("test_victory"));
    assert!(flow.scenes.is_empty());
    // With no scene left: what follows them.
    let flow = start(&mut c, false, &["only_knight"]);
    assert!(matches!(flow.stage(), Stage::SavePrompt(_)));
}
