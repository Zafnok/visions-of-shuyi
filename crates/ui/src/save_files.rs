//! Saves outside the game (ticket 0872, `docs/design/options.md`, *Saves
//! outside the game*): a profile's saves exported as one file the player
//! keeps, and where the game keeps them, for the Profile screen and the
//! Load Game list.
//!
//! - **What a file holds** ([`Export`]): the whole profile from the
//!   Profile screen; one save, and the options and keys if the player
//!   says so, from Load Game. Either is a profile's file (a save bundle,
//!   `shuyi_ui::storage::bundle`) whose keys have no `p<n>-` prefix, so it
//!   imports into any profile (0873).
//! - **Its name** ([`export_name`]): `visions-of-shuyi-<name>-<n>.ron`.
//! - **Offering it** ([`export`], [`offer`]): a download on the web, a
//!   file in the storage folder's `exports` on a desktop
//!   ([`Ctx::offer_file`]). The platform says where it went in a later
//!   frame; [`hear`] hands that to the screen as an [`Exported`] notice.
//!   [`offer`] takes entries that aren't in a storage too: the demo's end
//!   (0914) bundles a chapter save it never stored.
//! - **Where the saves are** ([`place`]): the folder on a desktop; nothing
//!   on the web, whose export is a download (Nick), or on the Playdate.
//! - **Importing a file** ([`import`], ticket 0873): what this build can
//!   read of it, where it goes, and writing it.

use crate::prelude::*;
use alloc::collections::BTreeMap;

use shuyi_core::Made;
use shuyi_ui::player_files::Folder;
use shuyi_ui::storage::bundle::SaveBundle;
use shuyi_ui::{PlatformFacts, StoragePlace};

use crate::profiles::ProfileId;
use crate::save::slot_key;
use crate::screen::{Ctx, KEYBINDINGS_KEY, LAYOUT_KEY};
use crate::settings::SETTINGS_KEY;
use crate::storage::{Storage, StorageError};

pub mod import;

/// The start of every exported file's name.
const NAME_START: &str = "visions-of-shuyi";

/// The word before a slot's number in the name of a file of one save.
const SLOT_WORD: &str = "slot";

/// The ending of the files Import asks for (0873): what an export writes.
pub const IMPORT_EXTENSION: &str = ".ron";

/// This build's version, as the stamp of an exported file names it.
// check-text: not player text (the build's version)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Text key of the place line's words before the folder's path.
pub const PLACE: &str = "save_files.place";

/// What one exported file holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Export {
    /// Every key of the profile: its slots, its suspend save, its options,
    /// keys and layout, the tips it has seen (the Profile screen's).
    Profile,
    /// Save slot `slot` alone (1 to [`SLOTS`](crate::save::SLOTS)), and
    /// with `settings` the profile's options, key bindings and layout
    /// (Load Game's).
    Slot {
        /// The slot's number.
        slot: usize,
        /// Whether the options and keys go with it.
        settings: bool,
    },
}

/// Whether the storage key `key` (without the profile's prefix) goes in a
/// file that holds `what`.
pub fn export_keys(what: Export) -> impl Fn(&str) -> bool {
    move |key| match what {
        Export::Profile => true,
        Export::Slot { slot, settings } => {
            key == slot_key(slot)
                || (settings && [SETTINGS_KEY, KEYBINDINGS_KEY, LAYOUT_KEY].contains(&key))
        }
    }
}

/// The file's name for `what`, of the profile `id` named `name`:
/// `visions-of-shuyi-<name>-<n>.ron`, or `…-<n>-slot-<NN>.ron` for one
/// save. The name is lower case with a `-` for each run of spaces, and
/// every character but `a`–`z` and a digit left out, so it is a file name
/// on every system; the profile's number keeps a name with none of those
/// apart.
pub fn export_name(name: &str, id: ProfileId, what: Export) -> String {
    let safe = safe_name(name);
    let name = if safe.is_empty() {
        String::new()
    } else {
        format!("-{safe}")
    };
    let slot = match what {
        Export::Slot { slot, .. } => format!("-{SLOT_WORD}-{slot:02}"),
        Export::Profile => String::new(),
    };
    // check-text: not player text (a file name)
    format!("{NAME_START}{name}-{}{slot}.ron", id.get())
}

/// `name` in lower case with a `-` for each run of spaces, and only
/// `a`–`z`, digits and those `-`.
fn safe_name(name: &str) -> String {
    let mut safe = String::new();
    for word in name.split_whitespace() {
        let word: String = word
            .chars()
            .map(|c| c.to_ascii_lowercase())
            .filter(char::is_ascii_alphanumeric)
            .collect();
        if word.is_empty() {
            continue;
        }
        if !safe.is_empty() {
            safe.push('-');
        }
        safe.push_str(&word);
    }
    safe
}

/// The stamp of this build: the game's version and its content's
/// fingerprint, read once per export or import (it reads every byte that
/// ships).
pub fn made() -> Made {
    Made::new(VERSION, trpg_content::bundle::install().fingerprint())
}

/// Whether the platform has files a player reaches, so a screen offers
/// export: a desktop's folder or a browser; not the Playdate.
pub fn can_export(facts: &PlatformFacts) -> bool {
    !facts.storage_place.is_unknown()
}

/// Where the saves are, to show: the folder on a desktop; `None` on the
/// web (an export is a download there) and where the platform doesn't
/// say.
pub fn place(facts: &PlatformFacts) -> Option<&str> {
    match &facts.storage_place {
        StoragePlace::Folder(path) => Some(path),
        _ => None,
    }
}

/// The keys of profile `id` in `computer`, read without its prefix.
struct ProfileKeys<'a> {
    computer: &'a dyn Storage,
    prefix: String,
}

impl ProfileKeys<'_> {
    /// Each of the profile's entries `keep` keeps, by key.
    fn entries(
        &self,
        keep: impl Fn(&str) -> bool,
    ) -> Result<BTreeMap<String, String>, StorageError> {
        let mut entries = BTreeMap::new();
        for full in self.computer.list()? {
            let Some(key) = full.strip_prefix(&self.prefix) else {
                continue;
            };
            if !keep(key) || key.starts_with(shuyi_ui::storage::bundle::RESERVED_PREFIX) {
                continue;
            }
            if let Some(value) = self.computer.read(&full)? {
                entries.insert(key.to_owned(), value);
            }
        }
        Ok(entries)
    }
}

/// Exports `what` of profile `id` as a file the player keeps: the file's
/// name, once it is offered (the platform says where it went in a later
/// frame, [`hear`]).
///
/// # Errors
///
/// When the storage can't be read.
pub fn export(ctx: &mut Ctx, id: ProfileId, what: Export) -> Result<String, StorageError> {
    let keys = ProfileKeys {
        computer: ctx.storage.computer(),
        prefix: id.prefix(),
    };
    let entries = keys.entries(export_keys(what))?;
    let name = ctx
        .profiles()
        .all()
        .iter()
        .find(|p| p.id == id)
        // check-text: not a data name (the player's own)
        .map(|p| p.name.clone())
        .unwrap_or_default();
    let file = export_name(&name, id, what);
    offer(ctx, &file, entries);
    Ok(file)
}

/// Offers the player a file called `file` holding `entries` (keys without
/// a profile's prefix), stamped with this build: what [`export`] does with
/// a profile's keys, and the demo's end with a save it never stored
/// (0914).
pub fn offer(ctx: &mut Ctx, file: &str, entries: BTreeMap<String, String>) {
    let text = SaveBundle::new(made(), entries).to_text();
    ctx.offer_file(file, text);
}

/// Where an exported file went, for the line under a screen's panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Exported {
    /// The browser downloaded the file called this.
    Downloaded(String),
    /// It was written in the exports folder under this name.
    Written(String),
    /// It couldn't be written.
    Failed,
}

impl Exported {
    /// The notice's text, in `ctx`'s language.
    pub fn text(&self, ctx: &Ctx) -> String {
        match self {
            Exported::Downloaded(file) => ctx.text_with("save_files.downloaded", &[("file", file)]),
            Exported::Written(file) => ctx.text_with("save_files.written", &[("file", file)]),
            Exported::Failed => ctx.text("save_files.failed").to_owned(),
        }
    }

    /// The folder the open-folder key opens after it: the exports folder
    /// once a file is written there, else the saves'.
    pub fn folder(this: Option<&Self>) -> Folder {
        match this {
            Some(Exported::Written(_)) => Folder::Exports,
            _ => Folder::Storage,
        }
    }
}

/// The last part of `path`, after its last `/` or `\`: the name the file
/// was written as (`a (2).ron`).
#[cfg(feature = "std")]
fn file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

/// Where the platform says an offered file went, as a screen's notice
/// takes it: `None` for a download (the screen names the file it offered).
#[cfg(feature = "std")]
fn exported_of(offered: &shuyi_platform::player_files::Offered) -> Option<Exported> {
    use shuyi_platform::player_files::Offered;
    match offered {
        Offered::Downloaded => None,
        Offered::Written(path) => Some(Exported::Written(file_name(path).to_owned())),
        _ => Some(Exported::Failed),
    }
}

/// The game's system hook: hands the Profile screen or the Load Game list
/// where the file it offered went, and the Import screen the file it
/// waits for (0873): the one picked, or one file dropped on the window.
/// A file dropped anywhere else is ignored.
#[cfg(feature = "std")]
pub fn hear(game: &mut crate::game::Game, event: shuyi_platform::SystemEvent) {
    use crate::screens::{ImportScreen, ProfileScreen, SlotPickerScreen};
    use shuyi_platform::SystemEvent;
    use shuyi_platform::player_files::ChosenFile;

    match event {
        SystemEvent::FileOffered(offered) => {
            let exported = exported_of(&offered);
            if let Some(screen) = game.screen_mut::<SlotPickerScreen>() {
                screen.heard(exported);
            } else if let Some(screen) = game.screen_mut::<ProfileScreen>() {
                screen.heard(exported);
            }
        }
        SystemEvent::FileChosen(chosen) => {
            let Some(screen) = game.screen_mut::<ImportScreen>() else {
                return;
            };
            match chosen {
                ChosenFile::Picked { name, text, .. } => screen.chosen(&name, text),
                ChosenFile::Cancelled => screen.not_chosen(true),
                _ => screen.not_chosen(false),
            }
        }
        SystemEvent::FilesDropped(mut files) if files.len() == 1 => {
            if let (Some(screen), Some(file)) = (game.screen_mut::<ImportScreen>(), files.pop()) {
                let name = file.name().to_owned();
                screen.dropped(&name, file.into_bytes());
            }
        }
        _ => {}
    }
}

/// What a screen keeps of its exports: the file offered last, until the
/// platform says where it went, and the notice then.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExportNotice {
    /// The file offered and not yet heard of.
    pending: Option<String>,
    /// Where the last one went.
    shown: Option<Exported>,
}

impl ExportNotice {
    /// The file `file` was offered.
    pub fn offered(&mut self, file: String) {
        self.pending = Some(file);
        self.shown = None;
    }

    /// The platform's answer: `None` for a download (named by the file
    /// offered), else where it went.
    pub fn heard(&mut self, exported: Option<Exported>) {
        let pending = self.pending.take();
        self.shown = match exported {
            Some(exported) => Some(exported),
            None => pending.map(Exported::Downloaded),
        };
    }

    /// Where the last file went, if the platform said.
    pub fn shown(&self) -> Option<&Exported> {
        self.shown.as_ref()
    }

    /// Forgets the notice (the player moved on).
    pub fn clear(&mut self) {
        self.shown = None;
    }
}

#[cfg(test)]
mod tests;
