use shuyi_core::play;
use trpg_core::{BattleSetup, CharacterId, Event, Trigger, TriggerWhen, Turn};

use super::*;
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
    // Hardcore plays better than Normal: a bigger budget, no strays.
    let (normal, hardcore) = (personas["normal"].settings, personas["hardcore"].settings);
    assert!(hardcore.candidates * hardcore.samples > 2 * normal.candidates * normal.samples);
    assert_eq!(hardcore.temperature, 0);
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
        turns: 9,
        items: 10,
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
    assert_eq!(only(|w| w.turns = 1000), 9000);
    assert_eq!(only(|w| w.items = -1000), -10_000);
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
        turns: 1,
        items: 1,
    };
    assert_eq!(tally.score(&all), 66);
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
    // Casual: a win is a win, whoever fell.
    assert_eq!(score("casual", won_one_fell), score("casual", won));
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
    assert_eq!(Tally::between(&begin, &begin), Tally::default());
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
    });
    assert_eq!(progress(&boss, &fell_3(&boss)), 1);
    // Unit 2 is a tile closer to the boss once it has fallen: no boss, no
    // goal.
    assert_eq!(approach(&boss, &fell_3(&boss)), 0);
    let other_boss = with(Objective::DefeatUnit {
        unit: UnitId(4),
        turn_limit: None,
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
    let first_non_attack = ordered.iter().position(|c| !is_attack(c)).unwrap();
    let attack_list = &ordered[..first_non_attack];
    assert!(!ordered[first_non_attack..].iter().any(is_attack));
    // One of each attack (target, weapon, art or active), from its best
    // tile: every one the legal list has, from one tile or another.
    let action = |c: &Command| match c {
        Command::Act { action, .. } => action.clone(),
        _ => unreachable!(),
    };
    assert_eq!(
        attack_list.len(),
        legal.iter().filter(|c| is_attack(c)).count() - dropped_attacks(&legal)
    );
    for (i, a) in attack_list.iter().enumerate() {
        assert!(
            !attack_list[..i].iter().any(|b| action(b) == action(a)),
            "{a:?} twice"
        );
    }
    let rank = |c: &Command| match c {
        Command::Act { unit, dest, action } => {
            let forecast = s.preview_attack(*unit, *dest, action).unwrap().forecast;
            let taken = forecast.defender.as_ref().map_or(0, expected_damage);
            (expected_damage(&forecast.attacker), -taken)
        }
        _ => unreachable!(),
    };
    for pair in attack_list.windows(2) {
        assert!(rank(&pair[0]) >= rank(&pair[1]), "{pair:?}");
    }
    // Each from its best tile.
    for a in attack_list {
        let best = legal
            .iter()
            .filter(|c| is_attack(c) && action(c) == action(a))
            .map(rank)
            .max()
            .unwrap();
        assert_eq!(rank(a), best, "{a:?}");
    }
    // The baseline's own pick comes next if it isn't an attack already.
    let pick = next_command(&s, &content().ai).unwrap();
    assert!(attack_list.contains(&pick) || ordered[first_non_attack] == pick);
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
    assert!(advances.is_empty());
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
fn a_personas_copies_end_its_own_phase_and_the_ai_replies() {
    let s = start(vec![player(1, 4), rigged(enemy(2, 6))]);
    let mut playout = PersonaPlayout(BaselineBot::new(content().ai));
    assert_eq!(playout.choose(&s), Command::EndPhase);
    let mut enemy_phase = s.clone();
    enemy_phase.apply(&Command::EndPhase).unwrap();
    let reply = playout.choose(&enemy_phase);
    assert!(attacks(&reply, 2, 1), "{reply:?}");
    assert_eq!(reply, BaselineBot::new(content().ai).choose(&enemy_phase));
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
        arts: Default::default(),
        skills: Default::default(),
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
    let casual = play_out(sacrifice(GameMode::Casual), &mut bot("casual", 1), 1);
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
