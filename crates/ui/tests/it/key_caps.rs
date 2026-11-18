//! Key caps in the game's hints (ticket 0254, `docs/design/look-and-feel.md`,
//! *Key names in hints*): each switched view holds its keys in key spans,
//! in the order `key_name` and its siblings name them; the frame draws
//! them as caps, the arrow keys as their picture; a controller's buttons
//! read as before (0255).

use insta::assert_snapshot;
use shuyi_ui::text::{StyledText, TextRole};
use trpg_content::image::KEY_ARROWS_PATH;
use trpg_ui::ShuyiCtx;
use trpg_ui::harness::Harness;
use trpg_ui::input::{Action, LEFT_HANDED, PadKind, RIGHT_HANDED};
use trpg_ui::screens::battle::BattleScreen;
use trpg_ui::screens::battle::layout::{HELP_BAR, MAP_VIEW};
use trpg_ui::screens::dialogue::DialogueScreen;
use trpg_ui::widgets::help::{cursor_keys_name, key_name};

/// The keys of `text`, in order: `!` for the arrows picture.
fn keys(text: &StyledText) -> Vec<String> {
    let key = |s: &&shuyi_ui::text::TextSpan| s.role.is_some();
    let name = |s: &shuyi_ui::text::TextSpan| match s.role {
        Some(TextRole::Arrows) => "!".to_owned(),
        _ => s.text.replace('\u{a0}', " "),
    };
    text.spans().iter().filter(key).map(name).collect()
}

/// The battle, its objective and phase banners closed, with key caps
/// and the tips on, in `layout`.
fn battle_in(layout: trpg_ui::input::Layout, tips: bool) -> Harness {
    let mut h = Harness::with_layout(layout);
    h.with_key_caps();
    if tips {
        h.with_tips();
    }
    // Quick Battle, then Preparations: Left wraps to `Fight!`; the
    // objective and the phase banner closed (a tip then opens).
    let (down, left, confirm) = if layout == LEFT_HANDED {
        ("s", "a", "j")
    } else {
        ("Down", "Left", "f")
    };
    h.keys(&format!(
        "{down} {confirm} {left} {confirm} {confirm} {confirm}"
    ));
    h.wait(0.5);
    h
}

fn battle(h: &Harness) -> &BattleScreen {
    h.screen::<BattleScreen>()
        .unwrap_or_else(|| panic!("no battle: {:?}", h.screens()))
}

/// Acceptance (0254): the battle's help bar and toggles hold each key the
/// keymap names, in order, as a key span.
#[test]
fn the_battle_help_bar_holds_the_keys_key_name_gives_in_order() {
    let h = battle_in(RIGHT_HANDED, false);
    let ctx = h.game().ctx();
    let view = battle(&h).view(ctx);
    let km = ctx.help_keys();
    let name = |a| key_name(km, a);
    // Over the ready lord: select (Confirm's key while Select has none),
    // info, next unit, rewind, back, end turn.
    assert_eq!(
        keys(&view.help),
        [
            name(Action::Confirm),
            name(Action::Info),
            name(Action::NextUnit),
            name(Action::Rewind),
            name(Action::Cancel),
            name(Action::EndTurn),
        ],
        "{}",
        view.help.plain()
    );
    let status = toggles(&view);
    assert_eq!(
        keys(&status),
        [name(Action::DangerZone), name(Action::ToggleAutoEnd)],
        "{}",
        status.plain()
    );
}

/// The toggles' row of `view` (ticket 0445) as one line: the left hint,
/// then auto-end; empty while it is hidden.
fn toggles(view: &trpg_ui::screens::battle::view::BattleView) -> StyledText {
    let Some(row) = view.toggles.as_ref() else {
        return StyledText::new();
    };
    let mut line = row.left.clone();
    if let Some((auto_end, _)) = &row.auto_end {
        line.push_plain(" · ").append(auto_end);
    }
    line
}

/// The help bar drawn: bold capitals in boxes (a picture to review).
#[test]
fn the_battle_help_bar_draws_its_keys_as_caps() {
    let h = battle_in(RIGHT_HANDED, false);
    let snap = h.snapshot_whole();
    // `f select` as a cap `F` and the room after it.
    assert!(snap.contains("Space  end turn"), "{snap}");
    // Auto-end starts where end turn does, counting the caps' cells
    // (ticket 0445).
    let rows: Vec<&str> = snap.lines().collect();
    let column = |row: &str, text: &str| row.find(text).map(|at| row[..at].chars().count());
    let toggles = rows
        .iter()
        .find(|r| r.contains("auto-end:"))
        .expect("the toggles' row");
    let help = rows
        .iter()
        .find(|r| r.contains("end turn"))
        .expect("the help row");
    assert_eq!(
        column(toggles, "Shift+Space"),
        column(help, "Space  end turn"),
        "{snap}"
    );
    assert_snapshot!(h.snapshot_of(HELP_BAR));
}

/// The text box's corner: Confirm's key and the log's, as caps.
#[test]
fn the_text_box_corner_draws_its_keys_as_caps() {
    let ctx = trpg_ui::Ctx::embedded()
        .unwrap_or_else(|e| panic!("{e}"))
        .with_layout(RIGHT_HANDED);
    let game = trpg_ui::game::start_on_scene(ctx, "test", trpg_core::LeadGender::Male)
        .unwrap_or_else(|e| panic!("{e}"));
    let mut h = Harness::from_game(game);
    h.wait(10.0);
    let dialogue = h.screen::<DialogueScreen>().expect("the scene");
    let ctx = h.game().ctx();
    let wait = dialogue.view(ctx).text.waiting.expect("waiting");
    let km = ctx.help_keys();
    assert_eq!(keys(&wait.key), [key_name(km, Action::Confirm)]);
    let log = wait.log_hint.expect("the log's hint");
    assert_eq!(keys(&log), [key_name(km, Action::DialogueLog)]);
    let snap = h.snapshot();
    assert!(snap.contains("B  log   F  ▼"), "{snap}");
    assert_snapshot!(snap);
}

/// A tip: the arrow keys as one picture of the arrow cluster, each other
/// key as a cap.
#[test]
fn a_tip_draws_the_arrow_keys_as_their_picture() {
    let h = battle_in(RIGHT_HANDED, true);
    let ctx = h.game().ctx();
    let tip = battle(&h).view(ctx).tip.expect("the first tip");
    let how: Vec<String> = tip.how.iter().flat_map(keys).collect();
    let km = ctx.help_keys();
    // `Move the cursor: arrows. Pick up a unit: f`, then next and info.
    let name = |a| key_name(km, a);
    let expected = [
        "!".to_owned(),
        name(Action::Confirm),
        name(Action::NextUnit),
        name(Action::Info),
    ];
    assert_eq!(how, expected, "{:?}", tip.how);
    assert_eq!(keys(&tip.close), [key_name(km, Action::Confirm)]);
    let arrows = h.game().buffer().sprites();
    assert!(
        arrows.iter().any(|s| s.image.path() == KEY_ARROWS_PATH),
        "no arrow cluster among {arrows:?}"
    );
    assert_snapshot!(h.snapshot_of(MAP_VIEW));
}

/// Acceptance (0254): with the cursor on W A S D, a tip names it as four
/// key spans, up, left, down and right.
#[test]
fn a_cursor_on_w_a_s_d_is_four_key_spans_in_a_tip() {
    let h = battle_in(LEFT_HANDED, true);
    let ctx = h.game().ctx();
    let tip = battle(&h).view(ctx).tip.expect("the first tip");
    let km = ctx.help_keys();
    assert_eq!(cursor_keys_name(km), "w a s d");
    let how: Vec<String> = tip.how.iter().flat_map(keys).collect();
    assert_eq!(how[..4], ["w", "a", "s", "d"], "{how:?}");
    assert_snapshot!(h.snapshot_of(MAP_VIEW));
}

/// Acceptance (0254, 0255): with a controller in use, a hint names its
/// buttons as plain text, as before key caps.
#[test]
fn a_controller_s_buttons_read_as_text_with_no_caps() {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    h.with_key_caps().with_tips();
    h.use_pad(PadKind::Xbox)
        .pad("DpadDown South DpadLeft South South South");
    h.wait(0.5);
    let ctx = h.game().ctx();
    let view = battle(&h).view(ctx);
    let tip = view.tip.clone().expect("the first tip");
    assert_eq!(keys(&tip.close), [] as [String; 0]);
    assert_eq!(tip.close.plain(), "A close");
    assert!(tip.how.iter().all(|l| keys(l).is_empty()));
    assert_eq!(keys(&view.help), [] as [String; 0]);
    assert_eq!(keys(&toggles(&view)), [] as [String; 0]);
}

/// Options, an engine screen, draws its keys as caps too.
#[test]
fn options_draws_its_keys_as_caps() {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    h.with_key_caps();
    // The title's menu: Options is its third entry.
    h.keys("Down Down f");
    assert_eq!(h.screens(), ["title", "options"]);
    // The help line alone, the bottom row: the look snapshot of the help
    // line every screen but the battle's draws there (1290).
    assert_snapshot!(h.snapshot_of(trpg_ui::glyph_buffer::Rect::new(0, 31, 100, 1)));
}
