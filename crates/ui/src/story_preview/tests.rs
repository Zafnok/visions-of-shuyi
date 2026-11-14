use super::*;
use crate::screen::tests::ctx;

/// Row `y` of `buf`, as text.
fn row(buf: &GlyphBuffer, y: i32) -> String {
    (0..i32::from(buf.width()))
        .map(|x| buf.get(x, y).map_or('?', |c| c.glyph))
        .collect()
}

fn buffer(ctx: &Ctx) -> GlyphBuffer {
    let black = ctx.palette.get(UiColor::Black);
    GlyphBuffer::new(60, 12, Cell::new(' ', black, black))
}

#[test]
fn the_note_is_on_the_second_row_on_the_right_and_the_faces_on_the_left() {
    let ctx = ctx();
    let mut buf = buffer(&ctx);
    let view = StoryPreviewView {
        reference: None,
        only_here: Some("ONLY HERE".to_owned()),
        faces: None,
    };
    paint(&ctx, &view, &mut buf);
    assert!(row(&buf, 1).ends_with("ONLY HERE "), "{:?}", row(&buf, 1));
    let mut buf = buffer(&ctx);
    let view = StoryPreviewView {
        reference: None,
        only_here: None,
        faces: Some("Faces: a[b]".to_owned()),
    };
    paint(&ctx, &view, &mut buf);
    assert!(
        row(&buf, 1).starts_with(" Faces: a[b] "),
        "{:?}",
        row(&buf, 1)
    );
    let panel = ctx.palette.get(UiColor::PanelBg);
    assert_eq!(buf.get(59, 1).map(|c| c.bg), Some(panel), "the strip's row");
    assert_eq!(
        row(&buf, 0).trim(),
        "",
        "the strip's own row is the engine's"
    );
}

#[test]
fn the_page_of_names_and_faces_has_its_title_rows_and_help() {
    let ctx = ctx();
    let mut buf = buffer(&ctx);
    let rows = vec![
        ReferenceRow::Heading("NAMES".to_owned()),
        ReferenceRow::Entry("{n:king}".to_owned(), "Aldric".to_owned()),
        ReferenceRow::Entry("{n:queen}".to_owned(), "Mira".to_owned()),
    ];
    let view = StoryPreviewView {
        reference: Some(ReferenceView {
            title: "TITLE".to_owned(),
            rows,
            first: 1,
            help: "HELP".to_owned(),
        }),
        only_here: Some("never shown over the page".to_owned()),
        faces: None,
    };
    paint(&ctx, &view, &mut buf);
    assert_eq!(row(&buf, 0).trim(), "TITLE");
    // From the first row shown: the heading scrolled away.
    let first = row(&buf, 2);
    assert!(first.starts_with("   {n:king}"), "{first:?}");
    // Two cells past the longest token, "{n:queen}" (from column 3).
    assert_eq!(&first[14..20], "Aldric");
    assert!(row(&buf, 3).contains("{n:queen}"));
    assert_eq!(row(&buf, 11).trim(), "HELP");
    assert!(!(0..12).any(|y| row(&buf, y).contains("never shown")));
}

#[test]
fn the_page_lists_every_name_and_every_characters_expressions() {
    let ctx = ctx();
    let rows = reference_rows(&ctx);
    let names = &ctx.game.content.names.names;
    let entries = rows
        .iter()
        .filter(|r| matches!(r, ReferenceRow::Entry(..)))
        .count();
    let cast = &ctx.game.content.characters;
    assert!(entries >= names.len() + cast.characters.len(), "{entries}");
    let headings: Vec<&ReferenceRow> = rows
        .iter()
        .filter(|r| matches!(r, ReferenceRow::Heading(_)))
        .collect();
    assert_eq!(headings.len(), 2);
    assert_eq!(
        rows[0],
        ReferenceRow::Heading(ctx.text(NAMES_KEY).to_owned())
    );
}

#[test]
fn the_lead_follows_the_switches() {
    let scene = trpg_content::Scene::default();
    let present = trpg_content::Present::Everyone;
    let mut play = Play::new(&scene, &present);
    let lead = lead_of(&play);
    assert_eq!(
        (lead.gender, lead.name.as_str()),
        (LeadGender::Male, DEFAULT_NAME)
    );
    play.switches = &[true, false];
    assert_eq!(lead_of(&play).gender, LeadGender::Female);
    play.switches = &[false, true];
    let lead = lead_of(&play);
    assert_eq!(
        (lead.gender, lead.name.as_str()),
        (LeadGender::Male, LONG_NAME)
    );
    assert_eq!(LONG_NAME.chars().count(), MAX_NAME_LEN);
}

#[test]
fn the_preview_declares_unit_info_and_the_cursor_with_the_engines_actions() {
    let ctx = ctx();
    let screen = StoryPreview::new(&ctx);
    let actions = screen.actions().unwrap_or_default();
    for action in [
        Action::CursorUp,
        Action::CursorDown,
        Action::CursorLeft,
        Action::CursorRight,
        Action::Confirm,
        Action::Cancel,
        Action::Info,
    ] {
        assert!(actions.contains(&action), "{action:?}");
    }
    assert_eq!(screen.name(), "story_preview");
    assert!(screen.as_any().is_some());
}
