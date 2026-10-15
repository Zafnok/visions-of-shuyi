//! Tests of the combat shot (ticket 0832), on the combat test cinematic
//! (`assets-test/cinematics/combat.ron`): six seconds of the lead, put
//! beside the brigand of the test battle, attacking it on the combat
//! stage. What happens is read from the stage's view (ADR-0054); only the
//! snapshot reads cells.

use insta::assert_snapshot;
use trpg_core::{Event, Faction, Side, UnitId};

use super::*;
use crate::console::ConsoleSpec;
use crate::screen::tests::ctx;
use crate::screens::battle::playback::{Playback, TIMINGS};

const SCREEN: Rect = Rect::new(0, 0, 100, 32);
const LEAD: UnitId = UnitId(1);
const BRIGAND: UnitId = UnitId(3);

fn player(ctx: &Ctx) -> CinematicPlayer {
    CinematicPlayer::new(ctx, "combat", None).unwrap()
}

/// The shot's combat, as loaded.
fn shot(ctx: &Ctx) -> trpg_content::CombatShot {
    match &ctx.content.cinematics["combat"].shots[0].shot {
        Shot::Combat(shot) => shot.clone(),
        other => panic!("not a combat: {other:?}"),
    }
}

/// The fight the rules work out for the shot, played back as a battle
/// plays it.
fn playback(ctx: &Ctx) -> Playback {
    let lead = LeadProfile::new(
        ctx.words().name(&ctx.content.names, LEAD_ID),
        LeadGender::Male,
    );
    let fight = trpg_content::cinematic::fight(&ctx.content, &shot(ctx), lead).unwrap();
    let words = ctx.words();
    let before = &fight.before;
    let played = Playback::new(words, &fight.events, before, fight.after.fallen(), TIMINGS);
    played.unwrap()
}

#[test]
fn half_a_second_in_the_stage_shows_the_two_named_units_the_attacker_striking() {
    let ctx = ctx();
    let player = player(&ctx);
    assert_eq!(
        player.frame(&ctx, 0.5, SCREEN),
        Frame::Combat { seconds: 0.5 }
    );
    let view = player.stage(&ctx, 0.5).unwrap();
    let names = view.fighters.each_ref().map(|f| f.name.as_str());
    assert_eq!(names, ["Ellery", "Brigand"]);
    let (lead, brigand) = (&view.fighters[0], &view.fighters[1]);
    assert_eq!(
        (lead.faction, brigand.faction),
        (Faction::Player, Faction::Enemy)
    );
    // Both have their pictures; the lead stands on the left, hand to hand.
    assert!(lead.picture.is_some() && brigand.picture.is_some());
    assert_eq!((view.left, view.melee), (0, true));
    // After the 0.3 s intro, the lead's name flashes before its strike.
    assert_eq!(view.striker, Some(0));
    // No map: the shot is the stage alone.
    assert_eq!(player.scene(0.5), None);
}

#[test]
fn the_fight_is_the_rules_own_played_as_a_battle_plays_it() {
    let ctx = ctx();
    let player = player(&ctx);
    let pb = playback(&ctx);
    let bout = &pb.bouts()[0];
    assert_eq!((bout.attacker.unit, bout.defender.unit), (LEAD, BRIGAND));
    // Each moment is the battle's stage at that moment of the playback,
    // over the ground the brigand stands on, in the battle's light.
    let def = &ctx.content.battles["test"];
    let tile = def.map.tiles.get(Pos::new(8, 3)).unwrap();
    let ground = ctx
        .content
        .terrain
        .display
        .get(*tile)
        .map(|t| t.id.as_str());
    assert_eq!(ground, Some("plain"));
    let light = ctx.content.battle_look(def).light;
    for t in [0.0, 0.2, 0.45, 0.7, 1.0, 1.6, 2.4, 3.3] {
        let mut now = pb.clone();
        now.tick(t, false);
        let battle = stage::view(&ctx, &now, ground, &light);
        assert!(battle.is_some());
        assert_eq!(player.stage(&ctx, t), battle, "{t}");
    }
    // The numbers are the forecast's and the HP the strikes': the lead
    // has a strike and the brigand a counter.
    let view = player.stage(&ctx, 0.5).unwrap();
    assert!(view.fighters.iter().all(|f| f.numbers.is_some()));
    let last = bout.strikes.last().unwrap().target_hp_after;
    let end = player.stage(&ctx, 5.99).unwrap();
    let target = usize::from(bout.strikes.last().unwrap().by == Side::Attacker);
    assert_eq!(end.fighters[target].hp, last);
}

#[test]
fn it_plays_at_the_fights_own_speed_and_holds_its_end() {
    let ctx = ctx();
    let player = player(&ctx);
    let length = playback(&ctx).total();
    // The fight is shorter than the shot.
    assert!(length > 1.0 && length < 6.0, "{length}");
    let end = player.stage(&ctx, length).unwrap();
    assert_eq!(
        player.frame(&ctx, 5.9, SCREEN),
        Frame::Combat { seconds: length }
    );
    assert_eq!(player.stage(&ctx, 5.9), Some(end.clone()));
    assert_ne!(player.stage(&ctx, length - 0.5), Some(end));
    // The same time is the same frame, a loop later too.
    assert_eq!(player.stage(&ctx, 0.7), player.stage(&ctx, 6.7));
}

#[test]
fn the_scenes_an_attack_sets_off_are_left_out() {
    let ctx = ctx();
    let fight = trpg_content::cinematic::fight(&ctx.content, &shot(&ctx), ctx.lead.clone());
    let scenes = fight.unwrap().events;
    let scenes = scenes
        .iter()
        .filter(|e| matches!(e, Event::SceneTriggered { .. }));
    // The test battle sets none off: a playback with a scene would stop
    // at it, which a cinematic can't wait for (`Duel::new` drops them).
    assert_eq!(scenes.count(), 0);
}

#[test]
fn the_lead_is_the_cinematics() {
    let ctx = ctx();
    let mara = LeadProfile::new("Mara", LeadGender::Female);
    let player = CinematicPlayer::new(&ctx, "combat", Some(mara)).unwrap();
    let lead = player.stage(&ctx, 0.5).unwrap().fighters[0].clone();
    assert_eq!(lead.name, "Mara");
    let female = ctx.content.battlers.pick(["lead_f", "lead", "exile"]);
    assert_eq!(lead.picture.as_ref(), female);
}

#[test]
fn a_fight_the_rules_no_longer_allow_is_blank() {
    let mut ctx = ctx();
    // A rebalance after the content was checked: the brigand out of reach.
    let unreachable = Shot::Combat(trpg_content::CombatShot {
        from: None,
        ..shot(&ctx)
    });
    let def = ctx.content.cinematics.get_mut("combat").unwrap();
    def.shots[0].shot = unreachable;
    let player = player(&ctx);
    assert_eq!(player.frame(&ctx, 0.5, SCREEN), Frame::Blank);
    assert_eq!(player.stage(&ctx, 0.5), None);
    let p = &ctx.palette;
    let blank = Cell::new(' ', p.get(UiColor::Text), p.get(UiColor::Black));
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, blank);
    player.draw(&ctx, 0.5, 1.0, &mut buf);
    assert!(buf.sprites().is_empty());
    assert_eq!(buf.get(50, 10), Some(&blank));
}

#[test]
fn a_frame_of_the_fight() {
    let ctx = ctx();
    let player = player(&ctx);
    let p = &ctx.palette;
    let stale = Cell::new('x', p.get(UiColor::Enemy), p.get(UiColor::PanelBg));
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, stale);
    // The lead's first strike has landed: its result over the brigand.
    player.draw(&ctx, 0.9, 1.0, &mut buf);
    let view = player.stage(&ctx, 0.9).unwrap();
    assert_eq!(view.result.as_ref().map(|r| r.0), Some(1));
    // Painted as the battle paints its stage, the help bar's rows black.
    let blank = Cell::new(' ', p.get(UiColor::Text), p.get(UiColor::Black));
    let mut battle = GlyphBuffer::with_spec(ConsoleSpec::PC, blank);
    stage::glyph::paint(&ctx, &view, &mut battle);
    assert_eq!(buf, battle);
    assert_snapshot!("combat_first_strike", buf.to_snapshot(&ctx.palette));
}
