//! Act 1's battle rules (ticket 0319, `turn-structure.md`, *Act 1's battle
//! rules*): the escape objective, a unit that retreats instead of falling
//! and one that can't be killed (it goes down). Test units deal their
//! weapon's might with every strike and each side strikes once (see the
//! parent module).

use super::bots::random_play;
use super::*;
use crate::progression::exp_for_combat;
use crate::unit::Defeat;

/// The exit area of the escape tests: column 3 of the open map.
const EXIT: TileRect = TileRect {
    x: 3,
    y: 0,
    w: 1,
    h: 5,
};

fn escape() -> Objective {
    Objective::Escape { area: EXIT }
}

/// `u` with this [`Defeat`].
fn defeat(u: Unit, defeat: Defeat) -> Unit {
    Unit { defeat, ..u }
}

/// A battle of `units` with `objective`, with levels on (so EXP is given).
fn leveled(units: Vec<Unit>, objective: Objective) -> BattleState {
    start(BattleSetup {
        objective,
        classes: Arc::new(leveling(classes())),
        ..setup(units)
    })
}

/// [`cast`] with unit 2 at (0,3), three plain steps from the exit.
fn runners() -> Vec<Unit> {
    let mut units = cast();
    units[1].pos = p(0, 3);
    units
}

/// The EXP each player unit gained in `events`, in order.
fn exp(events: &[Event]) -> Vec<(UnitId, u32)> {
    events
        .iter()
        .filter_map(|e| match *e {
            Event::ExpGained { unit, amount } => Some((unit, amount)),
            _ => None,
        })
        .collect()
}

// ---- Escape ----------------------------------------------------------------

#[test]
fn escape_needs_a_player_unit_in_the_exit_area() {
    let s = with_objective(cast(), escape());
    assert!(s.can_escape(UnitId(1), p(3, 0)));
    assert!(s.can_escape(UnitId(2), p(3, 4)));
    assert!(!s.can_escape(UnitId(1), p(2, 0)));
    // The enemy side never escapes.
    assert!(!s.can_escape(UnitId(3), p(3, 0)));
    assert!(!s.can_escape(UnitId(99), p(3, 0)));
    let mut rout = with_objective(cast(), Objective::Rout { turn_limit: None });
    assert!(!rout.can_escape(UnitId(1), p(3, 0)));
    refused_act(
        &mut rout,
        1,
        p(3, 0),
        UnitAction::Escape,
        CommandError::CannotEscape,
    );
    let mut s = s;
    refused_act(
        &mut s,
        1,
        p(2, 0),
        UnitAction::Escape,
        CommandError::CannotEscape,
    );
    assert_eq!(escape().turn_limit(), None);
}

#[test]
fn escape_is_won_when_the_last_one_steps_out() {
    let mut s = leveled(runners(), escape());
    let events = act(&mut s, 1, p(3, 0), UnitAction::Escape);
    assert_eq!(
        events,
        [
            Event::UnitMoved {
                unit: UnitId(1),
                path: vec![p(0, 0), p(1, 0), p(2, 0), p(3, 0)],
            },
            Event::UnitEscaped { unit: UnitId(1) },
        ]
    );
    assert_eq!(s.unit(UnitId(1)), None);
    assert_eq!(s.escaped().len(), 1);
    assert_eq!(s.outcome(), None);
    // Gone for the battle: it can't act again.
    let equip = Command::Equip {
        unit: UnitId(1),
        equipped: Equipped::Weapon(0),
    };
    refused(&mut s, &equip, CommandError::UnitEscaped(UnitId(1)));
    let events = act(&mut s, 2, p(3, 3), UnitAction::Escape);
    assert_eq!(events.last(), Some(&ended(Outcome::Victory)));
    // Escaping gives no EXP.
    assert_eq!(exp(&events), []);
    assert_eq!(s.units().len(), 2, "only the enemies are left");
}

#[test]
fn the_last_one_falling_with_others_out_wins_and_the_escaped_lead_cant_lose_it() {
    let units = vec![
        lord(1, p(0, 0)),
        unit(2, Faction::Player, p(0, 2)),
        unit(3, Faction::Enemy, p(7, 0)),
        armed(unit(4, Faction::Enemy, p(0, 4)), 10),
    ];
    let mut s = with_objective(units, escape());
    act(&mut s, 1, p(3, 0), UnitAction::Escape);
    act(&mut s, 2, p(0, 2), UnitAction::Wait);
    end(&mut s);
    // The lead is out, so it can't fall: the lord rule can't lose this.
    assert_eq!(s.outcome(), None);
    // Every unit still on the map falls, but one escaped: a win.
    let events = act(&mut s, 4, p(0, 3), attack(2));
    assert!(events.contains(&Event::UnitFell { unit: UnitId(2) }));
    assert_eq!(events.last(), Some(&ended(Outcome::Victory)));
}

#[test]
fn with_nobody_out_the_last_fall_still_loses() {
    let units = vec![
        unit(2, Faction::Player, p(0, 2)),
        armed(unit(4, Faction::Enemy, p(0, 4)), 10),
    ];
    let mut s = with_objective(units, escape());
    end(&mut s);
    let events = act(&mut s, 4, p(0, 3), attack(2));
    assert_eq!(events.last(), Some(&ended(Outcome::Defeat)));
}

#[test]
fn escaped_units_share_the_exp_pool() {
    let mut units = runners();
    units[3].pos = p(6, 4);
    units.push(armed(unit(5, Faction::Ally, p(7, 4)), 12));
    let mut s = leveled(units, escape());
    walk(&mut s, 2);
    // The ally's kill fills the pool.
    act(&mut s, 5, p(7, 4), attack(4));
    assert!(s.exp_pool() > 0);
    end(&mut s);
    assert_eq!(exp(&act(&mut s, 1, p(3, 0), UnitAction::Escape)), []);
    let events = act(&mut s, 2, p(3, 3), UnitAction::Escape);
    let shared: Vec<UnitId> = exp(&events).into_iter().map(|(u, _)| u).collect();
    assert_eq!(shared, [UnitId(1), UnitId(2)]);
    assert_eq!(s.exp_pool(), 0);
    assert_eq!(events.last(), Some(&ended(Outcome::Victory)));
}

// ---- A unit that retreats --------------------------------------------------

#[test]
fn a_boss_who_retreats_at_the_killing_blow_wins_defeat_unit_with_kill_exp() {
    let mut units = skirmish();
    units[3] = Unit {
        role: Role::Boss,
        ..defeat(units[3].clone(), Defeat::Retreats)
    };
    let objective = Objective::DefeatUnit {
        unit: UnitId(4),
        turn_limit: None,
        leftovers: Leftovers::Stay,
    };
    let mut s = leveled(units, objective);
    let events = act(&mut s, 2, p(1, 2), attack(4));
    assert!(events.contains(&Event::UnitFell { unit: UnitId(4) }));
    let kill = exp_for_combat(1, 1, CombatResult::Killed, true);
    assert_eq!(exp(&events), [(UnitId(2), kill)]);
    assert_eq!(events.last(), Some(&ended(Outcome::Victory)));
    // It left the map: it counts as defeated.
    assert_eq!(s.unit(UnitId(4)), None);
    assert!(s.is_defeated(UnitId(4)));
    assert_eq!(s.fallen()[0].id, UnitId(4));
}

// ---- A unit that can't be killed -------------------------------------------

/// [`skirmish`] with enemy 4 (at (2,2)) unkillable and enemy 5 far away,
/// so the battle goes on.
fn unkillable() -> Vec<Unit> {
    let mut units = skirmish();
    units[3] = defeat(units[3].clone(), Defeat::GoesDown);
    units.push(unit(5, Faction::Enemy, p(7, 4)));
    units
}

#[test]
fn an_unkillable_unit_goes_down_with_kill_exp_to_the_striker() {
    let mut s = leveled(unkillable(), Objective::Rout { turn_limit: None });
    let events = act(&mut s, 2, p(1, 2), attack(4));
    assert!(events.contains(&Event::UnitDowned { unit: UnitId(4) }));
    assert!(!events.contains(&Event::UnitFell { unit: UnitId(4) }));
    let kill = exp_for_combat(1, 1, CombatResult::Killed, false);
    assert_eq!(exp(&events), [(UnitId(2), kill)]);
    assert_eq!(events.last(), Some(&Event::UnitActed { unit: UnitId(2) }));
    // Down on its tile at 0 HP, defeated, the battle going on.
    assert_eq!(s.unit(UnitId(4)), None);
    let down = &s.down()[0];
    assert_eq!((down.id, down.pos, down.hp), (UnitId(4), p(2, 2), 0));
    assert_eq!(s.fallen(), []);
    assert!(s.is_defeated(UnitId(4)));
    assert_eq!(s.outcome(), None);
}

#[test]
fn a_down_unit_cant_act_be_targeted_or_be_walked_through() {
    let mut s = start(setup(unkillable()));
    act(&mut s, 2, p(1, 2), attack(4));
    let err = CommandError::UnitDown(UnitId(4));
    // Nobody can target it.
    refused_act(&mut s, 1, p(2, 1), attack(4), err.clone());
    // Nobody stops on its tile or walks through it, on either side.
    refused_act(
        &mut s,
        1,
        p(2, 2),
        UnitAction::Wait,
        CommandError::CannotStop(p(2, 2)),
    );
    let through = |s: &BattleState, id: u32, path: &[Pos]| {
        crate::movement::path_cost(
            s.map(),
            s.terrain(),
            s.classes(),
            s.units(),
            UnitId(id),
            path,
        )
    };
    let path = [p(1, 2), p(2, 2), p(3, 2)];
    assert_eq!(
        through(&s, 2, &path),
        Err(crate::movement::PathError::Impassable { index: 1 })
    );
    let path = [p(2, 0), p(2, 1), p(2, 2), p(2, 3)];
    assert_eq!(
        through(&s, 3, &path),
        Err(crate::movement::PathError::Impassable { index: 2 })
    );
    let reach = reachable(s.map(), s.terrain(), s.classes(), s.units(), UnitId(3)).unwrap();
    assert!(!reach.is_passable(p(2, 2)));
    // It never acts again.
    end(&mut s);
    refused_act(&mut s, 4, p(2, 2), UnitAction::Wait, err);
    let legal = legal_commands(&s);
    assert!(
        !legal
            .iter()
            .any(|c| matches!(c, Command::Act { unit, .. } if *unit == UnitId(4))),
        "{legal:?}"
    );
}

#[test]
fn a_down_unit_counts_as_defeated_for_the_objective() {
    let objective = Objective::DefeatUnit {
        unit: UnitId(4),
        turn_limit: None,
        leftovers: Leftovers::Stay,
    };
    let mut s = with_objective(unkillable(), objective);
    let events = act(&mut s, 2, p(1, 2), attack(4));
    assert_eq!(events.last(), Some(&ended(Outcome::Victory)));
    // A rout is won with it down.
    let mut units = unkillable();
    units.pop();
    let mut s = start(setup(units));
    act(&mut s, 2, p(1, 2), attack(4));
    let events = act(&mut s, 1, p(1, 0), attack(3));
    assert_eq!(events.last(), Some(&ended(Outcome::Victory)));
}

#[test]
fn a_reinforcement_waits_while_a_down_unit_holds_its_tile() {
    let mut s = start(BattleSetup {
        reinforcements: vec![Reinforcement {
            turn: 1,
            unit: unit(9, Faction::Enemy, p(2, 2)),
        }],
        ..setup(unkillable())
    });
    act(&mut s, 2, p(1, 2), attack(4));
    end(&mut s);
    assert_eq!(s.unit(UnitId(9)), None);
    assert_eq!(s.reinforcements().len(), 1);
}

#[test]
fn a_down_units_defeated_line_plays_and_it_stays_in_the_scenes() {
    let named = |mut u: Unit| {
        u.character = Some(CharacterId(format!("c{}", u.id.0)));
        u
    };
    let units: Vec<Unit> = unkillable().into_iter().map(named).collect();
    let when = TriggerWhen::UnitFell {
        unit: CharacterId("c4".into()),
        mode: None,
        recruit: true,
    };
    let triggers = vec![
        Trigger {
            when,
            scene: "down".into(),
            once: true,
        },
        Trigger {
            when: TriggerWhen::TurnStart {
                turn: 1,
                phase: Phase::Enemy,
            },
            scene: "later".into(),
            once: true,
        },
    ];
    let mut s = start(BattleSetup {
        triggers,
        ..setup(units)
    });
    let events = act(&mut s, 2, p(1, 2), attack(4));
    let at = events
        .iter()
        .position(|e| *e == Event::UnitDowned { unit: UnitId(4) })
        .unwrap();
    let c4 = CharacterId("c4".into());
    assert!(matches!(&events[at - 1],
        Event::SceneTriggered { scene, present } if scene == "down" && present.contains(&c4)));
    // A downed unit is never recruited.
    assert_eq!(s.recruited(), []);
    let events = end(&mut s);
    assert!(matches!(events.last(),
        Some(Event::SceneTriggered { scene, present }) if scene == "later" && present.contains(&c4)));
}

#[test]
fn the_new_errors_read_well() {
    assert_eq!(
        CommandError::UnitDown(UnitId(4)).to_string(),
        "unit 4 is down"
    );
    assert_eq!(
        CommandError::UnitEscaped(UnitId(1)).to_string(),
        "unit 1 has escaped"
    );
    assert_eq!(CommandError::CannotEscape.to_string(), "can't escape here");
}

// ---- Properties ------------------------------------------------------------

prop_compose! {
    /// A random battle ([`arb_setup`]) whose enemies retreat or can't be
    /// killed at random, an escape battle half the time (the exit: the
    /// two right-hand columns).
    fn arb_act1_setup()(
        setup in arb_setup(),
        defeats in prop::collection::vec(0usize..3, 11),
        escape in any::<bool>(),
    ) -> BattleSetup {
        let mut setup = setup;
        let kinds = [Defeat::Falls, Defeat::Retreats, Defeat::GoesDown];
        let enemies = setup.units.iter_mut().chain(setup.reinforcements.iter_mut().map(|r| &mut r.unit));
        for (u, &d) in enemies.zip(&defeats) {
            if u.faction == Faction::Enemy {
                u.defeat = kinds[d];
            }
        }
        if escape {
            setup.objective = Objective::Escape { area: TileRect { x: 6, y: 0, w: 2, h: 6 } };
        }
        setup
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        max_shrink_iters: 32,
        max_shrink_time: 5_000,
        ..ProptestConfig::with_cases(96)
    })]

    #[test]
    fn no_legal_play_kills_an_unkillable_unit_or_lets_a_down_or_escaped_unit_act(
        setup in arb_act1_setup(),
        choices in prop::collection::vec(any::<u16>(), 200),
    ) {
        let unkillable: BTreeSet<UnitId> = setup
            .units
            .iter()
            .chain(setup.reinforcements.iter().map(|r| &r.unit))
            .filter(|u| u.defeat == Defeat::GoesDown)
            .map(|u| u.id)
            .collect();
        let mut gone: BTreeSet<UnitId> = BTreeSet::new();
        random_play(setup, &choices, |s, legal| {
            for u in s.fallen() {
                prop_assert!(!unkillable.contains(&u.id), "{:?} died", u.id);
            }
            for u in s.down().iter().chain(s.escaped()) {
                gone.insert(u.id);
            }
            for u in s.down() {
                prop_assert!(s.map().blocked.contains(&u.pos));
                prop_assert!(s.units().iter().all(|o| o.pos != u.pos));
            }
            prop_assert_eq!(s.map().blocked.len(), s.down().len());
            for u in s.units() {
                prop_assert!(!gone.contains(&u.id), "{:?} is back", u.id);
            }
            for cmd in legal {
                let actor = match cmd {
                    Command::Act { unit, .. }
                    | Command::Equip { unit, .. }
                    | Command::MoveAfter { unit, .. }
                    | Command::Talk { unit, .. } => Some(*unit),
                    Command::EndPhase => None,
                };
                prop_assert!(actor.is_none_or(|a| !gone.contains(&a)), "{:?}", cmd);
                if let Command::Act { action: UnitAction::Attack { target, .. }, .. }
                | Command::Talk { target, .. } = cmd
                {
                    prop_assert!(!gone.contains(target), "{:?}", cmd);
                }
            }
            Ok(())
        })?;
    }
}

/// A unit escaping is still on the map for a scene of its move into the
/// exit area: it leaves after it.
#[test]
fn a_unit_escaping_is_there_for_the_scene_of_its_move() {
    let mut units = cast();
    units[0].character = Some(CharacterId("c1".into()));
    let when = TriggerWhen::UnitEntersArea {
        who: Who::Faction(Faction::Player),
        area: EXIT,
    };
    let trigger = Trigger {
        when,
        scene: "out".into(),
        once: true,
    };
    let mut s = start(BattleSetup {
        objective: escape(),
        triggers: vec![trigger],
        ..setup(units)
    });
    let events = act(&mut s, 1, p(3, 0), UnitAction::Escape);
    let c1 = CharacterId("c1".into());
    assert!(matches!(&events[1],
        Event::SceneTriggered { scene, present } if scene == "out" && present.contains(&c1)));
    assert_eq!(events[2], Event::UnitEscaped { unit: UnitId(1) });
}

/// A unit's [`Defeat`] is saved, but for the default (every roster unit),
/// which a save leaves out.
#[test]
fn a_units_defeat_is_saved_unless_it_falls() {
    for kind in [Defeat::Retreats, Defeat::GoesDown] {
        let u = defeat(unit(4, Faction::Enemy, p(0, 0)), kind);
        let text = ron::to_string(&u).unwrap();
        assert!(text.contains(&format!("{kind:?}")), "{text}");
        assert_eq!(ron::from_str::<Unit>(&text).unwrap(), u);
    }
    let falls = ron::to_string(&unit(4, Faction::Enemy, p(0, 0))).unwrap();
    assert!(!falls.contains("defeat"), "{falls}");
}
