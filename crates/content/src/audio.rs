//! The audio manifest (`assets/audio/audio.ron`, ADR-0026): every sound and
//! music cue the game can play, the music pools, and the credits for every
//! third-party work. Screens name cues; `app` plays the files.
//!
//! Sound files are embedded (under `assets/audio/`). Music files are not:
//! they live in the top-level `music/` folder shipped next to the game, so
//! the bundle can't check them. [`load`] checks the embedded sounds only;
//! a test checks the music folder with [`from_source`], including each
//! track's `length_ms` against its file ([`ogg_length_ms`], ADR-0037). The
//! Playdate's bundle leaves the sound files out (ticket 1036): there the
//! manifest is read, its sounds unchecked, and the Playdate plays its own
//! copies of the files, which its build converts and packs in the `.pdx`
//! with the music it lists ([`playdate_music`], ticket 1047, ADR-0084).
//!
//! The manifest is Shuyi Engine's (`shuyi_assets::audio`, ticket 1104);
//! this module names the game's files and folders.
use crate::prelude::*;

use alloc::borrow::Cow;
use alloc::collections::BTreeSet;

pub use shuyi_assets::audio::{
    AudioManifest, Bytes, Credit, CreditRef, Files, LENGTH_TOLERANCE_MS, LICENSES, MAX_VOLUME,
    MusicCue, OWN, SAMPLE_RATE, SoundCue, format_problem, from_source, ogg_length_ms,
};

use crate::bundle;
use crate::error::ContentError;
use crate::formats;

/// Path of the manifest inside the asset bundle.
pub const AUDIO_PATH: &str = "audio/audio.ron";

/// Bundle directory that sound file names are relative to.
pub const SOUND_DIR: &str = "audio";

/// Folder, next to the game (and in the repo root), that music file names
/// are relative to. Not embedded (ADR-0026).
pub const MUSIC_DIR: &str = "music";

/// The bundle path of a sound file named in the manifest.
pub fn sound_path(file: &str) -> String {
    format!("{SOUND_DIR}/{file}")
}

/// The game's files: sounds from the embedded bundle, and music read by
/// `music` (`None` in the game itself, which can't see the music folder).
pub fn files(music: Option<&dyn Fn(&str) -> Bytes>) -> Files<'_> {
    Files::new(&embedded_sound, &shown_sound, &left_out_sound, MUSIC_DIR).with_music(music)
}

/// The embedded sound file named `file` in the manifest.
fn embedded_sound(file: &str) -> Bytes {
    bundle::bytes(&sound_path(file)).map(Cow::Borrowed)
}

/// How errors name the sound file named `file` in the manifest.
fn shown_sound(file: &str) -> String {
    bundle::display_path(&sound_path(file))
}

/// Whether this build leaves out the sound file named `file` in the
/// manifest: the Playdate's plays no sound yet (ticket 1036).
fn left_out_sound(file: &str) -> bool {
    bundle::left_out(&sound_path(file))
}

/// The music the Playdate build ships: `crates/content/playdate-music.txt`
/// (ticket 1047, ADR-0084).
const PLAYDATE_MUSIC: &str = include_str!("../playdate-music.txt");

/// The music cues the Playdate build ships (`playdate-music.txt`,
/// Chapter 1's): each line of the list is a music cue or a pool of
/// `manifest`, which ships all its tracks. An id that is neither, or a
/// sound's, is an error naming it.
pub fn playdate_music(manifest: &AudioManifest) -> Result<BTreeSet<String>, String> {
    let mut cues = BTreeSet::new();
    let ids = PLAYDATE_MUSIC
        .lines()
        .map(|line| line.split('#').next().unwrap_or_default().trim())
        .filter(|id| !id.is_empty());
    for id in ids {
        if manifest.music.contains_key(id) {
            cues.insert(id.to_owned());
        } else if let Some(pool) = manifest.pools.get(id) {
            cues.extend(pool.iter().cloned());
        } else {
            // check-text: not player text (a build problem)
            return Err(format!(
                "playdate-music.txt: {id:?} is not a music cue or pool of {AUDIO_PATH}"
            ));
        }
    }
    Ok(cues)
}

/// Where the Playdate build puts the sound file named `file` in the
/// manifest, as the SDK opens it: `audio/sfx/menu_move` for
/// `sfx/menu_move.wav`. The SDK's `pdc` packs it as `.pda` and the SDK
/// finds it without the extension (ADR-0084).
pub fn playdate_sound_path(file: &str) -> String {
    format!("{SOUND_DIR}/{}", without_extension(file))
}

/// Where the Playdate build puts the music file named `file` in the
/// manifest, as the SDK opens it: `music/new_sunrise_v1` for
/// `new_sunrise_v1.ogg` (as [`playdate_sound_path`]).
pub fn playdate_music_path(file: &str) -> String {
    format!("{MUSIC_DIR}/{}", without_extension(file))
}

/// `file` without its extension, if its last part has one.
fn without_extension(file: &str) -> &str {
    let name = file.rfind('/').map_or(0, |slash| slash + 1);
    match file[name..].rfind('.') {
        Some(dot) if dot > 0 => &file[..name + dot],
        _ => file,
    }
}

/// Loads and validates the embedded manifest. Music files aren't checked
/// (see the module docs).
pub fn load() -> Result<AudioManifest, Vec<ContentError>> {
    formats::load(AUDIO_PATH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_embedded_manifest_is_valid() {
        let m = load();
        assert!(m.is_ok(), "{m:?}");
        // audio.md's starting fade (TUNABLE): 0.5 s.
        assert_eq!(m.map(|m| m.music_fade_ms), Ok(500));
    }

    /// The PC's bundle checks every sound file; the Playdate's, which
    /// leaves them out (ticket 1036), checks none and still loads.
    #[test]
    fn the_playdates_bundle_leaves_the_sound_files_out_unchecked() {
        let heal = "sfx/heal.wav";
        assert!(!(files(None).sound_left_out)(heal));
        assert!(embedded_sound(heal).is_some());
        let playdate = Box::leak(Box::new(bundle::Bundle::new(bundle::playdate_layers())));
        bundle::reading(playdate, || {
            assert!((files(None).sound_left_out)(heal));
            assert!(embedded_sound(heal).is_none());
            let m = load();
            assert!(m.is_ok(), "{m:?}");
            assert!(m.is_ok_and(|m| m.sounds.values().any(|s| s.files.iter().any(|f| f == heal))));
        });
    }

    /// Chapter 1's tracks, a pool as all its tracks, and nothing else.
    #[test]
    fn the_playdates_music_is_chapter_1s() {
        let m = load().unwrap_or_default();
        let cues = playdate_music(&m).unwrap_or_else(|e| panic!("{e}"));
        for cue in ["title", "battle_bright", "defeat", "village_home"] {
            assert!(cues.contains(cue), "{cue}: {cues:?}");
        }
        assert!(m.pools["victory"].iter().all(|cue| cues.contains(cue)));
        assert!(!cues.contains("victory"), "a pool is its tracks");
        assert!(!cues.contains("battle_epic_a"), "not Chapter 1's");
        assert!(cues.iter().all(|cue| m.music.contains_key(cue)));
    }

    #[test]
    fn an_id_the_playdates_list_does_not_know_is_an_error() {
        let mut m = load().unwrap_or_default();
        m.music.remove("title");
        assert_eq!(
            playdate_music(&m),
            Err(
                "playdate-music.txt: \"title\" is not a music cue or pool of audio/audio.ron"
                    .to_owned()
            )
        );
    }

    #[test]
    fn the_playdates_paths_drop_the_extension() {
        assert_eq!(
            playdate_sound_path("sfx/menu_move.wav"),
            "audio/sfx/menu_move"
        );
        assert_eq!(playdate_sound_path("sfx/a.b.ogg"), "audio/sfx/a.b");
        assert_eq!(
            playdate_music_path("new_sunrise_v1.ogg"),
            "music/new_sunrise_v1"
        );
        // Nothing to drop: no extension, a dot only in a folder, a name
        // that starts with its dot.
        assert_eq!(playdate_music_path("theme"), "music/theme");
        assert_eq!(playdate_sound_path("v1.2/hit"), "audio/v1.2/hit");
        assert_eq!(playdate_sound_path("sfx/.hidden"), "audio/sfx/.hidden");
    }

    #[test]
    fn sound_files_are_read_from_the_bundle_and_named_by_it() {
        let files = files(None);
        assert_eq!((files.sound_shown)("sfx/a.wav"), "assets/audio/sfx/a.wav");
        assert_eq!(files.music_dir, "music");
        assert!(files.music.is_none());
        let m = load().unwrap_or_default();
        let file = m.sounds.values().flat_map(|s| &s.files).next();
        let file = file.map(String::as_str).unwrap_or_default();
        assert_eq!(
            (files.sound)(file).as_deref(),
            bundle::bytes(&sound_path(file))
        );
        assert!((files.sound)(file).is_some());
        assert_eq!((files.sound)("sfx/nope.wav"), None);
    }

    /// Music files aren't embedded, so the game can't check them; this
    /// test checks the repo's `music/` folder instead (ADR-0026): every
    /// file is there, is playable, and is as long as its cue's `length_ms`
    /// says (ADR-0037). A wrong length fails with the value to write.
    #[test]
    fn every_music_file_is_in_the_music_folder() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../music");
        assert!(dir.is_dir(), "the {MUSIC_DIR}/ folder is missing");
        let exists = |f: &str| std::fs::read(dir.join(f)).ok().map(Cow::Owned);
        let source = bundle::file(AUDIO_PATH).unwrap_or_default();
        let errors = match from_source("a.ron", source, &files(Some(&exists))) {
            Ok(_) => vec![],
            Err(e) => e.iter().map(ToString::to_string).collect(),
        };
        assert_eq!(errors, Vec::<String>::new());
    }

    /// The cue ids in the first column of `docs/design/audio.md`'s "Music
    /// cues" and "Sound effects" tables, with whether the row is ours
    /// ("in-house").
    fn design_cues() -> Vec<(String, bool)> {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/design/audio.md");
        let doc = std::fs::read_to_string(path).unwrap_or_default();
        let mut cues = Vec::new();
        let mut in_table = false;
        for line in doc.lines() {
            if line.starts_with("## ") {
                in_table = line == "## Music cues" || line == "## Sound effects";
            } else if in_table && line.starts_with("| `") {
                let cells: Vec<&str> = line.split('|').collect();
                let own = cells.iter().any(|c| c.trim() == "in-house");
                let first = cells.get(1).copied().unwrap_or_default();
                for (i, id) in first.split('`').enumerate() {
                    if i % 2 == 1 {
                        cues.push((id.to_owned(), own));
                    }
                }
            }
        }
        cues
    }

    /// The sounds we make ourselves; ticket 0213 renders them and adds
    /// their cues. Until then they may be missing from the manifest.
    const IN_HOUSE: [&str; 6] = [
        "menu_move",
        "cursor_move",
        "menu_select",
        "menu_cancel",
        "miss",
        "heal",
    ];

    /// Cues `audio.md` reserves for a moment the story doesn't have yet
    /// (0022, 0023, 0025, 0026, 0027): each is imported by the ticket that
    /// first plays it (`camp` by 1003;
    /// Chapters 2-8's cues from 0071 by their chapter battle tickets,
    /// 0848–0854).
    const RESERVED: [&str; 15] = [
        "victory_grand",
        "ceremony_intro",
        "scene_loss",
        "world_map_war",
        "world_map_somber",
        "battle_rival",
        "battle_night_siege",
        "battle_war",
        "battle_finale",
        "scene_departure",
        "scene_confide",
        "scene_lamplight",
        "holy_city",
        "scene_dread",
        "camp",
    ];

    /// Every cue `audio.md` lists is in the manifest (ticket 0214), but
    /// for the [`RESERVED`] ones.
    #[test]
    fn every_design_cue_is_in_the_manifest() {
        let m = load().unwrap_or_default();
        let cues = design_cues();
        assert!(cues.len() > 30, "audio.md's tables not found: {cues:?}");
        let own: Vec<&str> = cues.iter().filter(|c| c.1).map(|c| c.0.as_str()).collect();
        assert_eq!(own, IN_HOUSE, "audio.md's in-house sounds changed");
        let missing: Vec<&str> = cues
            .iter()
            .map(|c| c.0.as_str())
            .filter(|id| {
                !(m.sounds.contains_key(*id)
                    || m.music.contains_key(*id)
                    || m.pools.contains_key(*id)
                    || IN_HOUSE.contains(id)
                    || RESERVED.contains(id))
            })
            .collect();
        assert_eq!(missing, Vec::<&str>::new());
        let imported = |id: &&str| m.music.contains_key(*id);
        assert!(!RESERVED.iter().any(imported), "drop it from RESERVED");
        // Every third-party cue has a credit (ours are `Own`).
        for (id, credit) in m
            .sounds
            .iter()
            .map(|(id, s)| (id, &s.credit))
            .chain(m.music.iter().map(|(id, t)| (id, &t.credit)))
        {
            assert!(
                *credit == CreditRef::Own || !credit.ids().is_empty(),
                "{id} has no credit"
            );
        }
        for (id, c) in &m.credits {
            assert!(
                !c.tags.is_empty() && !c.note.is_empty(),
                "credit {id} needs tags and a note"
            );
        }
    }

    #[test]
    fn sound_paths_are_under_the_audio_directory() {
        assert_eq!(sound_path("sfx/a.wav"), "audio/sfx/a.wav");
    }
}
