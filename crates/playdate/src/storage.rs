//! The game's [`Storage`] on the Playdate (ticket 1033, ADR-0009): one
//! `<key>.ron` file per key in the game's data folder, as `shuyi-app`'s
//! `FileStorage` keeps them in the PC's. Profiles and saves are kept
//! exactly as on the PC (ADR-0069): this only stores the text.
//!
//! A write puts the new value in `<key>.ron.tmp`, copies the old `<key>.ron`
//! (when it is a good copy: not empty, not all zero bytes, UTF-8) to
//! `<key>.ron.bak`, each flushed by [`Files::write`], then moves the `.tmp`
//! onto the `.ron`. Where the SDK won't move a file onto another, the old
//! `.ron` goes first and its `.bak` holds it meanwhile. A read of a `.ron`
//! that is missing or not a good copy, beside a good `.bak`, reads the `.bak`.
//! So after a power cut (a flat battery, the console crashing) in a write, the
//! key reads the old value or the new one, and a write that returned `Ok`
//! reads the new one (ticket 1196, the engine's `power_cut_suite`). A `.tmp`
//! is there only between a cut and the next start's first write, which
//! removes it. A value that is not a good copy itself (`""`) keeps no backup,
//! so it reads back as written, and has only the plain move's protection.
//!
//! The SDK's file calls are behind [`Files`] (`src/pd/` has the real
//! ones), so the rules here are tested on any machine, and the debug
//! build also runs `trpg-ui`'s conformance suite on the real files in the
//! Simulator ([`check`]).

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use trpg_ui::storage::is_valid_key;
use trpg_ui::{Storage, StorageError};

/// What is at a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    /// A file.
    File,
    /// A folder.
    Folder,
}

/// The file calls [`FileStorage`] needs, on paths in the game's data
/// folder; errors are the SDK's message.
pub trait Files {
    /// What is at `path`, or `None` if nothing is.
    fn entry(&self, path: &str) -> Option<Entry>;
    /// The whole of file `path`.
    fn read(&self, path: &str) -> Result<Vec<u8>, String>;
    /// Replaces file `path` with `bytes`, making it if it is not there, and
    /// flushes them to the disk.
    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String>;
    /// Moves file `from` to `to`.
    fn rename(&mut self, from: &str, to: &str) -> Result<(), String>;
    /// Deletes file `path`.
    fn unlink(&mut self, path: &str) -> Result<(), String>;
    /// Makes folder `path`.
    fn mkdir(&mut self, path: &str) -> Result<(), String>;
    /// The names in folder `folder` (`""`: the data folder itself), a
    /// folder's with a `/` after it.
    fn list(&self, folder: &str) -> Result<Vec<String>, String>;
}

/// The files behind a borrow are the same files.
impl<F: Files> Files for &mut F {
    fn entry(&self, path: &str) -> Option<Entry> {
        (**self).entry(path)
    }
    fn read(&self, path: &str) -> Result<Vec<u8>, String> {
        (**self).read(path)
    }
    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        (**self).write(path, bytes)
    }
    fn rename(&mut self, from: &str, to: &str) -> Result<(), String> {
        (**self).rename(from, to)
    }
    fn unlink(&mut self, path: &str) -> Result<(), String> {
        (**self).unlink(path)
    }
    fn mkdir(&mut self, path: &str) -> Result<(), String> {
        (**self).mkdir(path)
    }
    fn list(&self, folder: &str) -> Result<Vec<String>, String> {
        (**self).list(folder)
    }
}

/// The suffix of a key's file.
const EXT: &str = ".ron";
/// The suffix of the file a key's new text is written to first.
const TMP: &str = ".ron.tmp";
/// The suffix of the file that holds a key's value before the last write.
const BAK: &str = ".ron.bak";

/// A [`Storage`] of one file per key in a folder of the game's data
/// folder, which keeps a key's last good copy (the module's docs).
#[derive(Debug)]
pub struct FileStorage<F> {
    files: F,
    /// The folder, `""` or ending in `/`.
    folder: String,
    /// Whether a write has removed the `.tmp` files a cut left.
    swept: bool,
}

impl<F: Files> FileStorage<F> {
    /// The keys in the data folder itself: the game's.
    pub fn new(files: F) -> Self {
        Self {
            files,
            folder: String::new(),
            swept: false,
        }
    }

    /// The keys in `folder` (a name without `/`) of the data folder.
    pub fn in_folder(files: F, folder: &str) -> Self {
        Self {
            files,
            folder: format!("{folder}/"),
            swept: false,
        }
    }

    fn path(&self, key: &str) -> String {
        format!("{}{key}{EXT}", self.folder)
    }

    fn tmp_path(&self, key: &str) -> String {
        format!("{}{key}{TMP}", self.folder)
    }

    fn bak_path(&self, key: &str) -> String {
        format!("{}{key}{BAK}", self.folder)
    }

    /// The backup's text, if it is a good copy.
    fn good_backup(&self, key: &str) -> Option<String> {
        let bytes = self.files.read(&self.bak_path(key)).ok()?;
        if is_good(&bytes) {
            String::from_utf8(bytes).ok()
        } else {
            None
        }
    }

    /// Removes `path`; not an error if it isn't there.
    fn remove_if_there(&mut self, path: &str) -> Result<(), StorageError> {
        if self.files.entry(path).is_none() {
            return Ok(());
        }
        self.files.unlink(path).map_err(|e| backend(path, &e))
    }

    /// Removes the `.tmp` files a cut left in the folder.
    fn sweep(&mut self) -> Result<(), StorageError> {
        let folder = self.folder.trim_end_matches('/');
        let names = self.files.list(folder).map_err(|e| backend(folder, &e))?;
        for name in names.iter().filter(|name| name.ends_with(TMP)) {
            let path = format!("{}{name}", self.folder);
            self.remove_if_there(&path)?;
        }
        Ok(())
    }
}

impl<F: Files + core::fmt::Debug> Storage for FileStorage<F> {
    fn read(&self, key: &str) -> Result<Option<String>, StorageError> {
        check_key(key)?;
        let path = self.path(key);
        let main = match self.files.entry(&path) {
            None => None,
            Some(Entry::Folder) => return Err(backend(&path, "is a folder")),
            Some(Entry::File) => Some(self.files.read(&path).map_err(|e| backend(&path, &e))?),
        };
        if !main.as_deref().is_some_and(is_good)
            && let Some(backup) = self.good_backup(key)
        {
            return Ok(Some(backup));
        }
        main.map(String::from_utf8)
            .transpose()
            .map_err(|_| backend(&path, "is not UTF-8 text"))
    }

    fn write(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        check_key(key)?;
        let folder = self.folder.trim_end_matches('/');
        if !folder.is_empty() && self.files.entry(folder).is_none() {
            self.files.mkdir(folder).map_err(|e| backend(folder, &e))?;
        }
        if !self.swept {
            self.sweep()?;
            self.swept = true;
        }
        let (tmp, path, bak) = (self.tmp_path(key), self.path(key), self.bak_path(key));
        self.files
            .write(&tmp, value.as_bytes())
            .map_err(|e| backend(&tmp, &e))?;
        if !is_good(value.as_bytes()) {
            self.remove_if_there(&bak)?;
        } else if self.files.entry(&path) == Some(Entry::File) {
            let old = self.files.read(&path).map_err(|e| backend(&path, &e))?;
            if is_good(&old) {
                self.files
                    .write(&bak, &old)
                    .map_err(|e| backend(&bak, &e))?;
            }
        }
        if self.files.rename(&tmp, &path).is_ok() {
            return Ok(());
        }
        // A file system that will not move a file onto another: the old
        // one goes first, and its backup holds it meanwhile.
        self.remove_if_there(&path)?;
        self.files
            .rename(&tmp, &path)
            .map_err(|e| backend(&path, &e))
    }

    fn delete(&mut self, key: &str) -> Result<(), StorageError> {
        check_key(key)?;
        let (bak, path) = (self.bak_path(key), self.path(key));
        self.remove_if_there(&bak)?;
        self.remove_if_there(&path)
    }

    fn list(&self) -> Result<Vec<String>, StorageError> {
        let folder = self.folder.trim_end_matches('/');
        if !folder.is_empty() && self.files.entry(folder).is_none() {
            return Ok(Vec::new());
        }
        let names = self.files.list(folder).map_err(|e| backend(folder, &e))?;
        let key = |name: &String| {
            let lone_backup = || {
                name.strip_suffix(BAK).filter(|key| {
                    !names.contains(&format!("{key}{EXT}")) && self.good_backup(key).is_some()
                })
            };
            name.strip_suffix(EXT)
                .or_else(lone_backup)
                .filter(|key| is_valid_key(key))
                .map(String::from)
        };
        Ok(names.iter().filter_map(key).collect())
    }
}

/// Whether a file's bytes are a good copy of a value: not empty, not all
/// zero bytes (what a power cut can leave), and UTF-8.
fn is_good(bytes: &[u8]) -> bool {
    !bytes.is_empty() && bytes.iter().any(|&b| b != 0) && core::str::from_utf8(bytes).is_ok()
}

fn check_key(key: &str) -> Result<(), StorageError> {
    if is_valid_key(key) {
        Ok(())
    } else {
        Err(StorageError::InvalidKey(key.into()))
    }
}

fn backend(path: &str, message: &str) -> StorageError {
    // check-text: not player text (for the log)
    StorageError::Backend(format!("{path}: {message}"))
}

/// The folder of the data folder [`check`] runs in, away from the game's
/// own keys.
pub const CHECK_FOLDER: &str = "storage-check";

/// Runs `trpg-ui`'s storage conformance suite on `files`, in
/// [`CHECK_FOLDER`], and leaves the folder empty. Panics if it fails.
/// The debug build runs it at start (the Simulator's real files).
pub fn check<F: Files + core::fmt::Debug>(files: F) {
    let mut storage = FileStorage::in_folder(files, CHECK_FOLDER);
    let clear = |storage: &mut FileStorage<F>| {
        let keys = storage.list().unwrap_or_else(|e| panic!("{e}"));
        for key in keys {
            storage.delete(&key).unwrap_or_else(|e| panic!("{e}"));
        }
    };
    clear(&mut storage);
    trpg_ui::storage::conformance_suite(&mut storage);
    clear(&mut storage);
}

#[cfg(test)]
mod tests {
    use alloc::collections::{BTreeMap, BTreeSet};
    use alloc::string::ToString;

    use trpg_ui::storage::{PowerCutRig, power_cut_suite};

    use super::*;

    /// Files in memory, laid out as the SDK's.
    #[derive(Debug, Default)]
    struct MemoryFiles {
        files: BTreeMap<String, Vec<u8>>,
        folders: BTreeSet<String>,
        /// Whether `rename` refuses to move a file onto another.
        no_replace: bool,
        /// Every path written and every folder made, in order.
        calls: Vec<String>,
        /// The `write`s, `rename`s, `unlink`s and `mkdir`s left before the
        /// power goes, if a cut is pending.
        ops_left: Option<u32>,
        /// Whether the power has gone.
        off: bool,
    }

    impl MemoryFiles {
        /// One operation: fails at and after the cut.
        fn op(&mut self) -> Result<(), String> {
            match self.ops_left {
                Some(0) => {
                    self.off = true;
                    return Err("the power is off".into());
                }
                Some(left) => self.ops_left = Some(left - 1),
                None => {}
            }
            Ok(())
        }
    }

    impl Files for MemoryFiles {
        fn entry(&self, path: &str) -> Option<Entry> {
            if self.files.contains_key(path) {
                Some(Entry::File)
            } else {
                self.folders.contains(path).then_some(Entry::Folder)
            }
        }

        fn read(&self, path: &str) -> Result<Vec<u8>, String> {
            self.files.get(path).cloned().ok_or("no such file".into())
        }

        fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
            let folder = path.rsplit_once('/').map_or("", |(folder, _)| folder);
            if !folder.is_empty() && !self.folders.contains(folder) {
                return Err("no such folder".into());
            }
            let was_on = !self.off;
            if let Err(power) = self.op() {
                // The write the power goes in empties the file and is torn.
                if was_on {
                    let torn = bytes[..bytes.len() / 2].to_vec();
                    self.files.insert(path.into(), torn);
                }
                return Err(power);
            }
            self.calls.push(format!("write {path}"));
            self.files.insert(path.into(), bytes.to_vec());
            Ok(())
        }

        fn rename(&mut self, from: &str, to: &str) -> Result<(), String> {
            self.op()?;
            if self.no_replace && self.files.contains_key(to) {
                return Err("file exists".into());
            }
            let bytes = self.files.remove(from).ok_or("no such file")?;
            self.files.insert(to.into(), bytes);
            Ok(())
        }

        fn unlink(&mut self, path: &str) -> Result<(), String> {
            self.op()?;
            self.files
                .remove(path)
                .map(|_| ())
                .ok_or("no such file".into())
        }

        fn mkdir(&mut self, path: &str) -> Result<(), String> {
            self.op()?;
            if !self.folders.insert(path.into()) {
                return Err("folder exists".into());
            }
            self.calls.push(format!("mkdir {path}"));
            Ok(())
        }

        fn list(&self, folder: &str) -> Result<Vec<String>, String> {
            let prefix = if folder.is_empty() {
                String::new()
            } else {
                format!("{folder}/")
            };
            let files = self.files.keys().filter_map(|path| {
                let name = path.strip_prefix(&prefix)?;
                (!name.contains('/')).then(|| name.to_string())
            });
            let folders = self
                .folders
                .iter()
                .filter(|_| folder.is_empty())
                .map(|f| format!("{f}/"));
            Ok(files.chain(folders).collect())
        }
    }

    /// A storage over [`MemoryFiles`], which can lose the power.
    #[derive(Debug)]
    struct Rig {
        storage: FileStorage<MemoryFiles>,
    }

    impl Rig {
        fn new(no_replace: bool) -> Self {
            let files = MemoryFiles {
                no_replace,
                ..MemoryFiles::default()
            };
            Self {
                storage: FileStorage::new(files),
            }
        }
    }

    impl PowerCutRig for Rig {
        fn storage(&mut self) -> &mut dyn Storage {
            &mut self.storage
        }

        fn cut_after(&mut self, ops: u32) {
            self.storage.files.ops_left = Some(ops);
        }

        fn restart(&mut self) {
            let files = core::mem::take(&mut self.storage.files);
            self.storage = FileStorage::new(MemoryFiles {
                ops_left: None,
                off: false,
                ..files
            });
        }
    }

    #[test]
    fn passes_the_power_cut_suite() {
        power_cut_suite(&mut Rig::new(false));
    }

    #[test]
    fn passes_the_power_cut_suite_where_a_file_cannot_be_moved_onto_another() {
        power_cut_suite(&mut Rig::new(true));
    }

    #[test]
    fn passes_the_conformance_suite_in_the_data_folder_and_in_a_folder() {
        trpg_ui::storage::conformance_suite(&mut FileStorage::new(MemoryFiles::default()));
        trpg_ui::storage::conformance_suite(&mut FileStorage::in_folder(
            MemoryFiles::default(),
            "saves",
        ));
    }

    /// The device's file system may not move a file onto another.
    #[test]
    fn passes_where_a_file_cannot_be_moved_onto_another() {
        let files = MemoryFiles {
            no_replace: true,
            ..MemoryFiles::default()
        };
        trpg_ui::storage::conformance_suite(&mut FileStorage::new(files));
    }

    /// One `<key>.ron` per key, no `.tmp` left behind; other files and
    /// folders in the data folder are not keys.
    #[test]
    fn keys_are_ron_files_and_nothing_else_is_a_key() {
        let mut storage = FileStorage::new(MemoryFiles::default());
        storage.write("settings", "()").unwrap();
        storage
            .files
            .files
            .insert("load-report.txt".into(), Vec::new());
        storage
            .files
            .files
            .insert("Bad Name.ron".into(), Vec::new());
        storage.files.folders.insert("storage-check".into());
        let names: Vec<&String> = storage.files.files.keys().collect();
        assert_eq!(names, ["Bad Name.ron", "load-report.txt", "settings.ron"]);
        assert_eq!(storage.list().unwrap(), ["settings"]);
    }

    #[test]
    fn the_check_runs_in_its_own_folder_and_leaves_it_empty() {
        let mut files = MemoryFiles::default();
        files
            .files
            .insert("settings.ron".into(), b"(theme: 1)".to_vec());
        // Left over from a run that stopped half way.
        files.folders.insert(CHECK_FOLDER.into());
        let left_over = format!("{CHECK_FOLDER}/settings.ron");
        files.files.insert(left_over, Vec::new());
        check(&mut files);
        let names: Vec<&String> = files.files.keys().collect();
        assert_eq!(names, ["settings.ron"]);
        assert_eq!(files.files["settings.ron"], b"(theme: 1)");
    }

    /// A write goes to `<key>.ron.tmp` first; a folder is made only
    /// once, and the data folder itself never.
    #[test]
    fn writes_go_through_a_tmp_file_and_make_a_folder_once() {
        let mut storage = FileStorage::new(MemoryFiles::default());
        storage.write("settings", "()").unwrap();
        storage.write("settings", "(a)").unwrap();
        assert_eq!(
            storage.files.calls,
            [
                "write settings.ron.tmp",
                "write settings.ron.tmp",
                "write settings.ron.bak"
            ]
        );
        let mut files = MemoryFiles::default();
        let mut storage = FileStorage::in_folder(&mut files, "saves");
        storage.write("slot", "()").unwrap();
        storage.write("slot", "(a)").unwrap();
        assert_eq!(
            files.calls,
            [
                "mkdir saves",
                "write saves/slot.ron.tmp",
                "write saves/slot.ron.tmp",
                "write saves/slot.ron.bak"
            ]
        );
        assert_eq!(
            files.files.keys().collect::<Vec<_>>(),
            ["saves/slot.ron", "saves/slot.ron.bak"]
        );
    }

    /// A storage holding `value`, `first` being the value before it.
    fn written_twice(first: &str, value: &str) -> FileStorage<MemoryFiles> {
        let mut storage = FileStorage::new(MemoryFiles::default());
        storage.write("slot", first).unwrap();
        storage.write("slot", value).unwrap();
        storage
    }

    #[test]
    fn a_second_write_keeps_the_first_value_as_the_backup() {
        let storage = written_twice("first", "second");
        assert_eq!(storage.files.files["slot.ron.bak"], b"first");
        assert_eq!(storage.read("slot").unwrap().as_deref(), Some("second"));
    }

    #[test]
    fn a_damaged_main_file_reads_its_backup() {
        let damages: [Option<&[u8]>; 4] =
            [None, Some(b""), Some(b"\0\0\0"), Some(b"\xff\xfe(gold")];
        for damage in damages {
            let mut storage = written_twice("first", "second");
            storage.files.files.remove("slot.ron");
            if let Some(bytes) = damage {
                storage
                    .files
                    .files
                    .insert("slot.ron".into(), bytes.to_vec());
            }
            assert_eq!(
                storage.read("slot").unwrap().as_deref(),
                Some("first"),
                "{damage:?}"
            );
        }
    }

    #[test]
    fn without_a_good_backup_a_main_file_reads_as_it_is() {
        for backup in [None, Some(&b""[..]), Some(b"\0\0"), Some(b"\xff")] {
            let mut storage = FileStorage::new(MemoryFiles::default());
            if let Some(bytes) = backup {
                storage
                    .files
                    .files
                    .insert("slot.ron.bak".into(), bytes.to_vec());
            }
            assert_eq!(storage.read("slot").unwrap(), None, "{backup:?}");
            storage.files.files.insert("slot.ron".into(), Vec::new());
            assert_eq!(storage.read("slot").unwrap().as_deref(), Some(""));
            storage
                .files
                .files
                .insert("slot.ron".into(), b"\0\0".to_vec());
            assert_eq!(storage.read("slot").unwrap().as_deref(), Some("\0\0"));
        }
    }

    /// Damage is never copied over the last good copy.
    #[test]
    fn a_damaged_main_file_does_not_become_the_backup() {
        for damage in [&b""[..], b"\0\0", b"\xff"] {
            let mut storage = written_twice("first", "second");
            storage
                .files
                .files
                .insert("slot.ron".into(), damage.to_vec());
            storage.write("slot", "third").unwrap();
            assert_eq!(storage.files.files["slot.ron.bak"], b"first");
        }
    }

    #[test]
    fn writing_an_empty_or_zero_value_removes_the_backup_and_reads_back() {
        for value in ["", "\0\0"] {
            let mut storage = written_twice("first", "second");
            storage.write("slot", value).unwrap();
            assert_eq!(
                storage.files.files.keys().collect::<Vec<_>>(),
                ["slot.ron"],
                "{value:?}"
            );
            assert_eq!(storage.read("slot").unwrap().as_deref(), Some(value));
        }
    }

    #[test]
    fn delete_removes_the_backup_too() {
        let mut storage = written_twice("first", "second");
        storage.delete("slot").unwrap();
        assert!(storage.files.files.is_empty());
        assert_eq!(storage.read("slot").unwrap(), None);
    }

    /// The first write of a start removes what a cut left, in its own
    /// folder only; later writes don't look.
    #[test]
    fn only_the_first_write_removes_the_tmp_files_a_cut_left() {
        let mut files = MemoryFiles::default();
        files.folders.insert("saves".into());
        for left in ["saves/left.ron.tmp", "left.ron.tmp"] {
            files.files.insert(left.into(), b"(gold: 2".to_vec());
        }
        let mut storage = FileStorage::in_folder(files, "saves");
        assert_eq!(storage.read("left").unwrap(), None);
        assert_eq!(storage.list().unwrap(), Vec::<String>::new());
        storage.write("settings", "()").unwrap();
        assert_eq!(
            storage.files.files.keys().collect::<Vec<_>>(),
            ["left.ron.tmp", "saves/settings.ron"]
        );
        storage
            .files
            .files
            .insert("saves/late.ron.tmp".into(), Vec::new());
        storage.write("settings", "(a)").unwrap();
        assert!(storage.files.files.contains_key("saves/late.ron.tmp"));
    }

    #[test]
    fn list_names_keys_and_good_lone_backups_but_never_bak_or_tmp_files() {
        let mut files = MemoryFiles::default();
        for (name, bytes) in [
            ("main.ron", &b"1"[..]),
            ("main.ron.bak", b"0"),
            ("lone-backup.ron.bak", b"1"),
            ("empty-backup.ron.bak", b""),
            ("Bad Name.ron.bak", b"1"),
            ("left.ron.tmp", b"1"),
        ] {
            files.files.insert(name.into(), bytes.to_vec());
        }
        let storage = FileStorage::new(files);
        assert_eq!(storage.list().unwrap(), ["lone-backup", "main"]);
    }

    /// On a first run the check makes its folder.
    #[test]
    fn the_check_makes_its_folder_on_a_first_run() {
        let mut files = MemoryFiles::default();
        check(&mut files);
        assert!(files.folders.contains(CHECK_FOLDER));
        assert!(files.files.is_empty());
    }

    #[test]
    fn a_file_that_is_not_text_is_an_error() {
        let mut storage = FileStorage::new(MemoryFiles::default());
        storage
            .files
            .files
            .insert("settings.ron".into(), vec![0xff]);
        assert!(matches!(
            storage.read("settings"),
            Err(StorageError::Backend(_))
        ));
    }
}
