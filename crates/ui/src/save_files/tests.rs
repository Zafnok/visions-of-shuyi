use super::*;
use crate::screens::files::glyph::place_line;

fn profile(n: u32) -> ProfileId {
    let mut id = ProfileId::FIRST;
    while id.get() < n {
        id = ron::from_str(&format!("{}", id.get() + 1)).unwrap_or_else(|e| panic!("{e}"));
    }
    id
}

#[test]
fn a_file_name_is_safe_everywhere() {
    let first = ProfileId::FIRST;
    assert_eq!(
        export_name("Player 1", first, Export::Profile),
        "visions-of-shuyi-player-1-1.ron"
    );
    // Runs of spaces are one `-`; other characters are left out.
    assert_eq!(
        export_name("  Mei   Ling! ", profile(3), Export::Profile),
        "visions-of-shuyi-mei-ling-3.ron"
    );
    // A name with no `a`-`z` or digits keeps the profile's number.
    assert_eq!(
        export_name("淑怡", profile(2), Export::Profile),
        "visions-of-shuyi-2.ron"
    );
    assert_eq!(
        export_name("Zoë", first, Export::Profile),
        "visions-of-shuyi-zo-1.ron"
    );
    let slot = Export::Slot {
        slot: 7,
        settings: true,
    };
    assert_eq!(
        export_name("Odo", first, slot),
        "visions-of-shuyi-odo-1-slot-07.ron"
    );
}

#[test]
fn a_profile_file_takes_every_key() {
    let keep = export_keys(Export::Profile);
    for key in [
        "settings",
        "slot_01",
        "slot_30",
        "suspend",
        "tips_seen",
        "last_save",
    ] {
        assert!(keep(key), "{key}");
    }
}

#[test]
fn a_save_file_takes_its_slot_and_maybe_the_options() {
    let alone = export_keys(Export::Slot {
        slot: 2,
        settings: false,
    });
    assert!(alone("slot_02"));
    for key in [
        "slot_01",
        "slot_20",
        "settings",
        "keybindings",
        "layout",
        "suspend",
    ] {
        assert!(!alone(key), "{key}");
    }
    let with = export_keys(Export::Slot {
        slot: 2,
        settings: true,
    });
    for key in ["slot_02", "settings", "keybindings", "layout"] {
        assert!(with(key), "{key}");
    }
    for key in ["slot_01", "suspend", "tips_seen", "last_save"] {
        assert!(!with(key), "{key}");
    }
}

#[test]
fn only_a_folder_is_a_place_to_show() {
    let mut facts = PlatformFacts::default();
    assert!(!can_export(&facts));
    assert_eq!(place(&facts), None);
    facts.storage_place = StoragePlace::Browser {
        prefix: "tactical-rpg/".into(),
    };
    assert!(can_export(&facts));
    assert_eq!(place(&facts), None);
    facts.storage_place = StoragePlace::Folder("C:\\saves".into());
    assert!(can_export(&facts));
    assert_eq!(place(&facts), Some("C:\\saves"));
}

#[test]
fn a_long_path_loses_its_start() {
    assert_eq!(place_line("In", "C:\\a\\b", 20), "In C:\\a\\b");
    let line = place_line("In", "C:\\Users\\Someone\\tactical-rpg", 20);
    assert_eq!(line, "In …one\\tactical-rpg");
    assert_eq!(line.chars().count(), 20);
}

#[test]
fn a_notice_names_where_the_file_went() {
    let mut notice = ExportNotice::default();
    assert_eq!(notice.shown(), None);
    notice.offered("a.ron".into());
    assert_eq!(notice.shown(), None);
    // A download is named by the file offered.
    notice.heard(None);
    assert_eq!(notice.shown(), Some(&Exported::Downloaded("a.ron".into())));
    notice.offered("a.ron".into());
    notice.heard(Some(Exported::Written("a (2).ron".into())));
    assert_eq!(notice.shown(), Some(&Exported::Written("a (2).ron".into())));
    assert_eq!(Exported::folder(notice.shown()), Folder::Exports);
    notice.clear();
    assert_eq!(notice.shown(), None);
    assert_eq!(Exported::folder(None), Folder::Storage);
}

#[test]
fn a_written_path_gives_its_file_name() {
    assert_eq!(file_name("C:\\x\\exports\\a (2).ron"), "a (2).ron");
    assert_eq!(file_name("<memory>/exports/a.ron"), "a.ron");
    assert_eq!(file_name("a.ron"), "a.ron");
}

#[test]
fn the_entries_of_another_profile_lose_its_prefix() {
    let mut computer = crate::storage::MemoryStorage::new();
    for (key, value) in [
        ("p1-slot_01", "a"),
        ("p2-slot_01", "b"),
        ("p2-settings", "c"),
        ("profiles", "d"),
    ] {
        computer.write(key, value).unwrap_or_else(|e| panic!("{e}"));
    }
    let keys = ProfileKeys {
        computer: &computer,
        prefix: profile(2).prefix(),
    };
    let entries = keys.entries(|_| true).unwrap_or_else(|e| panic!("{e}"));
    let entries: Vec<_> = entries
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    assert_eq!(entries, [("settings", "c"), ("slot_01", "b")]);
}
