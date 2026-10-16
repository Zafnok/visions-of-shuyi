//! The game's [`Storage`] on the Playdate (ticket 1033, ADR-0009): one
//! `<key>.ron` file per key in the game's data folder, as `trpg-app`'s
//! `FileStorage` keeps them in the PC's. Profiles and saves are kept
//! exactly as on the PC (ADR-0069): this only stores the text.
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
    /// Replaces file `path` with `bytes`, making it if it is not there.
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

/// A [`Storage`] of one file per key in a folder of the game's data
/// folder. A write lands in `<key>.ron.tmp` first and is then moved onto
/// `<key>.ron`, so a crash mid-write leaves the old file whole.
#[derive(Debug)]
pub struct FileStorage<F> {
    files: F,
    /// The folder, `""` or ending in `/`.
    folder: String,
}

impl<F: Files> FileStorage<F> {
    /// The keys in the data folder itself: the game's.
    pub fn new(files: F) -> Self {
        Self {
            files,
            folder: String::new(),
        }
    }

    /// The keys in `folder` (a name without `/`) of the data folder.
    pub fn in_folder(files: F, folder: &str) -> Self {
        Self {
            files,
            folder: format!("{folder}/"),
        }
    }

    fn path(&self, key: &str) -> String {
        format!("{}{key}{EXT}", self.folder)
    }

    fn tmp_path(&self, key: &str) -> String {
        format!("{}{key}{TMP}", self.folder)
    }
}

impl<F: Files + core::fmt::Debug> Storage for FileStorage<F> {
    fn read(&self, key: &str) -> Result<Option<String>, StorageError> {
        check_key(key)?;
        let path = self.path(key);
        match self.files.entry(&path) {
            None => Ok(None),
            Some(Entry::Folder) => Err(backend(&path, "is a folder")),
            Some(Entry::File) => {
                let bytes = self.files.read(&path).map_err(|e| backend(&path, &e))?;
                String::from_utf8(bytes)
                    .map(Some)
                    .map_err(|_| backend(&path, "is not UTF-8 text"))
            }
        }
    }

    fn write(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        check_key(key)?;
        let folder = self.folder.trim_end_matches('/');
        if !folder.is_empty() && self.files.entry(folder).is_none() {
            self.files.mkdir(folder).map_err(|e| backend(folder, &e))?;
        }
        let (tmp, path) = (self.tmp_path(key), self.path(key));
        self.files
            .write(&tmp, value.as_bytes())
            .map_err(|e| backend(&tmp, &e))?;
        if self.files.rename(&tmp, &path).is_ok() {
            return Ok(());
        }
        // A file system that will not move a file onto another: the old
        // one goes first.
        self.files.unlink(&path).map_err(|e| backend(&path, &e))?;
        self.files
            .rename(&tmp, &path)
            .map_err(|e| backend(&path, &e))
    }

    fn delete(&mut self, key: &str) -> Result<(), StorageError> {
        check_key(key)?;
        let path = self.path(key);
        if self.files.entry(&path).is_none() {
            return Ok(());
        }
        self.files.unlink(&path).map_err(|e| backend(&path, &e))
    }

    fn list(&self) -> Result<Vec<String>, StorageError> {
        let folder = self.folder.trim_end_matches('/');
        if !folder.is_empty() && self.files.entry(folder).is_none() {
            return Ok(Vec::new());
        }
        let names = self.files.list(folder).map_err(|e| backend(folder, &e))?;
        let key = |name: &String| {
            name.strip_suffix(EXT)
                .filter(|key| is_valid_key(key))
                .map(String::from)
        };
        Ok(names.iter().filter_map(key).collect())
    }
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
            self.calls.push(format!("write {path}"));
            self.files.insert(path.into(), bytes.to_vec());
            Ok(())
        }

        fn rename(&mut self, from: &str, to: &str) -> Result<(), String> {
            if self.no_replace && self.files.contains_key(to) {
                return Err("file exists".into());
            }
            let bytes = self.files.remove(from).ok_or("no such file")?;
            self.files.insert(to.into(), bytes);
            Ok(())
        }

        fn unlink(&mut self, path: &str) -> Result<(), String> {
            self.files
                .remove(path)
                .map(|_| ())
                .ok_or("no such file".into())
        }

        fn mkdir(&mut self, path: &str) -> Result<(), String> {
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
            ["write settings.ron.tmp", "write settings.ron.tmp"]
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
                "write saves/slot.ron.tmp"
            ]
        );
        assert_eq!(files.files.keys().collect::<Vec<_>>(), ["saves/slot.ron"]);
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
