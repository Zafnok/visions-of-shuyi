//! Scripted tests of the dialogue log (ticket 0729,
//! `docs/design/look-and-feel.md`, *Dialogue log*) over the Chapter 1
//! opening, `ch01_intro`: what it lists, jumping back and forward, a reply
//! asked again, and its key.

use insta::assert_snapshot;
use trpg_core::LeadGender;
use trpg_ui::harness::Harness;
use trpg_ui::input::{Action, Chord, Layout};
use trpg_ui::screens::DialogueScreen;
use trpg_ui::screens::dialogue::log::{LogView, RowKind};
use trpg_ui::{Ctx, Game};

/// `ch01_intro` from its first box, as `--scene` opens it, in `layout`.
fn intro_in(layout: Layout) -> Harness {
    let ctx = Ctx::embedded()
        .unwrap_or_else(|e| panic!("{e}"))
        .with_layout(layout);
    let game =
        Game::start_on_scene(ctx, "ch01_intro", LeadGender::Male).unwrap_or_else(|e| panic!("{e}"));
    Harness::from_game(game)
}

/// [`intro_in`] the right-handed layout.
fn intro() -> Harness {
    intro_in(Layout::RightHanded)
}

/// The dialogue screen on the stack.
fn dialogue(h: &Harness) -> &DialogueScreen {
    h.game()
        .screen::<DialogueScreen>()
        .unwrap_or_else(|| panic!("no scene is playing: {:?}", h.screens()))
}

/// The open log.
fn log(h: &Harness) -> LogView {
    dialogue(h)
        .log_view(h.game().ctx())
        .unwrap_or_else(|| panic!("the log isn't open"))
}

/// Moves on `n` text boxes: each is shown in full first, then Confirm.
fn read_on(h: &mut Harness, n: usize) {
    for _ in 0..n {
        h.wait(10.0);
        h.keys("f");
    }
}

/// The first line of the text box on screen.
fn on_screen(h: &Harness) -> String {
    dialogue(h)
        .page_lines()
        .first()
        .cloned()
        .unwrap_or_default()
}

/// The log's rows' texts.
fn texts(view: &LogView) -> Vec<&str> {
    view.rows.iter().map(|r| r.text.as_str()).collect()
}

/// The expression of the portrait on the right on screen.
fn right_expression(h: &Harness) -> String {
    let view = dialogue(h).player().current();
    view.right
        .map(|p| p.expression.to_owned())
        .unwrap_or_default()
}

#[test]
fn three_boxes_in_the_log_lists_the_three_lines_in_order() {
    let mut h = intro();
    read_on(&mut h, 2);
    assert!(!dialogue(&h).is_log_open());
    h.keys("b");
    let view = log(&h);
    let texts = texts(&view);
    assert_eq!(texts.len(), 3, "{texts:?}");
    assert!(texts[0].starts_with("Three years after"), "{texts:?}");
    assert!(texts[1].starts_with("The war is won."), "{texts:?}");
    assert!(texts[2].starts_with("In the Thornmarch"), "{texts:?}");
    // Narration: no name, no face. The cursor is on the line on screen.
    assert!(view.rows.iter().all(|r| r.kind == RowKind::Narration));
    assert!(
        view.rows
            .iter()
            .all(|r| r.name.is_none() && r.face.is_none())
    );
    // Each knows its line, for its voice (ticket 0720).
    let lines: Vec<String> = view
        .rows
        .iter()
        .filter_map(|r| r.line.as_ref().map(ToString::to_string))
        .collect();
    assert_eq!(lines[0], "ch01_intro_3aec73da");
    assert_eq!(lines.len(), 3);
    assert!(view.rows[2].focused);
    assert_eq!(view.back_to_latest, None);
    assert_eq!(view.title, "Log");
    assert_eq!(view.help, "arrows move · f go to this line · d close");
    // The text box waits underneath: Cancel closes the log.
    h.keys("d");
    assert!(!dialogue(&h).is_log_open());
    assert!(on_screen(&h).starts_with("In the Thornmarch"));
}

#[test]
fn jumping_back_shows_the_first_box_and_back_to_latest_the_third() {
    let mut h = intro();
    read_on(&mut h, 2);
    h.keys("b Up Up f");
    assert!(!dialogue(&h).is_log_open());
    assert!(on_screen(&h).starts_with("Three years after"));
    // As it was: no portraits yet, no caption.
    let view = dialogue(&h).player().current();
    assert_eq!((view.left, view.right, view.caption), (None, None, None));
    // The lines past it are kept, dimmer, with Back to latest after them.
    h.keys("b");
    let view = log(&h);
    let ahead: Vec<bool> = view.rows.iter().map(|r| r.ahead).collect();
    assert_eq!(ahead, [false, true, true]);
    assert!(view.rows[0].focused);
    assert_eq!(
        view.back_to_latest.as_ref().map(|b| b.label.as_str()),
        Some("Back to latest")
    );
    h.keys("Down Down Down");
    assert!(log(&h).back_to_latest.is_some_and(|b| b.focused));
    // Down at the last row stays there.
    h.keys("Down f");
    assert!(on_screen(&h).starts_with("In the Thornmarch"));
    h.keys("b");
    assert_eq!(log(&h).back_to_latest, None);
}

#[test]
fn a_jump_back_shows_the_portraits_as_they_were_and_reading_on_keeps_the_lines() {
    let mut h = intro();
    // Box 9: Hollis, sad, "Leave it where it is."
    read_on(&mut h, 8);
    assert!(on_screen(&h).starts_with("Leave it where it is."));
    assert_eq!(right_expression(&h), "sad");
    h.keys("b");
    let view = log(&h);
    let faces: Vec<_> = view.rows[6..9]
        .iter()
        .map(|r| r.face.as_ref().map(|f| f.expression.as_str()))
        .collect();
    // Narration, then Hollis surprised, then sad.
    assert_eq!(faces, [None, Some("surprised"), Some("sad")]);
    assert_eq!(view.rows[8].name.as_deref(), Some("Hollis Marr"));
    assert_eq!(view.rows[5].name.as_deref(), Some("Ellery"));
    assert_eq!(
        view.rows[5].face.as_ref().map(|f| f.portrait.as_str()),
        Some("lead_m")
    );
    // Back two lines, to "Three strokes.": Hollis as he was then.
    h.keys("Up Up f");
    assert!(on_screen(&h).starts_with("Three strokes."));
    assert_eq!(right_expression(&h), "neutral");
    let caption = dialogue(&h)
        .player()
        .current()
        .caption
        .map(std::borrow::Cow::into_owned);
    assert_eq!(caption.as_deref(), Some("Harrowby, the Thornmarch"));
    // Reading on goes through the lines kept; nothing is added.
    read_on(&mut h, 2);
    assert!(on_screen(&h).starts_with("Leave it where it is."));
    assert_eq!(right_expression(&h), "sad");
    assert_eq!(dialogue(&h).history().seen().len(), 9);
    assert_eq!(dialogue(&h).history().at(), 8);
}

#[test]
fn a_reply_passed_on_the_way_back_is_asked_again() {
    let mut h = intro();
    // "Four years. Not a line to you…", then the replies.
    read_on(&mut h, 13);
    assert!(dialogue(&h).menu().is_some());
    h.keys("Down f");
    assert!(on_screen(&h).starts_with("Ha. He never did write"));
    h.keys("b");
    let view = log(&h);
    let reply = &view.rows[13];
    assert_eq!(reply.kind, RowKind::Reply);
    assert_eq!(reply.text, "Four years of silence, and now he's urgent.");
    assert_eq!(reply.name.as_deref(), Some("Ellery"));
    assert!(reply.face.is_some(), "the lead's face");
    let line = reply.line.as_ref().map(ToString::to_string);
    assert!(line.is_some_and(|l| l.starts_with("ch01_intro_")));
    // Back to the line it answers: the reply and what follows are gone.
    h.keys("Up Up f");
    assert!(on_screen(&h).starts_with("Four years. Not a line"));
    h.keys("b");
    let view = log(&h);
    assert_eq!(view.rows.len(), 13);
    assert_eq!(view.back_to_latest, None, "forward stops before the reply");
    // Reading on asks again, and another answer is kept.
    h.keys("d");
    read_on(&mut h, 1);
    let replies = dialogue(&h).menu().map(trpg_ui::widgets::Menu::focus);
    assert_eq!(replies, Some(0), "the replies are up again");
    h.keys("f");
    assert!(on_screen(&h).starts_with("Aye. That's what worries me."));
    h.keys("b");
    let view = log(&h);
    assert_eq!(view.rows[13].kind, RowKind::Reply);
    assert!(view.rows[13].text.starts_with("He's warning us."));
    assert_eq!(view.rows.len(), 15);
}

#[test]
fn going_to_a_reply_opens_its_replies_again() {
    let mut h = intro();
    read_on(&mut h, 13);
    h.keys("Down f b Up f");
    let d = dialogue(&h);
    assert!(d.menu().is_some(), "the replies are up");
    assert!(on_screen(&h).starts_with("Four years. Not a line"));
    assert_eq!(d.history().seen().len(), 13);
}

#[test]
fn the_log_waits_for_a_line_and_not_at_skip_scene() {
    let mut h = intro();
    // Cancel asks to skip; the log key does nothing then.
    h.keys("d b");
    assert!(!dialogue(&h).is_log_open());
    assert!(dialogue(&h).is_asking_skip());
    h.keys("d b");
    assert!(dialogue(&h).is_log_open());
    // While it is open the text doesn't type on.
    let shown = dialogue(&h).is_revealed();
    h.wait(10.0);
    assert_eq!(dialogue(&h).is_revealed(), shown);
}

#[test]
fn on_a_key_the_scene_uses_the_scene_wins() {
    let mut h = intro();
    let layout = Layout::RightHanded;
    let mut b = h.game().ctx().layout_bindings(layout);
    let up = Chord::parse("Up").unwrap_or_else(|e| panic!("{e}"));
    // Another group's key: it stays on Cursor up too.
    assert_eq!(b.bind(Action::DialogueLog, 0, up), Ok(None));
    if let Err(e) = h.ctx_mut().set_layout_bindings(layout, b) {
        panic!("saving key bindings: {e}");
    }
    // Reading, Up opens the log, and moves in it.
    read_on(&mut h, 1);
    h.keys("Up");
    assert!(dialogue(&h).is_log_open());
    h.keys("Up");
    assert!(log(&h).rows[0].focused);
    h.keys("d");
    // At the replies, Up moves between them.
    read_on(&mut h, 12);
    h.keys("Down");
    assert_eq!(
        dialogue(&h).menu().map(trpg_ui::widgets::Menu::focus),
        Some(1)
    );
    h.keys("Up");
    assert!(!dialogue(&h).is_log_open());
    assert_eq!(
        dialogue(&h).menu().map(trpg_ui::widgets::Menu::focus),
        Some(0)
    );
}

#[test]
fn the_controller_opens_it_with_the_left_face_button() {
    let mut h = intro();
    read_on(&mut h, 1);
    h.pad("West");
    assert!(dialogue(&h).is_log_open());
    h.pad("DpadUp South");
    assert!(on_screen(&h).starts_with("Three years after"));
}

#[test]
fn the_text_box_names_the_log_key_only_while_it_has_one() {
    let mut h = intro();
    h.wait(10.0);
    let bottom = |h: &Harness| {
        let buf = h.game().buffer();
        (0..100)
            .map(|x| buf.get(x, 26).map_or(' ', |c| c.glyph))
            .collect::<String>()
    };
    assert!(bottom(&h).contains("b log   f ▼"), "{}", bottom(&h));
    let layout = Layout::RightHanded;
    let mut b = h.game().ctx().layout_bindings(layout);
    b.clear(Action::DialogueLog, 0);
    if let Err(e) = h.ctx_mut().set_layout_bindings(layout, b) {
        panic!("saving key bindings: {e}");
    }
    h.wait(0.1);
    assert!(!bottom(&h).contains("log"), "{}", bottom(&h));
    h.keys("b");
    assert!(!dialogue(&h).is_log_open());
}

#[test]
fn log_snapshot_right_handed() {
    let mut h = intro();
    read_on(&mut h, 13);
    h.keys("Down f b Up Up Up");
    assert_snapshot!(h.snapshot());
}

#[test]
fn log_snapshot_left_handed() {
    let mut h = intro_in(Layout::LeftHanded);
    for _ in 0..8 {
        h.wait(10.0);
        h.keys("j");
    }
    h.keys("n w w j n");
    assert_snapshot!(h.snapshot());
}
