//! The credits (ticket 0808; `docs/design/audio.md` rule 3): every
//! third-party work the game uses, for the credits screen. Music and sounds
//! are credited in the audio manifest ([`crate::audio`]); everything else
//! (the font, bought art, vendored software) in `assets/data/credits.ron`.
//! [`load`] merges the two into one list, so nothing is typed twice.
//!
//! A bought work (ADR-0032) names the folders of `assets-private/game/` it
//! covers (ADR-0051). It is shown only in a build that has files there, and
//! [`uncredited`] finds the bought files no credit covers.
//!
//! The credits are Shuyi Engine's (`shuyi_assets::credits`, ticket 1104);
//! this module names the game's files and folders.
use crate::prelude::*;

pub use shuyi_assets::credits::{
    CreditEntry, CreditGroup, Credits, CreditsFile, FileCredit, LICENSES, Places, merge, uncredited,
};

use crate::audio::{self, AudioManifest};
use crate::bundle;
use crate::error::ContentError;
use crate::formats;

/// Path of the credits file inside the asset bundle.
pub const CREDITS_PATH: &str = "data/credits.ron";

/// Loads and validates the embedded credits file.
pub fn load_file() -> Result<CreditsFile, Vec<ContentError>> {
    formats::load(CREDITS_PATH)
}

/// Loads the embedded credits file and merges it with `audio`'s credits
/// and the bought files this build has. Without the manifest (it failed to
/// load) the list has no audio credits.
pub fn load(audio: Option<&AudioManifest>) -> Result<Credits, Vec<ContentError>> {
    let file = load_file()?;
    let mut bought = bundle::files_under("");
    bought.retain(|p| bundle::is_private(&bundle::display_path(p)));
    let no_audio = AudioManifest::default();
    Ok(merge(&file, audio.unwrap_or(&no_audio), &bought))
}

/// Parses and validates credits `source`, attributing errors to `file`.
/// Reports every problem found.
pub fn from_source(file: &str, source: &str) -> Result<CreditsFile, Vec<ContentError>> {
    let audio_manifest = bundle::display_path(audio::AUDIO_PATH);
    let places = Places {
        audio_manifest: &audio_manifest,
        private_root: bundle::PRIVATE_DISPLAY_ROOT,
    };
    shuyi_assets::credits::from_source(file, source, &places)
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;

    use super::*;

    #[test]
    fn errors_name_the_games_audio_manifest_and_private_folder() {
        let source = r#"(credits: [
            (id: "a", group: Music, title: "A", author: "a", source: "https://a",
             license: "Custom (a)", private: ["/x"]),
        ])"#;
        let errors = from_source("c.ron", source).err().unwrap_or_default();
        let messages: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
        assert_eq!(
            messages,
            [
                "credit \"a\": Music credits go in assets/audio/audio.ron",
                "credit \"a\": private path \"/x\" must be a path inside assets-private/game/",
            ]
        );
    }

    #[test]
    fn the_embedded_file_loads_and_merges_with_the_audio_manifest() {
        let file = load_file();
        assert!(file.is_ok(), "{file:?}");
        let audio = audio::load().unwrap_or_default();
        let credits = load(Some(&audio)).unwrap_or_default();
        // Every third-party work in the manifest is shown, once.
        let mut want: Vec<&str> = audio
            .credits
            .values()
            .filter(|c| c.license != audio::OWN)
            .map(|c| c.source.as_str())
            .collect();
        let mut got: Vec<&str> = credits
            .entries
            .iter()
            .filter(|e| e.group.is_audio())
            .map(|e| e.source.as_str())
            .collect();
        want.sort_unstable();
        got.sort_unstable();
        assert_eq!(got, want);
        assert!(credits.in_group(CreditGroup::Music).count() > 15);
        assert!(credits.in_group(CreditGroup::SoundEffects).count() > 10);
        let fonts: Vec<&str> = credits
            .in_group(CreditGroup::Fonts)
            .map(|e| e.title.as_str())
            .collect();
        assert_eq!(fonts, ["Terminus Font"]);
        // The bought art is credited in the file, and shown only in a build
        // that has it: never in a gate (ADR-0040).
        let file = file.unwrap_or_default();
        let bought: Vec<&FileCredit> = file
            .credits
            .iter()
            .filter(|c| !c.private.is_empty())
            .collect();
        let ids: Vec<&str> = bought.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, ["tiny_tales", "shironejiya"]);
        assert!(bought.iter().all(|c| c.group == CreditGroup::Art));
        assert!(bought[0].covers("units/witch.png"));
        // Each combat picture is its own artist's (0413).
        assert!(bought[0].covers("battlers/guard.png") && bought[0].covers("battlers/guard.ron"));
        assert!(bought[0].covers("battlebacks/fields.png"));
        assert!(!bought[0].covers("battlers/brigand.png"));
        assert!(bought[1].covers("battlers/brigand.png"));
        assert!(!bought[1].covers("battlers/guard.png"));
        #[cfg(not(feature = "private-assets"))]
        assert_eq!(credits.in_group(CreditGroup::Art).count(), 0);
        // Without the manifest: the file's entries alone.
        let alone = load(None).unwrap_or_default();
        assert_ne!(alone.entries, [] as [shuyi_assets::credits::CreditEntry; 0]);
        assert!(alone.entries.iter().all(|e| !e.group.is_audio()));
    }

    /// The first link in each row's "Source URL" cell of
    /// `THIRD_PARTY_ASSETS.md`'s table.
    fn asset_sources() -> Vec<String> {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../THIRD_PARTY_ASSETS.md");
        let doc = std::fs::read_to_string(path).unwrap_or_default();
        doc.lines()
            .filter(|l| l.starts_with("| ") && !l.starts_with("| Item ") && !l.starts_with("| -"))
            .map(|row| {
                let cell = row.split('|').nth(2).unwrap_or_default();
                let link = cell.split_whitespace().find(|w| w.starts_with("http"));
                link.unwrap_or_default().trim_end_matches(',').to_owned()
            })
            .collect()
    }

    /// Every work in `THIRD_PARTY_ASSETS.md` has a credit, in the credits
    /// file or the audio manifest, and every credit has a row there, so
    /// the credits screen can't fall out of date (ticket 0808). They are
    /// matched by source link.
    #[test]
    fn every_third_party_asset_has_a_credit() {
        let rows = asset_sources();
        assert!(rows.len() > 40, "the table wasn't found: {rows:?}");
        let file = load_file().unwrap_or_default();
        let audio = audio::load().unwrap_or_default();
        let credited: BTreeSet<&str> = file
            .credits
            .iter()
            .map(|c| c.source.as_str())
            .chain(
                audio
                    .credits
                    .values()
                    .filter(|c| c.license != audio::OWN)
                    .map(|c| c.source.as_str()),
            )
            .collect();
        let listed: BTreeSet<&str> = rows.iter().map(String::as_str).collect();
        let no_credit: Vec<&&str> = listed.difference(&credited).collect();
        assert!(
            no_credit.is_empty(),
            "in THIRD_PARTY_ASSETS.md without a credit: {no_credit:?}"
        );
        let no_row: Vec<&&str> = credited.difference(&listed).collect();
        assert!(
            no_row.is_empty(),
            "credited but not in THIRD_PARTY_ASSETS.md: {no_row:?}"
        );
        // Hidden entries are software only: every font and picture is
        // shown.
        for c in file.credits.iter().filter(|c| c.hidden) {
            assert_eq!(c.group, CreditGroup::Software, "{} is hidden", c.id);
        }
    }
}
