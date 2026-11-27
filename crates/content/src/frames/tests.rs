use shuyi_assets::image::ImageInfo;

use super::*;

const GOOD: &str = r##"(
    frames: {
        "panel": (picture: "panel.png", borders: (left: 22, top: 33, right: 22, bottom: 32), fill: "#3a2c36", outset: 5),
        "menu": (picture: "menu.png", borders: (left: 22, top: 22, right: 22, bottom: 22), fill: "#382b33"),
        "map_menu": (picture: "map_menu.png", borders: (left: 22, top: 33, right: 22, bottom: 32), fill: "#1f3f80", outset: 5),
    },
    plates: {
        "title": (picture: "plate.png", caps: (left: 16, right: 16), pad: 6, text_top: 8),
    },
)"##;

/// The bought pictures' sizes, or none of them (a public build).
fn images(bought: bool) -> ImageTable {
    let panel = ImageInfo::new(96, 96);
    let all = [
        ("ui/panel.png", panel),
        ("ui/menu.png", panel),
        ("ui/map_menu.png", panel),
        ("ui/plate.png", ImageInfo::new(33, 32)),
    ];
    let wanted = if bought { &all[..] } else { &[] };
    ImageTable::new(wanted.iter().copied().collect())
}

fn errors(source: &str) -> Vec<String> {
    from_source("ui/frames.ron", source, &images(true))
        .unwrap_err()
        .iter()
        .map(ToString::to_string)
        .collect()
}

#[test]
fn the_frames_and_the_plate_load_with_their_pictures() {
    let ui = from_source("ui/frames.ron", GOOD, &images(true)).unwrap();
    let panel = ui.frame("panel").unwrap();
    assert_eq!(panel.fill, [0x3a, 0x2c, 0x36]);
    assert_eq!(panel.outset, 5);
    assert_eq!(
        panel.slice.borders,
        Borders {
            left: 22,
            top: 33,
            right: 22,
            bottom: 32
        }
    );
    assert_eq!(
        panel.slice.picture.rect,
        ImageRect {
            x: 0,
            y: 0,
            w: 96,
            h: 96
        }
    );
    assert_eq!(ui.frame("menu").unwrap().outset, 0);
    let title = ui.plate("title").unwrap();
    assert_eq!((title.pad, title.text_top), (6, 8));
    assert_eq!(
        title.slice.borders,
        Borders {
            left: 16,
            top: 0,
            right: 16,
            bottom: 0
        }
    );
}

#[test]
fn without_the_pictures_there_are_no_frames() {
    let ui = from_source("ui/frames.ron", GOOD, &images(false)).unwrap();
    assert_eq!(ui, UiFrames::default());
    assert_eq!(ui.frame("panel"), None);
}

#[test]
fn a_missing_or_unknown_name_is_an_error() {
    let missing = GOOD.replace("\"menu\":", "\"menus\":");
    let found = errors(&missing);
    assert!(
        found.iter().any(|e| e.contains("no frame \"menu\"")),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|e| e.contains("frame \"menus\": no window asks")),
        "{found:?}"
    );
    let no_plate = GOOD.replace("\"title\":", "\"name\":");
    assert!(
        errors(&no_plate)
            .iter()
            .any(|e| e.contains("no plate \"title\""))
    );
}

#[test]
fn borders_and_caps_must_leave_a_middle_and_a_fill_must_be_a_colour() {
    let wide = GOOD.replace(
        "left: 22, top: 22, right: 22",
        "left: 50, top: 22, right: 46",
    );
    assert!(errors(&wide).iter().any(|e| e.contains("leave no middle")));
    let caps = GOOD.replace("left: 16, right: 16", "left: 16, right: 17");
    assert!(errors(&caps).iter().any(|e| e.contains("leave no body")));
    let fill = GOOD.replace("#382b33", "plum");
    assert!(
        errors(&fill)
            .iter()
            .any(|e| e.contains("frame \"menu\": fill"))
    );
}

#[test]
fn the_embedded_file_loads() {
    load(&images(true)).unwrap();
    load(&images(false)).unwrap();
}
