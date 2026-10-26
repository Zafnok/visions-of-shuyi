use shuyi_core::play;
use trpg_core::{BattleSetup, CharacterId, Event, Trigger, TriggerWhen, Turn};

use super::*;
use crate::BaselineBot;
use crate::testkit::{
    Scripted, attack, content, enemy, lord, player, rigged, setup, start, stationary, tile, weak,
};

fn personas() -> Personas {
    personas_from_ron(include_str!("../../personas.ron")).unwrap()
}

fn persona(name: &str) -> Persona {
    personas()[name]
}

fn bot(name: &str, seed: u64) -> PersonaBot {
    PersonaBot::new(&persona(name), content().ai, seed)
}

/// Whether `command` is unit `unit` attacking `target`.
fn attacks(command: &Command, unit: u32, target: u32) -> bool {
    matches!(command, Command::Act {
        unit: u,
        action: UnitAction::Attack { target: t, .. },
        ..
    } if *u == UnitId(unit) && *t == UnitId(target))
}

/// Plays `state` with `bot` on the Player phase and the enemy AI on the
/// others, until the battle ends or `turns` have passed.
fn play_out(mut state: BattleState, bot: &mut PersonaBot, turns: Turn) -> BattleState {
    let mut enemy = BaselineBot::new(content().ai);
    let mut other = enemy;
    let mut bots: [(Phase, &mut dyn Bot<BattleState>); 3] = [
        (Phase::Player, bot),
        (Phase::Enemy, &mut enemy),
        (Phase::Other, &mut other),
    ];
    play(&mut state, &mut bots, turns).unwrap_or_else(|e| panic!("{e}"));
    state
}

#[test]
fn personas_file_lists_the_three_types() {
    let personas = personas();
    let names: Vec<&str> = personas.keys().map(String::as_str).collect();
    assert_eq!(names, ["casual", "hardcore", "normal"]);
    assert_eq!(personas["casual"].mode, GameMode::Casual);
    assert_eq!(personas["normal"].mode, GameMode::Classic);
    assert_eq!(personas["hardcore"].mode, GameMode::Classic);
    // Hardcore plays better than Normal, and Normal than Casual: a wider,
    // deeper plan on more copies, and fewer strays.
    let budget = |name: &str| {
        let s = personas[name].settings;
        (s.width * s.candidates * s.samples, s.depth, s.temperature)
    };
    let (casual, normal, hardcore) = (budget("casual"), budget("normal"), budget("hardcore"));
    assert!(hardcore.0 > 2 * normal.0 && normal.0 > 2 * casual.0);
    assert!(hardcore.1 > normal.1 && normal.1 > casual.1);
    assert!(hardcore.2 == 0 && normal.2 < casual.2);
    assert!(personas_from_ron("{\"x\": (mode: Classic)}").is_err());
}

// ---- Scoring ------------------------------------------------------------------

#[test]
fn score_weighs_every_count() {
    let tally = Tally {
        won: 1,
        lost: 2,
        lord_fell: 3,
        unit_fell: 4,
        hp_lost: 5,
        lord_hp_lost: 6,
        enemy_fell: 7,
        progress: 8,
        approach: 11,
        exposed: 12,
        lord_exposed: 13,
        at_risk: 14,
        lord_at_risk: 15,
        counter: 16,
        turns: 9,
        items: 10,
        houses: 17,
    };
    let only = |set: fn(&mut Weights)| {
        let mut weights = Weights::default();
        set(&mut weights);
        tally.score(&weights)
    };
    assert_eq!(only(|w| w.won = 1000), 1000);
    assert_eq!(only(|w| w.lost = 1000), 2000);
    assert_eq!(only(|w| w.lord_fell = 1000), 3000);
    assert_eq!(only(|w| w.unit_fell = 1000), 4000);
    assert_eq!(only(|w| w.hp_lost = 1000), 5000);
    assert_eq!(only(|w| w.lord_hp_lost = 1000), 6000);
    assert_eq!(only(|w| w.enemy_fell = 1000), 7000);
    assert_eq!(only(|w| w.progress = 1000), 8000);
    assert_eq!(only(|w| w.approach = 1000), 11_000);
    assert_eq!(only(|w| w.exposed = 1000), 12_000);
    assert_eq!(only(|w| w.lord_exposed = 1000), 13_000);
    assert_eq!(only(|w| w.at_risk = 1000), 14_000);
    assert_eq!(only(|w| w.lord_at_risk = 1000), 15_000);
    assert_eq!(only(|w| w.counter = 1000), 16_000);
    assert_eq!(only(|w| w.turns = 1000), 9000);
    assert_eq!(only(|w| w.items = -1000), -10_000);
    assert_eq!(only(|w| w.houses = 1000), 17_000);
    let all = Weights {
        won: 1,
        lost: 1,
        lord_fell: 1,
        unit_fell: 1,
        hp_lost: 1,
        lord_hp_lost: 1,
        enemy_fell: 1,
        progress: 1,
        approach: 1,
        exposed: 1,
        lord_exposed: 1,
        at_risk: 1,
        lord_at_risk: 1,
        counter: 1,
        turns: 1,
        items: 1,
        houses: 1,
    };
    assert_eq!(tally.score(&all), 153);
    // Clamped, never wrapped.
    let huge = Tally {
        progress: i64::from(i32::MAX),
        ..Tally::default()
    };
    assert_eq!(
        huge.score(&Weights {
            progress: 2,
            ..Weights::default()
        }),
        i32::MAX
    );
    assert_eq!(
        huge.score(&Weights {
            progress: -2,
            ..Weights::default()
        }),
        i32::MIN
    );
}

#[test]
fn scoring_per_persona() {
    let score = |name: &str, tally: Tally| tally.score(&persona(name).weights);
    let won = Tally {
        won: 1,
        ..Tally::default()
    };
    let won_one_fell = Tally {
        unit_fell: 1,
        ..won
    };
    let nothing = Tally::default();
    // Casual: a fall costs a little (units are tools), never a win.
    assert!(score("casual", won_one_fell) < score("casual", won));
    assert!(score("casual", won_one_fell) > score("casual", nothing));
    // Normal and Hardcore: a fallen unit costs more than a win is worth.
    for name in ["normal", "hardcore"] {
        assert!(score(name, won_one_fell) < score(name, nothing), "{name}");
        assert!(score(name, won) > score(name, nothing), "{name}");
    }
    // The lord falling is the worst for everyone.
    let lord_fell = Tally {
        lord_fell: 1,
        lost: 1,
        ..Tally::default()
    };
    for name in ["casual", "normal", "hardcore"] {
        assert!(score(name, lord_fell) < score(name, won_one_fell), "{name}");
    }
    // Items: free for Casual, a little for Normal, more for Hardcore.
    let items = Tally {
        items: 1,
        ..Tally::default()
    };
    assert_eq!(score("casual", items), 0);
    assert!(score("normal", items) < 0);
    assert!(score("hardcore", items) < score("normal", items));
    // Turns cost only Hardcore.
    let turn = Tally {
        turns: 1,
        ..Tally::default()
    };
    assert_eq!(score("casual", turn), 0);
    assert_eq!(score("normal", turn), 0);
    assert!(score("hardcore", turn) < 0);
    // A heal of 10 HP is worth Normal's item, one of 5 isn't.
    let healed = |hp: i64| Tally {
        hp_lost: -hp,
        items: 1,
        ..Tally::default()
    };
    assert!(score("normal", healed(10)) >= 0);
    assert!(score("normal", healed(5)) < 0);
}

#[test]
fn tally_counts_what_changed() {
    let mut units = vec![
        lord(player(1, 0)),
        weak(player(2, 5)),
        rigged(player(3, 1)),
        rigged(enemy(4, 6)),
        weak(enemy(5, 2)),
        enemy(6, 8),
    ];
    units[0].hp -= 2;
    let mut battle = Scripted::new(start(units));
    let begin = battle.state.clone();
    // Nothing changed: only the danger the units stand in counts.
    let danger = danger(&begin);
    assert_eq!(
        Tally::between(&begin, &begin),
        Tally {
            exposed: danger.exposed,
            lord_exposed: danger.lord_exposed,
            at_risk: danger.at_risk,
            lord_at_risk: danger.lord_at_risk,
            counter: danger.counter,
            ..Tally::default()
        }
    );
    // Player 3 fells enemy 5 and the lord drinks a potion; then enemy 4
    // fells player 2.
    battle.act(3, 3, attack(5));
    battle.act(
        1,
        0,
        UnitAction::UseItem {
            pack_index: 0,
            target: UnitId(1),
        },
    );
    battle.end_phase();
    battle.act(4, 6, attack(2));
    let end = battle.state.clone();
    let tally = Tally::between(&begin, &end);
    let enemy_hp = |s: &BattleState, id| s.unit(UnitId(id)).map_or(0, |u| i64::from(u.hp));
    assert_eq!(tally.won, 0);
    assert_eq!(tally.lost, 0);
    assert_eq!(tally.lord_fell, 0);
    assert_eq!(tally.unit_fell, 1);
    assert_eq!(tally.lord_hp_lost, -2);
    let hp3 = |s: &BattleState| i64::from(s.unit(UnitId(3)).unwrap().hp);
    assert_eq!(tally.hp_lost, hp3(&begin) - hp3(&end));
    assert_eq!(tally.enemy_fell, 1);
    assert_eq!(
        tally.progress,
        1 + enemy_hp(&begin, 4) - enemy_hp(&end, 4) + enemy_hp(&begin, 6) - enemy_hp(&end, 6)
    );
    // The lord at x 0 is now 6 from the nearest enemy (was 2), player 3 at
    // x 3 is 3 away (was 1); fallen player 2 doesn't count.
    assert_eq!(tally.approach, (2 - 6) + (1 - 3));
    assert_eq!(tally.turns, 0);
    assert_eq!(tally.items, 1);
    battle.next_player_phase();
    assert_eq!(Tally::between(&begin, &battle.state).turns, 1);
}

/// Houses saved count up and houses burned count down (0313).
#[test]
fn tally_counts_houses_saved_less_houses_burned() {
    let mut raider = enemy(2, 8);
    raider.raider = true;
    let village = trpg_core::VillageGoal {
        houses: vec![tile(2), tile(4), tile(6)],
        minimum: 1,
        rewards: vec![],
        burnt: content().terrain.display.id_of("burnt").unwrap(),
    };
    let (state, _) = BattleState::new(BattleSetup {
        village: Some(village),
        ..setup(vec![lord(player(1, 0)), raider])
    });
    let mut battle = Scripted::new(state);
    let begin = battle.state.clone();
    battle.act(1, 2, UnitAction::Evacuate);
    let saved = battle.state.clone();
    assert_eq!(Tally::between(&begin, &saved).houses, 1);
    battle.end_phase();
    battle.act(2, 7, UnitAction::Wait);
    let burned = battle.state.clone();
    assert_eq!(
        burned.village().map(|v| (v.saved(), v.lost())),
        Some((1, 1))
    );
    assert_eq!(Tally::between(&saved, &burned).houses, -1);
    assert_eq!(Tally::between(&begin, &burned).houses, 0);
}

#[test]
fn tally_counts_the_win_the_loss_and_the_lord() {
    let mut won = Scripted::new(start(vec![rigged(player(1, 5)), weak(enemy(2, 6))]));
    let begin = won.state.clone();
    won.act(1, 5, attack(2));
    let tally = Tally::between(&begin, &won.state);
    assert_eq!((tally.won, tally.lost, tally.enemy_fell), (1, 0, 1));

    let mut lost = Scripted::new(start(vec![lord(weak(player(1, 5))), rigged(enemy(2, 6))]));
    let begin = lost.state.clone();
    lost.end_phase();
    lost.act(2, 6, attack(1));
    let tally = Tally::between(&begin, &lost.state);
    assert_eq!(
        (tally.won, tally.lost, tally.lord_fell, tally.unit_fell),
        (0, 1, 1, 0)
    );
    assert_eq!(
        tally.lord_hp_lost, 0,
        "a fallen lord's HP isn't counted twice"
    );
}

#[test]
fn progress_follows_the_objective() {
    let units = || {
        vec![
            lord(player(1, 0)),
            rigged(player(2, 4)),
            weak(enemy(3, 6)),
            enemy(4, 8),
        ]
    };
    let with = |objective| {
        BattleState::new(BattleSetup {
            objective,
            ..setup(units())
        })
        .0
    };
    let fell_3 = |s: &BattleState| {
        let mut s = s.clone();
        s.apply(&Command::Act {
            unit: UnitId(2),
            dest: tile(5),
            action: attack(3),
        })
        .unwrap();
        s
    };
    let boss = with(Objective::DefeatUnit {
        unit: UnitId(3),
        turn_limit: None,
        leftovers: trpg_core::Leftovers::Stay,
    });
    assert_eq!(progress(&boss, &fell_3(&boss)), 1);
    // Unit 2 is a tile closer to the boss once it has fallen: no boss, no
    // goal.
    assert_eq!(approach(&boss, &fell_3(&boss)), 0);
    let other_boss = with(Objective::DefeatUnit {
        unit: UnitId(4),
        turn_limit: None,
        leftovers: trpg_core::Leftovers::Stay,
    });
    assert_eq!(progress(&other_boss, &fell_3(&other_boss)), 0);
    // Unit 2 walked from x 4 to 5, a tile closer to boss 4 at x 8.
    assert_eq!(approach(&other_boss, &fell_3(&other_boss)), 1);
    // A seize: unit 2 walked a tile closer; only the lord counts if only a
    // lord may seize.
    for (by_lord, closer) in [(false, 1), (true, 0)] {
        let seize = with(Objective::Seize {
            pos: tile(8),
            by_lord,
            turn_limit: None,
        });
        assert_eq!(
            progress(&seize, &fell_3(&seize)),
            closer,
            "by_lord {by_lord}"
        );
        assert_eq!(approach(&seize, &fell_3(&seize)), 1, "by_lord {by_lord}");
    }
    let survive = with(Objective::Survive { turns: 3 });
    assert_eq!(progress(&survive, &fell_3(&survive)), 0);
    assert_eq!(approach(&survive, &fell_3(&survive)), 0);
    // A rout: the lord goes from 6 (enemy 3) to 8 (enemy 4) from the
    // nearest enemy, unit 2 from 2 to 3.
    let rout = with(Objective::Rout { turn_limit: None });
    assert_eq!(approach(&rout, &fell_3(&rout)), (6 - 8) + (2 - 3));
    // An escape (0319): each unit that got out, from where it started.
    let area = trpg_core::TileRect {
        x: 0,
        y: 4,
        w: 2,
        h: 1,
    };
    let mut one_out = with(Objective::Escape { area });
    let out = |s: &mut BattleState, unit, x| {
        let dest = tile(x);
        let action = UnitAction::Escape;
        s.apply(&Command::Act {
            unit: UnitId(unit),
            dest,
            action,
        })
        .unwrap();
    };
    out(&mut one_out, 1, 0);
    let mut two_out = one_out.clone();
    out(&mut two_out, 2, 1);
    assert_eq!(progress(&one_out, &two_out), 1);
    // No one who may seize: no distance.
    let empty = start(vec![enemy(4, 8)]);
    assert_eq!(seize_distance(&empty, tile(0), false), 0);
}

// ---- Order --------------------------------------------------------------------

#[test]
fn order_drops_equip_and_talk_and_keeps_end_phase_last() {
    let s = start(vec![player(1, 4), weak(enemy(2, 6)), enemy(3, 2)]);
    let legal = legal_commands(&s);
    let mut given = legal.clone();
    given.push(Command::Talk {
        unit: UnitId(1),
        dest: tile(4),
        target: UnitId(2),
    });
    given.push(Command::Equip {
        unit: UnitId(1),
        equipped: trpg_core::Equipped::Weapon(0),
    });
    let ordered = order(&s, given, &content().ai, usize::MAX);
    assert!(
        !ordered
            .iter()
            .any(|c| matches!(c, Command::Equip { .. } | Command::Talk { .. }))
    );
    assert_eq!(ordered.last(), Some(&Command::EndPhase));
    assert_eq!(
        ordered.iter().filter(|c| **c == Command::EndPhase).count(),
        1
    );
    assert!(ordered.iter().all(|c| legal.contains(c)));
    let equip_or_talk = legal
        .iter()
        .filter(|c| matches!(c, Command::Equip { .. } | Command::Talk { .. }))
        .count();
    assert_eq!(
        ordered.len(),
        legal.len() - dropped_attacks(&legal) - equip_or_talk
    );
    // The cap would cut it: it takes the last place within the cap.
    for cap in [1, 2, 5] {
        let capped = order(&s, legal.clone(), &content().ai, cap);
        assert_eq!(capped.len(), cap);
        assert_eq!(capped[cap - 1], Command::EndPhase, "cap {cap}");
        assert_eq!(capped[..cap - 1], ordered[..cap - 1]);
    }
    assert_eq!(
        order(&s, legal.clone(), &content().ai, 0),
        [Command::EndPhase]
    );
    // No EndPhase given (a move after an attack): none added.
    let moves = vec![Command::MoveAfter {
        unit: UnitId(1),
        to: None,
    }];
    assert_eq!(order(&s, moves.clone(), &content().ai, 3), moves);
}

/// The attacks `order` drops: every one but the first of each unit and
/// action, which differ only in their tile.
fn dropped_attacks(legal: &[Command]) -> usize {
    let attacks: Vec<(UnitId, &UnitAction)> = legal
        .iter()
        .filter_map(|c| match c {
            Command::Act {
                unit,
                action: action @ (UnitAction::Attack { .. } | UnitAction::Cast { .. }),
                ..
            } => Some((*unit, action)),
            _ => None,
        })
        .collect();
    let mut distinct = attacks.clone();
    distinct.sort_by_key(|(u, a)| (u.0, format!("{a:?}")));
    distinct.dedup();
    attacks.len() - distinct.len()
}

#[test]
fn order_puts_attacks_first_by_expected_damage() {
    // Enemy 2 at 1 HP can't strike back; enemy 3 can.
    let s = start(vec![player(1, 4), weak(enemy(2, 6)), enemy(3, 2)]);
    let legal = legal_commands(&s);
    let ordered = order(&s, legal.clone(), &content().ai, usize::MAX);
    let is_attack = |c: &Command| attacks(c, 1, 2) || attacks(c, 1, 3);
    let action = |c: &Command| match c {
        Command::Act { action, .. } => action.clone(),
        _ => unreachable!(),
    };
    let rank = |c: &Command| match c {
        Command::Act { unit, dest, action } => {
            let forecast = s.preview_attack(*unit, *dest, action).unwrap().forecast;
            let taken = forecast.defender.as_ref().map_or(0, expected_damage);
            (expected_damage(&forecast.attacker), -taken)
        }
        _ => unreachable!(),
    };
    // First, one attack on each target: the best way of striking it, from
    // any tile, the better first.
    let firsts = &ordered[..2];
    assert!(
        attacks(&firsts[0], 1, 2) != attacks(&firsts[1], 1, 2),
        "{firsts:?}"
    );
    assert!(rank(&firsts[0]) >= rank(&firsts[1]));
    for first in firsts {
        let target = |c: &Command| attacks(c, 1, 2) == attacks(first, 1, 2);
        let best = legal
            .iter()
            .filter(|c| is_attack(c) && target(c))
            .map(rank)
            .max();
        assert_eq!(Some(rank(first)), best, "{first:?}");
    }
    assert!(!is_attack(&ordered[2]), "{:?}", ordered[2]);
    // Every other way of striking comes later, once each, from its best
    // tile: every one the legal list has.
    let all: Vec<&Command> = ordered.iter().filter(|c| is_attack(c)).collect();
    assert_eq!(
        all.len(),
        legal.iter().filter(|c| is_attack(c)).count() - dropped_attacks(&legal)
    );
    for (i, a) in all.iter().enumerate() {
        assert!(
            !all[..i].iter().any(|b| action(b) == action(a)),
            "{a:?} twice"
        );
        let best = legal
            .iter()
            .filter(|c| is_attack(c) && action(c) == action(a))
            .map(rank)
            .max()
            .unwrap();
        assert_eq!(rank(a), best, "{a:?}");
    }
    let later = &all[2..];
    for pair in later.windows(2) {
        assert!(rank(pair[0]) >= rank(pair[1]), "{pair:?}");
    }
    // The baseline's own pick comes right after the firsts, if it isn't
    // one of them.
    let pick = next_command(&s, &content().ai).unwrap();
    assert!(firsts.contains(&pick) || ordered[2] == pick);
}

#[test]
fn every_unit_that_can_strike_has_a_say_before_a_second_way_of_striking() {
    // Two units can each strike the same enemy in several ways (arts and
    // actives): each unit's best comes before anyone's second.
    let s = start(vec![player(1, 4), player(2, 3), enemy(3, 6)]);
    let ordered = order(&s, legal_commands(&s), &content().ai, usize::MAX);
    let firsts: Vec<UnitId> = ordered[..2]
        .iter()
        .map(|c| match c {
            Command::Act { unit, .. } => *unit,
            _ => unreachable!(),
        })
        .collect();
    assert!(
        firsts.contains(&UnitId(1)) && firsts.contains(&UnitId(2)),
        "{:?}",
        &ordered[..2]
    );
    assert!(attacks(&ordered[0], 1, 3) || attacks(&ordered[0], 2, 3));
    assert!(attacks(&ordered[1], 1, 3) || attacks(&ordered[1], 2, 3));
    assert_eq!(attack_key(&ordered[0]).1, Some(UnitId(3)));
    assert_eq!(attack_key(&Command::EndPhase), (UnitId(0), None));
}

#[test]
fn expected_damage_counts_every_strike_by_its_hit() {
    let mut side = SideForecast {
        damage: 7,
        followup_damage: 5,
        hit: 80,
        crit: 0,
        strikes: 1,
        effective: false,
        broken: false,
        affinity: None,
    };
    assert_eq!(expected_damage(&side), 560);
    side.strikes = 3;
    assert_eq!(expected_damage(&side), (7 + 5 + 5) * 80);
}

#[test]
fn order_lists_heals_on_the_wounded_most_wounded_first() {
    let mut hurt = player(2, 3);
    hurt.hp -= 6;
    let mut scratched = player(3, 5);
    scratched.hp -= 1;
    // The enemy is out of everyone's reach, past the wall.
    let mut far = stationary(enemy(9, 0));
    far.pos = Pos::new(13, 7);
    let s = start(vec![player(1, 4), hurt, scratched, far]);
    let ordered = order(&s, legal_commands(&s), &content().ai, usize::MAX);
    let heals: Vec<(UnitId, Pos, UnitId)> = ordered
        .iter()
        .filter_map(|c| match c {
            Command::Act {
                unit,
                dest,
                action:
                    UnitAction::UseItem {
                        target,
                        pack_index: 0,
                    },
            } if *target != UnitId(1) => Some((*unit, *dest, *target)),
            _ => None,
        })
        .collect();
    // The heals come right after the attacks and the baseline's pick, each
    // once.
    let first = ordered
        .iter()
        .position(|c| matches!(c, Command::Act { action: UnitAction::UseItem { target, .. }, .. } if *target != UnitId(1)))
        .unwrap();
    assert!(first <= 1, "{:?}", &ordered[..first]);
    let wounded = |c: &Command| {
        matches!(c, Command::Act { action: UnitAction::UseItem { target, .. }, .. }
            if *target == UnitId(2) || *target == UnitId(3))
    };
    let heal_count = ordered.iter().filter(|c| wounded(c)).count();
    assert!(ordered[first..first + heal_count].iter().all(wounded));
    // Unit 1 heals unit 2, then unit 3, from its own tile; units 2 and 3
    // heal themselves (and each other, if they can reach).
    let of_1: Vec<UnitId> = heals
        .iter()
        .filter(|h| h.0 == UnitId(1))
        .map(|h| h.2)
        .collect();
    assert_eq!(of_1, [UnitId(2), UnitId(3)]);
    assert!(
        heals
            .iter()
            .filter(|h| h.0 == UnitId(1))
            .all(|h| h.1 == tile(4))
    );
    let first_heal = heals.first().unwrap();
    assert_eq!(first_heal.2, UnitId(2));
    // An item on a unit at full HP isn't a heal: it comes with the rest.
    let full = ordered.iter().position(|c| {
        matches!(c, Command::Act { action: UnitAction::UseItem { target, .. }, .. } if *target == UnitId(1))
    });
    assert!(full.unwrap() > first + heal_count);
}

#[test]
fn order_offers_each_ready_unit_its_advance() {
    // The enemy far off on the right; units 1 and 2 can walk towards it.
    let mut far = stationary(enemy(9, 0));
    far.pos = Pos::new(13, 4);
    let s = start(vec![player(1, 0), player(2, 1), far]);
    let mut goals = Goals::of(&s).unwrap();
    let legal = legal_commands(&s);
    // Each unit's nearest wait, the first in the legal order on a tie.
    let expected: Vec<Command> = [1, 2]
        .into_iter()
        .map(|id| {
            let mover = s.unit(UnitId(id)).unwrap().clone();
            let waits = legal.iter().filter(|c| {
                matches!(c, Command::Act { unit, action: UnitAction::Wait, .. } if *unit == UnitId(id))
            });
            let here = goals.distance(&mover, mover.pos).unwrap();
            let mut distance = |c: &Command| match c {
                Command::Act { dest, .. } => goals.distance(&mover, *dest).unwrap(),
                _ => unreachable!(),
            };
            let best = waits.min_by_key(|c| distance(c)).unwrap().clone();
            assert!(distance(&best) < here);
            best
        })
        .collect();
    let ordered = order(&s, legal.clone(), &content().ai, usize::MAX);
    let pick = next_command(&s, &content().ai).unwrap();
    let first_other_wait = ordered
        .iter()
        .position(|c| {
            matches!(
                c,
                Command::Act {
                    action: UnitAction::Wait,
                    ..
                }
            ) && *c != pick
                && !expected.contains(c)
        })
        .unwrap();
    for advance in &expected {
        let at = ordered.iter().position(|c| c == advance).unwrap();
        assert!(at < first_other_wait, "{advance:?} at {at}");
    }
    // A unit already as near as it can get has none; a nearer wait
    // replaces a farther one, never the other way round.
    let near = start(vec![player(1, 7), stationary(enemy(9, 8))]);
    let mut goals = Goals::of(&near).unwrap();
    let wait_at = |x| Command::Act {
        unit: UnitId(1),
        dest: tile(x),
        action: UnitAction::Wait,
    };
    let mut advances = Vec::new();
    advance(
        &near,
        &mut goals,
        UnitId(1),
        tile(6),
        &wait_at(6),
        &mut advances,
    );
    assert_eq!(
        advances,
        [] as [(trpg_core::UnitId, i64, trpg_core::Command); 0]
    );
    let mut goals = Goals::of(&s).unwrap();
    advance(
        &s,
        &mut goals,
        UnitId(1),
        tile(2),
        &wait_at(2),
        &mut advances,
    );
    advance(
        &s,
        &mut goals,
        UnitId(1),
        tile(4),
        &wait_at(4),
        &mut advances,
    );
    advance(
        &s,
        &mut goals,
        UnitId(1),
        tile(3),
        &wait_at(3),
        &mut advances,
    );
    let kept: Vec<&Command> = advances.iter().map(|a| &a.2).collect();
    assert_eq!(kept, [&wait_at(4)]);
    // A unit that isn't on the map has none.
    advance(
        &s,
        &mut goals,
        UnitId(7),
        tile(5),
        &wait_at(5),
        &mut advances,
    );
    assert_eq!(advances.len(), 1);
}

#[test]
fn with_no_attack_the_enemy_ais_own_pick_comes_first() {
    let mut far = stationary(enemy(9, 0));
    far.pos = Pos::new(13, 4);
    let s = start(vec![player(1, 0), far]);
    let pick = next_command(&s, &content().ai).unwrap();
    assert!(matches!(pick, Command::Act { .. }), "{pick:?}");
    let ordered = order(&s, legal_commands(&s), &content().ai, usize::MAX);
    assert_eq!(ordered[0], pick);
    // Nothing is listed twice.
    for (i, c) in ordered.iter().enumerate() {
        assert!(!ordered[..i].contains(c), "{c:?} twice");
    }
}

#[test]
fn the_ais_pick_comes_first_even_when_it_is_no_advance() {
    // Unit 1 thinks like a healer: the enemy AI would walk it towards its
    // ally on the left, not towards the enemy far on the right.
    let mut healer_minded = player(1, 3);
    healer_minded.ai = trpg_core::AiBehavior::Healer;
    let mut far = stationary(enemy(9, 0));
    far.pos = Pos::new(13, 4);
    let s = start(vec![healer_minded, player(2, 0), far]);
    let pick = next_command(&s, &content().ai).unwrap();
    let ordered = order(&s, legal_commands(&s), &content().ai, usize::MAX);
    assert_eq!(ordered[0], pick);
    let mut goals = Goals::of(&s).unwrap();
    let Command::Act { unit, dest, .. } = &pick else {
        unreachable!()
    };
    let mover = s.unit(*unit).unwrap();
    assert!(
        goals.distance(mover, *dest) >= goals.distance(mover, mover.pos),
        "{pick:?}"
    );
    // Nothing is listed twice, though each unit's hold is its advance here
    // (the enemy strikes only what stands next to it).
    for (i, c) in ordered.iter().enumerate() {
        assert!(!ordered[..i].contains(c), "{c:?} twice");
    }
}

#[test]
fn an_attack_comes_from_the_tile_that_takes_the_least_back() {
    // Enemy 9 at (1, 4) can be struck from the plain at (0, 4), the road at
    // (2, 4) or the forest at (1, 5); the forest makes its counter miss more.
    let mut target = enemy(9, 0);
    target.pos = Pos::new(1, 4);
    let s = start(vec![player(1, 3), target]);
    let ordered = order(&s, legal_commands(&s), &content().ai, usize::MAX);
    let taken = |dest: Pos| {
        let preview = s.preview_attack(UnitId(1), dest, &attack(9)).unwrap();
        preview
            .forecast
            .defender
            .as_ref()
            .map_or(0, expected_damage)
    };
    assert!(taken(Pos::new(1, 5)) < taken(Pos::new(2, 4)));
    assert!(
        matches!(ordered[0], Command::Act { dest, .. } if dest == Pos::new(1, 5)),
        "{:?}",
        ordered[0]
    );
}

#[test]
fn order_offers_each_ready_unit_its_hold_out_of_reach() {
    // Enemy 9 at x 8 can reach most of the row; units 1 and 2 stand safe
    // at x 0 and 1.
    let s = start(vec![player(1, 0), player(2, 1), enemy(9, 8)]);
    let legal = legal_commands(&s);
    let threats = threats(&s);
    let safe = |at: Pos| !threats.iter().any(|(_, strikes)| strikes(at));
    let mut goals = Goals::of(&s).unwrap();
    let ordered = order(&s, legal.clone(), &content().ai, usize::MAX);
    let pick = next_command(&s, &content().ai).unwrap();
    // Each unit's nearest wait and nearest safe wait (its advance and its
    // hold), the first in the legal order on a tie.
    let mut listed = vec![pick.clone()];
    let mut holds = Vec::new();
    for id in [1, 2] {
        let mover = s.unit(UnitId(id)).unwrap().clone();
        let mut nearest = |only_safe: bool| {
            let mut best: Option<(i64, &Command)> = None;
            for c in &legal {
                if let Command::Act {
                    unit,
                    dest,
                    action: UnitAction::Wait,
                } = c
                    && *unit == UnitId(id)
                    && (safe(*dest) || !only_safe)
                {
                    let d = goals.distance(&mover, *dest).unwrap();
                    if best.is_none_or(|(b, _)| d < b) {
                        best = Some((d, c));
                    }
                }
            }
            best.unwrap().1.clone()
        };
        let (advance, hold) = (nearest(false), nearest(true));
        assert_ne!(advance, hold, "the danger zone shapes the hold");
        let Command::Act { dest, .. } = &hold else {
            unreachable!()
        };
        assert!(safe(*dest));
        listed.extend([advance, hold.clone()]);
        holds.push(hold);
    }
    // The holds come with the advances, before any other wait of any unit.
    let first_plain_wait = ordered
        .iter()
        .position(|c| {
            matches!(
                c,
                Command::Act {
                    action: UnitAction::Wait,
                    ..
                }
            ) && !listed.contains(c)
        })
        .unwrap();
    for hold in &holds {
        let at = ordered.iter().position(|c| c == hold).unwrap();
        assert!(
            at < first_plain_wait,
            "{hold:?} at {at}, a plain wait at {first_plain_wait}"
        );
    }
    // Every waiting unit is listed once, and the danger zone is real here.
    assert!(
        legal.iter().any(
            |c| matches!(c, Command::Act { dest, action: UnitAction::Wait, .. } if !safe(*dest))
        )
    );
    for (i, c) in ordered.iter().enumerate() {
        assert!(!ordered[..i].contains(c), "{c:?} twice");
    }
}

#[test]
fn hold_and_advance_keep_each_units_nearest_the_first_on_a_tie() {
    let mut far = stationary(enemy(9, 0));
    far.pos = Pos::new(13, 4);
    let s = start(vec![player(1, 0), player(2, 1), far]);
    let mut goals = Goals::of(&s).unwrap();
    let wait = |unit, x| Command::Act {
        unit: UnitId(unit),
        dest: tile(x),
        action: UnitAction::Wait,
    };
    let other = |unit, x| Command::Act {
        unit: UnitId(unit),
        dest: tile(x),
        action: UnitAction::Seize,
    };
    let mut holds = Vec::new();
    hold(&s, &mut goals, UnitId(1), tile(2), &wait(1, 2), &mut holds);
    hold(&s, &mut goals, UnitId(2), tile(3), &wait(2, 3), &mut holds);
    hold(&s, &mut goals, UnitId(1), tile(4), &wait(1, 4), &mut holds);
    hold(&s, &mut goals, UnitId(1), tile(3), &wait(1, 3), &mut holds);
    hold(&s, &mut goals, UnitId(1), tile(4), &other(1, 4), &mut holds);
    hold(&s, &mut goals, UnitId(7), tile(5), &wait(7, 5), &mut holds);
    let kept: Vec<&Command> = holds.iter().map(|h| &h.2).collect();
    assert_eq!(kept, [&wait(1, 4), &wait(2, 3)]);
    let mut advances = Vec::new();
    // Staying put is no advance.
    advance(
        &s,
        &mut goals,
        UnitId(1),
        tile(0),
        &wait(1, 0),
        &mut advances,
    );
    assert_eq!(
        advances,
        [] as [(trpg_core::UnitId, i64, trpg_core::Command); 0]
    );
    advance(
        &s,
        &mut goals,
        UnitId(1),
        tile(4),
        &wait(1, 4),
        &mut advances,
    );
    advance(
        &s,
        &mut goals,
        UnitId(1),
        tile(4),
        &other(1, 4),
        &mut advances,
    );
    let kept: Vec<&Command> = advances.iter().map(|a| &a.2).collect();
    assert_eq!(kept, [&wait(1, 4)]);
    // A wait is no attack: its key has no target.
    assert_eq!(attack_key(&wait(2, 3)), (UnitId(2), None));
}

#[test]
fn a_heal_is_a_heal_spell_on_a_wounded_ally() {
    let mut healer = trpg_core::Unit::generic(
        UnitId(1),
        &trpg_core::ClassId("cleric".into()),
        &content().classes,
        1,
        trpg_core::Faction::Player,
        tile(4),
    )
    .unwrap();
    let heal = trpg_core::SpellId::new("heal");
    healer.learned.insert(heal.clone());
    healer.spells.uses_left.insert(heal.clone(), 3);
    let mut hurt = player(2, 5);
    hurt.hp -= 6;
    let mut foe = enemy(3, 3);
    foe.hp -= 6;
    let s = start(vec![healer, hurt, foe]);
    let cast = |target| UnitAction::Cast {
        spell: heal.clone(),
        target: CastTarget::Unit(UnitId(target)),
        active: None,
    };
    assert_eq!(heal_need(&s, UnitId(1), tile(4), &cast(2)), Some(6));
    // At an enemy, the spell heals nothing: no heal.
    assert_eq!(heal_need(&s, UnitId(1), tile(4), &cast(3)), None);
    assert_eq!(heal_need(&s, UnitId(1), tile(4), &UnitAction::Wait), None);
}

#[test]
fn goals_are_measured_walking() {
    // Enemy 8 is two tiles from unit 1 as the crow flies, but behind the
    // walls of a room with no way in; enemy 9 is three tiles off in the
    // open.
    let mut walled = enemy(8, 0);
    walled.pos = Pos::new(10, 6);
    let mut walker = player(1, 0);
    walker.pos = Pos::new(8, 6);
    let mut open = enemy(9, 0);
    open.pos = Pos::new(5, 6);
    let s = start(vec![walker.clone(), walled, open]);
    let mut goals = Goals::of(&s).unwrap();
    // Two steps onto plain; the step onto the enemy's tile is free.
    assert_eq!(goals.distance(&walker, walker.pos), Some(2));
    // A survive has no goals.
    let survive = BattleState::new(BattleSetup {
        objective: Objective::Survive { turns: 2 },
        ..setup(vec![walker])
    })
    .0;
    assert!(Goals::of(&survive).is_none());
    assert_eq!(approach(&survive, &survive), 0);
}

#[test]
fn danger_counts_the_threats_a_player_can_see() {
    // Player 2 (1 HP) stands where enemy 3 can reach it next phase; the
    // lord, at x 0, is beyond enemy 3's reach, and enemy 4 next to it
    // never moves and strikes only what stands next to it.
    let threatened = |enemy_4: Unit| {
        start(vec![
            lord(player(1, 0)),
            weak(player(2, 4)),
            enemy(3, 8),
            enemy_4,
        ])
    };
    let s = threatened(stationary(enemy(4, 2)));
    let forecast = s.threat_forecast(UnitId(3), UnitId(2)).unwrap();
    let expected = (expected_damage(&forecast.attacker) / 100).min(1);
    let back = forecast.defender.as_ref().map_or(0, expected_damage) / 100;
    assert_eq!(
        danger(&s),
        Danger {
            exposed: expected,
            lord_exposed: 0,
            at_risk: 1,
            lord_at_risk: 0,
            counter: back.min(20),
        }
    );
    // Enemy 4 free to move reaches the lord too.
    let s = threatened(enemy(4, 2));
    let on_lord = s.threat_forecast(UnitId(4), UnitId(1)).unwrap().attacker;
    let both = danger(&s);
    assert_eq!(both.lord_exposed, expected_damage(&on_lord) / 100);
    assert!(both.counter > danger(&threatened(stationary(enemy(4, 2)))).counter);
    assert_eq!(both.lord_at_risk, i64::from(all_hit(&on_lord) >= 20));
    // A lord left alone: its danger doesn't count.
    let alone = start(vec![lord(player(1, 0)), enemy(4, 2)]);
    assert_eq!(danger(&alone).lord_exposed, 0);
    assert_eq!(danger(&alone).lord_at_risk, 0);
    assert!(danger(&alone).counter > 0);
    // Nobody in reach: no danger.
    let mut far = stationary(enemy(3, 0));
    far.pos = Pos::new(13, 7);
    assert_eq!(danger(&start(vec![player(1, 4), far])), Danger::default());
}

#[test]
fn tally_counts_the_danger_where_the_copy_ends() {
    let s = start(vec![weak(player(2, 4)), enemy(3, 8)]);
    let tally = Tally::between(&start(vec![player(2, 0), enemy(3, 8)]), &s);
    assert_eq!(tally.at_risk, danger(&s).at_risk);
    assert_eq!(tally.exposed, danger(&s).exposed);
    assert_eq!(tally.at_risk, 1);
    assert_eq!(tally.counter, danger(&s).counter);
}

#[test]
fn counter_is_at_most_the_enemys_hp() {
    // A rigged player unit would fell a 1-HP enemy many times over.
    let s = start(vec![rigged(player(2, 4)), weak(enemy(3, 6))]);
    assert_eq!(danger(&s).counter, 1);
}

// ---- The bot ------------------------------------------------------------------

#[test]
fn hardcore_never_attacks_into_a_forecast_it_scores_worse() {
    // Unit 1 kills enemy 2 (1 HP) for sure; enemy 3 shrugs it off and
    // strikes back to kill.
    let mut wall = rigged(enemy(3, 1));
    wall.stats.def = 99;
    wall.stats.hp = 99;
    wall.hp = 99;
    let s = start(vec![
        rigged(player(1, 4)),
        stationary(weak(enemy(2, 7))),
        stationary(wall),
    ]);
    for seed in 1..=3 {
        let command = bot("hardcore", seed).choose(&s);
        assert!(attacks(&command, 1, 2), "seed {seed}: {command:?}");
    }
}

/// The battle with its real luck reseeded: unit 1's attack on enemy 2
/// (some 50% to hit) hits or misses as `hits` says.
fn coin_flip(hits: bool) -> BattleState {
    let mut attacker = player(1, 5);
    attacker.stats.dex = 0;
    let mut s = start(vec![attacker, stationary(enemy(2, 6))]);
    let swing = Command::Act {
        unit: UnitId(1),
        dest: tile(5),
        action: attack(2),
    };
    let hit = s
        .preview_attack(UnitId(1), tile(5), &attack(2))
        .unwrap()
        .forecast
        .attacker
        .hit;
    assert!((20..=80).contains(&hit), "hit {hit}");
    let first_hits = |s: &BattleState| {
        let events = s.clone().apply(&swing).unwrap();
        events.iter().any(|e| {
            matches!(e, Event::CombatResolved { outcome, .. }
                if outcome.strikes.first().is_some_and(|x| x.hit))
        })
    };
    let seed = (0..200)
        .find(|&seed| {
            s.reseed_luck(seed);
            first_hits(&s) == hits
        })
        .unwrap();
    s.reseed_luck(seed);
    s
}

#[test]
fn planning_never_peeks_at_real_luck() {
    let (hit, miss) = (coin_flip(true), coin_flip(false));
    assert_ne!(hit.rng(), miss.rng());
    for name in ["casual", "normal", "hardcore"] {
        for seed in 1..=2 {
            let on_hit = bot(name, seed).choose(&hit);
            let on_miss = bot(name, seed).choose(&miss);
            assert_eq!(on_hit, on_miss, "{name}, seed {seed}");
        }
    }
}

#[test]
fn same_seed_same_choice() {
    let s = start(vec![player(1, 4), player(2, 3), enemy(3, 7), enemy(4, 8)]);
    let picks: Vec<Command> = (1..=8).map(|seed| bot("casual", seed).choose(&s)).collect();
    for (seed, pick) in (1..=8).zip(&picks) {
        assert_eq!(&bot("casual", seed).choose(&s), pick);
    }
}

/// Unit 1 (not a lord) and the lord, unit 2, can each reach enemy 3, who
/// kills anything that strikes it and takes exactly two blows to fell:
/// winning this turn means unit 1 strikes first and falls, then the lord
/// finishes it. Striking with the lord first loses it; nobody striking
/// keeps everyone, for now.
fn sacrifice(mode: GameMode) -> BattleState {
    let units = |enemy_hp| {
        let mut wall = stationary(rigged(enemy(3, 6)));
        // Never crit: a crit could fell it in one blow.
        wall.stats.dex = 400;
        wall.stats.def = 0;
        wall.stats.hp = 99;
        wall.hp = enemy_hp;
        let mut fodder = player(1, 3);
        let mut leader = lord(player(2, 2));
        for unit in [&mut fodder, &mut leader] {
            unit.stats.dex = 99;
            unit.stats.spd = wall.stats.spd;
        }
        vec![fodder, leader, wall]
    };
    // Plain blows only: no art or active that spares a counter.
    let plain = |units| BattleSetup {
        mode,
        arts: std::sync::Arc::default(),
        skills: std::sync::Arc::default(),
        ..setup(units)
    };
    let probe = BattleState::new(plain(units(99))).0;
    // Its hardest blow, of every attack it has (arts and actives too).
    let damage = |unit| {
        let blows = legal_commands(&probe).into_iter().filter_map(|c| match c {
            Command::Act {
                unit: u,
                dest,
                action,
            } if u == UnitId(unit) => {
                let forecast = probe.preview_attack(u, dest, &action).ok()?.forecast;
                Some(forecast.attacker)
            }
            _ => None,
        });
        blows
            .map(|blow| {
                assert_eq!((blow.hit, blow.crit, blow.strikes), (100, 0, 1), "{blow:?}");
                blow.damage
            })
            .max()
            .unwrap()
    };
    BattleState::new(plain(units(damage(1) + damage(2)))).0
}

#[test]
fn casual_accepts_retreats_normal_does_not() {
    // What each type prefers when it sees the whole line: Casual's own
    // weights on a plan wide enough to keep every first step (its own is
    // too narrow to keep one that loses a unit: Nick's "units are tools").
    let mut casual = persona("casual");
    casual.settings.width = casual.settings.candidates;
    casual.settings.temperature = 0;
    let mut casual = PersonaBot::new(&casual, content().ai, 1);
    let casual = play_out(sacrifice(GameMode::Casual), &mut casual, 1);
    assert_eq!(casual.outcome(), Some(Outcome::Victory));
    let fell: Vec<UnitId> = casual.fallen().iter().map(|u| u.id).collect();
    assert_eq!(fell, [UnitId(1), UnitId(3)]);

    let start = sacrifice(GameMode::Classic);
    for seed in 1..=3 {
        let first = bot("normal", seed).choose(&start);
        assert!(
            !attacks(&first, 1, 3) && !attacks(&first, 2, 3),
            "{first:?}"
        );
    }
    let normal = play_out(start, &mut bot("normal", 1), 2);
    assert!(normal.fallen().is_empty(), "{:?}", normal.fallen());
    assert_eq!(normal.outcome(), None);
}

#[test]
fn a_persona_talks_first_once_per_pair() {
    let mut speaker = player(1, 4);
    speaker.character = Some(CharacterId("speaker".to_owned()));
    let mut listener = player(2, 6);
    listener.character = Some(CharacterId("listener".to_owned()));
    let mut talking = BattleSetup {
        triggers: vec![Trigger {
            when: TriggerWhen::Talk {
                a: CharacterId("speaker".to_owned()),
                b: CharacterId("listener".to_owned()),
            },
            scene: "chat".to_owned(),
            once: false,
        }],
        ..setup(vec![speaker, listener, stationary(enemy(3, 0))])
    };
    talking.units[2].pos = Pos::new(0, 0);
    let mut s = BattleState::new(talking).0;
    let mut bot = bot("casual", 1);
    let first = bot.choose(&s);
    assert!(
        matches!(
            first,
            Command::Talk {
                unit: UnitId(1),
                target: UnitId(2),
                ..
            }
        ),
        "{first:?}"
    );
    s.apply(&first).unwrap();
    // The trigger fires every time, so the pair could talk again, either
    // way round: the persona doesn't.
    assert!(
        legal_commands(&s)
            .iter()
            .any(|c| matches!(c, Command::Talk { .. }))
    );
    let second = bot.choose(&s);
    assert!(!matches!(second, Command::Talk { .. }), "{second:?}");
}
