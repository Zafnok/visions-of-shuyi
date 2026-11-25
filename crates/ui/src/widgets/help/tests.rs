//! Hints in key spans (ticket 0254): key caps on the keyboard, plain text
//! on a controller and for a key not mapped; the theme's style; wrapping.

use shuyi_ui::GameData;
use trpg_content::{Chord, RepeatDef};

use super::*;
use crate::input::{Keymap, PadKind};
use crate::screen::tests::ctx;

fn keymap(pairs: &[(&str, Action)]) -> Keymap {
    Keymap::new(
        pairs.iter().map(|&(c, a)| (Chord::parse(c).unwrap(), a)),
        RepeatDef::default(),
    )
}

/// Each span of `text` as `(text, is a key)`, `!` for the arrows picture.
fn spans(text: &StyledText) -> Vec<(String, char)> {
    let kind = |role| match role {
        Some(TextRole::Key) => 'k',
        Some(TextRole::Arrows) => '!',
        _ => ' ',
    };
    text.spans()
        .iter()
        .map(|s| (s.text.clone(), kind(s.role)))
        .collect()
}

/// The keys of `text`, in order.
fn keys(text: &StyledText) -> Vec<String> {
    let key = |s: &&shuyi_ui::text::TextSpan| s.role.is_some();
    text.spans()
        .iter()
        .filter(key)
        .map(|s| s.text.clone())
        .collect()
}

#[test]
fn a_key_is_a_key_span_on_the_keyboard_and_plain_on_a_controller() {
    let km = keymap(&[("f", Action::Confirm), ("d", Action::Cancel)]);
    let keyboard = HelpKeys::keyboard(&km);
    assert_eq!(
        spans(&action_key(keyboard, Action::Confirm)),
        [("f".into(), 'k')]
    );
    let pad = HelpKeys::new(&km, Device::Pad(PadKind::Xbox));
    let named = key_name(pad, Action::Confirm);
    assert_eq!(spans(&action_key(pad, Action::Confirm)), [(named, ' ')]);
    // A key not mapped reads as text, with no cap.
    let rewind = action_key(keyboard, Action::Rewind);
    assert_eq!(spans(&rewind), [(NOT_MAPPED.to_owned(), ' ')]);
}

#[test]
fn a_help_line_and_a_pair_of_keys_hold_each_key_in_its_own_span() {
    let km = keymap(&[
        ("f", Action::Confirm),
        ("d", Action::Cancel),
        ("w", Action::CursorUp),
        ("s", Action::CursorDown),
    ]);
    let keyboard = HelpKeys::keyboard(&km);
    let pair = key_pair(keyboard, Action::CursorUp, Action::CursorDown);
    assert_eq!(
        spans(&pair),
        [("w".into(), 'k'), ("/".into(), ' '), ("s".into(), 'k')]
    );
    let line = help_line_styled(
        keyboard,
        &[
            (Some("f".into()), "pick"),
            (None, "hidden"),
            (Some("d".into()), "back"),
        ],
    );
    assert_eq!(line.plain(), "f pick · d back");
    assert_eq!(keys(&line), ["f", "d"]);
    let joined = join_hints([pair, line]);
    assert_eq!(joined.plain(), "w/s · f pick · d back");
    assert_eq!(keys(&joined), ["w", "s", "f", "d"]);
}

#[test]
fn the_theme_turns_on_bold_capital_caps_with_the_arrow_cluster() {
    let c = ctx();
    let style = c.game.key_caps().expect("key caps on");
    assert!(style.bold && style.capitals && !style.one_bit);
    assert_eq!(style.margin_px, 2);
    let (image, src) = style.arrows.expect("the arrow cluster");
    assert_eq!(image.path(), trpg_content::image::KEY_ARROWS_PATH);
    assert_eq!((src.x, src.y, src.w, src.h), (0, 0, 24, 16));
    // The frame draws them: three cells for the arrows, room for a cap.
    let caps = c.key_caps().expect("on the PC");
    assert_eq!(caps.arrows_cells(), Some(3));
    assert!(caps.extra_cells() >= 1);
    // The default theme has the mockup's colours (tunable).
    let rgb = shuyi_ui::frame::Rgb::new;
    let (fill, edge) = (rgb(0x4a, 0x40, 0x2e), rgb(0x80, 0x70, 0x52));
    assert_eq!(caps.colors(), (Some(fill), Some(edge)));
    assert_eq!(c.palette.lookup("key"), Some(rgb(0xd0, 0xc4, 0xa8)));
}

#[test]
fn the_frame_measures_a_key_as_its_cap_and_the_arrows_as_their_picture() {
    let mut c = ctx();
    let mut text = StyledText::new();
    text.push_key("f")
        .push_plain(" pick · ")
        .push_arrows("arrows");
    let caps = c.key_caps().expect("on the PC");
    let cap = 1 + usize::from(caps.extra_cells());
    let arrows = usize::from(caps.arrows_cells().expect("the picture"));
    assert_eq!(text.width(&frame_widths(&c)), cap + 8 + arrows);
    // With no caps, the words' own cells.
    c.game.key_caps = false;
    assert_eq!(text.width(&frame_widths(&c)), text.width(c.widths()));
}

#[test]
fn a_placeholder_fills_in_key_spans_and_the_arrow_keys_as_one() {
    let mut c = ctx();
    let filled = |c: &Ctx| crate::tips::fill_styled(c, "{Cursor} move · {Confirm} pick");
    assert_eq!(
        spans(&filled(&c)),
        [
            ("arrows".into(), '!'),
            (" move · ".into(), ' '),
            ("f".into(), 'k'),
            (" pick".into(), ' '),
        ]
    );
    // The left-handed keys: the cursor as four caps, a space between.
    c.use_layout(crate::input::LEFT_HANDED);
    let wasd = filled(&c);
    assert_eq!(keys(&wasd), ["w", "a", "s", "d", "j"]);
    // On a controller, all plain.
    c.device = Device::Pad(PadKind::Xbox);
    assert_eq!(keys(&filled(&c)), [] as [String; 0]);
}

#[test]
fn text_with_keys_puts_the_callers_keys_in_as_spans() {
    let c = ctx();
    let km = c.help_keys();
    let pair = key_pair(km, Action::CursorLeft, Action::CursorRight);
    let swap = text_with_keys(&c, "battle.hint.swap", &[("keys", &pair)]);
    assert_eq!(swap.plain(), "Left/Right swap");
    assert_eq!(keys(&swap), ["Left", "Right"]);
}

#[test]
fn styled_text_wraps_at_plain_spaces_with_a_key_and_what_follows_it_whole() {
    let mut text = StyledText::new();
    text.push_plain("Press ")
        .push_key("f")
        .push_plain(", then ")
        .push_key("Shift+Space")
        .push_plain(" to end.");
    let narrow = CellWidths::NARROW;
    let rows = wrap_styled(&text, 12, &narrow);
    let plain: Vec<String> = rows.iter().map(StyledText::plain).collect();
    assert_eq!(plain, ["Press f,", "then", "Shift+Space", "to end."]);
    assert_eq!(keys(&rows[0]), ["f"]);
    assert_eq!(spans(&rows[0])[2], (",".into(), ' '));
    // A cap's extra cells count: with two more a key, `Press f,` is 10.
    let capped = narrow.clone().with_key_cap_cells(Some(2));
    let rows = wrap_styled(&text, 9, &capped);
    assert_eq!(rows[0].plain(), "Press");
    assert_eq!(
        wrap_styled(&StyledText::new(), 9, &narrow),
        [] as [StyledText; 0]
    );
}

#[test]
fn styled_chinese_wraps_between_characters_in_cells_with_a_key_whole() {
    let mut text = StyledText::new();
    text.push_plain("按")
        .push_key("f")
        .push_plain("键确认，然后结束。");
    let wide = CellWidths::wide("按键确认，然后结束。".chars());
    let rows = wrap_styled(&text, 6, &wide);
    let plain: Vec<String> = rows.iter().map(StyledText::plain).collect();
    // No row starts with `，` or `。`; each is at most 6 cells.
    assert_eq!(plain, ["按f键", "确认，", "然后结", "束。"]);
    assert_eq!(keys(&rows[0]), ["f"]);
    // A key wider than the row has a row of its own.
    let mut long = StyledText::new();
    long.push_plain("a ")
        .push_key("Shift+Space")
        .push_plain(" b");
    let plain: Vec<String> = wrap_styled(&long, 5, &wide)
        .iter()
        .map(StyledText::plain)
        .collect();
    assert_eq!(plain, ["a", "Shift+Space", "b"]);
}

#[test]
fn styled_lines_split_at_line_breaks_keeping_roles() {
    let mut text = StyledText::new();
    text.push_plain("a: ")
        .push_key("f")
        .push_plain("\nb: ")
        .push_key("d");
    let lines = styled_lines(&text);
    assert_eq!(lines.len(), 2);
    assert_eq!(spans(&lines[1]), [("b: ".into(), ' '), ("d".into(), 'k')]);
}

/// `words` as plain hints.
fn hints(words: &[&str]) -> Vec<StyledText> {
    words.iter().map(|w| StyledText::from(*w)).collect()
}

const NARROW: CellWidths = CellWidths::NARROW;

#[test]
fn hint_column_counts_the_hints_and_separators_before() {
    let h = hints(&["f select", "e info", "Space end turn"]);
    assert_eq!(hint_column(&h, 0, &NARROW), 0);
    assert_eq!(hint_column(&h, 1, &NARROW), 11);
    let line = join_hints(h.iter().cloned()).plain();
    let at = line.find("Space").unwrap();
    assert_eq!(hint_column(&h, 2, &NARROW), line[..at].chars().count());
}

#[test]
fn hint_column_counts_cells_not_bytes() {
    // `→` and `×` take three and two bytes, one cell each.
    let h = hints(&["q zoom: 2×", "a → b", "end"]);
    assert_eq!(hint_column(&h, 2, &NARROW), 10 + 3 + 5 + 3);
}

#[test]
fn hint_column_counts_a_key_caps_extra_cells() {
    let mut select = StyledText::new();
    select.push_key("f").push_plain(" select");
    let h = vec![select, StyledText::from("end")];
    let capped = NARROW.clone().with_key_cap_cells(Some(2));
    assert_eq!(hint_column(&h, 1, &NARROW), 8 + 3);
    assert_eq!(hint_column(&h, 1, &capped), 8 + 2 + 3);
}

#[test]
fn hint_column_is_not_fooled_by_a_label_with_the_same_words() {
    let h = hints(&["x end turn soon", "Space end turn"]);
    assert_eq!(hint_column(&h, 1, &NARROW), 18);
}

#[test]
fn pad_last_hint_moves_the_last_hint_to_the_column() {
    let h = hints(&["f select", "Space end turn"]);
    let line = pad_last_hint(&h, 20, &NARROW).plain();
    assert_eq!(line, format!("f select{}Space end turn", " ".repeat(12)));
    assert_eq!(line.find("Space"), Some(20));
}

#[test]
fn pad_last_hint_never_pulls_a_hint_left() {
    let h = hints(&["f select", "Space end turn"]);
    let joined = join_hints(h.iter().cloned());
    assert_eq!(pad_last_hint(&h, 5, &NARROW), joined);
    assert_eq!(pad_last_hint(&h, 11, &NARROW), joined);
    assert_eq!(pad_last_hint(&hints(&["one"]), 9, &NARROW).plain(), "one");
    assert_eq!(pad_last_hint(&[], 9, &NARROW), StyledText::new());
}

/// `spans_of` names the cursor keys only for `Cursor`: any other name that
/// is no action is a mistake in the test that asks (ticket 0257).
#[test]
#[should_panic(expected = "no action")]
fn spans_of_a_name_that_is_no_action_panics() {
    let c = ctx();
    let _ = spans_of(c.help_keys(), &["NoSuchAction"]);
}

/// A help bar (0257): its shown hints in order with the separator, a
/// hidden one left out with no separator of its own.
#[test]
fn a_help_bar_joins_its_shown_hints_and_leaves_out_the_hidden() {
    let c = ctx();
    let bar = HelpBar::new()
        .hint("hint.choose")
        .hint("hint.load")
        .hint_if("hint.open_folder", false)
        .hint("hint.back");
    let line = bar.line(&c);
    assert_eq!(line.plain(), "arrows choose · f load · d back");
    assert_eq!(keys(&line), ["arrows", "f", "d"]);
    let hidden: Vec<_> = bar.hints().iter().filter(|(_, s)| !s).collect();
    assert_eq!(hidden, [&("hint.open_folder", false)]);
    assert_eq!(HelpBar::new().line(&c), StyledText::new());
}
