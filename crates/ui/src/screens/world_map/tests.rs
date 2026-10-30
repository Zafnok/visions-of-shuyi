//! Tests of the world map screen on the test world map
//! (`assets-test/worldmaps/test_world.ron`): the army starts in Millbrook
//! (a town with an Armory and a Vendor); the open places are the
//! lighthouse, the crossroads, the farm, Stone Mill (the story battle) and
//! Kingsgate (a capital with a Tailor and a Blacksmith).

use trpg_content::battle_campaign;
use trpg_core::{GameMode, LeadGender, LeadProfile};

use super::*;
use crate::audio::AudioRequest;
use crate::color::UiColor;
use crate::screen::tests::ctx;
use crate::screens::popped_with;

/// The test battle's army (the lead and the test knight) at the test world
/// map's first chapter, on its map, with 2 000 gold.
fn army(ctx: &Ctx) -> Campaign {
    let content = &ctx.game.content;
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let mut c = battle_campaign(content, &content.battles["test"], GameMode::Classic, lead);
    c.gold = 2000;
    c.chapter = "test_wm1".into();
    c.enter_world(&content.worldmaps["test_world"].map);
    c
}

fn screen(ctx: &Ctx) -> WorldMapScreen {
    WorldMapScreen::new(army(ctx), 0.0)
}

/// One frame with `actions`, `dt` seconds after the last.
fn frame(s: &mut WorldMapScreen, ctx: &mut Ctx, actions: &[Action], dt: f64) -> Transition {
    ctx.clock_s += dt;
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    s.update(ctx, &input)
}

fn press(s: &mut WorldMapScreen, ctx: &mut Ctx, actions: &[Action]) -> Transition {
    frame(s, ctx, actions, 0.0)
}

/// Lets the army walk where it was sent: the transition at its arrival.
fn arrive(s: &mut WorldMapScreen, ctx: &mut Ctx) -> Transition {
    for _ in 0..40 {
        let t = frame(s, ctx, &[], STEP_S / 2.0);
        if !s.walking() {
            return t;
        }
    }
    panic!("the army never arrived");
}

fn at(s: &WorldMapScreen) -> &str {
    &s.campaign().world.as_ref().expect("on the map").at
}

/// The open places shown, with their marks and level markers; the army's
/// own isn't one (the army is drawn there).
#[test]
fn only_the_open_places_show_battles_with_their_level() {
    let ctx = ctx();
    let view = screen(&ctx).view(&ctx);
    let marks: Vec<(PlaceMark, Option<&str>)> = view
        .places
        .iter()
        .filter(|p| !p.locked)
        .map(|p| (p.mark, p.level.as_deref()))
        .collect();
    assert_eq!(
        marks,
        [
            (PlaceMark::Empty, None),
            (PlaceMark::Empty, None),
            (PlaceMark::Empty, None),
            (PlaceMark::StoryBattle, Some("Lv1")),
            (PlaceMark::Capital, None),
        ]
    );
    // One road from Stone Mill: Wolf Pass and Old Bridge, greyed; the
    // chapel is hidden.
    assert_eq!(view.places.iter().filter(|p| p.locked).count(), 2);
    assert_eq!(view.army, Pos::new(14, 20));
    assert_eq!(view.army_label, "Ma");
    // The roads between open places: none yet to Wolf Pass, the chapel or
    // Old Bridge; those to the first and last are shown dimmed.
    assert_eq!(view.roads.len(), 5);
    assert_eq!(view.locked_roads.len(), 2);
    assert_eq!(view.panel.title, "Millbrook");
    assert_eq!(view.panel.kind, "Town");
    assert_eq!(view.panel.lines, ["Armory", "Vendor", "Your army is here"]);
}

/// Once Stone Mill is the army's, the places one road from it show, greyed
/// (not the hidden chapel), with their roads dimmed: the cursor goes there,
/// the panel says what each is, but the army can't travel there.
#[test]
fn places_one_road_away_show_greyed_and_cant_be_reached() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    press(
        &mut s,
        &mut ctx,
        &[Action::CursorRight, Action::CursorRight, Action::Confirm],
    );
    let t = arrive(&mut s, &mut ctx);
    let battle: WorldBattle = popped_with(t).expect("Stone Mill's battle");
    let mut s = WorldMapScreen::new(battle.campaign, 0.0);
    let view = s.view(&ctx);
    let locked: Vec<(Pos, PlaceMark)> = view
        .places
        .iter()
        .filter(|p| p.locked)
        .map(|p| (p.pos, p.mark))
        .collect();
    assert_eq!(
        locked,
        [
            (Pos::new(30, 5), PlaceMark::SideQuest),
            (Pos::new(47, 14), PlaceMark::StoryBattle)
        ]
    );
    assert!(
        view.places
            .iter()
            .filter(|p| p.locked)
            .all(|p| p.level.is_none())
    );
    assert_eq!(view.locked_roads.len(), 2);
    press(&mut s, &mut ctx, &[Action::CursorRight]);
    let view = s.view(&ctx);
    assert_eq!(view.panel.title, "Old Bridge");
    assert_eq!(view.panel.lines, ["Not open yet", "Enemy level 1"]);
    press(&mut s, &mut ctx, &[Action::Confirm]);
    assert!(!s.walking());
    assert_eq!(at(&s), "ford");
}

/// The cursor's keys jump to the nearest place that way.
#[test]
fn the_cursor_jumps_to_the_nearest_place_that_way() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    for (action, to) in [
        (Action::CursorUp, "camp"),
        (Action::CursorRight, "farm"),
        (Action::CursorDown, "cross"),
        (Action::CursorDown, "city"),
        (Action::CursorUp, "cross"),
        (Action::CursorRight, "ford"),
    ] {
        press(&mut s, &mut ctx, &[action]);
        assert_eq!(s.cursor(), to, "{action:?}");
    }
    let view = s.view(&ctx);
    assert_eq!(view.panel.title, "Stone Mill");
    assert_eq!(view.panel.lines, ["Enemy level 1"]);
    // Old Bridge, not open yet, then nothing further right: it stays.
    press(&mut s, &mut ctx, &[Action::CursorRight]);
    assert_eq!(s.cursor(), "bridge");
    press(&mut s, &mut ctx, &[Action::CursorRight]);
    assert_eq!(s.cursor(), "bridge");
    press(&mut s, &mut ctx, &[Action::CursorLeft]);
    assert_eq!(s.cursor(), "ford");
}

/// Previous and Next unit go through the places in the map's order; Your
/// army jumps back to the army.
#[test]
fn next_and_previous_go_through_the_places_and_your_army_comes_back() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    press(&mut s, &mut ctx, &[Action::NextUnit]);
    assert_eq!(s.cursor(), "camp");
    press(&mut s, &mut ctx, &[Action::PrevUnit, Action::PrevUnit]);
    assert_eq!(s.cursor(), "city");
    press(&mut s, &mut ctx, &[Action::WorldArmy]);
    assert_eq!(s.cursor(), "home");
}

/// Find: the highlighted kind's nearest place gets the cursor; Previous
/// and Next unit go through that kind while Find is open.
#[test]
fn find_jumps_to_a_kind_and_cycles_through_it() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    press(&mut s, &mut ctx, &[Action::NextUnit]);
    press(&mut s, &mut ctx, &[Action::WorldFind]);
    assert_eq!(s.cursor(), "home", "Your army first");
    let view = s.view(&ctx);
    let list = view.list.expect("Find is open");
    assert_eq!(list.title, "Find");
    press(&mut s, &mut ctx, &[Action::CursorDown]);
    assert_eq!(s.cursor(), "home", "the nearest town is the army's");
    press(&mut s, &mut ctx, &[Action::NextUnit]);
    assert_eq!(s.cursor(), "city");
    press(&mut s, &mut ctx, &[Action::CursorDown]);
    assert_eq!(s.cursor(), "ford");
    press(&mut s, &mut ctx, &[Action::Confirm]);
    assert!(s.view(&ctx).list.is_none());
    assert_eq!(s.cursor(), "ford");
}

/// Find's nearest place is the nearest to the army, across and down
/// (here Kingsgate, moved beside Stone Mill, beats Millbrook); with three
/// towns, Next and Previous go opposite ways round them.
#[test]
fn find_picks_the_nearest_and_cycles_both_ways() {
    let mut ctx = ctx();
    let def = ctx
        .game
        .content
        .worldmaps
        .get_mut("test_world")
        .expect("a map");
    def.places.insert("city".into(), Pos::new(36, 20));
    let farm = def
        .map
        .nodes
        .iter_mut()
        .find(|n| n.id == "farm")
        .expect("the farm");
    farm.kind = NodeKind::Town {
        shops: Vec::new(),
        capital: false,
    };
    let mut c = army(&ctx);
    c.world.as_mut().expect("on the map").at = "ford".into();
    let mut s = WorldMapScreen::new(c, 0.0);
    press(&mut s, &mut ctx, &[Action::WorldFind, Action::CursorDown]);
    assert_eq!(s.cursor(), "city");
    // The towns in the map's order: Millbrook, the farm, Kingsgate.
    press(&mut s, &mut ctx, &[Action::NextUnit]);
    assert_eq!(s.cursor(), "home");
    press(&mut s, &mut ctx, &[Action::PrevUnit, Action::PrevUnit]);
    assert_eq!(s.cursor(), "farm");
}

#[test]
fn the_cursor_pulses_from_full_to_half_and_back() {
    assert!((pulse(0.0) - 1.0).abs() < 1e-4);
    assert!((pulse(0.5) - 0.5).abs() < 1e-4);
    assert!((pulse(1.0) - 1.0).abs() < 1e-4);
    assert!((pulse(10.5) - 0.5).abs() < 1e-3);
}

/// The help line: `enter` with the cursor on the army, `travel` elsewhere.
#[test]
fn the_help_says_enter_on_the_army_and_travel_elsewhere() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    assert!(
        s.view(&ctx).help.contains("f enter"),
        "{}",
        s.view(&ctx).help
    );
    press(&mut s, &mut ctx, &[Action::CursorRight]);
    assert!(
        s.view(&ctx).help.contains("f travel"),
        "{}",
        s.view(&ctx).help
    );
}

/// The Find key again closes Find.
#[test]
fn the_find_key_again_closes_it() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    press(&mut s, &mut ctx, &[Action::WorldFind]);
    assert!(s.view(&ctx).list.is_some());
    press(&mut s, &mut ctx, &[Action::WorldFind]);
    assert!(s.view(&ctx).list.is_none());
}

/// A side quest not open yet says what it is in the panel.
#[test]
fn a_side_quest_says_so_in_the_panel() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    // The places in the map's order: home, camp, cross, farm, ford, pass.
    for _ in 0..5 {
        press(&mut s, &mut ctx, &[Action::NextUnit]);
    }
    assert_eq!(s.cursor(), "pass");
    let panel = s.view(&ctx).panel;
    assert_eq!(panel.kind, "Side quest");
    assert_eq!(panel.lines, ["Not open yet", "Enemy level 1"]);
}

/// The army walks one place every `STEP_S`: halfway, it is at the
/// crossroads, not yet at the farm.
#[test]
fn the_army_walks_one_place_a_step() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    press(
        &mut s,
        &mut ctx,
        &[Action::NextUnit, Action::NextUnit, Action::NextUnit],
    );
    assert_eq!(s.cursor(), "farm");
    press(&mut s, &mut ctx, &[Action::Confirm]);
    frame(&mut s, &mut ctx, &[], STEP_S * 1.1);
    assert!(s.walking());
    assert_eq!(s.view(&ctx).army, Pos::new(24, 16));
    frame(&mut s, &mut ctx, &[], STEP_S);
    assert!(!s.walking());
    assert_eq!(s.view(&ctx).army, Pos::new(18, 11));
}

#[test]
fn distance_counts_a_cell_down_as_twice_one_across() {
    assert_eq!(distance(Pos::new(0, 0), Pos::new(1, 2)), 8 + 32);
    assert_eq!(distance(Pos::new(3, 4), Pos::new(1, 1)), 16 + 48);
}

/// A map smaller than the screen's map area is drawn centred: the second
/// test act (40 × 12) starts at cell (15, 9).
#[test]
fn a_small_map_is_drawn_where_it_is_centred() {
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::glyph_buffer::Cell;
    let ctx = ctx();
    let mut c = army(&ctx);
    c.chapter = "test_wm3".into();
    c.world = None;
    c.enter_world(&ctx.game.content.worldmaps["test_world_2"].map);
    let view = WorldMapScreen::new(c, 0.0).view(&ctx);
    let black = crate::color::Rgb::new(0, 0, 0);
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, Cell::new(' ', black, black));
    glyph::paint(&ctx, &view, &mut buf);
    // Salt Keep at (30, 6), the army at Grey Harbor (6, 6), its flag above.
    assert_eq!(buf.get(45, 15).map(|c| c.glyph), Some('‼'));
    assert_eq!(buf.get(21, 15).map(|c| c.glyph), Some('M'));
    assert_eq!(buf.get(21, 14).map(|c| c.glyph), Some('▼'));
}

/// A road running down is wider than its cell: the ground's glyphs beside
/// it give way (a look test).
#[test]
fn the_ground_gives_way_beside_a_road_running_down() {
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::glyph_buffer::Cell;
    let ctx = ctx();
    let view = screen(&ctx).view(&ctx);
    let black = crate::color::Rgb::new(0, 0, 0);
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, Cell::new(' ', black, black));
    glyph::paint(&ctx, &view, &mut buf);
    let mut checked = 0;
    for pair in view.roads.iter().flat_map(|r| r.windows(2)) {
        let (a, b) = (pair[0], pair[1]);
        if a.x != b.x {
            continue;
        }
        for y in a.y.min(b.y) + 1..a.y.max(b.y) {
            for x in [a.x - 1, a.x + 1] {
                assert_eq!(buf.get(x, y).map(|c| c.glyph), Some(' '), "({x}, {y})");
                checked += 1;
            }
        }
    }
    assert!(checked > 20, "{checked}");
}

/// Confirm on another place sends the army there, one place at a time;
/// the campaign has it there at once.
#[test]
fn the_army_walks_to_where_it_is_sent() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    press(&mut s, &mut ctx, &[Action::CursorRight]);
    assert_eq!(s.cursor(), "cross");
    press(&mut s, &mut ctx, &[Action::Confirm]);
    assert!(s.walking());
    assert_eq!(at(&s), "cross");
    assert_eq!(s.view(&ctx).army, Pos::new(14, 20), "still at its start");
    let t = arrive(&mut s, &mut ctx);
    assert!(matches!(t, Transition::None));
    assert_eq!(s.view(&ctx).army, Pos::new(24, 16));
    assert_eq!(s.view(&ctx).panel.title, "Crossroads");
}

/// Confirm while walking finishes the walk at once.
#[test]
fn confirm_finishes_the_walk() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    press(&mut s, &mut ctx, &[Action::CursorRight, Action::Confirm]);
    press(&mut s, &mut ctx, &[Action::Confirm]);
    assert!(!s.walking());
    assert_eq!(s.view(&ctx).army, Pos::new(24, 16));
}

/// Arriving at a battle not yet won starts it (Nick: forced).
#[test]
fn arriving_at_a_battle_pops_with_it() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    press(
        &mut s,
        &mut ctx,
        &[Action::CursorRight, Action::CursorRight],
    );
    assert_eq!(s.cursor(), "ford");
    press(&mut s, &mut ctx, &[Action::Confirm]);
    let t = arrive(&mut s, &mut ctx);
    let battle: WorldBattle = popped_with(t).expect("a battle");
    assert_eq!(battle.node, "ford");
    let world = battle.campaign.world.expect("on the map");
    assert_eq!(world.at, "ford");
}

/// Confirm on a battle the army already stands at (a campaign loaded
/// there) starts it.
#[test]
fn confirm_on_an_unwon_battle_the_army_is_at_starts_it() {
    let mut ctx = ctx();
    let mut c = army(&ctx);
    c.world.as_mut().expect("on the map").at = "ford".into();
    let mut s = WorldMapScreen::new(c, 0.0);
    let t = press(&mut s, &mut ctx, &[Action::Confirm]);
    let battle: WorldBattle = popped_with(t).expect("a battle");
    assert_eq!(battle.node, "ford");
}

/// Arriving at a town opens it, with the setting on (the default); with
/// it off, only Confirm on it does.
#[test]
fn a_town_opens_on_arrival_unless_the_setting_is_off() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    press(&mut s, &mut ctx, &[Action::CursorDown, Action::Confirm]);
    arrive(&mut s, &mut ctx);
    let list = s.view(&ctx).list.expect("Kingsgate's shops");
    assert_eq!(list.title, "Kingsgate");
    let rows: Vec<String> = list
        .menu
        .items
        .iter()
        .map(|i| crate::widgets::menu::text_of(&i.label).to_string())
        .collect();
    assert_eq!(rows, ["Tailor", "Blacksmith"]);
    press(&mut s, &mut ctx, &[Action::Cancel]);
    assert!(s.view(&ctx).list.is_none());

    let mut ctx = crate::screen::tests::ctx();
    ctx.preview_settings(|s| s.game.towns = crate::settings::TownsOpen::OnConfirm);
    let mut s = screen(&ctx);
    press(&mut s, &mut ctx, &[Action::CursorDown, Action::Confirm]);
    arrive(&mut s, &mut ctx);
    assert!(s.view(&ctx).list.is_none());
    press(&mut s, &mut ctx, &[Action::Confirm]);
    assert_eq!(s.view(&ctx).list.map(|l| l.title), Some("Kingsgate".into()));
}

/// A town's shop opens over the map; the campaign comes back from it.
#[test]
fn a_shop_opens_from_a_town_and_its_campaign_comes_back() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    press(&mut s, &mut ctx, &[Action::Confirm]);
    let t = press(&mut s, &mut ctx, &[Action::CursorDown, Action::Confirm]);
    let Transition::Push(shop) = t else {
        panic!("no shop");
    };
    assert_eq!(shop.name(), "town_shop");
    let mut richer = s.campaign().clone();
    richer.gold = 1;
    s.uncovered(&mut ctx, Some(ScreenResult::new(richer)));
    assert_eq!(s.campaign().gold, 1);
    assert!(s.view(&ctx).list.is_none());
}

/// Cancel opens the menu: Save, Camp, Options; Save opens the slot picker.
#[test]
fn the_menu_saves_camps_and_opens_options() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    press(&mut s, &mut ctx, &[Action::Cancel]);
    let list = s.view(&ctx).list.expect("the menu");
    assert_eq!(list.menu.items.len(), 3);
    let t = press(&mut s, &mut ctx, &[Action::Confirm]);
    let Transition::Push(picker) = t else {
        panic!("no slot picker");
    };
    assert_eq!(picker.name(), "save_slots");
    press(&mut s, &mut ctx, &[Action::Cancel]);
    let t = press(&mut s, &mut ctx, &[Action::CursorDown, Action::Confirm]);
    assert!(matches!(t, Transition::Push(ref camp) if camp.name() == "camp"));
    press(&mut s, &mut ctx, &[Action::Cancel]);
    let t = press(&mut s, &mut ctx, &[Action::CursorUp, Action::Confirm]);
    assert!(matches!(t, Transition::Push(ref o) if o.name() == "options"));
}

/// Music: the map's track on the map, a town's own cue while it is open
/// (Nick: "while interacting with a town"), the map's again once it
/// closes, and after a screen over the map.
#[test]
fn a_towns_music_while_it_is_open_then_the_maps() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    press(&mut s, &mut ctx, &[]);
    let map = AudioRequest::play_music("world_map", None);
    assert_eq!(ctx.audio.take(), std::slice::from_ref(&map));
    // Confirm on Millbrook, where the army stands: its shops and its cue.
    press(&mut s, &mut ctx, &[Action::Confirm]);
    ctx.audio.take();
    press(&mut s, &mut ctx, &[]);
    let village = AudioRequest::play_music("village", None);
    assert_eq!(ctx.audio.take(), [village]);
    press(&mut s, &mut ctx, &[Action::Cancel]);
    ctx.audio.take();
    press(&mut s, &mut ctx, &[]);
    assert_eq!(ctx.audio.take(), std::slice::from_ref(&map));
    s.uncovered(&mut ctx, None);
    frame(&mut s, &mut ctx, &[], 0.01);
    assert_eq!(ctx.audio.take(), [map]);
}

/// The map's track plays to its end, fades, the map is quiet 30 s, then
/// it starts again (`audio.md`, *The world map*).
#[test]
fn the_maps_track_rests_between_rounds() {
    let mut ctx = ctx();
    let mut s = screen(&ctx);
    press(&mut s, &mut ctx, &[Action::CursorUp, Action::Confirm]);
    arrive(&mut s, &mut ctx);
    ctx.audio.take();
    let manifest = &ctx.game.content.audio;
    let length = f64::from(manifest.music["world_map"].length_ms) / 1000.0;
    let fade = f64::from(manifest.music_fade_ms) / 1000.0;
    frame(&mut s, &mut ctx, &[], length - fade);
    assert_eq!(ctx.audio.take(), [AudioRequest::StopMusic]);
    frame(&mut s, &mut ctx, &[], fade + QUIET_S - 1.0);
    assert_eq!(ctx.audio.take(), []);
    frame(&mut s, &mut ctx, &[], 1.0);
    let again = AudioRequest::play_music("world_map", None);
    assert_eq!(ctx.audio.take(), [again]);
}

/// The glyph look: each place's plate and the army's in pixels under the
/// glyphs, and the roads as bands (a look test: it reads overlays).
#[test]
fn the_glyph_look_draws_plates_and_road_bands() {
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::glyph_buffer::{Cell, Layer};
    let ctx = ctx();
    let view = screen(&ctx).view(&ctx);
    let mut buf = GlyphBuffer::with_spec(
        ConsoleSpec::PC,
        Cell::new(
            ' ',
            crate::color::Rgb::new(0, 0, 0),
            crate::color::Rgb::new(0, 0, 0),
        ),
    );
    glyph::paint(&ctx, &view, &mut buf);
    let story = ctx.palette.lookup("text_highlight").expect("a color");
    let under: Vec<_> = buf
        .overlays()
        .into_iter()
        .filter(|o| o.layer == Layer::Under)
        .collect();
    // Stone Mill's plate: 16 × 16, centred on its cell (34, 13).
    let plate = under.iter().find(|o| o.color == story).expect("a plate");
    assert_eq!(
        (plate.rect.x, plate.rect.y, plate.rect.w, plate.rect.h),
        (268, 208, 16, 16)
    );
    let symbol = buf.get(34, 13).map(|c| c.glyph);
    assert_eq!(symbol, Some('‼'));
    assert_eq!(buf.get(33, 12).map(|c| c.glyph), Some('L'), "Lv1 above");
    // The army's flag.
    assert_eq!(buf.get(14, 19).map(|c| c.glyph), Some('▼'));
}

/// The picture look, for a map with a picture: the picture under the
/// places, stretched over the map; the roads as dots; the same plates;
/// no flag (Nick). A look test: it reads the sprites and overlays.
#[test]
fn the_picture_look_draws_the_maps_picture_dotted_roads_and_no_flag() {
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::glyph_buffer::{Cell, Layer};
    let mut ctx = ctx();
    let def = ctx
        .game
        .content
        .worldmaps
        .get_mut("test_world")
        .expect("a map");
    def.picture = Some("images/test_card.png".into());
    let view = screen(&ctx).view(&ctx);
    let black = crate::color::Rgb::new(0, 0, 0);
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, Cell::new(' ', black, black));
    picture::paint(&ctx, &view, "images/test_card.png", &mut buf);
    let sprites = buf.sprites();
    let card = ctx
        .game
        .content
        .images
        .id("images/test_card.png")
        .expect("the card");
    let ground = sprites
        .iter()
        .find(|s| s.image == card)
        .expect("the picture");
    assert_eq!(
        (ground.dest.x, ground.dest.y, ground.dest.w, ground.dest.h),
        (0, 0, 560, 480)
    );
    assert_eq!(ground.layer, Layer::Under);
    // Dots: 4 × 4 squares in the path colour.
    let path = ctx.palette.get(UiColor::Path);
    let dots = buf
        .overlays()
        .into_iter()
        .filter(|o| o.color == path && o.rect.w == 4 && o.rect.h == 4)
        .count();
    assert!(dots > 20, "{dots} dots");
    // Stone Mill's plate, and no flag over the army.
    let story = ctx.palette.lookup("text_highlight").expect("a color");
    assert!(
        buf.overlays()
            .iter()
            .any(|o| o.color == story && o.rect.w == 16)
    );
    assert_eq!(buf.get(34, 13).map(|c| c.glyph), Some('‼'));
    assert_ne!(buf.get(14, 19).map(|c| c.glyph), Some('▼'));
}

/// The screen paints with pictures only when the player's look is
/// pictures and the map has a picture.
#[test]
fn pictures_need_the_look_and_a_picture() {
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::glyph_buffer::Cell;
    let mut ctx = ctx();
    let black = crate::color::Rgb::new(0, 0, 0);
    let painted = |ctx: &Ctx| {
        let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, Cell::new(' ', black, black));
        screen(ctx).draw(ctx, &mut buf);
        buf.sprites().len()
    };
    ctx.preview_settings(|s| s.game.map_look = MapLook::Pictures);
    assert_eq!(painted(&ctx), 0, "no picture: glyphs");
    let def = ctx
        .game
        .content
        .worldmaps
        .get_mut("test_world")
        .expect("a map");
    def.picture = Some("images/test_card.png".into());
    assert!(painted(&ctx) > 0);
    ctx.preview_settings(|s| s.game.map_look = MapLook::Glyphs);
    assert_eq!(painted(&ctx), 0, "glyphs chosen");
}
