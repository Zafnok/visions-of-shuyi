use trpg_content::CreditEntry;

use super::*;
use crate::audio::AudioRequest;
use crate::screen::tests::ctx;

pub(super) fn entry(group: CreditGroup, title: &str) -> CreditEntry {
    CreditEntry {
        group,
        title: title.into(),
        author: "Ann".into(),
        license: "CC0-1.0".into(),
        source: format!("https://example.org/{title}"),
    }
}

/// A context whose credits are `entries`.
pub(super) fn ctx_with(entries: Vec<CreditEntry>) -> Ctx {
    let mut c = ctx();
    c.content.credits = Credits { entries };
    c
}

fn texts(s: &CreditsScreen) -> Vec<&str> {
    s.rows.iter().map(|(_, t)| t.as_str()).collect()
}

pub(super) fn input(actions: &[Action]) -> FrameInput {
    FrameInput::new(actions.to_vec(), 0.0, vec![])
}

/// The sounds an update with `actions` plays, as cue names.
fn sounds_of(s: &mut CreditsScreen, c: &mut Ctx, actions: &[Action]) -> Vec<String> {
    s.update(c, &input(actions));
    c.audio
        .take()
        .iter()
        .filter(|r| matches!(r, AudioRequest::PlaySound { .. }))
        .filter_map(|r| r.cue().map(str::to_owned))
        .collect()
}

#[test]
fn rows_group_the_entries_under_headings() {
    use CreditRowKind::{Detail, Entry, Heading};
    let c = ctx_with(vec![
        entry(CreditGroup::Music, "One"),
        entry(CreditGroup::Music, "Two"),
        entry(CreditGroup::Fonts, "Font"),
    ]);
    let s = CreditsScreen::new(&c);
    assert_eq!(
        texts(&s),
        [
            "Music",
            "",
            "\"One\" by Ann",
            "CC0-1.0 · https://example.org/One",
            "",
            "\"Two\" by Ann",
            "CC0-1.0 · https://example.org/Two",
            "",
            "Fonts",
            "",
            "\"Font\" by Ann",
            "CC0-1.0 · https://example.org/Font",
        ]
    );
    let kinds: Vec<CreditRowKind> = s.rows.iter().map(|&(k, _)| k).collect();
    assert_eq!(
        kinds,
        [
            Heading, Heading, Entry, Detail, Entry, Entry, Detail, Entry, Heading, Heading, Entry,
            Detail
        ]
    );
    assert!(CreditsScreen::new(&ctx_with(vec![])).rows.is_empty());
}

#[test]
fn every_group_has_its_heading() {
    let c = ctx();
    let headings: Vec<&str> = CreditGroup::ALL
        .iter()
        .map(|&g| c.text(heading(g)))
        .collect();
    assert_eq!(
        headings,
        ["Music", "Sound effects", "Art", "Fonts", "Software"]
    );
}

/// 20 entries: 2 + 20 × 3 − 1 = 61 rows, 36 more than fit.
pub(super) fn long_list() -> Ctx {
    ctx_with(
        (0..20)
            .map(|i| entry(CreditGroup::Music, &format!("Track{i:02}")))
            .collect(),
    )
}

#[test]
fn scrolling_moves_a_row_at_a_time_and_stops_at_the_ends() {
    use Action::{CursorDown, CursorUp};
    let mut c = long_list();
    let mut s = CreditsScreen::new(&c);
    assert_eq!(s.rows.len(), 61);
    assert_eq!((s.top, s.max_top()), (0, 36));
    assert!(!s.more_above() && s.more_below());
    // Already at the top: nothing moves, no sound.
    assert!(sounds_of(&mut s, &mut c, &[CursorUp]).is_empty());
    assert_eq!(s.top, 0);
    assert_eq!(
        sounds_of(&mut s, &mut c, &[CursorDown, CursorDown]),
        ["menu_move"; 2]
    );
    assert_eq!(s.top, 2);
    assert!(s.more_above() && s.more_below());
    assert_eq!(sounds_of(&mut s, &mut c, &[CursorUp]), ["menu_move"]);
    assert_eq!(s.top, 1);
    s.update(&mut c, &input(&[CursorDown; 34]));
    assert_eq!(s.top, 35);
    assert!(s.more_below());
    c.audio.take();
    // The last step down, then the bottom: no further, no sound.
    assert_eq!(
        sounds_of(&mut s, &mut c, &[CursorDown, CursorDown, CursorDown]),
        ["menu_move"]
    );
    assert_eq!(s.top, 36);
    assert!(s.more_above() && !s.more_below());
}

#[test]
fn a_short_list_does_not_scroll() {
    let mut c = ctx_with(vec![entry(CreditGroup::Fonts, "Font")]);
    let mut s = CreditsScreen::new(&c);
    assert!(!s.more_above() && !s.more_below());
    assert!(sounds_of(&mut s, &mut c, &[Action::CursorDown]).is_empty());
    assert_eq!(s.top, 0);
    let view = s.view(&c);
    assert!(!view.more_above && !view.more_below);
    // A list of exactly the rows that fit doesn't scroll either:
    // 2 + 8 × 3 − 1 = 25.
    let fits = ctx_with(
        (0..8)
            .map(|i| entry(CreditGroup::Music, &format!("Track {i}")))
            .collect(),
    );
    let s = CreditsScreen::new(&fits);
    assert_eq!(s.rows.len(), LIST_H);
    assert!(!s.more_below());
}

#[test]
fn cancel_closes_with_the_cancel_sound_and_other_actions_do_nothing() {
    let mut c = long_list();
    let mut s = CreditsScreen::new(&c);
    assert_eq!(s.name(), "credits");
    assert!(!s.is_overlay());
    let others = [Action::CursorLeft, Action::CursorRight, Action::Info];
    assert!(sounds_of(&mut s, &mut c, &others).is_empty());
    assert!(s.is_rolling());
    assert_eq!(s.top, 0);
    // Actions after Cancel are dropped.
    let actions = [Action::CursorDown, Action::Cancel, Action::CursorDown];
    assert_eq!(format!("{:?}", s.update(&mut c, &input(&actions))), "Pop");
    assert_eq!(s.top, 1);
    let sounds: Vec<String> = c
        .audio
        .take()
        .iter()
        .filter_map(|r| r.cue().map(str::to_owned))
        .collect();
    assert_eq!(sounds, ["menu_move", "menu_cancel"]);
}

#[test]
fn help_names_the_layout_keys_and_what_confirm_does() {
    let mut c = ctx();
    let mut s = CreditsScreen::new(&c);
    assert_eq!(s.help(&c), "arrows scroll · f pause · d back");
    s.update(&mut c, &input(&[Action::Confirm]));
    assert_eq!(s.help(&c), "arrows scroll · f auto-scroll · d back");
    c.use_layout(crate::input::Layout::LeftHanded);
    assert_eq!(s.help(&c), "wasd scroll · j auto-scroll · k back");
}

/// An update with no actions after `dt` seconds.
fn pass(s: &mut CreditsScreen, c: &mut Ctx, dt: f32) {
    s.update(c, &FrameInput::new(vec![], dt, vec![]));
}

#[test]
fn the_list_rolls_a_row_every_half_second_after_a_rest() {
    let mut c = long_list();
    let mut s = CreditsScreen::new(&c);
    assert!(s.is_rolling());
    pass(&mut s, &mut c, 1.5);
    assert_eq!(s.top, 0, "resting at the top");
    pass(&mut s, &mut c, 0.5);
    assert_eq!(s.top, 1, "the rest is over at exactly 2 s");
    pass(&mut s, &mut c, 0.25);
    assert_eq!(s.top, 1);
    pass(&mut s, &mut c, 0.25);
    assert_eq!(s.top, 2);
    // A long frame rolls every row it covers.
    pass(&mut s, &mut c, 2.0);
    assert_eq!(s.top, 6);
    // Rolling is silent (only the music was asked for).
    let sounds = c.audio.take();
    assert!(
        sounds
            .iter()
            .all(|r| !matches!(r, AudioRequest::PlaySound { .. })),
        "{sounds:?}"
    );
    // A frame time that isn't a number changes nothing.
    pass(&mut s, &mut c, f32::NAN);
    pass(&mut s, &mut c, f32::INFINITY);
    assert_eq!(s.top, 6);
    pass(&mut s, &mut c, 0.5);
    assert_eq!(s.top, 7);
}

#[test]
fn at_the_bottom_the_list_rests_then_starts_again_from_the_top() {
    let mut c = long_list();
    let mut s = CreditsScreen::new(&c);
    // 2 s rest, then 36 rows: the last lands at 2 + 35 × 0.5 = 19.5 s.
    pass(&mut s, &mut c, 19.25);
    assert_eq!(s.top, 35);
    pass(&mut s, &mut c, 0.25);
    assert_eq!(s.top, 36);
    assert!(!s.more_below());
    pass(&mut s, &mut c, 1.75);
    assert_eq!(s.top, 36, "resting at the bottom");
    pass(&mut s, &mut c, 0.25);
    assert_eq!(s.top, 0, "back to the top after 2 s");
    pass(&mut s, &mut c, 1.75);
    assert_eq!(s.top, 0, "resting at the top again");
    pass(&mut s, &mut c, 0.25);
    assert_eq!(s.top, 1);
}

#[test]
fn a_frame_rolls_at_most_once_round_the_list() {
    let mut c = long_list();
    let mut s = CreditsScreen::new(&c);
    pass(&mut s, &mut c, 2.0);
    assert_eq!(s.top, 1);
    // From row 1 with half a second to go, once round the list is 37
    // steps: 35 rows down, the jump to the top, and row 1 again, due
    // 21.5 s from now. A frame of 21.75 s does exactly that.
    pass(&mut s, &mut c, 21.75);
    assert_eq!(s.top, 1);
    pass(&mut s, &mut c, 0.25);
    assert_eq!(s.top, 2, "the next row was a quarter second away");

    // A frame that reaches past the 37th step (here, to the moment the
    // 38th is due) puts the list back at the top, resting.
    let mut s = CreditsScreen::new(&c);
    pass(&mut s, &mut c, 2.0);
    pass(&mut s, &mut c, 22.0);
    assert_eq!(s.top, 0);
    pass(&mut s, &mut c, 1.75);
    assert_eq!(s.top, 0, "resting");
    pass(&mut s, &mut c, 0.25);
    assert_eq!(s.top, 1);

    // So does a frame time far too large to count rows by.
    let mut s = CreditsScreen::new(&c);
    pass(&mut s, &mut c, 5.0);
    assert_eq!(s.top, 7);
    pass(&mut s, &mut c, 1.0e30);
    assert_eq!(s.top, 0);
    pass(&mut s, &mut c, 1.75);
    assert_eq!(s.top, 0, "resting");
    pass(&mut s, &mut c, 0.25);
    assert_eq!(s.top, 1);
}

#[test]
fn a_list_that_fits_does_not_roll() {
    let mut c = ctx_with(vec![entry(CreditGroup::Fonts, "Font")]);
    let mut s = CreditsScreen::new(&c);
    pass(&mut s, &mut c, 60.0);
    assert_eq!(s.top, 0);
    assert!((s.wait - REST_SECS).abs() < f32::EPSILON);
}

#[test]
fn confirm_stops_the_rolling_and_starts_it_again() {
    let mut c = long_list();
    let mut s = CreditsScreen::new(&c);
    pass(&mut s, &mut c, 3.0);
    assert_eq!(s.top, 3);
    assert_eq!(
        sounds_of(&mut s, &mut c, &[Action::Confirm]),
        ["menu_select"]
    );
    assert!(!s.is_rolling());
    pass(&mut s, &mut c, 10.0);
    assert_eq!(s.top, 3, "stopped");
    assert_eq!(
        sounds_of(&mut s, &mut c, &[Action::Confirm]),
        ["menu_select"]
    );
    assert!(s.is_rolling());
    // It goes on from where it is, half a second later.
    pass(&mut s, &mut c, 0.25);
    assert_eq!(s.top, 3);
    pass(&mut s, &mut c, 0.25);
    assert_eq!(s.top, 4);
}

#[test]
fn scrolling_by_hand_stops_the_rolling() {
    let mut c = long_list();
    let mut s = CreditsScreen::new(&c);
    pass(&mut s, &mut c, 3.0);
    assert_eq!(
        sounds_of(&mut s, &mut c, &[Action::CursorUp]),
        ["menu_move"]
    );
    assert_eq!(s.top, 2);
    assert!(!s.is_rolling());
    pass(&mut s, &mut c, 10.0);
    assert_eq!(s.top, 2);
    // Down stops it too, and so does Up with nowhere to go.
    let mut s = CreditsScreen::new(&c);
    s.update(&mut c, &input(&[Action::CursorDown]));
    assert_eq!((s.top, s.is_rolling()), (1, false));
    c.audio.take();
    let mut s = CreditsScreen::new(&c);
    assert!(sounds_of(&mut s, &mut c, &[Action::CursorUp]).is_empty());
    assert_eq!((s.top, s.is_rolling()), (0, false));
    // The frame's time passes before its keys: the row that was due
    // rolls, then Down scrolls one more.
    let mut s = CreditsScreen::new(&c);
    s.update(
        &mut c,
        &FrameInput::new(vec![Action::CursorDown], 2.0, vec![]),
    );
    assert_eq!(s.top, 2);
}

#[test]
fn the_screen_asks_for_the_title_music_once() {
    let mut c = long_list();
    let mut s = CreditsScreen::new(&c);
    let music = |c: &mut Ctx| -> Vec<String> {
        c.audio
            .take()
            .iter()
            .filter(|r| !matches!(r, AudioRequest::PlaySound { .. }))
            .map(|r| r.cue().unwrap_or("-").to_owned())
            .collect()
    };
    pass(&mut s, &mut c, 0.0);
    assert_eq!(music(&mut c), [TITLE_MUSIC]);
    pass(&mut s, &mut c, 1.0);
    s.update(&mut c, &input(&[Action::Confirm, Action::CursorDown]));
    assert!(music(&mut c).is_empty());
}

/// The real list: every entry's four parts are on a row.
#[test]
fn the_games_credits_show_title_author_license_and_link() {
    let c = ctx();
    let s = CreditsScreen::new(&c);
    // Joined, so a wrapped line reads on.
    let all = texts(&s).join(" ");
    assert!(c.content.credits.entries.len() > 30);
    for e in &c.content.credits.entries {
        assert!(all.contains(&format!("\"{}\" by {}", e.title, e.author)));
        // The link may be wrapped; its start follows the license.
        let start: String = e.source.chars().take(40).collect();
        assert!(
            all.contains(&format!("{}{SEPARATOR}{start}", e.license)),
            "{e:?}"
        );
    }
    assert_eq!(texts(&s)[0], "Music");
    assert!(texts(&s).contains(&"Sound effects"));
    assert!(texts(&s).contains(&"Fonts"));
    assert!(!texts(&s).contains(&"Software"), "hidden for now");
    assert!(!texts(&s).contains(&"Art"), "no bought art in a gate");
}

#[test]
fn long_lines_wrap_to_the_width_of_their_kind() {
    let mut long = entry(CreditGroup::Music, &"word ".repeat(30));
    long.source = format!("https://example.org/{}", "x".repeat(120));
    let c = ctx_with(vec![long]);
    let s = CreditsScreen::new(&c);
    // The title wraps at a space; the link is cut at the edge.
    assert_eq!(s.rows.len(), 2 + 2 + 3);
    for (kind, text) in &s.rows {
        assert!(text.chars().count() <= width(*kind), "{text:?}");
    }
    assert_eq!(s.rows[4].1, "CC0-1.0 ·");
    assert_eq!(s.rows[5].1.chars().count(), 87);
}

/// The first frame of a long list: its first rows, a screenful of them,
/// with more below.
#[test]
fn the_view_is_a_screenful_of_rows_from_the_top() {
    let c = long_list();
    let s = CreditsScreen::new(&c);
    let view = s.view(&c);
    assert_eq!(view.title, "Credits");
    assert_eq!(view.rows.len(), LIST_H);
    let shown: Vec<_> = view
        .rows
        .iter()
        .take(5)
        .map(|r| (r.kind, r.text.as_str()))
        .collect();
    assert_eq!(
        shown,
        [
            (CreditRowKind::Heading, "Music"),
            (CreditRowKind::Heading, ""),
            (CreditRowKind::Entry, "\"Track00\" by Ann"),
            (
                CreditRowKind::Detail,
                "CC0-1.0 · https://example.org/Track00"
            ),
            (CreditRowKind::Entry, ""),
        ]
    );
    // 25 rows: the last is the eighth entry's second line.
    assert_eq!(
        view.rows.last().unwrap().text,
        "CC0-1.0 · https://example.org/Track07"
    );
    assert!(!view.more_above && view.more_below);
    assert_eq!(view.help, s.help(&c));
}

#[test]
fn the_view_scrolls_with_the_list_and_says_where_it_goes_on() {
    let mut c = long_list();
    let mut s = CreditsScreen::new(&c);
    s.update(&mut c, &input(&[Action::CursorDown; 3]));
    let view = s.view(&c);
    assert_eq!(view.rows[0].text, "CC0-1.0 · https://example.org/Track00");
    assert!(view.more_above && view.more_below);
    s.update(&mut c, &input(&[Action::CursorDown; 40]));
    let view = s.view(&c);
    assert_eq!(view.rows.len(), LIST_H);
    assert_eq!(
        view.rows.last().unwrap().text,
        "CC0-1.0 · https://example.org/Track19"
    );
    assert!(view.more_above && !view.more_below);
}

#[test]
fn a_list_that_fits_is_all_in_the_view() {
    let c = ctx_with(vec![entry(CreditGroup::Fonts, "Font")]);
    let view = CreditsScreen::new(&c).view(&c);
    assert_eq!(view.rows.len(), 4);
    assert!(!view.more_above && !view.more_below);
}

#[test]
fn the_view_help_follows_the_rolling() {
    let mut c = ctx();
    let mut s = CreditsScreen::new(&c);
    assert_eq!(s.view(&c).help, "arrows scroll · f pause · d back");
    s.update(&mut c, &input(&[Action::Confirm]));
    assert_eq!(s.view(&c).help, "arrows scroll · f auto-scroll · d back");
}
