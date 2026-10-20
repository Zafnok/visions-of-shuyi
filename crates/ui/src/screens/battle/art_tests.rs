//! Tests of Combat Arts in the attack flow (tickets 0414 and 0442): the
//! arts menu (arts and combat actives, dimmed when they can't be used), the
//! forecast with the chosen line and without a list, debuffs on
//! the info screen, and the playback banner for a boss's or a green unit's
//! art. The numbers are always `core`'s.

use crate::widgets::menu::LabelText;
use insta::assert_snapshot;
use trpg_core::{
    ArtId, BattleState, ClassId, Command, EffectSource, Equipped, Event, Faction, Objective, Pos,
    SkillId, Unit, UnitId,
};

use super::BattleScreen;
use super::art_list::{Technique, art_lines, arts_menu, playback_banner};
use super::attack::reach;
use super::mode::{ListKind, MenuEntry, Mode};
use super::playback::{Playback, TIMINGS, box_view};
use super::testing::{battle_with, quick_units};
use super::view::{ForecastView, InfoView, SkillLineView};
use crate::FrameInput;
use crate::color::Rgb;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::glyph_buffer::{Cell, GlyphBuffer};
use crate::input::Action;
use crate::screen::tests::ctx;
use crate::screen::{Ctx, Screen};
use crate::widgets::menu::SuffixKind;
use crate::words::Words;

fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

fn step(s: &mut BattleScreen, c: &mut Ctx, actions: &[Action]) {
    s.update(c, &FrameInput::new(actions.to_vec(), 0.0, vec![]));
}

fn wait(s: &mut BattleScreen, c: &mut Ctx, seconds: f32) {
    s.update(c, &FrameInput::new(vec![], seconds, vec![]));
}

fn render(s: &BattleScreen, c: &Ctx) -> GlyphBuffer {
    let stale = Cell::new('x', Rgb::new(1, 2, 3), Rgb::new(1, 2, 3));
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, stale);
    s.draw(c, &mut buf);
    buf
}

/// Cells `x..x + w` of row `y`, trimmed.
fn text(buf: &GlyphBuffer, x: i32, y: i32, w: i32) -> String {
    (x..x + w)
        .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
        .collect::<String>()
        .trim()
        .to_owned()
}

/// Every row of the screen, joined.
fn screen_text(buf: &GlyphBuffer) -> String {
    (0..i32::from(ConsoleSpec::PC.rows))
        .map(|y| text(buf, 0, y, i32::from(ConsoleSpec::PC.cols)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The Quick Battle's units on a rout battle, `edit` applied to them first:
/// the lord at (6, 2) beside the raider (7, 1) and the brigand (8, 2), the
/// archer two tiles below the brigand at (8, 4).
fn battle(c: &Ctx, edit: impl FnOnce(&mut Vec<Unit>)) -> BattleState {
    let (map, mut units) = quick_units(c);
    units[0].pos = p(6, 2);
    units[2].pos = p(8, 4);
    edit(&mut units);
    battle_with(c, map, units, Objective::Rout { turn_limit: None })
}

/// The lord as a Swordsman (Sword D: Flowing Cut and Guard Break), its
/// iron sword at `durability`.
fn swordsman(c: &Ctx, durability: u32) -> BattleState {
    battle(c, |units| {
        units[0].class = ClassId("swordsman".to_owned());
        units[0].loadout.weapons[0]
            .as_mut()
            .unwrap()
            .durability_left = durability;
    })
}

/// The targeting state of `s`.
fn targeting(s: &BattleScreen) -> &super::attack::Targeting {
    match s.mode() {
        Mode::Targeting(t) => t,
        m => panic!("{m:?}"),
    }
}

/// Chooses `entry` in the action menu that is open.
fn choose_entry(s: &mut BattleScreen, c: &mut Ctx, entry: MenuEntry) {
    let Mode::ActionMenu { menu, entries, .. } = s.mode() else {
        panic!("{:?}", s.mode());
    };
    let to = entries.iter().position(|&e| e == entry).unwrap();
    let from = menu.focus();
    let key = if to > from {
        Action::CursorDown
    } else {
        Action::CursorUp
    };
    for _ in 0..to.abs_diff(from) {
        step(s, c, &[key]);
    }
    step(s, c, &[Action::Confirm]);
}

/// The lord moved to (7, 2), its action menu open.
fn lord_menu(c: &mut Ctx, state: BattleState) -> BattleScreen {
    let mut s = BattleScreen::new(state);
    step(
        &mut s,
        c,
        &[Action::Confirm, Action::CursorRight, Action::Confirm],
    );
    wait(&mut s, c, 0.5);
    s
}

/// The archer where it stands, two tiles below the brigand, its action menu
/// open.
fn archer_menu(c: &mut Ctx, state: BattleState) -> BattleScreen {
    let mut s = BattleScreen::new(state);
    s.cursor.jump(p(8, 4));
    step(&mut s, c, &[Action::Confirm, Action::Confirm]);
    s
}

/// Cycles the forecast's targets until `target` is under the cursor.
fn aim(s: &mut BattleScreen, c: &mut Ctx, target: UnitId) {
    for _ in 0..4 {
        if targeting(s).target() == target {
            return;
        }
        step(s, c, &[Action::CursorDown]);
    }
    panic!("{target:?} isn't a target: {:?}", targeting(s).targets);
}

/// From the action menu: `Arts`, the line named `name`, then `target`.
fn use_art(s: &mut BattleScreen, c: &mut Ctx, name: &str, target: UnitId) {
    choose_entry(s, c, MenuEntry::Arts);
    let at = lines(s).iter().position(|l| l.0 == name);
    let at = at.unwrap_or_else(|| panic!("{name} isn't listed: {:?}", lines(s)));
    // Down to the line (it opened on the first one that can be chosen).
    for _ in 0..at {
        let Mode::List {
            menu,
            kind: ListKind::Arts(_),
            ..
        } = s.mode()
        else {
            unreachable!()
        };
        if menu.focus() == at {
            break;
        }
        step(s, c, &[Action::CursorDown]);
    }
    step(s, c, &[Action::Confirm]);
    aim(s, c, target);
}

/// The lord moved to (7, 2), attacking the brigand (unit 4) with the art or
/// active named `name` and its iron sword.
fn lord_on_brigand(c: &mut Ctx, state: BattleState, name: &str) -> BattleScreen {
    let mut s = lord_menu(c, state);
    use_art(&mut s, c, name, UnitId(4));
    s
}

/// The archer attacking the brigand two tiles up from where it stands, with
/// the art or active named `name`.
fn archer_on_brigand(c: &mut Ctx, state: BattleState, name: &str) -> BattleScreen {
    let mut s = archer_menu(c, state);
    use_art(&mut s, c, name, UnitId(4));
    s
}

/// The lines of the arts menu that is open: name as shown, enabled, dimmed
/// reason.
fn lines(s: &BattleScreen) -> Vec<(String, bool, Option<String>)> {
    let Mode::List {
        menu,
        kind: ListKind::Arts(_),
        ..
    } = s.mode()
    else {
        panic!("{:?}", s.mode());
    };
    shown(menu)
}

/// The lines of `menu`: name as shown, enabled, dimmed reason.
fn shown(menu: &crate::widgets::menu::Menu) -> Vec<(String, bool, Option<String>)> {
    menu.items()
        .iter()
        .map(|i| {
            let name = i.label.as_str().split("  ").next().unwrap_or("").to_owned();
            (
                name,
                i.enabled,
                i.suffix.clone().map(|(t, _)| t.trim().to_owned()),
            )
        })
        .collect()
}

/// The durability left on `unit`'s weapon in slot 0.
fn durability(state: &BattleState, unit: UnitId) -> u32 {
    let u = state.unit(unit).unwrap();
    u.loadout.weapon(0).unwrap().durability_left
}

/// The info screen of unit `id` of `state`, rendered.
fn info(c: &mut Ctx, state: &BattleState, id: UnitId) -> InfoView {
    let mut s = BattleScreen::new(state.clone());
    s.cursor.jump(state.unit(id).unwrap().pos);
    step(&mut s, c, &[Action::Info]);
    assert!(matches!(s.mode(), Mode::Info { .. }), "{:?}", s.mode());
    s.view(c).info.expect("the info screen")
}

/// The forecast on screen.
fn forecast_of(s: &BattleScreen, c: &Ctx) -> ForecastView {
    s.view(c).forecast.expect("a forecast")
}

/// The forecast's art or active line: its name and the durability before
/// and after.
fn skill_line(name: &str, from: u32, to: u32) -> SkillLineView {
    SkillLineView {
        name: name.to_owned(),
        durability: Some((from, to)),
    }
}

#[test]
fn guard_break_shows_cores_forecast_and_is_paid_when_attacking() {
    let mut c = ctx();
    let state = swordsman(&c, 20);
    let art = Technique::Art(ArtId::new("guard_break"));
    let expected = state
        .preview_attack(
            UnitId(1),
            p(7, 2),
            &art.action(UnitId(4), &Equipped::Weapon(0)),
        )
        .unwrap();
    let mut s = lord_on_brigand(&mut c, state, "Guard Break");
    assert_eq!(targeting(&s).preview, expected);
    let shown = forecast_of(&s, &c);
    assert_eq!(shown.skill, Some(skill_line("Guard Break", 20, 16)));
    assert_eq!(shown.sides[0].hit, Some(100));
    assert_eq!(expected.forecast.attacker.hit, 100);
    assert_eq!(shown.no_counter.as_deref(), Some("no counter"));
    assert_eq!(shown.sides[1].hit, None);
    // The art was chosen in the arts menu: the forecast has no list, left
    // and right swap between the two swords and up and down pick the target.
    let help = s.help(&c);
    assert!(!help.contains("art"), "{help}");
    assert!(help.contains("Left/Right swap"), "{help}");
    assert!(help.contains("Up/Down target"), "{help}");
    // Attack: the playback, then the weapon has paid 4.
    step(&mut s, &mut c, &[Action::Confirm]);
    for _ in 0..40 {
        wait(&mut s, &mut c, 0.5);
        step(&mut s, &mut c, &[Action::Confirm]);
    }
    assert_eq!(durability(s.state(), UnitId(1)), 16);
    let after = s.state().clone();
    let sword = info(&mut c, &after, UnitId(1)).weapons[0].clone().unwrap();
    assert_eq!((sword.left, sword.max, sword.broken), (16, 20, None));
}

#[test]
fn guard_break_with_three_left_goes_through_and_breaks_the_sword() {
    // Nick (0414 review): "it has 3 dur left and the art costs 5, ok, art
    // goes thru, then weapon is broken".
    let mut c = ctx();
    let state = swordsman(&c, 3);
    let mut s = lord_menu(&mut c, state);
    choose_entry(&mut s, &mut c, MenuEntry::Arts);
    let shown = lines(&s);
    let gb = shown.iter().find(|l| l.0 == "Guard Break").unwrap();
    assert_eq!((gb.1, gb.2.as_deref()), (true, None));
    step(&mut s, &mut c, &[Action::Cancel]);
    use_art(&mut s, &mut c, "Guard Break", UnitId(4));
    let shown = forecast_of(&s, &c);
    assert_eq!(shown.skill, Some(skill_line("Guard Break", 3, 0)));
    // Fought with the unbroken sword: the same hit as at 20/20.
    assert_eq!(shown.sides[0].hit, Some(100));
    step(&mut s, &mut c, &[Action::Confirm]);
    for _ in 0..40 {
        wait(&mut s, &mut c, 0.5);
        step(&mut s, &mut c, &[Action::Confirm]);
    }
    assert_eq!(durability(s.state(), UnitId(1)), 0);
    let after = s.state().clone();
    let sword = info(&mut c, &after, UnitId(1)).weapons[0].clone().unwrap();
    assert_eq!(sword.broken.as_deref(), Some("broken"));
}

/// The lord as a Swordsman with only its iron sword, broken.
fn broken_sword(c: &Ctx) -> BattleState {
    battle(c, |units| {
        units[0].class = ClassId("swordsman".to_owned());
        units[0].loadout.weapons[0]
            .as_mut()
            .unwrap()
            .durability_left = 0;
        units[0].loadout.weapons[1] = None;
    })
}

#[test]
fn a_broken_weapon_dims_every_art_with_the_reason() {
    let mut c = ctx();
    let state = broken_sword(&c);
    let found = art_lines(&state, UnitId(1), p(7, 2));
    assert!(
        found.iter().all(|l| !l.usable() && l.slot == 0),
        "{found:?}"
    );
    let menu = arts_menu(&state, Words::ENGLISH, &found);
    let names: Vec<String> = shown(&menu).into_iter().map(|l| l.0).collect();
    // The Swordsman's arts and its own Keen Edge.
    assert_eq!(names, ["Flowing Cut", "Guard Break", "Keen Edge"]);
    for line in shown(&menu) {
        assert_eq!(
            (line.1, line.2.as_deref()),
            (false, Some("broken")),
            "{line:?}"
        );
    }
    // Nothing to choose: `Arts` is listed, dimmed. The broken sword still
    // attacks plainly.
    let s = lord_menu(&mut c, state);
    let Mode::ActionMenu { menu, entries, .. } = s.mode() else {
        panic!("{:?}", s.mode());
    };
    let enabled = |e| menu.items()[entries.iter().position(|&x| x == e).unwrap()].enabled;
    assert!(enabled(MenuEntry::Attack));
    assert!(!enabled(MenuEntry::Arts));
    // With a second sword that isn't broken, the arts are used with that.
    let state = swordsman(&c, 0);
    let mut s = lord_menu(&mut c, state);
    choose_entry(&mut s, &mut c, MenuEntry::Arts);
    assert!(lines(&s).iter().all(|l| l.1 && l.2.is_none()));
    assert!(screen_text(&render(&s, &c)).contains("Steel Sword"));
    step(&mut s, &mut c, &[Action::Confirm]);
    let t = targeting(&s);
    assert_eq!((&t.with, t.can_swap()), (&Equipped::Weapon(1), false));
}

/// Ticket 0442: a dimmed line shows its reason in the menu and can't be
/// chosen.
#[test]
fn a_dimmed_line_shows_its_reason_and_effect_and_cant_be_chosen() {
    let mut c = ctx();
    // Only the brigand beside the archer is left near it: Close Shot
    // reaches it; Pinning Shot and Vault (the bow's own range) reach
    // nobody.
    let state = battle(&c, |units| units[3].pos = p(12, 0));
    let mut s = archer_menu(&mut c, state);
    choose_entry(&mut s, &mut c, MenuEntry::Arts);
    assert_eq!(
        lines(&s),
        [
            ("Close Shot".to_owned(), true, None),
            (
                "Pinning Shot".to_owned(),
                false,
                Some("out of range".to_owned())
            ),
            ("Vault".to_owned(), false, Some("out of range".to_owned())),
        ]
    );
    let all = screen_text(&render(&s, &c));
    assert!(all.contains("Pinning Shot"), "{all}");
    assert!(all.contains("out of range"), "{all}");
    // Close Shot's effect shows under the list (ticket 0429).
    let all = screen_text(&render(&s, &c));
    assert!(all.contains("-10 hit; min range 1"), "{all}");
    // Down stops on the dimmed line, which shows its effect too, but
    // Confirm on it does nothing.
    step(&mut s, &mut c, &[Action::CursorDown]);
    let all = screen_text(&render(&s, &c));
    assert!(all.contains("pins: Mov −3"), "{all}");
    assert!(!all.contains("min range 1"), "{all}");
    step(&mut s, &mut c, &[Action::Confirm]);
    assert!(
        matches!(
            s.mode(),
            Mode::List {
                kind: ListKind::Arts(_),
                ..
            }
        ),
        "{:?}",
        s.mode()
    );
    step(&mut s, &mut c, &[Action::CursorDown]);
    let all = screen_text(&render(&s, &c));
    assert!(all.contains("Bow: move 1 after"), "{all}");
    // Back up to Close Shot: Confirm uses it.
    step(
        &mut s,
        &mut c,
        &[Action::CursorUp, Action::CursorUp, Action::Confirm],
    );
    let t = targeting(&s);
    assert_eq!(t.technique, Technique::Art(ArtId::new("close_shot")));
    assert_eq!(t.targets, [UnitId(5)]);
}

#[test]
fn lines_are_arts_then_actives_and_only_what_fits() {
    let mut c = ctx();
    let state = battle(&c, |units| {
        assert!(units[0].learn_skill(&SkillId::new("keen_edge"), &c.game.content.skills));
    });
    let mut s = lord_menu(&mut c, state);
    choose_entry(&mut s, &mut c, MenuEntry::Arts);
    let names: Vec<String> = lines(&s).into_iter().map(|l| l.0).collect();
    // No `Attack` line, no bow or axe arts, and not the lord's Inspire (not
    // a combat active).
    assert_eq!(names, ["Flowing Cut", "Guard Break", "Keen Edge"]);
    let Mode::List {
        menu,
        kind: ListKind::Arts(_),
        ..
    } = s.mode()
    else {
        unreachable!()
    };
    let labels: Vec<&str> = menu
        .items()
        .iter()
        .map(|i| i.label.as_str().trim_end())
        .collect();
    assert_eq!(
        labels,
        [
            "Flowing Cut     −2 dur  E",
            "Guard Break     −4 dur  D",
            "Keen Edge       −3 dur  active",
        ]
    );
    // The weapon the focused line is used with, in the menu's border.
    assert!(screen_text(&render(&s, &c)).contains("Iron Sword 20/20"));
    // The archer: bow arts, and Vault as its active.
    let state = battle(&c, |units| {
        assert!(units[2].learn_skill(&SkillId::new("vault"), &c.game.content.skills));
    });
    let mut s = archer_menu(&mut c, state);
    choose_entry(&mut s, &mut c, MenuEntry::Arts);
    let names: Vec<String> = lines(&s).into_iter().map(|l| l.0).collect();
    assert_eq!(names, ["Close Shot", "Pinning Shot", "Vault"]);
}

#[test]
fn a_chosen_active_is_used_and_paid_for() {
    let mut c = ctx();
    let state = battle(&c, |units| {
        assert!(units[0].learn_skill(&SkillId::new("keen_edge"), &c.game.content.skills));
    });
    let mut s = lord_on_brigand(&mut c, state, "Keen Edge");
    assert_eq!(
        forecast_of(&s, &c).skill,
        Some(skill_line("Keen Edge", 20, 17))
    );
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(durability(s.state(), UnitId(1)), 17);
}

#[test]
fn the_chosen_art_stays_across_targets() {
    let mut c = ctx();
    let state = swordsman(&c, 20);
    let mut s = lord_on_brigand(&mut c, state, "Guard Break");
    // To the raider: still Guard Break, the forecast against the raider.
    step(&mut s, &mut c, &[Action::CursorDown]);
    let t = targeting(&s);
    assert_eq!(t.target(), UnitId(6));
    assert_eq!(t.technique, Technique::Art(ArtId::new("guard_break")));
    let art = Technique::Art(ArtId::new("guard_break"));
    let expected = s
        .state()
        .preview_attack(
            UnitId(1),
            p(7, 2),
            &art.action(UnitId(6), &Equipped::Weapon(0)),
        )
        .unwrap();
    assert_eq!(t.preview, expected);
}

/// Ticket 0442: the forecast shows no arts list, and up and down pick the
/// target.
#[test]
fn the_forecast_has_no_arts_list_and_up_down_pick_targets() {
    let mut c = ctx();
    let state = swordsman(&c, 20);
    let mut s = lord_menu(&mut c, state);
    choose_entry(&mut s, &mut c, MenuEntry::Attack);
    assert_eq!(targeting(&s).technique, Technique::Attack);
    let all = screen_text(&render(&s, &c));
    for gone in ["Flowing Cut", "Guard Break", "dur"] {
        assert!(!all.contains(gone), "{gone}: {all}");
    }
    assert!(!s.help(&c).contains("art"), "{}", s.help(&c));
    let first = targeting(&s).target();
    step(&mut s, &mut c, &[Action::CursorDown]);
    assert_ne!(targeting(&s).target(), first);
    assert_eq!(targeting(&s).technique, Technique::Attack);
    step(&mut s, &mut c, &[Action::CursorUp]);
    assert_eq!(targeting(&s).target(), first);
}

/// The lord's (Swordsman) arts menu beside it, on Guard Break.
#[test]
fn arts_menu_snapshot() {
    let mut c = ctx();
    let state = swordsman(&c, 20);
    let mut s = lord_menu(&mut c, state);
    choose_entry(&mut s, &mut c, MenuEntry::Arts);
    step(&mut s, &mut c, &[Action::CursorDown]);
    assert_snapshot!(render(&s, &c).to_snapshot(&c.palette));
}

/// The lord's arts menu in the left-handed layout (0208).
#[test]
fn arts_menu_left_handed_snapshot() {
    let mut c = ctx();
    c.use_layout(crate::input::LEFT_HANDED);
    let state = swordsman(&c, 20);
    let mut s = lord_menu(&mut c, state);
    choose_entry(&mut s, &mut c, MenuEntry::Arts);
    step(&mut s, &mut c, &[Action::CursorDown]);
    assert_snapshot!(render(&s, &c).to_snapshot(&c.palette));
}

/// The archer's arts menu, on Close Shot, its other lines dimmed.
#[test]
fn archer_arts_menu_snapshot() {
    let mut c = ctx();
    let state = battle(&c, |units| units[3].pos = p(12, 0));
    let mut s = archer_menu(&mut c, state);
    choose_entry(&mut s, &mut c, MenuEntry::Arts);
    assert_snapshot!(render(&s, &c).to_snapshot(&c.palette));
}

/// What the focused line of the arts menu says it does (ticket 0429).
fn focused_note(s: &BattleScreen) -> Option<String> {
    let Mode::List {
        menu,
        kind: ListKind::Arts(_),
        ..
    } = s.mode()
    else {
        panic!("{:?}", s.mode());
    };
    menu.view().focused().and_then(|i| i.note.clone())
}

/// Moving down the arts menu changes the effect under it to the next
/// line's.
#[test]
fn the_arts_menu_describes_the_focused_line() {
    let mut c = ctx();
    let state = battle(&c, |units| {
        units[0].class = ClassId("swordsman".to_owned());
        assert!(units[0].learn_skill(&SkillId::new("keen_edge"), &c.game.content.skills));
    });
    let mut s = lord_menu(&mut c, state);
    choose_entry(&mut s, &mut c, MenuEntry::Arts);
    let mut seen = vec![];
    for _ in 0..3 {
        seen.push(focused_note(&s).unwrap());
        step(&mut s, &mut c, &[Action::CursorDown]);
    }
    assert_eq!(
        seen,
        [
            "follow-up ×3/2",
            "+10 hit; no counter",
            "Sword: +30 hit +10 crit"
        ]
    );
}

/// The lord (Swordsman) with Guard Break chosen: the art's line and `no
/// counter`, and no list.
#[test]
fn forecast_with_an_art_snapshot() {
    let mut c = ctx();
    let state = swordsman(&c, 20);
    let s = lord_on_brigand(&mut c, state, "Guard Break");
    assert_snapshot!(render(&s, &c).to_snapshot(&c.palette));
}

/// The archer with Pinning Shot: a note-only art.
#[test]
fn forecast_with_pinning_shot_snapshot() {
    let mut c = ctx();
    let state = battle(&c, |_| {});
    let s = archer_on_brigand(&mut c, state, "Pinning Shot");
    let shown = forecast_of(&s, &c);
    assert_eq!(shown.skill, Some(skill_line("Pinning Shot", 20, 17)));
    assert_eq!(shown.notes, ["pins: Mov −3"]);
    assert_snapshot!(render(&s, &c).to_snapshot(&c.palette));
}

/// The lord's only sword broken: every line of the arts menu dimmed with
/// `broken`, and `(broken)` in the forecast of a plain attack.
#[test]
fn broken_weapon_lines_snapshot() {
    let mut c = ctx();
    let state = broken_sword(&c);
    let mut s = lord_menu(&mut c, state);
    // `Arts` is dimmed, so the menu is shown as it would be drawn.
    let Mode::ActionMenu { sel, .. } = s.mode().clone() else {
        panic!("{:?}", s.mode());
    };
    let lines = art_lines(s.state(), UnitId(1), p(7, 2));
    let menu = arts_menu(s.state(), Words::ENGLISH, &lines);
    let (text, color) = menu.items()[0].suffix.clone().unwrap();
    assert_eq!((text.trim(), color), ("broken", SuffixKind::Warning));
    let action_menu = std::mem::replace(
        &mut s.mode,
        Mode::List {
            sel,
            menu,
            kind: ListKind::Arts(lines),
        },
    );
    let list = render(&s, &c).to_snapshot(&c.palette);
    s.mode = action_menu;
    choose_entry(&mut s, &mut c, MenuEntry::Attack);
    aim(&mut s, &mut c, UnitId(4));
    let forecast = render(&s, &c).to_snapshot(&c.palette);
    assert_snapshot!(format!("{list}\n=== forecast ===\n{forecast}"));
}

#[test]
fn a_pin_shows_on_the_info_screen_with_when_it_ends() {
    let mut c = ctx();
    let plain = battle(&c, |_| {});
    // The archer's Pinning Shot hits the brigand (seed 0), which lives.
    let mut pinned = plain.clone();
    let art = Technique::Art(ArtId::new("pinning_shot"));
    let cmd = Command::Act {
        unit: UnitId(3),
        dest: p(8, 4),
        action: art.action(UnitId(4), &Equipped::Weapon(0)),
    };
    pinned.apply(&cmd).unwrap();
    let brigand = pinned.unit(UnitId(4)).unwrap();
    let [effect] = brigand.effects.as_slice() else {
        panic!("{:?}", brigand.effects);
    };
    assert_eq!(effect.source, EffectSource::Art(ArtId::new("pinning_shot")));
    // Until the end of the Enemy phase: the start of the next one.
    let until = format!("until {:?} phase", effect.until);
    let all = info(&mut c, &pinned, UnitId(4)).texts().join("\n");
    for part in ["Effects", "Pinning Shot", "Mov -3", until.as_str()] {
        assert!(all.contains(part), "{part}: {all}");
    }
    assert!(all.contains("Mov 2"), "{all}");
    // On the map, the brigand at (8, 2) is shown under an effect (the glyph
    // skin puts the effect colour behind its label).
    let under_effect = |state: &BattleState, c: &Ctx| {
        let scene = BattleScreen::new(state.clone()).scene(c);
        let brigand = scene.unit_at(p(8, 2)).unwrap();
        assert_eq!((brigand.id, brigand.label.as_str()), (UnitId(4), "Br"));
        brigand.has_effect()
    };
    assert!(under_effect(&pinned, &c));
    assert!(!under_effect(&plain, &c));
}

#[test]
fn a_boss_or_green_units_art_names_the_playback_and_the_players_doesnt() {
    let c = ctx();
    let state = swordsman(&c, 20);
    let used = |unit| Event::ArtUsed {
        unit,
        art: ArtId::new("guard_break"),
        weapon: trpg_core::ItemId::new("iron_sword"),
        durability_before: 20,
        durability_after: 16,
    };
    let units = state.units().to_vec();
    assert_eq!(
        playback_banner(&state, Words::ENGLISH, &[used(UnitId(1))], &units),
        None
    );
    assert_eq!(
        playback_banner(&state, Words::ENGLISH, &[used(UnitId(4))], &units).as_deref(),
        Some("Guard Break")
    );
    let mut green = units.clone();
    green[3].faction = Faction::Ally;
    let skill = Event::SkillUsed {
        unit: UnitId(4),
        skill: SkillId::new("keen_edge"),
    };
    assert_eq!(
        playback_banner(&state, Words::ENGLISH, &[skill], &green).as_deref(),
        Some("Keen Edge")
    );
    let own = Event::SkillUsed {
        unit: UnitId(1),
        skill: SkillId::new("keen_edge"),
    };
    assert_eq!(
        playback_banner(&state, Words::ENGLISH, &[own], &units),
        None
    );
    // Named in the box.
    let mut after = state.clone();
    let events = after
        .apply(&trpg_core::Command::Act {
            unit: UnitId(1),
            dest: p(7, 2),
            action: Technique::Attack.action(UnitId(4), &Equipped::Weapon(0)),
        })
        .unwrap();
    let pb = Playback::new(
        Words::ENGLISH,
        &events,
        state.units(),
        after.fallen(),
        TIMINGS,
    )
    .unwrap()
    .with_banner(Some("Guard Break".to_owned()));
    let shown = box_view(Words::ENGLISH, &pb).expect("the box");
    assert_eq!(shown.banner.as_deref(), Some("Guard Break"));
}

#[test]
fn close_shot_reaches_an_adjacent_enemy_that_a_plain_shot_cant() {
    let mut c = ctx();
    // The brigand (unit 5) at (7, 4), beside the archer at (8, 4).
    let state = battle(&c, |_| {});
    assert_eq!(state.unit(UnitId(5)).unwrap().pos, p(7, 4));
    // A plain shot doesn't offer it.
    let mut s = archer_menu(&mut c, state.clone());
    choose_entry(&mut s, &mut c, MenuEntry::Attack);
    assert_eq!(targeting(&s).targets, [UnitId(4)]);
    // Close Shot does, and the brigand two tiles up as well.
    let mut s = archer_on_brigand(&mut c, state, "Close Shot");
    assert_eq!(targeting(&s).targets, [UnitId(4), UnitId(5)]);
    assert_eq!(
        forecast_of(&s, &c).skill,
        Some(skill_line("Close Shot", 20, 18))
    );
    // Shoot the adjacent one: paid 2.
    aim(&mut s, &mut c, UnitId(5));
    assert_eq!(
        targeting(&s).technique,
        Technique::Art(ArtId::new("close_shot"))
    );
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(durability(s.state(), UnitId(3)), 18);
}

/// The archer knowing Long Shot, the brigand (unit 4) at `brigand`.
fn long_shot(c: &Ctx, brigand: Pos) -> BattleState {
    battle(c, |units| {
        units[3].pos = brigand;
        assert!(units[2].learn_skill(&SkillId::new("long_shot"), &c.game.content.skills));
    })
}

#[test]
fn long_shot_reaches_a_target_just_beyond_the_bows_range() {
    let mut c = ctx();
    // Three tiles up: past the bow's two, within Long Shot's extra reach.
    let state = long_shot(&c, p(8, 1));
    let mut s = archer_menu(&mut c, state);
    // A plain shot reaches nobody: the menu opens on `Arts`.
    let Mode::ActionMenu { menu, entries, .. } = s.mode() else {
        panic!("{:?}", s.mode());
    };
    assert!(!menu.items()[0].enabled);
    assert_eq!(entries[menu.focus()], MenuEntry::Arts);
    use_art(&mut s, &mut c, "Long Shot", UnitId(4));
    // The raider four tiles away (only Long Shot reaches it, +2) is first.
    assert_eq!(targeting(&s).targets[..2], [UnitId(6), UnitId(4)]);
    let before = durability(s.state(), UnitId(3));
    step(&mut s, &mut c, &[Action::Confirm]);
    assert!(durability(s.state(), UnitId(3)) < before);
    // Long Shot is one more line after the arts, before Vault.
    let state = long_shot(&c, p(8, 2));
    let mut s = archer_menu(&mut c, state);
    choose_entry(&mut s, &mut c, MenuEntry::Arts);
    let names: Vec<String> = lines(&s).into_iter().map(|l| l.0).collect();
    assert_eq!(names, ["Close Shot", "Pinning Shot", "Long Shot", "Vault"]);
}

#[test]
fn a_target_beyond_every_line_isnt_offered() {
    let c = ctx();
    // Three tiles up, without Long Shot: nothing reaches.
    let state = battle(&c, |units| units[3].pos = p(8, 1));
    let bow = Equipped::Weapon(0);
    let with = |state: &BattleState, technique: &Technique| {
        reach(state, UnitId(3), p(8, 4), &bow, technique)
    };
    let long = Technique::Active(SkillId::new("long_shot"));
    assert!(!with(&state, &Technique::Attack).contains(&UnitId(4)));
    // Without Long Shot the core refuses it, so nobody is offered.
    assert_eq!(with(&state, &long), []);
    let state = long_shot(&c, p(8, 1));
    assert!(!with(&state, &Technique::Attack).contains(&UnitId(4)));
    assert!(with(&state, &long).contains(&UnitId(4)));
}
