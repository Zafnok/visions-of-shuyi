use trpg_core::{Command, LeadGender, LeadProfile, Pos, StatKind, UnitAction, UnitId};

use super::*;
use crate::audio::AudioRequest;
use crate::harness::{FRAME_DT, Harness};
use crate::input::Action::Cancel;
use crate::screen::ScreenStack;
use crate::screen::tests::ctx;
use crate::screens::GameOverScreen;
use crate::screens::battle::progress::stat_row;

/// Playtime counts the seconds since the campaign began, carrying on
/// from what it had (a loaded campaign, 0802).
#[test]
fn playtime_carries_on_from_the_campaigns() {
    let mut c = ctx();
    c.clock_s = 10.0;
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let mut campaign = trpg_content::new_campaign(&c.game.content, GameMode::Casual, lead.clone());
    campaign.playtime_s = 100;
    let mut flow = FlowScreen::new_game(&c);
    flow.begin(&mut c, campaign);
    assert_eq!(c.game.lead, lead);
    c.clock_s = 12.5;
    let campaign = flow.campaign_now(&c);
    assert_eq!(campaign.map(|c| c.playtime_s), Some(102));
}

/// One frame of the stack with `actions`.
fn update(stack: &mut ScreenStack, c: &mut Ctx, actions: &[crate::input::Action]) {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    stack.update(c, &input);
}

/// `flow` on a stack of its own, after the frame that pushes its first
/// screen.
fn stacked(c: &mut Ctx, flow: FlowScreen) -> ScreenStack {
    let mut stack = ScreenStack::new(Box::new(flow));
    update(&mut stack, c, &[]);
    stack
}

/// The name of the screen on top.
fn top(stack: &ScreenStack) -> &'static str {
    stack.top_name().unwrap_or_default()
}

/// The flow as the root of a running game, its first screen pushed.
fn harness(c: Ctx, flow: FlowScreen) -> Harness {
    let mut h = Harness::from_game(crate::game::new(c, Box::new(flow)));
    h.frame(&[], FRAME_DT);
    h
}

/// The flow pushes its first screen over itself on its first update; until
/// then it is named, and drawn, as that screen.
#[test]
fn the_flow_pushes_its_first_screen() {
    let mut c = ctx();
    let flow = FlowScreen::new_game(&c);
    assert_eq!(flow.name(), "mode_select");
    let stack = stacked(&mut c, flow);
    assert_eq!(stack.names(), ["game_flow", "mode_select"]);
}

/// On the frame it is pushed, before its first screen is, the flow looks
/// like that screen.
#[test]
fn the_flow_draws_its_first_screen_until_it_pushes_it() {
    use crate::color::UiColor;
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::glyph_buffer::Cell;
    let c = ctx();
    let p = &c.palette;
    let stale = Cell::new('x', p.get(UiColor::Enemy), p.get(UiColor::PanelBg));
    let mut flow = GlyphBuffer::with_spec(ConsoleSpec::PC, stale);
    FlowScreen::new_game(&c).draw(&c, &mut flow);
    let mut mode = GlyphBuffer::with_spec(ConsoleSpec::PC, stale);
    ModeSelectScreen::new(&c).draw(&c, &mut mode);
    assert_eq!(flow, mode);
    assert_ne!(flow, GlyphBuffer::with_spec(ConsoleSpec::PC, stale));
}

/// Each screen of New Game pops with its answer, and the flow pushes the
/// next; going back pops with nothing, and the flow pushes the one before.
#[test]
fn new_game_reads_what_each_screen_pops_with() {
    use crate::input::Action::{Confirm, CursorDown};
    let mut c = ctx();
    let flow = FlowScreen::new_game(&c);
    let mut stack = stacked(&mut c, flow);
    // Casual, then back from the familiarity screen: the mode again.
    update(&mut stack, &mut c, &[CursorDown, Confirm]);
    assert_eq!(stack.names(), ["game_flow", "familiarity"]);
    update(&mut stack, &mut c, &[Cancel]);
    assert_eq!(top(&stack), "mode_select");
    // Casual, familiar with other strategy RPGs, the default lead.
    update(&mut stack, &mut c, &[CursorDown, Confirm]);
    update(&mut stack, &mut c, &[CursorDown, Confirm]);
    assert_eq!(top(&stack), "lead_select");
    update(&mut stack, &mut c, &[CursorDown, CursorDown, Confirm]);
    let flow = stack.find::<FlowScreen>();
    let campaign = flow.and_then(FlowScreen::campaign);
    let campaign = campaign.unwrap_or_else(|| panic!("no campaign"));
    assert_eq!(campaign.mode, GameMode::Casual);
    assert_eq!(campaign.tutorial, TutorialLevel::Short);
    assert_eq!(top(&stack), "dialogue");
    // Back from the mode screen: the flow is over.
    let flow = FlowScreen::new_game(&c);
    let mut stack = stacked(&mut c, flow);
    update(&mut stack, &mut c, &[Cancel]);
    assert!(stack.is_empty());
}

/// Nick (0408): after a defeat, `Retry Battle` goes back to Preparations
/// with the loadouts and pack as the player left them.
#[test]
fn retry_goes_back_to_preparations_as_they_were_left() {
    use crate::input::Action::{Confirm, CursorLeft, CursorRight};
    let mut c = ctx();
    let flow = FlowScreen::quick_battle(&mut c).unwrap_or_else(|| panic!("no quick battle"));
    let mut stack = stacked(&mut c, flow);
    assert_eq!(top(&stack), "preparations");
    assert!(stack.find::<BattleScreen>().is_none());
    // Pack an Elixir, then Fight!.
    update(&mut stack, &mut c, &[CursorRight, Confirm, Confirm]);
    // Past the Shop and Options tabs.
    update(
        &mut stack,
        &mut c,
        &[Cancel, CursorRight, CursorRight, CursorRight, Confirm],
    );
    assert_eq!(stack.names(), ["game_flow", "battle"]);
    let elixir = [trpg_core::ItemId::new("elixir")];
    let pack = |s: &ScreenStack| {
        let battle = s.find::<BattleScreen>();
        battle.map(|b| b.state().pack().items.clone())
    };
    assert_eq!(pack(&stack).as_deref(), Some(&elixir[..]));
    // A defeat's Game Over, over the battle, then Retry.
    stack.push(Box::new(GameOverScreen::new(&c, false)));
    // The first press ends Game Over's animation, the second picks.
    update(&mut stack, &mut c, &[Confirm]);
    assert_eq!(top(&stack), "game_over");
    update(&mut stack, &mut c, &[Confirm]);
    assert_eq!(stack.names(), ["game_flow", "preparations"]);
    let prep = stack.find::<PreparationsScreen>();
    let prep = prep.unwrap_or_else(|| panic!("no prep"));
    assert_eq!(prep.setup().pack.items, elixir);
    assert_eq!(prep.setup().stock.count(&elixir[0]), 1);
    // Fight! again: the same battle.
    update(&mut stack, &mut c, &[CursorLeft, Confirm]);
    assert_eq!(pack(&stack).as_deref(), Some(&elixir[..]));
}

/// A battle without Preparations starts straight away, with its default
/// pack.
#[test]
fn a_battle_without_preparations_starts_at_once() {
    let mut c = ctx();
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let mut campaign = trpg_content::new_campaign(&c.game.content, GameMode::Casual, lead);
    campaign.chapter = "test".into();
    let mut flow = FlowScreen::new_game(&c);
    flow.begin(&mut c, campaign);
    let mut stack = stacked(&mut c, flow);
    // Skip the intro scene: Cancel, then Confirm.
    update(&mut stack, &mut c, &[Cancel]);
    update(&mut stack, &mut c, &[crate::input::Action::Confirm]);
    assert_eq!(top(&stack), "battle");
    let pack = stack.find::<BattleScreen>();
    let pack = pack.map(|b| b.state().pack().items.len());
    assert_eq!(pack, Some(2));
}

/// The flow gives a battle the look its map's file names (ADR-0052): when
/// it starts it, when it restarts it, and when a suspended battle is
/// continued.
#[test]
fn a_battle_has_the_look_its_maps_file_names() {
    use crate::input::Action::{Confirm, CursorLeft};
    let look = |s: &ScreenStack, c: &Ctx| {
        let battle = s.find::<BattleScreen>();
        battle.map(|b| b.scene(c).look.tiles)
    };
    // The Quick Battle opens on Preparations: Fight!.
    let fight = |s: &mut ScreenStack, c: &mut Ctx| {
        assert_eq!(top(s), "preparations");
        update(s, c, &[CursorLeft, Confirm]);
    };
    // The Quick Battle's map names no look: a map outdoors.
    let mut c = ctx();
    let flow = FlowScreen::quick_battle(&mut c).unwrap();
    let mut stack = stacked(&mut c, flow);
    fight(&mut stack, &mut c);
    assert_eq!(look(&stack, &c).as_deref(), Some("outdoor"));
    // With one: started.
    let mut c = ctx();
    let map = c.game.content.maps.get_mut("test_small").unwrap();
    map.look.tiles = "indoor".to_owned();
    let flow = FlowScreen::quick_battle(&mut c).unwrap();
    let mut stack = stacked(&mut c, flow);
    fight(&mut stack, &mut c);
    assert_eq!(look(&stack, &c).as_deref(), Some("indoor"));
    // Restarted: Preparations again, then the same look.
    stack.apply_in(&mut c, Transition::pop_with(BattleEnd::Restart));
    fight(&mut stack, &mut c);
    assert_eq!(look(&stack, &c).as_deref(), Some("indoor"));
    // Suspended (the flow is over), then continued.
    let battle = stack.find::<BattleScreen>().cloned().unwrap();
    let suspend = BattleEnd::Suspend(Box::new(battle));
    assert!(stack.apply_in(&mut c, Transition::pop_with(suspend)));
    let continued = FlowScreen::resume(&mut c).unwrap();
    let stack = stacked(&mut c, continued);
    assert_eq!(look(&stack, &c).as_deref(), Some("indoor"));
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
    let mut campaign = trpg_content::new_campaign(&c.game.content, GameMode::Classic, lead);
    // Three unused charges (21 EXP) level her.
    campaign.roster[0].exp = 85;
    let before = campaign.roster[0].clone();
    let mut flow = FlowScreen::new_game(&c);
    flow.begin(&mut c, campaign);
    let mut h = harness(c, flow);
    // Skip the intro, close the battle notes and turn 1's banner; the lead
    // seizes the fort.
    h.keys("d f f f");
    assert_eq!(h.top_screen(), "battle");
    let seize = Command::Act {
        unit: UnitId(1),
        dest: Pos::new(5, 5),
        action: UnitAction::Seize,
    };
    let battle = h.battle_mut();
    battle.unwrap_or_else(|| panic!("no battle")).send(&seize);
    h.keys("f");
    assert_eq!(h.screens(), ["game_flow", "results"]);
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
        let row = stat_row(
            crate::Words::ENGLISH,
            kind,
            was,
            after.stats.get(kind) - was,
        );
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
    let battle = c.game.content.battles.get_mut("test");
    let brigand = &mut battle.unwrap_or_else(|| panic!("no test battle")).enemies[0];
    brigand.pos = Pos::new(4, 6);
    for kind in [StatKind::Hp, StatKind::Str, StatKind::Dex, StatKind::Def] {
        brigand.stats.set(kind, 60);
    }
    brigand.hp = 60;
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let mut campaign = trpg_content::new_campaign(&c.game.content, mode, lead);
    assert_eq!(campaign.roster[1].character.as_ref(), Some(&knight));
    campaign.roster[1].stats.hp = 1;
    let mut flow = FlowScreen::new_game(&c);
    flow.begin(&mut c, campaign);
    let mut h = harness(c, flow);
    // Skip the intro, close the battle notes and turn 1's banner.
    h.keys("d f f f");
    assert_eq!(h.top_screen(), "battle");
    let act = |h: &mut Harness, unit, dest, action| {
        let battle = h.battle_mut();
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
    let fallen = h.battle().map(|b| b.state().fallen().len());
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
    let scene = h.screen::<DialogueScreen>();
    let scene = scene.unwrap_or_else(|| panic!("no scene"));
    assert_eq!(scene.player().scene_id(), "test_victory");
    let text = scene.player().current().text.map(String::from);
    let campaign = h.flow().and_then(FlowScreen::campaign);
    let campaign = campaign.unwrap_or_else(|| panic!("no campaign"));
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
    c.game.content.dialogue.scenes.extend(scenes.scenes);
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let start = |c: &mut Ctx, with_knight: bool, scenes: &[&str]| {
        let mut campaign =
            trpg_content::new_campaign(&c.game.content, GameMode::Classic, lead.clone());
        if !with_knight {
            campaign.roster.truncate(1);
        }
        let mut flow = FlowScreen::new_game(c);
        flow.adopt(c, campaign);
        flow.scenes = scenes.iter().map(|s| (*s).to_owned()).collect();
        flow.then = Then::NextChapter;
        flow.next_scene(c);
        flow
    };
    let playing = |flow: &FlowScreen| {
        let next = flow.next.as_ref().and_then(|s| s.as_any());
        let scene = next.and_then(|s| s.downcast_ref::<DialogueScreen>());
        scene.map(|s| s.player().scene_id().to_owned())
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
    assert_eq!(flow.stage, Stage::SavePrompt);
    assert_eq!(flow.name(), "save_prompt");
}

/// A battle lost on the turn limit with `charges` rewind charges, after
/// the lord waited once, its music `battle_easy`: on Game Over, past its
/// opening animation (0830).
fn lost_flow(c: &mut Ctx, charges: u8) -> ScreenStack {
    use crate::input::Action::Confirm;
    use crate::screens::battle::quick_battle;
    use crate::screens::battle::testing::battle_charged;
    use trpg_core::Objective;
    let quick = quick_battle(&c.game.content).unwrap_or_else(|e| panic!("{e}"));
    let rout = Objective::Rout {
        turn_limit: Some(1),
    };
    let state = battle_charged(
        c,
        quick.map().clone(),
        quick.units().to_vec(),
        rout,
        charges,
    );
    let mut battle = BattleScreen::new(state).with_music(Some("battle_easy".to_owned()));
    let lord = battle.state().units()[0].clone();
    battle.send(&Command::Act {
        unit: lord.id,
        dest: lord.pos,
        action: UnitAction::Wait,
    });
    battle.send(&Command::EndPhase);
    let mut flow = FlowScreen::new_game(c);
    flow.show(Stage::Battle, Box::new(battle));
    let mut stack = stacked(c, flow);
    for _ in 0..3000 {
        // A press ends Game Over's opening animation.
        let on_game_over = top(&stack) == "game_over";
        let input = FrameInput::new(vec![Confirm], 0.25, vec![]);
        stack.update(c, &input);
        if on_game_over {
            return stack;
        }
    }
    panic!("never lost: {:?}", stack.names());
}

/// The labels of Game Over's menu, on top of `stack`.
fn game_over_items(stack: &ScreenStack, c: &Ctx) -> Vec<String> {
    let screen = stack.find::<GameOverScreen>();
    let menu = screen
        .unwrap_or_else(|| panic!("no game over"))
        .view(c)
        .menu;
    let menu = menu.unwrap_or_else(|| panic!("no menu"));
    menu.items
        .iter()
        .map(|i| crate::widgets::menu::text_of(&i.label).to_owned())
        .collect()
}

/// Acceptance (0822): Game Over lists `Rewind` while charges are left;
/// picking a point plays on, undecided, with one charge fewer.
#[test]
fn game_over_rewinds_a_lost_battle() {
    use crate::input::Action::{Cancel, Confirm};
    let mut c = ctx();
    let mut stack = lost_flow(&mut c, 2);
    // Over the lost battle.
    assert_eq!(stack.names(), ["game_flow", "battle", "game_over"]);
    // 0809: `defeat` was asked for once, at the banner.
    let music = |c: &mut Ctx| -> Vec<String> {
        let cue = |r: AudioRequest| match r {
            AudioRequest::PlayMusic { cue, .. } => Some(cue),
            AudioRequest::StopMusic => Some("-".to_owned()),
            _ => None,
        };
        c.audio.take().into_iter().filter_map(cue).collect()
    };
    assert_eq!(music(&mut c), ["defeat"]);
    assert_eq!(
        game_over_items(&stack, &c),
        [
            c.text("game_over.rewind"),
            c.text("game_over.retry"),
            c.text("game_over.title")
        ]
    );
    // Backing out of the rewind screen returns to Game Over.
    update(&mut stack, &mut c, &[Confirm]);
    assert_eq!(stack.names(), ["game_flow", "battle", "rewind"]);
    update(&mut stack, &mut c, &[Cancel]);
    assert_eq!(top(&stack), "game_over");
    // As it was left: its opening animation doesn't play again (0830).
    let screen = stack.find::<GameOverScreen>();
    let screen = screen.unwrap_or_else(|| panic!("no game over"));
    assert!(screen.finished());
    assert!(!screen.shows_battle());
    // `defeat` plays on all the while (0809).
    update(&mut stack, &mut c, &[]);
    assert_eq!(music(&mut c), [""; 0]);
    // Rewind again and pick the point: the battle is on again.
    update(&mut stack, &mut c, &[Confirm]);
    // The oldest point: before the lord's first action.
    let down = [crate::input::Action::CursorDown; 10];
    update(&mut stack, &mut c, &down);
    update(&mut stack, &mut c, &[Confirm, Confirm]);
    assert_eq!(stack.names(), ["game_flow", "battle"]);
    let b = stack.find::<BattleScreen>();
    let b = b.unwrap_or_else(|| panic!("no battle"));
    assert!(b.state().outcome().is_none());
    assert_eq!(b.history().charges_left(), 1);
    assert_eq!(b.history().len(), 0);
    // The battle's own track again (0809), asked for once.
    assert_eq!(music(&mut c), ["battle_easy"]);
    // And it carries on: it is not sent back to Game Over.
    update(&mut stack, &mut c, &[]);
    assert_eq!(top(&stack), "battle");
    assert_eq!(music(&mut c), [""; 0]);
}

/// Acceptance (0822): no charges left, no `Rewind`; `Retry Battle` is first.
#[test]
fn game_over_without_charges_has_no_rewind() {
    let mut c = ctx();
    let stack = lost_flow(&mut c, 0);
    assert_eq!(
        game_over_items(&stack, &c),
        [c.text("game_over.retry"), c.text("game_over.title")]
    );
}

/// `Title` on Game Over pops the battle and the flow: back to the title,
/// with no campaign.
#[test]
fn title_on_game_over_ends_the_flow() {
    use crate::input::Action::{Confirm, CursorDown};
    let mut c = ctx();
    let mut stack = lost_flow(&mut c, 0);
    c.game.campaign_mode = Some(GameMode::Classic);
    update(&mut stack, &mut c, &[CursorDown, Confirm]);
    assert!(stack.is_empty());
    assert_eq!(c.game.campaign_mode, None);
}

/// The first text of the hold's closing scene (`test_hold_won`, 0319)
/// played by a new game in `mode` (`with_knight` or not in the army), and
/// the campaign after it starts.
fn hold_won(mode: GameMode, with_knight: bool) -> (Option<String>, Campaign) {
    let mut c = ctx();
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let mut campaign = trpg_content::new_campaign(&c.game.content, mode, lead);
    if !with_knight {
        campaign.roster.truncate(1);
    }
    let mut flow = FlowScreen::new_game(&c);
    flow.adopt(&mut c, campaign);
    flow.scenes = std::iter::once("test_hold_won".to_owned()).collect();
    flow.then = Then::NextChapter;
    flow.next_scene(&mut c);
    let next = flow.next.as_ref().and_then(|s| s.as_any());
    let scene = next.and_then(|s| s.downcast_ref::<DialogueScreen>());
    let text = scene.and_then(|s| s.player().current().text.map(String::from));
    let campaign = flow.campaign().cloned();
    (text, campaign.unwrap_or_else(|| panic!("no campaign")))
}

/// A scene's death (`@dies`, Hollis, 0319): with the knight in the army,
/// the closing scene plays their version and they leave the army for good,
/// in Classic and Casual alike, their weapons to the stock; without them
/// (died earlier), the lead's version plays and nobody leaves.
#[test]
fn a_closing_scene_kills_the_knight_in_every_mode_or_plays_without_them() {
    let knight = trpg_core::CharacterId("test_knight".into());
    for mode in [GameMode::Classic, GameMode::Casual] {
        let c = ctx();
        let lead = LeadProfile::new("Mara", LeadGender::Female);
        let before = trpg_content::new_campaign(&c.game.content, mode, lead);
        let carried = before.roster[1].loadout.weapons.iter().flatten().count();
        let (text, after) = hold_won(mode, true);
        assert_eq!(
            text.as_deref(),
            Some("Go, Mara. I'll hold the line."),
            "{mode:?}"
        );
        assert!(after.member(&knight).is_none(), "{mode:?}");
        assert_eq!(after.roster.len(), before.roster.len() - 1);
        let stocked = after.stock.weapons.len() - before.stock.weapons.len();
        assert_eq!(stocked, carried, "{mode:?}");
    }
    let (text, after) = hold_won(GameMode::Classic, false);
    assert_eq!(
        text.as_deref(),
        Some("Mara holds the line alone, and remembers who should be here.")
    );
    assert_eq!(after.roster.len(), 1);
}

/// The Quick Battle's army with the lord and the knight's C conversation
/// unlocked, at chapter `chapter`.
/// Ticket 0465: the campaign takes in a set of lessons as long as its own
/// but different ("Restart with the lessons" took one back as the answer to
/// the box went in), or its saves would keep the old one.
#[test]
fn the_campaign_takes_in_lessons_that_changed_without_growing() {
    let mut c = ctx();
    let mut campaign = bonded_army(&c, QUICK_CHAPTER);
    campaign.lessons_done = ["last".to_owned()].into();
    c.game.campaign_lessons = ["answered".to_owned()].into();
    FlowScreen::take_in(&mut campaign, &c, 0.0);
    assert_eq!(campaign.lessons_done, c.game.campaign_lessons);
}

fn bonded_army(c: &Ctx, chapter: &str) -> Campaign {
    let content = &c.game.content;
    let def = &content.battles["quick"];
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let mut campaign = battle_campaign(content, def, GameMode::Classic, lead);
    campaign.chapter = chapter.into();
    let pair = trpg_core::SupportPair::new(
        trpg_core::CharacterId("test_lord".into()),
        trpg_core::CharacterId("test_knight".into()),
    );
    campaign.supports.gain(&pair, 20, &content.supports);
    campaign
}

/// 1003: with something to list, camp opens before a battle's
/// Preparations; what was viewed there goes into the battle's setup and
/// the campaign. With nothing to list, it is skipped (the Quick Battle
/// test above goes straight to Preparations).
#[test]
fn camp_opens_before_preparations_and_the_battle_takes_what_it_changed() {
    use crate::input::Action::Confirm;
    let mut c = ctx();
    let campaign = bonded_army(&c, QUICK_CHAPTER);
    let mut flow = FlowScreen::new_game(&c);
    flow.begin(&mut c, campaign);
    let mut stack = stacked(&mut c, flow);
    assert_eq!(stack.names(), ["game_flow", "camp"]);
    // Watch the lord and the knight's NEW conversation, then break camp.
    update(&mut stack, &mut c, &[Confirm, Confirm, Confirm]);
    assert_eq!(top(&stack), "dialogue");
    for _ in 0..20 {
        if top(&stack) != "dialogue" {
            break;
        }
        update(&mut stack, &mut c, &[Confirm]);
    }
    assert_eq!(top(&stack), "camp");
    // Back to the units, to the sections, then to Break camp.
    update(&mut stack, &mut c, &[Cancel, Cancel, Cancel]);
    update(&mut stack, &mut c, &[Confirm]);
    assert_eq!(stack.names(), ["game_flow", "preparations"]);
    let pair = trpg_core::SupportPair::new(
        trpg_core::CharacterId("test_lord".into()),
        trpg_core::CharacterId("test_knight".into()),
    );
    let prep = stack
        .find::<PreparationsScreen>()
        .map(|p| p.setup().clone());
    let rank = prep.and_then(|s| s.bonds.rank(&pair));
    assert_eq!(rank, Some(trpg_core::SupportRank::C));
    let campaign = stack.find::<FlowScreen>().and_then(FlowScreen::campaign);
    let rank = campaign.and_then(|c| c.supports.rank(&pair));
    assert_eq!(rank, Some(trpg_core::SupportRank::C));
}

/// 1003: camp opens between two chapters, before the next one's scenes,
/// and only once between two battles: the next battle's Preparations
/// come straight after its chapter's scenes.
#[test]
fn camp_opens_between_chapters_once() {
    use crate::input::Action::Confirm;
    let mut c = ctx();
    if let Some(test) = c.game.content.chapters.get_mut("test") {
        test.next = Some(QUICK_CHAPTER.into());
    }
    let campaign = bonded_army(&c, "test");
    let mut flow = FlowScreen::new_game(&c);
    flow.adopt(&mut c, campaign);
    flow.chapter = c.game.content.chapters.get("test").cloned();
    flow.next_chapter(&mut c);
    let mut stack = stacked(&mut c, flow);
    assert_eq!(stack.names(), ["game_flow", "camp"]);
    update(&mut stack, &mut c, &[Cancel]);
    update(&mut stack, &mut c, &[Confirm]);
    // The Quick Battle's chapter has no scenes: its Preparations.
    assert_eq!(stack.names(), ["game_flow", "preparations"]);
    let campaign = stack.find::<FlowScreen>().and_then(FlowScreen::campaign);
    assert_eq!(campaign.map(|c| c.chapter.as_str()), Some(QUICK_CHAPTER));
}

/// Story victory -> deterministic group -> ordinary battle -> map, without
/// advancing the story a second time. Includes suspend and restart rebuilding.
#[test]
fn a_story_win_spawns_a_group_which_returns_to_the_map_after_victory() {
    use trpg_core::{Faction, Outcome, WorldCommand, WorldEvent};
    let (mut c, mut flow, map) = story_won_with_training();
    let mut campaign = flow.campaign.clone().expect("campaign");
    assert_eq!(campaign.chapter, "test_wm2");
    let group = map
        .random_at("training", &campaign)
        .expect("spawned")
        .clone();
    assert_eq!(group.level, 4);
    let events = campaign
        .world_command(&map, &WorldCommand::TravelTo("training".into()))
        .expect("travel");
    assert!(events.contains(&WorldEvent::BattleAhead("training".into())));
    flow.campaign = Some(campaign);
    flow.arrived(&mut c, "training");
    let fight = flow.fight.as_ref().expect("random fight").clone();
    assert_eq!(fight.def.enemies.len(), group.size as usize);
    assert_eq!(fight.def.difficulty, trpg_core::Difficulty::Easy);
    assert_eq!(fight.prep.setup.rewind_charges, 2);
    assert!(fight.def.enemies.iter().all(|u| u.level == 4));
    // Suspend through the real flow, then continue and check restart's def.
    let mut stack = stacked(&mut c, flow);
    update(
        &mut stack,
        &mut c,
        &[
            crate::input::Action::CursorLeft,
            crate::input::Action::Confirm,
        ],
    );
    let battle = stack.find::<BattleScreen>().expect("battle").clone();
    stack.apply_in(
        &mut c,
        Transition::pop_with(BattleEnd::Suspend(Box::new(battle))),
    );
    let mut flow = FlowScreen::resume(&mut c).expect("resume random group");
    assert_eq!(flow.fight.as_ref().expect("resumed").def, fight.def);
    let rng = flow.campaign.as_ref().expect("campaign").rng.clone();
    flow.restart(&mut c);
    assert_eq!(flow.fight.as_ref().expect("retry").def, fight.def);
    assert_eq!(flow.campaign.as_ref().expect("campaign").rng, rng);
    let (mut state, _) = BattleState::new(fight.prep.setup.clone());
    for _ in 0..100 {
        if state.outcome().is_some() {
            break;
        }
        let legal = trpg_core::legal_commands(&state);
        let attack = legal.iter().find(|cmd| {
            matches!(
                cmd,
                Command::Act {
                    action: UnitAction::Attack { .. },
                    ..
                }
            )
        });
        let advance = legal
            .iter()
            .filter_map(|cmd| {
                let Command::Act {
                    unit,
                    dest,
                    action: UnitAction::Wait,
                } = cmd
                else {
                    return None;
                };
                let faction = state.unit(*unit)?.faction;
                let distance = state
                    .units()
                    .iter()
                    .filter(|u| (u.faction == Faction::Player) != (faction == Faction::Player))
                    .map(|u| (u.pos.x - dest.x).abs() + (u.pos.y - dest.y).abs())
                    .min()?;
                Some((distance, cmd))
            })
            .min_by_key(|(d, _)| *d)
            .map(|(_, cmd)| cmd);
        let cmd = attack
            .or(advance)
            .or_else(|| legal.first())
            .expect("legal")
            .clone();
        state.apply(&cmd).expect("legal command");
    }
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    flow.won(&mut c, &state, 2);
    flow.next_scene(&mut c);
    assert_eq!(flow.stage, Stage::WorldMap);
    let campaign = flow.campaign.expect("campaign");
    assert_eq!(
        campaign.chapter, "test_wm2",
        "training never advances the story"
    );
    assert!(map.is_cleared("training", &campaign));
    assert!(map.random_at("training", &campaign).is_none());
    assert_eq!(campaign.world.as_ref().expect("world").at, "training");
}

fn story_won_with_training() -> (Ctx, FlowScreen, trpg_core::WorldMap) {
    use trpg_core::WorldCommand;
    let mut c = ctx();
    let lead = LeadProfile::new("Mara", trpg_core::LeadGender::Female);
    let mut campaign = trpg_content::battle_campaign(
        &c.game.content,
        &c.game.content.battles["test"],
        GameMode::Casual,
        lead,
    );
    campaign.chapter = "test_wm1".into();
    campaign.flags.insert("test_skirmishes".into(), true);
    // This integration exercises the loop every run, with spawning forced
    // through the same tunable data used by production.
    let map = &mut c
        .game
        .content
        .worldmaps
        .get_mut("test_world")
        .expect("map")
        .map;
    map.skirmishes.as_mut().expect("rules").spawn = (2, 2);
    let map = map.clone();
    campaign.enter_world(&map);
    campaign.clear_node(&map, "training", None, None);
    campaign
        .world_command(&map, &WorldCommand::TravelTo("ford".into()))
        .expect("travel");
    // Strengthen the fixture army, so the test is about the flow, not balance.
    for u in &mut campaign.roster {
        u.stats.str = 100;
        u.stats.dex = 100;
        u.stats.def = 100;
        u.stats.res = 100;
    }
    let mut flow = FlowScreen::empty();
    flow.adopt(&mut c, campaign);
    flow.arrived(&mut c, "ford");
    flow.start_battle(&mut c);
    let fight = flow.fight.as_ref().expect("story fight");
    let (mut state, _) = BattleState::new(fight.prep.setup.clone());
    state
        .apply(&Command::Act {
            unit: UnitId(1),
            dest: Pos::new(5, 5),
            action: UnitAction::Seize,
        })
        .expect("story win");
    flow.won(&mut c, &state, 3);
    assert_eq!(flow.stage, Stage::Results);
    flow.world_won(&mut c, "ford");
    (c, flow, map)
}

#[test]
fn training_on_a_cleared_story_place_does_not_advance_the_story() {
    let (mut c, mut flow, map) = story_won_with_training();
    let campaign = flow.campaign.as_mut().expect("campaign");
    let world = campaign.world.as_mut().expect("world");
    let group = world.skirmishes.remove("training").expect("group");
    world.at = "home".into();
    world.skirmishes.insert("ford".into(), group.clone());
    campaign
        .world_command(&map, &trpg_core::WorldCommand::TravelTo("ford".into()))
        .expect("arrive");
    let rng = campaign.rng.clone();
    flow.arrived(&mut c, "ford");
    assert_eq!(flow.chapter.as_ref().expect("chapter").battle, group.battle);
    flow.world_won(&mut c, "ford");
    assert_eq!(flow.stage, Stage::WorldMap);
    let campaign = flow.campaign.expect("campaign");
    assert_eq!(campaign.chapter, "test_wm2");
    assert_eq!(campaign.rng, rng);
    assert!(map.random_at("ford", &campaign).is_none());
    assert!(map.is_cleared("ford", &campaign));
}

#[test]
fn flow_declares_its_actions() {
    let screen = FlowScreen::new_game(&crate::screen::tests::ctx());
    crate::screens::actions::assert_declares(&screen, crate::screens::actions::FLOW);
}
