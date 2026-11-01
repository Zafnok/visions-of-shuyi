use super::*;
use trpg_core::{BattleMusic, Difficulty, ItemId, RandomSkirmish};

fn refs(c: &Content) -> WorldRefs<'_> {
    WorldRefs {
        battles: &c.battles,
        chapters: &c.chapters,
        dialogue: &c.dialogue,
        characters: &c.characters,
        items: &c.items,
        audio: &c.audio,
    }
}

#[test]
fn templates_build_at_the_saved_level_size_seed_and_reward() {
    let c = crate::load_embedded().expect("content");
    let rules = c.worldmaps["test_world"]
        .map
        .skirmishes
        .as_ref()
        .expect("pool");
    assert_eq!(
        (rules.spawn, rules.maximum, rules.level_offset, rules.size),
        ((0, 2), 3, 1, (4, 6))
    );
    for id in &rules.pool {
        for size in 4..=6 {
            for level in 1..=6 {
                for reward in [
                    None,
                    Some(Loot::Gold(100)),
                    Some(Loot::Item(ItemId("potion".into()))),
                ] {
                    let group = RandomSkirmish {
                        battle: id.clone(),
                        level,
                        size,
                        seed: 42,
                        reward: reward.clone(),
                    };
                    let b = battle::random_skirmish(&group, &c.battle_refs()).expect("built");
                    assert_eq!(b.enemies.len(), size as usize);
                    assert!(b.enemies.iter().all(|u| u.level == level
                        && u.character.is_none()
                        && u.loadout.weapon(0).is_some()));
                    assert_eq!(b.seed, 42);
                    assert_eq!(b.difficulty, Difficulty::Easy);
                    assert_eq!(b.music, BattleMusic::Pool("skirmish".into()));
                    assert_eq!(
                        b.clear_gold,
                        if reward == Some(Loot::Gold(100)) {
                            100
                        } else {
                            0
                        }
                    );
                    assert_eq!(
                        b.clear_items,
                        if matches!(reward, Some(Loot::Item(_))) {
                            vec![ItemId("potion".into())]
                        } else {
                            vec![]
                        }
                    );
                    // Rebuilt stats and spell learning come from the template's class at this level.
                    let enemy = &b.enemies[0];
                    let expected = trpg_core::Unit::generic(
                        enemy.id,
                        &enemy.class,
                        &c.classes,
                        level,
                        enemy.faction,
                        enemy.pos,
                    )
                    .expect("class");
                    assert_eq!(enemy.stats, expected.stats);
                    assert_eq!(enemy.learned, expected.learned);
                }
            }
        }
    }
}

#[test]
fn invalid_ranges_pool_files_and_special_rewards_are_rejected() {
    let c = crate::load_embedded().expect("content");
    let rules = c.worldmaps["test_world"]
        .map
        .skirmishes
        .clone()
        .expect("pool");
    let errors = |r: &SkirmishRules| {
        let mut errors = Vec::new();
        check(r, refs(&c), &mut |e| errors.push(e));
        errors
    };
    assert_eq!(errors(&rules), Vec::<String>::new());
    // Both ends are inclusive: fixed counts, including the capacity itself,
    // are valid tuning choices. Zero capacity with zero spawns disables groups.
    for (count, size) in [(0, 4), (3, 6)] {
        let fixed = SkirmishRules {
            spawn: (count, count),
            maximum: count,
            size: (size, size),
            ..rules.clone()
        };
        assert_eq!(errors(&fixed), Vec::<String>::new());
    }
    for bad in [
        SkirmishRules {
            spawn: (3, 2),
            ..rules.clone()
        },
        SkirmishRules {
            maximum: 1,
            ..rules.clone()
        },
        SkirmishRules {
            size: (0, 6),
            ..rules.clone()
        },
        SkirmishRules {
            size: (6, 4),
            ..rules.clone()
        },
        SkirmishRules {
            size: (4, 7),
            ..rules.clone()
        },
        SkirmishRules {
            pool: vec![],
            ..rules.clone()
        },
        SkirmishRules {
            pool: vec!["missing".into()],
            ..rules.clone()
        },
        SkirmishRules {
            pool: vec!["test".into()],
            ..rules.clone()
        },
        SkirmishRules {
            rewards: vec![],
            ..rules.clone()
        },
        SkirmishRules {
            rewards: vec![Some(Loot::Item(ItemId("moonblade".into())))],
            ..rules.clone()
        },
        SkirmishRules {
            rewards: vec![Some(Loot::Item(ItemId("tier_2_seal".into())))],
            ..rules.clone()
        },
        SkirmishRules {
            rewards: vec![Some(Loot::Item(ItemId("missing".into())))],
            ..rules
        },
    ] {
        assert!(!errors(&bad).is_empty(), "{bad:?}");
    }
}

fn saved_group() -> RandomSkirmish {
    RandomSkirmish {
        battle: "test_random_meadow".into(),
        level: 4,
        size: 4,
        seed: 42,
        reward: Some(Loot::Gold(100)),
    }
}

#[test]
fn saved_groups_reject_invalid_level_count_and_missing_battle() {
    let c = crate::load_embedded().expect("content");
    let group = saved_group();
    for invalid in [
        RandomSkirmish {
            level: 0,
            ..group.clone()
        },
        RandomSkirmish {
            size: 0,
            ..group.clone()
        },
        RandomSkirmish {
            size: 7,
            ..group.clone()
        },
        RandomSkirmish {
            battle: "missing".into(),
            ..group
        },
    ] {
        assert!(
            battle::random_skirmish(&invalid, &c.battle_refs()).is_err(),
            "{invalid:?}"
        );
    }
}

#[test]
fn resolving_a_saved_group_uses_its_rolls_and_preserves_ordinary_battle_lookup() {
    use trpg_core::{GameMode, LeadGender, LeadProfile};
    let c = crate::load_embedded().expect("content");
    let mut campaign = crate::battle_campaign(
        &c,
        &c.battles["test"],
        GameMode::Classic,
        LeadProfile::new("Mara", LeadGender::Female),
    );
    campaign.enter_world(&c.worldmaps["test_world"].map);
    let group = saved_group();
    let world = campaign.world.as_mut().expect("world");
    world.at = "ford".into();
    world.cleared.insert("ford".into());
    world.skirmishes.insert("ford".into(), group.clone());
    let expected = battle::random_skirmish(&group, &c.battle_refs()).expect("generated");
    assert_eq!(battle(&c, &campaign, &group.battle), Some(expected));
    assert_eq!(
        battle(&c, &campaign, "test"),
        Some(c.battles["test"].clone())
    );
    assert_eq!(battle(&c, &campaign, "missing"), None);
    // A corrupt matching group must not silently become the raw pool file.
    campaign
        .world
        .as_mut()
        .expect("world")
        .skirmishes
        .get_mut("ford")
        .expect("group")
        .level = 0;
    assert_eq!(battle(&c, &campaign, &group.battle), None);
}
