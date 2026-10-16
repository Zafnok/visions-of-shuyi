//! The SDK's file calls behind [`Files`], on paths in the game's data
//! folder (the SDK's `Disk/Data/<bundle ID>/` in the Simulator).

use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::{CStr, c_char, c_uint, c_void};

use super::{
    pd_file_close, pd_file_error, pd_file_kind, pd_file_list, pd_file_mkdir,
    pd_file_open_read_data, pd_file_open_write, pd_file_read, pd_file_rename, pd_file_unlink,
    pd_file_write,
};
use crate::storage::{Entry, Files};
use crate::text::c_string;

/// The game's data folder, through the SDK.
#[derive(Debug, Clone, Copy, Default)]
pub struct SdkFiles;

/// How many bytes a read asks the SDK for at a time.
const CHUNK: usize = 4096;

/// The SDK's message for the last file call that failed.
fn last_error() -> String {
    let error = unsafe { pd_file_error() };
    if error.is_null() {
        return String::from("unknown file error");
    }
    // SAFETY: the SDK hands back a C string it keeps.
    String::from(unsafe { CStr::from_ptr(error) }.to_string_lossy())
}

/// `Ok` if the SDK call returned 0, else its message.
fn ok(result: core::ffi::c_int) -> Result<(), String> {
    if result == 0 {
        Ok(())
    } else {
        Err(last_error())
    }
}

/// Adds the name the SDK lists to the `Vec<String>` behind `names`.
extern "C" fn listed(name: *const c_char, names: *mut c_void) {
    // SAFETY: `list` passes its own `Vec<String>` and the SDK a C string,
    // both alive for the call.
    let names = unsafe { &mut *names.cast::<Vec<String>>() };
    let name = unsafe { CStr::from_ptr(name) };
    names.push(String::from(name.to_string_lossy()));
}

impl Files for SdkFiles {
    fn entry(&self, path: &str) -> Option<Entry> {
        let path = c_string(path);
        match unsafe { pd_file_kind(path.as_ptr().cast()) } {
            1 => Some(Entry::File),
            0 => Some(Entry::Folder),
            _ => None,
        }
    }

    fn read(&self, path: &str) -> Result<Vec<u8>, String> {
        let name = c_string(path);
        let file = unsafe { pd_file_open_read_data(name.as_ptr().cast()) };
        if file.is_null() {
            return Err(last_error());
        }
        let mut bytes = Vec::new();
        let mut chunk = [0_u8; CHUNK];
        let result = loop {
            let len = c_uint::try_from(chunk.len()).unwrap_or(c_uint::MAX);
            let read = unsafe { pd_file_read(file, chunk.as_mut_ptr().cast(), len) };
            match usize::try_from(read) {
                Ok(0) => break Ok(()),
                Ok(read) => bytes.extend_from_slice(&chunk[..read.min(CHUNK)]),
                Err(_) => break Err(last_error()),
            }
        };
        unsafe { pd_file_close(file) };
        result.map(|()| bytes)
    }

    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        let name = c_string(path);
        let file = unsafe { pd_file_open_write(name.as_ptr().cast()) };
        if file.is_null() {
            return Err(last_error());
        }
        let len = c_uint::try_from(bytes.len()).map_err(|_| String::from("file too big"))?;
        let written = unsafe { pd_file_write(file, bytes.as_ptr().cast(), len) };
        let all = usize::try_from(written).ok() == Some(bytes.len());
        let error = (!all).then(last_error);
        let closed = ok(unsafe { pd_file_close(file) });
        match error {
            Some(error) => Err(error),
            None => closed,
        }
    }

    fn rename(&mut self, from: &str, to: &str) -> Result<(), String> {
        let (from, to) = (c_string(from), c_string(to));
        ok(unsafe { pd_file_rename(from.as_ptr().cast(), to.as_ptr().cast()) })
    }

    fn unlink(&mut self, path: &str) -> Result<(), String> {
        let path = c_string(path);
        ok(unsafe { pd_file_unlink(path.as_ptr().cast()) })
    }

    fn mkdir(&mut self, path: &str) -> Result<(), String> {
        let path = c_string(path);
        ok(unsafe { pd_file_mkdir(path.as_ptr().cast()) })
    }

    fn list(&self, folder: &str) -> Result<Vec<String>, String> {
        let path = c_string(folder);
        let mut names: Vec<String> = Vec::new();
        let names_ptr: *mut Vec<String> = &raw mut names;
        ok(unsafe { pd_file_list(path.as_ptr().cast(), listed, names_ptr.cast()) })?;
        Ok(names)
    }
}
