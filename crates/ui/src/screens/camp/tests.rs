use insta::assert_snapshot;

use trpg_content::battle_campaign;
use trpg_core::{GameMode, LeadGender, LeadProfile, SupportPair};

use super::*;
use crate::audio::AudioRequest;
use crate::harness::Harness;
use crate::screen::tests::ctx;
use crate::screens::popped_with;

fn c(id: &str) -> CharacterId {
    CharacterId(id.into())
}

fn pair(a: &str, b: &str) -> SupportPair {
    SupportPair::new(c(a), c(b))
}

/// The Quick Battle's army (the test characters: the lord, the knight,
/// the mage, …), with nothing unlocked or seen.
fn army(ctx: &Ctx) -> Campaign {
    let content = &ctx.game.content;
    let def = &content.battles["quick"];
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    battle_campaign(content, def, GameMode::Classic, lead)
}

/// [`army`], the lord and the knight at their C conversation (unlocked,
/// not viewed), the lord and the mage at C seen and B unlocked.
fn bonded(ctx: &Ctx) -> Campaign {
    let table = &ctx.game.content.supports;
    let mut game = army(ctx);
    game.supports.gain(&pair("test_lord", "test_knight"), 20, table);
    let mage = pair("test_lord", "test_mage");
    game.supports.gain(&mage, 20, table);
    game.supports.view(&mage, table).unwrap();
    game.supports.gain(&mage, 80, table);
    game
}

/// One frame with `actions`.
fn update(s: &mut CampScreen, ctx: &mut Ctx, actions: &[Action]) -> Transition {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    s.update(ctx, &input)
}

/// The labels of `menu`'s items, each with its suffix.
fn labels(menu: &crate::widgets::MenuView) -> Vec<String> {
    let label = |i: &crate::widgets::MenuItemView| {
        let text = crate::widgets::menu::text_of(&i.label);
        match &i.suffix {
            Some((suffix, _)) => format!("{text} {suffix}"),
            None => text.to_string(),
        }
    };
    menu.items.iter().map(label).collect()
}

#[test]
fn camp_has_nothing_to_list_until_a_conversation_or_event_is_there() {
    let ctx = ctx();
    assert!(!CampScreen::has_anything(&ctx, &army(&ctx)));
    assert!(CampScreen::has_anything(&ctx, &bonded(&ctx)));
    // The test event waits for the test chapter's battle.
    let mut won = army(&ctx);
    won.battles_won.insert("test".into());
    assert!(CampScreen::has_anything(&ctx, &won));
}

/// The Supports list: the units with a conversation, NEW beside the ones
/// with one unlocked; a unit's list has a row per conversation, its rank
/// and NEW on the unlocked one.
#[test]
fn the_supports_list_shows_ranks_and_what_is_new() {
    let mut ctx = ctx();
    let mut s = CampScreen::new(&ctx, bonded(&ctx));
    let view = s.view(&ctx);
    assert_eq!(
        labels(&view.sections),
        ["Supports NEW", "Camp events", "Break camp"]
    );
    let list = view.list.unwrap();
    assert_eq!(
        labels(&list),
        ["Test Lord NEW", "Test Knight NEW", "Test Mage NEW"]
    );
    assert!(list.items.iter().all(|i| !i.focused), "focus in the sections");
    update(&mut s, &mut ctx, &[Action::Confirm, Action::Confirm]);
    let view = s.view(&ctx);
    assert_eq!(view.list_title, "Test Lord's supports");
    assert_eq!(
        labels(&view.list.unwrap()),
        ["C  Test Knight NEW", "C  Test Mage", "B  Test Mage NEW"]
    );
    // Back to the units, on the lord.
    update(&mut s, &mut ctx, &[Action::Cancel]);
    assert_eq!(s.rows()[s.list.focus()], Row::Unit(c("test_lord")));
}

/// Watching a NEW conversation views it (1002's rule: the rank is gained
/// then) and plays its scene; a seen one plays again and changes nothing.
#[test]
fn watching_a_new_conversation_raises_the_rank() {
    let mut ctx = ctx();
    let mut s = CampScreen::new(&ctx, bonded(&ctx));
    let knight = pair("test_lord", "test_knight");
    let t = update(&mut s, &mut ctx, &[Action::Confirm, Action::Confirm, Action::Confirm]);
    assert!(matches!(t, Transition::Push(_)));
    assert_eq!(s.campaign().supports.rank(&knight), Some(SupportRank::C));
    // The seen C of the mage plays again; the rank stays.
    let mage = pair("test_lord", "test_mage");
    let t = update(&mut s, &mut ctx, &[Action::CursorDown, Action::Confirm]);
    assert!(matches!(t, Transition::Push(_)));
    assert_eq!(s.campaign().supports.rank(&mage), Some(SupportRank::C));
    let state = s.campaign().supports.state(&mage, &ctx.game.content.supports);
    assert_eq!(state.unwrap().unlocked(), Some(SupportRank::B));
}

/// A partner who left the army keeps the conversations seen (they can be
/// watched again), but an unlocked one is gone with them.
#[test]
fn a_departed_partners_seen_conversations_stay() {
    let ctx = ctx();
    let mut game = bonded(&ctx);
    let table = &ctx.game.content.supports;
    game.dies_in_scene(&c("test_mage"), table);
    let s = CampScreen::new(&ctx, game.clone());
    let lord = s.list_rows_of(&ctx, Focus::Talks(c("test_lord")));
    let rows: Vec<(SupportRank, bool)> = lord
        .iter()
        .filter_map(|r| match r {
            Row::Talk(t) if t.partner == c("test_mage") => Some((t.rank, t.new)),
            _ => None,
        })
        .collect();
    assert_eq!(rows, [(SupportRank::C, false)]);
}

/// Camp events: only the available ones, NEW until watched; watching one
/// marks it seen in the campaign.
#[test]
fn camp_events_show_when_available_and_are_remembered() {
    let mut ctx = ctx();
    let mut game = army(&ctx);
    let s = CampScreen::new(&ctx, game.clone());
    assert!(s.list_rows_of(&ctx, Focus::Events).is_empty());
    game.battles_won.insert("test".into());
    let mut s = CampScreen::new(&ctx, game);
    update(&mut s, &mut ctx, &[Action::CursorDown]);
    let view = s.view(&ctx);
    assert_eq!(labels(&view.sections)[1], "Camp events NEW");
    assert_eq!(labels(&view.list.unwrap()), ["Around the campfire NEW"]);
    let t = update(&mut s, &mut ctx, &[Action::Confirm, Action::Confirm]);
    assert!(matches!(t, Transition::Push(_)));
    assert!(s.campaign().camp_seen.contains("test_campfire"));
    s.refresh(&ctx, 0);
    assert_eq!(labels(&s.view(&ctx).list.unwrap()), ["Around the campfire"]);
}

/// Cancel in the sections only moves the focus to Break camp; Confirm
/// there pops with the campaign as camp left it.
#[test]
fn break_camp_pops_with_the_campaign() {
    let mut ctx = ctx();
    let mut s = CampScreen::new(&ctx, bonded(&ctx));
    assert!(matches!(update(&mut s, &mut ctx, &[Action::Cancel]), Transition::None));
    assert_eq!(s.section(), Section::Leave);
    assert_eq!(s.view(&ctx).empty, ctx.text("camp.leave_line"));
    let left = popped_with::<Campaign>(update(&mut s, &mut ctx, &[Action::Confirm]));
    assert_eq!(left, Some(bonded(&ctx)));
}

/// An empty list can't be entered.
#[test]
fn an_empty_list_keeps_the_focus_in_the_sections() {
    let mut ctx = ctx();
    let mut s = CampScreen::new(&ctx, bonded(&ctx));
    update(&mut s, &mut ctx, &[Action::CursorDown, Action::Confirm]);
    let view = s.view(&ctx);
    assert!(view.sections_focused);
    assert!(view.list.is_none());
    assert_eq!(view.empty, ctx.text("camp.empty.events"));
}

/// Camp plays a track of the `camp` pool as it opens, and the next one
/// after the first ends, fades and a quiet passes.
#[test]
fn camp_plays_the_camp_pool_in_turn() {
    let mut ctx = ctx();
    let pool = ctx.game.content.audio.pools[POOL].clone();
    let mut s = CampScreen::new(&ctx, bonded(&ctx));
    update(&mut s, &mut ctx, &[]);
    let asked: Vec<AudioRequest> = ctx.audio.take();
    let first = match asked.as_slice() {
        [AudioRequest::PlayMusic { cue }] => cue.clone(),
        other => panic!("{other:?}"),
    };
    let at = pool.iter().position(|c| *c == first).unwrap();
    let length = f64::from(ctx.game.content.audio.music[&first].length_ms) / 1000.0;
    ctx.clock_s += length;
    update(&mut s, &mut ctx, &[]);
    assert_eq!(ctx.audio.take(), [AudioRequest::StopMusic]);
    ctx.clock_s += music::QUIET_S + 1.0;
    update(&mut s, &mut ctx, &[]);
    let next = AudioRequest::PlayMusic {
        cue: pool[(at + 1) % pool.len()].clone(),
    };
    assert_eq!(ctx.audio.take(), [next]);
}

/// Scripted keys: open Supports, watch the lord's NEW conversation with
/// the knight to its end; back at camp, the pair is at C and nothing of
/// theirs is NEW.
#[test]
fn watching_a_conversation_from_the_keys() {
    let c0 = ctx();
    let mut h = Harness::with_screen(Box::new(CampScreen::new(&c0, bonded(&c0))));
    h.keys("f f f");
    assert_eq!(h.top_screen(), "dialogue");
    for _ in 0..20 {
        if h.top_screen() != "dialogue" {
            break;
        }
        h.keys("f");
    }
    assert_eq!(h.top_screen(), CampScreen::NAME);
    let camp = h.screen::<CampScreen>().unwrap();
    let knight = pair("test_lord", "test_knight");
    assert_eq!(camp.campaign().supports.rank(&knight), Some(SupportRank::C));
    let view = camp.view(h.game().ctx());
    assert_eq!(view.list_title, "Test Lord's supports");
    assert_eq!(
        labels(&view.list.unwrap()),
        ["C  Test Knight", "C  Test Mage", "B  Test Mage NEW"]
    );
}

/// The Supports list, the lord's list, and the Camp events list.
#[test]
fn camp_snapshots() {
    let c0 = ctx();
    let mut game = bonded(&c0);
    game.battles_won.insert("test".into());
    let mut h = Harness::with_screen(Box::new(CampScreen::new(&c0, game)));
    h.frame(&[], crate::harness::FRAME_DT);
    assert_snapshot!("supports", h.snapshot());
    h.keys("f f");
    assert_snapshot!("talks", h.snapshot());
    h.keys("d d Down");
    assert_snapshot!("events", h.snapshot());
}
