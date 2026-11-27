use trpg_core::{SkillId, SpellId};

use super::*;
use crate::screen::tests::ctx;
use crate::screens::battle::quick_battle;

const T: ProgressTimings = PROGRESS_TIMINGS;

#[test]
fn the_leads_pages_show_the_portrait_of_their_gender() {
    use trpg_core::lead::{LEAD_ID, LeadGender};
    let portraits: BTreeMap<String, Portrait> = ["lead_m", "lead_f", "test_lord"]
        .iter()
        .map(|id| {
            let p = Portrait {
                character: (*id).into(),
                expressions: vec![],
            };
            ((*id).to_string(), p)
        })
        .collect();
    let shown = |gender, id: Option<&str>| {
        portrait_of(&portraits, &LeadProfile::new("Ren", gender), id)
            .map(|(_, p)| p.character.clone())
    };
    assert_eq!(
        shown(LeadGender::Male, Some(LEAD_ID)).as_deref(),
        Some("lead_m")
    );
    assert_eq!(
        shown(LeadGender::Female, Some(LEAD_ID)).as_deref(),
        Some("lead_f")
    );
    assert_eq!(
        shown(LeadGender::Male, Some("test_lord")).as_deref(),
        Some("test_lord")
    );
    assert_eq!(shown(LeadGender::Male, None), None);
}

fn state() -> BattleState {
    quick_battle(&ctx().game.content).unwrap()
}

fn gains(g: [StatValue; 7]) -> StatGains {
    StatGains(g)
}

/// The lord (unit 1, 19 HP, Str 6 …) with 90 EXP gains 30: one level
/// up; then Exile CL 9 → 10, mastery and a passive.
fn events() -> Vec<Event> {
    let lord = UnitId(1);
    vec![
        Event::ExpGained {
            unit: lord,
            amount: 30,
        },
        Event::LeveledUp {
            unit: lord,
            level: 2,
            gains: gains([1, 1, 0, 0, 2, 0, 0]),
        },
        Event::ClassPointsGained {
            unit: lord,
            class: ClassId("exile".into()),
            amount: 3,
        },
        Event::ClassLeveledUp {
            unit: lord,
            class: ClassId("exile".into()),
            class_level: 10,
        },
        Event::ClassMastered {
            unit: lord,
            class: ClassId("exile".into()),
        },
        Event::SkillLearned {
            unit: lord,
            skill: SkillId::new("leadership_1"),
        },
    ]
}

fn progress() -> Progress {
    let s = state();
    let mut before = s.units().to_vec();
    before[0].exp = 90;
    Progress::new(&events(), &before, (&s, Words::ENGLISH), T).unwrap()
}

#[test]
fn pages_come_exp_then_level_up_then_class() {
    let pb = progress();
    let pages = pb.pages();
    assert_eq!(pages.len(), 3);
    assert_eq!(
        pages[0],
        Page::Exp(ExpPage {
            unit: UnitId(1),
            name: "Test Lord".into(),
            from: 90,
            gained: 30,
        })
    );
    let Page::LevelUp(lv) = &pages[1] else {
        panic!("{pages:?}")
    };
    assert_eq!((lv.level, lv.before[0], lv.before[1]), (2, 19, 6));
    assert_eq!(lv.character.as_deref(), Some("test_lord"));
    let Page::Class(cl) = &pages[2] else {
        panic!("{pages:?}")
    };
    assert_eq!(class_line(Words::ENGLISH, cl), "Exile  CL 9 → 10");
    assert!(cl.mastered);
    // Mastery alone is worth stopping for.
    let mastery = Page::Class(ClassPage {
        learned: vec![],
        ..cl.clone()
    });
    assert!(mastery.waits());
    assert!(mastery.len(&T).abs() < f32::EPSILON);
    assert_eq!(cl.learned.len(), 1);
}

#[test]
fn without_exp_bars_keeps_the_other_pages_in_order() {
    let pb = progress().without_exp_bars().unwrap();
    assert_eq!(pb.pages().len(), 2);
    assert!(matches!(pb.pages()[0], Page::LevelUp(_)));
    assert!(matches!(pb.pages()[1], Page::Class(_)));
    // EXP alone: nothing left.
    let s = state();
    let exp = [Event::ExpGained {
        unit: UnitId(1),
        amount: 10,
    }];
    let pb = Progress::new(&exp, s.units(), (&s, Words::ENGLISH), T).unwrap();
    assert_eq!(pb.without_exp_bars(), None);
}

#[test]
fn nothing_gained_no_pages_and_other_factions_are_ignored() {
    let s = state();
    let before = s.units().to_vec();
    assert_eq!(Progress::new(&[], &before, (&s, Words::ENGLISH), T), None);
    let brigand = [Event::ExpGained {
        unit: UnitId(4),
        amount: 10,
    }];
    assert_eq!(
        Progress::new(&brigand, &before, (&s, Words::ENGLISH), T),
        None
    );
}

#[test]
fn two_level_ups_stack_their_stats_and_awards_add_up() {
    let s = state();
    let before = s.units().to_vec();
    let lord = UnitId(1);
    let events = [
        Event::ExpGained {
            unit: lord,
            amount: 150,
        },
        Event::LeveledUp {
            unit: lord,
            level: 2,
            gains: gains([1, 1, 0, 0, 0, 0, 0]),
        },
        Event::ExpGained {
            unit: lord,
            amount: 60,
        },
        Event::LeveledUp {
            unit: lord,
            level: 3,
            gains: gains([2, 0, 0, 0, 0, 0, 1]),
        },
        Event::SpellLearned {
            unit: lord,
            spell: SpellId::new("fire"),
        },
    ];
    let pb = Progress::new(&events, &before, (&s, Words::ENGLISH), T).unwrap();
    let Page::Exp(exp) = &pb.pages()[0] else {
        panic!()
    };
    assert_eq!(exp.gained, 210);
    let Page::LevelUp(second) = &pb.pages()[2] else {
        panic!()
    };
    assert_eq!((second.before[0], second.before[1]), (20, 7));
    let Page::Class(cl) = &pb.pages()[3] else {
        panic!()
    };
    // A spell learned on its own: the class's name, no level.
    assert_eq!(
        (class_line(Words::ENGLISH, cl), cl.mastered),
        ("Exile".to_owned(), false)
    );
    assert_eq!(cl.learned, ["Fire"]);
    // So is something learned.
    assert!(pb.pages()[3].waits());
}

#[test]
fn the_exp_bar_fills_over_its_time_and_wraps_at_100() {
    let mut pb = progress();
    assert_eq!(pb.exp_shown(), Some(90));
    pb.tick(T.exp_fill / 2.0, false);
    assert_eq!(pb.exp_shown(), Some(105));
    // The bar starts over at each level.
    let bar = |pb: &Progress| match shown(pb) {
        Some(ProgressView::Exp(e)) => (e.exp, e.max),
        other => panic!("{other:?}"),
    };
    assert_eq!(bar(&pb), (5, 100));
    pb.tick(T.exp_fill / 2.0, false);
    assert_eq!(pb.exp_shown(), Some(120));
    assert_eq!(bar(&pb), (20, 100));
    // The hold, then the level-up page by itself.
    pb.tick(T.exp_hold - 0.01, false);
    assert_eq!(pb.index(), 0);
    pb.tick(0.02, false);
    assert_eq!(pb.index(), 1);
    assert!(pb.time().abs() < f32::EPSILON);
}

#[test]
fn holding_confirm_speeds_up_and_pressing_finishes_the_bar() {
    let mut pb = progress();
    pb.tick(0.1, true);
    assert!((pb.time() - 0.4).abs() < 1e-5);
    pb.confirm();
    assert_eq!(pb.exp_shown(), Some(120));
    assert!(pb.page_played());
    // A played page that doesn't wait closes on the next frame.
    pb.tick(0.0, false);
    assert_eq!(pb.index(), 1);
}

#[test]
fn stats_are_revealed_one_at_a_time_then_confirm_closes() {
    let mut pb = progress();
    pb.confirm();
    pb.tick(0.0, false);
    assert_eq!(pb.stats_shown(), 0);
    // The fanfare plays through first (Nick, 0605).
    pb.tick(T.fanfare - 0.05, false);
    assert_eq!(pb.stats_shown(), 0);
    pb.tick(0.05, false);
    assert_eq!(pb.stats_shown(), 1);
    pb.tick(T.stat_reveal, false);
    assert_eq!(pb.stats_shown(), 2);
    pb.tick(T.stat_reveal * 2.5, false);
    assert_eq!(pb.stats_shown(), 4);
    pb.tick(10.0, false);
    assert_eq!(pb.stats_shown(), 7);
    // It waits for Confirm.
    assert_eq!(pb.index(), 1);
    pb.confirm();
    assert_eq!(pb.index(), 2);
}

#[test]
fn a_press_reveals_every_stat_before_closing() {
    let mut pb = progress();
    pb.confirm();
    pb.tick(0.0, false);
    pb.tick(T.level_intro, false);
    pb.confirm();
    assert_eq!((pb.index(), pb.stats_shown()), (1, 7));
    pb.confirm();
    // The mastery page waits too; Confirm closes the last page.
    assert!(matches!(pb.page(), Some(Page::Class(_))));
    pb.tick(10.0, false);
    assert!(!pb.done());
    pb.confirm();
    assert!(pb.done());
    assert!(pb.page_played());
}

#[test]
fn a_class_level_alone_closes_by_itself() {
    let s = state();
    let before = s.units().to_vec();
    let events = [Event::ClassLeveledUp {
        unit: UnitId(1),
        class: ClassId("exile".into()),
        class_level: 4,
    }];
    let mut pb = Progress::new(&events, &before, (&s, Words::ENGLISH), T).unwrap();
    assert!(!pb.pages()[0].waits());
    pb.tick(T.class_hold / 2.0, false);
    assert!(!pb.done(), "it stays up a moment");
    pb.tick(T.class_hold / 2.0 + 0.01, false);
    assert!(pb.done());
    // Bad frame times count as nothing.
    let mut pb = Progress::new(&events, &before, (&s, Words::ENGLISH), T).unwrap();
    pb.tick(f32::NAN, false);
    pb.tick(-1.0, false);
    assert!(pb.time().abs() < f32::EPSILON);
}

/// What the page on screen of `pb` shows, the lead a man.
fn shown(pb: &Progress) -> Option<ProgressView> {
    let c = ctx();
    view(
        Words::ENGLISH,
        (&c.game.content.portraits, &c.game.lead),
        pb,
    )
}

#[test]
fn each_page_says_what_it_shows_as_far_as_it_has_played() {
    let mut pb = progress();
    // The EXP bar fills from 90 and starts over at 100.
    let exp = |pb: &Progress| match shown(pb) {
        Some(ProgressView::Exp(e)) => (e.name, e.exp_name, e.exp, e.max),
        other => panic!("{other:?}"),
    };
    assert_eq!(exp(&pb), ("Test Lord".into(), "EXP".into(), 90, 100));
    pb.tick(T.exp_fill, false);
    assert_eq!(exp(&pb).2, 20);
    pb.tick(T.exp_hold, false);
    // The level up: no stat yet, then one at a time.
    let Some(ProgressView::Gains(g)) = shown(&pb) else {
        panic!("{:?}", shown(&pb))
    };
    assert_eq!(
        (g.banner.as_str(), g.name.as_str(), g.line.as_str()),
        ("LEVEL UP!", "Test Lord", "Lv 1 → 2")
    );
    assert_eq!(g.placeholder, "portrait");
    assert_eq!(
        g.stats,
        [] as [crate::screens::battle::view::StatGainView; 0]
    );
    pb.tick(T.fanfare + T.stat_reveal, false);
    let Some(ProgressView::Gains(g)) = shown(&pb) else {
        panic!()
    };
    let hp = StatGainView {
        name: "HP".into(),
        before: 19,
        gain: 1,
    };
    assert_eq!(g.stats.len(), 2);
    assert_eq!(g.stats[0], hp);
    assert_eq!((g.stats[1].name.as_str(), g.stats[1].gain), ("Str", 1));
    pb.confirm();
    pb.confirm();
    // The class page: the class levels, the mastery and what was learned.
    let Some(ProgressView::Class(cl)) = shown(&pb) else {
        panic!("{:?}", shown(&pb))
    };
    assert_eq!(cl.name, "Test Lord");
    assert_eq!(cl.line, "Exile  CL 9 → 10");
    assert_eq!(cl.mastered.as_deref(), Some("CLASS MASTERED!"));
    assert_eq!(cl.lines.len(), 1);
    assert!(cl.lines[0].starts_with("Learned: "), "{:?}", cl.lines);
    pb.confirm();
    assert_eq!(shown(&pb), None);
}

#[test]
fn a_level_up_shows_the_face_of_its_portrait_or_none() {
    let c = ctx();
    let mut pb = progress();
    pb.confirm();
    pb.tick(0.0, false);
    let face = |pb: &Progress| match shown(pb) {
        Some(ProgressView::Gains(g)) => g.face,
        other => panic!("{other:?}"),
    };
    // The test lord's portrait, its level-up expression (or its first).
    let f = face(&pb).expect("the test lord has a portrait");
    assert_eq!(f.portrait, "test_lord");
    let art = &c.game.content.portraits["test_lord"];
    assert!(art.expression(&f.expression).is_some());
    let wanted = LEVEL_UP_EXPRESSIONS
        .iter()
        .find(|e| art.expression(e).is_some())
        .map_or(art.expressions[0].name.as_str(), |e| *e);
    assert_eq!(f.expression, wanted);
    // Someone without a portrait: none.
    let none = BTreeMap::new();
    let lead = LeadProfile::new("Ren", trpg_core::lead::LeadGender::Male);
    assert_eq!(face_of((&none, &lead), Some("test_lord")), None);
    assert_eq!(face_of((&c.game.content.portraits, &lead), None), None);
    // A portrait without any expression: none.
    let blank: BTreeMap<String, Portrait> = [(
        "test_lord".to_owned(),
        Portrait {
            character: "test_lord".into(),
            expressions: vec![],
        },
    )]
    .into();
    assert_eq!(face_of((&blank, &lead), Some("test_lord")), None);
}

/// The lord promotes (ticket 0603): its bonus, a spell and a weapon
/// sent to the stock.
fn promotion_events() -> Vec<Event> {
    let lord = UnitId(1);
    vec![
        Event::Promoted {
            unit: lord,
            from: ClassId("exile".into()),
            to: ClassId("blade_heir".into()),
            gains: gains([6, 3, 0, 4, 5, 2, 2]),
        },
        Event::SpellLearned {
            unit: lord,
            spell: SpellId::new("fire"),
        },
        Event::ItemStowed {
            unit: lord,
            item: trpg_core::ItemId::new("iron_sword"),
        },
    ]
}

#[test]
fn a_promotion_shows_its_bonus_like_a_level_up_then_what_came_with_it() {
    let s = state();
    let before = s.units().to_vec();
    let tables = ProgressTables::of(&s, Words::ENGLISH);
    let mut pb = Progress::with_tables(&promotion_events(), &before, &tables, T).unwrap();
    assert_eq!(
        pb.pages()[0],
        Page::Promotion(PromotionPage {
            unit: UnitId(1),
            name: "Test Lord".into(),
            character: Some("test_lord".into()),
            from: "Exile".into(),
            to: "Blade Heir".into(),
            before: [19, 6, 1, 7, 7, 4, 3],
            gains: gains([6, 3, 0, 4, 5, 2, 2]),
        })
    );
    let Page::Class(cl) = &pb.pages()[1] else {
        panic!("{:?}", pb.pages())
    };
    assert_eq!(class_line(Words::ENGLISH, cl), "Blade Heir");
    assert_eq!((cl.mastered, cl.changed), (false, false));
    assert_eq!(
        (&cl.learned, &cl.stowed),
        (&vec!["Fire".into()], &vec!["Iron Sword".to_owned()])
    );
    assert_eq!(pb.pages().len(), 2);
    // The stats are revealed one at a time, and the page waits.
    assert!(pb.pages()[0].waits());
    assert_eq!(pb.stats_shown(), 0);
    pb.tick(T.level_intro + T.stat_reveal, false);
    assert_eq!(pb.stats_shown(), 2);
    pb.tick(60.0, false);
    assert_eq!((pb.stats_shown(), pb.index()), (7, 0));
    assert!(pb.page_played());
    pb.confirm();
    assert_eq!((pb.stats_shown(), pb.index()), (0, 1));
    // The banner and the classes where a level up has its own.
    let mut pb = Progress::with_tables(&promotion_events(), &before, &tables, T).unwrap();
    pb.tick(60.0, false);
    let Some(ProgressView::Gains(g)) = shown(&pb) else {
        panic!("{:?}", shown(&pb))
    };
    assert_eq!(
        (g.banner.as_str(), g.name.as_str(), g.line.as_str()),
        ("PROMOTED!", "Test Lord", "Exile → Blade Heir")
    );
    let texts = ProgressView::Gains(g.clone()).texts();
    assert!(texts.contains(&"HP   19 → 25  +6".to_owned()), "{texts:?}");
    assert!(texts.contains(&"Mag   1".to_owned()), "{texts:?}");
    assert_eq!(g.stats.len(), 7);
}

#[test]
fn a_reclass_or_a_stowed_item_is_worth_stopping_for() {
    let s = state();
    let before = s.units().to_vec();
    let lord = UnitId(1);
    // Class ids missing from the table are shown as they are.
    let reclassed = [Event::Reclassed {
        unit: lord,
        from: ClassId("exile".into()),
        to: ClassId("gone".into()),
    }];
    let pb = Progress::new(&reclassed, &before, (&s, Words::ENGLISH), T).unwrap();
    let [Page::Class(cl)] = pb.pages() else {
        panic!("{:?}", pb.pages())
    };
    assert_eq!(class_line(Words::ENGLISH, cl), "Exile → gone");
    assert!(cl.changed && cl.learned.is_empty() && cl.stowed.is_empty());
    assert!(pb.pages()[0].waits());
    assert!(pb.pages()[0].len(&T).abs() < f32::EPSILON);
    let stowed = [Event::ItemStowed {
        unit: lord,
        item: trpg_core::ItemId::new("leather_vest"),
    }];
    let pb = Progress::new(&stowed, &before, (&s, Words::ENGLISH), T).unwrap();
    let [Page::Class(cl)] = pb.pages() else {
        panic!("{:?}", pb.pages())
    };
    assert_eq!(
        (class_line(Words::ENGLISH, cl), cl.changed),
        ("Exile".to_owned(), false)
    );
    assert_eq!(cl.stowed, ["Leather Vest"]);
    assert!(pb.pages()[0].waits());
    // Its lines: what was learned, then what went to the stock.
    let page = ClassPage {
        learned: vec!["Fire".into()],
        ..cl.clone()
    };
    let pb = Progress {
        pages: vec![Page::Class(page)],
        page: 0,
        t: 0.0,
        timings: T,
        heard: super::sounds::Heard::default(),
    };
    let Some(ProgressView::Class(v)) = shown(&pb) else {
        panic!("{:?}", shown(&pb))
    };
    assert_eq!(v.lines, ["Learned: Fire", "To stock: Leather Vest"]);
    assert_eq!((v.line.as_str(), v.mastered), ("Exile", None));
    // An item missing from the table has no line.
    let ghost = [Event::ItemStowed {
        unit: lord,
        item: trpg_core::ItemId::new("ghost"),
    }];
    assert_eq!(
        Progress::new(&ghost, &before, (&s, Words::ENGLISH), T),
        None
    );
}

#[test]
fn the_help_line_follows_the_page() {
    let c = ctx();
    let mut pb = progress();
    assert_eq!(help(&c, &pb).plain(), "f skip · hold f fast");
    pb.confirm();
    assert_eq!(help(&c, &pb).plain(), "f continue");
}

#[test]
fn stat_rows_show_the_gain_or_just_the_value() {
    assert_eq!(
        stat_row(Words::ENGLISH, StatKind::Str, 6, 1),
        "Str   6 → 7   +1"
    );
    assert_eq!(
        stat_row(Words::ENGLISH, StatKind::Hp, 19, 2),
        "HP   19 → 21  +2"
    );
    assert_eq!(stat_row(Words::ENGLISH, StatKind::Def, 5, 0), "Def   5");
}

// ---------- the pages' sounds (ticket 0605) ----------

use super::sounds::{CLASS_MASTERED, EXP_FILL, EXP_FULL, LEVEL_UP, LEVEL_UP_STATS, ProgressSound};

/// Plays `pb` to its end at `dt` a frame (Confirm held if `held`), pressing
/// Confirm on each page that waits once it has played; each frame's sounds
/// with the page and its time.
fn listen(pb: &mut Progress, dt: f32, held: bool) -> Vec<(usize, f32, Vec<ProgressSound>)> {
    let mut frames = Vec::new();
    for _ in 0..10_000 {
        let sounds = pb.take_sounds();
        frames.push((pb.index(), pb.time(), sounds));
        if pb.done() {
            return frames;
        }
        if pb.page_played() && pb.page().is_some_and(Page::waits) {
            pb.confirm();
        } else {
            pb.tick(dt, held);
        }
    }
    panic!("never done")
}

fn heard(frames: &[(usize, f32, Vec<ProgressSound>)]) -> Vec<ProgressSound> {
    frames
        .iter()
        .flat_map(|(_, _, s)| s.iter().copied())
        .collect()
}

/// The lord's level up from 90 (HP, Str and Spd grew), then its mastery:
/// each moment's sound once, in order, at its moment.
#[test]
fn a_level_up_plays_each_moments_sound_once_in_order() {
    use ProgressSound::{Play, Start};
    let mut pb = progress();
    let frames = listen(&mut pb, 1.0 / 60.0, false);
    assert_eq!(
        heard(&frames),
        [
            Start(EXP_FILL),
            Play(EXP_FULL),
            Start(LEVEL_UP),
            Play(LEVEL_UP_STATS[0]),
            Play(LEVEL_UP_STATS[1]),
            Play(LEVEL_UP_STATS[2]),
            Play(CLASS_MASTERED),
        ]
    );
    let when = |sound: ProgressSound| {
        frames
            .iter()
            .find(|(_, _, s)| s.contains(&sound))
            .map(|(page, t, _)| (*page, *t))
            .unwrap()
    };
    // 90 + 30: the bar reaches 100 a third of the way through its fill.
    let (page, t) = when(Play(EXP_FULL));
    assert!(page == 0 && (0.2..0.22).contains(&t), "{t}");
    // The first stat waits for the fanfare (Nick, 0605); Spd is the fifth
    // row.
    let (page, t) = when(Play(LEVEL_UP_STATS[0]));
    assert!(
        page == 1 && (T.fanfare..T.fanfare + 0.02).contains(&t),
        "{t}"
    );
    let (_, t) = when(Play(LEVEL_UP_STATS[2]));
    let spd = T.fanfare + 4.0 * T.stat_reveal;
    assert!((spd..spd + 0.02).contains(&t), "{t}");
    assert_eq!(when(Play(CLASS_MASTERED)), (2, 0.0));
}

#[test]
fn an_exp_gain_without_a_level_up_plays_only_the_fill() {
    let s = state();
    let exp = [Event::ExpGained {
        unit: UnitId(1),
        amount: 30,
    }];
    let mut pb = Progress::new(&exp, s.units(), (&s, Words::ENGLISH), T).unwrap();
    let frames = listen(&mut pb, 1.0 / 60.0, false);
    assert_eq!(heard(&frames), [ProgressSound::Start(EXP_FILL)]);
}

/// Confirm finishes a page at once: its long sound stops, and the stat
/// rows it revealed play nothing; the bar's level up still sounds.
#[test]
fn a_confirm_press_skips_the_sounds_forward() {
    use ProgressSound::{Play, Start, Stop};
    let mut pb = progress();
    assert_eq!(pb.take_sounds(), [Start(EXP_FILL)]);
    pb.tick(0.05, false);
    assert_eq!(pb.take_sounds(), []);
    pb.confirm();
    assert_eq!(pb.take_sounds(), [Play(EXP_FULL), Stop]);
    pb.tick(0.0, false);
    assert_eq!(pb.take_sounds(), [Start(LEVEL_UP)]);
    pb.tick(0.3, false);
    pb.confirm();
    assert_eq!(pb.stats_shown(), 7);
    assert_eq!(pb.take_sounds(), [Stop]);
    pb.confirm();
    assert_eq!(pb.take_sounds(), [Play(CLASS_MASTERED)]);
    // Skipped on its first frame: started and stopped at once.
    let mut pb = progress();
    pb.confirm();
    assert_eq!(pb.take_sounds(), [Start(EXP_FILL), Play(EXP_FULL), Stop]);
}

/// Held Confirm: the fill and the fanfare stop where their part of the
/// page ends, and a frame that shows several rows plays one tick.
#[test]
fn held_confirm_cuts_the_long_sounds_and_plays_a_tick_a_frame() {
    use ProgressSound::{Play, Start, Stop};
    let mut pb = progress();
    // 0.1 s frames at ×4: 0.4 s of page each, three rows or more.
    let frames = listen(&mut pb, 0.1, true);
    for (_, _, sounds) in &frames {
        let plays = sounds.iter().filter(|s| matches!(s, Play(_))).count();
        assert!(plays <= 1, "{sounds:?}");
    }
    let all = heard(&frames);
    assert_eq!(all.iter().filter(|s| **s == Stop).count(), 2, "{all:?}");
    let stop_after = |cue| {
        let start = all.iter().position(|s| *s == Start(cue)).unwrap();
        all[start..].iter().position(|s| *s == Stop).unwrap() + start
    };
    assert!(stop_after(EXP_FILL) < stop_after(LEVEL_UP));
    // The fanfare stops in the frame the fanfare's time passes.
    let (_, t, _) = frames
        .iter()
        .find(|(page, _, s)| *page == 1 && s.contains(&Stop))
        .unwrap();
    assert!((T.fanfare..T.fanfare + 0.4).contains(t), "{t}");
    // Played no faster than the frames: fewer ticks than stats grown.
    let ticks = all
        .iter()
        .filter(|s| matches!(s, Play(c) if LEVEL_UP_STATS.contains(c)))
        .count();
    assert!((1..3).contains(&ticks), "{all:?}");
}

/// A promotion has none of the level up's sounds, and its stats come
/// after the short intro.
#[test]
fn a_promotion_plays_nothing_and_doesnt_wait_for_the_fanfare() {
    let p = Page::Promotion(PromotionPage {
        unit: UnitId(1),
        name: "Test Lord".into(),
        character: None,
        from: "Guard".into(),
        to: "Iron Rider".into(),
        before: [0; 7],
        gains: gains([1; 7]),
    });
    assert!((p.intro(&T) - T.level_intro).abs() < f32::EPSILON);
    let mut pb = Progress {
        pages: vec![p],
        page: 0,
        t: 0.0,
        timings: T,
        heard: super::sounds::Heard::default(),
    };
    assert_eq!(heard(&listen(&mut pb, 1.0 / 60.0, false)), []);
}

/// The class box sounds for a mastery or something learned, either alone,
/// and not for a class level alone.
#[test]
fn the_class_box_sounds_for_a_mastery_or_something_learned() {
    let Page::Class(cl) = progress().pages()[2].clone() else {
        panic!()
    };
    let box_sounds = |mastered: bool, learned: Vec<String>| {
        let page = ClassPage {
            mastered,
            learned,
            ..cl.clone()
        };
        let mut pb = Progress {
            pages: vec![Page::Class(page)],
            page: 0,
            t: 0.0,
            timings: T,
            heard: super::sounds::Heard::default(),
        };
        pb.take_sounds()
    };
    let mastered = [ProgressSound::Play(CLASS_MASTERED)];
    assert_eq!(box_sounds(true, vec![]), mastered);
    assert_eq!(box_sounds(false, vec!["Leadership 1".into()]), mastered);
    assert_eq!(box_sounds(false, vec![]), []);
}

/// Only Confirm held while the clock runs hurries a page: a held frame of
/// no time doesn't, and a hurried frame after the fill's end has nothing
/// left to stop.
#[test]
fn only_a_hurried_pass_over_the_fills_end_stops_it() {
    use ProgressSound::{Play, Start};
    let mut pb = progress();
    assert_eq!(pb.take_sounds(), [Start(EXP_FILL)]);
    pb.tick(0.0, true);
    pb.tick(T.exp_fill, false);
    assert_eq!(pb.take_sounds(), [Play(EXP_FULL)]);
    pb.tick(0.1, true);
    assert_eq!(pb.take_sounds(), []);
}
