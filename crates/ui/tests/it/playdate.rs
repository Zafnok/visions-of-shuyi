//! The game as the Playdate's front end runs it (ticket 1033): its
//! 50 × 15 console, no keyboard, and only the D-pad, A (`South`) and B
//! (`East`) of a `PadKind::Playdate` pad. Screens keep their PC look until
//! their Playdate skins land (1037 on), so this checks where the buttons
//! lead, not how it looks; and that the one-bit renderer draws those
//! frames as 1032's did, whole or only what changed (ticket 1055).

use insta::assert_snapshot;
use trpg_content::FontAtlasDef;
use trpg_content::font::ATLAS_1BIT_PATH;
use trpg_core::lead::DEFAULT_NAME;
use trpg_core::{Faction, Phase};
use trpg_ui::game::system_menu::SystemMenuEntry;
use trpg_ui::harness::Harness;
use trpg_ui::input::{
    Action, Button, ButtonChord, Device, PLAYDATE_KEY_ROWS, PadKind, RIGHT_HANDED,
};
use trpg_ui::onebit::{self, FRAME_BYTES, Frame};
use trpg_ui::save::SUSPEND_KEY;
use trpg_ui::screen::Keyboard;
use trpg_ui::screens::battle::BattleScreen;
use trpg_ui::screens::battle::map_menu::MapEntry;
use trpg_ui::screens::battle::mode::Mode;
use trpg_ui::screens::battle::rewind::RewindScreen;
use trpg_ui::screens::key_bindings::view::SlotView;
use trpg_ui::screens::{
    FamiliarityScreen, KeyBindingsScreen, LeadSelectScreen, ModeSelectScreen, OptionsScreen,
};
use trpg_ui::widgets::help::key_name;
use trpg_ui::{
    ConsoleSizes, ConsoleSpec, Ctx, GlyphBuffer, KeyPrompt, Palette, RawInputEvent, ShuyiCtx,
    Storage, UiColor,
};

/// The context as `crates/playdate` sets it up (the tips and lessons
/// are turned on below), with the Playdate's own buttons (ticket 1034).
fn playdate(ctx: &mut Ctx) {
    ctx.console = ConsoleSpec::PLAYDATE;
    ctx.keyboard = Keyboard::Absent;
    ctx.device = Device::Pad(PadKind::Playdate);
    ctx.use_playdate_buttons();
    ctx.key_prompt = KeyPrompt::Waiting;
}

/// Skips every scene on screen: B, then A on "Skip scene?".
fn skip_scenes(h: &mut Harness) {
    for _ in 0..8 {
        if h.top_screen() != "dialogue" {
            return;
        }
        h.pad("East South");
    }
    panic!("the scenes never end: {:?}", h.screens());
}

/// The ticket's walk-through: the title, New Game, the lead's name on the
/// letter grid, and into Chapter 1's first battle, with no layout ever
/// asked for.
#[test]
fn the_d_pad_a_and_b_play_from_the_title_into_chapter_ones_first_battle() {
    let mut h = Harness::first_launch_and(playdate).with_shipped_story();
    h.use_pad(PadKind::Playdate).with_tips().with_lessons();
    assert_eq!(h.game().buffer().spec(), ConsoleSpec::PLAYDATE);
    // A ends the title's wait, then picks New Game, Classic and the
    // familiarity answer.
    h.pad("South South South South");
    assert_eq!(h.top_screen(), "lead_select");
    // The name: A on it opens the letter grid; B deletes a letter, A adds
    // the grid's first one (A), and A on Done (up, then left, from the
    // first letter) closes it.
    h.pad("DpadDown South East South DpadUp DpadLeft South");
    assert_eq!(h.top_screen(), "lead_select");
    // Then down to Start, and A.
    h.pad("DpadDown South");
    skip_scenes(&mut h);
    // The battle's banner (A), then its opening scene.
    h.pad("South");
    skip_scenes(&mut h);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert_eq!(h.game().ctx().layout(), None);
    assert_eq!(h.game().ctx().device, Device::Pad(PadKind::Playdate));
    let mut name = DEFAULT_NAME.to_owned();
    name.pop();
    name.push('A');
    assert_eq!(h.game().ctx().game.lead.name, name);
    // A closes the first guide; A on the empty tile above the lead opens
    // the map menu, whose seventh entry that can be chosen is Suspend
    // (after the shortcuts with no button, ticket 1035; Unit Info is
    // dimmed on an empty tile); A on it and A on "Suspend the battle?"
    // write the suspend save and go back to the title.
    h.wait(1.0);
    h.pad("South DpadUp South");
    for _ in 0..6 {
        h.pad("DpadDown");
    }
    h.pad("South South");
    assert_eq!(h.screens(), ["title"]);
    let saved = h.game().ctx().storage.read(SUSPEND_KEY);
    assert!(saved.is_ok_and(|s| s.is_some()), "no suspend save");
}

/// The context as `crates/playdate` sets it up, with its palette of inks.
fn playdate_with_inks(ctx: &mut Ctx) {
    playdate(ctx);
    let palette = trpg_content::palette::load()
        .and_then(|base| trpg_content::ink::load_playdate(&base))
        .ok()
        .and_then(|inks| Palette::new::<UiColor>(&inks).ok());
    let Some(palette) = palette else {
        panic!("the Playdate palette does not load");
    };
    ctx.palette = palette;
}

/// The context as `crates/playdate` sets it up, with its inks and its
/// screens' skins (ticket 1045).
fn playdate_skins(ctx: &mut Ctx) {
    playdate_with_inks(ctx);
    ctx.use_playdate_skins();
}

/// The top screen's view, of a screen of type `S`.
fn view_of<S: 'static, V>(h: &Harness, view: impl Fn(&S, &Ctx) -> V) -> V {
    let Some(screen) = h.screen::<S>() else {
        panic!("not on that screen: {:?}", h.screens());
    };
    view(screen, h.game().ctx())
}

/// Ticket 1045: New Game to the first scene on the Playdate's skins: the
/// mode, the familiarity, the lead (the name spelled on the letter grid),
/// each screen as the Playdate shows it, then Chapter 1's opening scene.
#[test]
fn new_game_to_the_first_scene_on_the_playdates_skins() {
    let mut h = Harness::first_launch_and(playdate_skins).with_shipped_story();
    h.use_pad(PadKind::Playdate);
    let mut one_bit = OneBit::new(&h);
    // A ends the title's wait, then picks New Game (on its release: the
    // game flow pushes its first screen on the next frame).
    h.pad("South South").wait(trpg_ui::harness::FRAME_DT);
    let Some(mode) = h.screen::<ModeSelectScreen>() else {
        panic!("no game mode screen: {:?}", h.screens());
    };
    let view = mode.view(h.game().ctx());
    assert_eq!(
        view.menu.focused().map(|i| i.label.as_str()),
        Some("Classic")
    );
    assert_snapshot!("playdate_mode_select", h.snapshot());
    one_bit.check(&h);
    h.pad("South");
    let Some(familiarity) = h.screen::<FamiliarityScreen>() else {
        panic!("no familiarity screen: {:?}", h.screens());
    };
    assert_eq!(familiarity.view(h.game().ctx()).menu.items.len(), 3);
    assert_snapshot!("playdate_familiarity", h.snapshot());
    one_bit.check(&h);
    h.pad("South");
    assert_eq!(h.top_screen(), "lead_select");
    assert_snapshot!("playdate_lead_select", h.snapshot());
    one_bit.check(&h);
    // A on the name opens the grid; B deletes a letter, A types the first.
    h.pad("DpadDown South East South");
    let Some(lead) = h.screen::<LeadSelectScreen>() else {
        panic!("no lead screen: {:?}", h.screens());
    };
    let view = lead.view(h.game().ctx());
    let mut name = DEFAULT_NAME.to_owned();
    name.pop();
    name.push('A');
    assert_eq!(view.grid.map(|g| g.spelled), Some(name.clone()));
    assert_snapshot!("playdate_letter_grid", h.snapshot());
    one_bit.check(&h);
    // Done (up, then left), down to Start, A: the opening scene.
    h.pad("DpadUp DpadLeft South DpadDown South");
    for _ in 0..40 {
        if h.top_screen() == "dialogue" {
            break;
        }
        h.wait(0.25);
    }
    assert_eq!(h.top_screen(), "dialogue");
    assert_eq!(h.game().ctx().game.lead.name, name);
}

/// Ticket 1045: Options from the title and from the system menu, as the
/// Playdate shows it (no layout, window, colour theme or map look); its
/// Key bindings has the controller side alone, a slot takes a button chord
/// (B + down) and Restore defaults puts the buttons back.
#[test]
fn options_and_a_chord_rebound_and_restored_on_the_playdate() {
    use trpg_ui::screens::options::Row;
    let mut h = Harness::first_launch_and(playdate_skins).with_shipped_story();
    h.use_pad(PadKind::Playdate);
    // The title's Options: third in its menu.
    h.pad("South DpadDown DpadDown South");
    assert_eq!(h.screens(), ["title", "options"]);
    let rows = |h: &Harness| -> Vec<&'static str> {
        let view = view_of(h, |s: &OptionsScreen, c| s.view(c));
        view.rows.iter().map(|r| r.key).collect()
    };
    let shown = rows(&h);
    for hidden in [Row::Layout, Row::Fullscreen, Row::Theme, Row::MapLook] {
        assert!(!shown.contains(&hidden.key()), "{hidden:?} shown");
    }
    assert!(shown.contains(&Row::KeyBindings.key()));
    assert_snapshot!("playdate_options", h.snapshot());
    // B leaves; the system menu's Options opens the same screen.
    h.pad("East");
    h.system_menu(SystemMenuEntry::Options);
    assert_eq!(h.screens(), ["title", "options"]);
    assert_eq!(rows(&h), shown);
    // Key bindings: up from the first row, past Restore defaults and Reset
    // tips.
    h.pad("DpadUp DpadUp DpadUp South");
    let kb = |h: &Harness| {
        view_of(h, |s: &KeyBindingsScreen, c| {
            (s.is_controller_only(), s.view(c))
        })
    };
    let (controller_only, view) = kb(&h);
    assert!(controller_only && view.switch.is_none());
    assert!(view.row(Action::RewindBack).is_some());
    assert_snapshot!("playdate_key_bindings", h.snapshot());
    one_bit_frame(&h);
    // Danger zone's first slot: Change, then B and down together.
    let rows_down = PLAYDATE_KEY_ROWS
        .iter()
        .position(|r| r.0 == Action::DangerZone);
    for _ in 0..rows_down.unwrap_or(0) {
        h.pad("DpadDown");
    }
    h.pad("South South");
    assert!(view_of(&h, |s: &KeyBindingsScreen, _| s.is_capturing()));
    // Hold B, press down, let go of down, then of B (B is Cancel when let
    // go: not here, after the capture).
    let frame = |h: &mut Harness, event: RawInputEvent| {
        h.frame(&[event], 1.0 / 30.0);
    };
    let pad = PadKind::Playdate;
    frame(&mut h, RawInputEvent::PadDown(Button::East, pad));
    frame(&mut h, RawInputEvent::PadDown(Button::DpadDown, pad));
    frame(&mut h, RawInputEvent::PadUp(Button::DpadDown));
    frame(&mut h, RawInputEvent::PadUp(Button::East));
    assert_eq!(h.top_screen(), "key_bindings");
    let (_, view) = kb(&h);
    let chord = |h: &Harness| h.game().ctx().pad_bindings().slots(Action::DangerZone)[0];
    let named = view.row(Action::DangerZone).map(|r| r.slots[0].clone());
    assert_eq!(named, Some(SlotView::Bound("B+↓".to_owned())));
    // B leaves Key bindings: saved.
    h.pad("East");
    assert_eq!(h.top_screen(), "options");
    assert_eq!(chord(&h), ButtonChord::pair(Button::East, Button::DpadDown));
    // Again, and Restore defaults (up from the first row): A, then A.
    h.pad("South DpadUp South South East");
    assert_eq!(h.top_screen(), "options");
    assert_eq!(chord(&h), None);
}

/// Checks the frame on screen draws as 1032's reference renderer does.
fn one_bit_frame(h: &Harness) {
    OneBit::new(h).check(h);
}

/// Draws each frame it is shown as the Playdate's front end does: whole
/// the first time, then only what changed since the last; and checks
/// each against 1032's reference renderer.
struct OneBit {
    atlas: FontAtlasDef,
    shown: Option<GlyphBuffer>,
    frame: Box<Frame>,
    checked: usize,
}

impl OneBit {
    fn new(h: &Harness) -> Self {
        Self {
            atlas: h.game().ctx().game.content.font.clone(),
            shown: None,
            frame: Box::new([0; FRAME_BYTES]),
            checked: 0,
        }
    }

    fn check(&mut self, h: &Harness) {
        let bits = trpg_content::bundle::bytes(ATLAS_1BIT_PATH);
        let Some(font) = bits.and_then(|bits| onebit::Font::new(&self.atlas, bits).ok()) else {
            panic!("no one-bit font");
        };
        let buffer = h.game().buffer();
        match &self.shown {
            Some(old) => onebit::render_changed(old, buffer, &font, &(), &mut self.frame),
            None => onebit::render(buffer, &font, &(), &mut self.frame),
        }
        let mut whole = Box::new([0; FRAME_BYTES]);
        onebit::render(buffer, &font, &(), &mut whole);
        let mut reference = Box::new([0; FRAME_BYTES]);
        onebit::reference::render(buffer, &font, &(), &mut reference);
        let screen = h.top_screen();
        assert!(whole == reference, "{screen}: not as the reference drew it");
        assert!(self.frame == whole, "{screen}: what changed, drawn wrong");
        self.shown = Some(buffer.clone());
        self.checked += 1;
    }

    /// `check` after each of `frames` frames of `seconds`.
    fn frames(&mut self, h: &mut Harness, frames: usize, seconds: f32) {
        for _ in 0..frames {
            h.wait(seconds);
            self.check(h);
        }
    }
}

/// The title, the opening scene's lines typing out, and Chapter 1's first
/// battle with its cursor moving, on the Playdate's console and palette.
#[test]
fn the_one_bit_frames_are_the_references_whole_or_changed() {
    let mut h = Harness::first_launch_and(playdate_with_inks).with_shipped_story();
    h.use_pad(PadKind::Playdate).with_tips().with_lessons();
    let mut one_bit = OneBit::new(&h);
    one_bit.check(&h);
    // The title, then its menu.
    h.pad("South");
    one_bit.frames(&mut h, 3, 0.2);
    // New Game, Classic, the familiarity answer, then Start.
    h.pad("South South South");
    one_bit.check(&h);
    h.pad("DpadDown DpadDown South");
    one_bit.check(&h);
    // The opening scene, a line typing out a frame at a time.
    let mut saw_dialogue = false;
    for _ in 0..40 {
        h.wait(1.0 / 30.0);
        one_bit.check(&h);
        saw_dialogue |= h.top_screen() == "dialogue";
    }
    assert!(saw_dialogue, "no scene: {:?}", h.screens());
    skip_scenes(&mut h);
    one_bit.check(&h);
    // The battle's banner, its scene, then the map.
    h.pad("South");
    one_bit.check(&h);
    skip_scenes(&mut h);
    one_bit.frames(&mut h, 3, 0.4);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    // The guide closed, the cursor around the lead.
    for step in [
        "South",
        "DpadUp",
        "DpadRight",
        "DpadDown",
        "DpadDown",
        "DpadLeft",
    ] {
        h.pad(step);
        one_bit.frames(&mut h, 2, 1.0 / 30.0);
    }
    assert!(one_bit.checked > 60);
}

/// Ticket 1034: on the Playdate, A + B together ends the turn (its
/// question shows, as on the PC) and holding B while pressing right or
/// left picks the next or previous ready unit, in Chapter 1's first
/// battle; B alone still cancels.
#[test]
fn a_and_b_end_the_turn_and_hold_b_with_left_or_right_picks_a_unit() {
    let mut h = Harness::first_launch_and(playdate).with_shipped_story();
    h.use_pad(PadKind::Playdate);
    // A to New Game, Classic and the familiarity answer, the name as it
    // is, Start; past the scenes and the banner.
    h.pad("South South South South DpadDown DpadDown South");
    skip_scenes(&mut h);
    h.pad("South");
    skip_scenes(&mut h);
    h.wait(1.0);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    let battle = |h: &Harness| h.battle().map(|b| b.mode().clone());
    let idle = |h: &Harness| matches!(battle(h), Some(Mode::Idle { .. }));
    let on_a_ready_unit = |h: &Harness| {
        let unit = h.cursor_tile().and_then(|at| h.unit_at(at));
        unit.is_some_and(|u| u.faction == Faction::Player && !u.acted)
    };
    assert!(idle(&h) && on_a_ready_unit(&h));
    let start = h.cursor_tile();
    // Hold B, press right: the next ready unit, and B cancels nothing.
    h.pad("East+DpadRight");
    assert!(idle(&h) && on_a_ready_unit(&h));
    assert_ne!(h.cursor_tile(), start);
    // And left: back to the first.
    h.pad("East+DpadLeft");
    assert_eq!(h.cursor_tile(), start);
    assert!(idle(&h));
    // Help text names the chords through the keymap.
    let keys = h.game().ctx().help_keys();
    assert_eq!(key_name(keys, Action::EndTurn), "A+B");
    assert_eq!(key_name(keys, Action::NextUnit), "B+→");
    // A + B: the question, as on the PC.
    h.pad("South+East");
    assert!(matches!(battle(&h), Some(Mode::EndTurnPrompt { .. })));
    // B alone backs out of it; A + B again, then A, ends the turn.
    h.pad("East");
    assert!(idle(&h));
    assert_eq!(h.cursor_tile(), start, "B cancels, it doesn't move");
    h.pad("South+East South");
    let phase = h.battle().map(|b| b.state().phase());
    assert_ne!(phase, Some(Phase::Player));
}

/// Chapter 1's first battle on the Playdate, lessons off: New Game,
/// Classic, the familiarity answer, the name as it is, Start, past the
/// scenes and the banner, with the cursor on the lead.
fn first_battle() -> Harness {
    first_battle_and(|_| {})
}

/// [`first_battle`], with `setup` changing the context first.
fn first_battle_and(setup: impl FnOnce(&mut Ctx)) -> Harness {
    let mut h = Harness::first_launch_and(|ctx| {
        playdate(ctx);
        setup(ctx);
    })
    .with_shipped_story();
    h.use_pad(PadKind::Playdate);
    h.pad("South South South South DpadDown DpadDown South");
    skip_scenes(&mut h);
    h.pad("South");
    skip_scenes(&mut h);
    h.wait(1.0);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    h
}

fn battle(h: &Harness) -> &BattleScreen {
    h.battle().unwrap_or_else(|| panic!("no battle"))
}

/// Ends the turn (A + B, then A on its question) and plays on to the
/// player's turn 2: the enemy's phase at the held speed, its banners and
/// scenes closed.
fn to_turn_two(h: &mut Harness) {
    h.pad("South+East South");
    for _ in 0..200 {
        skip_scenes(h);
        let b = battle(h);
        let state = b.state();
        if state.turn() == 2 && state.phase() == Phase::Player && b.banner().is_none() {
            h.wait(0.1);
            return;
        }
        if b.banner().is_some() {
            h.pad("South");
        } else {
            h.hold_pad("South", 0.5);
        }
    }
    panic!("turn 2 never came");
}

/// The rewind screen's past actions (their points), newest first, and the
/// one highlighted.
fn rewind_points(h: &Harness) -> Option<(Vec<usize>, Option<usize>)> {
    let r = h.screen::<RewindScreen>()?;
    let points = r.entries().iter().map(|e| e.point).collect();
    Some((points, r.focused().map(|e| e.point)))
}

/// Ticket 1035: in Chapter 1's first battle, the crank wound back opens
/// the rewind and steps back one action per notch; wound forwards it
/// steps towards the present; B closes it; A asks "Use 1 of N charges?"
/// and A again rewinds, as on the PC.
#[test]
fn winding_the_crank_back_rewinds_one_action_per_notch() {
    let mut h = first_battle();
    // Wound forwards on the map: nothing.
    h.crank(30.0);
    assert!(h.screen::<RewindScreen>().is_none());
    assert!(matches!(battle(&h).mode(), Mode::Idle { .. }));
    to_turn_two(&mut h);
    let charges = battle(&h).history().charges_left();
    // Three notches back: open on the newest, then two further back.
    h.crank(-30.0);
    let Some((points, focus)) = rewind_points(&h) else {
        panic!("the crank did not open the rewind");
    };
    assert!(points.len() >= 3, "{points:?}");
    assert_eq!(focus, Some(points[0]));
    h.crank(-30.0).crank(-30.0);
    assert_eq!(rewind_points(&h), Some((points.clone(), Some(points[2]))));
    // One forward: the second newest.
    h.crank(30.0);
    assert_eq!(rewind_points(&h), Some((points.clone(), Some(points[1]))));
    // Half a notch is nothing yet; the other half is a notch.
    h.crank(-15.0);
    assert_eq!(rewind_points(&h), Some((points.clone(), Some(points[1]))));
    h.crank(-15.0);
    assert_eq!(rewind_points(&h), Some((points.clone(), Some(points[2]))));
    // A big wind: several notches in one frame, never past the oldest.
    h.crank(-3600.0);
    let oldest = points.last().copied();
    assert_eq!(rewind_points(&h), Some((points.clone(), oldest)));
    // B closes it, the battle as it was.
    h.pad("East");
    assert!(h.screen::<RewindScreen>().is_none());
    assert_eq!(battle(&h).history().len(), points.len());
    // Again: the newest. A asks, and the crank does nothing meanwhile.
    h.crank(-30.0).pad("South");
    let asking = |h: &Harness| {
        h.screen::<RewindScreen>()
            .is_some_and(RewindScreen::is_confirming)
    };
    assert!(asking(&h));
    h.crank(-30.0);
    assert_eq!(rewind_points(&h), Some((points.clone(), Some(points[0]))));
    // B backs out of the question; A, A rewinds for one charge (to the
    // enemy's phase, which plays on).
    h.pad("East");
    assert!(!asking(&h) && h.screen::<RewindScreen>().is_some());
    h.pad("South South");
    assert!(h.screen::<RewindScreen>().is_none());
    assert_eq!(battle(&h).history().charges_left(), charges - 1);
}

/// Ticket 1035: a notch is as far as the keymap file's `crank` says.
#[test]
fn a_notch_is_the_keymaps() {
    let mut h = first_battle_and(|ctx| ctx.game.content.keymap.crank.notch_degrees = 90);
    h.crank(-60.0);
    assert!(
        h.screen::<RewindScreen>().is_none(),
        "60 of 90 degrees: no notch yet"
    );
    h.crank(-30.0);
    assert!(h.screen::<RewindScreen>().is_some());
}

/// The map menu's entries, with whether each can be chosen.
fn map_menu_entries(h: &Harness) -> Vec<(MapEntry, bool)> {
    let Mode::MapMenu { menu, entries } = battle(h).mode() else {
        panic!("no map menu: {:?}", battle(h).mode());
    };
    let items = menu.items().iter().map(|i| i.enabled);
    entries.iter().copied().zip(items).collect()
}

/// The entries alone.
fn map_menu_shown(h: &Harness) -> Vec<MapEntry> {
    map_menu_entries(h).iter().map(|&(e, _)| e).collect()
}

/// Opens the map menu (B, with nothing to cancel) and chooses `entry`.
fn choose(h: &mut Harness, entry: MapEntry) {
    h.pad("East");
    let at = map_menu_shown(h).iter().position(|&e| e == entry);
    let Some(at) = at else {
        panic!("no {entry:?} in the map menu");
    };
    for _ in 0..at {
        h.pad("DpadDown");
    }
    h.pad("South");
}

/// Ticket 1035: with no keyboard, the map menu lists Danger Zone,
/// Auto-End and Unit Info after Objective while they have no button, and
/// each does what its key does; Unit Info is dimmed on an empty tile.
#[test]
fn the_playdates_map_menu_lists_the_shortcuts_with_no_button() {
    use MapEntry::{AutoEnd, DangerZone, UnitInfo};
    let mut h = first_battle();
    let lead = h.cursor_tile();
    h.pad("East");
    let shown = map_menu_shown(&h);
    assert_eq!(&shown[..2], &MapEntry::ALL[..2]);
    assert_eq!(&shown[2..5], &[DangerZone, AutoEnd, UnitInfo]);
    assert_eq!(&shown[5..], &MapEntry::ALL[2..]);
    assert!(map_menu_entries(&h).iter().all(|&(_, enabled)| enabled));
    h.pad("East");
    // Danger Zone: the menu closes and the zone shows; again, it hides.
    choose(&mut h, DangerZone);
    assert!(matches!(battle(&h).mode(), Mode::Idle { .. }));
    assert!(battle(&h).danger().is_some());
    choose(&mut h, DangerZone);
    assert!(battle(&h).danger().is_none());
    // Auto-End: on, then off.
    let auto_end = battle(&h).auto_end();
    choose(&mut h, AutoEnd);
    assert_eq!(battle(&h).auto_end(), !auto_end);
    choose(&mut h, AutoEnd);
    assert_eq!(battle(&h).auto_end(), auto_end);
    // Unit Info: the lead's.
    choose(&mut h, UnitInfo);
    let units = battle(&h).state().units();
    let lead_id = units.iter().find(|u| Some(u.pos) == lead).map(|u| u.id);
    assert!(lead_id.is_some());
    assert!(matches!(battle(&h).mode(), Mode::Info { unit } if Some(*unit) == lead_id));
    h.pad("East");
    assert_eq!(h.cursor_tile(), lead);
    // On an empty tile (above the lead) it is dimmed.
    h.pad("DpadUp");
    assert!(h.cursor_tile().and_then(|at| h.unit_at(at)).is_none());
    h.pad("East");
    let unit_info = map_menu_entries(&h)
        .into_iter()
        .find(|&(e, _)| e == UnitInfo);
    assert_eq!(unit_info, Some((UnitInfo, false)));
    h.pad("East");
    // Danger Zone given a button by the player (A + down here): gone.
    let mut buttons = h.game().ctx().pad_bindings();
    let Some(chord) = ButtonChord::pair(Button::South, Button::DpadDown) else {
        panic!("not a chord");
    };
    assert!(buttons.bind(Action::DangerZone, 0, chord).is_ok());
    assert!(h.ctx_mut().set_pad_bindings(buttons).is_ok());
    h.pad("East");
    let shown = map_menu_shown(&h);
    assert!(!shown.contains(&DangerZone), "{shown:?}");
    assert!(shown.contains(&AutoEnd) && shown.contains(&UnitInfo));
}

/// Ticket 1035: the PC's map menu is unchanged, with the keyboard or a
/// controller.
#[test]
fn the_pcs_map_menu_has_no_shortcut_entries() {
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    h.keys("f f f Up f");
    for _ in 0..8 {
        if h.top_screen() == "dialogue" {
            h.keys("d f");
        } else if h.battle().is_some() {
            h.keys("f");
        }
    }
    h.wait(1.0);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert!(matches!(battle(&h).mode(), Mode::Idle { .. }));
    h.keys("d");
    assert_eq!(map_menu_shown(&h), MapEntry::ALL);
    h.keys("d").pad("East");
    assert_eq!(h.game().ctx().device, Device::Pad(PadKind::Generic));
    assert_eq!(map_menu_shown(&h), MapEntry::ALL);
}

/// Ticket 1035: the system menu's Danger Zone, Dialogue Log and Options
/// each do what the same thing does in the game where it can, and
/// nothing elsewhere.
#[test]
fn the_system_menu_entries_work_where_they_can() {
    let mut h = Harness::first_launch_and(playdate).with_shipped_story();
    h.use_pad(PadKind::Playdate);
    // The title waiting for a button: nothing opens over it.
    h.system_menu(SystemMenuEntry::Options);
    assert_eq!(h.screens(), ["title"]);
    // Its menu: Options opens, B closes it.
    h.pad("South");
    h.system_menu(SystemMenuEntry::Options);
    assert_eq!(h.screens(), ["title", "options"]);
    h.pad("East");
    assert_eq!(h.screens(), ["title"]);
    // No danger zone or log there.
    h.system_menu(SystemMenuEntry::DangerZone)
        .system_menu(SystemMenuEntry::DialogueLog);
    assert_eq!(h.screens(), ["title"]);
    // Into the opening scene: the log opens once a line has shown.
    h.pad("South South South DpadDown DpadDown South");
    for _ in 0..40 {
        if h.top_screen() == "dialogue" {
            break;
        }
        h.wait(0.25);
    }
    assert_eq!(h.top_screen(), "dialogue");
    // The chapter's scene, over the game flow.
    let log_open = |h: &Harness| {
        let scene = h.screen::<trpg_ui::screens::DialogueScreen>();
        scene.is_some_and(trpg_ui::screens::DialogueScreen::is_log_open)
    };
    h.wait(3.0);
    h.system_menu(SystemMenuEntry::DialogueLog);
    assert!(log_open(&h));
    // Options does not open over a scene.
    h.system_menu(SystemMenuEntry::Options);
    assert_eq!(h.top_screen(), "dialogue");
    h.pad("East");
    assert!(!log_open(&h));
    skip_scenes(&mut h);
    h.pad("South");
    skip_scenes(&mut h);
    h.wait(1.0);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    // The battle map: the danger zone toggles; no log.
    h.system_menu(SystemMenuEntry::DangerZone);
    assert!(battle(&h).danger().is_some());
    h.system_menu(SystemMenuEntry::DangerZone);
    assert!(battle(&h).danger().is_none());
    h.system_menu(SystemMenuEntry::DialogueLog);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    // Options over the map, and over the open map menu.
    h.system_menu(SystemMenuEntry::Options);
    assert_eq!(h.screens(), ["title", "game_flow", "battle", "options"]);
    h.pad("East");
    h.pad("East");
    assert!(matches!(battle(&h).mode(), Mode::MapMenu { .. }));
    h.system_menu(SystemMenuEntry::Options);
    assert_eq!(h.screens(), ["title", "game_flow", "battle", "options"]);
    h.pad("East");
    assert!(matches!(battle(&h).mode(), Mode::MapMenu { .. }));
    // A unit selected: Options waits for the map.
    h.pad("East South");
    assert!(matches!(battle(&h).mode(), Mode::Selected(_)));
    h.system_menu(SystemMenuEntry::Options);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
}
