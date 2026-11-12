//! Importing a file of saves (ticket 0873, `docs/design/options.md`,
//! *Saves outside the game*): what this build can read of it, where it
//! goes, and writing it.
//!
//! - **The check** ([`GameCheck`]): a verdict per key of the file, before
//!   anything is written. A save of this build's
//!   [`SAVE_VERSION`](trpg_core::SAVE_VERSION) imports whatever build made
//!   the file (most patches change no saved type); a save of another
//!   version is refused, as loading one is, since nothing migrates saves
//!   yet (ADR-0039). The options, key bindings and layout are read as the
//!   game reads them; `last_save` is kept only with the save it names
//!   ([`plan`]).
//! - **Where it goes** ([`Report::read`], Nick in 0872): through the
//!   Profile screen, always a new profile; through Load Game, a file of
//!   one save goes into this profile (the first empty slot, else the slot
//!   under the cursor) without its options and keys, and any other file
//!   becomes a new profile, after a warning.
//! - **Writing it** ([`Report::apply`]): every entry that is ready, all
//!   or nothing (the engine's `apply`); a new profile that couldn't be
//!   filled is taken away again.

use crate::prelude::*;

use shuyi_core::save::{SaveFormat, SaveHeader};
use shuyi_ui::input::PlayerKeys;
use shuyi_ui::input::bindings::PLAYER_KEYS_VERSION;
use shuyi_ui::storage::bundle::{self, BundleError, EntryCheck, FormatCheck, ImportPlan, Verdict};
use trpg_content::Content;
use trpg_core::{SaveFile, SavePoint};

use super::{NAME_START, SLOT_WORD, made};
use crate::input::{Action, KeymapDef, Layout};
use crate::profiles::{MAX_NAME_CHARS, MAX_PROFILES, ProfileError, ProfileId, SET_UP_KEY};
use crate::save::{self, LAST_SAVE_KEY, SLOTS, SUSPEND_KEY, Slot, slot_key};
use crate::screen::{Ctx, KEYBINDINGS_KEY, LAYOUT_KEY};
use crate::settings::{READ_VERSIONS, SETTINGS_KEY, SETTINGS_VERSION, Settings};
use crate::storage::{MemoryStorage, Storage, StorageError};
use crate::tips::TIPS_SEEN_KEY;

/// The save slot whose key is `key` (`slot_07` is 7), if it is one.
pub fn slot_of(key: &str) -> Option<usize> {
    (1..=SLOTS).find(|&n| slot_key(n) == key)
}

/// Whether `key` holds a save: a slot's or the suspend save.
fn is_save(key: &str) -> bool {
    key == SUSPEND_KEY || slot_of(key).is_some()
}

/// The keys of the options, key bindings and layout, which a save
/// imported through Load Game leaves out.
const OPTION_KEYS: [&str; 3] = [SETTINGS_KEY, KEYBINDINGS_KEY, LAYOUT_KEY];

/// What this build can do with each entry of a file of saves: the game's
/// knowledge of its own keys, for the engine's `plan_import`.
#[derive(Debug, Clone, Copy)]
pub struct GameCheck<'a> {
    content: &'a Content,
}

impl<'a> GameCheck<'a> {
    /// The check with the game's `content`: the chapters a suspend save
    /// needs, the keymap the key bindings are read against.
    pub fn new(content: &'a Content) -> Self {
        Self { content }
    }
}

impl EntryCheck for GameCheck<'_> {
    fn check(&self, key: &str, text: &str) -> Verdict {
        let keymap = &self.content.keymap;
        if slot_of(key).is_some() {
            // As the Load Game list reads a slot: a chapter or world map
            // save (`save::slot`).
            let fits = |save: &SaveFile| {
                matches!(save.point, SavePoint::ChapterCleared | SavePoint::WorldMap)
            };
            return save_check(key, text, fits);
        }
        match key {
            // A battle of a chapter the content has, as Continue needs.
            SUSPEND_KEY => save_check(key, text, |save| {
                matches!(save.point, SavePoint::Battle(_))
                    && self.content.chapters.contains_key(&save.campaign.chapter)
            }),
            SETTINGS_KEY => versioned::<SettingsFormat>(
                text,
                |v| READ_VERSIONS.contains(&v),
                |t| Settings::from_ron(t).is_ok(),
            ),
            KEYBINDINGS_KEY => versioned::<KeysFormat>(
                text,
                |v| (1..=PLAYER_KEYS_VERSION).contains(&v),
                |t| keys_read(t, keymap),
            ),
            LAYOUT_KEY => {
                let layout = Layout::from_name(text.trim());
                if layout.is_some_and(|l| keymap.has_layout(l)) {
                    Verdict::Ready
                } else {
                    Verdict::Unreadable
                }
            }
            // Free text, and the note of the save written last (kept with
            // that save only: `plan`).
            TIPS_SEEN_KEY | SET_UP_KEY | LAST_SAVE_KEY => Verdict::Ready,
            _ => Verdict::Unknown,
        }
    }
}

/// A save's verdict: the engine's for its version and form, then
/// `Unreadable` unless `fits` takes it.
fn save_check(key: &str, text: &str, fits: impl Fn(&SaveFile) -> bool) -> Verdict {
    let verdict = FormatCheck::<SaveFile>::new(|_| true).check(key, text);
    if verdict != Verdict::Ready {
        return verdict;
    }
    match save::decode(text) {
        Ok(save) if fits(&save) => Verdict::Ready,
        _ => Verdict::Unreadable,
    }
}

/// The settings' saved form, by its version.
struct SettingsFormat;

impl SaveFormat for SettingsFormat {
    // check-text: not player text (the struct name in the saved form)
    const NAME: &'static str = "Settings";
    const VERSION: u32 = SETTINGS_VERSION;
}

/// The key bindings' saved form (the engine's), by its version.
struct KeysFormat;

impl SaveFormat for KeysFormat {
    // check-text: not player text (the struct name in the saved form)
    const NAME: &'static str = "PlayerKeys";
    const VERSION: u32 = PLAYER_KEYS_VERSION;
}

/// The verdict on a text of format `F`: `Incompatible` for a version
/// `reads_version` refuses, `Ready` if `reads` reads it, else
/// `Unreadable`.
fn versioned<F: SaveFormat>(
    text: &str,
    reads_version: impl Fn(u32) -> bool,
    reads: impl Fn(&str) -> bool,
) -> Verdict {
    let Ok(header) = ron::from_str::<SaveHeader<F>>(text) else {
        return Verdict::Unreadable;
    };
    if !reads_version(header.version) {
        return Verdict::Incompatible {
            found: header.version,
            wants: F::VERSION,
        };
    }
    if reads(text) {
        Verdict::Ready
    } else {
        Verdict::Unreadable
    }
}

/// Whether key bindings read: the engine repairs what it can, so they
/// don't only when nothing was left but the defaults, with a warning.
fn keys_read(text: &str, keymap: &KeymapDef) -> bool {
    let (keys, warnings) = PlayerKeys::<Action>::from_ron(text, keymap);
    warnings.is_empty() || keys != PlayerKeys::default()
}

/// What importing `text` into `storage` would do: the engine's plan with
/// [`GameCheck`], and `last_save` kept only when the save it names is
/// imported too or `storage` holds it.
///
/// # Errors
///
/// When `text` isn't a file of saves this build reads, or the storage
/// fails.
pub fn plan(
    text: &str,
    storage: &dyn Storage,
    content: &Content,
) -> Result<ImportPlan, BundleError> {
    let check = GameCheck::new(content);
    let mut plan = bundle::plan_import(text, storage, &check, &made())?;
    let ready: Vec<String> = plan.ready().map(|e| e.key.clone()).collect();
    let last = plan
        .entries
        .iter_mut()
        .find(|e| e.key == LAST_SAVE_KEY && e.verdict == Verdict::Ready);
    if let Some(last) = last {
        let named = last.text.trim();
        let held = matches!(storage.read(named), Ok(Some(_)));
        if !held && !ready.iter().any(|k| k == named) {
            last.verdict = Verdict::Unreadable;
        }
    }
    Ok(plan)
}

/// The screen an import was asked from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// The Profile screen.
    Profile,
    /// Load Game, its cursor on slot `cursor` (1 to [`SLOTS`]).
    LoadGame {
        /// The slot under the cursor.
        cursor: usize,
    },
}

/// Where a file goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    /// A new profile, called this; `warned` through Load Game, which says
    /// first that a whole profile becomes a new one.
    NewProfile {
        /// Its name.
        name: String,
        /// Whether it came through Load Game.
        warned: bool,
    },
    /// A new profile, but there are [`MAX_PROFILES`] already.
    Full,
    /// This profile's slot `slot`, replacing its save if `replaces`.
    Slot {
        /// The slot's number.
        slot: usize,
        /// Whether it holds a save (all 30 do).
        replaces: bool,
    },
}

/// One save of the file, as the report shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveLine {
    /// Its slot in the file; `None` for the suspend save.
    pub slot: Option<usize>,
    /// The chapter's title, as Load Game shows it, if it reads.
    pub chapter: Option<String>,
    /// Seconds played, if it reads.
    pub playtime_s: Option<u64>,
    /// What this build can do with it.
    pub verdict: Verdict,
}

/// What importing a file would do, before anything is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// The file's name.
    pub file: String,
    /// The engine's plan, its keys as they will be written.
    pub plan: ImportPlan,
    /// Where it goes.
    pub to: Destination,
    /// Its saves, slots in order then the suspend save.
    pub saves: Vec<SaveLine>,
    /// The verdict on its options, key bindings and layout (the first
    /// that isn't ready); `None` if it has none.
    pub options: Option<Verdict>,
}

/// What an import did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Imported {
    /// The new profile `name` holds `saves` of the file's saves;
    /// `refused` couldn't be read.
    Profile {
        /// Its name.
        name: String,
        /// The saves imported.
        saves: usize,
        /// The saves refused.
        refused: usize,
    },
    /// The save went into this profile's slot.
    Slot(usize),
    /// Nothing could be; `refused` saves were refused.
    Nothing {
        /// The saves refused.
        refused: usize,
    },
}

/// Why an import wrote nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportFailed {
    /// There are [`MAX_PROFILES`] already.
    Full,
    /// The storage failed; nothing changed.
    Storage,
}

impl Report {
    /// The report on the file `file` holding `text`, asked for through
    /// `channel` with the profile in use in `ctx`.
    ///
    /// # Errors
    ///
    /// When `text` isn't a file of saves this build reads.
    pub fn read(ctx: &Ctx, file: &str, text: &str, channel: Channel) -> Result<Self, BundleError> {
        let content = &ctx.game.content;
        // Checked as a new profile, which holds nothing.
        let mut plan = plan(text, &MemoryStorage::new(), content)?;
        let saves = save_lines(ctx, &plan);
        let options = plan
            .entries
            .iter()
            .filter(|e| OPTION_KEYS.contains(&e.key.as_str()))
            .map(|e| e.verdict)
            .reduce(|first, v| if first == Verdict::Ready { v } else { first });
        let one_slot = matches!(saves.as_slice(), [SaveLine { slot: Some(_), .. }]);
        let to = match channel {
            Channel::LoadGame { cursor } if one_slot => into_slot(ctx, &mut plan, cursor),
            _ => match new_name(ctx, file) {
                Some(name) => Destination::NewProfile {
                    name,
                    warned: channel != Channel::Profile,
                },
                None => Destination::Full,
            },
        };
        Ok(Self {
            file: file.to_owned(),
            plan,
            to,
            saves,
            options,
        })
    }

    /// How many of the file's saves this build refuses.
    pub fn refused(&self) -> usize {
        self.saves
            .iter()
            .filter(|s| s.verdict != Verdict::Ready)
            .count()
    }

    /// Writes every entry that is ready where [`to`](Self::to) says, all
    /// or nothing.
    ///
    /// # Errors
    ///
    /// [`ImportFailed`]: nothing was written, and no profile added.
    pub fn apply(&self, ctx: &mut Ctx) -> Result<Imported, ImportFailed> {
        let refused = self.refused();
        let name = match &self.to {
            Destination::Full => return Err(ImportFailed::Full),
            Destination::Slot { slot, .. } => {
                let key = slot_key(*slot);
                let applied = bundle::apply(&self.plan, &mut ctx.storage, |e| e.key == key)
                    .map_err(|_| ImportFailed::Storage)?;
                return Ok(if applied.written.is_empty() {
                    Imported::Nothing { refused }
                } else {
                    Imported::Slot(*slot)
                });
            }
            Destination::NewProfile { name, .. } => name,
        };
        if self.plan.ready().next().is_none() {
            return Ok(Imported::Nothing { refused });
        }
        let id = ctx.add_profile(name).map_err(|e| match e {
            ProfileError::TooMany => ImportFailed::Full,
            _ => ImportFailed::Storage,
        })?;
        let mut store = OtherProfile {
            computer: ctx.storage.computer_mut(),
            id,
        };
        let Ok(applied) = bundle::apply(&self.plan, &mut store, |_| true) else {
            // Its keys are deleted with it.
            let _ = ctx.delete_profile(id);
            return Err(ImportFailed::Storage);
        };
        Ok(Imported::Profile {
            name: name.clone(),
            saves: applied.written.iter().filter(|k| is_save(k)).count(),
            refused,
        })
    }
}

/// The file's saves, as the report shows them: each slot's chapter and
/// playtime as the Load Game list reads them, then the suspend save's.
fn save_lines(ctx: &Ctx, plan: &ImportPlan) -> Vec<SaveLine> {
    let (content, words) = (&ctx.game.content, ctx.words());
    let mut file = MemoryStorage::new();
    for entry in plan.entries.iter().filter(|e| is_save(&e.key)) {
        // A memory storage takes any valid key, as the bundle's are.
        let _ = file.write(&entry.key, &entry.text);
    }
    let mut lines: Vec<SaveLine> = Vec::new();
    for entry in &plan.entries {
        let (chapter, playtime_s) = if let Some(n) = slot_of(&entry.key) {
            match save::slot(&file, content, words, n) {
                Slot::Saved(s) if entry.verdict == Verdict::Ready => {
                    (Some(s.title(ctx)), Some(s.playtime_s))
                }
                _ => (None, None),
            }
        } else if entry.key == SUSPEND_KEY {
            match save::read(&file, SUSPEND_KEY) {
                Ok(Some(s)) if entry.verdict == Verdict::Ready => (
                    Some(save::chapter_title(content, words, &s.campaign)),
                    Some(s.campaign.playtime_s),
                ),
                _ => (None, None),
            }
        } else {
            continue;
        };
        lines.push(SaveLine {
            slot: slot_of(&entry.key),
            chapter,
            playtime_s,
            verdict: entry.verdict,
        });
    }
    // The suspend save sorts before the slots by key: put it last.
    lines.sort_by_key(|l| l.slot.is_none());
    lines
}

/// The one save of `plan` going into this profile (Nick, 0872): the first
/// empty slot; if none is, the slot under the cursor, replacing it.
/// `plan`'s slot entry takes the slot's key.
fn into_slot(ctx: &Ctx, plan: &mut ImportPlan, cursor: usize) -> Destination {
    let empty = (1..=SLOTS).find(|&n| matches!(ctx.storage.read(&slot_key(n)), Ok(None)));
    let slot = empty.unwrap_or(cursor);
    let replaces = empty.is_none();
    if let Some(entry) = plan.entries.iter_mut().find(|e| slot_of(&e.key).is_some()) {
        entry.key = slot_key(slot);
        entry.replaces = replaces;
    }
    Destination::Slot { slot, replaces }
}

/// The name of the profile a file becomes: the one in the file's name if
/// it is free, else with a number after it, else the engine's numbered
/// name; `None` with [`MAX_PROFILES`] already.
fn new_name(ctx: &Ctx, file: &str) -> Option<String> {
    let profiles = ctx.profiles();
    if profiles.all().len() >= MAX_PROFILES {
        return None;
    }
    let free = |name: &String| profiles.clone().add(name).is_ok();
    let base = name_in(file);
    let named = (!base.is_empty()).then(|| base.clone());
    // Fewer than `MAX_PROFILES` profiles take fewer names than either list
    // has: one of each is free.
    let numbered_base = (2..=MAX_PROFILES).filter(|_| !base.is_empty());
    let numbered_base = numbered_base.map(|n| with_number(&base, n));
    let numbered = (1..=MAX_PROFILES)
        .map(|n| ctx.text_with(shuyi_ui::ctx::PROFILE_NUMBERED_TEXT, &[("n", &n)]));
    named
        .into_iter()
        .chain(numbered_base)
        .chain(numbered)
        .find(free)
}

/// `base` with ` <n>` after it, `base` cut short to stay a name.
fn with_number(base: &str, n: usize) -> String {
    // check-text: not player text (a number after the player's own name)
    let number = format!(" {n}");
    let room = MAX_NAME_CHARS.saturating_sub(number.chars().count());
    let cut: String = base.chars().take(room).collect();
    format!("{}{number}", cut.trim_end())
}

/// The profile's name in an exported file's name, as `export_name` wrote
/// it: `visions-of-shuyi-odo-kai-2.ron` (or `…-2-slot-04 (3).ron`) gives
/// `Odo Kai`. Each word capitalized, at most a name's length; empty if the
/// file names none.
pub fn name_in(file: &str) -> String {
    let stem = file.strip_suffix(".ron").unwrap_or(file);
    // The desktop's ` (2)` after a second file of one name.
    let stem = stem.split(" (").next().unwrap_or(stem);
    let Some(rest) = stem.strip_prefix(NAME_START) else {
        return String::new();
    };
    let number = |w: &&str| !w.is_empty() && w.bytes().all(|b| b.is_ascii_digit());
    let mut words: Vec<&str> = rest.split('-').filter(|w| !w.is_empty()).collect();
    if let [.., slot, n] = words.as_slice()
        && *slot == SLOT_WORD
        && number(n)
    {
        words.truncate(words.len() - 2);
    }
    // The profile's number.
    if words.last().is_some_and(number) {
        words.pop();
    }
    let words: Vec<String> = words.iter().map(|w| capitalized(w)).collect();
    let name: String = words.join(" ").chars().take(MAX_NAME_CHARS).collect();
    name.trim_end().to_owned()
}

/// `word` with its first letter in capitals.
fn capitalized(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// The storage of the profile `id`, which isn't the one in use: its keys
/// read and written with its prefix in the computer's storage.
#[derive(Debug)]
struct OtherProfile<'a> {
    computer: &'a mut dyn Storage,
    id: ProfileId,
}

impl Storage for OtherProfile<'_> {
    fn read(&self, key: &str) -> Result<Option<String>, StorageError> {
        self.computer.read(&self.id.key(key))
    }

    fn write(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        self.computer.write(&self.id.key(key), value)
    }

    fn delete(&mut self, key: &str) -> Result<(), StorageError> {
        self.computer.delete(&self.id.key(key))
    }

    fn list(&self) -> Result<Vec<String>, StorageError> {
        let prefix = self.id.prefix();
        let keys = self.computer.list()?;
        Ok(keys
            .iter()
            .filter_map(|k| k.strip_prefix(&prefix).map(str::to_owned))
            .collect())
    }
}

#[cfg(test)]
mod tests;
