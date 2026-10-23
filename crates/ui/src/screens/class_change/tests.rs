//! Tests of the class-choice screen, on the debug menu's test knight: a
//! Guard (mastered) with Def 99 of 100, a Rider record at class level 6 and
//! one of every seal. They read the view and the state; where the skin
//! puts things is tested in `glyph::tests`.

use insta::assert_snapshot;
use trpg_core::{ItemId, WeaponKind, WeaponRank};

use super::*;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::glyph_buffer::Cell;
use crate::screen::tests::ctx;
use crate::screens::battle::view::ProgressView;
use crate::widgets::AskView;
use Action::{Cancel, Confirm, CursorLeft, CursorRight};

pub(super) fn demo(c: &Ctx, kind: ChangeKind) -> ClassChangeScreen {
    crate::debug::class_change_demo(c, kind).expect("the test knight")
}

pub(super) fn step(s: &mut ClassChangeScreen, c: &mut Ctx, actions: &[Action]) -> Transition {
    s.update(c, &FrameInput::new(actions.to_vec(), 0.0, vec![]))
}

pub(super) fn render(s: &ClassChangeScreen, c: &Ctx) -> GlyphBuffer {
    let stale = Cell::new(
        'x',
        crate::color::Rgb::new(1, 2, 3),
        crate::color::Rgb::new(1, 2, 3),
    );
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, stale);
    s.draw(c, &mut buf);
    buf
}

/// The columns the screen shows.
pub(super) fn columns(s: &ClassChangeScreen, c: &Ctx) -> ColumnsView {
    match s.view(c).body {
        ClassChangeBody::Columns(columns) => *columns,
        other => panic!("not the columns: {other:?}"),
    }
}

/// The names of the columns on show.
fn names(view: &ColumnsView) -> Vec<&str> {
    view.columns.iter().map(|col| col.name.as_str()).collect()
}

/// A column's stat rows as `(name, before, after, at_max)`.
fn stats(column: &ColumnView) -> Vec<(&str, StatValue, StatValue, bool)> {
    let row = |r| -> (&str, StatValue, StatValue, bool) {
        let r: &StatRowView = r;
        (&r.name, r.before, r.after, r.at_max)
    };
    column.stats.iter().map(row).collect()
}

fn classes(s: &ClassChangeScreen) -> Vec<&str> {
    s.options().iter().map(|o| o.class.0.as_str()).collect()
}

fn item(id: &str) -> ItemId {
    ItemId::new(id)
}

#[test]
fn a_promotion_offers_the_branches_with_what_promote_would_do() {
    let c = ctx();
    let s = demo(&c, ChangeKind::Promote);
    assert_eq!(s.name(), "class_change");
    assert_eq!(classes(&s), ["bulwark", "iron_rider"]);
    assert_eq!(s.focus(), 0);
    // Each preview is `promote` on a copy of the unit and the stock.
    for option in s.options() {
        let (mut unit, mut stock) = (s.unit().clone(), s.stock().clone());
        let made = promote(&mut unit, &option.class, &mut stock, &tables(&c));
        assert!(made.is_ok(), "{made:?}");
        assert_eq!(option.after.as_ref(), Ok(&unit), "{:?}", option.class);
    }
    // The screen's own unit and stock are untouched until a choice is made.
    assert_eq!(s.unit().class, ClassId("guard".into()));
    assert_eq!(s.stock().count(&item("tier_2_seal")), 1);
}

/// The knight: HP 28, Str 11, Mag 0, Dex 7, Spd 5, Def 99, Res 0. Bulwark
/// base `28 11 0 6 4 14 2` less Guard `20 7 0 4 2 9 0`; Iron Rider
/// `27 10 0 6 6 11 2`.
#[test]
fn the_columns_say_each_stat_before_and_after_and_mark_a_maxed_one() {
    let c = ctx();
    let s = demo(&c, ChangeKind::Promote);
    let view = columns(&s, &c);
    assert_eq!(view.title, "Promotion");
    assert_eq!(
        (view.unit.as_str(), view.class_level.as_str()),
        ("Test Knight", "Guard  Lv 12")
    );
    assert_eq!(
        view.seal,
        Some(SealView {
            name: "Tier 2 Seal".into(),
            held: 1
        })
    );
    assert_eq!(view.none, None);
    // Two columns only: no scroll marks.
    assert_eq!((view.more_left, view.more_right), (None, None));
    assert_eq!(
        (view.weapons_heading.as_str(), view.active_heading.as_str()),
        ("Weapons", "Active")
    );
    assert_eq!(view.max_mark, "MAX");
    assert_eq!(view.question, None);
    let [bulwark, rider] = view.columns.as_slice() else {
        panic!("{:?}", view.columns)
    };
    assert_eq!(
        (
            bulwark.focused,
            bulwark.offered,
            rider.focused,
            rider.offered
        ),
        (true, true, false, true)
    );
    assert_eq!(bulwark.name, "Bulwark");
    assert_eq!((bulwark.tier.as_str(), &bulwark.record), ("Tier 2", &None));
    assert_eq!(
        (bulwark.mov.as_str(), bulwark.movement.as_str()),
        ("Mov   4", "armored")
    );
    assert_eq!(bulwark.tags, ["Armored"]);
    assert_eq!(bulwark.weapons, ["Spear    D → C", "Axe      E → D"]);
    assert_eq!(bulwark.active.as_deref(), Some("Fortify"));
    assert_eq!(
        stats(bulwark),
        [
            ("HP", 28, 36, false),
            ("Str", 11, 15, false),
            ("Mag", 0, 0, false),
            ("Dex", 7, 9, false),
            ("Spd", 5, 7, false),
            // Def +5 is cut at the hard ceiling (100).
            ("Def", 99, 100, true),
            ("Res", 0, 2, false),
        ]
    );
    assert_eq!(bulwark.refused, None);
    assert_eq!(rider.name, "Iron Rider");
    assert_eq!(
        (rider.mov.as_str(), rider.movement.as_str()),
        ("Mov   4 → 6", "mounted")
    );
    assert_eq!(rider.tags, ["Mounted", "Armored"]);
    assert_eq!(
        rider.weapons,
        ["Spear    D → C", "Axe      E → D", "Sword    E → D"]
    );
    assert_eq!(rider.active.as_deref(), Some("Trample"));
    assert_eq!(
        stats(rider),
        [
            ("HP", 28, 35, false),
            ("Str", 11, 14, false),
            ("Mag", 0, 0, false),
            ("Dex", 7, 9, false),
            ("Spd", 5, 9, false),
            ("Def", 99, 100, true),
            ("Res", 0, 2, false),
        ]
    );
    let v = s.view(&c);
    assert_eq!(v.help, "Left/Right class · f choose · d back");
    assert_eq!(v.help, s.help(&c));
}

#[test]
fn text_rows() {
    let row = |kind, before, after, ceiling| stat_row(Words::ENGLISH, kind, before, after, ceiling);
    let max = |name: &str, before, after| StatRowView {
        name: name.to_owned(),
        before,
        after,
        at_max: true,
    };
    assert!(!row(StatKind::Str, 9, 12, 50).at_max);
    assert_eq!(row(StatKind::Def, 48, 50, 50), max("Def", 48, 50));
    // Already at the ceiling, or past it.
    assert_eq!(row(StatKind::Res, 50, 50, 50), max("Res", 50, 50));
    assert_eq!(row(StatKind::Hp, 81, 81, 80), max("HP", 81, 81));
    assert!(!row(StatKind::Hp, 70, 79, 80).at_max);
    assert_eq!(mov_line(Words::ENGLISH, 5, 5), "Mov   5");
    assert_eq!(mov_line(Words::ENGLISH, 4, 6), "Mov   4 → 6");
    let c = ctx();
    let mut unit = demo(&c, ChangeKind::Promote).unit().clone();
    unit.weapon_ranks.insert(WeaponKind::Spear, WeaponRank::A);
    let class = |id: &str| c.game.content.classes.get(&ClassId(id.into())).unwrap();
    // A rank at or above the start rank stays.
    assert_eq!(
        weapon_lines(Words::ENGLISH, &unit, class("iron_rider")),
        ["Spear    A", "Axe      E → D", "Sword    E → D"]
    );
    assert_eq!(
        weapon_lines(Words::ENGLISH, &unit, class("archmage")),
        Vec::<String>::new()
    );
}

#[test]
fn choosing_asks_first_then_promotes_and_shows_the_bonus() {
    let mut c = ctx();
    let mut s = demo(&c, ChangeKind::Promote);
    // Left and right move between the columns, wrapping.
    step(&mut s, &mut c, &[CursorRight]);
    assert_eq!(s.focus(), 1);
    let focused = |s: &ClassChangeScreen, c: &Ctx| {
        let view = columns(s, c);
        view.columns
            .iter()
            .map(|col| col.focused)
            .collect::<Vec<_>>()
    };
    assert_eq!(focused(&s, &c), [false, true]);
    step(&mut s, &mut c, &[CursorRight]);
    assert_eq!(s.focus(), 0);
    step(&mut s, &mut c, &[CursorLeft]);
    assert_eq!(s.focus(), 1);
    // Up and down do nothing.
    step(&mut s, &mut c, &[Action::CursorUp, Action::CursorDown]);
    assert_eq!((s.focus(), s.is_confirming()), (1, false));
    // Confirm asks; Cancel backs out, the cursor where it was.
    step(&mut s, &mut c, &[Confirm]);
    assert!(s.is_confirming());
    assert_eq!(s.help(&c), "f yes · d no");
    let view = columns(&s, &c);
    assert_eq!(
        view.question,
        Some(QuestionView {
            ask: AskView {
                text: "Promote Test Knight to Iron Rider?".into(),
                answers: Vec::new(),
                keys: Some("f yes / d no".into()),
            },
            cost: "Uses 1 Tier 2 Seal (1 in stock).".into(),
        })
    );
    // The columns stay under it.
    assert_eq!(names(&view), ["Bulwark", "Iron Rider"]);
    assert_eq!(s.view(&c).help, "f yes · d no");
    // The cursor keys do nothing while it asks.
    step(&mut s, &mut c, &[CursorLeft]);
    assert_eq!((s.focus(), s.is_confirming()), (1, true));
    step(&mut s, &mut c, &[Cancel]);
    assert_eq!((s.focus(), s.is_confirming()), (1, false));
    assert_eq!(columns(&s, &c).question, None);
    assert_eq!(s.unit().class, ClassId("guard".into()));
    // Confirm twice makes the change.
    let before = s.unit().clone();
    let expected = s.options()[1].after.clone();
    let t = step(&mut s, &mut c, &[Confirm, Confirm]);
    assert!(matches!(t, Transition::None));
    assert_eq!(Ok(s.unit()), expected.as_ref());
    assert_eq!(s.unit().class, ClassId("iron_rider".into()));
    assert_eq!((s.unit().level, s.unit().exp), (before.level, before.exp));
    assert_eq!(s.stock().count(&item("tier_2_seal")), 0);
    assert_eq!(s.stock().count(&item("reclass_seal")), 1);
    // Its page: the level-up panel with the bonus, one stat at a time.
    let pages = s.progress().expect("pages").pages().to_vec();
    let [progress::Page::Promotion(page)] = pages.as_slice() else {
        panic!("{pages:?}")
    };
    assert_eq!(
        (page.from.as_str(), page.to.as_str()),
        ("Guard", "Iron Rider")
    );
    assert_eq!(page.before, [28, 11, 0, 7, 5, 99, 0]);
    assert_eq!(page.gains.0, [7, 3, 0, 2, 4, 1, 2]);
    assert_eq!(page.character.as_deref(), Some("test_knight"));
    assert_eq!(s.help(&c), "f skip · hold f fast");
    // The view holds the page on screen, and nothing else.
    let view = s.view(&c);
    let faces = (&c.game.content.portraits, &c.game.lead);
    let page = progress::view(c.words(), faces, s.progress().unwrap());
    assert_eq!(
        view.body,
        ClassChangeBody::Result(page.clone().map(Box::new))
    );
    let Some(ProgressView::Gains(g)) = page else {
        panic!("{page:?}")
    };
    assert_eq!(
        (g.banner.as_str(), g.line.as_str()),
        ("PROMOTED!", "Guard → Iron Rider")
    );
    assert!(g.stats.is_empty());
    assert_eq!(view.help, "f skip · hold f fast");
    // A press reveals every stat, the next closes the page and the screen.
    let t = step(&mut s, &mut c, &[Confirm]);
    assert!(matches!(t, Transition::None));
    assert_eq!(s.help(&c), "f continue");
    assert!(s.progress().is_some_and(Progress::page_played));
    let t = step(&mut s, &mut c, &[Cancel]);
    assert!(matches!(t, Transition::Pop), "{t:?}");
}

#[test]
fn once_the_pages_are_done_the_view_is_empty() {
    let c = ctx();
    let mut s = demo(&c, ChangeKind::Promote);
    s.stage = Stage::Done;
    let view = s.view(&c);
    assert_eq!(view.body, ClassChangeBody::Done);
    assert_eq!(view.help, "");
}

#[test]
fn the_pages_play_by_themselves_and_holding_confirm_speeds_them_up() {
    let mut c = ctx();
    let mut s = demo(&c, ChangeKind::Promote);
    step(&mut s, &mut c, &[Confirm, Confirm]);
    let frame = |dt, held: &[Action]| FrameInput::new(vec![], dt, held.to_vec());
    s.update(&mut c, &frame(0.1, &[]));
    let t = s.progress().map(Progress::time).unwrap_or_default();
    assert!((t - 0.1).abs() < 1e-5, "{t}");
    s.update(&mut c, &frame(0.1, &[Confirm]));
    let t = s.progress().map(Progress::time).unwrap_or_default();
    assert!((t - 0.5).abs() < 1e-5, "{t}");
    // Played to its end, the page still waits for a key.
    let t = s.update(&mut c, &frame(60.0, &[]));
    assert!(matches!(t, Transition::None));
    assert!(s.progress().is_some_and(Progress::page_played));
}

#[test]
fn cancel_closes_the_screen_with_nothing_changed() {
    let mut c = ctx();
    let mut s = demo(&c, ChangeKind::Promote);
    let (unit, stock) = (s.unit().clone(), s.stock().clone());
    let t = step(&mut s, &mut c, &[CursorRight, Cancel]);
    assert!(matches!(t, Transition::Pop));
    assert_eq!((s.unit(), s.stock()), (&unit, &stock));
}

#[test]
fn a_refused_class_says_why_and_cannot_be_chosen() {
    let mut c = ctx();
    let knight = demo(&c, ChangeKind::Promote).unit().clone();
    // No seal in stock.
    let mut s = ClassChangeScreen::new(&c, ChangeKind::Promote, knight.clone(), Stock::default());
    assert_eq!(
        s.options()[0].after,
        Err(ClassChangeError::NoSeal(SealKind::Tier(2)))
    );
    let view = columns(&s, &c);
    let refused = &view.columns[0];
    assert!(!refused.offered);
    assert_eq!(refused.refused.as_deref(), Some("No Tier 2 Seal"));
    // Its stats stay as they are.
    assert_eq!(stats(refused)[0], ("HP", 28, 28, false));
    assert_eq!(stats(refused)[5], ("Def", 99, 99, false));
    assert_eq!(
        view.seal,
        Some(SealView {
            name: "Tier 2 Seal".into(),
            held: 0
        })
    );
    assert_eq!(s.help(&c), "Left/Right class · d back");
    step(&mut s, &mut c, &[Confirm]);
    assert!(!s.is_confirming());
    assert_eq!(columns(&s, &c).question, None);
    // Not mastered.
    let mut fresh = knight;
    fresh
        .class_records
        .insert(ClassId("guard".into()), trpg_core::ClassRecord::UNLOCKED);
    let mut stock = Stock::default();
    stock.add(item("tier_2_seal"));
    let s = ClassChangeScreen::new(&c, ChangeKind::Promote, fresh, stock);
    let view = columns(&s, &c);
    assert!(!view.columns[1].offered);
    assert_eq!(
        view.columns[1].refused.as_deref(),
        Some("Class not mastered")
    );
}

#[test]
fn a_unit_with_nowhere_to_go_sees_an_empty_screen() {
    let mut c = ctx();
    let mut top = demo(&c, ChangeKind::Promote).unit().clone();
    top.class = ClassId("bastion".into());
    let mut s = ClassChangeScreen::new(&c, ChangeKind::Promote, top, Stock::default());
    assert_eq!(s.options().len(), 0);
    let view = columns(&s, &c);
    assert_eq!(view.none.as_deref(), Some("No class to change to."));
    assert!(view.columns.is_empty());
    assert_eq!(
        (view.seal, view.more_left, view.more_right),
        (None, None, None)
    );
    assert_eq!(s.help(&c), "d back");
    let t = step(&mut s, &mut c, &[CursorRight, Confirm]);
    assert!(matches!(t, Transition::None));
    assert!(!s.is_confirming());
    assert!(matches!(step(&mut s, &mut c, &[Cancel]), Transition::Pop));
}

#[test]
fn a_reclass_lists_every_class_the_seal_reaches_and_scrolls() {
    let mut c = ctx();
    let mut s = demo(&c, ChangeKind::Reclass);
    // Tier 1 (but its own Guard), then what the mastered Guard promotes to.
    assert_eq!(
        classes(&s),
        [
            "archer",
            "brawler",
            "cleric",
            "mage",
            "raider",
            "rider",
            "swordsman",
            "bulwark",
            "iron_rider"
        ]
    );
    for option in s.options() {
        let (mut unit, mut stock) = (s.unit().clone(), s.stock().clone());
        let made = reclass(&mut unit, &option.class, &mut stock, &tables(&c));
        assert!(made.is_ok(), "{made:?}");
        assert_eq!(option.after.as_ref(), Ok(&unit), "{:?}", option.class);
    }
    let view = columns(&s, &c);
    assert_eq!(view.title, "Class change");
    assert_eq!(
        view.seal,
        Some(SealView {
            name: "Reclass Seal".into(),
            held: 1
        })
    );
    assert_eq!(names(&view), ["Archer", "Brawler", "Cleric", "Mage"]);
    assert_eq!(view.more_left, None);
    assert_eq!(view.more_right.as_deref(), Some("more →"));
    // A class never entered is `new`; no stat changes, none at its ceiling.
    let archer = &view.columns[0];
    assert_eq!(
        (archer.tier.as_str(), archer.record.as_deref()),
        ("Tier 1", Some("new"))
    );
    assert_eq!(
        (archer.mov.as_str(), archer.movement.as_str()),
        ("Mov   4 → 5", "foot")
    );
    assert_eq!(stats(archer)[0], ("HP", 28, 28, false));
    assert_eq!(stats(archer)[5], ("Def", 99, 99, false));
    // Moving past the last column shown scrolls.
    for _ in 0..5 {
        step(&mut s, &mut c, &[CursorRight]);
    }
    assert_eq!(s.focus(), 5);
    let view = columns(&s, &c);
    assert_eq!(names(&view), ["Cleric", "Mage", "Raider", "Rider"]);
    assert_eq!(view.more_left.as_deref(), Some("← more"));
    assert_eq!(view.more_right.as_deref(), Some("more →"));
    // The Rider's saved class level; the cursor is on it.
    let rider = &view.columns[3];
    assert_eq!(
        (rider.tier.as_str(), rider.record.as_deref()),
        ("Tier 1", Some("CL 6"))
    );
    let focused: Vec<bool> = view.columns.iter().map(|col| col.focused).collect();
    assert_eq!(focused, [false, false, false, true]);
    // One more: both marks still show, with two columns left to the right.
    step(&mut s, &mut c, &[CursorRight]);
    let view = columns(&s, &c);
    assert_eq!(names(&view), ["Mage", "Raider", "Rider", "Swordsman"]);
    assert!(view.more_left.is_some() && view.more_right.is_some());
    // Wrapping to the first column scrolls back; and to the last.
    for _ in 0..3 {
        step(&mut s, &mut c, &[CursorRight]);
    }
    assert_eq!(s.focus(), 0);
    assert_eq!(names(&columns(&s, &c))[0], "Archer");
    step(&mut s, &mut c, &[CursorLeft]);
    let view = columns(&s, &c);
    assert_eq!(
        names(&view),
        ["Rider", "Swordsman", "Bulwark", "Iron Rider"]
    );
    assert_eq!(view.more_left.as_deref(), Some("← more"));
    assert_eq!(view.more_right, None);
    // No bonus by this road: a reclass never changes stats.
    let rider = &view.columns[3];
    assert_eq!(stats(rider)[0], ("HP", 28, 28, false));
    assert_eq!(
        (rider.tier.as_str(), rider.record.as_deref()),
        ("Tier 2", Some("new"))
    );
}

#[test]
fn a_reclass_keeps_stats_and_shows_what_was_learned_and_stowed() {
    let mut c = ctx();
    let mut s = demo(&c, ChangeKind::Reclass);
    let stats = s.unit().stats;
    step(&mut s, &mut c, &[CursorRight, CursorRight, CursorRight]);
    step(&mut s, &mut c, &[Confirm]);
    let question = columns(&s, &c).question.expect("the question");
    assert_eq!(question.ask.text, "Change Test Knight to Mage?");
    assert_eq!(question.cost, "Uses 1 Reclass Seal (1 in stock).");
    step(&mut s, &mut c, &[Confirm]);
    assert_eq!(s.unit().class, ClassId("mage".into()));
    // Only Mov follows the class.
    let mut expected = stats;
    expected.mov = 5;
    assert_eq!(s.unit().stats, expected);
    assert_eq!(s.stock().count(&item("reclass_seal")), 0);
    assert_eq!(s.stock().count(&item("tier_2_seal")), 1);
    // The Mage wears light armour only: the chain mail went to the stock.
    assert_eq!(s.stock().count(&item("chain_mail")), 1);
    let pages = s.progress().expect("pages").pages().to_vec();
    let [progress::Page::Class(page)] = pages.as_slice() else {
        panic!("{pages:?}")
    };
    assert_eq!(page.class, "Guard → Mage");
    assert!(page.changed);
    assert_eq!(page.learned, ["Fire"]);
    assert_eq!(page.stowed, ["Chain Mail"]);
    assert!(matches!(s.view(&c).body, ClassChangeBody::Result(_)));
    // The page waits for a key, then the screen closes.
    let idle = s.update(&mut c, &FrameInput::new(vec![], 60.0, vec![]));
    assert!(matches!(idle, Transition::None));
    assert!(matches!(step(&mut s, &mut c, &[Confirm]), Transition::Pop));
}

#[test]
fn sounds() {
    let mut c = ctx();
    let mut s = demo(&c, ChangeKind::Promote);
    let heard = |s: &mut ClassChangeScreen, c: &mut Ctx, action| {
        step(s, c, &[action]);
        let sounds: Vec<String> = c
            .audio
            .take()
            .into_iter()
            .map(|r| format!("{r:?}"))
            .collect();
        sounds.join(",")
    };
    let moved = heard(&mut s, &mut c, CursorRight);
    assert!(moved.contains("menu_move"), "{moved}");
    assert_eq!(heard(&mut s, &mut c, Action::CursorUp), "");
    let chosen = heard(&mut s, &mut c, Confirm);
    assert!(chosen.contains("menu_select"), "{chosen}");
    let back = heard(&mut s, &mut c, Cancel);
    assert!(back.contains("menu_cancel"), "{back}");
    heard(&mut s, &mut c, Confirm);
    let made = heard(&mut s, &mut c, Confirm);
    assert!(made.contains("menu_select"), "{made}");
    // A single column has nowhere to move to; a refused class is denied.
    let knight = demo(&c, ChangeKind::Promote).unit().clone();
    let mut none = ClassChangeScreen::new(&c, ChangeKind::Promote, knight, Stock::default());
    let denied = heard(&mut none, &mut c, Confirm);
    assert!(denied.contains("menu_cancel"), "{denied}");
    let mut lancer = demo(&c, ChangeKind::Promote).unit().clone();
    lancer.class = ClassId("duelist".into());
    let mut one = ClassChangeScreen::new(&c, ChangeKind::Promote, lancer, Stock::default());
    assert_eq!(one.options().len(), 1);
    // Nothing to pick between, and nothing to choose (no seal).
    assert_eq!(one.help(&c), "d back");
    assert_eq!(heard(&mut one, &mut c, CursorRight), "");
    assert_eq!(one.focus(), 0);
    let closed = heard(&mut one, &mut c, Cancel);
    assert!(closed.contains("menu_cancel"), "{closed}");
}

/// The tests (and the game flow) reach the screen through `as_any`.
#[test]
fn opts_into_as_any() {
    let c = ctx();
    let s = demo(&c, ChangeKind::Promote);
    let any = Screen::as_any(&s).expect("opts into as_any");
    let found = any.downcast_ref::<ClassChangeScreen>().expect("the screen");
    assert_eq!(found.view(&c), s.view(&c));
}

#[test]
fn the_demo_needs_its_test_character() {
    let mut c = ctx();
    c.game.content.characters.characters.clear();
    assert!(crate::debug::class_change_demo(&c, ChangeKind::Promote).is_none());
}

#[test]
fn promotion_choice_snapshot() {
    let c = ctx();
    let s = demo(&c, ChangeKind::Promote);
    assert_snapshot!(render(&s, &c).to_snapshot(&c.palette));
}

#[test]
fn promotion_question_snapshot() {
    let mut c = ctx();
    let mut s = demo(&c, ChangeKind::Promote);
    step(&mut s, &mut c, &[CursorRight, Confirm]);
    assert_snapshot!(render(&s, &c).to_snapshot(&c.palette));
}

#[test]
fn promotion_result_snapshot() {
    let mut c = ctx();
    let mut s = demo(&c, ChangeKind::Promote);
    step(&mut s, &mut c, &[Confirm, Confirm, Confirm]);
    assert_snapshot!(render(&s, &c).to_snapshot(&c.palette));
}

#[test]
fn reclass_choice_snapshot() {
    let mut c = ctx();
    let mut s = demo(&c, ChangeKind::Reclass);
    for _ in 0..5 {
        step(&mut s, &mut c, &[CursorRight]);
    }
    assert_snapshot!(render(&s, &c).to_snapshot(&c.palette));
}

#[test]
fn reclass_result_snapshot() {
    let mut c = ctx();
    let mut s = demo(&c, ChangeKind::Reclass);
    step(&mut s, &mut c, &[CursorRight, CursorRight, CursorRight]);
    step(&mut s, &mut c, &[Confirm, Confirm]);
    assert_snapshot!(render(&s, &c).to_snapshot(&c.palette));
}
