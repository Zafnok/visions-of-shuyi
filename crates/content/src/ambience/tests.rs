use super::*;

use crate::audio::{CreditRef, SoundCue};

/// A manifest with a day and a night bed in the ambience group, a looped
/// sound of another group and a bed played once.
fn audio() -> AudioManifest {
    let mut audio = AudioManifest::default();
    let sound = |looped: bool, group: Option<&str>| {
        let mut cue = SoundCue::new(vec!["a.ogg".into()], 100, CreditRef::Own);
        cue.looped = looped;
        cue.group = group.map(str::to_owned);
        cue
    };
    for (id, looped, group) in [
        ("day", true, Some(AMBIENCE)),
        ("night", true, Some(AMBIENCE)),
        ("fire", true, None),
        ("once", false, Some(AMBIENCE)),
    ] {
        audio.sounds.insert(id.into(), sound(looped, group));
    }
    audio
}

/// `source` loaded against [`audio`], the embedded lights, no maps or
/// world maps, and `battles`.
fn load(source: &str, battles: &[(&str, &str)]) -> Result<AmbienceTable, Vec<String>> {
    let (audio, lighting) = (audio(), crate::lighting::load().unwrap());
    let battles = battles.iter().map(|&(b, a)| (b.into(), a.into())).collect();
    let refs = AmbienceRefs {
        audio: &audio,
        lighting: &lighting,
        maps: &BTreeMap::new(),
        worldmaps: &BTreeMap::new(),
        battles: &battles,
    };
    from_source("ambience.ron", source, refs)
        .map_err(|e| e.into_iter().map(|e| e.message).collect())
}

const FIELD: &str = r#"(ambiences: {"field": {"noon": "day", "night": "night"}})"#;

#[test]
fn a_map_at_night_plays_its_night_bed_and_a_light_without_one_the_noon_bed() {
    let table = load(FIELD, &[("test", "field")]).unwrap();
    assert_eq!(table.bed("field", "night"), Some("night"));
    assert_eq!(table.bed("field", "noon"), Some("day"));
    assert_eq!(table.bed("field", "dusk"), Some("day"));
    assert_eq!(table.bed("nowhere", "noon"), None);
    assert_eq!(table.battles["test"], "field");
}

#[test]
fn an_ambience_naming_an_unknown_cue_fails() {
    let errors = load(r#"(ambiences: {"field": {"noon": "rain"}})"#, &[]).unwrap_err();
    assert_eq!(
        errors,
        ["ambience \"field\": sound cue \"rain\" is not a sound cue"]
    );
}

#[test]
fn a_bed_is_a_looped_cue_of_the_ambience_group_in_a_light_there_is() {
    let source = r#"(ambiences: {
        "a": {"noon": "fire"},
        "b": {"noon": "once"},
        "c": {"night": "night", "noon": "day", "twilight": "day"},
    })"#;
    let errors = load(source, &[]).unwrap_err();
    assert_eq!(errors.len(), 3, "{errors:?}");
    assert!(errors[0].ends_with("\"fire\" is not in the \"ambience\" group"));
    assert!(errors[1].ends_with("\"once\" is not looped"));
    assert!(errors[2].starts_with("ambience \"c\": \"twilight\" is not a light"));
}

#[test]
fn every_ambience_has_a_noon_bed() {
    let errors = load(r#"(ambiences: {"woods": {"night": "night"}})"#, &[]).unwrap_err();
    assert_eq!(
        errors,
        ["ambience \"woods\": no \"noon\" bed (a light without its own plays it)"]
    );
}

#[test]
fn every_name_of_an_ambience_is_one() {
    let source =
        r#"(ambiences: {"field": {"noon": "day"}}, places: {"yard": "hall"}, camp: Some("fire"))"#;
    let errors = load(source, &[("test", "marsh")]).unwrap_err();
    assert_eq!(
        errors,
        [
            "places: no map \"yard\"",
            "places: \"yard\": no ambience \"hall\"",
            "camp: no ambience \"fire\"",
            "ambience: no ambience \"marsh\" in data/ambience.ron",
        ]
    );
}

/// The embedded file loads with the rest of the content, and the test
/// battle plays the meadow (so the gates hear a bed without the private
/// content).
#[test]
fn the_test_battle_plays_the_meadow_by_day() {
    let content = crate::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let meadow = content.ambience.bed("meadow", NOON);
    assert_eq!(meadow, Some("bed_summer_meadow"));
    if crate::bundle::TEST_FILES {
        assert_eq!(content.ambience.battles["test"], "meadow");
        let world = &content.worldmaps["test_world"];
        assert_eq!(world.ambience.as_deref(), Some("meadow"));
    }
}
