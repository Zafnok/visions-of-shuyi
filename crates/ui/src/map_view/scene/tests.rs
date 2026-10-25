use super::*;
use crate::screen::tests::ctx;

fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

fn brigand(id: u32, pos: Pos) -> UnitView {
    UnitView {
        id: UnitId(id),
        pos,
        faction: Faction::Enemy,
        label: "Br".into(),
        class: ClassId("brigand".into()),
        character: None,
        acted: false,
        fight: crate::map_view::scene::Fight::In,
        hp: (20, 30),
        effects: UnitEffects::default(),
        fade: 0.0,
        highlight: false,
        facing: Facing::Down,
        frame: STANDING_FRAME,
        offset: (0.0, 0.0),
        flash: 0.0,
        marks: true,
    }
}

#[test]
fn a_walking_unit_has_its_facing_offset_and_frame_in_the_text() {
    let line = |u: &UnitView| unit_text(u);
    let still = brigand(4, p(0, 2));
    assert_eq!(line(&still), "#4 Br enemy brigand (0,2) hp 20/30");
    assert!(!still.between_tiles());
    // Stepping on the spot: the frame isn't shown.
    let mut stepping = still.clone();
    stepping.frame = 0;
    assert_eq!(line(&stepping), line(&still));
    let mut walker = still.clone();
    walker.facing = Facing::Right;
    walker.offset = (0.5, 0.0);
    walker.frame = 2;
    assert!(walker.between_tiles());
    assert_eq!(
        line(&walker),
        "#4 Br enemy brigand (0,2) hp 20/30 facing=right offset=(0.50,0.00) frame=2"
    );
    // Walking down: it faces the camera, so only the offset shows it.
    walker.facing = Facing::Down;
    walker.offset = (0.0, 0.25);
    walker.frame = STANDING_FRAME;
    assert_eq!(
        line(&walker),
        "#4 Br enemy brigand (0,2) hp 20/30 offset=(0.00,0.25)"
    );
    assert!(walker.between_tiles());
    // On a tile of its walk, turned.
    walker.offset = (0.0, 0.0);
    for (facing, name) in [(Facing::Left, "left"), (Facing::Up, "up")] {
        walker.facing = facing;
        walker.frame = 0;
        let expect = format!("#4 Br enemy brigand (0,2) hp 20/30 facing={name} frame=0");
        assert_eq!(line(&walker), expect);
    }
    assert_eq!((Facing::default(), STANDING_FRAME), (Facing::Down, 1));
}

#[test]
fn a_struck_unit_has_its_flash_in_the_text() {
    let mut struck = brigand(4, p(0, 2));
    assert!(!unit_text(&struck).contains("flash"));
    struck.flash = 0.5;
    assert_eq!(
        unit_text(&struck),
        "#4 Br enemy brigand (0,2) hp 20/30 flash=0.50"
    );
    // After everything else.
    struck.fade = 0.25;
    struck.facing = Facing::Left;
    struck.flash = 1.0;
    assert_eq!(
        unit_text(&struck),
        "#4 Br enemy brigand (0,2) hp 20/30 fade=0.25 facing=left flash=1.00"
    );
    // Its line in the scene is this text.
    assert_eq!(struck.text(), unit_text(&struck));
    assert_eq!((struck.id(), MapUnit::pos(&struck)), (UnitId(4), p(0, 2)));
}

#[test]
fn ranges_are_named_by_their_colours() {
    let names = [
        (range::DANGER, "danger_zone"),
        (range::MOVE, "move_range"),
        (range::ATTACK, "attack_range"),
        (range::HEAL, "heal_range"),
        (range::EXIT, "exit_area"),
    ];
    for (kind, name) in names {
        assert_eq!(kind, RangeName(name));
    }
    assert_ne!(range::MOVE, range::HEAL);
    assert_eq!(range::ALL.len(), names.len());
    // Their letters in the scene's text.
    let marks: String = range::ALL.iter().map(|k| k.mark()).collect();
    assert_eq!(marks, "dmahe");
}

#[test]
fn tints_go_on_visible_tiles_in_the_order_laid_on() {
    let mut s = MapScene::new(p(1, 1), (3, 2));
    s.tint([p(1, 1), p(3, 2), p(0, 0), p(4, 1)], range::DANGER);
    s.tint([p(3, 2), p(2, 1)], range::MOVE);
    assert_eq!(s.tints_at(p(1, 1)), [range::DANGER]);
    assert_eq!(s.tints_at(p(3, 2)), [range::DANGER, range::MOVE]);
    assert_eq!(s.tinted(range::DANGER), [p(1, 1), p(3, 2)]);
    assert_eq!(s.tinted(range::MOVE), [p(2, 1), p(3, 2)]);
    assert_eq!(s.tinted(range::ATTACK), [] as [trpg_core::Pos; 0]);
}

#[test]
fn only_units_on_visible_tiles_are_kept() {
    let mut s = MapScene::new(p(1, 1), (3, 2));
    s.push_unit(brigand(4, p(3, 2)));
    s.push_unit(brigand(5, p(4, 2)));
    s.push_unit(brigand(6, p(1, 1)));
    let ids: Vec<u32> = s.units.iter().map(|u| u.id.0).collect();
    assert_eq!(ids, [4, 6]);
    assert_eq!(s.unit(UnitId(6)).map(|u| u.pos), Some(p(1, 1)));
    assert_eq!(s.unit(UnitId(5)), None);
    assert_eq!(s.unit_at(p(3, 2)).map(|u| u.id), Some(UnitId(4)));
}

#[test]
fn a_unit_view_copies_what_the_map_shows_of_a_unit() {
    let c = ctx();
    let state = crate::screens::battle::quick_battle(&c.game.content).unwrap();
    let lord = &state.units()[0];
    let v = UnitView::of(lord);
    assert_eq!((v.id, v.pos, v.faction), (lord.id, lord.pos, lord.faction));
    assert_eq!(v.label, "Lo");
    assert_eq!(v.class, lord.class);
    assert_eq!(v.character, lord.character);
    assert!(v.character.is_some());
    assert_eq!(v.hp, (lord.hp, lord.stats.hp));
    assert!(!v.acted && !v.has_effect() && !v.highlight);
    assert_eq!((v.facing, v.frame, v.offset), (Facing::Down, 1, (0.0, 0.0)));
    assert!(!v.effects.bonus && !v.effects.penalty);
    assert!(v.fade.abs() < f32::EPSILON);
    let mut hurt = lord.clone();
    hurt.hp = 3;
    hurt.acted = true;
    let v = UnitView::of(&hurt).at(p(9, 9)).fading(0.5);
    assert!(!v.highlight && v.clone().highlighted(true).highlight);
    assert!(!v.clone().highlighted(true).highlighted(false).highlight);
    assert_eq!((v.pos, v.hp.0, v.acted), (p(9, 9), 3, true));
    assert!((v.fade - 0.5).abs() < f32::EPSILON);
    assert_eq!(v.id, lord.id);
}

/// A timed effect of `mods` from the skill `source`.
fn effect(source: &str, mods: TimedMods) -> trpg_core::skill::TimedEffect {
    trpg_core::skill::TimedEffect {
        source: trpg_core::SkillId::new(source).into(),
        mods,
        until: trpg_core::Phase::Player,
    }
}

#[test]
fn an_effect_that_lowers_a_number_is_a_penalty_and_any_other_a_bonus() {
    use trpg_core::StatKind;
    use trpg_core::combat::CombatMods;
    let stat = |amount| TimedMods {
        stats: vec![(StatKind::Mov, amount)],
        ..TimedMods::default()
    };
    let combat = |combat| TimedMods {
        combat,
        ..TimedMods::default()
    };
    let none = CombatMods::default;
    assert!(hinders(&stat(-3)));
    assert!(!hinders(&stat(3)));
    assert!(!hinders(&stat(0)));
    assert!(!hinders(&TimedMods::default()));
    assert!(!hinders(&combat(CombatMods {
        avoid: 20,
        ..none()
    })));
    for lowered in [
        CombatMods { hit: -1, ..none() },
        CombatMods { crit: -1, ..none() },
        CombatMods {
            might: -1,
            ..none()
        },
        CombatMods {
            avoid: -1,
            ..none()
        },
        CombatMods {
            attack_speed: -1,
            ..none()
        },
        CombatMods {
            pierce: -1,
            ..none()
        },
        CombatMods {
            single_strike: true,
            ..none()
        },
    ] {
        assert!(hinders(&combat(lowered)), "{lowered:?}");
    }
    // Raised in one number, lowered in another: a penalty.
    let mixed = TimedMods {
        stats: vec![(StatKind::Str, 2), (StatKind::Spd, -3)],
        ..TimedMods::default()
    };
    assert!(hinders(&mixed));
    // On a unit: either kind, or both.
    let c = ctx();
    let state = crate::screens::battle::quick_battle(&c.game.content).unwrap();
    let mut unit = state.units()[0].clone();
    unit.add_effect(effect(
        "sidestep",
        combat(CombatMods {
            avoid: 20,
            ..none()
        }),
    ));
    let v = UnitView::of(&unit);
    assert!(v.effects.bonus && !v.effects.penalty && v.has_effect());
    unit.add_effect(effect("pinning_shot", stat(-3)));
    let v = UnitView::of(&unit);
    assert!(v.effects.bonus && v.effects.penalty && v.has_effect());
    unit.effects.remove(0);
    let v = UnitView::of(&unit);
    assert!(!v.effects.bonus && v.effects.penalty && v.has_effect());
}

#[test]
fn the_terrains_are_the_contents_by_id() {
    let c = ctx();
    let display = &c.game.content.terrain.display;
    let looks = TerrainLooks(display);
    let forest = display.id_of("forest").unwrap();
    let t = display.get(forest).unwrap();
    assert_eq!(looks.name(forest), Some("forest"));
    let glyphs = looks.glyphs(forest).unwrap();
    assert_eq!(
        (glyphs.glyphs, glyphs.fg, glyphs.bg),
        (t.glyphs, t.fg.as_str(), t.bg.as_str())
    );
    assert_eq!(looks.name(TerrainId(999)), None);
    assert!(looks.glyphs(TerrainId(999)).is_none());
}

#[test]
fn to_text_lists_tiles_units_cursor_and_path() {
    let c = ctx();
    let display = &c.game.content.terrain.display;
    let id = |name| display.id_of(name);
    let mut s = MapScene::new(p(-1, 2), (5, 3));
    for pos in [p(0, 2), p(1, 2), p(2, 2), p(0, 3), p(1, 3)] {
        s.tile_mut(pos).unwrap().terrain = id("plain");
    }
    s.tile_mut(p(3, 2)).unwrap().terrain = id("forest");
    s.tile_mut(p(2, 3)).unwrap().terrain = Some(TerrainId(999));
    s.tint([p(1, 2), p(2, 2)], range::MOVE);
    s.tint([p(3, 2)], range::DANGER);
    s.tint([p(3, 2)], range::ATTACK);
    s.tint([p(0, 3)], range::HEAL);
    s.tile_mut(p(1, 3)).unwrap().flashes.push(0.5);
    s.tile_mut(p(0, 2)).unwrap().becomes = id("forest");
    s.tile_mut(p(2, 2)).unwrap().becomes = Some(TerrainId(998));
    s.push_unit(brigand(4, p(0, 2)));
    let mut lord = brigand(1, p(1, 3)).fading(0.25);
    lord.faction = Faction::Player;
    lord.label = "Lo".into();
    lord.class = ClassId("lord".into());
    lord.character = Some(CharacterId("test_lord".into()));
    lord.acted = true;
    lord.effects.bonus = true;
    lord.highlight = true;
    s.push_unit(lord);
    s.cursor = Some(CursorView::new(p(1, 3), 0.756, CursorStyle::Corners));
    s.path = vec![p(1, 3), p(1, 2)];
    assert_eq!(
        s.to_text(&TerrainLooks(display)),
        "origin (-1,2) size 5x3\n\
         \x20  2: - plain>forest plain+m plain+m>?998 forest+da\n\
         \x20  3: - plain+h plain! ?999 -\n\
         \x20  4: -*5\n\
         units 2\n\
         \x20 #4 Br enemy brigand (0,2) hp 20/30\n\
         \x20 #1 Lo player lord [test_lord] (1,3) hp 20/30 acted bonus fade=0.25 highlight\n\
         cursor (1,3) corners 0.76\n\
         path (1,3) (1,2)\n"
    );
    // Enemies threatening a tile add a line: the tile, then theirs.
    s.threat = Some(ThreatView {
        tile: p(1, 2),
        enemies: vec![p(0, 2), p(9, -4)],
    });
    let text = s.to_text(&TerrainLooks(display));
    let end = "path (1,3) (1,2)\nthreat (1,2) from (0,2) (9,-4)\n";
    assert!(text.ends_with(end), "{text}");
}

#[test]
fn to_text_names_every_side() {
    let c = ctx();
    let looks = TerrainLooks(&c.game.content.terrain.display);
    let mut s = MapScene::new(p(0, -1), (2, 1));
    let factions = [(Faction::Ally, "ally"), (Faction::Neutral, "neutral")];
    for (faction, name) in factions {
        let mut u = brigand(7, p(0, -1));
        u.faction = faction;
        // Standing or (wrongly) negative: no fade shown.
        u.fade = -1.0;
        u.effects.penalty = true;
        u.effects.bonus = faction == Faction::Ally;
        s.units = vec![u];
        let effects = if faction == Faction::Ally {
            "bonus penalty"
        } else {
            "penalty"
        };
        let line = format!("  #7 Br {name} brigand (0,-1) hp 20/30 {effects}\n");
        assert!(s.to_text(&looks).contains(&line), "{name}");
    }
    // One out of a conversation's opening has no marks.
    let mut u = brigand(7, p(0, -1));
    u.marks = false;
    assert!(unit_text(&u).ends_with(" no-marks"));
}
