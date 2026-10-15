//! Tests of the dialogue log's history: lines kept in order, jumping back
//! and forward, and a reply asked again.

use trpg_content::{ChoiceOption, LineId, MusicLine, Names, Present, Scene, Step};
use trpg_core::{CharacterId, LeadGender, LeadProfile};

use super::*;

fn say(speaker: &str, text: &str) -> Step {
    Step::Say {
        speaker: CharacterId(speaker.into()),
        expression: None,
        text: text.into(),
        line: LineId::default(),
    }
}

fn reply(text: &str, then: &str) -> ChoiceOption {
    ChoiceOption {
        tone: "plain".into(),
        text: text.into(),
        line: LineId::default(),
        steps: vec![say("hollis", then)],
    }
}

/// `a`, `b`, a choice of `x` (answered `rx`) or `y` (answered `ry`), `c`.
fn scene() -> Scene {
    Scene {
        id: "t".into(),
        steps: vec![
            say("hollis", "a"),
            say("hollis", "b"),
            Step::Choice {
                options: vec![reply("x", "rx"), reply("y", "ry")],
            },
            say("hollis", "c"),
        ],
    }
}

fn start() -> DialoguePlayer {
    let lead = LeadProfile::new("Ellery", LeadGender::Male);
    DialoguePlayer::new(&scene(), lead, Names::default(), &Present::Everyone)
}

/// The text of each line kept: a text box's, or `> ` and a reply's.
fn texts(h: &History) -> Vec<String> {
    h.seen()
        .iter()
        .map(|s| {
            let view = s.player.current();
            match s.reply {
                Some(i) => format!("> {}", view.choices.unwrap_or_default()[i]),
                None => view.text.unwrap_or_default().into_owned(),
            }
        })
        .collect()
}

/// The text box `player` shows.
fn text(player: &DialoguePlayer) -> String {
    player.current().text.unwrap_or_default().into_owned()
}

/// Plays the whole scene answering `option`, telling `h` each step.
fn read_all(h: &mut History, option: usize) -> DialoguePlayer {
    let mut p = start();
    h.reached(&p, None);
    p.advance();
    h.reached(&p, None);
    p.advance();
    assert!(p.is_choosing());
    h.replied(&p, option, None);
    p.choose(option);
    h.reached(&p, None);
    p.advance();
    h.reached(&p, None);
    p
}

#[test]
fn every_line_is_kept_in_order_with_the_reply() {
    let mut h = History::new();
    assert!(h.is_empty());
    read_all(&mut h, 1);
    assert_eq!(texts(&h), ["a", "b", "> y", "ry", "c"]);
    assert_eq!((h.at(), h.latest()), (4, 4));
}

#[test]
fn jumping_back_keeps_the_lines_past_it_until_a_reply() {
    let mut h = History::new();
    let mut p = start();
    h.reached(&p, None);
    p.advance();
    h.reached(&p, None);
    // Back to `a`: `b` is still kept, ahead.
    let back = h.jump(0).unwrap();
    assert_eq!(text(&back.player), "a");
    assert_eq!((h.at(), h.latest()), (0, 1));
    // Reading on reaches `b` again: nothing new is kept.
    let mut p = back.player;
    p.advance();
    h.reached(&p, None);
    assert_eq!(texts(&h), ["a", "b"]);
    assert_eq!(h.at(), 1);
}

#[test]
fn jumping_forward_goes_to_a_line_seen_past_the_one_on_screen() {
    let mut h = History::new();
    read_all(&mut h, 0);
    // From `rx` (index 3) back to `a`… the reply and after are forgotten.
    h.jump(0);
    assert_eq!(texts(&h), ["a", "b"]);
    // …and forward again, as far as the line the reply answers.
    let forward = h.jump(h.latest()).unwrap();
    assert_eq!(text(&forward.player), "b");
    assert_eq!(h.at(), 1);
}

#[test]
fn a_reply_jumped_back_past_is_asked_again() {
    let mut h = History::new();
    read_all(&mut h, 0);
    let back = h.jump(1).unwrap();
    assert_eq!(texts(&h), ["a", "b"]);
    // Reading on opens the choice again; another answer is kept.
    let mut p = back.player;
    p.advance();
    assert!(p.is_choosing());
    h.replied(&p, 1, None);
    p.choose(1);
    h.reached(&p, None);
    assert_eq!(texts(&h), ["a", "b", "> y", "ry"]);
    assert_eq!(h.at(), 3);
}

#[test]
fn jumping_to_a_reply_opens_its_choice_on_the_line_it_answers() {
    let mut h = History::new();
    read_all(&mut h, 0);
    let choice = h.jump(2).unwrap();
    assert!(choice.player.is_choosing());
    assert_eq!(texts(&h), ["a", "b"]);
    assert_eq!(h.at(), 1, "the line answered is on screen");
}

#[test]
fn a_reply_after_a_skip_keeps_the_lines_read_before() {
    let mut h = History::new();
    read_all(&mut h, 0);
    // Back to `a`, then skip to the choice: `b` was read, and stays.
    let mut p = h.jump(0).unwrap().player;
    p.skip_to_choice();
    h.replied(&p, 1, None);
    assert_eq!(texts(&h), ["a", "b", "> y"]);
    assert_eq!(h.at(), 2);
}

#[test]
fn each_line_keeps_the_music_playing_there() {
    let mut h = History::new();
    let cue = MusicLine::Cue("village".into());
    let mut p = start();
    h.reached(&p, None);
    p.advance();
    h.reached(&p, Some(cue.clone()));
    assert_eq!(h.seen()[0].music, None);
    assert_eq!(h.seen()[1].music, Some(cue));
}

#[test]
fn there_is_no_line_past_the_last() {
    let mut h = History::new();
    assert_eq!(h.jump(0), None);
    read_all(&mut h, 0);
    assert_eq!(h.jump(5), None);
    assert_eq!(texts(&h).len(), 5);
    assert_eq!(h.at(), 4);
}
