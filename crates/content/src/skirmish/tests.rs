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
            rewards: vec![Some(Loot::Item(ItemId("missing".into())))],
            ..rules
        },
    ] {
        assert!(!errors(&bad).is_empty(), "{bad:?}");
    }
}
