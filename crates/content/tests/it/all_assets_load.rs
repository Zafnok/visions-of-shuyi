//! ADR-0005 rule 3: the whole embedded asset bundle must load and validate.

#[test]
fn all_embedded_assets_load() {
    match trpg_content::load_embedded() {
        Err(errors) => panic!("{errors}"),
        // The sprite map skin's test tileset is among them (ticket 0433).
        Ok(content) => assert!(content.tilesets.contains_key("test")),
    }
}

/// The load in parts, as the Playdate runs it (ticket 1086), gives the
/// same content as the load in one call, each scene checked once, and
/// finishing at any point checks what is left.
#[test]
fn the_staged_load_gives_the_same_content() {
    let whole = trpg_content::load_embedded().unwrap_or_else(|errors| panic!("{errors}"));
    let scenes = whole.dialogue.scenes.len();
    let mut load = trpg_content::StagedLoad::start();
    assert_eq!(load.scenes_to_check(), 0);
    load.read_dialogue();
    assert_eq!(load.scenes_to_check(), scenes);
    for left in (1..=scenes).rev() {
        assert_eq!(load.scenes_to_check(), left);
        load.check_scene();
    }
    assert_eq!(load.scenes_to_check(), 0);
    load.check_scene();
    assert_eq!(load.finish(), Ok(whole.clone()));
    // Finished before the dialogue is read, or with some of it checked.
    assert_eq!(
        trpg_content::StagedLoad::start().finish(),
        Ok(whole.clone())
    );
    let mut load = trpg_content::StagedLoad::start();
    load.read_dialogue();
    load.check_scene();
    assert_eq!(load.finish(), Ok(whole));
}

/// Every speech line, narration line and reply of every `.dlg` has a line
/// id that starts with its scene's id, and no two lines share one
/// (ADR-0045 §3).
#[test]
fn every_dialogue_line_has_a_unique_id() {
    let content = trpg_content::load_embedded().unwrap_or_else(|errors| panic!("{errors}"));
    let mut seen = std::collections::BTreeSet::new();
    for scene in content.dialogue.scenes.values() {
        let lines = scene.lines();
        assert!(!lines.is_empty(), "{}", scene.id);
        for line in lines {
            let id = line.id.as_str();
            let hash = id.strip_prefix(&format!("{}_", scene.id)).unwrap_or("");
            let hex = hash.split('_').next().unwrap_or("");
            assert!(
                hex.len() == 8 && hex.bytes().all(|b| b.is_ascii_hexdigit()),
                "{id}: {}",
                line.text
            );
            assert!(seen.insert(id.to_owned()), "two lines have the id {id}");
        }
    }
    assert!(seen.len() > 100, "{}", seen.len());
}
