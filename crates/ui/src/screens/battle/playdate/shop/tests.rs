//! The shop's one-bit look (ticket 1066), in the Playdate's palette on
//! its 50 × 15 screen.

use insta::assert_snapshot;
use trpg_content::Ink;

use super::*;
use crate::screens::battle::view::ShopLineView;
use crate::screens::playdate::tests::{blank, c, cell, find, is_bar, pd_ctx, row, rows};
use crate::widgets::{Ask, AskView};

/// A view of `n` lines named `Item 0`, `Item 1`, …, the second one
/// can't be chosen, line `focus` highlighted, on the Sell tab.
fn view(n: usize, focus: usize) -> ShopScreenView {
    let line = |i: usize| ShopLineView {
        name: format!("Item {i}"),
        gold: "1 250".into(),
        detail: format!("Mt {i}  Hit 90"),
        enabled: i != 1,
    };
    ShopScreenView {
        shop: "Armory".into(),
        gold: "Gold 2 000".into(),
        unit: "Millbrook".into(),
        tabs: vec!["Buy".into(), "Sell".into()],
        tab: 1,
        lines: (0..n).map(line).collect(),
        focus,
        empty: "Nothing here.".into(),
        selling: None,
        special: None,
        help: "↑ ↓ choose · ← → tab · A confirm · B leave".into(),
        debug_hint: None,
    }
}

fn painted(view: &ShopScreenView) -> GlyphBuffer {
    let mut buf = blank();
    paint(&pd_ctx(), view, &mut buf);
    buf
}

/// The whole screen: the shop and the gold in the frame, the town and
/// the tabs, the lines between two rules, what the focused one is, the
/// help on the bottom row.
#[test]
fn the_shop_on_the_playdate() {
    let buf = painted(&view(3, 2));
    assert_snapshot!("playdate_shop", rows(&buf).join("\n"));
    // The shown tab is the bar; the focused line a bar across the window.
    assert!(is_bar(&buf, TABS, "Sell"));
    assert!(!is_bar(&buf, TABS, "Buy"));
    assert!(is_bar(&buf, FIRST + 2, "Item 2"));
    assert_eq!(cell(&buf, 1, FIRST + 2).bg, c(Ink::Black));
    assert_eq!(cell(&buf, 48, FIRST + 2).bg, c(Ink::Black));
    // A line that can't be chosen is grey dots.
    let off = find(&buf, FIRST + 1, "Item 1").expect("the line");
    assert_eq!(cell(&buf, off, FIRST + 1).fg, c(Ink::Grey));
    // The gold right-aligned by the frame.
    assert!(row(&buf, FIRST).ends_with("1 250 ║"));
    assert_eq!(
        row(&buf, DETAIL),
        "║ Mt 2  Hit 90                                   ║"
    );
}

/// A tab with no lines says so in grey dots; nothing is under the rule.
#[test]
fn an_empty_tab_says_so() {
    let buf = painted(&view(0, 0));
    assert_eq!(find(&buf, FIRST, "Nothing here."), Some(2));
    assert_eq!(cell(&buf, 2, FIRST).fg, c(Ink::Grey));
    assert_eq!(
        row(&buf, DETAIL),
        "║                                                ║"
    );
}

/// A long list scrolls with its focus, kept in the middle; the rules
/// say there are more above or below.
#[test]
fn a_long_list_scrolls_and_marks_more() {
    let top = painted(&view(20, 0));
    assert!(row(&top, RULE_1).ends_with("─╢"));
    assert!(row(&top, RULE_2).ends_with("▼─╢"));
    assert_eq!(find(&top, RULE_2 - 1, "Item 7"), Some(2));
    let middle = painted(&view(20, 10));
    assert!(row(&middle, RULE_1).ends_with("▲─╢"));
    assert!(row(&middle, RULE_2).ends_with("▼─╢"));
    assert!(is_bar(&middle, FIRST + 4, "Item 10"));
    let end = painted(&view(20, 19));
    assert!(row(&end, RULE_1).ends_with("▲─╢"));
    assert!(row(&end, RULE_2).ends_with("─╢"));
    assert!(is_bar(&end, RULE_2 - 1, "Item 19"));
}

/// A sale asked about: a window in the middle, the count between ▲ and
/// ▼, an arrow in grey dots at its end.
#[test]
fn a_sale_counts_between_arrows_grey_at_the_ends() {
    let arrows = |count: usize| {
        let mut v = view(1, 0);
        v.selling = Some(SellConfirmView {
            question: "Sell Potion?".into(),
            count,
            max: 3,
            gold: "For 150 gold".into(),
        });
        let buf = painted(&v);
        let x = find(&buf, 6, "▲").expect("▲");
        assert_eq!(find(&buf, 7, &format!("{count} / 3")), Some(x - 2));
        (cell(&buf, x, 6).fg, cell(&buf, x, 8).fg)
    };
    let (black, grey) = (c(Ink::Black), c(Ink::Grey));
    assert_eq!(arrows(1), (black, grey));
    assert_eq!(arrows(2), (black, black));
    assert_eq!(arrows(3), (grey, black));
    let mut v = view(1, 0);
    v.selling = Some(SellConfirmView {
        question: "Sell Potion?".into(),
        count: 1,
        max: 3,
        gold: "For 150 gold".into(),
    });
    let buf = painted(&v);
    // Seven rows tall in the middle of the screen's fifteen.
    assert_eq!(find(&buf, 4, "╔═══"), Some(17));
    assert_eq!(find(&buf, 5, "Sell Potion?"), Some(19));
    assert_eq!(find(&buf, 9, "For 150 gold"), Some(19));
    assert_eq!(find(&buf, 10, "╚═══"), Some(17));
}

/// The question before selling a special weapon: the shared question
/// window, its answers on one line.
#[test]
fn selling_a_special_weapon_asks_in_the_question_window() {
    let ctx = pd_ctx();
    let ask = Ask::new(
        "battle.shop.no",
        [(false, "battle.shop.no"), (true, "battle.shop.yes")],
    );
    let special: AskView<bool> = ask.view(&ctx);
    let mut v = view(1, 0);
    v.special = Some(special);
    let buf = painted(&v);
    let all = rows(&buf).join("\n");
    assert!(all.contains("No    Yes"), "{all}");
}

/// Eight lines fill the list with nothing below; a ninth puts `▼` on
/// the rule.
#[test]
fn more_below_shows_from_the_ninth_line() {
    let eight = painted(&view(8, 0));
    assert!(row(&eight, RULE_2).ends_with("──╢"));
    let nine = painted(&view(9, 0));
    assert!(row(&nine, RULE_2).ends_with("▼─╢"));
}

/// A sale's question too long for the screen is cut inside its window.
#[test]
fn a_long_sale_question_is_cut_inside_its_window() {
    let mut v = view(1, 0);
    v.selling = Some(SellConfirmView {
        question: "Sell a weapon with a name far too long to fit on the screen?".into(),
        count: 1,
        max: 1,
        gold: "For 1 gold".into(),
    });
    let buf = painted(&v);
    assert_eq!(
        row(&buf, 5),
        "║Sell a weapon with a name far too long to fit o…║"
    );
}
